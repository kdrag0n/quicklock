package dev.kdrag0n.quicklock.server

import com.squareup.moshi.JsonClass
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.POST

interface ApiService {
    @POST("/api/pair/start")
    suspend fun startPair(@Body request: PairStartRequest): Response<Challenge>

    @POST("/api/pair/finish")
    suspend fun finishPair(@Body request: PairFinishRequest): Response<Unit>

    @POST("/api/unlock")
    suspend fun unlock(@Body request: WrappedUnlockRequest): Response<Unit>
}

@JsonClass(generateAdapter = true)
data class PairStartRequest(
    val pairingSecret: String,
)

@JsonClass(generateAdapter = true)
data class Challenge(
    val id: String,
    val timestamp: Long,
)

@JsonClass(generateAdapter = true)
data class PairFinishRequest(
    val challengeId: String,
    val publicKey: String,
    val attestationChain: List<String>,
)

@JsonClass(generateAdapter = true)
data class UnlockRequest(
    val publicKey: String,
    val timestamp: Long,
)

@JsonClass(generateAdapter = true)
data class WrappedUnlockRequest(
    val payload: String,
    val signature: String,
)
