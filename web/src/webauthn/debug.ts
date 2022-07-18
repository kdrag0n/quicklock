import * as base64 from 'base64-arraybuffer'
import { bytesToBuffer, bytesToHex, bytesToString, decodeBase64Url } from "../bytes"
import * as CBOR from 'cbor-x'

export function printCreate(cred: PublicKeyCredential) {
  console.log('credential id', bytesToHex(new Uint8Array(cred.rawId)))

  let pk = cred.response as AuthenticatorAttestationResponse
  console.log('raw', pk)

  let clientDataJson = bytesToString(pk.clientDataJSON)
  console.log('clientDataJson', clientDataJson)
  let clientData = JSON.parse(clientDataJson)
  console.log('clientData', clientData)
  let challenge = bytesToString(decodeBase64Url(clientData.challenge))
  console.log('challenge', challenge)

  let attestation = CBOR.decode(new Uint8Array(pk.attestationObject))
  console.log('attestation', attestation)
  let fmt = attestation.fmt
  console.log('fmt', fmt)
  let attStmt = attestation.attStmt
  console.log('attStmt', attStmt)
  let authData: Uint8Array = attestation.authData
  console.log('authData', base64.encode(authData))

  let authDataView = new DataView(bytesToBuffer(authData))
  let rpIdHash = authData.subarray(0, 32)
  console.log('rpIdHash', bytesToHex(rpIdHash))
  let flags = authDataView.getUint8(32)
  console.log('flags', flags.toString(2))
  let signCount = authDataView.getUint32(33, false)
  console.log('signCount', signCount)
  let attestedCredData = authData.subarray(37)
  console.log('attestedCredData', base64.encode(attestedCredData))

  let attestedCredDataView = new DataView(bytesToBuffer(attestedCredData))
  let aaguid = attestedCredData.subarray(0, 16)
  console.log('aaguid', bytesToHex(aaguid))
  let credentialIdLength = attestedCredDataView.getUint16(16, false)
  console.log('credentialIdLength', credentialIdLength)
  let credentialId = attestedCredData.subarray(18, 18 + credentialIdLength)
  console.log('credentialId', bytesToHex(credentialId))
  let credentialPublicKey = attestedCredData.subarray(18 + credentialIdLength)
  console.log('credentialPublicKey', base64.encode(credentialPublicKey))

  let cpk = CBOR.decode(credentialPublicKey)
  console.log('cpk', cpk)
  let x = cpk.x
  console.log('x', base64.encode(x))
  let y = cpk.y
  console.log('y', base64.encode(y))
}
