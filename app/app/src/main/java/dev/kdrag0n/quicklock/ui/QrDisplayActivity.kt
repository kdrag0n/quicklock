package dev.kdrag0n.quicklock.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.util.launchCollect
import dev.kdrag0n.quicklock.util.launchStarted
import dev.kdrag0n.quicklock.util.setAppContent
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@AndroidEntryPoint
class QrDisplayActivity : AppCompatActivity() {
    private val model: QrDisplayViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        if (model.qrBitmap == null) {
            finish()
        }

        setAppContent {
            QrDisplayScreen(model)
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
            Column(
                modifier = Modifier.padding(horizontal = 24.dp),
                verticalArrangement = Arrangement.spacedBy(32.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Image(
                    modifier = Modifier.fillMaxWidth(),
                    bitmap = bitmap.asImageBitmap(),
                    contentDescription = "QR code",
                )

                Text("Make sure these emoji are the same on your old device.")

                Text(
                    model.publicKeyEmoji,
                    fontSize = 64.sp,
                    lineHeight = 96.sp,
                    textAlign = TextAlign.Center,
                )
            }
        }
    }
}
