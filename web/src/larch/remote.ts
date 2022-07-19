import { decodeB64, rawStringToBytes } from "../bytes"
import { LogClient } from "./grpc/LogServiceClientPb"
import { AuthCheck2Request, AuthCheckRequest, AuthRequest, HintMsg, InitRequest, RegRequest } from "./grpc/log_pb"
import { NativeAuthCheck2Request, NativeAuthCheck2Response, NativeAuthCheckRequest, NativeAuthCheckResponse, NativeAuthRequest, NativeAuthResponse, NativeInitRequest, NativeInitResponse, NativeRegResponse, NativeVector } from "./types"
import * as base64 from 'base64-arraybuffer'

const SERVICE_URL = 'http://localhost:10000' // envoy proxy to 12345

function vecToArray<T>(vector: NativeVector<T>): T[] {
  let size = vector.size()
  let arr = new Array<T>(size)
  for (let i = 0; i < size; i++) {
    arr[i] = vector.get(i)
  }
  return arr
}

export class LogServiceProxy {
  private readonly service: LogClient

  constructor() {
    this.service = new LogClient(SERVICE_URL)
  }

  async init(native: NativeInitRequest): Promise<NativeInitResponse> {
    let req = new InitRequest()
    req.setKeyComm(decodeB64(native.key_comm))
    req.setId(native.id)
    req.setAuthPk(decodeB64(native.auth_pk))
    req.setLogSeed(decodeB64(native.log_seed))
    req.setHintsList(vecToArray(native.hints).map(nativeMsg => {
      let msg = new HintMsg()
      msg.setXcoord(decodeB64(nativeMsg.xcoord))
      msg.setAuthXcoord(decodeB64(nativeMsg.auth_xcoord))
      msg.setR(decodeB64(nativeMsg.r))
      msg.setAuthR(decodeB64(nativeMsg.auth_r))
      msg.setA(decodeB64(nativeMsg.a))
      msg.setB(decodeB64(nativeMsg.b))
      msg.setC(decodeB64(nativeMsg.c))
      msg.setF(decodeB64(nativeMsg.f))
      msg.setG(decodeB64(nativeMsg.g))
      msg.setH(decodeB64(nativeMsg.h))
      msg.setAlpha(decodeB64(nativeMsg.alpha))
      return msg
    }))

    console.log('init', req)
    let resp = await this.service.sendInit(req, null)
    console.log('init done', resp)
    return {
      pk: base64.encode(resp.getPk() as Uint8Array),
    }
  }

  async reg(): Promise<NativeRegResponse> {
    let req = new RegRequest()
    let resp = await this.service.sendReg(req, null)
    return {
      pk_x: base64.encode(resp.getPkX() as Uint8Array),
      pk_y: base64.encode(resp.getPkY() as Uint8Array),
    }
  }

  async auth(native: NativeAuthRequest): Promise<NativeAuthResponse> {
    let req = new AuthRequest()
    req.setProofList(vecToArray(native.proofs).map(decodeB64))
    req.setChallenge(decodeB64(native.challenge))
    req.setCt(decodeB64(native.ct))
    req.setIv(decodeB64(native.iv))
    req.setDigest(decodeB64(native.digest))
    req.setD(decodeB64(native.d))
    req.setE(decodeB64(native.e))
    req.setId(native.id)
    req.setTag(decodeB64(native.tag))

    let resp = await this.service.sendAuth(req, null)
    return {
      prod: base64.encode(resp.getProd() as Uint8Array),
      d: base64.encode(resp.getD() as Uint8Array),
      e: base64.encode(resp.getE() as Uint8Array),
      session_ctr: resp.getSessionCtr(),
      cm_check_d: base64.encode(resp.getCmCheckD() as Uint8Array),
    }
  }

  async authCheck(native: NativeAuthCheckRequest): Promise<NativeAuthCheckResponse> {
    let req = new AuthCheckRequest()
    req.setCmCheckD(decodeB64(native.cm_check_d))
    req.setSessionCtr(native.session_ctr)
    req.setId(native.id)

    let resp = await this.service.sendAuthCheck(req, null)
    return {
      check_d: base64.encode(resp.getCheckD() as Uint8Array),
      check_d_open: base64.encode(resp.getCheckDOpen() as Uint8Array),
    }
  }

  async authCheck2(native: NativeAuthCheck2Request): Promise<NativeAuthCheck2Response> {
    let req = new AuthCheck2Request()
    req.setCheckD(decodeB64(native.check_d))
    req.setCheckDOpen(decodeB64(native.check_d_open))
    req.setSessionCtr(native.session_ctr)
    req.setId(native.id)

    let resp = await this.service.sendAuthCheck2(req, null)
    return {
      out: base64.encode(resp.getOut() as Uint8Array),
    }
  }
}
