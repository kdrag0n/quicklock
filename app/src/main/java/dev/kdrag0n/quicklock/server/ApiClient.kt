package dev.kdrag0n.quicklock.server

import com.squareup.moshi.Moshi
import dagger.Reusable
import dev.kdrag0n.quicklock.CryptoService
import java.io.IOException
import javax.inject.Inject

private const val PAIRING_SECRET = "sjcCugNwAqr3t8ZnmR37sHi8ZRr5GkAF"

@Reusable
class ApiClient @Inject constructor(
    private val service: ApiService,
    private val crypto: CryptoService,
    moshi: Moshi,
) {
    private val adapter = moshi.adapter(UnlockRequest::class.java)

    suspend fun pair() {
        // Start: get a challenge
        val startRequest = PairStartRequest(
            pairingSecret = PAIRING_SECRET,
        )
        val challenge = service.startPair(startRequest).let {
            val body = if (it.isSuccessful) it.body() else null
            body ?: throw RequestException(it.errorBody()?.string() ?: "Unknown error")
        }

        // Generate keypair with challenge and finish
        crypto.generateKey(challenge.id)
        val finishRequest = PairFinishRequest(
            challengeId = challenge.id,
            publicKey = crypto.publicKeyEncoded,
            attestationChain = crypto.getAttestationChain(),
        )
        service.finishPair(finishRequest)
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

class RequestException(message: String) : IOException(message)
