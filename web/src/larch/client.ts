import { bytesToHex, decodeB64, encodeB64, rawStringToBytes } from "../bytes"
import { LogServiceProxy } from "./remote"
import { NativeAuthenticateResult, NativeRegisterResult } from "./types"

interface LarchClientModule extends EmscriptenModule {
  logServiceProxy: LogServiceProxy

  // MODULARIZE = no global fs
  FS: {
    mkdirTree(path: string): void
    writeFile(path: string, data: string): void
  }

  Client__Create(readFromStorage: boolean): number
  Client__Initialize(ptr: number): Promise<number>
  Client__Register(ptr: number, appId: string, challenge: string): Promise<NativeRegisterResult>
  Client__Authenticate(ptr: number, appId: string, challenge: string, keyHandle: string, lastCounter: number): Promise<NativeAuthenticateResult>
  Client__WriteToStorage(ptr: number): void
}

type LarchClientFactory = EmscriptenModuleFactory<LarchClientModule>

declare global {
  interface Window {
    createLarchClient: LarchClientFactory
  }
}

export class LarchClient {
  private readonly ptr: number
  private initialized: boolean

  private constructor(private readonly module: LarchClientModule, sha256Circuit: string) {
    // For client state
    module.FS.mkdirTree('/home/dragon/code/crypto/larch-wasm/out')

    // Write SHA-256 ZKBoo circuit
    module.FS.mkdirTree('/home/dragon/code/crypto/larch-wasm/zkboo/circuit_files')
    module.FS.writeFile('/home/dragon/code/crypto/larch-wasm/zkboo/circuit_files/sha-256-multiblock-aligned.txt', sha256Circuit)

    this.ptr = module.Client__Create(false)
    console.log('ptr', this.ptr)
    this.initialized = false
  }

  static async create() {
    let proxy = new LogServiceProxy()

    // Load WASM module
    let module = await window.createLarchClient({
      logServiceProxy: proxy,

      print(text: string) {
        console.log('[wasm]', text)
      }
    })

    // Load SHA-256 ZKBoo circuit
    let resp = await fetch('/sha-256-multiblock-aligned.txt')
    let sha256Circuit = await resp.text()

    return new LarchClient(module, sha256Circuit)
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

    let appIdStr = encodeB64(appId)
    let challengeStr = encodeB64(challenge)

    console.log('register', appIdStr, challengeStr)
    let result = await this.module.Client__Register(this.ptr, appIdStr, challengeStr)
    console.log('register done', result)
    await this.writeToStorage()
    localStorage.larch__lastCounter = 0
  
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

    let appIdStr = encodeB64(appId)
    let challengeStr = encodeB64(challenge)
    let keyHandleStr = encodeB64(keyHandle)
    let lastCounter = parseInt(localStorage.larch__lastCounter ?? '0')

    console.log('authenticate', appIdStr, challengeStr, keyHandleStr, lastCounter)
    let result = await this.module.Client__Authenticate(this.ptr, appIdStr, challengeStr, keyHandleStr, lastCounter)
    await this.writeToStorage()
    localStorage.larch__lastCounter = result.counter

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
