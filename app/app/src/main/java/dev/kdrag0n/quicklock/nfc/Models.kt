package dev.kdrag0n.quicklock.nfc

import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class NfcUnlockRequest(
    val entityId: String,
)
