use bls_signatures::Serialize;
use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use reqwest::blocking::Client;
use sha2::Digest;
use qlock::audit::{RegisterRequest, RegisterResponse, SignRequest, SignResponse};
use qlock::audit::client::AuditClientState;
use qlock::audit::store::LogEvent;
use qlock::bls::{verify_multi};
use qlock::error::AppResult;

// Commitment scheme:
// hash(key + nonce) with 96-bit nonce
// Must be deterministic, so the nonce is generated at registration time
// TODO: nonce could be removed for perf
fn commit(key: &[u8; 32], nonce: &[u8; 12]) -> [u8; 32] {
    let mut buf = key.to_vec();
    buf.extend_from_slice(nonce);
    sha2::Sha256::digest(&buf).into()
}

fn main() -> AppResult<()> {
    let client = Client::new();

    // Generate BLS keypair
    let seed: [u8; 32] = rand::random();
    let bls_sk = bls_signatures::PrivateKey::new(seed);
    let bls_pk = bls_sk.public_key();

    // Generate symmetric key
    let enc_key: [u8; 32] = rand::random();
    let nonce: [u8; 12] = rand::random();
    let mut cipher = ChaCha20::new(&enc_key.into(), &nonce.into());

    // Generate commitment to key
    let enc_commit_nonce: [u8; 12] = rand::random();
    let enc_commit = commit(&enc_key, &enc_commit_nonce);

    // Register
    let bls_pk_str = base64::encode(&bls_pk.as_bytes());
    println!("Register: pk={}", bls_pk_str);
    let resp: RegisterResponse = client.post("http://localhost:9001/api/register")
        .json(&RegisterRequest {
            client_pk: bls_pk_str.clone(),
            client_msg_enc_commit: enc_commit.into(),
        })
        .send()?
        .error_for_status()?
        .json()?;
    let agg_pk = bls_signatures::PublicKey::from_bytes(&resp.aggregate_pk)?;
    println!("Register -> aggregate_pk={}", base64::encode(&resp.aggregate_pk));

    // Sign
    let msg: [u8; 32] = rand::random();
    let msg_hash = sha2::Sha256::digest(&msg);

    let mut msg_enc = msg;
    cipher.apply_keystream(&mut msg_enc);

    let client_sig = bls_sk.sign(&msg_hash);
    println!("Sign: msg={} hash={}", base64::encode(&msg), base64::encode(&msg_hash));
    let resp: SignResponse = client.post("http://localhost:9001/api/sign")
        .json(&SignRequest {
            client_pk: bls_pk_str.clone(),
            enc_message: msg_enc.into(),
            enc_nonce: nonce.into(),
            message_hash: msg_hash.to_vec(),
            client_sig: client_sig.as_bytes(),
        })
        .send()?
        .error_for_status()?
        .json()?;
    println!("Sign -> aggregate_sig={}", base64::encode(&resp.aggregate_sig));

    // Verify sig under both PKs
    println!("Verify agg sig");
    let agg_sig = bls_signatures::Signature::from_bytes(&resp.aggregate_sig)?;
    if !verify_multi(&agg_sig, &msg_hash, &agg_pk) {
        println!("Invalid signature");
        return Ok(());
    }

    // Get logs
    println!("Get logs");
    let logs: Vec<LogEvent> = client
        .get(format!("http://localhost:9001/api/device/{}/logs", urlencoding::encode(&bls_pk_str)))
        .send()?
        .error_for_status()?
        .json()?;
    println!("Logs: {:?}", logs);

    // Decrypt log message
    println!("Decrypt log message");
    let event = &logs[0];

    let mut dec_msg = event.enc_message.clone();
    let dec_nonce: [u8; 12] = event.enc_nonce.as_slice().try_into()?;
    let mut dec_cipher = ChaCha20::new(&enc_key.into(), &dec_nonce.into());
    dec_cipher.apply_keystream(&mut dec_msg);

    println!("Decrypted message: {}", base64::encode(&dec_msg));

    let state = AuditClientState {
        bls_sk: bls_sk.as_bytes(),
        enc_key: enc_key.into(),
        enc_commit_nonce: enc_commit_nonce.into(),
    };
    println!("\nState: {}", serde_json::to_string_pretty(&state)?);


    Ok(())
}
