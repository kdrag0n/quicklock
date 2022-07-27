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
import okio.ByteString.Companion.toByteString
import java.security.SecureRandom

@Serializable
data class PairingChallenge(
    val id: String,
    val timestamp: Long,
    val isInitial: Boolean,
) {
    fun validate() {
        require(System.currentTimeMillis() - timestamp <= Config.TIME_GRACE_PERIOD)
    }
}

val pairingChallenges = HashMap<String, PairingChallenge>()
val finishPayloads = HashMap<String, String>()

@Serializable
private data class InitialPairQr(
    val secret: String,
)

@Serializable
private data class PairFinishPayload(
    val challengeId: String,
    val publicKey: String,
    val delegationKey: String,
    val blsPublicKeys: List<String>?,
    val mainAttestationChain: List<String>,
    val delegationAttestationChain: List<String>,
)

@Serializable
data class InitialPairFinishRequest(
    val finishPayload: String,
    val mac: String,
)

@Serializable
data class Delegation(
    val finishPayload: String,
    val expiresAt: Long,
    val allowedEntities: List<String>?,
)

@Serializable
private data class SignedDelegation(
    val device: String,
    val delegation: String,
    val blsSignature: String,
    val ecSignature: String,
)

fun generateSecret(): String {
    val secret = ByteArray(32)
    SecureRandom.getInstanceStrong().nextBytes(secret)
    return secret.toBase64()
}

@Volatile
var initialPairingSecret: String? = null

fun Application.pairingModule() = routing {
    fun finishPair(
        req: PairFinishPayload,
        delegatedBy: String?,
        expiresAt: Long = Long.MAX_VALUE,
        allowedEntities: List<String>? = null,
    ) {
        val challenge = pairingChallenges[req.challengeId]!!

        try {
            require(challenge.isInitial == (delegatedBy == null))

            // Verify timestamp
            challenge.validate()

            // Verify main attestation and certificate chain
            Attestation.verifyChain(req.mainAttestationChain, challenge.id)

            // Verify delegation attestation and certificate chain
            Attestation.verifyChain(req.delegationAttestationChain, challenge.id, isDelegation = true)

            Storage.addDevice(PairedDevice(
                publicKey = req.publicKey,
                delegationKey = req.delegationKey,
                // Params
                expiresAt = expiresAt,
                delegatedBy = delegatedBy,
                allowedEntities = allowedEntities,
            ))
        } finally {
            // Drop challenge
            pairingChallenges -= challenge.id
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
            // URL-encode secret
            val url = URLBuilder().run {
                protocol = URLProtocol.HTTP
                host = "localhost"
                port = 3002
                path("api", "pair", "initial", "start", "qr")
                parameters.append("secret", secret)
                buildString()
            }

            println("secret = $secret")
            println("url = xdg-open '$url'")
            Runtime.getRuntime().exec("xdg-open '$url'")
        }

        call.respond(EmptyObject)
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

        // Verify HMAC
        val secret = initialPairingSecret!!
        val expectedMac = req.finishPayload.serializeToByteArray().toByteString()
            .hmacSha256(secret.decodeBase64().toByteString())
            .toByteArray()
        require(expectedMac cryptoEq req.mac.decodeBase64())
        initialPairingSecret = null

        val payload = Json.decodeFromString<PairFinishPayload>(req.finishPayload)
        finishPair(payload, delegatedBy = null)

        call.respond(EmptyObject)
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
        require(id in pairingChallenges)
        require(id !in finishPayloads)

        // Raw string, not decoded JSON, to avoid potential formatting differences
        // between Moshi and KotlinX serialization
        val payload = call.receiveChannel().readRemaining().readText()
        finishPayloads[id] = payload
        call.respond(EmptyObject)
    }

    post("/api/pair/delegated/{challengeId}/finish") {
        val req = call.receive<SignedDelegation>()
        val (delegatedBy, delegationData, signature) = req
        val device = Storage.getDeviceByKey(delegatedBy)
        Crypto.verifySignature(delegationData, device.delegationKey, signature)

        val (confirmedPayload, expiresAt, allowedEntities) = Json.decodeFromString<Delegation>(delegationData)

        // Prevent delegator from changing request
        val id = call.parameters["challengeId"]!!
        val origPayload = finishPayloads[id]!!
        require(origPayload == confirmedPayload)

        val payload = Json.decodeFromString<PairFinishPayload>(confirmedPayload)

        finishPair(
            payload,
            delegatedBy = delegatedBy,
            expiresAt = expiresAt,
            allowedEntities = allowedEntities,
        )

        call.respond(EmptyObject)
    }

    /*
     * Common
     */
    post("/api/pair/get_challenge") {
        // Generate a new challenge
        val challengeId = generateSecret()
        val challenge = PairingChallenge(
            id = challengeId,
            timestamp = System.currentTimeMillis(),
            isInitial = !Storage.hasPairedDevices(),
        )
        pairingChallenges[challengeId] = challenge

        call.respond(challenge)
    }
}

infix fun ByteArray.cryptoEq(other: ByteArray): Boolean {
    var n = 0
    for (i in indices) {
        n = n or (this[i].toInt() xor other[i].toInt())
    }
    return n == 0
}
