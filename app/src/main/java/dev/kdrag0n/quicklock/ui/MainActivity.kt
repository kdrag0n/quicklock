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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.MainViewModel
import dev.kdrag0n.quicklock.ui.theme.AppTheme
import dev.kdrag0n.quicklock.util.collectAsLifecycleState
import dev.kdrag0n.quicklock.util.launchCollect
import dev.kdrag0n.quicklock.util.launchStarted

@AndroidEntryPoint
class MainActivity : AppCompatActivity() {
    private val model: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            AppTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background,
                ) {
                    MainScreen(model)
                }
            }
        }

        launchStarted {
            model.scanFlow.launchCollect(this) {
                startActivity(Intent(this@MainActivity, QrScanActivity::class.java))
            }

            model.displayFlow.launchCollect(this) {
                startActivity(Intent(this@MainActivity, QrDisplayActivity::class.java))
            }
        }
    }
}

@SuppressLint("UnsafeOptInUsageError")
@Composable
private fun MainScreen(
    model: MainViewModel,
) {
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
        }
    }
}
