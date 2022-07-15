#!/usr/bin/env bash

cd "$(dirname "$0")"

pushd "src/larch/grpc"
protoc -I=. log.proto \
  --js_out=import_style=commonjs,binary:. \
  --grpc-web_out=import_style=typescript,mode=grpcweb:.
