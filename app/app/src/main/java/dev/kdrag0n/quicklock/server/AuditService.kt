package dev.kdrag0n.quicklock.server

import com.squareup.moshi.JsonClass
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.POST

interface AuditService {
    @POST("/api/register")
    suspend fun register(@Body request: RegisterRequest): Response<RegisterResponse>

    @POST("/api/sign")
    suspend fun sign(@Body request: SignRequest): Response<SignResponse>
}

@JsonClass(generateAdapter = true)
data class RegisterRequest(
    val clientMacKey: ByteArray,
)

@JsonClass(generateAdapter = true)
data class RegisterResponse(
    val clientId: String,
    val serverPk: ByteArray,
)

@JsonClass(generateAdapter = true)
data class SignRequest(
    val clientId: String,
    val envelope: ByteArray,
    val clientMac: ByteArray,
)

@JsonClass(generateAdapter = true)
data class SignResponse(
    val stamp: ByteArray,
    val serverSig: ByteArray,
)
