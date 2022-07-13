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
import kotlinx.serialization.decodeFromString
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

@Serializable
data class DelegatedPairFinishWA(
    val delegationKeyId: String,
    val signature: String,
    val clientDataJSON: String,
    val authenticatorData: String,
)

@Serializable
data class ClientData(
    val type: String,
    val challenge: String,
    val origin: String,
    val crossOrigin: Boolean,
)

@Serializable
data class PairDelegationWA(
    val challengeId: String,
    val pairFinishPayload: PairFinishWA,
    val expiresAt: Long,
    val allowedEntities: List<String>?,
)

private val unlockChallenges = HashMap<String, UnlockChallenge>()
private val authenticators = HashMap<String, AuthenticatorImpl>()

private val logger = LoggerFactory.getLogger("WebAuthn")

private val manager = WebAuthnManager(
    listOf(NoneAttestationStatementValidator()),
    NullCertPathTrustworthinessValidator(),
    NullSelfAttestationTrustworthinessValidator(),
)

private fun verifyAuth(
    keyId: String,
    signature: String,
    clientDataJSON: String,
    authenticatorData: String,
    challengeData: String,
) {
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

    authenticators[keyId]!!.counter = data.authenticatorData!!.signCount
}

private fun verifyCrossSignature(request: DelegatedPairFinishWA): PairDelegationWA {
    val (delegationKeyId, signature, clientDataJSON, authenticatorData) = request

    // Trust the data in the challenge - it's only for delegation purposes. We can verify the finish payload
    // against the copy we have.
    val clientData = Json.decodeFromString<ClientData>(clientDataJSON.decodeBase64().decodeToString())
    val challengeData = clientData.challenge.decodeBase64Url().decodeToString()
    val delegation = Json.decodeFromString<PairDelegationWA>(challengeData)

    verifyAuth(delegationKeyId, signature, clientDataJSON, authenticatorData, challengeData)
    return delegation
}

private fun finishPair(
    request: PairFinishWA,
    challenge: PairingChallenge,
    challengeData: ByteArray,
    delegatedBy: String?,
    expiresAt: Long = Long.MAX_VALUE,
    allowedEntities: List<String>? = null,
) {
    val (keyId, attestationObject, clientDataJSON) = request
    val req = RegistrationRequest(attestationObject.decodeBase64(), clientDataJSON.decodeBase64())
    val data = manager.parse(req)

    val challengeWA = RawChallenge(challengeData)
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

    require(challenge.isInitial == (delegatedBy == null))
    challenge.validate()

    Storage.addDevice(PairedDevice(
        publicKey = keyId,
        delegationKey = keyId,
        // Params
        expiresAt = expiresAt,
        delegatedBy = delegatedBy,
        allowedEntities = allowedEntities,
    ))
}

fun Application.webAuthnModule() = routing {
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
            val request = call.receive<PairFinishWA>()
            finishPair(request, challenge, challengeData.encodeToByteArray(), delegatedBy = null)
            call.respond(EmptyObject)
        } finally {
            pairingChallenges -= challenge.id
        }
    }

    /*
     * Delegated pairing
     */
    post("/api/webauthn/pair/delegated/{challengeId}/finish") {
        val id = call.parameters["challengeId"]!!
        val challenge = pairingChallenges[id]!!

        try {
            // Verify cross-signature of the delegatee's entire WebAuthn request
            val request = call.receive<DelegatedPairFinishWA>()
            Storage.getDeviceByKey(request.delegationKeyId)
            val delegation = verifyCrossSignature(request)

            // Finish pair
            finishPair(
                delegation.pairFinishPayload,
                challenge,
                challengeData = id.decodeBase64(),
                delegatedBy = request.delegationKeyId,
                expiresAt = delegation.expiresAt,
                allowedEntities = delegation.allowedEntities,
            )
            call.respond(EmptyObject)
        } finally {
            pairingChallenges -= challenge.id
            finishPayloads -= challenge.id
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
            verifyAuth(keyId, signature, clientDataJSON, authenticatorData, challengeData)

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
        } finally {
            unlockChallenges -= challenge.id
        }
    }
}

private class RawChallenge(private val value: ByteArray) : Challenge {
    override fun getValue() = value
}
