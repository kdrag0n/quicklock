#!/usr/bin/env bash

emcmake cmake . \
    -DCMAKE_PREFIX_PATH=/home/dragon/code/crypto/prefix-emptool \
    -DCMAKE_BUILD_TYPE=Release \
    -DOPENSSL_ROOT_DIR=/home/dragon/code/crypto/prefix-openssl-threads \
    -DOPENSSL_INCLUDE_DIR=/home/dragon/code/crypto/prefix-openssl-threads/include \
    -DOPENSSL_CRYPTO_LIBRARY=/home/dragon/code/crypto/prefix-openssl-threads/lib/libcrypto.a \
    -DOPENSSL_SSL_LIBRARY=/home/dragon/code/crypto/prefix-openssl-threads/lib/libssl.a \
    -DCMAKE_FOLDER=/home/dragon/code/crypto/prefix-emptool \
    -Demp-tool_DIR=/home/dragon/code/crypto/prefix-emptool/cmake \
    -DEMP-TOOL_INCLUDE_DIR=/home/dragon/code/crypto/prefix-emptool/include \
    -DEMP-TOOL_LIBRARY=/home/dragon/code/crypto/prefix-emptool/lib/libemp-tool.so
emmake make -j48 client_lib

    # -DCMAKE_C_FLAGS="-O3 -sASYNCIFY -sPROXY_TO_PTHREAD -sWASM_BIGINT -msimd128 -msse -msse2 -msse3 -mssse3 -msse4.1 -msse4.2 -mfpu=neon" \
    # -DCMAKE_CXX_FLAGS="-O3 -sASYNCIFY -sPROXY_TO_PTHREAD -sWASM_BIGINT -msimd128 -msse -msse2 -msse3 -mssse3 -msse4.1 -msse4.2 -mfpu=neon" \
    # -DCMAKE_EXE_LINKER_FLAGS="-O3 -sASYNCIFY -sPROXY_TO_PTHREAD -sWASM_BIGINT -lembind -sNODERAWFS" && emmake make -j48

# -sASYNCIFY -s ASYNCIFY_STACK_SIZE=10000 unneeded stack size
# -sPROXY_TO_PTHREAD breaks embind object serialization
# -g --profiling for debug stack trace
# i
# ENVIRONMENT
# -flto
