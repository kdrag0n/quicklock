import { rawStringToBytes } from "../bytes"
import { LogClient } from "./grpc/LogServiceClientPb"
import { AuthCheck2Request, AuthCheckRequest, AuthRequest, HintMsg, InitRequest, RegRequest } from "./grpc/log_pb"
import { NativeAuthCheck2Request, NativeAuthCheck2Response, NativeAuthCheckRequest, NativeAuthCheckResponse, NativeAuthRequest, NativeAuthResponse, NativeInitRequest, NativeInitResponse, NativeRegResponse, NativeVector } from "./types"

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
    req.setKeyComm(rawStringToBytes(native.key_comm))
    req.setId(native.id)
    req.setAuthPk(rawStringToBytes(native.auth_pk))
    req.setLogSeed(rawStringToBytes(native.log_seed))
    req.setHintsList(vecToArray(native.hints).map(nativeMsg => {
      let msg = new HintMsg()
      msg.setXcoord(rawStringToBytes(nativeMsg.xcoord))
      msg.setAuthXcoord(rawStringToBytes(nativeMsg.auth_xcoord))
      msg.setR(rawStringToBytes(nativeMsg.r))
      msg.setAuthR(rawStringToBytes(nativeMsg.auth_r))
      msg.setA(rawStringToBytes(nativeMsg.a))
      msg.setB(rawStringToBytes(nativeMsg.b))
      msg.setC(rawStringToBytes(nativeMsg.c))
      msg.setF(rawStringToBytes(nativeMsg.f))
      msg.setG(rawStringToBytes(nativeMsg.g))
      msg.setH(rawStringToBytes(nativeMsg.h))
      msg.setAlpha(rawStringToBytes(nativeMsg.alpha))
      return msg
    }))

    console.log('init', req)
    let resp = await this.service.sendInit(req, null)
    console.log('init done', resp)
    return {
      pk: resp.getPk(),
    }
  }

  async reg(): Promise<NativeRegResponse> {
    let req = new RegRequest()
    let resp = await this.service.sendReg(req, null)
    return {
      pk_x: resp.getPkX(),
      pk_y: resp.getPkY(),
    }
  }

  async auth(native: NativeAuthRequest): Promise<NativeAuthResponse> {
    let req = new AuthRequest()
    req.setProofList(vecToArray(native.proofs).map(rawStringToBytes))
    req.setChallenge(rawStringToBytes(native.challenge))
    req.setCt(rawStringToBytes(native.ct))
    req.setIv(rawStringToBytes(native.iv))
    req.setDigest(rawStringToBytes(native.digest))
    req.setD(rawStringToBytes(native.d))
    req.setE(rawStringToBytes(native.e))
    req.setId(native.id)
    req.setTag(rawStringToBytes(native.tag))

    let resp = await this.service.sendAuth(req, null)
    return {
      prod: resp.getProd(),
      d: resp.getD(),
      e: resp.getE(),
      session_ctr: resp.getSessionCtr(),
      cm_check_d: resp.getCmCheckD(),
    }
  }

  async authCheck(native: NativeAuthCheckRequest): Promise<NativeAuthCheckResponse> {
    let req = new AuthCheckRequest()
    req.setCmCheckD(native.cm_check_d)
    req.setSessionCtr(native.session_ctr)
    req.setId(native.id)

    let resp = await this.service.sendAuthCheck(req, null)
    return {
      check_d: resp.getCheckD(),
      check_d_open: resp.getCheckDOpen(),
    }
  }

  async authCheck2(native: NativeAuthCheck2Request): Promise<NativeAuthCheck2Response> {
    let req = new AuthCheck2Request()
    req.setCheckD(native.check_d)
    req.setCheckDOpen(native.check_d_open)
    req.setSessionCtr(native.session_ctr)
    req.setId(native.id)

    let resp = await this.service.sendAuthCheck2(req, null)
    return {
      out: resp.getOut(),
    }
  }
}
