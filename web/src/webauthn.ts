import * as CBOR from 'cbor-x'
import * as base64 from 'base64-arraybuffer'
import * as base1024 from './base1024'
import { PairFinishWA } from './api-types'
import { bytesToBuffer } from './bytes'

interface AttestationObject {
  fmt: string
  attStmt: any
  authData: Uint8Array
}

export async function extractFinishPublicKey({ attestationObject }: PairFinishWA) {
  let { authData } = CBOR.decode(new Uint8Array(base64.decode(attestationObject))) as AttestationObject

  let rpIdHash = authData.subarray(0, 32)

  let view = new DataView(bytesToBuffer(authData.subarray(32)))
  let flags = view.getUint8(0)
  let attestedCredDataIncluded = (flags & (1 << 6)) !== 0
  let signCount = view.getUint32(1)

  console.log(authData)
  console.log(rpIdHash, flags)
  if (!attestedCredDataIncluded) {
    throw new Error('No attested credential data')
  }

  let attestedCredData = authData.subarray(32 + 1 + 4)
  let attestedView = new DataView(bytesToBuffer(attestedCredData))
  let credentialIdLength = attestedView.getUint16(16)
  let credentialId = attestedCredData.subarray(18, 18 + credentialIdLength)
  let credentialPublicKey = attestedCredData.subarray(18 + credentialIdLength)

  // credentialPublicKey is in COSE format. Just use it as the key data.
  return credentialPublicKey
}

export async function publicKeyToEmoji(key: Uint8Array) {
  let hash = await crypto.subtle.digest('SHA-256', key)
  return base1024.encode(new Uint8Array(hash))
}
