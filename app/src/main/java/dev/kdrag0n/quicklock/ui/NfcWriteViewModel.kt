package dev.kdrag0n.quicklock.ui

import android.nfc.NdefMessage
import android.nfc.NdefRecord
import android.nfc.Tag
import android.nfc.tech.Ndef
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.kdrag0n.quicklock.util.EventFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

private const val UNLOCK_URI = "qlock://nfc/unlock"

@HiltViewModel
class NfcWriteViewModel @Inject constructor() : ViewModel() {
    val writeFlow = EventFlow()

    fun writeTag(tag: Tag) {
        val record = NdefRecord.createUri(UNLOCK_URI)
        val msg = NdefMessage(record)

        Ndef.get(tag)?.use {
            it.connect()
            it.writeNdefMessage(msg)
        }

        viewModelScope.launch {
            writeFlow.emit()
        }
    }
}
