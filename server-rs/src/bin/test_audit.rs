use reqwest::blocking::Client;
use ring::{hmac, signature};
use ring::signature::UnparsedPublicKey;
use qlock::{audit::{RegisterRequest, RegisterResponse, SignRequest, SignResponse}, checks::require, envelope::RequestEnvelope};
use qlock::audit::client::AuditClientState;
use qlock::audit::store::LogEvent;
use qlock::error::AppResult;
use serde::{Deserialize, Serialize};
use qlock::crypto::hash;
use qlock::envelope::AuditStamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestRequest {
    msg: String,
}

fn main() -> AppResult<()> {
    let client = Client::new();

    // Generate MAC key
    let mac_key_data: [u8; 32] = rand::random();
    let mac_key = hmac::Key::new(hmac::HMAC_SHA256, &mac_key_data);

    // Generate symmetric key
    let enc_key: [u8; 32] = rand::random();

    // Register
    println!("Register: mac_key={}", base64::encode(&mac_key_data));
    let resp: RegisterResponse = client.post("http://localhost:9001/api/register")
        .json(&RegisterRequest {
            client_mac_key: mac_key_data.into(),
        })
        .send()?
        .error_for_status()?
        .json()?;
    let client_id = resp.client_id;
    let server_pk = UnparsedPublicKey::new(&signature::ED25519, &resp.server_pk);
    println!("Register -> client_id={} server_pk={}", client_id, base64::encode(&resp.server_pk));

    // Sign
    let req = TestRequest {
        msg: "hello world".into(),
    };
    let msg = serde_json::to_string(&req)?;

    // 1. Client envelope
    let client_envelope = RequestEnvelope::seal(&req, &enc_key)?;
    let client_envelope_data = client_envelope.serialize();
    let client_mac = hmac::sign(&mac_key, &client_envelope_data.as_bytes());
    println!("Sign: msg={} envelope={} mac={}", msg, client_envelope_data, base64::encode(&client_mac.as_ref()));
    let resp: SignResponse = client.post("http://localhost:9001/api/sign")
        .json(&SignRequest {
            client_id: client_id.clone(),
            envelope: client_envelope_data.as_bytes().into(),
            client_mac: client_mac.as_ref().into(),
        })
        .send()?
        .error_for_status()?
        .json()?;
    let stamp: AuditStamp = serde_json::from_slice(&resp.stamp)?;
    println!("Sign -> stamp={:?} server_sig={}", stamp, base64::encode(&resp.server_sig));

    // Verify server sig
    println!("Verify sig");
    server_pk.verify(&resp.stamp, &resp.server_sig)?;

    // Verify hash
    require(stamp.envelope_hash == hash(client_envelope_data.as_bytes()))?;

    // Get logs
    println!("Get logs");
    let logs: Vec<LogEvent> = client
        .get(format!("http://localhost:9001/api/device/{}/logs", urlencoding::encode(&client_id)))
        .send()?
        .error_for_status()?
        .json()?;
    println!("Logs: {:?}", logs);

    // Decrypt log message
    println!("Decrypt log message");
    let event = &logs[0];
    let dec_req: TestRequest = event.envelope.open(&enc_key)?;

    println!("Decrypted message: {:?}", dec_req);

    let state = AuditClientState {
        mac_key: mac_key_data.into(),
        enc_key: enc_key.into(),
    };
    println!("\nState: {}", serde_json::to_string_pretty(&state)?);

    Ok(())
}
