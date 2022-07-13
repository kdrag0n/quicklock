package dev.kdrag0n.quicklock.server.plugins

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.plugins.cors.routing.*
import io.ktor.server.plugins.defaultheaders.*

fun Application.configureHTTP() {
    install(DefaultHeaders)

    install(CORS) {
        // TODO
        anyHost()
        allowHeader(HttpHeaders.ContentType)
    }
}
