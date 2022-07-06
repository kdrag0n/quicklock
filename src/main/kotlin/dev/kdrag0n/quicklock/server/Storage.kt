package dev.kdrag0n.quicklock.server

import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.io.File
import java.io.FileInputStream
import java.io.FileNotFoundException

@Serializable
data class PairedDevice(
    val publicKey: String,
)

object Storage {
    private val devices = loadDevices().toMutableMap()

    private fun loadDevices() = try {
        val json = File("data/devices.json").readText()
        val list = Json.decodeFromString<List<PairedDevice>>(json)
        list.associateBy { it.publicKey }
    } catch (e: FileNotFoundException) {
        emptyMap()
    }

    private fun writeDevices() {
        File("data").mkdirs()
        File("data/devices.json")
            .writeText(Json.encodeToString(devices.keys.toList()))
    }

    fun addDevice(device: PairedDevice) {
        if (device.publicKey in devices) {
            return
        }

        devices[device.publicKey] = device
        writeDevices()
    }

    fun requirePaired(publicKey: String) =
        require(publicKey in devices)
}
