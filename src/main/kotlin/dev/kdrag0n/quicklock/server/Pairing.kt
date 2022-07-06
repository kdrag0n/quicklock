package dev.kdrag0n.quicklock.server

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.serialization.Serializable
import java.util.*
import kotlin.collections.HashMap

@Serializable
private data class Challenge(
    val id: String,
    val timestamp: Long,
)

private val challenges = HashMap<String, Challenge>()

@Serializable
private data class PairStartRequest(
    val pairingSecret: String,
)

@Serializable
private data class PairFinishRequest(
    val challengeId: String,
    val publicKey: String,
    val attestationChain: List<String>,
)

fun Application.pairingModule() = routing {
    post("/api/pair/start") {
        val (pairingSecret) = call.receive<PairStartRequest>()
        // TODO: fix side channel
        if (pairingSecret != Config.PAIRING_SECRET) {
            return@post call.respondText("Invalid pairing secret", status = HttpStatusCode.Unauthorized)
        }

        // Generate a new challenge
        val challengeId = UUID.randomUUID().toString()
        val challenge = Challenge(
            id = challengeId,
            timestamp = System.currentTimeMillis()
        )
        challenges[challengeId] = challenge

        call.respond(challenge)
    }

    post("/api/pair/finish") {
        val (challengeId, publicKey, attestationChain) = call.receive<PairFinishRequest>()
        val challenge = challenges[challengeId]
            ?: return@post call.respondText("Invalid challenge", status = HttpStatusCode.BadRequest)

        try {
            // Verify timestamp
            require(System.currentTimeMillis() - challenge.timestamp <= Config.TIME_GRACE_PERIOD)

            // Verify attestation certificate chain
            val certs = attestationChain.map { Crypto.parseCert(it) }
            Crypto.verifyCertChain(certs)

            Storage.addDevice(PairedDevice(
                publicKey = publicKey,
            ))
        } finally {
            // Drop challenge
            challenges.remove(challengeId)
        }

        call.respond(HttpStatusCode.OK)
    }
}
