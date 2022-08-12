package dev.kdrag0n.quicklock.ui

import android.nfc.NdefMessage
import android.nfc.NdefRecord
import android.nfc.Tag
import android.nfc.tech.IsoDep
import android.nfc.tech.Ndef
import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.squareup.moshi.Moshi
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.kdrag0n.quicklock.server.*
import dev.kdrag0n.quicklock.util.EventFlow
import dev.kdrag0n.quicklock.util.launchCollect
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.launch
import okio.ByteString.Companion.toByteString
import timber.log.Timber
import java.nio.ByteBuffer
import javax.inject.Inject

@HiltViewModel
class NfcInteractiveViewModel @Inject constructor(
    client: ApiClient,
    private val moshi: Moshi,
    private val wrapper: NfcApiServiceWrapper,
) : ViewModel() {
    val entities = client.entities
    var entity by mutableStateOf<Entity?>(null)

    val writeFlow = EventFlow()

    init {
        entities
            .filter { it.isNotEmpty() }
            .take(1)
            .launchCollect(viewModelScope) {
                entity = it[0]
            }
    }

    fun writeTag(tag: Tag) {
        val entityId = "front"

        IsoDep.get(tag)?.also {
            it.connect()
            Timber.d("Tag $it ext=${it.isExtendedLengthApduSupported} max=${it.maxTransceiveLength}")

            // select aid
            val select = apdu(0, 0xa4.toByte(), 0x04, 0, byteArrayOf(
                0xF0.toByte(), 0x01, 0x02, 0x03, 0x04, 0x05, 0x06
            ))

            Timber.d("Select req: ${select.toByteString().hex()}")
            val selectResp = it.transceive(select)
            Timber.d("Select response: ${selectResp.toByteString().hex()}")

            val service = NfcApiService(it, moshi)
            wrapper.serviceFlow.value = service
        }

        viewModelScope.launch {
            writeFlow.emit()
        }
    }
}

fun apdu(cla: Byte, inst: Byte, p1: Byte, p2: Byte, payload: ByteArray): ByteArray = ByteBuffer.allocate(5 + payload.size).run {
    put(cla)
    put(inst)
    put(p1)
    put(p2)
    put(payload.size.toByte())
    put(payload)
    array()
}

fun apduExt(cla: Byte, inst: Byte, p1: Byte, p2: Byte, payload: ByteArray): ByteArray = ByteBuffer.allocate(7 + payload.size).run {
    put(cla)
    put(inst)
    put(p1)
    put(p2)
    put(0x00.toByte())
    putShort(payload.size.toShort())
    put(payload)
    array()
}
