package dev.kdrag0n.quicklock

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.kdrag0n.quicklock.server.ApiClient
import dev.kdrag0n.quicklock.server.Entity
import dev.kdrag0n.quicklock.util.EventFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val client: ApiClient,
) : ViewModel() {
    val entities = client.entities

    val unlockFlow = EventFlow()
    val scanFlow = EventFlow()
    val displayFlow = EventFlow()

    fun pair() {
        viewModelScope.launch {
            val challenge = client.startPair()
            if (challenge.isInitial) {
                // Initial: scan QR to get the secret
                scanFlow.emit()
            } else {
                // Delegated: get signature from other device
                displayFlow.emit()
            }
        }
    }

    fun unlock(entityId: String) {
        viewModelScope.launch {
            client.unlock(entityId)
            unlockFlow.emit()
        }
    }

    fun updateEntities() {
        viewModelScope.launch {
            client.updateEntities()
        }
    }
}
