package dev.kdrag0n.quicklock.server

import com.google.zxing.BarcodeFormat
import com.google.zxing.client.j2se.MatrixToImageWriter
import com.google.zxing.qrcode.QRCodeWriter
import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.plugins.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.security.SecureRandom
import java.util.*

@Serializable
private data class Challenge(
    val id: String,
    val timestamp: Long,
    val isInitial: Boolean,
)

private val challenges = HashMap<String, Challenge>()
private val finishPayloads = HashMap<String, String>()
private val delegationSignatures = HashMap<String, DelegationSignature>()

@Serializable
private data class InitialPairQr(
    val secret: String,
)

@Serializable
private data class PairFinishPayload(
    val challengeId: String,
    val publicKey: String,
    val delegationKey: String,
    val mainAttestationChain: List<String>,
    val delegationAttestationChain: List<String>,
)

@Serializable
private data class InitialPairFinishRequest(
    val payload: String,
    val initialSecret: String,
)

@Serializable
private data class DelegationSignature(
    val device: String,
    val signature: String,
)

@Serializable
private data class DelegatedPairFinishRequest(
    val payload: String,
    val signature: DelegationSignature,
)

private fun generateSecret(): String {
    val random = SecureRandom()
    val secret = ByteArray(32)
    SecureRandom.getInstanceStrong().nextBytes(secret)
    return secret.toBase64Url()
}

@Volatile
private var initialPairingSecret: String? = null

fun Application.pairingModule() = routing {
    fun finishPair(req: PairFinishPayload, delegatedBy: String?) {
        val challenge = challenges[req.challengeId]!!

        try {
            require(challenge.isInitial == (delegatedBy == null))

            // Verify timestamp
            require(System.currentTimeMillis() - challenge.timestamp <= Config.TIME_GRACE_PERIOD)

            // Verify main attestation and certificate chain
            Attestation.verifyChain(req.mainAttestationChain, challenge.id)

            // Verify delegation attestation and certificate chain
            Attestation.verifyChain(req.delegationAttestationChain, challenge.id, isDelegation = true)

            Storage.addDevice(PairedDevice(
                publicKey = req.publicKey,
                delegationKey = req.delegationKey,
                expiresAt = Long.MAX_VALUE,
                delegatedBy = delegatedBy,
            ))
        } finally {
            // Drop challenge
            challenges -= challenge.id
            delegationSignatures -= challenge.id
            finishPayloads -= challenge.id
        }
    }

    /*
     * Initial
     */
    post("/api/pair/initial/start") {
        // Only for initial setup
        require(!Storage.hasPairedDevices())
        require(initialPairingSecret == null)

        // Generate secret
        val secret = generateSecret()
        initialPairingSecret = secret

        // Open QR page locally (on server)
        withContext(Dispatchers.IO) {
            println("url ${"open 'http://localhost:3002/api/pair/initial/start/qr?secret=$secret'"}")
            Runtime.getRuntime().exec("open 'http://localhost:3002/api/pair/initial/start/qr?secret=$secret'")
        }

        call.respond(HttpStatusCode.OK)
    }

    get("/api/pair/initial/start/qr") {
        require(call.request.origin.remoteHost == "localhost")

        val secret = initialPairingSecret!!
        require(call.request.queryParameters["secret"] == initialPairingSecret)

        val data = Json.encodeToString(InitialPairQr(
            secret = secret,
        ))
        val writer = QRCodeWriter()
        val matrix = writer.encode(data, BarcodeFormat.QR_CODE, 512, 512)
        call.respondOutputStream(ContentType.defaultForFileExtension("png")) {
            MatrixToImageWriter.writeToStream(matrix, "png", this)
        }
    }

    post("/api/pair/initial/finish") {
        val req = call.receive<InitialPairFinishRequest>()
        require(req.initialSecret == initialPairingSecret)
        initialPairingSecret = null

        val payload = Json.decodeFromString<PairFinishPayload>(req.payload)
        finishPair(payload, delegatedBy = null)

        call.respond(HttpStatusCode.OK)
    }

    /*
     * Delegation
     */
    get("/api/pair/delegated/{challengeId}/finish_payload") {
        val id = call.parameters["challengeId"]!!

        val payload = finishPayloads[id]
        if (payload == null) {
            call.respond(HttpStatusCode.NotFound)
        } else {
            call.respondText(payload)
        }
    }

    post("/api/pair/delegated/{challengeId}/finish_payload") {
        val id = call.parameters["challengeId"]!!
        require(id in challenges)
        require(id !in finishPayloads)
        require(id !in delegationSignatures)

        // Raw string, not decoded JSON, to avoid potential formatting differences
        // between Moshi and KotlinX serialization
        val payload = call.receiveChannel().readRemaining().readText()
        finishPayloads[id] = payload
        call.respond(HttpStatusCode.OK)
    }

    get("/api/pair/delegated/{challengeId}/signature") {
        val id = call.parameters["challengeId"]!!
        val signature = delegationSignatures[id]
        if (signature == null) {
            call.respond(HttpStatusCode.NotFound)
        } else {
            call.respond(signature)
        }
    }

    post("/api/pair/delegated/{challengeId}/signature") {
        val id = call.parameters["challengeId"]!!
        require(id in challenges)
        require(id in finishPayloads)
        require(id !in delegationSignatures)

        val signature = call.receive<DelegationSignature>()
        delegationSignatures[id] = signature
        call.respond(HttpStatusCode.OK)
    }

    post("/api/pair/delegated/finish") {
        val req = call.receive<DelegatedPairFinishRequest>()
        val (delegatedBy, signature) = req.signature
        val device = Storage.getDeviceByKey(delegatedBy)
        Crypto.verifySignature(req.payload, device.delegationKey, signature)

        val payload = Json.decodeFromString<PairFinishPayload>(req.payload)
        finishPair(payload, delegatedBy = delegatedBy)

        call.respond(HttpStatusCode.OK)
    }

    /*
     * Common
     */
    post("/api/pair/get_challenge") {
        // Generate a new challenge
        val challengeId = generateSecret()
        val challenge = Challenge(
            id = challengeId,
            timestamp = System.currentTimeMillis(),
            isInitial = !Storage.hasPairedDevices(),
        )
        challenges[challengeId] = challenge

        call.respond(challenge)
    }
}
