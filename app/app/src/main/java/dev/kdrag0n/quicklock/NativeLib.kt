package dev.kdrag0n.quicklock

object NativeLib {
    init {
        System.loadLibrary("qlock_android")
    }

    @JvmStatic external fun blsGeneratePrivateKey(): ByteArray
    @JvmStatic external fun blsDerivePublicKey(privateKey: ByteArray): ByteArray
    @JvmStatic external fun blsSignMessage(skData: ByteArray, message: ByteArray): ByteArray
    @JvmStatic external fun blsAggregateSigs(pk1: ByteArray, sig1: ByteArray, pk2: ByteArray, sig2: ByteArray): ByteArray

    @JvmStatic external fun envelopeSeal(key: ByteArray, msg: String): String

    @JvmStatic external fun startServer()
}
