import { bytesToHex, decodeB64, encodeB64, rawStringToBytes } from "../bytes"
import { LogServiceProxy } from "./remote"
import { NativeAuthenticateResult, NativeRegisterResult } from "./types"

interface LarchClientModule extends EmscriptenModule {
  logServiceProxy: LogServiceProxy

  // MODULARIZE = no global fs
  IDBFS: Emscripten.FileSystemType
  FS: {
    mkdir(path: string): void
    mkdirTree(path: string): void
    writeFile(path: string, data: string): void
    mount(type: Emscripten.FileSystemType, opts: any, mountpoint: string): void
    syncfs(populate: boolean, callback: (e: any) => any): void
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

  private constructor(private readonly module: LarchClientModule) {
    // For client state
    module.FS.mkdir('/data')
    module.FS.mount(module.IDBFS, {}, '/data')

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

    return new LarchClient(module)
  }

  syncFs(populate: boolean) {
    return new Promise<void>((resolve, reject) => {
      this.module.FS.syncfs(populate, err => {
        if (err) {
          reject(err)
        } else {
          resolve()
        }
      })
    })
  }

  async init() {
    console.log('load storage')
    await this.syncFs(true)

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
    await this.syncFs(false)
  }

  // TODO: finalizer, free client
}
