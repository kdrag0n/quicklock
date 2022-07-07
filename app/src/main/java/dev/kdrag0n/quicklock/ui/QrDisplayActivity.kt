package dev.kdrag0n.quicklock.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.util.launchCollect
import dev.kdrag0n.quicklock.util.launchStarted
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@AndroidEntryPoint
class QrDisplayActivity : ComponentActivity() {
    private val model: QrDisplayViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        if (model.qrBitmap == null) {
            finish()
        }

        setContent {
            Surface(
                modifier = Modifier.fillMaxSize(),
                color = MaterialTheme.colorScheme.background,
            ) {
                QrDisplayScreen(model)
            }
        }

        launchStarted {
            model.pairFinishedFlow.launchCollect(this) {
                finish()
            }

            launch {
                while (true) {
                    delay(250)
                    model.tryFinishDelegatedPair()
                }
            }
        }
    }
}

@Composable
private fun QrDisplayScreen(model: QrDisplayViewModel) {
    model.qrBitmap?.let { bitmap ->
        Box(contentAlignment = Alignment.Center) {
            Image(
                bitmap = bitmap.asImageBitmap(),
                contentDescription = "QR code",
            )
        }
    }
}
