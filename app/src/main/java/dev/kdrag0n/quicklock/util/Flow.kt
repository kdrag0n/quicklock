package dev.kdrag0n.quicklock.util

import android.annotation.SuppressLint
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.flowWithLifecycle
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext

@SuppressLint("StateFlowValueCalledInComposition")
@Composable
fun <T> StateFlow<T>.collectAsLifecycleState(
    context: CoroutineContext = EmptyCoroutineContext,
) = collectAsLifecycleState(
    context = context,
    initial = value,
)


@Composable
fun <T> Flow<T>.collectAsLifecycleState(
    context: CoroutineContext = EmptyCoroutineContext,
    initial: T,
): State<T> {
    val lifecycleOwner = LocalLifecycleOwner.current
    val flow = remember(this, lifecycleOwner) {
        flowWithLifecycle(lifecycleOwner.lifecycle, Lifecycle.State.STARTED)
    }

    return flow.collectAsState(initial, context)
}
