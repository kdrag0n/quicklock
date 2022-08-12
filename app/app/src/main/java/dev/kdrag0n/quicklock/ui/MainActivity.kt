package dev.kdrag0n.quicklock.ui

import android.annotation.SuppressLint
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.lifecycle.lifecycleScope
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.MainViewModel
import dev.kdrag0n.quicklock.NativeLib
import dev.kdrag0n.quicklock.ui.theme.AppTheme
import dev.kdrag0n.quicklock.util.collectAsLifecycleState
import dev.kdrag0n.quicklock.util.launchCollect
import dev.kdrag0n.quicklock.util.launchStarted
import dev.kdrag0n.quicklock.util.setAppContent
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@AndroidEntryPoint
class MainActivity : AppCompatActivity() {
    private val model: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setAppContent {
            MainScreen(model)
        }

        launchStarted {
            model.scanFlow.launchCollect(this) {
                startActivity(Intent(this@MainActivity, QrScanActivity::class.java))
            }

            model.displayFlow.launchCollect(this) {
                startActivity(Intent(this@MainActivity, QrDisplayActivity::class.java))
            }
        }

        NativeLib.startServer()
    }

    override fun onResume() {
        super.onResume()
        model.updateEntities()
    }
}

@SuppressLint("UnsafeOptInUsageError")
@Composable
private fun MainScreen(
    model: MainViewModel,
) {
    val scope = rememberCoroutineScope()

    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            verticalArrangement = Arrangement.spacedBy(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            val entities by model.entities.collectAsLifecycleState()

            entities.forEach { entity ->
                Button(onClick = { model.unlock(entity.id) }) {
                    Text("Unlock ‘${entity.name}’")
                }
            }
            if (entities.isEmpty()) {
                CircularProgressIndicator()
            }

            Divider()

            FilledTonalButton(onClick = { model.pair() }) {
                Text("Pair")
            }

            val context = LocalContext.current
            FilledTonalButton(
                onClick = {
                    context.startActivity(Intent(context, QrScanActivity::class.java))
                }
            ) {
                Text("Add device")
            }

            Divider()

            TextButton(
                onClick = {
                    context.startActivity(Intent(context, NfcWriteActivity::class.java))
                },
            ) {
                Text("Write tag")
            }

            TextButton(
                onClick = {
                    context.startActivity(Intent(context, NfcInteractiveActivity::class.java))
                },
            ) {
                Text("Interactive")
            }

            Divider()

            TextButton(
                onClick = {
                    scope.launch {
                        NativeLib.startServer()
                        delay(500)
                        model.updateEntities()
                    }
                },
            ) {
                Text("Start server")
            }
        }
    }
}
