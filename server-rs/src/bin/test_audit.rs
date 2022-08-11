use bls_signatures::{Serialize as BlsSerialize};
use reqwest::blocking::Client;
use qlock::{audit::{RegisterRequest, RegisterResponse, SignRequest, SignResponse}, checks::require, bls::aggregate_sigs_multi, envelope::RequestEnvelope};
use qlock::audit::client::AuditClientState;
use qlock::audit::store::LogEvent;
use qlock::bls::verify_multi;
use qlock::error::AppResult;
use serde::{Deserialize, Serialize};
use tracing::log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestRequest {
    msg: String,
}

fn main() -> AppResult<()> {
    let client = Client::new();

    // Generate BLS keypair
    let seed: [u8; 32] = rand::random();
    let bls_sk = bls_signatures::PrivateKey::new(seed);
    let bls_pk = bls_sk.public_key();

    // Generate symmetric key
    let enc_key: [u8; 32] = rand::random();

    // Register
    let bls_pk_str = base64::encode(&bls_pk.as_bytes());
    debug!("Register: pk={}", bls_pk_str);
    let resp: RegisterResponse = client.post("http://localhost:9001/api/register")
        .json(&RegisterRequest {
            client_pk: bls_pk_str.clone(),
        })
        .send()?
        .error_for_status()?
        .json()?;
    let agg_pk = bls_signatures::PublicKey::from_bytes(&resp.aggregate_pk)?;
    let server_pk = bls_signatures::PublicKey::from_bytes(&resp.server_pk)?;
    debug!("Register -> aggregate_pk={}", base64::encode(&resp.aggregate_pk));

    // Sign
    let req = TestRequest {
        msg: "hello world".into(),
    };
    let msg = serde_json::to_string(&req)?;

    // 1. Client envelope. Audit server injects metadata
    let client_envelope = RequestEnvelope::seal(&req, &enc_key)?;
    let client_envelope_data = client_envelope.serialize();
    let client_sig = bls_sk.sign(client_envelope_data.as_bytes());
    debug!("Sign: msg={} envelope={}", msg, client_envelope_data);
    let resp: SignResponse = client.post("http://localhost:9001/api/sign")
        .json(&SignRequest {
            client_pk: bls_pk_str.clone(),
            envelope: client_envelope_data.as_bytes().into(),
            client_sig: client_sig.as_bytes(),
        })
        .send()?
        .error_for_status()?
        .json()?;
    debug!("Sign -> server_sig={}", base64::encode(&resp.server_sig));

    // Verify & sign new envelope with metadata injected
    let envelope: RequestEnvelope = serde_json::from_slice(&resp.new_envelope)?;
    require(envelope.enc_payload == client_envelope.enc_payload)?;
    require(envelope.enc_nonce == client_envelope.enc_nonce)?;
    require(envelope.public_metadata.is_some())?;
    let new_sig = bls_sk.sign(&resp.new_envelope);

    // Aggregate signatures
    let server_sig = bls_signatures::Signature::from_bytes(&resp.server_sig)?;
    let agg_sig = aggregate_sigs_multi(&[
        (&new_sig, &bls_pk),
        (&server_sig, &server_pk),
    ]);

    // Verify sig under both PKs
    debug!("Verify agg sig");
    if !verify_multi(&agg_sig, &resp.new_envelope, &agg_pk) {
        debug!("Invalid signature");
        return Ok(());
    }

    // Get logs
    debug!("Get logs");
    let logs: Vec<LogEvent> = client
        .get(format!("http://localhost:9001/api/device/{}/logs", urlencoding::encode(&bls_pk_str)))
        .send()?
        .error_for_status()?
        .json()?;
    debug!("Logs: {:?}", logs);

    // Decrypt log message
    debug!("Decrypt log message");
    let event = &logs[0];
    let dec_req: TestRequest = event.envelope.open(&enc_key)?;

    debug!("Decrypted message: {:?}", dec_req);

    let state = AuditClientState {
        bls_sk: bls_sk.as_bytes(),
        enc_key: enc_key.into(),
    };
    debug!("\nState: {}", serde_json::to_string_pretty(&state)?);

    Ok(())
}
