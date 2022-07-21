#!/usr/bin/env bash

CFLAGS="-pthread" CXXFLAGS="-pthread" LDFLAGS="-pthread" \
emconfigure ./Configure \
    gcc -no-tests -no-asm -static -no-sock -no-afalgeng -enable-threads -DOPENSSL_SYS_NETWARE -DSIG_DFL=0 -DSIG_IGN=0 -DHAVE_FORK=0 -DOPENSSL_NO_AFALGENG=1 -pthread -D_THREAD_SAFE -D_REENTRANT \
    --prefix=$HOME/code/crypto/prefix-openssl-threads --openssldir=$HOME/code/crypto/prefix-openssl-threads \
    --with-rand-seed=devrandom

sed -i 's|^CROSS_COMPILE.*$|CROSS_COMPILE=|g' Makefile
emmake make -j48
emmake make install
cp apps/openssl.wasm ~/code/crypto/prefix-openssl-threads/bin
