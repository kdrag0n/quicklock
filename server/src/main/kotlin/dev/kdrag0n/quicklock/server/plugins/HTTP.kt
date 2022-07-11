package dev.kdrag0n.quicklock.server.plugins

import io.ktor.server.plugins.defaultheaders.*
import io.ktor.server.application.*

fun Application.configureHTTP() {
    install(DefaultHeaders)
}
