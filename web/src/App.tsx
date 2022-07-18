import React, { useEffect, useState } from 'react'
import logo from './logo.svg'
import './App.css'
import { Button, Checkbox, Chip, Chips, Divider, Loader, Modal, Text } from '@mantine/core'
import useSWR from 'swr'
import * as base64 from 'base64-arraybuffer'
import QRCode from 'react-qr-code'
import { DelegatedPairFinishWA, Delegation, Entity, InitialPairFinishRequest, PairFinishChallengeWA, PairFinishWA, PairingChallenge, UnlockFinishWA, UnlockStartRequest } from './api-types'
import { extractFinishPublicKey, publicKeyToEmoji } from './webauthn'
import { DatePicker } from '@mantine/dates'
import { stringToBytes } from './bytes'
import { BrowserWebAuthenticator } from './webauthn/browser'
import { LarchWebAuthenticator } from './webauthn/larch'
import { printCreate } from './webauthn/debug'

const API_BASE_URL = 'http://localhost:3002/api'

async function fetchApiJson(path: string, init?: RequestInit) {
  let resp = await fetch(API_BASE_URL + path, init)
  if (!resp.ok) {
    throw new Error(`${resp.status} ${resp.statusText}`)
  }
  return await resp.json()
}

export interface DelegatedPairState {
  challengeId: string
  keyEmoji: string
}

export interface DelegationState {
  challengeId: string
  keyEmoji: string
  finishRequest: PairFinishWA
}

let authenticator = new BrowserWebAuthenticator()

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
  let credential = await authenticator.get({
    publicKey: {
      challenge: stringToBytes(JSON.stringify(challenge)),
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

  // Sign response
  let credential = await authenticator.create({
    publicKey: {
      // Use WebAuthn as an envelope for the challenge ID
      challenge: stringToBytes(JSON.stringify({
        pairChallengeId: challenge.id,
      } as PairFinishChallengeWA)),
      rp: {
        id: 'localhost',
        name: 'Main',
      },
      user: {
        // Doesn't matter. We use PK as the identifier after registration
        id: stringToBytes(crypto.randomUUID()),
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
  printCreate(credential)
  localStorage.credentialId = base64.encode(credential.rawId) // pre-encoded id is base64url

  // Prepare finish request
  let credResp = credential.response as AuthenticatorAttestationResponse
  let finishPayload = JSON.stringify({
    keyId: base64.encode(credential.rawId), // pre-encoded id is base64url
    attestationObject: base64.encode(credResp.attestationObject),
    clientDataJSON: base64.encode(credResp.clientDataJSON),
  } as PairFinishWA)

  // HMAC to prove knowledge of secret
  let key = await crypto.subtle.importKey('raw', secretBytes, { name: 'HMAC', hash: 'SHA-256' }, false, ['sign'])
  let mac = await crypto.subtle.sign('HMAC', key, stringToBytes(finishPayload))

  // Finish
  await fetchApiJson(`/webauthn/pair/initial/${encodeURIComponent(challenge.id)}/finish`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      finishPayload,
      mac: base64.encode(mac),
    } as InitialPairFinishRequest),
  })
}

// Delegatee
async function startDelegatedPair(
  challenge: PairingChallenge,
  setDelegatedState: (state: DelegatedPairState) => void,
) {
  // Sign finish response
  let credential = await authenticator.create({
    publicKey: {
      // Use WebAuthn as an envelope for the challenge ID
      challenge: stringToBytes(JSON.stringify({
        pairChallengeId: challenge.id,
      } as PairFinishChallengeWA)),
      rp: {
        id: 'localhost',
        name: 'Main',
      },
      user: {
        // Doesn't matter. We use PK as the identifier after registration
        id: stringToBytes(crypto.randomUUID()),
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
  printCreate(credential)
  localStorage.credentialId = base64.encode(credential.rawId) // pre-encoded id is base64url

  // Upload for cross-signing
  let credResp = credential.response as AuthenticatorAttestationResponse
  let finishRequest: PairFinishWA = {
    keyId: base64.encode(credential.rawId), // pre-encoded id is base64url
    attestationObject: base64.encode(credResp.attestationObject),
    clientDataJSON: base64.encode(credResp.clientDataJSON),
  }
  await fetchApiJson(`/pair/delegated/${encodeURIComponent(challenge.id)}/finish_payload`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(finishRequest),
  })

  // Show QR code and emoji
  let publicKey = await extractFinishPublicKey(finishRequest)
  let keyEmoji = await publicKeyToEmoji(publicKey)
  setDelegatedState({
    challengeId: challenge.id,
    keyEmoji,
  })

  // Wait for the other side to sign and upload the response
  while (true) {
    await new Promise(resolve => setTimeout(resolve, 250))
    // Finish payload 404 = submitted
    try {
      await fetchApiJson(`/pair/delegated/${encodeURIComponent(challenge.id)}/finish_payload`)
    } catch (e) {
      break
    }
  }
}

async function pair(setDelegatedState: (state: DelegatedPairState) => void) {
  // Get a challenge
  let challenge: PairingChallenge = await fetchApiJson('/pair/get_challenge', { method: 'POST' })
  if (challenge.isInitial) {
    await startInitialPair(challenge)
  } else {
    await startDelegatedPair(challenge, setDelegatedState)
  }
}

async function startCrossSign(setDelegationState: (state: DelegationState) => void) {
  // Ask for challenge ID
  let challengeId = prompt('Enter challenge ID:')!

  // Download finish payload
  let finishRequest: PairFinishWA = await fetchApiJson(`/pair/delegated/${encodeURIComponent(challengeId)}/finish_payload`)

  let publicKey = await extractFinishPublicKey(finishRequest)
  let keyEmoji = await publicKeyToEmoji(publicKey)

  setDelegationState({
    challengeId,
    finishRequest,
    keyEmoji,
  })
}

async function finishCrossSign(
  { challengeId, finishRequest }: DelegationState,
  expiresAt: number,
  allowedEntities: string[] | null,
  setDelegationState: (state: DelegationState | null) => void,
) {
  // Sign it
  let credential = await authenticator.get({
    publicKey: {
      // Wrapper to avoid out-of-context replay
      challenge: stringToBytes(JSON.stringify({
        finishPayload: JSON.stringify(finishRequest),
        expiresAt,
        allowedEntities,
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

  // Finish
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

  setDelegationState(null)
}

function DelegationConfirmContent({ state, setState, entities }: {
  state: DelegationState
  entities: Entity[]
  setState: (state: DelegationState | null) => void
}) {
  let [useExpiry, setUseExpiry] = useState(false)
  let [limitEntities, setLimitEntities] = useState(false)
  let [expiresAt, setExpiresAt] = useState(new Date(Date.now() + (7 * 24 * 60 * 60 * 1000)))
  let [allowedEntities, setAllowedEntities] = useState<string[]>(entities.map(e => e.id))

  return <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'start' }}>
    <Text>Make sure these emoji are the same on your new device.</Text>
    <Text style={{ fontSize: 64, marginBottom: 16 }}>{state.keyEmoji}</Text>

    <div style={{ marginBottom: 16 }}>
      <Checkbox
        label='Limit access time'
        checked={useExpiry}
        onChange={event => setUseExpiry(event.currentTarget.checked)} />

      {useExpiry && <DatePicker
        value={expiresAt}
        onChange={date => setExpiresAt(date!)}
        label='Allow access until'
        clearable={false}
        minDate={new Date()}
      />}
    </div>

    <div style={{ marginBottom: 16 }}>
      <Checkbox
        label='Limit locks'
        checked={limitEntities}
        onChange={event => setLimitEntities(event.currentTarget.checked)} />
        
        {limitEntities && <Chips multiple value={allowedEntities} onChange={setAllowedEntities}>
          {entities.map(e => <Chip key={e.id} value={e.id}>{e.name}</Chip>)}
        </Chips>}
    </div>

    <Button onClick={() => {
      finishCrossSign(
        state,
        useExpiry ? expiresAt.getTime() : 8640000000000000,
        limitEntities ? allowedEntities : null,
        setState,
      )
    }}>Confirm</Button>
  </div>
}

function App() {
  let { data: entities } = useSWR<Entity[]>('/entity', fetchApiJson)
  let [isPairing, setIsPairing] = useState(false)
  let [delegatedState, setDelegatedState] = useState<DelegatedPairState | null>(null)
  let [delegationState, setDelegationState] = useState<DelegationState | null>(null)
  let [useLarch, setUseLarch] = useState(false)

  useEffect(() => {
    (async function() {
      let newAuth = useLarch ? await LarchWebAuthenticator.create() : new BrowserWebAuthenticator()
      console.log('authenticator:', authenticator, '->', newAuth)
      authenticator = newAuth
    })()
  }, [useLarch])

  return <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'start', gap: 16, padding: 16 }}>
    {entities?.map(ent =>
      <Button key={ent.id} onClick={() => unlock(ent)}>Unlock '{ent.name}'</Button>
    )}
    {!entities && <Loader />}

    <Divider />

    <Button onClick={async () => {
      setIsPairing(true)
      await pair(setDelegatedState)
      setIsPairing(false)
      setDelegatedState(null)
    }}>Pair</Button>
    <Button onClick={() => startCrossSign(setDelegationState)}>Add device</Button>

    <Divider />

    <Checkbox
      label='Use Larch'
      checked={useLarch}
      onChange={ev => setUseLarch(ev.currentTarget.checked)} />

    <Divider />

    {isPairing && <>
      <Text>Pairing...</Text>
      <Loader />

      <Divider />

      {delegatedState && <>
        <Text>Challenge ID:</Text>
        <Text size='lg'>{delegatedState.challengeId}</Text>
        
        <div style={{ backgroundColor: 'white', padding: 32 }}>
          <QRCode value={JSON.stringify({
            challenge: delegatedState.challengeId,
          })} />
        </div>
        
        <Text>Make sure these emoji are the same on your old device.</Text>
        <Text style={{ fontSize: 64 }}>{delegatedState.keyEmoji}</Text>
      </>}
    </>}

    <Modal
      opened={delegationState !== null}
      onClose={() => setDelegationState(null)}
      title='Confirm device'
    >
      {delegationState && entities && <DelegationConfirmContent
        entities={entities}
        state={delegationState}
        setState={setDelegationState} />}
    </Modal>
  </div>
}

export default App
