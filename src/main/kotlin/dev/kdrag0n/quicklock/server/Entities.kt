package dev.kdrag0n.quicklock.server

import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.serialization.Serializable

@Serializable
data class Entity(
    val id: String,
    val name: String,
    val haEntity: String,
)

fun Application.entitiesModule() = routing {
    // TODO: auth
    get("/api/entity") {
        call.respond(Config.HA_ENTITIES.values.toList())
    }
}
