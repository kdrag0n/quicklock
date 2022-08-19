# QuickLock protocol

QuickLock is a protocol that enables secure and convenient control of smart locks with support for flexible auditing and granting access to multiple users. It leverages efficient cryptographic primitives to minimize detrimental effects on user experience and enable practical implementations directly in smart lock controller firmware. The protocol can operate over multiple transports, including HTTP and NFC.

This document also defines a general-purpose auditing protocol based on QuickLock's auditing, which can be used to improve auditability and security of other applications.

This only describes the custom protocol without backwards compatibility with WebAuthn, which is a reasonable assumption for smart locks. WebAuthn is only supported by a legacy version of the server.

"Relying party" is defined as the smart lock controller or a separate server with the same responsibilities.

## Guarantees

- Devices with delegated access cannot compromise the integrity of the lock or perform unauthorized actions
- Sensitive actions (e.g. unlocking, resetting the lock) cannot be performed without being logged by Auditor
- Only the client is able to read the audit log; a malicious Auditor cannot read the requests being logged

## Primitives

The following cryptographic primitives have been selected for efficiency, and will be referenced by the following terms:

- Hash: BLAKE3
- HashIdShort: BLAKE3 truncated to 96 bits (for efficient ID generation)
- AEAD: XChaCha20-Poly1305
- MAC: HMAC-SHA256 (subject to change)
- ClientSig: ECDSA with SHA-256 and P-256 (for compatibility with Android StrongBox)
- AuditSig: Ed25519

## Keys

The protocol uses the following keys:

- AuditMacKey: Used by Alice to authenticate messages for Auditor. Known by Alice and Auditor.
- AuditSecretKey: Used by Auditor to sign logged messages. Known by Auditor.
- AuditPublicKey: Used by relying party to verify that Auditor has received a message. Known by Alice, Auditor, and Relying party.
- EnvelopeEncKey: Used by Alice to encrypt RequestEnvelopes and blind messages against Auditor. Known by Alice and Relying party.
- ClientSigSecretKey: Used by Alice to sign regular (i.e. unlock) requests to relying party. Known by Alice.
- ClientSigPublicKey: Used by relying party to verify that Alice is making a regular request and device integrity has not been compromised (bound to Android StrongBox). Known by Alice and Relying party.
- DelegationClientSigSecretKey: Used by Alice to sign dangerous (i.e. delegation) requests to relying party. Known by Alice.
- DelegationClientSigPublicKey: Used by relying party to verify that Alice is making a dangerous request and that device integrity has not been compromised (bound to Android StrongBox). Known by Alice and Relying party.

## Core messages

Below is a set of message structures shared between multiple flows.

All messages are serialized as JSON, with binary data encoded in Base64.

### RequestEnvelope

All requests are encapsulated in an encrypted and authenticated envelope. This is used to blind requests against Auditor and improve the security of the protocol over insecure transports.

Contents:

- Nonce
- AEAD(*request_payload*, EnvelopeEncKey, *nonce*)

### AuditStamp

When auditing is used, the Auditor will "stamp" the request envelope upon successful verification. This assures the relying party that the request has been logged and audited correctly.

Contents:

- Hash(*envelope*)
- Client IP address
- Timestamp (milliseconds since epoch)

### SignedRequestEnvelope

After signing a RequestEnvelope and obtaining an AuditStamp from the Auditor, the client combines its ID, the RequestEnvelope, the AuditStamp, and signatures from both parties into a SignedRequestEnvelope. This message is sent to the relying party for performing the requested action.

Contents:

- Client device ID
- RequestEnvelope (*envelope*)
- AuditStamp (*stamp*)
- ClientSig(*envelope*, ClientSigSecretKey)
- AuditSig(*stamp*, AuditSecretKey)

### AuditClientState

This structure contains persistent state that an audit client must store.

Contents:

- MAC key
- AEAD key

### LockClientState

This structure contains persistent state that a lock client must store.

Contents:

- Primary ClientSig keypair
- Delegation ClientSig keypair
- AuditClientState

The primar and delegation keypairs may be the same if attestation is disabled, in which case only the primary keypair needs to be stored.

## Common auditing procedures

The following auditing procedures are common to all authenticated flows.

### SealAndSign

All authenticated requests must be encapsulated in a [**SignedRequestEnvelope**](#signedrequestenvelope) with a valid signature from both Alice and Auditor:

1. Use AEAD to encrypt request message and "seal" it in a [**RequestEnvelope**](#requestenvelope)
2. Sign *RequestEnvelope* with provided ClientSig *keypair*
3. Compute MAC(*RequestEnvelope*, AuditMacKey) and send **SignRequest** to Auditor
4. Auditor performs the following steps and responds with a **SignResponse**:
   1. Verify the *MAC* in **SignRequest**
   2. Create **AuditStamp** with Hash(*RequestEnvelope*), timestamp, and client IP address
   3. Save **LogEvent** containing *RequestEnvelope*, *timestamp*, and *client IP address* to persistent storage
   4. Sign **AuditStamp** with AuditSig
5. Alice combines *RequestEnvelope*, *AuditStamp*, *ClientSig*, and *AuditSig* from **SignResponse** into a **SignedRequestEnvelope**

Contents of **SignRequest**:

- Alice's client ID issued at registration time (or alternatively, derived from her MAC key using Hash)
- *RequestEnvelope*
- MAC(*RequestEnvelope*, AuditMacKey)

To speed up the process, Alice can sign the request with ClientSig and obtain AuditSig in parallel.

### VerifyAndOpen

Relying party authenticates requests by verifying *SignedRequestEnvelope* signatures from both Alice and Auditor, then decrypting ("opening") the *RequestEnvelope*:

1. Verify *ClientSig* using ClientSigPublicKey (or DelegationClientSigPublicKey for dangerous requests)
2. Verify *AuditSig* using AuditPublicKey
3. Verify that *AuditStamp.envelopeHash* matches Hash(*RequestEnvelope*)
4. Verify that *AuditStamp* contains a valid timestamp within the grace period
5. If transport is IP-based, verify that *AuditStamp* contains a matching client IP address
6. Decrypt *RequestEnvelope* using AEAD and EnvelopeEncKey

## Registration

### Common procedures

#### FinishPair

Alice creates a **PairFinishPayload** during the pairing process:

- PairingChallenge *ID*
- AuditPublicKey
- ClientSigPublicKey
- DelegationClientSigPublicKey
- EnvelopeEncKey

To enroll a new device (Alice), the relying party verifies Alice's *PairFinishPayload*:

1. Verify that the request is tied to a valid, active *PairingChallenge*
2. Verify that no more than 5 minutes have elapsed since the challenge was generated
3. If attestation is enabled, verify *main attestation certificate chain* and *delegation attestation certificate chain*. Verify that the root of each certificate chain is a trusted Google root certificate. Verify that the ASN.1 fields in the `1.3.6.1.4.1.11129.2.1.17` extension match the parameters described in [Generate credentials](#2-generate-credentials).
4. Enroll Alice and store *ClientSigPublicKey*, *DelegationClientSigPublicKey*, *EnvelopeEncKey*, and *AuditPublicKey*. Alice will be identified by 96-bit client ID *HashIdShort(ClientSigPublicKey)* for future requests.

Access expiry time, delegator device, and allowed entities (locks) are also stored if this is a delegated registration.

### 1. Get pairing challenge

New client Alice sends a GetChallenge request to the relying party.

Relying party responds with a **PairingChallenge**:

- Challenge ID (random 256-bit nonce)
- Timestamp
- Flag: *isInitial* -- true if this is the first device to be paired, false otherwise

### 2. Generate credentials

Alice generates the following credentials, saved in **LockClientState**:

- ClientSigSecretKey
- DelegationClientSigSecretKey (for delegating new devices only)
- **AuditClientState**
  - AuditMacKey
  - EnvelopeEncKey

This must be done after getting a PairingChallenge in order to generate the key with the correct attestation challenge on Android. On Android devices, both keypairs must be generated with the following parameters:

- Attestation challenge = PairingChallenge *ID*
- Key validity start = now
- Device properties attestation = true
- Purpose = Signing with SHA-256 digest

Delegation keypairs must also be generated with the following parameters:

- User authentication required = true
- Unlocked device required = true

### 3. Register with Auditor

If auditing is enabled, Alice sends a **RegisterRequest** to Auditor:

- AuditMacKey

Auditor generates an **AuditSig** keypair for Alice and a persistent ID with Hash(*macKey*). It stores the keypair, associated with the ID, and responds with a **RegisterResponse**:

- Client ID
- AuditPublicKey

### 4. Create PairFinishPayload

Alice creates a **PairFinishPayload**:

- PairingChallenge *ID*
- ClientSigPublicKey
- DelegationClientSigPublicKey
- EnvelopeEncKey

This message acts as a registration request, but must be encapsulated in another request for authentication/authorization.

### Initial pairing

If *isInitial* is true, this is considered to be an initial pairing session. The rest of the flow proceeds as follows.

#### 5. Exchange pairing secret

The relying party exchanges a 256-bit shared secret for the initial pairing process. This secret can be generated randomly on-the-fly (as the reference implementation does) or generated and printed in the factory. The initial pairing flow is expected to be used rarely, generally only when Alice first sets up the lock or resets the lock due to a device loss, so physical access is required for security.

The secret can be exchanged by, for example, displaying a QR code that Alice can scan.

#### 6. Sign PairFinishPayload

To authenticate with the relying party and prove knowledge of the secret, Alice computes a MAC of the **PairFinishPayload** using the secret. Since the secret is shared but potentially static if generated in the factory, this scheme avoids disclosing the secret and prevents tampering with the message over insecure transports.

Alice sends a **InitialPairFinishRequest** to the relying party:

- **PairFinishPayload**
- MAC(*PairFinishPayload*, *secret*)

#### 7. Verify

Relying party verifies MAC(*PairFinishPayload*, *secret*), then verifies the *PairFinishPayload* and enrolls Alice by following [**FinishPair**](#finishpair).

### Delegated pairing

If *isInitial* is false, this is considered to be an initial pairing session.

#### 5. Upload PairFinishPayload

Alice uploads **PairFinishPayload** to the relying party in order to relay the request to an existing device (Bob) for signing.

#### 6. Delegate

Alice exchanges the PairingChallenge *ID* with Bob -- for example, by displaying a QR code with the ID.

Bob can then follow [Delegation](#delegation) to authorize Alice.

## Delegation

An existing paired device (Bob) can follow this flow to grant a new device (Alice) access to the lock.

### 1. Exchange challenge ID and PairFinishPayload

In the reference implementation, Alice displays a QR code with the PairingChallenge *ID*. Bob can then scan the QR code to obtain the ID.

Next, Bob uses the ID to fetch Alice's PairFinishPayload from the relying party.

### 2. Request user confirmation

Bob's device displays a visual representation of Alice's *public key* for user confirmation. Upon confirmation, Bob proceeds to [step 3](#3-seal--sign-delegatedpairfinishrequest).

Bob can also choose to impose restrictions on Alice's access:

- Access expiry time
- Allowed entities (locks)

### 3. Seal & sign DelegatedPairFinishRequest

Bob creates a **Delegation** for Alice:

- Alice's *PairFinishPayload*
- Access expiry time
- Allowed entities (locks)

To authorize Alice, Bob seals a request envelope with the **Delegation** and signs it by executing [**SealAndSign**](#sealandsign) with the delegation ClientSig keypair.

### 4. Verify

The relying party verifies and decrypts the *SignedRequestEnvelope* by executing [**VerifyAndOpen**](#verifyandopen), then verifies that the *PairFinishPayload* matches Alice's original request. Since the request has been authorized, the relying party can proceed to verify the *PairFinishPayload* and enroll Alice by following [**FinishPair**](#finishpair).

If Bob has user restrictions, the relying party limits Alice's access scope to Bob's access scope or the restrictions Bob attempted to impose, whichever one is more restrictive; that is, Alice cannot access more resources than Bob.

## Unlocking

### 1. Get unlock challenge

Alice sends an **UnlockStartRequest** to the relying party:

- Entity/lock ID to unlock

Relying party generates and responds with a new **UnlockChallenge**:

- Challenge ID (random 256-bit nonce)
- Timestamp
- Entity ID

If NFC is used, the relying party can expose a dynamic NDEF tag that generates and returns a challenge when read. Scanning the tag can trigger the unlock process. In this scenario, Alice never explicitly requests an unlock challenge.

### 2. Seal & sign UnlockChallenge

To authorize the request, Alice seals a request envelope with the **UnlockChallenge** from the relying party after validating its structure and signs it by executing [**SealAndSign**](#sealandsign) with the primary ClientSig keypair.

### 3. Verify

The relying party verifies and decrypts the *SignedRequestEnvelope* by executing [**VerifyAndOpen**](#verifyandopen), then checks whether Alice's user restrictions allow unlocking the requested entity. If so, it handles the request by dispatching the unlock action to the smart lock.
