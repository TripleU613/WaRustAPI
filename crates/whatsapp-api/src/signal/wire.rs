//! APK-compatible Signal envelope encoding; no encryption, trust or saved-state mutation.
use crate::{Error, Result};
use hkdf::hmac::{Hmac, KeyInit, Mac};
use prost::Message;
use wacore_libsignal::protocol::{IdentityKey, PublicKey, SignalMessage};

#[derive(Message)]
struct SignalBody {
    #[prost(bytes = "vec", optional, tag = "1")]
    ratchet: Option<Vec<u8>>,
    #[prost(uint32, optional, tag = "2")]
    counter: Option<u32>,
    #[prost(uint32, optional, tag = "3")]
    previous: Option<u32>,
    #[prost(bytes = "vec", optional, tag = "4")]
    ciphertext: Option<Vec<u8>>,
}
#[derive(Message)]
struct PreKeyBody {
    #[prost(uint32, optional, tag = "1")]
    prekey: Option<u32>,
    #[prost(bytes = "vec", optional, tag = "2")]
    base: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "3")]
    identity: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "4")]
    message: Option<Vec<u8>>,
    #[prost(uint32, optional, tag = "5")]
    registration: Option<u32>,
    #[prost(uint32, optional, tag = "6")]
    signed: Option<u32>,
    #[prost(uint32, optional, tag = "7")]
    kem_id: Option<u32>,
    #[prost(bytes = "vec", optional, tag = "8")]
    kem_ciphertext: Option<Vec<u8>>,
}
fn invalid(reason: &'static str) -> Error {
    Error::invalid("signal_envelope", reason)
}
fn version_byte(version: u8) -> Result<u8> {
    match version {
        3 | 4 => Ok((version << 4) | version),
        _ => Err(invalid("only versions 3 and 4 are supported")),
    }
}

/// Encode an already encrypted Signal body with the APK version byte and MAC.
/// Identity tuple order is sender/receiver; counters are current/previous.
/// This does not encrypt plaintext, select trusted keys, or advance a session.
pub fn encode_signal(
    version: u8,
    mac_key: &[u8; 32],
    identities: (&IdentityKey, &IdentityKey),
    ratchet: &PublicKey,
    counters: (u32, u32),
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    let header = version_byte(version)?;
    if ciphertext.len() > 65536 {
        return Err(invalid("envelope exceeds bound"));
    }
    let body = SignalBody {
        ratchet: Some(ratchet.serialize().to_vec()),
        counter: Some(counters.0),
        previous: Some(counters.1),
        ciphertext: Some(ciphertext.to_vec()),
    };
    if body.encoded_len() + 9 > 65536 {
        return Err(invalid("envelope exceeds bound"));
    }
    let mut wire = vec![header];
    body.encode(&mut wire)
        .map_err(|_| invalid("encoding failed"))?;
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(mac_key).expect("fixed 32-byte MAC key");
    mac.update(&identities.0.serialize());
    mac.update(&identities.1.serialize());
    mac.update(&wire);
    wire.extend_from_slice(&mac.finalize().into_bytes()[..8]);
    Ok(wire)
}

/// Wrap a canonical encoded Signal message, optionally retaining paired KEM fields.
/// Prekey tuple is ordinary optional ID/signed ID; keys are base/sender identity.
/// Nested bytes must already have the same canonical version header and MAC.
/// This checks structure, not authentication or KEM validity, and writes no state.
pub fn encode_prekey(
    version: u8,
    registration: u32,
    prekeys: (Option<u32>, u32),
    keys: (&PublicKey, &IdentityKey),
    nested: &[u8],
    kem: Option<(u32, &[u8])>,
) -> Result<Vec<u8>> {
    let header = version_byte(version)?;
    if nested.len() > 65536 || kem.is_some_and(|(_, bytes)| bytes.len() > 65536) {
        return Err(invalid("envelope exceeds bound"));
    }
    if nested.first() != Some(&header) {
        return Err(invalid("nested version header is not canonical"));
    }
    SignalMessage::try_from(nested).map_err(|_| invalid("invalid nested Signal message"))?;
    let body = PreKeyBody {
        prekey: prekeys.0,
        signed: Some(prekeys.1),
        registration: Some(registration),
        base: Some(keys.0.serialize().to_vec()),
        identity: Some(keys.1.serialize().to_vec()),
        message: Some(nested.to_vec()),
        kem_id: kem.map(|(id, _)| id),
        kem_ciphertext: kem.map(|(_, ciphertext)| ciphertext.to_vec()),
    };
    if body.encoded_len() + 1 > 65536 {
        return Err(invalid("envelope exceeds bound"));
    }
    let mut wire = vec![header];
    body.encode(&mut wire)
        .map_err(|_| invalid("encoding failed"))?;
    Ok(wire)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wacore_libsignal::protocol::{IdentityKey, PreKeySignalMessage, PublicKey, SignalMessage};
    #[test]
    fn exact_native_version_bytes_and_macs_match_independent_vectors() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/signal-wire-vectors.json"
        ))
        .unwrap();
        let bytes = |v: &serde_json::Value| hex::decode(v.as_str().unwrap()).unwrap();
        let sender = IdentityKey::decode(&bytes(&fixture["sender"])).unwrap();
        let receiver = IdentityKey::decode(&bytes(&fixture["receiver"])).unwrap();
        let ratchet = PublicKey::deserialize(&bytes(&fixture["ratchet"])).unwrap();
        let mac_key: [u8; 32] = bytes(&fixture["mac_key"]).try_into().unwrap();
        assert_eq!(fixture["signals"].as_array().unwrap().len(), 24);
        for case in fixture["signals"].as_array().unwrap() {
            let version = case["version"].as_u64().unwrap() as u8;
            let wire = encode_signal(
                version,
                &mac_key,
                (&sender, &receiver),
                &ratchet,
                (
                    case["counter"].as_u64().unwrap() as u32,
                    case["previous"].as_u64().unwrap() as u32,
                ),
                &bytes(&case["ciphertext"]),
            )
            .unwrap();
            assert_eq!(wire, bytes(&case["wire"]));
            let backend = SignalMessage::new(
                version,
                &mac_key,
                ratchet,
                case["counter"].as_u64().unwrap() as u32,
                case["previous"].as_u64().unwrap() as u32,
                &bytes(&case["ciphertext"]),
                &sender,
                &receiver,
            )
            .unwrap();
            if version == 3 {
                assert_eq!(backend.serialized(), wire);
            } else {
                assert_eq!(backend.serialized()[0], 0x43);
                assert_eq!(wire[0], 0x44);
                assert_ne!(backend.serialized(), wire);
            }
            assert!(
                SignalMessage::try_from(wire.as_slice())
                    .unwrap()
                    .verify_mac(&sender, &receiver, &mac_key)
                    .unwrap()
            );
            let mut tampered = wire.clone();
            tampered[0] ^= 1;
            assert!(
                !SignalMessage::try_from(tampered.as_slice())
                    .unwrap()
                    .verify_mac(&sender, &receiver, &mac_key)
                    .unwrap()
            );
        }
        assert_eq!(fixture["prekeys"].as_array().unwrap().len(), 8);
        for case in fixture["prekeys"].as_array().unwrap() {
            let nested = bytes(&case["nested"]);
            let one = case["one_time"].as_u64().map(|n| n as u32);
            let kem_bytes = case["kem"]["ciphertext"]
                .as_str()
                .map(|s| hex::decode(s).unwrap());
            let kem = kem_bytes
                .as_ref()
                .map(|b| (case["kem"]["id"].as_u64().unwrap() as u32, b.as_slice()));
            let wire = encode_prekey(
                case["version"].as_u64().unwrap() as u8,
                128,
                (one, u32::MAX),
                (&ratchet, &sender),
                &nested,
                kem,
            )
            .unwrap();
            assert_eq!(wire, bytes(&case["wire"]));
            let parsed = PreKeySignalMessage::try_from(wire.as_slice()).unwrap();
            assert_eq!(parsed.message().serialized(), nested);
            let metadata = crate::signal::direct::incoming_prekey_kem(&wire).unwrap();
            assert_eq!(metadata.as_ref().map(|m| (m.prekey_id, m.ciphertext)), kem);
            let empty = encode_prekey(
                case["version"].as_u64().unwrap() as u8,
                0,
                (None, 0),
                (&ratchet, &sender),
                &nested,
                Some((0, &[])),
            )
            .unwrap();
            let empty = crate::signal::direct::incoming_prekey_kem(&empty)
                .unwrap()
                .unwrap();
            assert_eq!(empty.prekey_id, 0);
            assert!(empty.ciphertext.is_empty());
            let mut incompatible = nested.clone();
            incompatible[0] ^= 1;
            assert!(
                encode_prekey(
                    case["version"].as_u64().unwrap() as u8,
                    128,
                    (one, u32::MAX),
                    (&ratchet, &sender),
                    &incompatible,
                    kem
                )
                .is_err()
            );
        }
        assert!(encode_signal(2, &mac_key, (&sender, &receiver), &ratchet, (0, 0), &[]).is_err());
        assert!(
            encode_signal(
                4,
                &mac_key,
                (&sender, &receiver),
                &ratchet,
                (0, 0),
                &vec![0; 65536]
            )
            .is_err()
        );
    }
}
