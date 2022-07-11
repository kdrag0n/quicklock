package dev.kdrag0n.quicklock.ui

import android.nfc.NdefMessage
import android.nfc.NdefRecord
import android.nfc.Tag
import android.nfc.tech.Ndef
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.kdrag0n.quicklock.server.ApiClient
import dev.kdrag0n.quicklock.server.Entity
import dev.kdrag0n.quicklock.util.EventFlow
import dev.kdrag0n.quicklock.util.launchCollect
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class NfcWriteViewModel @Inject constructor(
    client: ApiClient,
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
        val entityId = entity?.id ?: return

        val record = NdefRecord.createUri("qlock://nfc/unlock?entity=$entityId")
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
