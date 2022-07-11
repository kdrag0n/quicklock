package dev.kdrag0n.quicklock

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import dagger.Reusable
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PublicKey
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

    private fun getKeyEntry(alias: String = ALIAS) =
        keystore.getEntry(alias, null) as KeyStore.PrivateKeyEntry

    val publicKeyEncoded
        get() = getKeyEntry().certificate.publicKey.encoded.toBase64()

    fun generateKey(challengeId: String, isDelegation: Boolean = false): PublicKey {
        val alias = if (isDelegation) DELEGATION_ALIAS else ALIAS

        // Delete existing key if necessary
        try {
            keystore.deleteEntry(alias)
        } catch (e: Exception) {
            // ignored
        }

        val gen = KeyPairGenerator.getInstance("EC", "AndroidKeyStore")
        val spec = KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_SIGN).run {
            setKeyValidityStart(Date())
            setAttestationChallenge(challengeId.encodeToByteArray())
            setDevicePropertiesAttestationIncluded(true)

            if (isDelegation) {
                setUserAuthenticationRequired(true)
                setUserAuthenticationParameters(0, KeyProperties.AUTH_BIOMETRIC_STRONG or KeyProperties.AUTH_DEVICE_CREDENTIAL)
                setUnlockedDeviceRequired(true)
            }

            setDigests(KeyProperties.DIGEST_SHA256)
            setIsStrongBoxBacked(true)
            build()
        }

        gen.initialize(spec)
        return gen.generateKeyPair().public
    }

    fun prepareSignature(alias: String = ALIAS): Signature {
        val sig = Signature.getInstance("SHA256withECDSA")
        sig.initSign(getKeyEntry(alias).privateKey)
        return sig
    }

    fun finishSignature(sig: Signature, payload: String): String {
        val bytes = payload.encodeToByteArray()
        sig.update(bytes)

        val signature = sig.sign()
        return signature.toBase64()
    }

    fun signPayload(payload: String, alias: String = ALIAS): String {
        val sig = prepareSignature(alias)
        return finishSignature(sig, payload)
    }

    fun getAttestationChain(alias: String = ALIAS) = keystore.getCertificateChain(alias).map {
        it.encoded.toBase64()
    }

    companion object {
        const val ALIAS = "lock_main"
        const val DELEGATION_ALIAS = "lock_delegation"
    }
}

fun ByteArray.toBase64(): String =
    Base64.encodeToString(this, Base64.NO_WRAP)