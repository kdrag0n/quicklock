# emp-tool
cmake . -DCMAKE_INSTALL_PREFIX=$HOME/code/crypto/prefix-emptool-host -DCRYPTO_IN_CIRCUIT=true -DTHREADING=ON
make -j48
make install

# larch
cmake . \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_PREFIX_PATH=/home/dragon/code/crypto/prefix-emptool-host \
    -DCMAKE_FOLDER=/home/dragon/code/crypto/prefix-emptool-host
make -j48
