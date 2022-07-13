import React, { useEffect } from 'react';
import logo from './logo.svg';
import './App.css';
import { Button, Divider, Loader } from '@mantine/core';
import useSWR from 'swr';
import * as base64 from 'base64-arraybuffer'

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

interface UnlockStartWA {
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

function toBytes(str: string) {
  return Uint8Array.from(str, c => c.charCodeAt(0))
}

function getCredentialId() {
  return localStorage.credentialId! as string
}

async function unlock(entity: Entity) {
  // Get challenge with info
  let challenge = await fetchApiJson('/webauthn/unlock/start', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ 
      entityId: entity.id, 
    } as UnlockStartWA),
  })

  // Sign
  console.log('get cred')
  let credential = await navigator.credentials.get({
    publicKey: {
      challenge: toBytes(JSON.stringify(challenge)),
      rpId: 'localhost',
      allowCredentials: [{
        type: 'public-key',
        id: base64.decode(getCredentialId()),
      }],
      timeout: 60000,
    }
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
      pubKeyCredParams: [{ alg: -7, type: 'public-key' }],
      timeout: 60000,
      attestation: 'none',
    }
  }) as PublicKeyCredential
  console.log(credential)
  localStorage.credentialId = base64.encode(credential.rawId) // pre-encoded id is base64url

  // Finish
  let credResp = credential.response as AuthenticatorAttestationResponse
  let resp = await fetchApiJson(`/webauthn/pair/initial/${encodeURIComponent(challenge.id)}/finish`, {
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

  return resp
}

// Delegatee
async function startDelegatedPair(challenge: PairingChallenge) {

}

async function pair() {
  // Get a challenge
  let challenge: PairingChallenge = await fetchApiJson('/pair/get_challenge', { method: 'POST' })
  if (challenge.isInitial) {
    await startInitialPair(challenge)
  } else {
    await startDelegatedPair(challenge)
  }
}

function App() {
  let { data: entities } = useSWR<Entity[]>('/entity', fetchApiJson)

  return <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'start', gap: 16, padding: 16 }}>
    {entities?.map(ent =>
      <Button key={ent.id} onClick={() => unlock(ent)}>Unlock '{ent.name}'</Button>
    )}
    {!entities && <Loader />}

    <Divider />

    <Button onClick={pair}>Pair</Button>
    <Button>Add device</Button>
  </div>
}

export default App;
