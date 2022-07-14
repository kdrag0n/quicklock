package dev.kdrag0n.quicklock.server

import com.webauthn4j.authenticator.AuthenticatorImpl
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.io.File
import java.io.FileNotFoundException

@Serializable
data class PairedDevice(
    // For normal actions (unlock)
    val publicKey: String,
    // For adding a new device. This one requires protected confirmation, verified by attestation
    val delegationKey: String,
    // When this device's access expires (for temporary access)
    val expiresAt: Long,
    // Device that authorized this. Null for initial setup
    val delegatedBy: String?,
    // Null for all
    val allowedEntities: List<String>?,

    // WebAuthn Authenticator object
    val serializedAuthenticator: String? = null,
) {
    fun loadAuthenticator() = serializedAuthenticator?.let {
        it.decodeBase64().decodeSerializable() as AuthenticatorImpl
    }
}

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
            .writeText(Json.encodeToString(devices.values.toList()))
    }

    fun addDevice(device: PairedDevice) {
        if (device.publicKey in devices) {
            return
        }

        devices[device.publicKey] = device
        writeDevices()
    }

    fun updateDevice(device: PairedDevice) {
        require(device.publicKey in devices)
        devices[device.publicKey] = device
        writeDevices()
    }

    fun getDeviceByKey(publicKey: String, entityId: String? = null): PairedDevice {
        val device = devices[publicKey]
        requireNotNull(device)
        require(device.expiresAt >= System.currentTimeMillis())
        entityId?.let { require(device.allowedEntities == null || it in device.allowedEntities) }

        return device
    }

    fun hasPairedDevices() = devices.isNotEmpty()
}
