package dev.kdrag0n.quicklock

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import dagger.Reusable
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.Signature
import javax.inject.Inject

@Reusable
class CryptoService @Inject constructor() {
    private val entry by lazy {
        try {
            loadEntry()
        } catch (e: Exception) {
            generateKeypair()
            loadEntry()
        }
    }

    val publicKeyEncoded
        get() = Base64.encode(entry.certificate.publicKey.encoded, Base64.NO_WRAP).decodeToString()

    private fun loadEntry(): KeyStore.PrivateKeyEntry {
        val keystore = KeyStore.getInstance("AndroidKeyStore")
        keystore.load(null)
        return keystore.getEntry(ALIAS, null) as KeyStore.PrivateKeyEntry
    }

    private fun generateKeypair(alias: String = ALIAS) {
        val gen = KeyPairGenerator.getInstance("EC", "AndroidKeyStore")
        val spec = KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_SIGN).run {
            setDigests(KeyProperties.DIGEST_SHA256)
            setIsStrongBoxBacked(true)
            build()
        }

        gen.initialize(spec)
        gen.generateKeyPair()
    }

    fun signPayload(payload: String): String {
        val bytes = payload.encodeToByteArray()
        val sig = Signature.getInstance("SHA256withECDSA")
        sig.initSign(entry.privateKey)
        sig.update(bytes)
        val signature = sig.sign()
        return Base64.encode(signature, Base64.NO_WRAP).decodeToString()
    }

    companion object {
        const val ALIAS = "lock_main"
    }
}
