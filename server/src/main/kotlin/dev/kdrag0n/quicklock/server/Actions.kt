package dev.kdrag0n.quicklock.server

import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import org.slf4j.LoggerFactory
import kotlin.math.abs

@Serializable
data class UnlockChallenge(
    val id: String,
    val timestamp: Long,
    val entityId: String,
)

@Serializable
data class UnlockStartRequest(
    val entityId: String,
)

@Serializable
data class UnlockFinishRequest(
    // Challenge ID is in URL
    val publicKey: String,
    val blsSignature: String,
    val ecSignature: String,
)

private val logger = LoggerFactory.getLogger("Actions")

val unlockChallenges = HashMap<String, UnlockChallenge>()

fun Application.actionsModule() = routing {
    post("/api/unlock/start") {
        val (entityId) = call.receive<UnlockStartRequest>()
        val entity = Config.HA_ENTITIES[entityId]!!

        val challenge = UnlockChallenge(
            id = generateSecret(),
            timestamp = System.currentTimeMillis(),
            entityId = entity.id,
        )
        unlockChallenges[challenge.id] = challenge
        call.respond(challenge)
    }

    post("/api/unlock/{challengeId}/finish") {
        val id = call.parameters["challengeId"]!!
        val (publicKey, signature) = call.receive<UnlockFinishRequest>()

        val challenge = unlockChallenges[id]!!
        val (_, timestamp, entityId) = challenge
        Storage.getDeviceByKey(publicKey, entityId)
        Crypto.verifySignature(Json.encodeToString(challenge), publicKey, signature)

        // Verify timestamp
        require(abs(System.currentTimeMillis() - timestamp) <= Config.TIME_GRACE_PERIOD)

        // Unlock
        logger.info("Posting HA unlock")
        HomeAssistant.postLock(true, entityId)
        call.respond(EmptyObject)

        // Re-lock after delay
        launch {
            delay(Config.RELOCK_DELAY)
            logger.info("Posting HA lock")
            HomeAssistant.postLock(false, entityId)
        }
    }
}
