package dev.kdrag0n.quicklock.server

import java.io.ByteArrayInputStream
import java.security.KeyFactory
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
        certs.reversed().forEach {
            it.checkValidity()
            it.verify(parent.publicKey)
            require(it.issuerDN.name == parent.subjectDN.name)

            parent = it
        }

        // Return attestation cert
        return certs.first()
    }
}

fun String.decodeBase64() =
    Base64.getDecoder().decode(this)
