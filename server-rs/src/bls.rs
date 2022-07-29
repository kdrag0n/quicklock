use bls12_381::{G2Projective, hash_to_curve::HashToCurve, Scalar};
use bls12_381::hash_to_curve::ExpandMsgXmd;
use bls_signatures::{PrivateKey, PublicKey, Serialize, Signature};

// Augmentation scheme, min public key
// https://www.ietf.org/archive/id/draft-irtf-cfrg-bls-signature-05.html#name-message-augmentation
const CIPHER_SUITE_ID: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_AUG_";

// Hash with AugSchemeMPL instead of basic "NUL"
fn hash(msg: &[u8]) -> G2Projective {
    <G2Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(msg, CIPHER_SUITE_ID)
}

pub fn sign_aug(sk: &PrivateKey, msg: &[u8]) -> Signature {
    let mut aug_msg = sk.public_key().as_bytes();
    aug_msg.extend_from_slice(msg);

    let mut p = hash(&aug_msg);
    let scalar: Scalar = (*sk).into();
    p *= scalar;

    p.into()
}

pub fn verify_aug(sig: &Signature, msg: &[u8], pks: &[PublicKey]) -> bool {
    let hashes = pks.iter()
        .map(|pk| {
            let mut aug_msg = pk.as_bytes();
            aug_msg.extend_from_slice(msg);
            hash(&aug_msg)
        })
        .collect::<Vec<_>>();

    bls_signatures::verify(sig, &hashes, &pks)
}
