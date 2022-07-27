package dev.kdrag0n.quicklock

object NativeLib {
    init {
        System.loadLibrary("quicklock")
    }

    @JvmStatic external fun init()
    @JvmStatic external fun blsGeneratePrivateKey(seed: ByteArray): ByteArray
    @JvmStatic external fun blsDerivePublicKey(privateKey: ByteArray): ByteArray
    @JvmStatic external fun blsSignMessage(skData: ByteArray, message: ByteArray): ByteArray
}
