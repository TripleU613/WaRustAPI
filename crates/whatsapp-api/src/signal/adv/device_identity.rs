//! Cryptographic part of APK 37.71 11T.A0D, not device admission or pin adoption.
use super::{SignalSignatureVerifier, no_duplicate_singletons};
use crate::{
    Error, Result,
    proto::wa_adv::{AdvDeviceIdentity, AdvSignedDeviceIdentity},
    transport::cert::SignatureVerifier,
};
use prost::Message;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("companion_adv", reason)
}

/// Two valid signatures binding a companion key to an independently held
/// account key. This establishes neither current membership nor human trust.
/// Raw-ID transitions, teardown, freshness and atomic admission are separate.
#[derive(Clone, PartialEq, Eq)]
pub struct VerifiedDeviceIdentity {
    details: AdvDeviceIdentity,
}
impl std::fmt::Debug for VerifiedDeviceIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VerifiedDeviceIdentity(<redacted>)")
    }
}
impl VerifiedDeviceIdentity {
    /// Verify exact signed details and both signatures. A supplied account
    /// key must match the independent pin; it cannot grant itself authority.
    /// Bounds, singleton checks and required metadata are local parser policy.
    pub fn verify(bytes: &[u8], account_key: &[u8; 32], companion_key: &[u8; 32]) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > 16384 {
            return Err(invalid("invalid signed identity size"));
        }
        no_duplicate_singletons(bytes, None)
            .map_err(|_| invalid("malformed signed identity fields"))?;
        // Grz.dynamicMethod tags 1..4, independently recovered from APK 37.71.
        let signed = AdvSignedDeviceIdentity::decode(bytes)
            .map_err(|_| invalid("malformed signed identity"))?;
        if signed
            .account_signature_key
            .as_deref()
            .is_some_and(|key| !key.is_empty() && key != account_key)
        {
            return Err(invalid(
                "supplied account key differs from held primary pin",
            ));
        }
        let details_bytes = signed
            .details
            .ok_or_else(|| invalid("missing signed details"))?;
        if details_bytes.is_empty() || details_bytes.len() > 8192 {
            return Err(invalid("invalid signed details size"));
        }
        no_duplicate_singletons(&details_bytes, None)
            .map_err(|_| invalid("malformed signed details fields"))?;
        // GsB.dynamicMethod: rawId=1, timestamp=2, keyIndex=3, accountType=4,
        // deviceType=5. Missing encryption types default E2EE in 11T.A0D.
        let details = AdvDeviceIdentity::decode(details_bytes.as_ref())
            .map_err(|_| invalid("malformed signed details"))?;
        if details.raw_id.is_none() || details.timestamp.is_none() || details.key_index.is_none() {
            return Err(invalid("missing raw ID, timestamp or key index"));
        }
        let account_type = details.account_type.unwrap_or(0);
        let device_type = details.device_type.unwrap_or(0);
        if !matches!(account_type, 0 | 1) || !matches!(device_type, 0 | 1) {
            return Err(invalid("unsupported encryption type"));
        }
        // 11T.A0D 01e0..0226 refuses HOSTED device with E2EE account.
        if device_type == 1 && account_type == 0 {
            return Err(invalid("account and device encryption type mismatch"));
        }
        let account_signature: [u8; 64] = signed
            .account_signature
            .ok_or_else(|| invalid("missing account signature"))?
            .as_ref()
            .try_into()
            .map_err(|_| invalid("invalid account signature size"))?;
        let device_signature: [u8; 64] = signed
            .device_signature
            .ok_or_else(|| invalid("missing device signature"))?
            .as_ref()
            .try_into()
            .map_err(|_| invalid("invalid device signature size"))?;
        // 08F.<clinit> A0E=[06,00], A0F=[06,05], A0G=[06,01],
        // A0H=[06,06]. 11T.A0D 0292..0308 selects by DEVICE type and signs
        // raw 32-byte keys, not their 33-byte Signal type-prefixed encoding.
        let account_prefix = if device_type == 1 { [6, 5] } else { [6, 0] };
        let device_prefix = if device_type == 1 { [6, 6] } else { [6, 1] };
        let account_input = [&account_prefix[..], details_bytes.as_ref(), companion_key].concat();
        let device_input = [
            &device_prefix[..],
            details_bytes.as_ref(),
            companion_key,
            account_key,
        ]
        .concat();
        // 11T.A0D 030a..0374 requires BOTH B7H.A09 verifications.
        if !SignalSignatureVerifier.verify(account_key, &account_input, &account_signature) {
            return Err(invalid("account signature verification failed"));
        }
        if !SignalSignatureVerifier.verify(companion_key, &device_input, &device_signature) {
            return Err(invalid("device signature verification failed"));
        }
        Ok(Self { details })
    }

    /// Bind verification to the public identity inside the prekey envelope,
    /// as APK C8C.A00 does. Does not authenticate/decrypt the nested message.
    pub fn verify_prekey(bytes: &[u8], account_key: &[u8; 32], ciphertext: &[u8]) -> Result<Self> {
        use wacore_libsignal::protocol::PreKeySignalMessage;
        if ciphertext.len() > 65536 {
            return Err(invalid("prekey envelope exceeds bound"));
        }
        let message = PreKeySignalMessage::try_from(ciphertext)
            .map_err(|_| invalid("invalid prekey envelope"))?;
        let serialized = message.identity_key().serialize();
        let companion_key: &[u8; 32] = serialized
            .get(1..)
            .and_then(|key| key.try_into().ok())
            .ok_or_else(|| invalid("invalid companion identity key"))?;
        Self::verify(bytes, account_key, companion_key)
    }

    /// Signed raw ID; equality with the stored account is a separate check.
    pub fn raw_id(&self) -> u32 {
        self.details.raw_id.expect("verified presence")
    }
    /// Signed timestamp; this verifier does not establish freshness.
    pub fn timestamp(&self) -> u64 {
        self.details.timestamp.expect("verified presence")
    }
    /// Signed key index; does not identify a transport device ID.
    pub fn key_index(&self) -> u32 {
        self.details.key_index.expect("verified presence")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wacore_libsignal::{
        core::curve::PrivateKey,
        protocol::{IdentityKey, PreKeySignalMessage, SignalMessage},
    };

    // Literal GsB tags from APK 37.71: rawId=7, timestamp=9, keyIndex=2.
    // Signing below is an adapter fixture, not an Android interoperability capture.
    const DETAILS: &[u8] = &[8, 7, 16, 9, 24, 2];
    fn public(seed: u8) -> [u8; 32] {
        PrivateKey::deserialize(&[seed; 32])
            .unwrap()
            .public_key()
            .unwrap()
            .public_key_bytes()
            .try_into()
            .unwrap()
    }
    fn signed(details: &[u8], account_prefix: &[u8], device_prefix: &[u8]) -> Vec<u8> {
        let account = PrivateKey::deserialize(&[42; 32]).unwrap();
        let companion = PrivateKey::deserialize(&[43; 32]).unwrap();
        let a = public(42);
        let c = public(43);
        let account_sig = account
            .calculate_signature(&[account_prefix, details, &c].concat(), &mut rand::rng())
            .unwrap();
        let device_sig = companion
            .calculate_signature(&[device_prefix, details, &c, &a].concat(), &mut rand::rng())
            .unwrap();
        // Hand-assembled Grz, so fixture tags/order do not come from the reader.
        let mut bytes = vec![10, details.len() as u8];
        bytes.extend(details);
        bytes.extend([18, 32]);
        bytes.extend(a);
        bytes.extend([26, 64]);
        bytes.extend(account_sig);
        bytes.extend([34, 64]);
        bytes.extend(device_sig);
        bytes
    }
    fn prekey(seed: u8) -> Vec<u8> {
        let key = PrivateKey::deserialize(&[seed; 32])
            .unwrap()
            .public_key()
            .unwrap();
        let identity = IdentityKey::new(key);
        let message = SignalMessage::new(
            3,
            &[9; 32],
            key,
            0,
            0,
            b"synthetic ciphertext",
            &identity,
            &identity,
        )
        .unwrap();
        PreKeySignalMessage::new(3, 42, None, 7.into(), key, identity, message)
            .unwrap()
            .serialized()
            .to_vec()
    }
    #[test]
    fn both_signatures_bind_exact_details_and_both_keys() {
        let bytes = signed(DETAILS, &[6, 0], &[6, 1]);
        let verified = VerifiedDeviceIdentity::verify(&bytes, &public(42), &public(43)).unwrap();
        assert_eq!(
            (
                verified.raw_id(),
                verified.timestamp(),
                verified.key_index()
            ),
            (7, 9, 2)
        );
        assert_eq!(
            format!("{verified:?}"),
            "VerifiedDeviceIdentity(<redacted>)"
        );
        assert!(VerifiedDeviceIdentity::verify(&bytes, &public(44), &public(43)).is_err());
        assert!(VerifiedDeviceIdentity::verify(&bytes, &public(42), &public(44)).is_err());
        for (a, d) in [(&[6, 1][..], &[6, 1][..]), (&[6, 0][..], &[6, 0][..])] {
            assert!(
                VerifiedDeviceIdentity::verify(&signed(DETAILS, a, d), &public(42), &public(43))
                    .is_err()
            );
        }
        for field in ["account", "device"] {
            let mut changed = AdvSignedDeviceIdentity::decode(bytes.as_slice()).unwrap();
            let signature = if field == "account" {
                &mut changed.account_signature
            } else {
                &mut changed.device_signature
            };
            let mut damaged = signature.take().unwrap().to_vec();
            damaged[17] ^= 1;
            *signature = Some(damaged.into());
            assert!(
                VerifiedDeviceIdentity::verify(&changed.encode_to_vec(), &public(42), &public(43))
                    .is_err()
            );
        }
        let mut reordered = AdvSignedDeviceIdentity::decode(bytes.as_slice()).unwrap();
        reordered.details = Some(vec![24, 2, 16, 9, 8, 7].into());
        assert!(
            VerifiedDeviceIdentity::verify(&reordered.encode_to_vec(), &public(42), &public(43))
                .is_err()
        );
        // Equivalent protobuf semantics are fine when those EXACT bytes were signed.
        assert!(
            VerifiedDeviceIdentity::verify(
                &signed(&[24, 2, 16, 9, 8, 7], &[6, 0], &[6, 1]),
                &public(42),
                &public(43)
            )
            .is_ok()
        );
    }
    #[test]
    fn domain_selection_uses_device_type_and_checks_account_compatibility() {
        for (account, device, allowed) in [
            (0, 0, true),
            (1, 0, true),
            (1, 1, true),
            (0, 1, false),
            (2, 0, false),
            (1, 2, false),
        ] {
            let details = [DETAILS, &[32, account, 40, device]].concat();
            let (a, d) = if device == 1 {
                ([6, 5], [6, 6])
            } else {
                ([6, 0], [6, 1])
            };
            assert_eq!(
                VerifiedDeviceIdentity::verify(&signed(&details, &a, &d), &public(42), &public(43))
                    .is_ok(),
                allowed
            );
        }
        let hosted = [DETAILS, &[32, 1, 40, 1]].concat();
        assert!(
            VerifiedDeviceIdentity::verify(
                &signed(&hosted, &[6, 0], &[6, 1]),
                &public(42),
                &public(43)
            )
            .is_err()
        );
    }
    #[test]
    fn malformed_metadata_duplicates_and_untrusted_bundled_keys_refuse() {
        for details in [
            &[][..],
            &[8, 7, 24, 2],
            &[8, 7, 16, 9],
            &[16, 9, 24, 2],
            &[8, 7, 16, 9, 24, 2, 8, 7],
        ] {
            assert!(
                VerifiedDeviceIdentity::verify(
                    &signed(details, &[6, 0], &[6, 1]),
                    &public(42),
                    &public(43)
                )
                .is_err()
            );
        }
        let bytes = signed(DETAILS, &[6, 0], &[6, 1]);
        for bundled in [
            None,
            Some(vec![]),
            Some(public(42).to_vec()),
            Some(public(44).to_vec()),
            Some(vec![5]),
        ] {
            let allowed = bundled
                .as_deref()
                .is_none_or(|b| b.is_empty() || b == public(42));
            let mut outer = AdvSignedDeviceIdentity::decode(bytes.as_slice()).unwrap();
            outer.account_signature_key = bundled.map(Into::into);
            assert_eq!(
                VerifiedDeviceIdentity::verify(&outer.encode_to_vec(), &public(42), &public(43))
                    .is_ok(),
                allowed
            );
        }
        let mut duplicate = bytes.clone();
        duplicate.extend([10, 0]);
        assert!(VerifiedDeviceIdentity::verify(&duplicate, &public(42), &public(43)).is_err());
        assert!(VerifiedDeviceIdentity::verify(&vec![0; 16385], &public(42), &public(43)).is_err());
        assert!(
            VerifiedDeviceIdentity::verify(&bytes[..bytes.len() - 1], &public(42), &public(43))
                .is_err()
        );
    }
    #[test]
    fn ciphertext_identity_is_the_signature_candidate_not_an_external_guess() {
        let bytes = signed(DETAILS, &[6, 0], &[6, 1]);
        assert!(VerifiedDeviceIdentity::verify_prekey(&bytes, &public(42), &prekey(43)).is_ok());
        assert!(VerifiedDeviceIdentity::verify_prekey(&bytes, &public(42), &prekey(44)).is_err());
        assert!(VerifiedDeviceIdentity::verify_prekey(&bytes, &public(42), b"malformed").is_err());
        assert!(
            VerifiedDeviceIdentity::verify_prekey(&bytes, &public(42), &vec![0; 65537]).is_err()
        );
    }
}
