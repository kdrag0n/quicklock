package dev.kdrag0n.quicklock.server

import com.google.android.attestation.CertificateRevocationStatus
import java.io.ByteArrayInputStream
import java.security.KeyFactory
import java.security.Signature
import java.security.cert.CertificateFactory
import java.security.cert.X509Certificate
import java.security.interfaces.ECPublicKey
import java.security.spec.X509EncodedKeySpec
import java.util.*

object Crypto {
    fun parseCert(data: String): X509Certificate {
        val factory = CertificateFactory.getInstance("X.509")
        return factory.generateCertificate(ByteArrayInputStream(data.decodeBase64()))
            as X509Certificate
    }

    fun parsePublicKey(data: String): ECPublicKey {
        val spec = X509EncodedKeySpec(data.decodeBase64())
        val factory = KeyFactory.getInstance("EC")
        return factory.generatePublic(spec) as ECPublicKey
    }

    private fun verifyRootCert(cert: X509Certificate) =
        require(cert in Certificates.GOOGLE_ROOTS)

    fun verifyCertChain(certs: List<X509Certificate>): X509Certificate {
        val now = System.currentTimeMillis()

        // Verify root
        val root = certs.last()
        verifyRootCert(root)

        // Initial parent is root (last)
        var parent = root
        // Verify each cert starting from root, including parent
        certs.reversed().forEach { cert ->
            cert.checkValidity()
            cert.verify(parent.publicKey)
            // Issued by parent
            require(cert.issuerX500Principal.name == parent.subjectX500Principal.name)

            // Check revocation
            require(CertificateRevocationStatus.fetchStatus(cert.serialNumber) == null)

            parent = cert
        }

        // Return attestation cert
        return certs.first()
    }

    fun verifySignature(payload: String, publicKey: String, signature: String) {
        val sig = Signature.getInstance("SHA256withECDSA")
        sig.initVerify(Crypto.parsePublicKey(publicKey))
        sig.update(payload.toByteArray())
        require(sig.verify(signature.decodeBase64()))
    }
}

fun String.decodeBase64(): ByteArray =
    Base64.getDecoder().decode(this)

fun String.decodeBase64Url(): ByteArray =
    Base64.getUrlDecoder().decode(this)

fun ByteArray.toBase64(): String =
    Base64.getEncoder().encodeToString(this)

fun ByteArray.toBase64Url(): String =
    Base64.getUrlEncoder().encodeToString(this)
