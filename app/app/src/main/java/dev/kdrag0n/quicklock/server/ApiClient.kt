package dev.kdrag0n.quicklock.server

import com.squareup.moshi.Moshi
import dev.kdrag0n.quicklock.CryptoService
import dev.kdrag0n.quicklock.NativeLib
import dev.kdrag0n.quicklock.toBase64
import dev.kdrag0n.quicklock.util.EventFlow
import dev.kdrag0n.quicklock.util.profileLog
import dev.kdrag0n.quicklock.util.toBase1024
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import okio.ByteString
import okio.ByteString.Companion.decodeBase64
import okio.ByteString.Companion.toByteString
import retrofit2.Response
import timber.log.Timber
import java.io.IOException
import java.security.Signature
import javax.inject.Inject
import javax.inject.Singleton

data class PairState(
    val challenge: Challenge,
    val finishPayloadData: String,
    val delegatedQrPayload: String,
)

data class DelegationState(
    val finishPayload: PairFinishPayload,
    val finishPayloadData: String,
)

@OptIn(DelicateCoroutinesApi::class)
@Singleton
class ApiClient @Inject constructor(
    private val service: NfcApiServiceWrapper,
    private val auditor: AuditService,
    private val crypto: CryptoService,
    private val moshi: Moshi,
) {
    private val initialQrAdapter = moshi.adapter(InitialPairQr::class.java)
    private val delegatedQrAdapter = moshi.adapter(DelegatedPairQr::class.java)
    private val pairFinishAdapter = moshi.adapter(PairFinishPayload::class.java)
    private val envelopeAdapter = moshi.adapter(RequestEnvelope::class.java)
    private val stampAdapter = moshi.adapter(AuditStamp::class.java)

    var currentPairState: PairState? = null
    var currentDelegationState: DelegationState? = null

    val pairFinishedFlow = EventFlow()

    val entities = MutableStateFlow<List<Entity>>(emptyList())

    init {
        GlobalScope.launch {
            updateEntities()
        }
    }

    suspend fun updateEntities() {
        try {
            entities.value = getEntities()
            Timber.d("New value = ${entities.value}")
        } catch (e: Exception) {
            Timber.e(e, "Entities update failed")
        }
    }

    private suspend fun getEntities() = service.getEntities().unwrap()

    suspend fun startPair(): Challenge {
        currentPairState?.let {
            return it.challenge
        }

        // Get a challenge
        val challenge = service.getPairingChallenge().unwrap()

        val (payload, data) = getFinishPayload(challenge)
        if (challenge.isInitial) {
            service.startInitialPair()
        } else {
            service.uploadDelegatedPairFinishPayload(challenge.id, payload)
        }

        currentPairState = PairState(
            challenge,
            data,
            delegatedQrPayload = delegatedQrAdapter.toJson(DelegatedPairQr(
                challenge = challenge.id,
            ))
        )
        return challenge
    }

    private suspend fun getFinishPayload(challenge: Challenge): Pair<PairFinishPayload, String> {
        val mainKey = crypto.generateKey(challenge.id)
        val delegationKey = crypto.generateKey(challenge.id, isDelegation = true)

        crypto.generateKeys()
        val registerResp = auditor.register(RegisterRequest(
            clientMacKey = crypto.macKeyBytes,
        )).unwrap()
        crypto.saveServerKey(registerResp.clientId, registerResp.serverPk)

        val req = PairFinishPayload(
            challengeId = challenge.id,
            publicKey = mainKey.encoded.toBase64(),
            delegationKey = delegationKey.encoded.toBase64(),
            encKey = crypto.encKeyBytes,
            auditPublicKey = crypto.auditServerPublicKeyBytes,
            mainAttestationChain = crypto.getAttestationChain(),
            delegationAttestationChain = crypto.getAttestationChain(CryptoService.DELEGATION_ALIAS),
        )
        return req to pairFinishAdapter.toJson(req)
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
            finishPayload = state.finishPayloadData,
            // HMAC using secret. Proves auth without being vulnerable to MITM.
            mac = state.finishPayloadData.toByteArray().toByteString()
                .hmacSha256(payload.secret.decodeBase64()!!)
                .base64(),
        ))
        currentPairState = null

        pairFinishedFlow.emit()
    }

    suspend fun checkDelegatedPairStatus(): Boolean {
        val state = currentPairState ?: return true
        // 404 = challenge gone, pairing done
        val resp = service.getDelegatedPairFinishPayload(state.challenge.id)

        val finished = resp.code() == 404
        return if (finished) {
            currentPairState = null
            true
        } else {
            false
        }
    }

    suspend fun fetchDelegation(payload: String) {
        if (currentDelegationState != null) {
            return
        }

        val qr = delegatedQrAdapter.fromJson(payload)!!
        val req = service.getDelegatedPairFinishPayload(qr.challenge).unwrap()
        val reqData = pairFinishAdapter.toJson(req)
        require(req.challengeId == qr.challenge)

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

        service.finishDelegatedPair(req.challengeId, sealAndSign(delegation, signer = sig))
        currentDelegationState = null
    }

    suspend fun unlock(entityId: String) {
        profileLog("unlock") {
            val challenge = profileLog("start") {
                service.startUnlock(UnlockStartRequest(entityId)).unwrap()
            }

            require(challenge.entityId == entityId)
            // just sign challenge id to minimize size
            val challengeId = challenge.id.decodeBase64()!!.toByteArray()
            require(challengeId.size == 32)

            val envelope = profileLog("sealAndSign") {
                sealAndSign(challengeId)
            }
            profileLog("finish") {
                service.finishUnlock(challenge.id, envelope)
            }
        }
    }

    private suspend inline fun <reified T> sealAndSign(
        request: T,
        signer: Signature? = null,
    ): SignedRequestEnvelope<T> = coroutineScope {
        // Seal unsigned envelope
        val requestData = if (request is ByteArray) {
            request
        } else {
            moshi.adapter(T::class.java).toJson(request).encodeToByteArray()
        }
        val envelopeJson = profileLog("nativeSeal") {
            NativeLib.envelopeSeal(crypto.encKeyBytes, requestData)
        }
        val envelopeData = envelopeJson.encodeToByteArray()
        val envelope = envelopeAdapter.fromJson(envelopeJson)!!

        val auditJob = async(Dispatchers.IO) {
            val clientSig1 = profileLog("signMac") {
                crypto.signMac(envelopeData)
            }
            val (stampData, auditSig) = profileLog("auditSign") {
                auditor.sign(SignRequest(
                    clientId = crypto.auditClientId,
                    envelope = envelopeData,
                    clientMac = clientSig1,
                )).unwrap()
            }
            val stamp = stampAdapter.fromJson(stampData.decodeToString())!!
            Pair(stamp, auditSig)
        }

        val localJob = async(Dispatchers.IO) {
            // Sign EC
            profileLog("signEc") {
                crypto.finishSignature(
                    sig = signer ?: crypto.prepareSignature(),
                    payload = envelopeData,
                )
            }
        }

        awaitAll(auditJob, localJob)
        val (stamp, auditSig) = auditJob.await()
        val ecSig = localJob.await()

        SignedRequestEnvelope(
            deviceId = crypto.deviceId,
            envelope = envelope,
            clientSignature = ecSig,
            auditStamp = stamp,
            auditSignature = auditSig,
        )
    }
}

class RequestException(message: String) : IOException(message)

private operator fun ByteString.plus(other: ByteString) = (toByteArray() + other.toByteArray()).toByteString()

private fun <T> Response<T>.unwrap(): T {
    val body = if (isSuccessful) body() else null
    return body ?: throw RequestException(errorBody()?.string() ?: "Unknown error")
}
