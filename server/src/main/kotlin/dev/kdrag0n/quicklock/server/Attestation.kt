package dev.kdrag0n.quicklock.server

import com.google.android.attestation.ParsedAttestationRecord
import com.google.android.attestation.ParsedAttestationRecord.SecurityLevel
import java.security.cert.X509Certificate
import java.time.Instant
import kotlin.jvm.optionals.getOrNull

object Attestation {
    @OptIn(ExperimentalStdlibApi::class)
    private fun verifyAttestation(cert: X509Certificate, challengeId: String, isDelegation: Boolean = true) {
        val record = ParsedAttestationRecord.createParsedAttestationRecord(cert)

        require(record.attestationChallenge.contentEquals(challengeId.toByteArray()))
        require(record.attestationSecurityLevel == SecurityLevel.STRONG_BOX ||
                record.attestationSecurityLevel == SecurityLevel.TRUSTED_ENVIRONMENT)
        require(record.keymasterSecurityLevel == SecurityLevel.STRONG_BOX ||
                record.keymasterSecurityLevel == SecurityLevel.TRUSTED_ENVIRONMENT)

        record.softwareEnforced.activeDateTime.getOrNull()?.let {
            require(it.isBefore(Instant.now().plusMillis(Config.TIME_GRACE_PERIOD)))
        }
        record.softwareEnforced.creationDateTime.getOrNull()?.let {
            require(it.isBefore(Instant.now().plusMillis(Config.TIME_GRACE_PERIOD)))
        }
        record.softwareEnforced.usageExpireDateTime.getOrNull()?.let {
            require(it.isAfter(Instant.now().minusMillis(Config.TIME_GRACE_PERIOD)))
        }

        if (isDelegation) {
            require(!record.teeEnforced.noAuthRequired)
            require(record.teeEnforced.unlockedDeviceRequired)
        }
    }

    fun verifyChain(chain: List<String>, challengeId: String, isDelegation: Boolean = false) {
        val certs = chain.map { Crypto.parseCert(it) }
        val attestationCert = Crypto.verifyCertChain(certs)
        Attestation.verifyAttestation(attestationCert, challengeId, isDelegation)
    }
}
