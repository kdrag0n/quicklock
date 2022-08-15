package dev.kdrag0n.quicklock.server

import com.squareup.moshi.JsonClass
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path

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
        @Body request: SignedRequestEnvelope<UnlockChallenge>,
    ): Response<Unit>
}

/*
 * Audit/generic
 */
@JsonClass(generateAdapter = true)
data class RequestEnvelope(
    val encPayload: ByteArray,
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
    val deviceId: String,
    val envelope: RequestEnvelope,
    val clientSignature: ByteArray,
    val auditStamp: AuditStamp,
    val auditSignature: ByteArray,
)

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
    val entityId: String,
)
