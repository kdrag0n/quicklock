package dev.kdrag0n.quicklock.server

import com.squareup.moshi.Moshi
import dev.kdrag0n.quicklock.CryptoService
import dev.kdrag0n.quicklock.toBase64
import dev.kdrag0n.quicklock.util.EventFlow
import dev.kdrag0n.quicklock.util.toBase1024
import okio.ByteString.Companion.decodeBase64
import java.io.IOException
import java.security.Signature
import javax.inject.Inject
import javax.inject.Singleton

data class PairState(
    val challenge: Challenge,
    val finishPayload: String,
    val delegatedQrPayload: String,
)

data class DelegationState(
    val finishPayload: PairFinishPayload,
    val finishPayloadData: String,
)

@Singleton
class ApiClient @Inject constructor(
    private val service: ApiService,
    private val crypto: CryptoService,
    moshi: Moshi,
) {
    private val initialQrAdapter = moshi.adapter(InitialPairQr::class.java)
    private val delegatedQrAdapter = moshi.adapter(DelegatedPairQr::class.java)
    private val pairFinishAdapter = moshi.adapter(PairFinishPayload::class.java)
    private val unlockAdapter = moshi.adapter(UnlockPayload::class.java)
    var currentPairState: PairState? = null
    var currentDelegationState: DelegationState? = null

    val pairFinishedFlow = EventFlow()

    suspend fun startPair(): Challenge {
        currentPairState?.let {
            return it.challenge
        }

        // Get a challenge
        val challenge = service.getChallenge().let {
            val body = if (it.isSuccessful) it.body() else null
            body ?: throw RequestException(it.errorBody()?.string() ?: "Unknown error")
        }

        val finishPayload = getFinishPayload(challenge)
        if (challenge.isInitial) {
            service.startInitialPair()
        } else {
            service.uploadDelegatedPairFinishPayload(challenge.id, finishPayload)
        }

        currentPairState = PairState(
            challenge,
            finishPayload,
            delegatedQrPayload = delegatedQrAdapter.toJson(DelegatedPairQr(
                challengeId = challenge.id,
            ))
        )
        return challenge
    }

    private fun getFinishPayload(challenge: Challenge): String {
        val mainKey = crypto.generateKey(challenge.id)
        val delegationKey = crypto.generateKey(challenge.id, isDelegation = true)
        val req = PairFinishPayload(
            challengeId = challenge.id,
            publicKey = mainKey.encoded.toBase64(),
            delegationKey = delegationKey.encoded.toBase64(),
            mainAttestationChain = crypto.getAttestationChain(),
            delegationAttestationChain = crypto.getAttestationChain(CryptoService.DELEGATION_ALIAS),
        )
        return pairFinishAdapter.toJson(req)
    }

    fun getPublicKeyEmoji() =
        publicKeyToEmoji(crypto.publicKeyEncoded)

    fun getDelegateeKeyEmoji() =
        currentDelegationState?.finishPayload?.let {
            publicKeyToEmoji(it.publicKey)
        } ?: ""

    // First 80 bits of the public key's SHA-256, as emoji
    private fun publicKeyToEmoji(data: String) = data.decodeBase64()!!
        .sha256().toByteArray()
        .toBase1024()

    suspend fun finishInitialPair(scannedData: String) {
        val payload = initialQrAdapter.fromJson(scannedData)!!
        val state = currentPairState ?: return
        service.finishInitialPair(InitialPairFinishRequest(
            payload = state.finishPayload,
            initialSecret = payload.secret,
        ))
        currentPairState = null

        pairFinishedFlow.emit()
    }

    suspend fun tryFinishDelegatedPair() {
        val state = currentPairState ?: return
        val resp = service.getDelegatedPairSignature(state.challenge.id)
        if (resp.isSuccessful) {
            finishDelegatedPair(resp.body() ?: return)
        }
    }

    private suspend fun finishDelegatedPair(signature: DelegationSignature) {
        val state = currentPairState ?: return
        service.finishDelegatedPair(DelegatedPairFinishRequest(
            payload = state.finishPayload,
            signature = signature,
        ))
        currentPairState = null

        pairFinishedFlow.emit()
    }

    suspend fun fetchDelegation(payload: String) {
        if (currentDelegationState != null) {
            return
        }

        val qr = delegatedQrAdapter.fromJson(payload)!!
        val reqData = service.getDelegatedPairFinishPayload(qr.challengeId).let {
            if (it.isSuccessful) it.body() else null
        } ?: throw RequestException("Failed to get delegated pair finish payload")
        val req = pairFinishAdapter.fromJson(reqData)!!
        require(req.challengeId == qr.challengeId)

        currentDelegationState = DelegationState(
            finishPayload = req,
            finishPayloadData = reqData,
        )
    }

    fun prepareDelegationSignature(): Signature {
        return crypto.prepareSignature(alias = CryptoService.DELEGATION_ALIAS)
    }

    suspend fun signAndUploadDelegation(sig: Signature) {
        val (req, reqData) = currentDelegationState ?: return

        val signature = DelegationSignature(
            device = crypto.publicKeyEncoded,
            signature = crypto.finishSignature(sig, reqData),
        )
        service.uploadDelegatedPairSignature(req.challengeId, signature)
        currentDelegationState = null
    }

    suspend fun unlock() {
        val payload = UnlockPayload(
            publicKey = crypto.publicKeyEncoded,
            timestamp = System.currentTimeMillis(),
        )

        val payloadJson = unlockAdapter.toJson(payload)
        val request = UnlockRequest(
            payload = payloadJson,
            signature = crypto.signPayload(payloadJson),
        )

        service.unlock(request)
    }
}

class RequestException(message: String) : IOException(message)
