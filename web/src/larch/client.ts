import { bytesToHex, decodeB64, rawStringToBytes } from "../bytes"
import { LogServiceProxy } from "./remote"
import { NativeAuthenticateResult, NativeRegisterResult } from "./types"

interface LarchClientModule extends EmscriptenModule {
  logServiceProxy: LogServiceProxy

  FS: {
    mkdirTree(path: string): void
  }

  Client__Create(readFromStorage: boolean): number
  Client__Initialize(ptr: number): Promise<number>
  Client__Register(ptr: number, appId: string, challenge: string): Promise<NativeRegisterResult>
  Client__Authenticate(ptr: number, appId: string, challenge: string, keyHandle: string): Promise<NativeAuthenticateResult>
  Client__WriteToStorage(ptr: number): void
}

type LarchClientFactory = EmscriptenModuleFactory<LarchClientModule>

declare global {
  interface Window {
    createLarchClient: LarchClientFactory
  }
}

function bytesToString(bytes: Uint8Array): string {
  let string = ''
  for (let i = 0; i < bytes.length; i++) {
    string += String.fromCharCode(bytes[i])
  }
  return string
}

export class LarchClient {
  private readonly ptr: number
  private initialized: boolean

  private constructor(private readonly module: LarchClientModule) {
    module.FS.mkdirTree('/home/dragon/code/crypto/larch-wasm/out')
    this.ptr = module.Client__Create(false)
    console.log('ptr', this.ptr)
    this.initialized = false
  }

  static async create() {
    let proxy = new LogServiceProxy()
    let module = await window.createLarchClient({
      logServiceProxy: proxy,

      print(text: string) {
        console.log('[wasm]', text)
      }
    })
    return new LarchClient(module)
  }

  async init() {
    console.log('init')
    await this.module.Client__Initialize(this.ptr)
    this.initialized = true
  }

  async register(appId: Uint8Array, challenge: Uint8Array) {
    if (!this.initialized) {
      await this.init()
    }

    let appIdStr = bytesToString(appId)
    let challengeStr = bytesToString(challenge)

    console.log('register', appIdStr, challengeStr)
    let result = await this.module.Client__Register(this.ptr, appIdStr, challengeStr)
    console.log('register done', result)
    await this.writeToStorage()
  
    return {
      keyHandle: decodeB64(result.key_handle),
      publicKey: {
        x: decodeB64(result.pk_x),
        y: decodeB64(result.pk_y),
      },
      certificate: decodeB64(result.cert),
      signature: decodeB64(result.sig),
    }
  }

  async authenticate(appId: Uint8Array, challenge: Uint8Array, keyHandle: Uint8Array) {
    if (!this.initialized) {
      await this.init()
    }

    let appIdStr = bytesToString(appId)
    let challengeStr = bytesToString(challenge)
    let keyHandleStr = bytesToString(keyHandle)

    console.log('authenticate', appIdStr, challengeStr, keyHandleStr)
    let result = await this.module.Client__Authenticate(this.ptr, appIdStr, challengeStr, keyHandleStr)
    await this.writeToStorage()

    return {
      flags: result.flags,
      counter: result.counter,
      signature: decodeB64(result.sig),
    }
  }

  private async writeToStorage() {
    if (!this.initialized) {
      await this.init()
    }

    console.log('writeToStorage')
    this.module.Client__WriteToStorage(this.ptr)
  }

  // TODO: finalizer, free client
}
