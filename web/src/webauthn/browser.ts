import { WebAuthenticator } from "./types"

export class BrowserWebAuthenticator implements WebAuthenticator {
  async create(options: CredentialCreationOptions) {
    let cred = await navigator.credentials.create(options)
    if (cred === null) {
      throw new Error('Credential is null')
    }

    return cred as PublicKeyCredential
  }

  async get(options: CredentialRequestOptions) {
    let cred = await navigator.credentials.get(options)
    if (cred === null) {
      throw new Error('Credential is null')
    }
    return cred as PublicKeyCredential
  }
}
