package dev.kdrag0n.quicklock

import android.annotation.SuppressLint
import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import dev.kdrag0n.quicklock.server.ApiClient
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val client: ApiClient,
) : ViewModel() {
    fun pair() {
        viewModelScope.launch {
            client.pair()
        }
    }

    fun unlock() {
        viewModelScope.launch {
            client.unlock()
        }
    }
}
