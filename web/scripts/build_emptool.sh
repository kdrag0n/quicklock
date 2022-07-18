#!/usr/bin/env bash

emcmake cmake . -DCMAKE_INSTALL_PREFIX=/home/dragon/code/crypto/prefix-emptool -DCRYPTO_IN_CIRCUIT=true -DTHREADING=ON
emmake make -j48
emmake make install
