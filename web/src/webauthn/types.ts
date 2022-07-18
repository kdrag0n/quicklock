export interface WebAuthenticator {
  create(options: CredentialCreationOptions): Promise<PublicKeyCredential>
  get(options: CredentialRequestOptions): Promise<PublicKeyCredential>
}
