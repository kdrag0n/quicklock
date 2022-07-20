#!/usr/bin/env bash

CFLAGS="-sMEMORY64=2 -pthread" CXXFLAGS="-sMEMORY64=2 -pthread" LDFLAGS="-sMEMORY64=2 -pthread" \
emconfigure ./Configure \
    gcc -no-tests -no-asm -static -no-sock -no-afalgeng -DOPENSSL_SYS_NETWARE -DSIG_DFL=0 -DSIG_IGN=0 -DHAVE_FORK=0 -DOPENSSL_NO_AFALGENG=1 \
    --prefix=$HOME/crypto/prefix-openssl-wasm64 --openssldir=$HOME/crypto/prefix-openssl-wasm64 \
    --with-rand-seed=devrandom

sed -i 's|^CROSS_COMPILE.*$|CROSS_COMPILE=|g' Makefile
emmake make -j48
emmake make install
cp apps/openssl.wasm ~/code/crypto/prefix-openssl-wasm64/bin
