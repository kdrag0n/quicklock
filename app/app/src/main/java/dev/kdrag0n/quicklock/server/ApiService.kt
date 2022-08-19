package dev.kdrag0n.quicklock.server

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass
import com.squareup.moshi.Moshi
import dev.kdrag0n.quicklock.NativeLib
import okio.Buffer
import okio.BufferedSource
import okio.ByteString.Companion.decodeBase64
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path
import timber.log.Timber
import java.nio.ByteBuffer

interface ApiService {
    @GET("/api/entity")
    suspend fun getEntities(): Response<List<Entity>>

    @POST("/api/pair/initial/start")
    suspend fun startInitialPair(): Response<Unit>

    @POST("/api/pair/initial/finish")
    suspend fun finishInitialPair(@Body request: InitialPairFinishRequest): Response<Unit>

    @GET("/api/pair/delegated/{challengeId}/finish_payload")
    suspend fun getDelegatedPairFinishPayload(@Path("challengeId") challengeId: String): Response<PairFinishPayload>

    @POST("/api/pair/delegated/{challengeId}/finish_payload")
    suspend fun uploadDelegatedPairFinishPayload(
        @Path("challengeId") challengeId: String,
        @Body payload: PairFinishPayload,
    ): Response<Unit>

    @POST("/api/pair/delegated/{challengeId}/finish")
    suspend fun finishDelegatedPair(
        @Path("challengeId") challengeId: String,
        @Body request: SignedRequestEnvelope<Delegation>,
    ): Response<Unit>

    @POST("/api/pair/get_challenge")
    suspend fun getPairingChallenge(): Response<Challenge>

    @POST("/api/unlock/start")
    suspend fun startUnlock(@Body request: UnlockStartRequest): Response<UnlockChallenge>

    @POST("/api/unlock/{challengeId}/finish")
    suspend fun finishUnlock(
        @Path("challengeId") challengeId: String,
        @Body request: SignedRequestEnvelope<ByteArray>,
    ): Response<Unit>
}

/*
 * Audit/generic
 */
@JsonClass(generateAdapter = true)
data class RequestEnvelope(
    @Json(name = "p")
    val encPayload: ByteArray,
    @Json(name = "n")
    val encNonce: ByteArray,
)

@JsonClass(generateAdapter = true)
data class AuditStamp(
    val envelopeHash: ByteArray,
    val clientIp: String,
    val timestamp: Long,
)

@JsonClass(generateAdapter = true)
data class SignedRequestEnvelope<T>(
    val deviceId: ByteArray,
    val envelope: RequestEnvelope,
    val clientSignature: ByteArray,
    val auditStamp: AuditStamp,
    val auditSignature: ByteArray,
) {
    fun toByteArray(knownData: Boolean = false) = Buffer().run {
        write(deviceId) // 12
        if (!knownData) write(envelope.encPayload) // 48
        write(envelope.encNonce) // 24
        writeByte(clientSignature.size) // 1
        write(clientSignature) // clientSignature.size
        write(auditStamp.clientIp.toByteArray()) // 9 (TODO length)
        writeLong(auditStamp.timestamp) // 8
        write(auditSignature) // 64

        val data = snapshot().toByteArray()
        Timber.d("signed envelope size ${data.size}")
        data
    }

    companion object {
        inline fun <reified T> fromByteArray(data: ByteArray, moshi: Moshi, knownData: Boolean = false): SignedRequestEnvelope<T> {
            val buffer = Buffer().write(data).peek()
            Timber.d("total d = ${data.size}")

            val deviceId = buffer.readByteArray(12)
            val encPayload = if (!knownData) buffer.readByteArray(48) else ByteArray(0)
            val encNonce = buffer.readByteArray(24)
            val clientSigSize = buffer.readByte()
            val clientSig = buffer.readByteArray(clientSigSize.toLong())
            val clientIp = buffer.readUtf8(9)
            val timestamp = buffer.readLong()
            val auditSig = buffer.readByteArray(64)

            // Recompute envelope hash
            val envelope = RequestEnvelope(
                encPayload = encPayload,
                encNonce = encNonce,
            )
            val envelopeJson = moshi.adapter(RequestEnvelope::class.java).toJson(envelope).encodeToByteArray()
            val envelopeHash = NativeLib.hash(envelopeJson)

            return SignedRequestEnvelope(
                deviceId = deviceId,
                envelope = envelope,
                clientSignature = clientSig,
                auditStamp = AuditStamp(
                    envelopeHash = envelopeHash,
                    clientIp = clientIp,
                    timestamp = timestamp,
                ),
                auditSignature = auditSig,
            )
        }
    }
}

/*
 * Pairing
 */
@JsonClass(generateAdapter = true)
data class Challenge(
    val id: String,
    val timestamp: Long,
    val isInitial: Boolean,
)

@JsonClass(generateAdapter = true)
data class InitialPairQr(
    val secret: String,
)

@JsonClass(generateAdapter = true)
data class DelegatedPairQr(
    val challenge: String,
)

@JsonClass(generateAdapter = true)
data class PairFinishPayload(
    val challengeId: String,
    val publicKey: String,
    val delegationKey: String,
    val encKey: ByteArray,
    val auditPublicKey: ByteArray,
    val mainAttestationChain: List<String>,
    val delegationAttestationChain: List<String>,
)

@JsonClass(generateAdapter = true)
data class Delegation(
    val finishPayload: String,
    val expiresAt: Long,
    val allowedEntities: List<String>?,
)

@JsonClass(generateAdapter = true)
data class InitialPairFinishRequest(
    val finishPayload: String,
    val mac: String,
)

/*
 * Actions
 */
@JsonClass(generateAdapter = true)
data class Entity(
    val id: String,
    val name: String,
    val haEntity: String,
)

@JsonClass(generateAdapter = true)
data class UnlockChallenge(
    val id: String,
    val timestamp: Long,
    val entityId: String,
)

@JsonClass(generateAdapter = true)
data class UnlockStartRequest(
    @Json(name = "e")
    val entityId: String,
)
