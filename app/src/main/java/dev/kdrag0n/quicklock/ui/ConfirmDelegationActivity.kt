package dev.kdrag0n.quicklock.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.compose.foundation.layout.*
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.util.launchCollect
import dev.kdrag0n.quicklock.util.launchStarted

@AndroidEntryPoint
class ConfirmDelegationActivity : AppCompatActivity() {
    private val model: ConfirmDelegationViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            Surface(
                modifier = Modifier.fillMaxSize(),
                color = MaterialTheme.colorScheme.background,
            ) {
                ConfirmDelegationScreen(
                    model,
                    onConfirm = {
                        requestAuth()
                    }
                )
            }
        }

        launchStarted {
            model.finishFlow.launchCollect(this) {
                finish()
            }
        }
    }

    private fun requestAuth() {
        val sig = model.prepareSignature()

        val executor = ContextCompat.getMainExecutor(this)
        val prompt = BiometricPrompt(this, executor, object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                super.onAuthenticationSucceeded(result)
                model.confirm(sig)
            }
        })

        val info = BiometricPrompt.PromptInfo.Builder().run {
            setTitle("Confirm: Add new device")
            setSubtitle("Use your fingerprint or PIN")
            setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG or
                    BiometricManager.Authenticators.DEVICE_CREDENTIAL)
            build()
        }

        prompt.authenticate(info, BiometricPrompt.CryptoObject(sig))
    }
}

@Composable
private fun ConfirmDelegationScreen(
    model: ConfirmDelegationViewModel,
    onConfirm: () -> Unit,
) {
    Box(contentAlignment = Alignment.Center) {
        Column(
            modifier = Modifier.padding(horizontal = 24.dp),
            verticalArrangement = Arrangement.spacedBy(32.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text("Make sure these emoji are the same on your new device.")

            Text(
                model.publicKeyEmoji,
                fontSize = 64.sp,
                textAlign = TextAlign.Center,
            )

            Button(onClick = onConfirm) {
                Text("Confirm")
            }
        }
    }
}
