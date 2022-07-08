package dev.kdrag0n.quicklock.ui

import android.annotation.SuppressLint
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.google.android.material.datepicker.MaterialDatePicker
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.kdrag0n.quicklock.server.ApiClient
import dev.kdrag0n.quicklock.server.ApiService
import dev.kdrag0n.quicklock.server.Entity
import dev.kdrag0n.quicklock.util.EventFlow
import kotlinx.coroutines.launch
import java.security.Signature
import javax.inject.Inject

private const val DEFAULT_EXPIRY = 7 * 24 * 60 * 60 * 1000L // 1 week

@SuppressLint("MutableCollectionMutableState")
@HiltViewModel
class ConfirmDelegationViewModel @Inject constructor(
    private val client: ApiClient,
) : ViewModel() {
    var useExpiry by mutableStateOf(false)
    var expiresAt by mutableStateOf(MaterialDatePicker.todayInUtcMilliseconds() + DEFAULT_EXPIRY)

    var limitEntities by mutableStateOf(false)
    var selectEntities by mutableStateOf(false)
    var entityStates by mutableStateOf(mutableMapOf<Entity, Boolean>())

    val finishFlow = EventFlow()
    val pickExpiryDate = EventFlow()
    val publicKeyEmoji = client.getDelegateeKeyEmoji()

    init {
        viewModelScope.launch {
            entityStates = client.entities.value
                .associateWith { true }
                .toMutableMap()
        }
    }

    fun prepareSignature() = client.prepareDelegationSignature()

    fun confirm(sig: Signature) = viewModelScope.launch {
        client.signAndUploadDelegation(
            sig,
            expiresAt = if (useExpiry) expiresAt else Long.MAX_VALUE,
            allowedEntities = if (limitEntities) {
                entityStates
                    .filterValues { it }
                    .keys
                    .map { it.id }
            } else null,
        )
        finishFlow.emit()
    }
}
