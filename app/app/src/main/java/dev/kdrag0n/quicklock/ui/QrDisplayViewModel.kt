package dev.kdrag0n.quicklock.ui

import androidx.lifecycle.ViewModel
import com.google.zxing.BarcodeFormat
import com.journeyapps.barcodescanner.BarcodeEncoder
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.kdrag0n.quicklock.server.ApiClient
import javax.inject.Inject

@HiltViewModel
class QrDisplayViewModel @Inject constructor(
    private val client: ApiClient,
) : ViewModel() {
    val pairFinishedFlow = client.pairFinishedFlow

    val qrBitmap = client.currentPairState?.delegatedQrPayload?.let { payload ->
        val encoder = BarcodeEncoder()
        encoder.encodeBitmap(payload, BarcodeFormat.QR_CODE, 800, 800)
    }
    val publicKeyEmoji = client.getPublicKeyEmoji()

    suspend fun tryFinishDelegatedPair() = client.tryFinishDelegatedPair()
}
