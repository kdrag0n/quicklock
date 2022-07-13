package dev.kdrag0n.quicklock.server

import com.webauthn4j.WebAuthnManager
import com.webauthn4j.authenticator.AuthenticatorImpl
import com.webauthn4j.data.*
import com.webauthn4j.data.attestation.statement.COSEAlgorithmIdentifier
import com.webauthn4j.data.client.Origin
import com.webauthn4j.data.client.challenge.Challenge
import com.webauthn4j.server.ServerProperty
import com.webauthn4j.validator.attestation.statement.none.NoneAttestationStatementValidator
import com.webauthn4j.validator.attestation.trustworthiness.certpath.NullCertPathTrustworthinessValidator
import com.webauthn4j.validator.attestation.trustworthiness.self.NullSelfAttestationTrustworthinessValidator
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import io.ktor.utils.io.*
import io.ktor.utils.io.core.*
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import okio.ByteString.Companion.toByteString
import org.slf4j.LoggerFactory
import java.io.ByteArrayOutputStream
import java.io.ObjectOutputStream
import java.util.*

@Serializable
data class UnlockStartWA(
    val entityId: String,
)

@Serializable
data class UnlockChallenge(
    val id: String,
    val timestamp: Long,
    val entityId: String,
)

@Serializable
data class InitialPairFinishPayloadWA(
    val challengeId: String,
    val challengeMac: String,
)

@Serializable
data class PairFinishWA(
    val keyId: String,
    val attestationObject: String,
    val clientDataJSON: String,
)

@Serializable
data class UnlockFinishWA(
    val keyId: String,
    val signature: String,
    val clientDataJSON: String,
    val authenticatorData: String,
)

private val unlockChallenges = HashMap<String, UnlockChallenge>()
private val authenticators = HashMap<String, AuthenticatorImpl>()

private val logger = LoggerFactory.getLogger("WebAuthn")

fun Application.webAuthnModule() = routing {
    val manager = WebAuthnManager(
        listOf(NoneAttestationStatementValidator()),
        NullCertPathTrustworthinessValidator(),
        NullSelfAttestationTrustworthinessValidator(),
    )

    /*
     * Initial
     *
     * 1. POST /api/pair/get_challenge: same challenge as custom protocol
     * 2. POST /api/pair/initial/start: trigger QR (because isInitial=true)
     * 3. Scan QR to get secret
     * 4. HMAC challenge ID using secret to prove knowledge of secret
     * 5. POST /webauthn/pair/initial/finish: signed WebAuthn credential [*]
     */
    post("/api/webauthn/pair/initial/{challengeId}/finish") {
        val id = call.parameters["challengeId"]!!
        val challenge = pairingChallenges[id]!!

        // HMAC
        val secret = initialPairingSecret!!
        val mac = challenge.id.encodeToByteArray().toByteString()
            .hmacSha256(secret.decodeBase64().toByteString())
            .base64()
        initialPairingSecret = null

        val challengeData = Json.encodeToString(InitialPairFinishPayloadWA(
            challengeId = challenge.id,
            challengeMac = mac,
        ))

        try {
            val (keyId, attestationObject, clientDataJSON) = call.receive<PairFinishWA>()
            val req = RegistrationRequest(attestationObject.decodeBase64(), clientDataJSON.decodeBase64())
            val data = manager.parse(req)

            println("chal ${challengeData}")
            val challengeWA = RawChallenge(challengeData.encodeToByteArray())
            val params = RegistrationParameters(
                ServerProperty(Origin("http://localhost:3000"), "localhost", challengeWA, null),
                listOf(
                    PublicKeyCredentialParameters(PublicKeyCredentialType.PUBLIC_KEY, COSEAlgorithmIdentifier.ES256),
                ),
                false,
                true,
            )
            manager.validate(data, params)

            val attestation = data.attestationObject!!
            val authenticator = AuthenticatorImpl(
                attestation.authenticatorData.attestedCredentialData!!,
                attestation.attestationStatement,
                attestation.authenticatorData.signCount,
            )
            // TODO
            authenticators[keyId] = authenticator

            require(challenge.isInitial)
            challenge.validate()

            Storage.addDevice(PairedDevice(
                publicKey = keyId,
                delegationKey = keyId,
                // Params
                expiresAt = Long.MAX_VALUE,
                delegatedBy = null,
                allowedEntities = null,
            ))
            call.respond(EmptyObject)
        } finally {
            pairingChallenges -= challenge.id
        }
    }

    /*
     * Unlock
     */
    post("/api/webauthn/unlock/start") {
        val (entityId) = call.receive<UnlockStartWA>()
        val entity = Config.HA_ENTITIES[entityId]!!

        val challenge = UnlockChallenge(
            id = generateSecret(),
            timestamp = System.currentTimeMillis(),
            entityId = entity.id,
        )
        unlockChallenges[challenge.id] = challenge
        call.respond(challenge)
    }

    post("/api/webauthn/unlock/{challengeId}/finish") {
        val id = call.parameters["challengeId"]!!
        val challenge = unlockChallenges[id]!!
        require((System.currentTimeMillis() - challenge.timestamp) <= Config.TIME_GRACE_PERIOD)
        val challengeData = Json.encodeToString(challenge)

        try {
            val (keyId, signature, clientDataJSON, authenticatorData) = call.receive<UnlockFinishWA>()

            val req = AuthenticationRequest(
                keyId.decodeBase64(),
                authenticatorData.decodeBase64(),
                clientDataJSON.decodeBase64(),
                signature.decodeBase64(),
            )
            val params = AuthenticationParameters(
                ServerProperty(
                    Origin("http://localhost:3000"),
                    "localhost",
                    RawChallenge(challengeData.encodeToByteArray()),
                    null
                ),
                authenticators[keyId]!!,
                listOf(keyId.decodeBase64()),
                false,
                true,
            )

            val data = manager.parse(req)
            manager.validate(data, params)

            // Unlock
            logger.info("Posting HA unlock")
            HomeAssistant.postLock(true, challenge.entityId)
            call.respond(EmptyObject)

            // Re-lock after delay
            launch {
                delay(Config.RELOCK_DELAY)
                logger.info("Posting HA lock")
                HomeAssistant.postLock(false, challenge.entityId)
            }

            authenticators[keyId]!!.counter = data.authenticatorData!!.signCount
        } finally {
            unlockChallenges -= challenge.id
        }
    }
}

private class RawChallenge(private val value: ByteArray) : Challenge {
    override fun getValue() = value
}
