package dev.kdrag0n.quicklock.ui

import android.app.PendingIntent
import android.content.Intent
import android.content.IntentFilter
import android.nfc.NfcAdapter
import android.nfc.Tag
import android.nfc.tech.Ndef
import android.os.Bundle
import android.os.PersistableBundle
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
import androidx.compose.ui.unit.dp
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.ui.theme.AppTheme
import dev.kdrag0n.quicklock.util.collectAsLifecycleState
import dev.kdrag0n.quicklock.util.launchStarted
import dev.kdrag0n.quicklock.util.setAppContent

@AndroidEntryPoint
class NfcWriteActivity : AppCompatActivity() {
    private val model: NfcWriteViewModel by viewModels()
    private lateinit var nfcAdapter: NfcAdapter

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        nfcAdapter = NfcAdapter.getDefaultAdapter(this)!!

        setAppContent {
            NfcWriteScreen(model)
        }

        launchStarted {
            model.writeFlow.collect {
                finish()
            }
        }
    }

    override fun onResume() {
        super.onResume()

        val intent = Intent(this, javaClass)
        intent.addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP)
        // Mutable so system can add the Tag object
        val pi = PendingIntent.getActivity(this, 0, intent, PendingIntent.FLAG_MUTABLE)
        val intentFilters = arrayOf(
            IntentFilter(NfcAdapter.ACTION_NDEF_DISCOVERED).apply {
                addDataType("*/*")
            }
        )

        val techs = arrayOf(arrayOf(Ndef::class.java.name))
        nfcAdapter.enableForegroundDispatch(this, pi, intentFilters, techs)
    }

    override fun onPause() {
        super.onPause()
        nfcAdapter.disableForegroundDispatch(this)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        val tag = intent.getParcelableExtra<Tag>(NfcAdapter.EXTRA_TAG) ?: return
        model.writeTag(tag)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun NfcWriteScreen(
    model: NfcWriteViewModel,
) {
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            verticalArrangement = Arrangement.spacedBy(32.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            val entities by model.entities.collectAsLifecycleState()

            Column {
                Text("Select lock")

                entities.forEach { entity ->
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        RadioButton(
                            selected = entity == model.entity,
                            onClick = { model.entity = entity },
                        )
                        Text(entity.name)
                    }
                }
            }

            Column(
                verticalArrangement = Arrangement.spacedBy(16.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text("Tap NFC tag to write")
                CircularProgressIndicator()
            }
        }
    }
}
