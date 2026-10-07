//! Native V4 binary payload serialization and explicit-input comparison, APK 2.26.37.71.
//! This exports comparison data, not a QR image, authentication, scheme selection
//! or a human-verification result. Evidence: docs/protocol/safety-number.md.
use super::{MAX_DISPLAY_IDENTIFIER_BYTES, serialized_keys};
use crate::{Error, Result};
use prost::Message;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

/// Native QR hosted-state enum; distinct from saved identity attestation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostedState {
    /// Native descriptor E2EE=0.
    E2ee,
    /// Native descriptor HOSTED=1.
    Hosted,
}

/// Explicit caller-supplied native V4 individual inputs.
/// Keys and full JID strings are private comparison data; Debug redacts them.
pub struct V4Side<'a> {
    /// Selected raw public-key inventory, not inferred from a primary session.
    pub keys: &'a [[u8; 32]],
    /// Full ASCII LID JID used in QR tag 3, not its parsed display user.
    pub lid_jid: &'a str,
    /// Already normalized stable PN identifier, when selected by the caller.
    pub pn_identifier: Option<&'a str>,
    /// Caller-selected username identifier; presence can suppress local PN.
    pub username_identifier: Option<&'a str>,
    /// None represents the APK's absent/sentinel state; never encoded as -1.
    pub hosted_state: Option<HostedState>,
}
impl std::fmt::Debug for V4Side<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V4Side")
            .field("key_count", &self.keys.len())
            .field("has_pn", &self.pn_identifier.is_some())
            .field("has_username", &self.username_identifier.is_some())
            .field("hosted_state", &self.hosted_state)
            .finish_non_exhaustive()
    }
}

// 37.71 merged descriptor 0x740b60, fields 0x740c80..0x740de8;
// combined descriptor 0x740c08. These are the recovered native schema,
// not Java legacy M76/BTe whose nested tag 2 has different meaning.
#[derive(Clone, PartialEq, Message)]
struct Individual {
    #[prost(bytes = "vec", optional, tag = "1")]
    public_key: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pn_identifier: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "3")]
    lid_identifier: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "4")]
    username_identifier: Option<Vec<u8>>,
    // Native HostedState descriptor 0x740f58 declares only 0/1.
    // These nonnegative enum members have int32 varint wire encoding; the
    // typed encoder below cannot emit an unknown enum or -1 sentinel.
    #[prost(int32, optional, tag = "5")]
    hosted_state: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "6")]
    hashed_public_key: Option<Vec<u8>>,
}
#[derive(Clone, PartialEq, Message)]
struct Combined {
    #[prost(uint32, optional, tag = "1")]
    version: Option<u32>,
    #[prost(message, optional, tag = "2")]
    local: Option<Individual>,
    #[prost(message, optional, tag = "3")]
    remote: Option<Individual>,
}

fn ascii(value: &str) -> Result<Vec<u8>> {
    // MCF ASCII validation 37.71 merged 0x531200 accepts bytes 1..127,
    // including an empty value. Length bound is our resource policy.
    if value.len() > MAX_DISPLAY_IDENTIFIER_BYTES
        || !value.bytes().all(|byte| (1..=127).contains(&byte))
    {
        return Err(Error::invalid(
            "fingerprint.qr_identifier",
            "requires ASCII without NUL, at most 256 bytes",
        ));
    }
    Ok(value.as_bytes().to_vec())
}

fn individual(side: &V4Side<'_>, version: u8, local: bool) -> Result<Individual> {
    if side.lid_jid.is_empty() {
        return Err(Error::invalid(
            "fingerprint.lid_jid",
            "full LID JID input is missing",
        ));
    }
    let keys = serialized_keys(side.keys)?;
    let mut public_key = None;
    let mut hashed_public_key = None;
    if version == 0 {
        public_key = Some(keys);
    } else {
        // Native create 0x714764..0x71476c stores H(K,K), not the display
        // digest. 0x713938 hashes seed||K then 5199 digest||K rounds.
        let mut hash = Sha512::new();
        hash.update(&keys);
        hash.update(&keys);
        let mut digest = hash.finalize();
        for _ in 1..5200 {
            let mut hash = Sha512::new();
            hash.update(digest);
            hash.update(&keys);
            digest = hash.finalize();
        }
        hashed_public_key = Some(digest.to_vec());
    }
    let pn = side.pn_identifier.map(ascii).transpose()?;
    let username = side.username_identifier.map(ascii).transpose()?;
    Ok(Individual {
        public_key,
        // Main 0x715050..0x71506c suppresses PN with present username
        // only when flag bit 0 is set. Combined 0x714974..0x714998 passes
        // flag 1 for local and flag 0 for remote; do not suppress both.
        pn_identifier: if local && username.is_some() {
            None
        } else {
            pn
        },
        lid_identifier: Some(ascii(side.lid_jid)?),
        username_identifier: username,
        hosted_state: side.hosted_state.map(|state| match state {
            HostedState::E2ee => 0,
            HostedState::Hosted => 1,
        }),
        hashed_public_key,
    })
}

/// Encode native V4 combined fingerprint comparison bytes for version 0 or 1.
///
/// Version 0 carries sorted serialized keys; version 1 their identifier-free
/// digest. Full LID/optional identifier bytes are preserved without normalization.
/// Callers must provide actual chosen inputs; this function does not validate
/// APK JID grammar, attest device membership, choose the active scheme, render a
/// QR image, compare a peer payload or commit verification. Optional identifiers
/// can be empty as in the native ASCII primitive. Serialization alone does not
/// establish that comparison requirements (including PN/username) are satisfied.
pub fn encode_v4(local: &V4Side<'_>, remote: &V4Side<'_>, version: u8) -> Result<Vec<u8>> {
    if version > 1 {
        return Err(Error::invalid(
            "fingerprint.qr_version",
            "native QR version must be 0 or 1",
        ));
    }
    Ok(Combined {
        // Explicit Some(0) preserves the mandatory version presence bit.
        version: Some(u32::from(version)),
        local: Some(individual(local, version, true)?),
        remote: Some(individual(remote, version, false)?),
    }
    .encode_to_vec())
}

/// Local payload bound, not a WhatsApp limit. Checked before protobuf decoding.
pub const MAX_COMPARISON_PAYLOAD_BYTES: usize = 64 * 1024;

/// How the scanned sides were paired with the explicit known local identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanOrientation {
    /// Known local LID matched scanned local; native result is remapped.
    LocalFirst,
    /// Known local LID matched scanned remote; swap without remapping.
    RemoteFirst,
}

/// Internal native comparison result against caller-selected inputs only.
/// No identifier, key, scanned bytes or human/server verification is returned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct V4Comparison {
    /// APK internal result code, not a WhatsApp server reason or UI label.
    pub status: i32,
    /// Zero status only; does not assert current inventory or human verification.
    pub matches_explicit_inputs: bool,
    /// Null until a payload version was decoded.
    pub payload_version: Option<u32>,
    /// Null when conversion failed or no scanned LID matched known local.
    pub orientation: Option<ScanOrientation>,
    /// Explicit-input scope, never account-derived selection.
    pub input_authority: &'static str,
    /// Schema decoding is implemented with prost; full protobuf-C edge equivalence is unverified.
    pub parser_authority: &'static str,
}

fn comparison(
    status: i32,
    version: Option<u32>,
    orientation: Option<ScanOrientation>,
) -> V4Comparison {
    V4Comparison {
        status,
        matches_explicit_inputs: status == 0,
        payload_version: version,
        orientation,
        input_authority: "caller_supplied_selection_unverified",
        parser_authority: "prost_native_schema_edge_equivalence_unverified",
    }
}

fn ascii_bytes(value: &[u8]) -> bool {
    // APK merged ASCII creator 0x531200 accepts empty and bytes 1..127.
    value.iter().all(|byte| (1..=127).contains(byte))
}

fn validate_scanned(side: &Individual, version: u32) -> std::result::Result<(), i32> {
    // APK 37.71 converter 0x7150ac..0x715264; branch order matters.
    let lid = side.lid_identifier.as_ref().ok_or(-43)?;
    if !ascii_bytes(lid) {
        return Err(-1);
    }
    if side
        .pn_identifier
        .as_ref()
        .is_some_and(|pn| !ascii_bytes(pn))
    {
        return Err(-44);
    }
    if side
        .username_identifier
        .as_ref()
        .is_some_and(|username| !ascii_bytes(username))
    {
        return Err(-23);
    }
    if side
        .hosted_state
        .is_some_and(|state| !matches!(state, 0 | 1))
    {
        return Err(-38);
    }
    let key = if version == 0 {
        &side.public_key
    } else {
        &side.hashed_public_key
    };
    if key.is_none() {
        return Err(-4);
    }
    Ok(())
}

fn compare_sides(known: [&Individual; 2], scanned: [&Individual; 2], version: u32) -> i32 {
    // Actual native branch statuses executed by native_v4_comparison_check.py;
    // 37.71 core 0x7152fc..0x7155d8. Local LID is the wrapper's orientation guard.
    if known[1].lid_identifier != scanned[1].lid_identifier {
        return -12;
    }
    for side in 0..2 {
        let a = if version == 0 {
            &known[side].public_key
        } else {
            &known[side].hashed_public_key
        };
        let b = if version == 0 {
            &scanned[side].public_key
        } else {
            &scanned[side].hashed_public_key
        };
        if a != b {
            return if side == 0 { -14 } else { -15 };
        }
    }
    for side in 0..2 {
        if [known[side], scanned[side]]
            .iter()
            .any(|value| value.pn_identifier.is_none() && value.username_identifier.is_none())
        {
            return if side == 0 { -39 } else { -41 };
        }
    }
    for side in 0..2 {
        if let (Some(a), Some(b)) = (&known[side].pn_identifier, &scanned[side].pn_identifier)
            && a != b
        {
            return if side == 0 { -17 } else { -19 };
        }
    }
    for side in 0..2 {
        match (
            &known[side].username_identifier,
            &scanned[side].username_identifier,
        ) {
            (None, None) => {}
            (Some(a), Some(b)) if a.eq_ignore_ascii_case(b) => {}
            // APK flag 1 reaches MCFStringCompare/strncasecmp. Accepted ASCII
            // equality plus exact length is independently executed against
            // the pinned merged library and public Bionic fixture. This is
            // not Unicode normalization or a claim about every Android libc.
            (Some(_), Some(_)) => return if side == 0 { -24 } else { -29 },
            (Some(_), None) => return if side == 0 { -26 } else { -31 },
            (None, Some(_)) => return if side == 0 { -28 } else { -33 },
        }
    }
    // Native 0x7154f0..0x715584: both scanned states absent selects legacy
    // success regardless of known states. All 81 combinations executed.
    if scanned.iter().all(|side| side.hosted_state.is_none()) {
        return 0;
    }
    for side in 0..2 {
        if known[side].hosted_state != scanned[side].hosted_state {
            return if side == 0 { -34 } else { -36 };
        }
    }
    0
}

fn remap(code: i32) -> i32 {
    // APK 37.71 0x7152b8 with signed table at 0x302404, executed independently.
    // Only the local-first orientation uses this table. Core emits this subset.
    match code {
        -41 => -42,
        -39 => -40,
        -36 => -37,
        -34 => -35,
        -31 => -32,
        -29 => -30,
        -26 => -27,
        -24 => -25,
        -19 => -18,
        -17 => -16,
        -12 => -11,
        0 | -33 | -28 | -15 | -14 => code,
        _ => unreachable!("comparison core emitted an untraced status"),
    }
}

/// Compare native V4 binary protobuf with explicit known local/remote inputs.
///
/// Implements traced conversion, orientation and comparison rules; no store,
/// network, image decoding, scheme/inventory selection or verification commit.
/// Known parameters retain both optional PN/username values, unlike local QR
/// serialization's PN omission. Returns native internal statuses for decoded
/// payload failures/mismatches. Empty/oversized buffers refuse as local resource
/// policy rather than emulating the APK's empty-buffer abort. Prost schema
/// parsing is used; duplicate-field/unknown-field edge equivalence with native
/// protobuf-C is unverified. ASCII username semantics are fixture-validated,
/// not Unicode normalization or a guarantee across all Android libc versions.
pub fn compare_v4(local: &V4Side<'_>, remote: &V4Side<'_>, payload: &[u8]) -> Result<V4Comparison> {
    if payload.is_empty() || payload.len() > MAX_COMPARISON_PAYLOAD_BYTES {
        return Err(Error::invalid(
            "fingerprint.qr_payload",
            "requires nonempty binary protobuf at most 64 KiB",
        ));
    }
    // Wrapper 37.71 0x714a60..0x714c70: unpack/version/side/LID presence
    // checks precede per-side conversion; early failures are not remapped.
    let Ok(parsed) = Combined::decode(payload) else {
        return Ok(comparison(-8, None, None));
    };
    let Some(version) = parsed.version else {
        return Ok(comparison(-5, None, None));
    };
    if version > 1 {
        return Ok(comparison(-10, Some(version), None));
    }
    let (Some(first), Some(second)) = (parsed.local, parsed.remote) else {
        return Ok(comparison(-2, Some(version), None));
    };
    if first.lid_identifier.is_none() || second.lid_identifier.is_none() {
        return Ok(comparison(-43, Some(version), None));
    }
    for side in [&first, &second] {
        if let Err(code) = validate_scanned(side, version) {
            return Ok(comparison(code, Some(version), None));
        }
    }
    // Params constructor 0x71916c retains PN and username separately. Do not
    // apply QR serializer's local PN suppression to known comparison inputs.
    let known_local = individual(local, version as u8, false)?;
    let known_remote = individual(remote, version as u8, false)?;
    let (ordered, orientation) = if known_local.lid_identifier == first.lid_identifier {
        ([&first, &second], ScanOrientation::LocalFirst)
    } else if known_local.lid_identifier == second.lid_identifier {
        ([&second, &first], ScanOrientation::RemoteFirst)
    } else {
        return Ok(comparison(-13, Some(version), None));
    };
    let code = compare_sides([&known_local, &known_remote], ordered, version);
    Ok(comparison(
        if orientation == ScanOrientation::LocalFirst {
            remap(code)
        } else {
            code
        },
        Some(version),
        Some(orientation),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apk_wire_fixtures_preserve_version_keys_orientation_and_optional_presence() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/fingerprint/native-qr-3771.json"
        ))
        .unwrap();
        let keys = |name: &str| -> Vec<[u8; 32]> {
            fixture[name]["raw_identity_keys_hex"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| {
                    hex::decode(x.as_str().unwrap())
                        .unwrap()
                        .try_into()
                        .unwrap()
                })
                .collect()
        };
        let local_keys = keys("local");
        let remote_keys = keys("remote");
        let local = V4Side {
            keys: &local_keys,
            lid_jid: "222222222222222@lid",
            pn_identifier: Some("15550100"),
            username_identifier: Some("synthetic-local"),
            hosted_state: Some(HostedState::E2ee),
        };
        let remote = V4Side {
            keys: &remote_keys,
            lid_jid: "333333333333333@lid",
            pn_identifier: Some("15550101"),
            username_identifier: Some("synthetic-remote"),
            hosted_state: Some(HostedState::Hosted),
        };
        for version in 0..=1 {
            let bytes = encode_v4(&local, &remote, version).unwrap();
            assert_eq!(
                hex::encode(&bytes),
                fixture[format!("version_{version}_hex")]
            );
            let parsed = Combined::decode(bytes.as_slice()).unwrap();
            assert_eq!(parsed.version, Some(u32::from(version)));
            let first = parsed.local.unwrap();
            let second = parsed.remote.unwrap();
            assert_eq!(first.pn_identifier, None);
            assert_eq!(second.pn_identifier, Some(b"15550101".to_vec()));
            assert_eq!(
                first.lid_identifier,
                Some(local.lid_jid.as_bytes().to_vec())
            );
            assert_eq!(first.hosted_state, Some(0));
            assert_eq!(second.hosted_state, Some(1));
            assert_eq!(first.public_key.is_some(), version == 0);
            assert_eq!(
                first.hashed_public_key.as_ref().map(Vec::len),
                if version == 1 { Some(64) } else { None }
            );
        }
        assert!(!format!("{local:?}").contains("222222"));
        assert!(!format!("{local:?}").contains("15550100"));
    }

    #[test]
    fn missing_unknown_or_invalid_inputs_refuse_and_absence_differs_from_empty() {
        let keys = [[1u8; 32]];
        let mut side = V4Side {
            keys: &keys,
            lid_jid: "222222222222222@lid",
            pn_identifier: Some(""),
            username_identifier: None,
            hosted_state: None,
        };
        let bytes = encode_v4(&side, &side, 0).unwrap();
        let parsed = Combined::decode(bytes.as_slice()).unwrap().local.unwrap();
        assert_eq!(parsed.pn_identifier, Some(vec![]));
        assert_eq!(parsed.username_identifier, None);
        assert_eq!(parsed.hosted_state, None);
        side.username_identifier = Some("");
        let parsed = Combined::decode(encode_v4(&side, &side, 0).unwrap().as_slice()).unwrap();
        assert_eq!(parsed.local.unwrap().pn_identifier, None);
        assert_eq!(parsed.remote.unwrap().pn_identifier, Some(vec![]));
        assert!(encode_v4(&side, &side, 2).is_err());
        side.lid_jid = "";
        assert!(encode_v4(&side, &side, 0).is_err());
        side.lid_jid = "secret\0sentinel";
        assert!(
            !encode_v4(&side, &side, 0)
                .unwrap_err()
                .to_string()
                .contains("sentinel")
        );
        side.lid_jid = "222222222222222@lid";
        side.pn_identifier = Some("secret\u{80}sentinel");
        assert!(encode_v4(&side, &side, 0).is_err());
        side.pn_identifier = None;
        side.keys = &[];
        assert!(encode_v4(&side, &side, 0).is_err());
    }
    fn test_individual(local: bool) -> Individual {
        Individual {
            public_key: Some(if local {
                b"local-key".to_vec()
            } else {
                b"remote-key".to_vec()
            }),
            hashed_public_key: Some(if local {
                b"local-hash".to_vec()
            } else {
                b"remote-hash".to_vec()
            }),
            lid_identifier: Some(if local {
                b"synthetic-local".to_vec()
            } else {
                b"synthetic-remote".to_vec()
            }),
            pn_identifier: Some(if local {
                b"local-pn".to_vec()
            } else {
                b"remote-pn".to_vec()
            }),
            username_identifier: Some(if local {
                b"Local".to_vec()
            } else {
                b"Remote".to_vec()
            }),
            hosted_state: Some(if local { 0 } else { 1 }),
        }
    }

    #[test]
    fn comparison_core_and_remapper_match_all_113_native_non_abort_cases() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/fingerprint-native-comparison-cases.json"
        ))
        .unwrap();
        let base = [test_individual(true), test_individual(false)];
        let mut checked = std::collections::HashSet::new();
        let mut check =
            |name: String, known: &[Individual; 2], scanned: &[Individual; 2], version: u32| {
                let expected = fixture["cases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|case| case["case"] == name)
                    .unwrap();
                let code =
                    compare_sides([&known[0], &known[1]], [&scanned[0], &scanned[1]], version);
                assert_eq!(
                    i64::from(code),
                    expected["code"].as_i64().unwrap(),
                    "{name}"
                );
                assert_eq!(
                    i64::from(remap(code)),
                    expected["remapped_code"].as_i64().unwrap(),
                    "{name}"
                );
                assert!(checked.insert(name));
            };
        for version in 0..=1 {
            check(format!("baseline-v{version}"), &base, &base, version);
            for side in 0..2 {
                let mut scanned = base.clone();
                if version == 0 {
                    scanned[side].public_key = Some(b"different".to_vec());
                } else {
                    scanned[side].hashed_public_key = Some(b"different".to_vec());
                }
                check(
                    format!("key-side{side}-v{version}"),
                    &base,
                    &scanned,
                    version,
                );
                let mut scanned = base.clone();
                if version == 0 {
                    scanned[side].hashed_public_key = Some(b"ignored-other-version".to_vec());
                } else {
                    scanned[side].public_key = Some(b"ignored-other-version".to_vec());
                }
                check(
                    format!("unselected-key-side{side}-v{version}"),
                    &base,
                    &scanned,
                    version,
                );
            }
        }
        let mut scanned = base.clone();
        scanned[1].lid_identifier = Some(b"different".to_vec());
        check("remote-lid".into(), &base, &scanned, 0);
        let mut scanned = base.clone();
        scanned[0].lid_identifier = Some(b"different".to_vec());
        check(
            "core-does-not-repeat-local-orientation-check".into(),
            &base,
            &scanned,
            0,
        );
        for side in 0..2 {
            for target in ["known", "scanned"] {
                let mut known = base.clone();
                let mut scanned = base.clone();
                let changed = if target == "known" {
                    &mut known[side]
                } else {
                    &mut scanned[side]
                };
                changed.pn_identifier = None;
                changed.username_identifier = None;
                check(
                    format!("no-identifiers-{target}-side{side}"),
                    &known,
                    &scanned,
                    0,
                );
            }
            let mut scanned = base.clone();
            scanned[side].pn_identifier = Some(b"different".to_vec());
            check(format!("pn-mismatch-side{side}"), &base, &scanned, 0);
            for missing in ["known", "scanned"] {
                let mut known = base.clone();
                let mut scanned = base.clone();
                (if missing == "known" {
                    &mut known[side]
                } else {
                    &mut scanned[side]
                })
                .pn_identifier = None;
                check(
                    format!("pn-omission-{missing}-side{side}"),
                    &known,
                    &scanned,
                    0,
                );
            }
            for mode in ["different", "known-absent", "scanned-absent"] {
                let mut known = base.clone();
                let mut scanned = base.clone();
                match mode {
                    "different" => scanned[side].username_identifier = Some(b"different".to_vec()),
                    "known-absent" => known[side].username_identifier = None,
                    _ => scanned[side].username_identifier = None,
                }
                check(format!("username-{mode}-side{side}"), &known, &scanned, 0);
            }
            let mut known = base.clone();
            let mut scanned = base.clone();
            known[side].username_identifier = None;
            scanned[side].username_identifier = None;
            check(
                format!("both-usernames-absent-side{side}"),
                &known,
                &scanned,
                0,
            );
            let mut scanned = base.clone();
            scanned[side].username_identifier =
                base[side].username_identifier.as_ref().map(|bytes| {
                    bytes
                        .iter()
                        .map(|byte| {
                            if byte.is_ascii_uppercase() {
                                byte.to_ascii_lowercase()
                            } else {
                                byte.to_ascii_uppercase()
                            }
                        })
                        .collect()
                });
            check(
                format!("modeled-case-insensitive-side{side}"),
                &base,
                &scanned,
                0,
            );
        }
        for local in [-1, 0, 1] {
            for remote in [-1, 0, 1] {
                for first in [-1, 0, 1] {
                    for second in [-1, 0, 1] {
                        let mut known = base.clone();
                        let mut scanned = base.clone();
                        known[0].hosted_state = (local != -1).then_some(local);
                        known[1].hosted_state = (remote != -1).then_some(remote);
                        scanned[0].hosted_state = (first != -1).then_some(first);
                        scanned[1].hosted_state = (second != -1).then_some(second);
                        check(
                            format!("hosted-{local}-{remote}-{first}-{second}"),
                            &known,
                            &scanned,
                            0,
                        );
                    }
                }
            }
        }
        assert_eq!(checked.len(), 113);
    }

    #[test]
    fn binary_fixtures_compare_in_both_orientations_and_only_local_first_remaps() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/fingerprint/native-qr-3771.json"
        ))
        .unwrap();
        let get_keys = |side: &str| -> Vec<[u8; 32]> {
            fixture[side]["raw_identity_keys_hex"]
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
        };
        let local_keys = get_keys("local");
        let remote_keys = get_keys("remote");
        let local = V4Side {
            keys: &local_keys,
            lid_jid: "222222222222222@lid",
            pn_identifier: Some("15550100"),
            username_identifier: Some("synthetic-local"),
            hosted_state: Some(HostedState::E2ee),
        };
        let remote = V4Side {
            keys: &remote_keys,
            lid_jid: "333333333333333@lid",
            pn_identifier: Some("15550101"),
            username_identifier: Some("synthetic-remote"),
            hosted_state: Some(HostedState::Hosted),
        };
        for version in 0..=1 {
            let original =
                hex::decode(fixture[format!("version_{version}_hex")].as_str().unwrap()).unwrap();
            let direct = compare_v4(&local, &remote, &original).unwrap();
            assert_eq!(direct.status, 0);
            assert_eq!(direct.orientation, Some(ScanOrientation::LocalFirst));
            let mut ambiguous = Combined::decode(original.as_slice()).unwrap();
            ambiguous.remote.as_mut().unwrap().lid_identifier =
                Some(local.lid_jid.as_bytes().to_vec());
            let priority = compare_v4(&local, &remote, &ambiguous.encode_to_vec()).unwrap();
            assert_eq!(priority.orientation, Some(ScanOrientation::LocalFirst));
            assert_eq!(priority.status, -11);
            let reversed = encode_v4(&remote, &local, version).unwrap();
            let matched = compare_v4(&local, &remote, &reversed).unwrap();
            assert_eq!(matched.status, 0);
            assert_eq!(matched.orientation, Some(ScanOrientation::RemoteFirst));
            let mut same = Combined::decode(original.as_slice()).unwrap();
            same.remote.as_mut().unwrap().lid_identifier = Some(b"other@lid".to_vec());
            assert_eq!(
                compare_v4(&local, &remote, &same.encode_to_vec())
                    .unwrap()
                    .status,
                -11
            );
            let mut swapped = Combined::decode(reversed.as_slice()).unwrap();
            swapped.local.as_mut().unwrap().lid_identifier = Some(b"other@lid".to_vec());
            assert_eq!(
                compare_v4(&local, &remote, &swapped.encode_to_vec())
                    .unwrap()
                    .status,
                -12
            );
            same.local.as_mut().unwrap().lid_identifier = Some(b"not-local@lid".to_vec());
            assert_eq!(
                compare_v4(&local, &remote, &same.encode_to_vec())
                    .unwrap()
                    .status,
                -13
            );
            let json = serde_json::to_string(&direct).unwrap();
            assert!(!json.contains("222222"));
            assert!(!json.contains("synthetic-local"));
            assert!(!json.contains(&hex::encode(local_keys[0])));
        }
    }

    #[test]
    fn payload_failures_follow_wrapper_conversion_order_and_never_panic() {
        let keys = [[1; 32]];
        let known = V4Side {
            keys: &keys,
            lid_jid: "222222222222222@lid",
            pn_identifier: Some("15550100"),
            username_identifier: None,
            hosted_state: None,
        };
        assert!(compare_v4(&known, &known, &[]).is_err());
        assert!(compare_v4(&known, &known, &vec![0; MAX_COMPARISON_PAYLOAD_BYTES + 1]).is_err());
        assert_eq!(compare_v4(&known, &known, &[0xff]).unwrap().status, -8);
        let mut input = Combined {
            version: None,
            local: None,
            remote: None,
        };
        // Nonempty unknown field, with no version, exercises schema presence.
        assert_eq!(compare_v4(&known, &known, &[0x20, 0]).unwrap().status, -5);
        input.version = Some(2);
        assert_eq!(
            compare_v4(&known, &known, &input.encode_to_vec())
                .unwrap()
                .status,
            -10
        );
        input.version = Some(0);
        assert_eq!(
            compare_v4(&known, &known, &input.encode_to_vec())
                .unwrap()
                .status,
            -2
        );
        let valid = individual(&known, 0, false).unwrap();
        input.local = Some(valid.clone());
        input.remote = Some(valid.clone());
        input.local.as_mut().unwrap().lid_identifier = Some(b"invalid\0".to_vec());
        input.remote.as_mut().unwrap().lid_identifier = None;
        assert_eq!(
            compare_v4(&known, &known, &input.encode_to_vec())
                .unwrap()
                .status,
            -43
        );
        for (field, expected) in [
            ("lid", -1),
            ("pn", -44),
            ("username", -23),
            ("hosted", -38),
            ("key", -4),
        ] {
            input.local = Some(valid.clone());
            input.remote = Some(valid.clone());
            let side = input.local.as_mut().unwrap();
            match field {
                "lid" => side.lid_identifier = Some(vec![0]),
                "pn" => side.pn_identifier = Some(vec![128]),
                "username" => side.username_identifier = Some(vec![0]),
                "hosted" => side.hosted_state = Some(2),
                _ => side.public_key = None,
            }
            assert_eq!(
                compare_v4(&known, &known, &input.encode_to_vec())
                    .unwrap()
                    .status,
                expected,
                "{field}"
            );
        }
        input.local = Some(valid.clone());
        input.remote = Some(valid);
        input.version = Some(1);
        assert_eq!(
            compare_v4(&known, &known, &input.encode_to_vec())
                .unwrap()
                .status,
            -4
        );
        // A native invariant abort for direct-core state 2 becomes a normal
        // converter refusal before our core; no abort/panic path is exposed.
    }
    #[test]
    fn username_bytes_and_lengths_match_the_268_executed_native_string_cases() {
        let observations: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/fingerprint-native-string-check.json"
        ))
        .unwrap();
        let base = [test_individual(true), test_individual(false)];
        let bytes: Vec<u8> = (1..=127).collect();
        let flipped: Vec<u8> = bytes
            .iter()
            .map(|byte| {
                if byte.is_ascii_uppercase() {
                    byte.to_ascii_lowercase()
                } else {
                    byte.to_ascii_uppercase()
                }
            })
            .collect();
        let mut cases = 0;
        for side in 0..2 {
            let mismatch = if side == 0 { -24 } else { -29 };
            let mut variants = vec![(bytes.clone(), flipped.clone(), 0)];
            for (position, byte) in bytes.iter().enumerate() {
                let mut changed = bytes.clone();
                changed[position] = if *byte != 1 { 1 } else { 2 };
                variants.push((bytes.clone(), changed, mismatch));
            }
            variants.extend([
                (b"prefix".to_vec(), b"prefixx".to_vec(), mismatch),
                (b"prefixx".to_vec(), b"prefix".to_vec(), mismatch),
                (vec![], vec![], 0),
                (vec![], b"x".to_vec(), mismatch),
                (b"x".to_vec(), vec![], mismatch),
                (vec![b'A'; 256], vec![b'a'; 256], 0),
            ]);
            for (a, b, expected) in variants {
                let mut known = base.clone();
                let mut scanned = base.clone();
                known[side].username_identifier = Some(a);
                scanned[side].username_identifier = Some(b);
                assert_eq!(
                    compare_sides([&known[0], &known[1]], [&scanned[0], &scanned[1]], 0),
                    expected
                );
                cases += 1;
            }
        }
        assert_eq!(
            cases,
            observations["native_ascii_checks"]["cases"]
                .as_u64()
                .unwrap()
        );
    }
}
