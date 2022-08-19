package dev.kdrag0n.quicklock.server

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass
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
    fun toByteArray(): ByteArray = ByteBuffer.allocate(
        deviceId.size + // 32
                envelope.encPayload.size + // 48
                envelope.encNonce.size + // 24
                1 + // clientSignature.size
                clientSignature.size + // ~71
                auditStamp.envelopeHash.size + // 32
                auditStamp.clientIp.length + // 9 (TODO length)
                8 + // timestamp
                auditSignature.size // 64
    ).run {
        Timber.d("sizes: deviceId=${deviceId.size} encPayload=${envelope.encPayload.size} encNonce=${envelope.encNonce.size} 1 clientSig=${clientSignature.size} envelopeHash=${auditStamp.envelopeHash.size} clientIp=${auditStamp.clientIp.length} 8 auditSig=${auditSignature.size} total=${deviceId.size + // 32
                envelope.encPayload.size + // 48
                envelope.encNonce.size + // 24
                1 + // clientSignature.size
                clientSignature.size + // ~71
                auditStamp.envelopeHash.size + // 32
                auditStamp.clientIp.length + // 9 (TODO length)
                8 + // timestamp
                auditSignature.size}")
        put(deviceId)
        put(envelope.encPayload)
        put(envelope.encNonce)
        put(clientSignature.size.toByte())
        put(clientSignature)
        put(auditStamp.envelopeHash)
        put(auditStamp.clientIp.toByteArray())
        putLong(auditStamp.timestamp)
        put(auditSignature)
        array()
    }

    companion object {
        inline fun <reified T> fromByteArray(data: ByteArray): SignedRequestEnvelope<T> {
            val buffer = Buffer().write(data).peek()
            Timber.d("total d = ${data.size}")

            return SignedRequestEnvelope(
                deviceId = buffer.readByteArray(32),
                envelope = RequestEnvelope(
                    encPayload = buffer.readByteArray(48),
                    encNonce = buffer.readByteArray(24),
                ),
                clientSignature = run {
                    val size = buffer.readByte()
                    buffer.readByteArray(size.toLong())
                },
                auditStamp = AuditStamp(
                    envelopeHash = buffer.readByteArray(32),
                    clientIp = buffer.readUtf8(9),
                    timestamp = buffer.readLong(),
                ),
                auditSignature = buffer.readByteArray(64),
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
