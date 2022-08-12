package dev.kdrag0n.quicklock.server

import android.nfc.tech.IsoDep
import com.squareup.moshi.JsonClass
import com.squareup.moshi.Moshi
import com.squareup.moshi.adapter
import dev.kdrag0n.quicklock.ui.apduExt
import okio.ByteString.Companion.toByteString
import retrofit2.Response
import timber.log.Timber
import java.io.ByteArrayOutputStream
import java.util.zip.GZIPInputStream
import java.util.zip.GZIPOutputStream

class NfcApiService(
    private val tag: IsoDep,
    private val moshi: Moshi,
) : ApiService {
    private val reqAdapter = moshi.adapter(NfcRequest::class.java)

    override suspend fun getEntities() =
        transceive<_, List<Entity>>(1, payload = NoPayload)

    override suspend fun startInitialPair() =
        transceive<_, Unit>(2, payload = NoPayload)

    override suspend fun finishInitialPair(request: InitialPairFinishRequest) =
        transceive<_, Unit>(3, payload = request)

    override suspend fun getDelegatedPairFinishPayload(challengeId: String) =
        transceive<_, PairFinishPayload>(4, challengeId, payload = NoPayload)

    override suspend fun uploadDelegatedPairFinishPayload(
        challengeId: String,
        payload: PairFinishPayload
    ) = transceive<_, Unit>(5, challengeId, payload = payload)

    override suspend fun finishDelegatedPair(
        challengeId: String,
        request: SignedRequestEnvelope<Delegation>
    ) = transceive<_, Unit>(6, challengeId, payload = request)

    override suspend fun getPairingChallenge() =
        transceive<_, Challenge>(7, payload = NoPayload)

    override suspend fun startUnlock(request: UnlockStartRequest) =
        transceive<_, UnlockChallenge>(8, payload = request)

    override suspend fun finishUnlock(
        challengeId: String,
        request: SignedRequestEnvelope<UnlockChallenge>
    ) = transceive<_, Unit>(9, challengeId, payload = request)

    private inline fun <reified ReqT, reified RespT> transceive(
        endpoint: Int,
        challengeId: String? = null,
        payload: ReqT? = null,
    ): Response<RespT> {
        val payloadData = if (payload == NoPayload) null else payload?.let { encodeReq(it) }
        val req = NfcRequest(challengeId, payloadData)
        val reqData = reqAdapter.toJson(req).encodeToByteArray()
        val gzipData = gzipBytes(reqData)

        val apdu = apduExt(1, 1, endpoint.toByte(), 0, gzipData)
        Timber.d("req: payload=${reqData.size} apdu=${apdu.size}")
        val respData = tag.transceive(apdu)
        if (respData[0] != '{'.code.toByte() && respData[0] != '['.code.toByte()) {
            error("Invalid resp data: ${respData.toByteString().hex()}")
        }

        if (RespT::class == Unit::class) {
            return Response.success(Unit as RespT)
        }

        val resp = parseResp<RespT>(respData.decodeToString())
        Timber.d("NFC resp: ${resp}")
        return Response.success(resp)
    }

    @OptIn(ExperimentalStdlibApi::class)
    private inline fun <reified T> encodeReq(req: T): String {
        return moshi.adapter<T>().toJson(req)
    }

    @OptIn(ExperimentalStdlibApi::class)
    private inline fun <reified T> parseResp(req: String): T {
        return moshi.adapter<T>().fromJson(req)!!
    }

    fun test() = tag.isConnected
}

@JsonClass(generateAdapter = true)
data class NfcRequest(
    val challengeId: String?,
    val payload: String?,
)

private object NoPayload

fun gzipBytes(data: ByteArray): ByteArray {
    val bos = ByteArrayOutputStream()
    GZIPOutputStream(bos).use {
        it.write(data)
    }
    return bos.toByteArray()
}

fun gunzipBytes(data: ByteArray): ByteArray {
    val bos = ByteArrayOutputStream()
    GZIPInputStream(data.inputStream()).use {
        it.copyTo(bos)
    }
    return bos.toByteArray()
}
