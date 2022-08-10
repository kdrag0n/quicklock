package dev.kdrag0n.quicklock

import dev.kdrag0n.quicklock.lib.RustLog

object NativeLib {
    init {
        System.loadLibrary("quicklock")
        System.loadLibrary("qlock_android")
        RustLog.init()
    }

    @JvmStatic external fun blsGeneratePrivateKey(seed: ByteArray): ByteArray
    @JvmStatic external fun blsDerivePublicKey(privateKey: ByteArray): ByteArray
    @JvmStatic external fun blsSignMessage(skData: ByteArray, message: ByteArray): ByteArray
}
