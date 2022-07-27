#include <jni.h>
#include "bls.hpp"
#include <vector>

extern "C" {

JNIEXPORT jbyteArray JNICALL
Java_dev_kdrag0n_quicklock_NativeLib_blsGeneratePrivateKey(JNIEnv *env, jclass clazz, jbyteArray seed) {
    auto seed_ptr = env->GetByteArrayElements(seed, nullptr);
    bls::Bytes seed_bytes((uint8_t*) seed_ptr, env->GetArrayLength(seed));

    auto sk = bls::BasicSchemeMPL().KeyGen(seed_bytes);
    auto sk_data = sk.Serialize();

    auto sk_out = env->NewByteArray((jsize) sk_data.size());
    auto sk_ptr = env->GetByteArrayElements(sk_out, nullptr);
    memcpy(sk_ptr, sk_data.data(), sk_data.size());

    env->ReleaseByteArrayElements(sk_out, sk_ptr, 0);
    env->ReleaseByteArrayElements(seed, seed_ptr, JNI_ABORT);
    return sk_out;
}

JNIEXPORT jbyteArray JNICALL
Java_dev_kdrag0n_quicklock_NativeLib_blsSignMessage(JNIEnv *env, jclass clazz, jbyteArray sk_data,
                                                    jbyteArray message) {
    auto sk_ptr = env->GetByteArrayElements(sk_data, nullptr);
    bls::Bytes sk_bytes((uint8_t*) sk_ptr, env->GetArrayLength(sk_data));

    auto sk = bls::PrivateKey::FromBytes(sk_bytes);
    auto msg_data = env->GetByteArrayElements(message, nullptr);

    std::vector<uint8_t> sig_buf(96);
    std::vector<uint8_t> msg_vec(msg_data, msg_data + env->GetArrayLength(message));
    auto sig = bls::BasicSchemeMPL().Sign(sk, msg_vec);
    auto sig_data = sig.Serialize();

    auto sig_out = env->NewByteArray((jsize) sig_data.size());
    auto sig_ptr = env->GetByteArrayElements(sig_out, nullptr);
    memcpy(sig_ptr, sig_data.data(), sig_data.size());

    env->ReleaseByteArrayElements(sig_out, sig_ptr, 0);
    env->ReleaseByteArrayElements(sk_data, sk_ptr, JNI_ABORT);
    env->ReleaseByteArrayElements(message, msg_data, JNI_ABORT);
    return sig_out;
}

JNIEXPORT jbyteArray JNICALL
Java_dev_kdrag0n_quicklock_NativeLib_blsDerivePublicKey(JNIEnv *env, jclass clazz,
                                                        jbyteArray sk_in) {
    auto sk_ptr = env->GetByteArrayElements(sk_in, nullptr);
    bls::Bytes sk_bytes((uint8_t*) sk_ptr, env->GetArrayLength(sk_in));

    auto sk = bls::PrivateKey::FromBytes(sk_bytes);
    auto pk = sk.GetG1Element();
    auto pk_data = pk.Serialize();

    auto pk_out = env->NewByteArray((jsize) pk_data.size());
    auto pk_ptr = env->GetByteArrayElements(pk_out, nullptr);
    memcpy(pk_ptr, pk_data.data(), pk_data.size());

    env->ReleaseByteArrayElements(pk_out, pk_ptr, 0);
    env->ReleaseByteArrayElements(sk_in, sk_ptr, JNI_ABORT);
    return pk_out;
}

}
