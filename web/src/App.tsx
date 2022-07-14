import React, { useEffect, useState } from 'react';
import logo from './logo.svg';
import './App.css';
import { Button, Divider, Loader, Text } from '@mantine/core';
import useSWR from 'swr';
import * as base64 from 'base64-arraybuffer'
import QRCode from 'react-qr-code'

const API_BASE_URL = 'http://localhost:3002/api'

async function fetchApiJson(path: string, init?: RequestInit) {
  let resp = await fetch(API_BASE_URL + path, init)
  if (!resp.ok) {
    throw new Error(`${resp.status} ${resp.statusText}`)
  }
  return await resp.json()
}

interface Entity {
  id: string
  name: string
  haEntity: string
}

interface PairingChallenge {
  id: string
  timestamp: number
  isInitial: boolean
}

interface InitialPairFinishPayloadWA {
  challengeId: string
  challengeMac: string // to prove knowledge of secret
}

interface UnlockStartRequest {
  entityId: string
}

interface PairFinishWA {
  keyId: string
  attestationObject: string
  clientDataJSON: string
}

interface UnlockFinishWA {
  keyId: string
  signature: string
  clientDataJSON: string
  authenticatorData: string
}

interface Delegation {
  finishPayload: string
  expiresAt: number
  allowedEntities: string[] | null
}

interface DelegatedPairFinishWA {
  delegationKeyId: string
  signature: string
  clientDataJSON: string
  authenticatorData: string
}

function toBytes(str: string) {
  return Uint8Array.from(str, c => c.charCodeAt(0))
}

function getCredentialId() {
  return localStorage.credentialId! as string
}

async function unlock(entity: Entity) {
  // Get challenge with info
  let challenge = await fetchApiJson('/unlock/start', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ 
      entityId: entity.id, 
    } as UnlockStartRequest),
  })

  // Sign
  let credential = await navigator.credentials.get({
    publicKey: {
      challenge: toBytes(JSON.stringify(challenge)),
      rpId: 'localhost',
      allowCredentials: [{
        type: 'public-key',
        id: base64.decode(getCredentialId()),
      }],
      timeout: 60000,
      userVerification: 'discouraged',
    },
  }) as PublicKeyCredential
  console.log(credential)

  // Unlock
  let credResp = credential.response as AuthenticatorAssertionResponse
  let resp = await fetchApiJson(`/webauthn/unlock/${encodeURIComponent(challenge.id)}/finish`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      keyId: base64.encode(credential.rawId), // pre-encoded id is base64url
      signature: base64.encode(credResp.signature),
      clientDataJSON: base64.encode(credResp.clientDataJSON),
      authenticatorData: base64.encode(credResp.authenticatorData),
    } as UnlockFinishWA),
  })

  return resp
}

async function startInitialPair(challenge: PairingChallenge) {
  // Trigger QR
  await fetchApiJson('/pair/initial/start', { method: 'POST' })

  // Get secret (normally done by scanning QR)
  let secretB64 = prompt('Enter secret:')
  if (!secretB64) return
  let secretBytes = base64.decode(secretB64)

  // HMAC to prove knowledge of secret
  let key = await crypto.subtle.importKey('raw', secretBytes, { name: 'HMAC', hash: 'SHA-256' }, false, ['sign'])
  let mac = await crypto.subtle.sign('HMAC', key, toBytes(challenge.id))

  // Use WebAuthn as an envelope for the challenge ID and MAC
  let authPayload: InitialPairFinishPayloadWA = {
    challengeId: challenge.id,
    challengeMac: base64.encode(mac),
  }

  // Sign response
  let credential = await navigator.credentials.create({
    publicKey: {
      challenge: toBytes(JSON.stringify(authPayload)),
      rp: {
        id: 'localhost',
        name: 'Main',
      },
      user: {
        // Doesn't matter. We use PK as the identifier after registration
        id: toBytes(crypto.randomUUID()),
        name: 'Main',
        displayName: 'Main',
      },
      authenticatorSelection: {
        userVerification: 'discouraged',
      },
      pubKeyCredParams: [{ alg: -7, type: 'public-key' }],
      timeout: 60000,
      attestation: 'none',
    }
  }) as PublicKeyCredential
  console.log(credential)
  localStorage.credentialId = base64.encode(credential.rawId) // pre-encoded id is base64url

  // Finish
  let credResp = credential.response as AuthenticatorAttestationResponse
  await fetchApiJson(`/webauthn/pair/initial/${encodeURIComponent(challenge.id)}/finish`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      keyId: base64.encode(credential.rawId), // pre-encoded id is base64url
      attestationObject: base64.encode(credResp.attestationObject),
      clientDataJSON: base64.encode(credResp.clientDataJSON),
    } as PairFinishWA),
  })
}

// Delegatee
async function startDelegatedPair(challenge: PairingChallenge, showChallenge: (id: string) => void) {
  // Sign finish response
  let credential = await navigator.credentials.create({
    publicKey: {
      // TODO: wrapper to avoid signing arbitrary data?
      challenge: base64.decode(challenge.id),
      rp: {
        id: 'localhost',
        name: 'Main',
      },
      user: {
        // Doesn't matter. We use PK as the identifier after registration
        id: toBytes(crypto.randomUUID()),
        name: 'Main',
        displayName: 'Main',
      },
      authenticatorSelection: {
        userVerification: 'discouraged',
      },
      pubKeyCredParams: [{ alg: -7, type: 'public-key' }],
      timeout: 60000,
      attestation: 'none',
    }
  }) as PublicKeyCredential
  console.log(credential)
  localStorage.credentialId = base64.encode(credential.rawId) // pre-encoded id is base64url

  // Upload for cross-signing
  let credResp = credential.response as AuthenticatorAttestationResponse
  await fetchApiJson(`/pair/delegated/${encodeURIComponent(challenge.id)}/finish_payload`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      keyId: base64.encode(credential.rawId), // pre-encoded id is base64url
      attestationObject: base64.encode(credResp.attestationObject),
      clientDataJSON: base64.encode(credResp.clientDataJSON),
    } as PairFinishWA),
  })

  // Show QR code
  showChallenge(challenge.id)

  // Wait for the other side to sign and upload the response
  while (true) {
    await new Promise(resolve => setTimeout(resolve, 250))
    // Finish payload 404 = submitted
    // TODO: cross-confirmation? So delegatee submits downloaded signature
    try {
      await fetchApiJson(`/pair/delegated/${encodeURIComponent(challenge.id)}/finish_payload`)
    } catch (e) {
      break
    }
  }
}

async function pair(showChallenge: (id: string) => void) {
  // Get a challenge
  let challenge: PairingChallenge = await fetchApiJson('/pair/get_challenge', { method: 'POST' })
  if (challenge.isInitial) {
    await startInitialPair(challenge)
  } else {
    await startDelegatedPair(challenge, showChallenge)
  }
}

async function addDevice() {
  // Ask for challenge ID
  let challengeId = prompt('Enter challenge ID:')!

  // Download finish payload
  let payload: PairFinishWA = await fetchApiJson(`/pair/delegated/${encodeURIComponent(challengeId)}/finish_payload`)

  // Sign it
  let credential = await navigator.credentials.get({
    publicKey: {
      // Wrapper to avoid out-of-context replay
      challenge: toBytes(JSON.stringify({
        finishPayload: JSON.stringify(payload),
        expiresAt: Date.now() + (14 * 24 * 60 * 60 * 1000), // TODO
        allowedEntities: null,
      } as Delegation)),
      rpId: 'localhost',
      allowCredentials: [{
        type: 'public-key',
        id: base64.decode(getCredentialId()),
      }],
      timeout: 60000,
      userVerification: 'required', // critical action
    },
  }) as PublicKeyCredential
  console.log(credential)

  // Finish (no cross-confirmation for now)
  let credResp = credential.response as AuthenticatorAssertionResponse
  await fetchApiJson(`/webauthn/pair/delegated/${encodeURIComponent(challengeId)}/finish`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      delegationKeyId: base64.encode(credential.rawId), // pre-encoded id is base64url
      signature: base64.encode(credResp.signature),
      clientDataJSON: base64.encode(credResp.clientDataJSON),
      authenticatorData: base64.encode(credResp.authenticatorData),
    } as DelegatedPairFinishWA),
  })
}

function App() {
  let { data: entities } = useSWR<Entity[]>('/entity', fetchApiJson)
  let [isPairing, setIsPairing] = useState(false)
  let [challengeId, setChallengeId] = useState<string | null>(null)

  return <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'start', gap: 16, padding: 16 }}>
    {entities?.map(ent =>
      <Button key={ent.id} onClick={() => unlock(ent)}>Unlock '{ent.name}'</Button>
    )}
    {!entities && <Loader />}

    <Divider />

    <Button onClick={async () => {
      setIsPairing(true)
      await pair(setChallengeId)
      setIsPairing(false)
      setChallengeId(null)
    }}>Pair</Button>
    <Button onClick={addDevice}>Add device</Button>

    <Divider />

    {isPairing && <>
      <Text>Pairing...</Text>
      <Loader />

      <Divider />

      {challengeId && <>
        <Text>Challenge ID:</Text>
        <Text size='lg'>{challengeId}</Text>
        <div style={{ backgroundColor: 'white', padding: 32 }}>
          <QRCode value={JSON.stringify({
            challenge: challengeId,
          })} />
        </div>
      </>}
    </>}
  </div>
}

export default App;
