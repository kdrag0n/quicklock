use bls_signatures::Serialize;
use reqwest::blocking::Client;
use qlock::audit::{RegisterRequest, RegisterResponse, SignRequest, SignResponse};
use qlock::bls::{sign_aug, verify_aug};
use qlock::error::AppResult;

fn main() -> AppResult<()> {
    let client = Client::new();

    // Generate BLS keypair
    let seed: [u8; 32] = rand::random();
    let bls_sk = bls_signatures::PrivateKey::new(seed);
    let bls_pk = bls_sk.public_key();

    // Register
    let bls_pk_str = base64::encode(&bls_pk.as_bytes());
    println!("Register: pk={}", bls_pk_str);
    let resp: RegisterResponse = client.post("http://localhost:9001/api/register")
        .json(&RegisterRequest {
            client_pk: bls_pk_str.clone(),
        })
        .send()?
        .error_for_status()?
        .json()?;
    let server_pk = bls_signatures::PublicKey::from_bytes(&resp.server_pk)?;
    println!("Register -> server_pk={}", base64::encode(&resp.server_pk));

    // Sign
    let msg: [u8; 32] = rand::random();
    let client_sig = sign_aug(&bls_sk, &msg);
    println!("Sign: msg={}", base64::encode(&msg));
    let resp: SignResponse = client.post("http://localhost:9001/api/sign")
        .json(&SignRequest {
            client_pk: bls_pk_str.clone(),
            message: msg.into(),
            client_sig: client_sig.as_bytes(),
        })
        .send()?
        .error_for_status()?
        .json()?;
    println!("Sign -> aggregate_sig={}", base64::encode(&resp.aggregate_sig));

    // Verify sig under both PKs
    let agg_sig = bls_signatures::Signature::from_bytes(&resp.aggregate_sig)?;
    let pks = [bls_pk, server_pk];
    if !verify_aug(&agg_sig, &msg, &pks) {
        println!("Invalid signature");
        return Ok(());
    }

    Ok(())
}
