package dev.kdrag0n.quicklock.server

import com.squareup.moshi.Moshi
import dev.kdrag0n.quicklock.CryptoService
import dev.kdrag0n.quicklock.decodeBase64Url
import dev.kdrag0n.quicklock.toBase64
import dev.kdrag0n.quicklock.util.EventFlow
import dev.kdrag0n.quicklock.util.toBase1024
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import okio.ByteString
import okio.ByteString.Companion.decodeBase64
import okio.ByteString.Companion.toByteString
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

@OptIn(DelicateCoroutinesApi::class)
@Singleton
class ApiClient @Inject constructor(
    private val service: ApiService,
    private val crypto: CryptoService,
    moshi: Moshi,
) {
    private val delegationAdapter = moshi.adapter(Delegation::class.java)
    private val initialQrAdapter = moshi.adapter(InitialPairQr::class.java)
    private val delegatedQrAdapter = moshi.adapter(DelegatedPairQr::class.java)
    private val pairFinishAdapter = moshi.adapter(PairFinishPayload::class.java)
    private val unlockAdapter = moshi.adapter(UnlockPayload::class.java)
    var currentPairState: PairState? = null
    var currentDelegationState: DelegationState? = null

    val pairFinishedFlow = EventFlow()

    val entities = MutableStateFlow<List<Entity>>(emptyList())

    init {
        GlobalScope.launch {
            entities.value = getEntities()
        }
    }

    private suspend fun getEntities() = service.getEntities().let {
        val body = if (it.isSuccessful) it.body() else null
        body ?: throw RequestException(it.errorBody()?.string() ?: "Unknown error")
    }

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
        publicKeysToEmoji(crypto.publicKeyEncoded, crypto.delegationKeyEncoded)

    fun getDelegateeKeyEmoji() =
        currentDelegationState?.finishPayload?.let {
            publicKeysToEmoji(it.publicKey, it.delegationKey)
        } ?: ""

    // First 80 bits of the public key's SHA-256, as emoji
    private fun publicKeysToEmoji(mainPk: String, delegationPk: String) =
        (mainPk.decodeBase64()!! + delegationPk.decodeBase64()!!)
        .sha256().toByteArray()
        .toBase1024()

    suspend fun finishInitialPair(scannedData: String) {
        val payload = initialQrAdapter.fromJson(scannedData)!!
        val state = currentPairState ?: return
        service.finishInitialPair(InitialPairFinishRequest(
            payload = state.finishPayload,
            // HMAC using secret. Proves auth without being vulnerable to MITM.
            hmac = state.finishPayload.toByteArray().toByteString()
                .hmacSha256(payload.secret.decodeBase64Url().toByteString())
                .base64(),
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

    private suspend fun finishDelegatedPair(signature: SignedDelegation) {
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

    suspend fun signAndUploadDelegation(sig: Signature, expiresAt: Long, allowedEntities: List<String>?) {
        val (req, reqData) = currentDelegationState ?: return

        val delegation = Delegation(
            finishPayload = reqData,
            expiresAt = expiresAt,
            allowedEntities = allowedEntities,
        )
        val delegationData = delegationAdapter.toJson(delegation)

        val signature = SignedDelegation(
            device = crypto.publicKeyEncoded,
            delegation = delegationData,
            signature = crypto.finishSignature(sig, delegationData),
        )
        service.uploadDelegatedPairSignature(req.challengeId, signature)
        currentDelegationState = null
    }

    suspend fun unlock(entityId: String) {
        val payload = UnlockPayload(
            entityId = entityId,
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

private operator fun ByteString.plus(other: ByteString) = (toByteArray() + other.toByteArray()).toByteString()
