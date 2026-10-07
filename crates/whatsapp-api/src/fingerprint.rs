//! APK 2.26.37.71 native identifier-bound fingerprint display primitive.
//!
//! Callers must supply the selected scheme's already normalized identifier and
//! complete key inventory. This module does not select the current APK scheme,
//! fetch device keys, parse/compare QR codes, or commit human verification.
//! Evidence and independent fixtures: docs/protocol/safety-number.md.

use crate::{Error, Result};
use serde::Serialize;
use sha2::{Digest, Sha512};
use std::fmt::Write;

/// Local resource limit, not an APK or server inventory limit.
pub const MAX_IDENTITY_KEYS: usize = 128;
/// Local resource limit for a normalized display identifier, in bytes.
pub const MAX_DISPLAY_IDENTIFIER_BYTES: usize = 256;

/// Compute one native 30-digit side from raw 32-byte public identity keys.
///
/// `identifier` is the display identifier, not a full QR/JID string: the native
/// V3 path uses its stable PN input, V4 the parsed type-18 LID user. No implicit
/// normalization or deduplication occurs. Results cannot establish that these
/// inputs match the active account, peer, APK scheme, or device inventory.
/// Empty identifiers are refused as a local input policy, not a server rule.
pub fn native_display(identifier: &str, keys: &[[u8; 32]]) -> Result<String> {
    if identifier.is_empty()
        || identifier.len() > MAX_DISPLAY_IDENTIFIER_BYTES
        || !identifier.bytes().all(|b| (1..=127).contains(&b))
    {
        return Err(Error::invalid(
            "fingerprint.display_identifier",
            "requires a nonempty normalized ASCII identifier without NUL, at most 256 bytes",
        ));
    }
    if keys.is_empty() || keys.len() > MAX_IDENTITY_KEYS {
        return Err(Error::invalid(
            "fingerprint.identity_keys",
            "requires 1..128 raw 32-byte public identity keys",
        ));
    }
    // 37.71 main 0x712ef0..0x712f84 sorts unsigned raw 32-byte keys;
    // 0x712dc0..0x712e58 prepends 0x05; create 0x7146bc..0x714730
    // concatenates serialized keys without deduplication. Input order is irrelevant.
    let mut sorted = keys.to_vec();
    sorted.sort_unstable();
    let mut serialized = Vec::with_capacity(sorted.len() * 33);
    for key in sorted {
        serialized.push(0x05);
        serialized.extend_from_slice(&key);
    }
    // V3 0x7139e4 / V4 0x714c70: display seed 00 00 || K || identifier.
    // Shared helper 0x713938: SHA512(seed || K), then 5199 SHA512(digest || K).
    // This is distinct from the identifier-free H(K,K) used for QR version 1.
    let mut hash = Sha512::new();
    hash.update([0, 0]);
    hash.update(&serialized);
    hash.update(identifier.as_bytes());
    hash.update(&serialized);
    let mut digest = hash.finalize();
    for _ in 1..5200 {
        let mut hash = Sha512::new();
        hash.update(digest);
        hash.update(&serialized);
        digest = hash.finalize();
    }
    // Display 0x714d58..0x714f38: six big-endian 40-bit groups modulo
    // 100000, each formatted %.5lu. Rust decimal output always uses ASCII.
    let mut display = String::with_capacity(30);
    for bytes in digest[..30].as_chunks::<5>().0 {
        let value = bytes
            .iter()
            .fold(0u64, |value, byte| (value << 8) | u64::from(*byte));
        write!(&mut display, "{:05}", value % 100_000)
            .map_err(|_| Error::invalid("fingerprint.display", "cannot format number"))?;
    }
    Ok(display)
}

/// Caller-supplied native display result; never a verified account assertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeDisplayPair {
    /// Native algorithm and evidence build, not the account's selected scheme.
    pub algorithm: &'static str,
    /// Provenance of inputs; no account/directory selection has been performed.
    pub input_authority: &'static str,
    /// Local 30-digit display, preserving the caller's side orientation.
    pub local: String,
    /// Remote 30-digit display, preserving the caller's side orientation.
    pub remote: String,
    /// Lexically sorted concatenation as in Java KKq.A00.
    pub combined: String,
}

/// Compute both sides and their orientation-independent 60-digit display.
/// This does not compare fingerprints with another device or mark a key verified.
pub fn native_display_pair(
    local_identifier: &str,
    local_keys: &[[u8; 32]],
    remote_identifier: &str,
    remote_keys: &[[u8; 32]],
) -> Result<NativeDisplayPair> {
    let local = native_display(local_identifier, local_keys)?;
    let remote = native_display(remote_identifier, remote_keys)?;
    // APK 37.71 0jc.A0U constructs KKq from both displays; KKq.A00
    // 0000..003a joins in lexical order, not caller/local-first order.
    let combined = if local <= remote {
        format!("{local}{remote}")
    } else {
        format!("{remote}{local}")
    };
    Ok(NativeDisplayPair {
        algorithm: "apk_2.26.37.71_native_identifier_bound",
        input_authority: "caller_supplied_selection_unverified",
        local,
        remote,
        combined,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../fixtures/fingerprint/native-display-3771.json"
        ))
        .unwrap()
    }

    fn keys(case: &serde_json::Value) -> Vec<[u8; 32]> {
        case["raw_identity_keys_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|key| {
                hex::decode(key.as_str().unwrap())
                    .unwrap()
                    .try_into()
                    .unwrap()
            })
            .collect()
    }

    #[test]
    fn matches_independent_arm64_hash_flow_fixtures() {
        let fixture = fixture();
        for case in fixture["cases"].as_array().unwrap() {
            let identifier = case["display_identifier"].as_str().unwrap();
            let mut keys = keys(case);
            assert_eq!(native_display(identifier, &keys).unwrap(), case["display"]);
            keys.reverse();
            assert_eq!(native_display(identifier, &keys).unwrap(), case["display"]);
        }
        // The third fixture intentionally repeats a key: the native inventory
        // concatenation preserves that duplicate, changing the display.
        assert_ne!(
            fixture["cases"][0]["display"],
            fixture["cases"][2]["display"]
        );
    }

    #[test]
    fn pair_join_is_symmetric_but_side_labels_are_preserved() {
        let fixture = fixture();
        let cases = fixture["cases"].as_array().unwrap();
        let first = native_display_pair(
            "synthetic-local",
            &keys(&cases[0]),
            "synthetic-remote",
            &keys(&cases[1]),
        )
        .unwrap();
        let second = native_display_pair(
            "synthetic-remote",
            &keys(&cases[1]),
            "synthetic-local",
            &keys(&cases[0]),
        )
        .unwrap();
        assert_eq!(first.local, second.remote);
        assert_eq!(first.remote, second.local);
        assert_eq!(first.combined, second.combined);
        assert_eq!(
            first.combined,
            "128588181678439595160634291930622605791988983247407933286410"
        );
        assert_eq!(
            first.input_authority,
            "caller_supplied_selection_unverified"
        );
        assert_ne!(
            native_display("synthetic-local@lid", &keys(&cases[0])).unwrap(),
            first.local
        );
    }

    #[test]
    fn invalid_or_unbounded_inputs_refuse_without_echo() {
        let key = [[1u8; 32]];
        for id in ["", "secret\0value", "secret\u{80}value", &"s".repeat(257)] {
            let error = native_display(id, &key).unwrap_err().to_string();
            assert!(!error.contains("secret"));
        }
        assert!(native_display("synthetic", &[]).is_err());
        assert!(native_display("synthetic", &[[1; 32]; 129]).is_err());
        assert_eq!(
            native_display(&"s".repeat(256), &[[1; 32]; 128])
                .unwrap()
                .len(),
            30
        );
    }
}
