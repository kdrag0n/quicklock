package dev.kdrag0n.quicklock.server

import com.google.android.attestation.ParsedAttestationRecord
import com.google.android.attestation.ParsedAttestationRecord.SecurityLevel
import java.security.cert.X509Certificate
import java.time.Instant
import kotlin.jvm.optionals.getOrNull

object Attestation {
    @OptIn(ExperimentalStdlibApi::class)
    fun verifyAttestation(cert: X509Certificate, challengeId: String) {
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
    }
}
