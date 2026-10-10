//! Comparison signed ADV lists and device identities (11T.A07/A0D, 08F).
//! Verifies a pinned account key only; no identity replacement or activation.
use super::{app_state::Fingerprint, crypto::SignalSignatureVerifier};
use crate::{Error, Result, transport::cert::SignatureVerifier};
use prost::Message;
use std::collections::BTreeSet;
mod device_identity;
mod device_identity_details;
mod devices;
pub use device_identity::VerifiedDeviceIdentity;
pub use devices::{DeviceHashVersion, ReconciledDevices};

#[derive(Message)]
struct SignedList {
    #[prost(bytes = "vec", optional, tag = "1")]
    details: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "2")]
    signature: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "3")]
    account_key: Option<Vec<u8>>,
}
#[derive(Message)]
struct IndexList {
    #[prost(uint32, optional, tag = "1")]
    raw_id: Option<u32>,
    #[prost(uint64, optional, tag = "2")]
    timestamp: Option<u64>,
    #[prost(uint32, optional, tag = "3")]
    current_index: Option<u32>,
    #[prost(uint32, repeated, packed = "true", tag = "4")]
    indexes: Vec<u32>,
    #[prost(int32, optional, tag = "5")]
    account_type: Option<i32>,
}
fn bad() -> Error {
    Error::invalid(
        "adv_index_list",
        "invalid, stale or untrusted signed index list",
    )
}

/// Signature-verified E2EE key-index metadata. This is not a transport device
/// list, a linked-device session, or permission to use a received sync key.
#[derive(Clone, PartialEq, Eq)]
pub struct VerifiedIndexList {
    raw_id: u32,
    timestamp: u64,
    current_index: u32,
    indexes: BTreeSet<u32>,
}
impl std::fmt::Debug for VerifiedIndexList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifiedIndexList")
            .field("index_count", &self.indexes.len())
            .finish_non_exhaustive()
    }
}

fn no_duplicate_singletons(mut bytes: &[u8], repeated: Option<u32>) -> Result<()> {
    use prost::encoding::{DecodeContext, decode_key, skip_field};
    let mut seen = BTreeSet::new();
    while !bytes.is_empty() {
        let (tag, wire) = decode_key(&mut bytes).map_err(|_| bad())?;
        if Some(tag) != repeated && !seen.insert(tag) {
            return Err(bad());
        }
        skip_field(wire, tag, &mut bytes, DecodeContext::default()).map_err(|_| bad())?;
    }
    Ok(())
}

impl VerifiedIndexList {
    /// Verify exact signed bytes against an independently trusted account key.
    /// `expected_timestamp` comes from the authenticated key-index-list stanza;
    /// `minimum_timestamp` comes from the saved account baseline, not the list.
    /// A bundled different key is refused, unlike APK identity-change branches
    /// that require additional reconciliation not implemented here. Local bounds,
    /// required metadata and duplicate refusals are conservative parser policy.
    pub fn verify(
        bytes: &[u8],
        pinned_account_key: &[u8; 32],
        expected_timestamp: u64,
        minimum_timestamp: u64,
    ) -> Result<Self> {
        if bytes.len() > 16384 {
            return Err(bad());
        }
        no_duplicate_singletons(bytes, None)?;
        let signed = SignedList::decode(bytes).map_err(|_| bad())?;
        if signed
            .account_key
            .as_deref()
            .is_some_and(|key| key != pinned_account_key)
        {
            return Err(bad());
        }
        let details = signed.details.ok_or_else(bad)?;
        if details.is_empty() || details.len() > 8192 {
            return Err(bad());
        }
        let signature: [u8; 64] = signed
            .signature
            .ok_or_else(bad)?
            .try_into()
            .map_err(|_| bad())?;
        let mut signed_bytes = Vec::with_capacity(details.len() + 2);
        signed_bytes.extend_from_slice(&[6, 2]);
        signed_bytes.extend_from_slice(&details);
        if !SignalSignatureVerifier.verify(pinned_account_key, &signed_bytes, &signature) {
            return Err(bad());
        }
        no_duplicate_singletons(&details, Some(4))?;
        let list = IndexList::decode(details.as_slice()).map_err(|_| bad())?;
        let timestamp = list.timestamp.ok_or_else(bad)?;
        let current_index = list.current_index.ok_or_else(bad)?;
        if list.account_type.unwrap_or(0) != 0
            || timestamp != expected_timestamp
            || timestamp < minimum_timestamp
            || list.indexes.is_empty()
            || list.indexes.len() > 1024
        {
            return Err(bad());
        }
        let indexes: BTreeSet<_> = list.indexes.iter().copied().collect();
        if indexes.len() != list.indexes.len() || indexes.iter().any(|index| *index > current_index)
        {
            return Err(bad());
        }
        Ok(Self {
            raw_id: list.raw_id.ok_or_else(bad)?,
            timestamp,
            current_index,
            indexes,
        })
    }
    /// Whether this authenticated list includes a companion key index.
    /// APK 37.71 11T.A08 uses signed indexes separately from retained devices.
    pub fn contains_index(&self, index: u32) -> bool {
        self.indexes.contains(&index)
    }
    /// Account raw id the signed list belongs to.
    pub fn raw_id(&self) -> u32 {
        self.raw_id
    }
    /// Original authenticated list timestamp, for replay/ordering checks.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
    /// Compare a share fingerprint with authenticated ADV indexes. This alone
    /// does not prove that each advertised device is still registered; reconcile
    /// the account's companion-device records before activating a sync key.
    pub fn matches_fingerprint(&self, fingerprint: &Fingerprint) -> bool {
        fingerprint.matches_registered_devices(self.raw_id, self.current_index, &self.indexes)
    }
}

/// Test-only: a signed E2EE key-index list for the given account private key.
/// Returns the encoded `BRD` bytes and the account public key.
#[cfg(test)]
pub(crate) fn test_signed_list(
    raw_id: u32,
    timestamp: u64,
    current_index: u32,
    indexes: &[u32],
    private: &[u8; 32],
) -> (Vec<u8>, [u8; 32]) {
    use wacore_libsignal::core::curve::PrivateKey;
    let details = IndexList {
        raw_id: Some(raw_id),
        timestamp: Some(timestamp),
        current_index: Some(current_index),
        indexes: indexes.to_vec(),
        account_type: Some(0),
    }
    .encode_to_vec();
    let private = PrivateKey::deserialize(private).unwrap();
    let public = private.public_key().unwrap();
    let key: [u8; 32] = public.public_key_bytes().try_into().unwrap();
    let signature = private
        .calculate_signature(&[&[6u8, 2u8][..], &details].concat(), &mut rand::rng())
        .unwrap();
    let signed = SignedList {
        details: Some(details),
        signature: Some(signature.to_vec()),
        account_key: Some(key.to_vec()),
    }
    .encode_to_vec();
    (signed, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wacore_libsignal::core::curve::PrivateKey;
    // Literal BUg protobuf: raw=7, timestamp=9, current=2, indexes=[1,2], E2EE.
    const DETAILS: &[u8] = &[8, 7, 16, 9, 24, 2, 34, 2, 1, 2, 40, 0];
    fn signed(details: &[u8], domain: &[u8]) -> (Vec<u8>, [u8; 32]) {
        let private = PrivateKey::deserialize(&[42; 32]).unwrap();
        let public = private.public_key().unwrap();
        let key: [u8; 32] = public.public_key_bytes().try_into().unwrap();
        let signature = private
            .calculate_signature(&[domain, details].concat(), &mut rand::rng())
            .unwrap();
        // Hand-assembled BRD fixture; adapter signature roundtrip, not an
        // independent Android interoperability capture.
        assert!(details.len() < 128);
        let mut bytes = vec![10, details.len() as u8];
        bytes.extend(details);
        bytes.extend([18, 64]);
        bytes.extend(signature);
        bytes.extend([26, 32]);
        bytes.extend(key);
        (bytes, key)
    }
    #[test]
    fn signature_domain_identity_and_time_are_checked() {
        let (bytes, key) = signed(DETAILS, &[6, 2]);
        let list = VerifiedIndexList::verify(&bytes, &key, 9, 9).unwrap();
        assert_eq!(list.timestamp(), 9);
        assert!(VerifiedIndexList::verify(&bytes, &[3; 32], 9, 0).is_err());
        assert!(VerifiedIndexList::verify(&bytes, &key, 8, 0).is_err());
        assert!(VerifiedIndexList::verify(&bytes, &key, 9, 10).is_err());
        let (wrong_domain, _) = signed(DETAILS, &[6, 1]);
        assert!(VerifiedIndexList::verify(&wrong_domain, &key, 9, 0).is_err());
        for i in 0..bytes.len() {
            let mut altered = bytes.clone();
            altered[i] ^= 1;
            assert!(VerifiedIndexList::verify(&altered, &key, 9, 0).is_err());
        }
        let share = super::super::app_state::KeyShare::decode(b"\x0a\x1e\x0a\x08\x0a\x06\x00\x01\x00\x00\x00\x02\x12\x12\x0a\x04test\x12\x08\x08\x07\x10\x02\x1a\x02\x01\x02\x18\x09").unwrap();
        assert!(list.matches_fingerprint(share.fingerprints().next().unwrap()));
        assert!(!format!("{list:?}").contains("raw_id"));
    }
    #[test]
    fn signed_but_malformed_or_unsupported_details_fail() {
        for details in [
            vec![],
            vec![8, 7, 16, 9, 24, 2, 34, 2, 1, 1],
            vec![8, 7, 16, 9, 24, 2, 34, 1, 3],
            vec![8, 7, 16, 9, 24, 2, 34, 1, 1, 40, 1],
            vec![8, 7, 16, 9, 24, 2],
            vec![8, 7, 8, 8, 16, 9, 24, 2, 34, 1, 1],
        ] {
            let (bytes, key) = signed(&details, &[6, 2]);
            assert!(VerifiedIndexList::verify(&bytes, &key, 9, 0).is_err());
        }
        let (mut bytes, key) = signed(DETAILS, &[6, 2]);
        bytes.extend([10, 0]);
        assert!(VerifiedIndexList::verify(&bytes, &key, 9, 0).is_err());
        assert!(VerifiedIndexList::verify(&vec![0; 16385], &key, 9, 0).is_err());
    }
}
