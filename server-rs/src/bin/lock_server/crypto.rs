use anyhow::anyhow;
use bls_signatures::{PublicKey, Serialize, Signature};
use ring::{signature};
use ring::signature::UnparsedPublicKey;
use spki::SubjectPublicKeyInfo;
use qlock::bls::verify_aug;
use qlock::checks::require;
use qlock::error::AppResult;

pub fn verify_ec_signature_str(data: &str, public_key: &str, signature: &str) -> AppResult<()> {
    let pk = base64::decode(public_key)?;
    let pk = SubjectPublicKeyInfo::try_from(pk.as_slice())
        .map_err(|_| anyhow!("Invalid public key"))?;

    let pk = UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_ASN1,
                                    pk.subject_public_key);

    pk.verify(data.as_bytes(), &base64::decode(signature)?)
        .map_err(|_| anyhow!("Signature verify failed"))?;

    Ok(())
}

pub fn verify_bls_signature_str(data: &str, public_keys: &[String], signature: &str) -> AppResult<()> {
    let sig = Signature::from_bytes(&base64::decode(signature)?)?;
    let pks_data = public_keys.iter()
        .map(|pk| base64::decode(pk))
        .collect::<Result<Vec<_>, _>>()?;
    let pks = pks_data.iter()
        .map(|pk| PublicKey::from_bytes(&pk))
        .collect::<Result<Vec<_>, _>>()?;

    require(verify_aug(&sig, data.as_bytes(), &pks))?;

    Ok(())
}

pub fn generate_secret() -> String {
    let secret: [u8; 32] = rand::random(); // ChaCha12 CSPRNG
    base64::encode(&secret)
}
