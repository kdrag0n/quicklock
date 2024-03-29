package dev.kdrag0n.quicklock.ui

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.kdrag0n.quicklock.server.ApiClient
import dev.kdrag0n.quicklock.util.EventFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

private const val TAG = "QrScan"

@HiltViewModel
class QrScanViewModel @Inject constructor(
    private val client: ApiClient,
) : ViewModel() {
    val pairFinishedFlow = client.pairFinishedFlow
    val confirmFlow = EventFlow()
    val isDelegated = client.currentPairState?.challenge?.isInitial != true
    private var scanned = false

    fun onQrScanned(payload: String) = viewModelScope.launch {
        if (scanned) {
            return@launch
        }
        scanned = true

        Log.d(TAG, "onQrScanned: $payload")

        if (isDelegated) {
            // Delegated: got the challenge ID, show confirmation prompt
            client.fetchDelegation(payload)
            confirmFlow.emit()
        } else {
            // Initial: got the secret, try pairing
            client.finishInitialPair(payload)
        }
    }
}
