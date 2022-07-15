/**
 * @fileoverview gRPC-Web generated client stub for 
 * @enhanceable
 * @public
 */

// GENERATED CODE -- DO NOT EDIT!


/* eslint-disable */
// @ts-nocheck


import * as grpcWeb from 'grpc-web';

import * as log_pb from './log_pb';


export class LogClient {
  client_: grpcWeb.AbstractClientBase;
  hostname_: string;
  credentials_: null | { [index: string]: string; };
  options_: null | { [index: string]: any; };

  constructor (hostname: string,
               credentials?: null | { [index: string]: string; },
               options?: null | { [index: string]: any; }) {
    if (!options) options = {};
    if (!credentials) credentials = {};
    options['format'] = 'binary';

    this.client_ = new grpcWeb.GrpcWebClientBase(options);
    this.hostname_ = hostname;
    this.credentials_ = credentials;
    this.options_ = options;
  }

  methodDescriptorSendInit = new grpcWeb.MethodDescriptor(
    '/Log/SendInit',
    grpcWeb.MethodType.UNARY,
    log_pb.InitRequest,
    log_pb.InitResponse,
    (request: log_pb.InitRequest) => {
      return request.serializeBinary();
    },
    log_pb.InitResponse.deserializeBinary
  );

  sendInit(
    request: log_pb.InitRequest,
    metadata: grpcWeb.Metadata | null): Promise<log_pb.InitResponse>;

  sendInit(
    request: log_pb.InitRequest,
    metadata: grpcWeb.Metadata | null,
    callback: (err: grpcWeb.RpcError,
               response: log_pb.InitResponse) => void): grpcWeb.ClientReadableStream<log_pb.InitResponse>;

  sendInit(
    request: log_pb.InitRequest,
    metadata: grpcWeb.Metadata | null,
    callback?: (err: grpcWeb.RpcError,
               response: log_pb.InitResponse) => void) {
    if (callback !== undefined) {
      return this.client_.rpcCall(
        this.hostname_ +
          '/Log/SendInit',
        request,
        metadata || {},
        this.methodDescriptorSendInit,
        callback);
    }
    return this.client_.unaryCall(
    this.hostname_ +
      '/Log/SendInit',
    request,
    metadata || {},
    this.methodDescriptorSendInit);
  }

  methodDescriptorSendReg = new grpcWeb.MethodDescriptor(
    '/Log/SendReg',
    grpcWeb.MethodType.UNARY,
    log_pb.RegRequest,
    log_pb.RegResponse,
    (request: log_pb.RegRequest) => {
      return request.serializeBinary();
    },
    log_pb.RegResponse.deserializeBinary
  );

  sendReg(
    request: log_pb.RegRequest,
    metadata: grpcWeb.Metadata | null): Promise<log_pb.RegResponse>;

  sendReg(
    request: log_pb.RegRequest,
    metadata: grpcWeb.Metadata | null,
    callback: (err: grpcWeb.RpcError,
               response: log_pb.RegResponse) => void): grpcWeb.ClientReadableStream<log_pb.RegResponse>;

  sendReg(
    request: log_pb.RegRequest,
    metadata: grpcWeb.Metadata | null,
    callback?: (err: grpcWeb.RpcError,
               response: log_pb.RegResponse) => void) {
    if (callback !== undefined) {
      return this.client_.rpcCall(
        this.hostname_ +
          '/Log/SendReg',
        request,
        metadata || {},
        this.methodDescriptorSendReg,
        callback);
    }
    return this.client_.unaryCall(
    this.hostname_ +
      '/Log/SendReg',
    request,
    metadata || {},
    this.methodDescriptorSendReg);
  }

  methodDescriptorSendAuth = new grpcWeb.MethodDescriptor(
    '/Log/SendAuth',
    grpcWeb.MethodType.UNARY,
    log_pb.AuthRequest,
    log_pb.AuthResponse,
    (request: log_pb.AuthRequest) => {
      return request.serializeBinary();
    },
    log_pb.AuthResponse.deserializeBinary
  );

  sendAuth(
    request: log_pb.AuthRequest,
    metadata: grpcWeb.Metadata | null): Promise<log_pb.AuthResponse>;

  sendAuth(
    request: log_pb.AuthRequest,
    metadata: grpcWeb.Metadata | null,
    callback: (err: grpcWeb.RpcError,
               response: log_pb.AuthResponse) => void): grpcWeb.ClientReadableStream<log_pb.AuthResponse>;

  sendAuth(
    request: log_pb.AuthRequest,
    metadata: grpcWeb.Metadata | null,
    callback?: (err: grpcWeb.RpcError,
               response: log_pb.AuthResponse) => void) {
    if (callback !== undefined) {
      return this.client_.rpcCall(
        this.hostname_ +
          '/Log/SendAuth',
        request,
        metadata || {},
        this.methodDescriptorSendAuth,
        callback);
    }
    return this.client_.unaryCall(
    this.hostname_ +
      '/Log/SendAuth',
    request,
    metadata || {},
    this.methodDescriptorSendAuth);
  }

  methodDescriptorSendAuthCheck = new grpcWeb.MethodDescriptor(
    '/Log/SendAuthCheck',
    grpcWeb.MethodType.UNARY,
    log_pb.AuthCheckRequest,
    log_pb.AuthCheckResponse,
    (request: log_pb.AuthCheckRequest) => {
      return request.serializeBinary();
    },
    log_pb.AuthCheckResponse.deserializeBinary
  );

  sendAuthCheck(
    request: log_pb.AuthCheckRequest,
    metadata: grpcWeb.Metadata | null): Promise<log_pb.AuthCheckResponse>;

  sendAuthCheck(
    request: log_pb.AuthCheckRequest,
    metadata: grpcWeb.Metadata | null,
    callback: (err: grpcWeb.RpcError,
               response: log_pb.AuthCheckResponse) => void): grpcWeb.ClientReadableStream<log_pb.AuthCheckResponse>;

  sendAuthCheck(
    request: log_pb.AuthCheckRequest,
    metadata: grpcWeb.Metadata | null,
    callback?: (err: grpcWeb.RpcError,
               response: log_pb.AuthCheckResponse) => void) {
    if (callback !== undefined) {
      return this.client_.rpcCall(
        this.hostname_ +
          '/Log/SendAuthCheck',
        request,
        metadata || {},
        this.methodDescriptorSendAuthCheck,
        callback);
    }
    return this.client_.unaryCall(
    this.hostname_ +
      '/Log/SendAuthCheck',
    request,
    metadata || {},
    this.methodDescriptorSendAuthCheck);
  }

  methodDescriptorSendAuthCheck2 = new grpcWeb.MethodDescriptor(
    '/Log/SendAuthCheck2',
    grpcWeb.MethodType.UNARY,
    log_pb.AuthCheck2Request,
    log_pb.AuthCheck2Response,
    (request: log_pb.AuthCheck2Request) => {
      return request.serializeBinary();
    },
    log_pb.AuthCheck2Response.deserializeBinary
  );

  sendAuthCheck2(
    request: log_pb.AuthCheck2Request,
    metadata: grpcWeb.Metadata | null): Promise<log_pb.AuthCheck2Response>;

  sendAuthCheck2(
    request: log_pb.AuthCheck2Request,
    metadata: grpcWeb.Metadata | null,
    callback: (err: grpcWeb.RpcError,
               response: log_pb.AuthCheck2Response) => void): grpcWeb.ClientReadableStream<log_pb.AuthCheck2Response>;

  sendAuthCheck2(
    request: log_pb.AuthCheck2Request,
    metadata: grpcWeb.Metadata | null,
    callback?: (err: grpcWeb.RpcError,
               response: log_pb.AuthCheck2Response) => void) {
    if (callback !== undefined) {
      return this.client_.rpcCall(
        this.hostname_ +
          '/Log/SendAuthCheck2',
        request,
        metadata || {},
        this.methodDescriptorSendAuthCheck2,
        callback);
    }
    return this.client_.unaryCall(
    this.hostname_ +
      '/Log/SendAuthCheck2',
    request,
    metadata || {},
    this.methodDescriptorSendAuthCheck2);
  }

}

