//! Independent synthetic vectors for the APK's hybrid initial KDF and Signal
//! version 3/4 wire format (ports of the former Python reproducers).
//!
//! Independence is the point: HMAC is computed here from the `hmac`/`sha2`
//! primitives and protobuf bytes are hand-encoded, never through
//! `whatsapp_api`'s HKDF or wire code that the committed vectors check.

use std::path::Path;

use hmac::{Hmac, KeyInit, Mac};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Set {
    /// docs/protocol/hybrid-kdf-vectors.json
    HybridKdf,
    /// docs/protocol/signal-wire-vectors.json
    SignalWire,
}

pub fn run(root: &Path, set: Set, check: bool) -> Result<(), String> {
    // Serialize the structs directly: serde_json's Value map would sort keys.
    let (file, encoded, summary) = match set {
        Set::HybridKdf => (
            "hybrid-kdf-vectors.json",
            serde_json::to_string_pretty(&hybrid_kdf()),
            "16 hybrid initial KDF vectors",
        ),
        Set::SignalWire => (
            "signal-wire-vectors.json",
            serde_json::to_string_pretty(&signal_wire()),
            "24 Signal and 8 prekey wire vectors",
        ),
    };
    let mut encoded = encoded.map_err(|e| e.to_string())?;
    encoded.push('\n');
    let path = root.join("docs/protocol").join(file);
    if check {
        let committed =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if committed != encoded {
            return Err(format!("{file} differs from the regenerated vectors"));
        }
        println!("Verified {summary}");
    } else {
        std::fs::write(&path, encoded).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("Wrote {summary}");
    }
    Ok(())
}

fn hmac_sha256(key: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut mac =
        <Hmac<Sha256> as KeyInit>::new_from_slice(key).expect("HMAC takes any key length");
    for part in parts {
        mac.update(part);
    }
    mac.finalize().into_bytes().into()
}

// Field order matches the former Python dicts so the committed JSON is
// reproduced byte for byte.
#[derive(Serialize)]
struct HybridKdf {
    scope: &'static str,
    label: &'static str,
    cases: Vec<KdfCase>,
}

#[derive(Serialize)]
struct KdfCase {
    name: String,
    agreements: Vec<String>,
    one_time: Option<String>,
    kem_shared: String,
    root: String,
    chain: String,
}

const KDF_LABEL: &str = "WhisperText_X25519_SHA-256_CRYSTALS-KYBER-1024";

fn hybrid_kdf() -> HybridKdf {
    let mut cases = Vec::new();
    for pattern in 0..8 {
        let values: Vec<[u8; 32]> = (0..5)
            .map(|index| match pattern {
                0 => [0; 32],
                1 => [255; 32],
                _ => Sha256::digest(format!("synthetic-kdf-{pattern}-{index}")).into(),
            })
            .collect();
        for one_time in [false, true] {
            let mut material = vec![0xff; 32];
            for agreement in &values[..if one_time { 4 } else { 3 }] {
                material.extend_from_slice(agreement);
            }
            material.extend_from_slice(&values[4]);
            let extracted = hmac_sha256(&[0; 32], &[&material]);
            let label = KDF_LABEL.as_bytes();
            let root = hmac_sha256(&extracted, &[label, &[1]]);
            let chain = hmac_sha256(&extracted, &[&root, label, &[2]]);
            cases.push(KdfCase {
                // Python's f-string spelling of the bool.
                name: format!(
                    "pattern-{pattern}-one-time-{}",
                    if one_time { "True" } else { "False" }
                ),
                agreements: values[..3].iter().map(hex::encode).collect(),
                one_time: one_time.then(|| hex::encode(values[3])),
                kem_shared: hex::encode(values[4]),
                root: hex::encode(root),
                chain: hex::encode(chain),
            });
        }
    }
    HybridKdf {
        scope: "Synthetic already-computed agreements; Python standard HMAC, not APK execution or DH/KEM verification.",
        label: KDF_LABEL,
        cases,
    }
}

#[derive(Serialize)]
struct SignalWire {
    scope: &'static str,
    sender: String,
    receiver: String,
    ratchet: String,
    mac_key: String,
    signals: Vec<SignalCase>,
    prekeys: Vec<PrekeyCase>,
}

#[derive(Serialize)]
struct SignalCase {
    version: u8,
    counter: u32,
    previous: u32,
    ciphertext: String,
    wire: String,
}

#[derive(Serialize)]
struct PrekeyCase {
    version: u8,
    registration: u32,
    signed: u32,
    one_time: Option<u32>,
    kem: Option<KemCase>,
    nested: String,
    wire: String,
}

#[derive(Serialize)]
struct KemCase {
    id: u32,
    ciphertext: String,
}

fn varint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        out.push((value & 127) as u8 | 128);
        value >>= 7;
    }
    out.push(value as u8);
}

fn number(out: &mut Vec<u8>, tag: u64, value: u64) {
    varint(out, tag << 3);
    varint(out, value);
}

fn blob(out: &mut Vec<u8>, tag: u64, value: &[u8]) {
    varint(out, tag << 3 | 2);
    varint(out, value.len() as u64);
    out.extend_from_slice(value);
}

fn key(range: std::ops::Range<u8>) -> Vec<u8> {
    std::iter::once(5).chain(range).collect()
}

fn signal_wire() -> SignalWire {
    let sender = key(1..33);
    let receiver = key(33..65);
    let ratchet = key(65..97);
    let mac_key: Vec<u8> = (0..32).collect();
    let (mut signals, mut prekeys) = (Vec::new(), Vec::new());
    for version in [3u8, 4] {
        for (counter, previous) in [(0, 0), (1, 0), (127, 128), (u32::MAX, u32::MAX)] {
            for length in [0u8, 16, 32] {
                let ciphertext: Vec<u8> = (0..length).collect();
                let mut wire = vec![version * 17];
                blob(&mut wire, 1, &ratchet);
                number(&mut wire, 2, counter.into());
                number(&mut wire, 3, previous.into());
                blob(&mut wire, 4, &ciphertext);
                let mac = hmac_sha256(&mac_key, &[&sender, &receiver, &wire]);
                wire.extend_from_slice(&mac[..8]);
                signals.push(SignalCase {
                    version,
                    counter,
                    previous,
                    ciphertext: hex::encode(&ciphertext),
                    wire: hex::encode(&wire),
                });
            }
        }
        let inner = hex::decode(&signals.last().expect("signals").wire).expect("own hex");
        for one_time in [None, Some(0u32)] {
            for with_kem in [false, true] {
                let kem = with_kem.then(|| KemCase {
                    id: u32::MAX,
                    ciphertext: hex::encode([0u8; 1568]),
                });
                let mut wire = vec![version * 17];
                if let Some(one_time) = one_time {
                    number(&mut wire, 1, one_time.into());
                }
                blob(&mut wire, 2, &ratchet);
                blob(&mut wire, 3, &sender);
                blob(&mut wire, 4, &inner);
                number(&mut wire, 5, 128);
                number(&mut wire, 6, u32::MAX.into());
                if let Some(kem) = &kem {
                    number(&mut wire, 7, kem.id.into());
                    blob(&mut wire, 8, &[0u8; 1568]);
                }
                prekeys.push(PrekeyCase {
                    version,
                    registration: 128,
                    signed: u32::MAX,
                    one_time,
                    kem,
                    nested: hex::encode(&inner),
                    wire: hex::encode(&wire),
                });
            }
        }
    }
    SignalWire {
        scope: "Synthetic serialization and HMAC inputs; not APK execution, encryption or valid KEM ciphertext.",
        sender: hex::encode(&sender),
        receiver: hex::encode(&receiver),
        ratchet: hex::encode(&ratchet),
        mac_key: hex::encode(&mac_key),
        signals,
        prekeys,
    }
}

#[cfg(test)]
mod tests {
    fn pretty<T: serde::Serialize>(value: &T) -> String {
        serde_json::to_string_pretty(value).unwrap() + "\n"
    }

    #[test]
    fn committed_vectors_reproduce_exactly() {
        assert_eq!(
            pretty(&super::hybrid_kdf()),
            include_str!("../../docs/protocol/hybrid-kdf-vectors.json")
        );
        assert_eq!(
            pretty(&super::signal_wire()),
            include_str!("../../docs/protocol/signal-wire-vectors.json")
        );
    }
}
