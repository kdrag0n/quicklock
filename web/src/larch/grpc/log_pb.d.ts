import * as jspb from 'google-protobuf'



export class HintMsg extends jspb.Message {
  getXcoord(): Uint8Array | string;
  getXcoord_asU8(): Uint8Array;
  getXcoord_asB64(): string;
  setXcoord(value: Uint8Array | string): HintMsg;

  getAuthXcoord(): Uint8Array | string;
  getAuthXcoord_asU8(): Uint8Array;
  getAuthXcoord_asB64(): string;
  setAuthXcoord(value: Uint8Array | string): HintMsg;

  getR(): Uint8Array | string;
  getR_asU8(): Uint8Array;
  getR_asB64(): string;
  setR(value: Uint8Array | string): HintMsg;

  getAuthR(): Uint8Array | string;
  getAuthR_asU8(): Uint8Array;
  getAuthR_asB64(): string;
  setAuthR(value: Uint8Array | string): HintMsg;

  getA(): Uint8Array | string;
  getA_asU8(): Uint8Array;
  getA_asB64(): string;
  setA(value: Uint8Array | string): HintMsg;

  getB(): Uint8Array | string;
  getB_asU8(): Uint8Array;
  getB_asB64(): string;
  setB(value: Uint8Array | string): HintMsg;

  getC(): Uint8Array | string;
  getC_asU8(): Uint8Array;
  getC_asB64(): string;
  setC(value: Uint8Array | string): HintMsg;

  getF(): Uint8Array | string;
  getF_asU8(): Uint8Array;
  getF_asB64(): string;
  setF(value: Uint8Array | string): HintMsg;

  getG(): Uint8Array | string;
  getG_asU8(): Uint8Array;
  getG_asB64(): string;
  setG(value: Uint8Array | string): HintMsg;

  getH(): Uint8Array | string;
  getH_asU8(): Uint8Array;
  getH_asB64(): string;
  setH(value: Uint8Array | string): HintMsg;

  getAlpha(): Uint8Array | string;
  getAlpha_asU8(): Uint8Array;
  getAlpha_asB64(): string;
  setAlpha(value: Uint8Array | string): HintMsg;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): HintMsg.AsObject;
  static toObject(includeInstance: boolean, msg: HintMsg): HintMsg.AsObject;
  static serializeBinaryToWriter(message: HintMsg, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): HintMsg;
  static deserializeBinaryFromReader(message: HintMsg, reader: jspb.BinaryReader): HintMsg;
}

export namespace HintMsg {
  export type AsObject = {
    xcoord: Uint8Array | string,
    authXcoord: Uint8Array | string,
    r: Uint8Array | string,
    authR: Uint8Array | string,
    a: Uint8Array | string,
    b: Uint8Array | string,
    c: Uint8Array | string,
    f: Uint8Array | string,
    g: Uint8Array | string,
    h: Uint8Array | string,
    alpha: Uint8Array | string,
  }
}

export class InitRequest extends jspb.Message {
  getKeyComm(): Uint8Array | string;
  getKeyComm_asU8(): Uint8Array;
  getKeyComm_asB64(): string;
  setKeyComm(value: Uint8Array | string): InitRequest;

  getId(): number;
  setId(value: number): InitRequest;

  getAuthPk(): Uint8Array | string;
  getAuthPk_asU8(): Uint8Array;
  getAuthPk_asB64(): string;
  setAuthPk(value: Uint8Array | string): InitRequest;

  getLogSeed(): Uint8Array | string;
  getLogSeed_asU8(): Uint8Array;
  getLogSeed_asB64(): string;
  setLogSeed(value: Uint8Array | string): InitRequest;

  getHintsList(): Array<HintMsg>;
  setHintsList(value: Array<HintMsg>): InitRequest;
  clearHintsList(): InitRequest;
  addHints(value?: HintMsg, index?: number): HintMsg;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): InitRequest.AsObject;
  static toObject(includeInstance: boolean, msg: InitRequest): InitRequest.AsObject;
  static serializeBinaryToWriter(message: InitRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): InitRequest;
  static deserializeBinaryFromReader(message: InitRequest, reader: jspb.BinaryReader): InitRequest;
}

export namespace InitRequest {
  export type AsObject = {
    keyComm: Uint8Array | string,
    id: number,
    authPk: Uint8Array | string,
    logSeed: Uint8Array | string,
    hintsList: Array<HintMsg.AsObject>,
  }
}

export class InitResponse extends jspb.Message {
  getPk(): Uint8Array | string;
  getPk_asU8(): Uint8Array;
  getPk_asB64(): string;
  setPk(value: Uint8Array | string): InitResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): InitResponse.AsObject;
  static toObject(includeInstance: boolean, msg: InitResponse): InitResponse.AsObject;
  static serializeBinaryToWriter(message: InitResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): InitResponse;
  static deserializeBinaryFromReader(message: InitResponse, reader: jspb.BinaryReader): InitResponse;
}

export namespace InitResponse {
  export type AsObject = {
    pk: Uint8Array | string,
  }
}

export class RegRequest extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RegRequest.AsObject;
  static toObject(includeInstance: boolean, msg: RegRequest): RegRequest.AsObject;
  static serializeBinaryToWriter(message: RegRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RegRequest;
  static deserializeBinaryFromReader(message: RegRequest, reader: jspb.BinaryReader): RegRequest;
}

export namespace RegRequest {
  export type AsObject = {
  }
}

export class RegResponse extends jspb.Message {
  getPkX(): Uint8Array | string;
  getPkX_asU8(): Uint8Array;
  getPkX_asB64(): string;
  setPkX(value: Uint8Array | string): RegResponse;

  getPkY(): Uint8Array | string;
  getPkY_asU8(): Uint8Array;
  getPkY_asB64(): string;
  setPkY(value: Uint8Array | string): RegResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RegResponse.AsObject;
  static toObject(includeInstance: boolean, msg: RegResponse): RegResponse.AsObject;
  static serializeBinaryToWriter(message: RegResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RegResponse;
  static deserializeBinaryFromReader(message: RegResponse, reader: jspb.BinaryReader): RegResponse;
}

export namespace RegResponse {
  export type AsObject = {
    pkX: Uint8Array | string,
    pkY: Uint8Array | string,
  }
}

export class AuthRequest extends jspb.Message {
  getProofList(): Array<Uint8Array | string>;
  setProofList(value: Array<Uint8Array | string>): AuthRequest;
  clearProofList(): AuthRequest;
  addProof(value: Uint8Array | string, index?: number): AuthRequest;

  getChallenge(): Uint8Array | string;
  getChallenge_asU8(): Uint8Array;
  getChallenge_asB64(): string;
  setChallenge(value: Uint8Array | string): AuthRequest;

  getCt(): Uint8Array | string;
  getCt_asU8(): Uint8Array;
  getCt_asB64(): string;
  setCt(value: Uint8Array | string): AuthRequest;

  getIv(): Uint8Array | string;
  getIv_asU8(): Uint8Array;
  getIv_asB64(): string;
  setIv(value: Uint8Array | string): AuthRequest;

  getDigest(): Uint8Array | string;
  getDigest_asU8(): Uint8Array;
  getDigest_asB64(): string;
  setDigest(value: Uint8Array | string): AuthRequest;

  getD(): Uint8Array | string;
  getD_asU8(): Uint8Array;
  getD_asB64(): string;
  setD(value: Uint8Array | string): AuthRequest;

  getE(): Uint8Array | string;
  getE_asU8(): Uint8Array;
  getE_asB64(): string;
  setE(value: Uint8Array | string): AuthRequest;

  getId(): number;
  setId(value: number): AuthRequest;

  getTag(): Uint8Array | string;
  getTag_asU8(): Uint8Array;
  getTag_asB64(): string;
  setTag(value: Uint8Array | string): AuthRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AuthRequest.AsObject;
  static toObject(includeInstance: boolean, msg: AuthRequest): AuthRequest.AsObject;
  static serializeBinaryToWriter(message: AuthRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AuthRequest;
  static deserializeBinaryFromReader(message: AuthRequest, reader: jspb.BinaryReader): AuthRequest;
}

export namespace AuthRequest {
  export type AsObject = {
    proofList: Array<Uint8Array | string>,
    challenge: Uint8Array | string,
    ct: Uint8Array | string,
    iv: Uint8Array | string,
    digest: Uint8Array | string,
    d: Uint8Array | string,
    e: Uint8Array | string,
    id: number,
    tag: Uint8Array | string,
  }
}

export class AuthResponse extends jspb.Message {
  getProd(): Uint8Array | string;
  getProd_asU8(): Uint8Array;
  getProd_asB64(): string;
  setProd(value: Uint8Array | string): AuthResponse;

  getD(): Uint8Array | string;
  getD_asU8(): Uint8Array;
  getD_asB64(): string;
  setD(value: Uint8Array | string): AuthResponse;

  getE(): Uint8Array | string;
  getE_asU8(): Uint8Array;
  getE_asB64(): string;
  setE(value: Uint8Array | string): AuthResponse;

  getSessionCtr(): number;
  setSessionCtr(value: number): AuthResponse;

  getCmCheckD(): Uint8Array | string;
  getCmCheckD_asU8(): Uint8Array;
  getCmCheckD_asB64(): string;
  setCmCheckD(value: Uint8Array | string): AuthResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AuthResponse.AsObject;
  static toObject(includeInstance: boolean, msg: AuthResponse): AuthResponse.AsObject;
  static serializeBinaryToWriter(message: AuthResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AuthResponse;
  static deserializeBinaryFromReader(message: AuthResponse, reader: jspb.BinaryReader): AuthResponse;
}

export namespace AuthResponse {
  export type AsObject = {
    prod: Uint8Array | string,
    d: Uint8Array | string,
    e: Uint8Array | string,
    sessionCtr: number,
    cmCheckD: Uint8Array | string,
  }
}

export class AuthCheckRequest extends jspb.Message {
  getCmCheckD(): Uint8Array | string;
  getCmCheckD_asU8(): Uint8Array;
  getCmCheckD_asB64(): string;
  setCmCheckD(value: Uint8Array | string): AuthCheckRequest;

  getSessionCtr(): number;
  setSessionCtr(value: number): AuthCheckRequest;

  getId(): number;
  setId(value: number): AuthCheckRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AuthCheckRequest.AsObject;
  static toObject(includeInstance: boolean, msg: AuthCheckRequest): AuthCheckRequest.AsObject;
  static serializeBinaryToWriter(message: AuthCheckRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AuthCheckRequest;
  static deserializeBinaryFromReader(message: AuthCheckRequest, reader: jspb.BinaryReader): AuthCheckRequest;
}

export namespace AuthCheckRequest {
  export type AsObject = {
    cmCheckD: Uint8Array | string,
    sessionCtr: number,
    id: number,
  }
}

export class AuthCheckResponse extends jspb.Message {
  getCheckD(): Uint8Array | string;
  getCheckD_asU8(): Uint8Array;
  getCheckD_asB64(): string;
  setCheckD(value: Uint8Array | string): AuthCheckResponse;

  getCheckDOpen(): Uint8Array | string;
  getCheckDOpen_asU8(): Uint8Array;
  getCheckDOpen_asB64(): string;
  setCheckDOpen(value: Uint8Array | string): AuthCheckResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AuthCheckResponse.AsObject;
  static toObject(includeInstance: boolean, msg: AuthCheckResponse): AuthCheckResponse.AsObject;
  static serializeBinaryToWriter(message: AuthCheckResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AuthCheckResponse;
  static deserializeBinaryFromReader(message: AuthCheckResponse, reader: jspb.BinaryReader): AuthCheckResponse;
}

export namespace AuthCheckResponse {
  export type AsObject = {
    checkD: Uint8Array | string,
    checkDOpen: Uint8Array | string,
  }
}

export class AuthCheck2Request extends jspb.Message {
  getCheckD(): Uint8Array | string;
  getCheckD_asU8(): Uint8Array;
  getCheckD_asB64(): string;
  setCheckD(value: Uint8Array | string): AuthCheck2Request;

  getCheckDOpen(): Uint8Array | string;
  getCheckDOpen_asU8(): Uint8Array;
  getCheckDOpen_asB64(): string;
  setCheckDOpen(value: Uint8Array | string): AuthCheck2Request;

  getSessionCtr(): number;
  setSessionCtr(value: number): AuthCheck2Request;

  getId(): number;
  setId(value: number): AuthCheck2Request;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AuthCheck2Request.AsObject;
  static toObject(includeInstance: boolean, msg: AuthCheck2Request): AuthCheck2Request.AsObject;
  static serializeBinaryToWriter(message: AuthCheck2Request, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AuthCheck2Request;
  static deserializeBinaryFromReader(message: AuthCheck2Request, reader: jspb.BinaryReader): AuthCheck2Request;
}

export namespace AuthCheck2Request {
  export type AsObject = {
    checkD: Uint8Array | string,
    checkDOpen: Uint8Array | string,
    sessionCtr: number,
    id: number,
  }
}

export class AuthCheck2Response extends jspb.Message {
  getOut(): Uint8Array | string;
  getOut_asU8(): Uint8Array;
  getOut_asB64(): string;
  setOut(value: Uint8Array | string): AuthCheck2Response;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AuthCheck2Response.AsObject;
  static toObject(includeInstance: boolean, msg: AuthCheck2Response): AuthCheck2Response.AsObject;
  static serializeBinaryToWriter(message: AuthCheck2Response, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AuthCheck2Response;
  static deserializeBinaryFromReader(message: AuthCheck2Response, reader: jspb.BinaryReader): AuthCheck2Response;
}

export namespace AuthCheck2Response {
  export type AsObject = {
    out: Uint8Array | string,
  }
}

