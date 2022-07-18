export interface NativeVector<T> {
  size(): number
  get(index: number): T
}

export interface NativeHintMsg {
  xcoord: string
  auth_xcoord: string
  r: string
  auth_r: string
  a: string
  b: string
  c: string
  f: string
  g: string
  h: string
  alpha: string
}

export interface NativeInitRequest {
  key_comm: string
  id: number
  auth_pk: string
  log_seed: string
  hints: NativeVector<NativeHintMsg>
}

export interface NativeInitResponse {
  pk: string | Uint8Array
}

export interface NativeRegResponse {
  pk_x: string | Uint8Array
  pk_y: string | Uint8Array
}

export interface NativeAuthRequest {
  proofs: NativeVector<string>
  challenge: string
  ct: string
  iv: string
  digest: string
  d: string
  e: string
  id: number
  tag: string
}

export interface NativeAuthResponse {
  prod: string | Uint8Array
  d: string | Uint8Array
  e: string | Uint8Array
  session_ctr: number
  cm_check_d: string | Uint8Array
}

export interface NativeAuthCheckRequest {
  cm_check_d: string
  session_ctr: number
  id: number
}

export interface NativeAuthCheckResponse {
  check_d: string | Uint8Array
  check_d_open: string | Uint8Array
}

export interface NativeAuthCheck2Request {
  check_d: string
  check_d_open: string
  session_ctr: number
  id: number
}

export interface NativeAuthCheck2Response {
  out: string | Uint8Array
}

/*
* Client
*/
export interface NativeRegisterResult {
  key_handle: string
  // Uncompressed P-256 point
  pk_x: string
  pk_y: string
  cert: string
  sig: string
}

export interface NativeAuthenticateResult {
  flags: number
  counter: number
  sig: string
}


/*
 * JS client
 */
export interface EcKeyPoint {
  x: Uint8Array
  y: Uint8Array
}

export interface RegisterResult {
  keyHandle: Uint8Array
  publicKey: EcKeyPoint
  certificate: Uint8Array
  signature: Uint8Array
}

export interface AuthenticateResult {
  flags: number
  counter: number
  signature: Uint8Array
}

