package dev.kdrag0n.quicklock.server

import android.nfc.tech.IsoDep
import com.squareup.moshi.JsonClass
import com.squareup.moshi.Moshi
import com.squareup.moshi.adapter
import dev.kdrag0n.quicklock.toBase64
import dev.kdrag0n.quicklock.ui.apduExt
import dev.kdrag0n.quicklock.util.profileLog
import okio.Buffer
import okio.ByteString.Companion.decodeBase64
import okio.ByteString.Companion.toByteString
import retrofit2.Response
import timber.log.Timber
import java.io.ByteArrayOutputStream
import java.util.zip.DeflaterInputStream
import java.util.zip.DeflaterOutputStream
import java.util.zip.GZIPInputStream
import java.util.zip.GZIPOutputStream
import java.util.zip.InflaterInputStream

const val FORMAT_BINARY = 0x00
const val FORMAT_ZLIB = 0x01

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
        request: SignedRequestEnvelope<ByteArray>,
    ) = transceive<_, Unit>(9, challengeId, payload = request.toByteArray())

    private inline fun <reified ReqT, reified RespT> transceive(
        endpoint: Int,
        challengeId: String? = null,
        payload: ReqT? = null,
    ): Response<RespT> {
        val payloadData = if (payload == NoPayload) null else payload?.let { encodeReq(it) }
        val req = NfcRequest(challengeId, payloadData)
        val reqData = req.toByteArray()
        val gzipData = if (payload is ByteArray || reqData.size < 64) byteArrayOf(FORMAT_BINARY.toByte()) + reqData else gzipBytes(reqData)

        val apdu = apduExt(1, 1, endpoint.toByte(), 0, gzipData)
        Timber.d("req: payload=${reqData.size} apdu=${apdu.size}")
        Timber.d("req: pl contents=${reqData.toBase64()}")
        val respData = profileLog("nfcTransceive") {
            tag.transceive(apdu)
        }
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
    private inline fun <reified T> encodeReq(req: T): ByteArray {
        if (req is ByteArray) {
            return req
        }

        return moshi.adapter<T>().toJson(req).encodeToByteArray()
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
    val payload: ByteArray?,
) {
    fun toByteArray() = Buffer().run {
        writeByte(if (challengeId == null) 0 else 1)
        if (challengeId != null) {
            write(challengeId.decodeBase64()!!)
        }

        writeByte(if (payload == null) 0 else 1)
        if (payload != null) {
            write(payload)
        }

        snapshot().toByteArray()
    }

    companion object {
        fun fromByteArray(data: ByteArray): NfcRequest {
            Timber.d("req parse: ${data.toBase64()}")
            val buf = Buffer().write(data).peek()
            val challengeId = if (buf.readByte() == 1.toByte()) buf.readByteArray(32) else null
            val payload = if (buf.readByte() == 1.toByte()) buf.readByteArray() else null
            return NfcRequest(challengeId?.toBase64(), payload)
        }
    }
}

private object NoPayload

fun gzipBytes(data: ByteArray): ByteArray = profileLog("gzip") {
    val bos = ByteArrayOutputStream()
    bos.write(FORMAT_ZLIB)
    DeflaterOutputStream(bos).use {
        it.write(data)
    }
    bos.toByteArray()
}

fun gunzipBytes(data: ByteArray): ByteArray = profileLog("gunzip") {
    val bos = ByteArrayOutputStream()
    val ins = data.inputStream()
    if (ins.read() != FORMAT_ZLIB) {
        Timber.d("gunzip fmt: binary")
        return@profileLog data.sliceArray(1..data.lastIndex)
    }
    Timber.d("gunzip fmt: zlib")

    InflaterInputStream(ins).use {
        it.copyTo(bos)
    }
    bos.toByteArray()
}
