# emp-tool
cmake . -DCMAKE_INSTALL_PREFIX=$HOME/code/crypto/prefix-emptool-host -DCRYPTO_IN_CIRCUIT=true -DTHREADING=ON
make -j48
make install

# larch
cmake . \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_PREFIX_PATH=$HOME/code/crypto/prefix-emptool-host \
    -DCMAKE_FOLDER=$HOME/code/crypto/prefix-emptool-host \
    -DOPENSSL_ROOT_DIR=$HOME/code/crypto/prefix-openssl-host-mdebug
make -j48
