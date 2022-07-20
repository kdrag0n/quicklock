import { LarchClient } from "../larch/client"
import { BrowserWebAuthenticator } from "./browser"
import { WebAuthenticator } from "./types"
import * as base64 from 'base64-arraybuffer'
import { bytesToHex, encodeBase64Url, stringToBytes } from "../bytes"
import * as CBOR from 'cbor-x'

export class LarchWebAuthenticator implements WebAuthenticator {
  private constructor(
    private readonly client: LarchClient,
    // Local authenticator impl (for signing the client share)
    private readonly localAuthenticator: WebAuthenticator,
  ) {}

  static async create() {
    let client = await LarchClient.create()
    return new LarchWebAuthenticator(client, new BrowserWebAuthenticator())
  }

  async create(options: CredentialCreationOptions): Promise<PublicKeyCredential> {
    let pkOptions = options.publicKey!

    // TODO: use local authenticator
    let clientData = {
      type: 'webauthn.create',
      challenge: encodeBase64Url(pkOptions.challenge as Uint8Array),
      origin: window.location.origin,
      crossOrigin: false,
    }
    let clientDataJson = JSON.stringify(clientData)
    let clientDataBytes = stringToBytes(clientDataJson)
    let clientDataHash = new Uint8Array(await crypto.subtle.digest('SHA-256', clientDataBytes))
    console.log('clientData', clientData)
    console.log('clientDataJson', clientDataJson)
    console.log('clientDataHash', bytesToHex(clientDataHash))

    let rpId = pkOptions.rp.id! // TODO: fallback
    let rpIdHash = new Uint8Array(await crypto.subtle.digest('SHA-256', stringToBytes(rpId)))
    console.log('rpId', rpId)
    console.log('rpIdHash', bytesToHex(rpIdHash))

    // U2F CTAP1 compat
    // https://fidoalliance.org/specs/fido-v2.1-rd-20210309/fido-client-to-authenticator-protocol-v2.1-rd-20210309.html#u2f-authenticatorGetAssertion-interoperability
    let challenge = clientDataHash
    let appId = rpIdHash
    let result = await this.client.register(appId, challenge)
    console.log('U2F result', result)

    // Compat: map back to CTAP2
    let credentialIdLength = result.keyHandle.length
    let credentialId = result.keyHandle
    console.log('credentialIdLength', credentialIdLength)
    console.log('credentialId', bytesToHex(credentialId))
    // COSE_Key: https://datatracker.ietf.org/doc/html/rfc8152#section-7.1
    let credentialPublicKeyData = {
      1: 2, // kty = EC
      3: -7, // alg = ES256
      [-1]: 1, // crv = P-256
      [-2]: result.publicKey.x, // x
      [-3]: result.publicKey.y, // y
    }
    console.log('credentialPublicKeyData', credentialPublicKeyData)
    console.log('pk x', base64.encode(result.publicKey.x))
    console.log('pk y', base64.encode(result.publicKey.y))
    let credentialPublicKey = CBOR.encode(credentialPublicKeyData)
    console.log('credentialPublicKey', base64.encode(credentialPublicKey))
    // should be 77 according to CTAP1 compat doc, but it doesn't actually matter
    console.log('credentialPublicKey length', credentialPublicKey.length)

    let attestedCredData = new Uint8Array(16 + 2 + credentialIdLength + credentialPublicKey.length)
    let attestedView = new DataView(attestedCredData.buffer)
    // aaguid 16: all zeros
    attestedView.setUint16(16, credentialIdLength, false)
    attestedCredData.set(credentialId, 18)
    attestedCredData.set(credentialPublicKey, 18 + credentialIdLength)

    // TODO: endian?
    let authenticatorData = new Uint8Array(32 + 1 + 4 + attestedCredData.length)
    let authDataView = new DataView(authenticatorData.buffer)
    // rpIdHash 32
    authenticatorData.set(rpIdHash, 0)
    // flags: set bits 0 and 6
    let flags = (1 << 0) | (1 << 6)
    authDataView.setUint8(32, flags)
    // signCount
    let signCount = 0 // init
    authDataView.setUint32(33, signCount, false)
    // attestedCredData
    authenticatorData.set(attestedCredData, 37)

    // Don't bother to do attestation
    let attestationObjectData = {
      authData: authenticatorData,
      fmt: 'none',
      attStmt: {},
    }
    console.log('attestationObjectData', attestationObjectData)
    let attestationObject = CBOR.encode(attestationObjectData)
    console.log('attestationObject', base64.encode(attestationObject))
    
    return {
      type: 'public-key',
      id: encodeBase64Url(credentialId),
      rawId: credentialId,
      response: {
        clientDataJSON: clientDataBytes,
        attestationObject: attestationObject,
      } as AuthenticatorAttestationResponse,

      getClientExtensionResults() {
          return {}
      },
    }
  }

  async get(options: CredentialRequestOptions): Promise<PublicKeyCredential> {
    let pkOptions = options.publicKey!
    let credentialId = pkOptions.allowCredentials![0].id as Uint8Array // TODO: check type

    // TODO: use local authenticator
    let clientData = {
      type: 'webauthn.get',
      challenge: encodeBase64Url(pkOptions.challenge as Uint8Array),
      origin: window.location.origin,
      crossOrigin: false,
    }
    let clientDataJson = JSON.stringify(clientData)
    let clientDataBytes = stringToBytes(clientDataJson)
    let clientDataHash = new Uint8Array(await crypto.subtle.digest('SHA-256', clientDataBytes))
    console.log('clientData', clientData)
    console.log('clientDataJson', clientDataJson)
    console.log('clientDataHash', bytesToHex(clientDataHash))

    let rpId = pkOptions.rpId! // TODO: fallback
    let rpIdHash = new Uint8Array(await crypto.subtle.digest('SHA-256', stringToBytes(rpId)))
    console.log('rpId', rpId)
    console.log('rpIdHash', bytesToHex(rpIdHash))

    // U2F CTAP1 compat
    // https://fidoalliance.org/specs/fido-v2.1-rd-20210309/fido-client-to-authenticator-protocol-v2.1-rd-20210309.html#u2f-authenticatorGetAssertion-interoperability
    let challenge = clientDataHash
    let appId = rpIdHash
    let keyHandle = credentialId
    let result = await this.client.authenticate(appId, challenge, keyHandle)
    console.log('U2F result', result)

    // Compat: map back to CTAP2
    let authenticatorData = new Uint8Array(32 + 1 + 4)
    let authDataView = new DataView(authenticatorData.buffer)
    // rpIdHash 32
    authenticatorData.set(rpIdHash, 0)
    // flags (larch sets bit 0 for user presence)
    let flags = result.flags & 0b11 // copy bits 0 and 1, set others to zero
    authDataView.setUint8(32, flags)
    // signCount
    authDataView.setUint32(33, result.counter, false)
    
    return {
      type: 'public-key',
      id: encodeBase64Url(credentialId),
      rawId: credentialId,
      response: {
        authenticatorData,
        clientDataJSON: stringToBytes(clientDataJson),
        signature: result.signature,
        userHandle: null,
      } as AuthenticatorAssertionResponse,

      getClientExtensionResults() {
          return {}
      },
    }
  }
}
