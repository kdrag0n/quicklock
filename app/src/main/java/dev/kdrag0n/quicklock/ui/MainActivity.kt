package dev.kdrag0n.quicklock.ui

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.MainViewModel
import dev.kdrag0n.quicklock.ui.theme.AppTheme
import dev.kdrag0n.quicklock.util.launchCollect
import dev.kdrag0n.quicklock.util.launchStarted

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
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
            Button(onClick = { model.pair() }) {
                Text("Pair")
            }

            Button(onClick = { model.unlock() }) {
                Text("Unlock")
            }

            Divider()


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
