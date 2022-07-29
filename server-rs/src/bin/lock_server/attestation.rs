use std::fmt::Debug;
use anyhow::anyhow;
use x509_parser::der_parser::oid;
use x509_parser::prelude::{FromDer, X509Certificate};
use asn1::{Asn1Read, Asn1Write, Enumerated, SetOf};
use qlock::checks::require;
use qlock::time::now;
use crate::certificates::GOOGLE_ROOTS;
use crate::CONFIG;

/*
 * KeyMint v100 schema
 */
#[derive(Asn1Read, Asn1Write)]
struct AuthorizationList<'a> {
    #[explicit(1)]
    purpose: Option<SetOf<'a, u64>>,
    #[explicit(2)]
    algorithm: Option<u64>,
    #[explicit(3)]
    key_size: Option<u64>,
    #[explicit(5)]
    digest: Option<SetOf<'a, u64>>,
    #[explicit(6)]
    padding: Option<SetOf<'a, u64>>,
    #[explicit(10)]
    ec_curve: Option<u64>,
    #[explicit(200)]
    rsa_public_exponent: Option<u64>,
    #[explicit(203)]
    mfg_digest: Option<SetOf<'a, u64>>,
    #[explicit(303)]
    rollback_resistance: Option<()>,
    #[explicit(305)]
    early_boot_only: Option<()>,
    #[explicit(400)]
    active_date_time: Option<u64>,
    #[explicit(401)]
    origination_expire_date_time: Option<u64>,
    #[explicit(402)]
    usage_expire_date_time: Option<u64>,
    #[explicit(405)]
    usage_count_limit: Option<u64>,
    #[explicit(503)]
    no_auth_required: Option<()>,
    #[explicit(504)]
    user_auth_type: Option<u64>,
    #[explicit(505)]
    auth_timeout: Option<u64>,
    #[explicit(506)]
    allow_while_on_body: Option<()>,
    #[explicit(507)]
    trusted_user_presence_required: Option<()>,
    #[explicit(508)]
    trusted_confirmation_required: Option<()>,
    #[explicit(509)]
    unlocked_device_required: Option<()>,
    #[explicit(701)]
    creation_date_time: Option<u64>,
    #[explicit(702)]
    origin: Option<u64>,
    #[explicit(704)]
    root_of_trust: Option<RootOfTrust<'a>>,
    #[explicit(705)]
    os_version: Option<u64>,
    #[explicit(706)]
    os_patch_level: Option<u64>,
    #[explicit(709)]
    attestation_application_id: Option<&'a [u8]>,
    #[explicit(710)]
    attestation_id_brand: Option<&'a [u8]>,
    #[explicit(711)]
    attestation_id_device: Option<&'a [u8]>,
    #[explicit(712)]
    attestation_id_product: Option<&'a [u8]>,
    #[explicit(713)]
    attestation_id_serial: Option<&'a [u8]>,
    #[explicit(714)]
    attestation_id_imei: Option<&'a [u8]>,
    #[explicit(715)]
    attestation_id_meid: Option<&'a [u8]>,
    #[explicit(716)]
    attestation_id_manufacturer: Option<&'a [u8]>,
    #[explicit(717)]
    attestation_id_model: Option<&'a [u8]>,
    #[explicit(718)]
    vendor_patch_level: Option<u64>,
    #[explicit(719)]
    boot_patch_level: Option<u64>,
    #[explicit(720)]
    device_unique_attestation: Option<()>,
}

#[derive(Debug, Asn1Read, Asn1Write)]
struct RootOfTrust<'a> {
    verified_boot_key: &'a [u8],
    device_locked: bool,
    verified_boot_state: Enumerated,
    verified_boot_hash: &'a [u8],
}

// #[derive(Clone, Debug, Eq, PartialEq, Sequence)]
#[derive(Asn1Read, Asn1Write)]
struct KeyDescription<'a> {
    attestation_version: i32,
    attestation_security_level: Enumerated,
    keymint_version: i32,
    keymint_security_level: Enumerated,
    attestation_challenge: &'a [u8],
    unique_id: &'a [u8],
    software_enforced: AuthorizationList<'a>,
    tee_enforced: AuthorizationList<'a>,
}

enum SecurityLevel {
    Software = 0,
    TrustedEnvironment = 1,
    StrongBox = 2,
}

enum VerifiedBootState {
    Verified = 0,
    SelfSigned = 1,
    Unverified = 2,
    Failed = 3,
}

fn verify_cert_chain<'c>(certs: &'c [X509Certificate], roots: &[X509Certificate]) -> anyhow::Result<&'c X509Certificate<'c>> {
    require(!certs.is_empty())?;

    // Verify root
    let root = certs.last().unwrap();
    require(roots.contains(root))?;

    // Initial parent is root (last)
    let mut parent = root;
    // Verify each cert starting from root, including parent
    for cert in certs.iter().rev() {
        cert.verify_signature(Some(parent.public_key()))?;
        require(cert.validity().is_valid())?;

        // Issued by parent
        require(cert.issuer == parent.subject)?;

        // TODO: check revocation

        parent = cert;
    }

    // Return attestation cert
    Ok(certs.first().unwrap())
}

fn verify_attestation(cert: &X509Certificate, challenge_id: &str, is_delegation: bool) -> anyhow::Result<()> {
    let ext = cert.get_extension_unique(&oid!(1.3.6.1.4.1.11129.2.1.17))?
        .ok_or(anyhow!("No attestation extension"))?;

    let record: KeyDescription = asn1::parse_single(ext.value)?;

    require(record.attestation_challenge == challenge_id.as_bytes())?;
    require(record.attestation_security_level.value() == SecurityLevel::StrongBox as u32 ||
        record.attestation_security_level.value() == SecurityLevel::TrustedEnvironment as u32)?;
    require(record.keymint_security_level.value() == SecurityLevel::StrongBox as u32 ||
        record.keymint_security_level.value() == SecurityLevel::TrustedEnvironment as u32)?;

    let now = now();
    if let Some(t) = record.software_enforced.active_date_time {
        require(t <= (now + CONFIG.time_grace_period))?;
    }
    if let Some(t) = record.software_enforced.creation_date_time {
        require(t <= (now + CONFIG.time_grace_period))?;
    }
    if let Some(t) = record.software_enforced.usage_expire_date_time {
        require(t >= (now - CONFIG.time_grace_period))?;
    }

    if is_delegation {
        require(!record.tee_enforced.no_auth_required.is_some())?;
        require(record.tee_enforced.unlocked_device_required.is_some())?;
    }

    Ok(())
}

pub fn verify_chain(raw_chain: &[String], challenge_id: &str, is_delegation: bool) -> anyhow::Result<()> {
    let data_chain = raw_chain.iter()
        .map(|c| base64::decode(c))
        .collect::<Result<Vec<_>, _>>()?;
    let certs: Vec<_> = data_chain.iter()
        .map(|c| X509Certificate::from_der(&c).map(|c| c.1))
        .collect::<Result<Vec<_>, _>>()?;

    let attestation_cert = verify_cert_chain(&certs, &GOOGLE_ROOTS)?;
    verify_attestation(attestation_cert, challenge_id, is_delegation)?;
    Ok(())
}
