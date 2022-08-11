use std::str::FromStr;

use blstrs::{Bls12, G1Affine, G1Projective, G2Projective, Gt, Scalar};
use num_bigint::BigUint;
use pairing::{MultiMillerLoop, MillerLoopResult};
use bls_signatures::{PrivateKey, PublicKey, Serialize, Signature};
use pairing::group::{Curve, Group};
use pairing::group::prime::PrimeCurveAffine;
use sha2::Digest;

// No official cipher suite ID for multi-sig, so use NUL (basic scheme) because that's what the
// client-signed messages are: plain sig with a single message.
const CIPHER_SUITE_ID: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

const FIELD_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_MUL_";

fn hash_to_curve(msg: &[u8]) -> G2Projective {
    G2Projective::hash_to_curve(msg, CIPHER_SUITE_ID, &[])
}

// TODO: expand hash to reduce modulo bias
fn hash_to_field(msg: &[u8]) -> Scalar {
    let hash_bytes = sha2::Sha256::digest(msg);
    let mut bigint = BigUint::from_bytes_le(&hash_bytes);
    bigint %= BigUint::from_str("52435875175126190479447740508185965837690552500527637822603658699938581184513").unwrap();
    Scalar::from_bytes_le(&bigint.to_bytes_le().try_into().unwrap()).unwrap()
}

pub fn sign_aug(sk: &PrivateKey, msg: &[u8]) -> Signature {
    let mut aug_msg = sk.public_key().as_bytes();
    aug_msg.extend_from_slice(msg);

    let mut p = hash_to_curve(&aug_msg);
    let scalar: Scalar = (*sk).into();
    p *= scalar;

    p.into()
}

pub fn verify_aug(sig: &Signature, msg: &[u8], pks: &[PublicKey]) -> bool {
    let hashes = pks.iter()
        .map(|pk| {
            let mut aug_msg = pk.as_bytes();
            aug_msg.extend_from_slice(msg);
            hash_to_curve(&aug_msg)
        })
        .collect::<Vec<_>>();

    bls_signatures::verify(sig, &hashes, pks)
}

// Aggregated public key
// No proof of possession or augmentation
// Protects against rogue key attacks using pseudorandom constants (hash_to_field)
// https://crypto.stanford.edu/~dabo/pubs/papers/BLSmultisig.html
pub fn aggregate_sigs_multi(sigs: &[(&Signature, &PublicKey)]) -> Signature {
    let all_pks = sigs.iter()
        .map(|(_, pk)| pk.as_bytes())
        .collect::<Vec<_>>()
        .concat();

    sigs.iter()
        .fold(G2Projective::identity(), |acc, (&sig, &pk)| {
            let mut data = pk.as_bytes();
            data.extend_from_slice(&all_pks);
            let mix_const = hash_to_field(&data);

            let sig_num: G2Projective = sig.into();
            acc + (sig_num * mix_const)
        })
        .into()
}

pub fn aggregate_pks_multi(pks: &[&PublicKey]) -> PublicKey {
    let all_pks = pks.iter()
        .map(|pk| pk.as_bytes())
        .collect::<Vec<_>>()
        .concat();

    pks.iter()
        .fold(G1Projective::identity(), |acc, &pk| {
            let mut data = pk.as_bytes();
            data.extend_from_slice(&all_pks);
            let mix_const = hash_to_field(&data);

            let pk_num: G1Projective = (*pk).into();
            acc + (pk_num * mix_const)
        })
        .into()
}

pub fn verify_multi(agg_sig: &Signature, msg: &[u8], agg_pk: &PublicKey) -> bool {
    let sig_n: G2Projective = (*agg_sig).into();
    let pk_n: G1Projective = (*agg_pk).into();

    // Key can't be 0
    if pk_n.is_identity().into() {
        return false;
    }

    // Optimization: instead of doing 2 full pairings and comparing them, we can use this to compute
    // the sum with one pairing negated, so the result should be zero if they're equal
    let pairings = Bls12::multi_miller_loop(&[
        (&-G1Affine::generator(), &sig_n.to_affine().into()),
        (&pk_n.to_affine(), &hash_to_curve(msg).to_affine().into()),
    ]);
    pairings.final_exponentiation() == Gt::identity()
}
