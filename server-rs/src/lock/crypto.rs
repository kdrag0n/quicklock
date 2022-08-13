use bls_signatures::{PublicKey, Serialize, Signature};
use ring::signature;
use ring::signature::UnparsedPublicKey;
use spki::SubjectPublicKeyInfo;
use crate::bls::verify_multi;
use crate::checks::require;
use crate::error::AppResult;

pub fn verify_ec_signature_str(data: &str, public_key: &str, signature: &[u8]) -> AppResult<()> {
    let pk = base64::decode(public_key)?;
    let pk = SubjectPublicKeyInfo::try_from(pk.as_slice())?;

    let pk = UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_ASN1,
                                    pk.subject_public_key);

    pk.verify(data.as_bytes(), signature)?;

    Ok(())
}

pub fn verify_ed25519_signature(data: &[u8], public_key: &[u8], signature: &[u8]) -> AppResult<()> {
    let pk = UnparsedPublicKey::new(&signature::ED25519, public_key);
    pk.verify(data, signature)?;

    Ok(())
}

pub fn verify_bls_signature_str(data: &str, public_key: &str, signature: &str) -> AppResult<()> {
    let sig = Signature::from_bytes(&base64::decode(signature)?)?;
    let pk = PublicKey::from_bytes(&base64::decode(public_key)?)?;

    require(verify_multi(&sig, data.as_bytes(), &pk))?;

    Ok(())
}

pub fn generate_secret() -> String {
    let secret: [u8; 32] = rand::random(); // ChaCha12 CSPRNG
    base64::encode(&secret)
}
