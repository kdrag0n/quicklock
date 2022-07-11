package dev.kdrag0n.quicklock.util

import androidx.lifecycle.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.AbstractFlow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.FlowCollector
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.launch

fun LifecycleOwner.launchStarted(block: suspend CoroutineScope.() -> Unit) {
    lifecycleScope.launch {
        repeatOnLifecycle(Lifecycle.State.STARTED, block)
    }
}

fun <T> Flow<T>.launchCollect(scope: CoroutineScope, collector: FlowCollector<T>) =
    scope.launch {
        collect(collector)
    }

@OptIn(FlowPreview::class)
class EventFlow : AbstractFlow<Unit>() {
    private val flow = MutableSharedFlow<Unit>()

    override suspend fun collectSafely(collector: FlowCollector<Unit>) =
        flow.collect(collector)

    suspend fun emit() {
        flow.emit(Unit)
    }

    fun emit(viewModel: ViewModel) {
        viewModel.viewModelScope.launch {
            flow.emit(Unit)
        }
    }

    fun emit(lifecycleOwner: LifecycleOwner) {
        lifecycleOwner.lifecycleScope.launch {
            flow.emit(Unit)
        }
    }

    fun tryEmit() {
        flow.tryEmit(Unit)
    }
}
