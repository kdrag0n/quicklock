package dev.kdrag0n.quicklock

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import dagger.Reusable
import dev.kdrag0n.quicklock.util.profileLog
import okio.ByteString.Companion.decodeBase64
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PublicKey
import java.security.SecureRandom
import java.security.Signature
import java.util.*
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec
import javax.inject.Inject

@Reusable
class CryptoService @Inject constructor(
    private val settings: SettingsRepository,
) {
    private val keystore by lazy {
        KeyStore.getInstance("AndroidKeyStore").apply {
            load(null)
        }
    }

    private fun getKeyEntry(alias: String = ALIAS) =
        keystore.getEntry(alias, null) as KeyStore.PrivateKeyEntry

    val publicKeyEncoded
        get() = getKeyEntry().certificate.publicKey.encoded.toBase64()

    val delegationKeyEncoded
        get() = getKeyEntry(alias = DELEGATION_ALIAS).certificate.publicKey.encoded.toBase64()

    val blsPublicKeyBytes by lazy {
        val sk = settings.macKey!!.decodeBase64()!!.toByteArray()
        profileLog("blsDerivePk") {
            NativeLib.blsDerivePublicKey(sk)
        }
    }

    val blsPublicKeyEncoded
        get() = blsPublicKeyBytes.toBase64()

    val encKeyBytes
        get() = settings.encKey!!.decodeBase64()!!.toByteArray()

    val macKeyBytes
        get() = settings.macKey!!.decodeBase64()!!.toByteArray()

    val auditServerPublicKeyBytes
        get() = settings.auditServerPublicKey!!.decodeBase64()!!.toByteArray()

    val auditClientId
        get() = settings.auditClientId!!

    fun generateKeys() {
        // Encryption
        val encKey = ByteArray(32)
        SecureRandom.getInstanceStrong().nextBytes(encKey)
        settings.encKey = encKey.toBase64()

        // MAC key
        val macKey = ByteArray(32)
        SecureRandom.getInstanceStrong().nextBytes(macKey)
        settings.macKey = macKey.toBase64()
    }

    fun signBls(data: ByteArray): ByteArray {
        val sk = settings.macKey!!.decodeBase64()!!.toByteArray()
        val sig = NativeLib.blsSignMessage(sk, data)
        return sig
    }

    fun signMac(data: ByteArray): ByteArray {
        val mac = Mac.getInstance("HmacSHA256")
        val key = SecretKeySpec(macKeyBytes, "HmacSHA256")
        mac.init(key)
        return mac.doFinal(data)
    }

    fun saveServerKey(clientId: String, serverPk: ByteArray) {
        settings.auditClientId = clientId
        settings.auditServerPublicKey = serverPk.toBase64()
    }

    fun aggregateBls(clientSig: ByteArray, serverSig: ByteArray): ByteArray {
        val serverPk = settings.auditServerPublicKey!!.decodeBase64()!!.toByteArray()
        val clientPk = blsPublicKeyBytes
        return profileLog("blsAggregate") {
            NativeLib.blsAggregateSigs(clientPk, clientSig, serverPk, serverSig)
        }
    }

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
//            setIsStrongBoxBacked(true)
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

    fun finishSignature(sig: Signature, payload: ByteArray): ByteArray {
        sig.update(payload)
        return sig.sign()
    }

    fun signPayload(payload: ByteArray, alias: String = ALIAS): ByteArray {
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

fun String.decodeBase64Url(): ByteArray =
    Base64.decode(this, Base64.URL_SAFE)
