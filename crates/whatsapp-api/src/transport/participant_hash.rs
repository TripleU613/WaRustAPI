//! APK-derived pure participant hashing; no live transport/account dependency.
//! Source: docs/protocol/groups.md participant cache/versioned hash trace.
use crate::model::jid::Jid;

/// Participant hash (`phash`) for a group send.
///
/// Comparison 2.26.37.71 `LX/1jL->A02(Collection<Jid>) -> String`
/// (**confirmed**): take each JID's raw string, sort the strings with natural
/// ordering, feed them to **SHA-1** concatenated with **no separator**, take the
/// **first 6 bytes** of the 20-byte digest, Base64-encode them with the standard
/// alphabet and `NO_WRAP`, and prefix `"1:"`.
///
/// Self-consistency check: six bytes encode to exactly eight Base64 characters
/// with no padding, which with the two-character prefix is exactly the ten
/// characters `LX/0hp->A06(value, 0, 10, …)` bounds `phash` to in the emitter
/// `LX/BpS`. The width matching the emitter's own bound is independent
/// corroboration that this is the right derivation.
///
/// Membership selection is separate. APK 37.71 A0R hashes raw user-key maps
/// A0A (non-PN) and A0B (PN); these maps are not device-keyed. A0N/A0O instead
/// collect their stored DeviceJid sets for the separate AzR V2/V3 hashes.
/// Full caller selection/own-device inventory remains unported. This raw-string
/// helper preserves repeats and sorts by Java UTF-16; it chooses no membership.
/// Evidence: docs/protocol/groups.md participant cache/versioned hash trace.
pub fn participant_hash(jids: &[String]) -> String {
    use sha1::{Digest, Sha1};
    let mut sorted: Vec<&str> = jids.iter().map(String::as_str).collect();
    sorted.sort_unstable_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
    let mut hasher = Sha1::new();
    for jid in &sorted {
        hasher.update(jid.as_bytes());
    }
    let digest = hasher.finalize();
    let mut out = String::from("1:");
    out.push_str(&base64_no_wrap(&digest[..6]));
    out
}

/// The **version 2** participant hash, solved against a value the server itself
/// supplied in one historical primary-device group sample and reproduced exactly.
///
/// On 2026-09-22 the `ack` for our first group fan-out carried
/// `phash="2:…"`. Searching locally for a construction that reproduces those six
/// bytes — every subset of the known membership, four JID renderings, four
/// orderings, five separators, four digests, with and without a group or count
/// prefix — produced exactly one match, and it answers all three questions
/// [`participant_hash`] had left open:
///
/// * the sampled set was each known member's primary device, including the sender;
/// * the JID form is `user.0:device@server`, with literal agent zero and
///   explicit device zero, rather than the display form;
/// * the digest is **SHA-256**, not the SHA-1 of the version 1 form.
///
/// The rest matches version 1: sort the rendered strings, concatenate with no
/// separator, take the first six digest bytes, Base64 them, and prefix the
/// version and a colon.
///
/// This is a local search against one known-correct target, not probing the
/// server. The committed vectors are computed independently and use synthetic
/// identities; the real value that anchored it stays in private lab state.
/// APK 37.71 AzR.A03 now independently confirms the digest/rendering. This
/// convenience helper deduplicates Rust model addresses before hashing; the
/// direct collection helper preserves every entry. The historical sample does
/// not establish complete companion-device coverage. Active feature selection
/// and the complete caller device inventory remain separate requirements.
pub fn participant_hash_v2(jids: &[Jid]) -> String {
    // This historical public helper prepares a set of model addresses. Keep
    // that producer policy outside AzR.A03's duplicate-preserving collection hash.
    let unique: Vec<Jid> = jids
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .cloned()
        .collect();
    participant_device_hash(&unique, DeviceParticipantHashVersion::V2)
}

/// APK 37.71 AzR.A03 selects V3 when feature 0x415b is true, V2 otherwise.
/// This enum does not assert the active account's feature value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceParticipantHashVersion {
    /// `user.0:device@server`, including literal agent zero.
    V2,
    /// DeviceJid.getRawStringWithNoAgent: `user:device@server`.
    V3,
}

/// APK AzR.A03's selected device-collection hash, preserving repeated entries.
/// Device components are interpreted from the Rust model; this is not an APK
/// DeviceJid subtype validator, membership selector or live feature negotiator.
/// Well-formed Unicode sorts by Java UTF-16, then UTF-8 bytes are concatenated.
/// Android default-charset/runtime equivalence is not exercised by this helper.
pub fn participant_device_hash(jids: &[Jid], version: DeviceParticipantHashVersion) -> String {
    use sha2::{Digest, Sha256};
    // APK 37.71 AzR.A03+00ce..0108 hardcodes ".0:" for V2;
    // +003c..0046 delegates V3 to DeviceJid.getRawStringWithNoAgent.
    let mut rendered: Vec<String> = jids
        .iter()
        .map(|jid| match version {
            DeviceParticipantHashVersion::V2 => {
                format!("{}.0:{}@{}", jid.user, jid.device, jid.server.as_str())
            }
            DeviceParticipantHashVersion::V3 => {
                format!("{}:{}@{}", jid.user, jid.device, jid.server.as_str())
            }
        })
        .collect();
    rendered.sort_unstable_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
    // No rendered-string dedup here: distinct agent-qualified device values
    // can render identically. The caller's device HashSet is a separate stage.
    let mut hasher = Sha256::new();
    for jid in &rendered {
        hasher.update(jid.as_bytes());
    }
    let digest = hasher.finalize();
    let mut out = String::from(match version {
        DeviceParticipantHashVersion::V2 => "2:",
        DeviceParticipantHashVersion::V3 => "3:",
    });
    out.push_str(&base64_no_wrap(&digest[..6]));
    out
}

/// Standard-alphabet Base64 without padding-free wrapping, matching Android's
/// `Base64.encodeToString(bytes, NO_WRAP)`. Six input bytes divide evenly into
/// four-character groups, so no padding is produced for this caller.
fn base64_no_wrap(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let take = chunk.len() + 1;
        for i in 0..take {
            out.push(ALPHABET[((n >> (18 - 6 * i)) & 0x3f) as usize] as char);
        }
        for _ in take..4 {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn participant_hashes_match_executed_java_collection_reference() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mex/2.26.37.71-participant-hash-reference.json"
        ))
        .unwrap();
        assert_eq!(fixture["cases"].as_array().unwrap().len(), 114);
        for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
            let raw: Vec<String> = case["input"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect();
            let jids: Vec<Jid> = raw.iter().map(|v| Jid::parse(v).unwrap()).collect();
            for (jid, fields) in jids.iter().zip(case["fields"].as_array().unwrap()) {
                assert_eq!(jid.user, fields["user"].as_str().unwrap());
                assert_eq!(u64::from(jid.agent), fields["agent"].as_u64().unwrap());
                assert_eq!(u64::from(jid.device), fields["device"].as_u64().unwrap());
                assert_eq!(jid.server.as_str(), fields["server"].as_str().unwrap());
            }
            assert_eq!(
                participant_hash(&raw),
                case["expected"]["v1"].as_str().unwrap(),
                "case {index}"
            );
            assert_eq!(
                participant_device_hash(&jids, DeviceParticipantHashVersion::V2),
                case["expected"]["v2"].as_str().unwrap(),
                "case {index}"
            );
            assert_eq!(
                participant_device_hash(&jids, DeviceParticipantHashVersion::V3),
                case["expected"]["v3"].as_str().unwrap(),
                "case {index}"
            );
            assert_eq!(
                participant_hash_v2(&jids),
                case["expected"]["v2_set"].as_str().unwrap(),
                "case {index}"
            );
        }
    }

    /// Vectors computed independently with Python's hashlib and base64 before
    /// this function existed: AD-form JIDs (`user.agent:device@server`, with the
    /// zeros written out), sorted as strings, concatenated with no separator,
    /// SHA-256, first six digest bytes, standard Base64, `"2:"` prefix.
    ///
    /// The construction was solved against a `phash` the server returned in an
    /// `ack`; that real value and the identities behind it stay in private lab
    /// state, so these vectors use synthetic ones.
    #[test]
    fn participant_hash_v2_matches_independent_vectors() {
        let jids = |v: &[&str]| v.iter().map(|s| Jid::parse(s).unwrap()).collect::<Vec<_>>();
        assert_eq!(
            participant_hash_v2(&jids(&["111111111111111@lid", "222222222222222@lid"])),
            "2:79jEAyGD"
        );
        assert_eq!(
            participant_hash_v2(&jids(&[
                "100000000000001@lid",
                "200000000000002@lid",
                "300000000000003@lid",
            ])),
            "2:/EhFlOmO"
        );
        assert_eq!(
            participant_hash_v2(&jids(&["999999999999999@lid"])),
            "2:VByC+b12"
        );
        // Order and duplicates must not change it; the set does.
        assert_eq!(
            participant_hash_v2(&jids(&[
                "222222222222222@lid",
                "111111111111111@lid",
                "111111111111111@lid",
            ])),
            "2:79jEAyGD"
        );
        assert_ne!(
            participant_hash_v2(&jids(&["111111111111111@lid"])),
            "2:79jEAyGD",
            "dropping a member must change the hash"
        );
        // The AD rendering is the point: the display form omits `.0:0`, and
        // hashing that instead silently produces a different, wrong value.
        assert_ne!(
            participant_hash_v2(&jids(&["999999999999999@lid"])),
            participant_hash(&["999999999999999@lid".to_owned()])
        );
        // Version 1 and version 2 are different constructions, not a rename.
        assert!(participant_hash_v2(&jids(&["999999999999999@lid"])).starts_with("2:"));
    }

    /// Vectors computed independently with Python's hashlib and base64, not by
    /// running this implementation: sorted raw JID strings concatenated with no
    /// separator, SHA-1, first six digest bytes, standard Base64, "1:" prefix.
    #[test]
    fn participant_hash_matches_independent_vectors() {
        let jids = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert_eq!(
            participant_hash(&jids(&["15551234567@s.whatsapp.net"])),
            "1:XS9Zi8u/"
        );
        let three = jids(&[
            "15551234567@s.whatsapp.net",
            "15559876543@s.whatsapp.net",
            "120363000000000001@g.us",
        ]);
        assert_eq!(participant_hash(&three), "1:Lz4D7Wb2");
        // Sorting is part of the derivation: input order must not matter.
        let shuffled = jids(&[
            "120363000000000001@g.us",
            "15559876543@s.whatsapp.net",
            "15551234567@s.whatsapp.net",
        ]);
        assert_eq!(participant_hash(&shuffled), participant_hash(&three));
        // The emitter bounds phash to ten characters; six digest bytes encode to
        // eight Base64 characters, so the prefixed value is exactly ten.
        assert_eq!(participant_hash(&three).len(), 10);
        assert_eq!(participant_hash(&jids(&[])).len(), 10);
        // Membership changes the hash, which is the point of it.
        assert_ne!(
            participant_hash(&three),
            participant_hash(&jids(&[
                "15551234567@s.whatsapp.net",
                "15559876543@s.whatsapp.net",
            ]))
        );
    }
}
