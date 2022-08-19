package dev.kdrag0n.quicklock

object NativeLib {
    init {
        System.loadLibrary("qlock_android")
    }

    @JvmStatic external fun blsGeneratePrivateKey(): ByteArray
    @JvmStatic external fun blsDerivePublicKey(privateKey: ByteArray): ByteArray
    @JvmStatic external fun blsSignMessage(skData: ByteArray, message: ByteArray): ByteArray
    @JvmStatic external fun blsAggregateSigs(pk1: ByteArray, sig1: ByteArray, pk2: ByteArray, sig2: ByteArray): ByteArray

    @JvmStatic external fun envelopeSeal(key: ByteArray, msg: ByteArray): String
    @JvmStatic external fun hash(msg: ByteArray): ByteArray
    @JvmStatic external fun hashIdShort(msg: ByteArray): ByteArray

    @JvmStatic external fun startServer()
    @JvmStatic external fun serverStartUnlock(entityId: String): String
    @JvmStatic external fun serverFinishUnlock(envelopeJson: String, id: String): String
}
