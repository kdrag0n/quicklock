package dev.kdrag0n.quicklock.ui

import android.os.Bundle
import android.os.PersistableBundle
import android.os.SystemClock
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.lifecycleScope
import com.google.android.gms.location.CurrentLocationRequest
import com.google.android.gms.location.Granularity
import com.google.android.gms.location.LocationServices
import com.google.android.gms.location.Priority
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.MainViewModel
import dev.kdrag0n.quicklock.ui.theme.AppTheme
import dev.kdrag0n.quicklock.util.launchCollect
import dev.kdrag0n.quicklock.util.launchStarted
import dev.kdrag0n.quicklock.util.setAppContent
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.take

@AndroidEntryPoint
class NfcUnlockActivity : AppCompatActivity() {
    private val model: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setAppContent {
            NfcUnlockScreen()
        }

        launchStarted {
            model.unlockFlow.launchCollect(this) {
                finish()
            }
        }

        val entityId = intent.data!!.getQueryParameter("entity")!!
        Log.d("NFU", "start")
        lifecycleScope.launchWhenStarted {
            val client = LocationServices.getFusedLocationProviderClient(this@NfcUnlockActivity)
            val req = CurrentLocationRequest.Builder().run {
                setGranularity(Granularity.GRANULARITY_FINE)
                setPriority(Priority.PRIORITY_BALANCED_POWER_ACCURACY)
                build()
            }

            client.getCurrentLocation(req, null).addOnSuccessListener { loc ->
                Log.d("NFU", "got loc $loc  (${loc.latitude}, ${loc.longitude})")
                model.unlock(entityId)
            }

        }
    }
}

@Composable
private fun NfcUnlockScreen() {
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            verticalArrangement = Arrangement.spacedBy(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text("Unlocking")
            CircularProgressIndicator()
        }
    }
}
