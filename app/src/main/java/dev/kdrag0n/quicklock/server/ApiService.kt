package dev.kdrag0n.quicklock.server

import com.squareup.moshi.JsonClass
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.POST

interface ApiService {
    @POST("/api/pair")
    suspend fun pair(@Body request: PairRequest): Response<Unit>

    @POST("/api/unlock")
    suspend fun unlock(@Body request: WrappedUnlockRequest): Response<Unit>
}

@JsonClass(generateAdapter = true)
data class PairRequest(
    val publicKey: String,
    val pairingKey: String,
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
