package dev.kdrag0n.quicklock.server

import io.ktor.server.application.*
import dev.kdrag0n.quicklock.server.plugins.*

fun main(args: Array<String>): Unit =
    io.ktor.server.netty.EngineMain.main(args)

@Suppress("unused") // application.conf references the main function. This annotation prevents the IDE from marking it as unused.
fun Application.module() {
    configureRouting()
    configureHTTP()
    configureMonitoring()
    configureSerialization()

    entitiesModule()
    pairingModule()
    actionsModule()
}
