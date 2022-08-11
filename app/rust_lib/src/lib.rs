use std::ffi::c_void;
use jni::{JNIEnv, JavaVM};

use android_logger::Config;
use bls_signatures::{Serialize};
use jni::objects::{JClass, JString};
use jni_sys::{jbyteArray, jint, JNI_VERSION_1_6, jstring};
use log::Level;
use qlock::bls::aggregate_sigs_multi;
use qlock::envelope::RequestEnvelope;


#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_blsGeneratePrivateKey(env: JNIEnv, _: JClass) -> jbyteArray {
    let seed: [u8; 32] = rand::random();
    let sk = bls_signatures::PrivateKey::new(seed);
    let sk_data = sk.as_bytes();

    env.byte_array_from_slice(&sk_data).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_blsDerivePublicKey(env: JNIEnv, _: JClass, sk_in: jbyteArray) -> jbyteArray {
    let sk_data = env.convert_byte_array(sk_in).unwrap();
    let sk = bls_signatures::PrivateKey::from_bytes(&sk_data).unwrap();

    let pk = sk.public_key();
    env.byte_array_from_slice(&pk.as_bytes()).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_blsSignMessage(
    env: JNIEnv,
    _: JClass,
    sk_in: jbyteArray,
    msg_in: jbyteArray,
) -> jbyteArray {
    let sk_data = env.convert_byte_array(sk_in).unwrap();
    let sk = bls_signatures::PrivateKey::from_bytes(&sk_data).unwrap();

    let sig = sk.sign(&env.convert_byte_array(msg_in).unwrap());
    env.byte_array_from_slice(&sig.as_bytes()).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_blsAggregateSigs(
    env: JNIEnv,
    _: JClass,
    pk1_in: jbyteArray,
    sig1_in: jbyteArray,
    pk2_in: jbyteArray,
    sig2_in: jbyteArray,
) -> jbyteArray {
    let pk1_data = env.convert_byte_array(pk1_in).unwrap();
    let pk1 = bls_signatures::PublicKey::from_bytes(&pk1_data).unwrap();
    let sig1_data = env.convert_byte_array(sig1_in).unwrap();
    let sig1 = bls_signatures::Signature::from_bytes(&sig1_data).unwrap();

    let pk2_data = env.convert_byte_array(pk2_in).unwrap();
    let pk2 = bls_signatures::PublicKey::from_bytes(&pk2_data).unwrap();
    let sig2_data = env.convert_byte_array(sig2_in).unwrap();
    let sig2 = bls_signatures::Signature::from_bytes(&sig2_data).unwrap();

    let agg_sig = aggregate_sigs_multi(&[
        (&sig1, &pk1),
        (&sig2, &pk2),
    ]);
    env.byte_array_from_slice(&agg_sig.as_bytes()).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_envelopeSeal(
    env: JNIEnv,
    _: JClass,
    key_in: jbyteArray,
    msg_in: JString,
) -> jstring {
    let key_data = env.convert_byte_array(key_in).unwrap();
    let msg: String = env.get_string(msg_in).unwrap().into();
    let envelope = RequestEnvelope::seal_raw(&msg, &key_data).unwrap();

    env.new_string(envelope.serialize()).unwrap().into_inner()
}

#[no_mangle]
pub extern "system" fn JNI_OnLoad(_: JavaVM, _: *const c_void) -> jint {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Debug)
            .with_tag("RustLib"),
    );
    log_panics::init();

    JNI_VERSION_1_6
}
