package dev.kdrag0n.quicklock

import android.nfc.cardemulation.HostApduService
import android.os.Bundle
import com.squareup.moshi.Moshi
import com.squareup.moshi.adapter
import dagger.hilt.android.AndroidEntryPoint
import dev.kdrag0n.quicklock.server.ApiService
import dev.kdrag0n.quicklock.server.NfcRequest
import dev.kdrag0n.quicklock.server.gunzipBytes
import kotlinx.coroutines.runBlocking
import okio.ByteString.Companion.toByteString
import retrofit2.Response
import timber.log.Timber
import java.nio.ByteBuffer
import javax.inject.Inject

@AndroidEntryPoint
class ServerHostApduService : HostApduService() {
    @Inject lateinit var service: ApiService
    @Inject lateinit var moshi: Moshi

    override fun processCommandApdu(commandApdu: ByteArray, extras: Bundle?): ByteArray {
        Timber.d("processCommandApdu: ${commandApdu.toByteString().base64()} (${commandApdu.size}) + $extras")

        val buf = ByteBuffer.wrap(commandApdu)
        val cla = buf.get()
        val ins = buf.get()
        val p1 = buf.get()
        val p2 = buf.get()

        var len = buf.get().toShort()
        if (len == 0.toShort()) {
            len = buf.short
        }

        Timber.d("processCommandApdu: cla: $cla, ins: $ins, p1: $p1, p2: $p2, len: $len")

        val payload = ByteArray(len.toInt())
        buf.get(payload)

        // SELECT: return OK
        if (cla == 0x00.toByte() && ins == 0xA4.toByte() && p1 == 0x04.toByte() && p2 == 0x00.toByte()) {
            Timber.d("SELECT: ok")
            return byteArrayOf(0x90.toByte(), 0x00.toByte())
        }

        // Custom protocol: HTTP service proxy
        if (cla == 0x01.toByte() && ins == 0x01.toByte()) {
            val payloadStr = gunzipBytes(payload).decodeToString()
            val req: NfcRequest = parseReq(payloadStr)
            Timber.d("Custom protocol: HTTP proxy $req")
            val resp = runBlocking {
                when (p1.toInt()) {
                    1 -> encodeResp(service.getEntities())
                    2 -> encodeResp(service.startInitialPair())
                    3 -> encodeResp(service.finishInitialPair(parseReq(req.payload!!)))
                    4 -> encodeResp(service.getDelegatedPairFinishPayload(req.challengeId!!))
                    5 -> encodeResp(service.uploadDelegatedPairFinishPayload(req.challengeId!!, parseReq(req.payload!!)))
                    6 -> encodeResp(service.finishDelegatedPair(req.challengeId!!, parseReq(req.payload!!)))
                    7 -> encodeResp(service.getPairingChallenge())
                    8 -> encodeResp(service.startUnlock(parseReq(req.payload!!)))
                    9 -> encodeResp(service.finishUnlock(req.challengeId!!, parseReq(req.payload!!)))
                    else -> null
                }
            }

            resp?.let {
                return it
            }
        }

        return byteArrayOf(0x00)
    }

    override fun onDeactivated(reason: Int) {
        Timber.d("onDeactivated: $reason")
    }

    @OptIn(ExperimentalStdlibApi::class)
    private inline fun <reified T> encodeResp(resp: Response<T>): ByteArray? {
        val body = resp.body() ?: return "{}".encodeToByteArray()
        if (body == Unit) return "{}".encodeToByteArray()
        return moshi.adapter<T>().toJson(body).encodeToByteArray()
    }

    @OptIn(ExperimentalStdlibApi::class)
    private inline fun <reified T> parseReq(req: String): T {
        return moshi.adapter<T>().fromJson(req)!!
    }
}

