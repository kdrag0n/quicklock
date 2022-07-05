package dev.kdrag0n.quicklock

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import dagger.Reusable
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.Signature
import java.util.*
import javax.inject.Inject

@Reusable
class CryptoService @Inject constructor() {
    private val keystore by lazy {
        KeyStore.getInstance("AndroidKeyStore").apply {
            load(null)
        }
    }

    private val entry
        get() = loadEntry()

    val publicKeyEncoded
        get() = Base64.encode(entry.certificate.publicKey.encoded, Base64.NO_WRAP).decodeToString()

    private fun loadEntry(): KeyStore.PrivateKeyEntry {
        return keystore.getEntry(ALIAS, null) as KeyStore.PrivateKeyEntry
    }

    fun generateKey(challengeId: String) {
        // Delete existing key if necessary
        try {
            keystore.deleteEntry(ALIAS)
        } catch (e: Exception) {
            // ignored
        }

        val gen = KeyPairGenerator.getInstance("EC", "AndroidKeyStore")
        val spec = KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_SIGN).run {
            setKeyValidityStart(Date())
            setAttestationChallenge(challengeId.encodeToByteArray())
            setDevicePropertiesAttestationIncluded(true)

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

    fun getAttestationChain() = keystore.getCertificateChain(ALIAS).map {
        Base64.encode(it.encoded, Base64.NO_WRAP).decodeToString()
    }

    companion object {
        const val ALIAS = "lock_main"
    }
}
