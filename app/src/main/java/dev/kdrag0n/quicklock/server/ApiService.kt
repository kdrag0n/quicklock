package dev.kdrag0n.quicklock.server

import com.squareup.moshi.JsonClass
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path

interface ApiService {
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
    suspend fun getDelegatedPairSignature(@Path("challengeId") challengeId: String): Response<DelegationSignature>

    @POST("/api/pair/delegated/{challengeId}/signature")
    suspend fun uploadDelegatedPairSignature(
        @Path("challengeId") challengeId: String,
        @Body signature: DelegationSignature,
    ): Response<Unit>

    @POST("/api/pair/delegated/finish")
    suspend fun finishDelegatedPair(@Body request: DelegatedPairFinishRequest): Response<Unit>

    @POST("/api/pair/get_challenge")
    suspend fun getChallenge(): Response<Challenge>

    @POST("/api/unlock")
    suspend fun unlock(@Body request: UnlockRequest): Response<Unit>
}

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
data class DelegationSignature(
    val device: String,
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
    val signature: DelegationSignature,
)

@JsonClass(generateAdapter = true)
data class UnlockPayload(
    val publicKey: String,
    val timestamp: Long,
)

@JsonClass(generateAdapter = true)
data class UnlockRequest(
    val payload: String,
    val signature: String,
)
