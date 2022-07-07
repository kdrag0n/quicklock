package dev.kdrag0n.quicklock.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.kdrag0n.quicklock.server.ApiClient
import dev.kdrag0n.quicklock.util.EventFlow
import kotlinx.coroutines.launch
import java.security.Signature
import javax.inject.Inject

@HiltViewModel
class ConfirmDelegationViewModel @Inject constructor(
    private val client: ApiClient,
) : ViewModel() {
    val finishFlow = EventFlow()
    val publicKeyEmoji = client.getDelegateeKeyEmoji()

    fun prepareSignature() = client.prepareDelegationSignature()

    fun confirm(sig: Signature) = viewModelScope.launch {
        client.signAndUploadDelegation(sig)
        finishFlow.emit()
    }
}
