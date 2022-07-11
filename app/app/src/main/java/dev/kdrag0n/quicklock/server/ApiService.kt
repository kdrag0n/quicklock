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
    suspend fun getDelegatedPairFinishPayload(@Path("challengeId") challengeId: String): Response<String>

    @POST("/api/pair/delegated/{challengeId}/finish_payload")
    suspend fun uploadDelegatedPairFinishPayload(
        @Path("challengeId") challengeId: String,
        @Body payload: String,
    ): Response<Unit>

    @GET("/api/pair/delegated/{challengeId}/signature")
    suspend fun getDelegatedPairSignature(@Path("challengeId") challengeId: String): Response<SignedDelegation>

    @POST("/api/pair/delegated/{challengeId}/signature")
    suspend fun uploadDelegatedPairSignature(
        @Path("challengeId") challengeId: String,
        @Body signature: SignedDelegation,
    ): Response<Unit>

    @POST("/api/pair/delegated/finish")
    suspend fun finishDelegatedPair(@Body request: DelegatedPairFinishRequest): Response<Unit>

    @POST("/api/pair/get_challenge")
    suspend fun getChallenge(): Response<Challenge>

    @POST("/api/unlock")
    suspend fun unlock(@Body request: UnlockRequest): Response<Unit>
}

@JsonClass(generateAdapter = true)
data class Entity(
    val id: String,
    val name: String,
    val haEntity: String,
)

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
    val challengeId: String,
)

@JsonClass(generateAdapter = true)
data class PairFinishPayload(
    val challengeId: String,
    val publicKey: String,
    val delegationKey: String,
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
data class SignedDelegation(
    val device: String,
    val delegation: String,
    val signature: String,
)

@JsonClass(generateAdapter = true)
data class InitialPairFinishRequest(
    val payload: String,
    val initialSecret: String,
)

@JsonClass(generateAdapter = true)
data class DelegatedPairFinishRequest(
    val payload: String,
    val signature: SignedDelegation,
)

@JsonClass(generateAdapter = true)
data class UnlockPayload(
    val entityId: String,
    val publicKey: String,
    val timestamp: Long,
)

@JsonClass(generateAdapter = true)
data class UnlockRequest(
    val payload: String,
    val signature: String,
)
