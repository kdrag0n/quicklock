package dev.kdrag0n.quicklock.ui

import android.os.Bundle
import android.os.Parcel
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import com.google.android.material.datepicker.CalendarConstraints
import com.google.android.material.datepicker.MaterialDatePicker
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.util.launchCollect
import dev.kdrag0n.quicklock.util.launchStarted
import dev.kdrag0n.quicklock.util.setAppContent

private const val TAG_DATE_PICKER = "datePicker"

@AndroidEntryPoint
class ConfirmDelegationActivity : AppCompatActivity() {
    private val model: ConfirmDelegationViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setAppContent {
            ConfirmDelegationScreen(
                model,
                onConfirm = {
                    requestAuth()
                }
            )
        }

        launchStarted {
            model.finishFlow.launchCollect(this) {
                finish()
            }

            model.pickExpiryDate.launchCollect(this) {
                val picker = MaterialDatePicker.Builder.datePicker().run {
                    setTitleText("Allow access until")
                    setCalendarConstraints(CalendarConstraints.Builder().run {
                        // Add 1 day, otherwise it might already be expired
                        val startTime = MaterialDatePicker.todayInUtcMilliseconds() + 24 * 60 * 60 * 1000
                        setStart(startTime)
                        setValidator(object : CalendarConstraints.DateValidator {
                            override fun isValid(date: Long) =
                                date >= startTime

                            override fun writeToParcel(dest: Parcel?, flags: Int) {}
                            override fun describeContents() = 0
                        })
                        build()
                    })
                    setSelection(model.expiresAt)
                    build()
                }
                picker.addOnDismissListener {
                    picker.selection?.let { model.expiresAt = it }
                }
                picker.show(supportFragmentManager, TAG_DATE_PICKER)
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

@OptIn(ExperimentalMaterial3Api::class)
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
                lineHeight = 96.sp,
                textAlign = TextAlign.Center,
            )

            Column {
                Column {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Checkbox(
                            checked = model.useExpiry,
                            onCheckedChange = { model.useExpiry = !model.useExpiry },
                        )
                        Text("Limit access time")
                    }

                    if (model.useExpiry) {
                        TextButton(onClick = { model.pickExpiryDate.emit(model) }) {
                            Text("Pick date")
                        }
                    }
                }

                Column {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Checkbox(
                            checked = model.limitEntities,
                            onCheckedChange = { model.limitEntities = !model.limitEntities },
                        )
                        Text("Limit locks")
                    }

                    if (model.limitEntities) {
                        TextButton(onClick = { model.selectEntities = true }) {
                            Text("Select locks")
                        }
                    }
                }
            }

            Button(onClick = onConfirm) {
                Text("Confirm")
            }
        }
    }

    if (model.selectEntities) {
        SelectEntitiesDialog(model)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SelectEntitiesDialog(model: ConfirmDelegationViewModel) {
    AlertDialog(
        onDismissRequest = { model.selectEntities = false },
        title = {
            Text("Allow locks")
        },
        text = {
            val states = model.entityStates

            Column {
                states.forEach { (entity, initialState) ->
                    var state by remember { mutableStateOf(initialState) }

                    Row(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Checkbox(
                            checked = state,
                            onCheckedChange = { state = !state; states[entity] = state },
                        )

                        Text(entity.name)
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = { model.selectEntities = false }) {
                Text("Done")
            }
        },
    )
}
