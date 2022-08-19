mod server;

use std::ffi::c_void;
use std::io::Read;
use std::net::SocketAddr;
use std::ptr::null;
use std::str::FromStr;
use jni::{JNIEnv, JavaVM};

use android_logger::Config;
use axum::body::HttpBody;
use axum::Json;
use axum::response::IntoResponse;
use bls_signatures::{Serialize};
use jni::objects::{JClass, JObject, JString};
use jni_sys::{jbyteArray, jint, JNI_VERSION_1_6, jstring};
use log::Level;
use qlock::bls::aggregate_sigs_multi;
use qlock::crypto::{hash, hash_id_short};
use qlock::envelope::RequestEnvelope;
use qlock::lock::actions::{finish_unlock, start_unlock};
use qlock::lock::model::UnlockStartRequest;


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
    msg_in: jbyteArray,
) -> jstring {
    let key_data = env.convert_byte_array(key_in).unwrap();
    let msg = env.convert_byte_array(msg_in).unwrap();
    let envelope = RequestEnvelope::seal_raw(&msg, &key_data).unwrap();

    env.new_string(envelope.serialize()).unwrap().into_inner()
}

#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_hash(
    env: JNIEnv,
    _: JClass,
    msg_in: jbyteArray,
) -> jbyteArray {
    let msg = env.convert_byte_array(msg_in).unwrap();
    let hash = hash(&msg);

    env.byte_array_from_slice(&hash).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_hashIdShort(
    env: JNIEnv,
    _: JClass,
    msg_in: jbyteArray,
) -> jbyteArray {
    let msg = env.convert_byte_array(msg_in).unwrap();
    let hash = hash_id_short(&msg);

    env.byte_array_from_slice(&hash).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_startServer(
    _: JNIEnv,
    _: JClass,
) {
    server::start_bg();
}

#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_serverStartUnlock(
    env: JNIEnv,
    _: JClass,
    entity_id: JString,
) -> jstring {
    start_unlock(UnlockStartRequest {
        entity_id: env.get_string(entity_id).unwrap().into(),
    }).map_or(JObject::null().into_inner(), |resp| {
        env.new_string(&serde_json::to_string(&resp).unwrap()).unwrap().into_inner()
    })
}

#[no_mangle]
pub extern "system" fn Java_dev_kdrag0n_quicklock_NativeLib_serverFinishUnlock(
    env: JNIEnv,
    _: JClass,
    envelope_json: JString,
    id: JString,
) -> jstring {
    let json: String = env.get_string(envelope_json).unwrap().into();
    let envelope = serde_json::from_str(&json).unwrap();
    let id = env.get_string(id).unwrap().into();

    finish_unlock(envelope, id, SocketAddr::from_str("127.0.0.1:8080").unwrap())
        .map_or(JObject::null().into_inner(), |resp| {
            env.new_string(&serde_json::to_string(&resp).unwrap()).unwrap().into_inner()
        })
}

#[no_mangle]
pub extern "system" fn JNI_OnLoad(_: JavaVM, _: *const c_void) -> jint {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Info)
            .with_tag("RustLib"),
    );
    log_panics::init();

    JNI_VERSION_1_6
}
