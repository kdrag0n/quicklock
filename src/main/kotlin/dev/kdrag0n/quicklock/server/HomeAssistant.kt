package dev.kdrag0n.quicklock.server

import io.ktor.client.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.request.*

object HomeAssistant {
    private val client = HttpClient(OkHttp)

    suspend fun postLock(unlocked: Boolean) {
        val service = if (unlocked) "unlock" else "lock"

        client.post("http://171.66.3.236:8123/api/services/lock/$service") {
            header("Content-Type", "application/json")
            header("Authorization", "Bearer ${Config.HA_API_KEY}")
            setBody("""{"entity_id": "${Config.HA_ENTITY}"}""")
        }
    }
}
