package dev.kdrag0n.quicklock

import android.nfc.cardemulation.HostApduService
import android.os.Bundle
import timber.log.Timber

class ServerHostApduService : HostApduService() {
    override fun processCommandApdu(commandApdu: ByteArray?, extras: Bundle?): ByteArray {
        Timber.d("processCommandApdu: $extras")
        return byteArrayOf(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)
    }

    override fun onDeactivated(reason: Int) {
        Timber.d("onDeactivated: $reason")
    }
}
