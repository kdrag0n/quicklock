package dev.kdrag0n.quicklock.server

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import org.slf4j.LoggerFactory
import java.security.Signature
import kotlin.math.abs

@Serializable
private data class UnlockPayload(
    val entityId: String,
    val publicKey: String,
    val timestamp: Long,
)

@Serializable
private data class WrappedUnlockRequest(
    val payload: String,
    val signature: String,
)

private val logger = LoggerFactory.getLogger("Actions")

fun Application.actionsModule() = routing {
    post("/api/unlock") {
        val (payload, signature) = call.receive<WrappedUnlockRequest>()
        val (entityId, publicKey, timestamp) = Json.decodeFromString<UnlockPayload>(payload)
        Storage.getDeviceByKey(publicKey, entityId)

        Crypto.verifySignature(payload, publicKey, signature)

        // Verify timestamp
        require(abs(System.currentTimeMillis() - timestamp) <= Config.TIME_GRACE_PERIOD)

        // Unlock
        logger.info("Posting HA unlock")
        HomeAssistant.postLock(true, entityId)
        call.respond(HttpStatusCode.OK)

        // Re-lock after delay
        launch {
            delay(Config.RELOCK_DELAY)
            logger.info("Posting HA lock")
            HomeAssistant.postLock(false, entityId)
        }
    }
}