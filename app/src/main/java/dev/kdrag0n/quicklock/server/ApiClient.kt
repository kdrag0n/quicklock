package dev.kdrag0n.quicklock.server

import com.squareup.moshi.JsonAdapter
import com.squareup.moshi.Moshi
import dagger.Reusable
import dev.kdrag0n.quicklock.CryptoService
import javax.inject.Inject

private const val PAIRING_KEY = "pairing-key"

@Reusable
class ApiClient @Inject constructor(
    private val service: ApiService,
    private val crypto: CryptoService,
    moshi: Moshi,
) {
    private val adapter = moshi.adapter(UnlockRequest::class.java)

    suspend fun pair() {
        val request = PairRequest(
            publicKey = crypto.publicKeyEncoded,
            pairingKey = PAIRING_KEY,
        )
        service.pair(request)
    }

    suspend fun unlock() {
        val payload = UnlockRequest(
            publicKey = crypto.publicKeyEncoded,
            timestamp = System.currentTimeMillis(),
        )

        val payloadJson = adapter.toJson(payload)
        val request = WrappedUnlockRequest(
            payload = payloadJson,
            signature = crypto.signPayload(payloadJson),
        )

        service.unlock(request)
    }
}
