//! APK 37.71 native V3 comparison. See safety-number.md V3 evidence/corpora.
use super::*;

/// Explicit selected V3 inputs; supports PN-only and LID-only identities.
/// Keys/identifiers are private; Debug reports only presence and key count.
pub struct V3Side<'a> {
    /// Selected raw public-key inventory; not inferred from a primary session.
    pub keys: &'a [[u8; 32]],
    /// Optional full primary LID JID; validated against the traced native subset.
    pub lid_jid: Option<&'a str>,
    /// Optional already-normalized stable PN identifier.
    pub pn_identifier: Option<&'a str>,
    /// Optional caller-selected username; V3 compares exact ASCII bytes.
    pub username_identifier: Option<&'a str>,
    /// None means absent native state, not E2EE.
    pub hosted_state: Option<HostedState>,
}
impl std::fmt::Debug for V3Side<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V3Side")
            .field("key_count", &self.keys.len())
            .field("has_lid", &self.lid_jid.is_some())
            .field("has_pn", &self.pn_identifier.is_some())
            .field("has_username", &self.username_identifier.is_some())
            .field("hosted_state", &self.hosted_state)
            .finish_non_exhaustive()
    }
}

/// V3 metadata only; matching does not commit human verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct V3Comparison {
    /// Shared decoded-version/orientation/result metadata.
    #[serde(flatten)]
    pub result: V4Comparison,
    /// Explicitly selected comparison algorithm, not an inferred APK flag.
    pub scheme: &'static str,
    /// APK UI options: username checks (1), plus hosted checks when selected (3).
    pub options: u8,
    /// Native parser decisions executed with modeled MCF primitives, other edges unverified.
    pub lid_parser_authority: &'static str,
}

fn primary_lid(bytes: &[u8]) -> Option<Vec<u8>> {
    // 37.71 MCF split 0x531978 preserves leading/trailing/adjacent empty
    // pieces; primary parser 71dfa4 -> 71f30c(option 0) -> kind-5 route
    // 71f860 -> 7204bc/71fdc0/71fd24/71fe0c. Native 38-case corpus.
    let mut address = bytes.split(|byte| *byte == b'@');
    let user = address.next()?;
    if address.next()? != b"lid" || address.next().is_some() {
        return None;
    }
    let mut parts = user.split(|byte| *byte == b':');
    let number = parts.next()?;
    if !(1..=20).contains(&number.len())
        || (16..=19).contains(&number.len())
        || !matches!(number.first(), Some(b'1'..=b'9'))
        || !number.iter().all(u8::is_ascii_digit)
    {
        return None;
    }
    if let Some(device) = parts.next()
        && (!(1..=2).contains(&device.len())
            || !device.iter().all(|byte| *byte == b'0')
            || parts.next().is_some())
    {
        // Nonzero device stays category 5 and fails the primary guard.
        // Zero (including 00) is flattened by 71ebd8 to category 0/kind 5.
        return None;
    }
    let mut value = number.to_vec();
    value.extend_from_slice(b"@lid");
    Some(value)
}

fn known(side: &V3Side<'_>, version: u8) -> Result<Individual> {
    let pn = side.pn_identifier.map(ascii).transpose()?;
    let lid = side
        .lid_jid
        .map(|value| {
            ascii(value)?;
            primary_lid(value.as_bytes()).ok_or_else(|| {
                Error::invalid(
                    "fingerprint.lid_jid",
                    "requires an APK-compatible primary LID JID",
                )
            })
        })
        .transpose()?;
    if pn.is_none() && lid.is_none() {
        return Err(Error::invalid(
            "fingerprint.v3_identity",
            "requires PN or LID on each known side",
        ));
    }
    let username = side.username_identifier.map(ascii).transpose()?;
    let (public_key, hashed_public_key) = key_material(side.keys, version)?;
    Ok(Individual {
        public_key,
        hashed_public_key,
        pn_identifier: pn,
        lid_identifier: lid,
        username_identifier: username,
        hosted_state: side.hosted_state.map(|state| match state {
            HostedState::E2ee => 0,
            HostedState::Hosted => 1,
        }),
    })
}

/// Encode a native V3 combined payload, preserving PN on both sides.
/// Explicit selected inputs only; no active feature/inventory or verification claim.
/// Present LIDs use the supported primary subset; their supplied ASCII text is
/// retained, whereas comparison normalizes zero-device suffixes. Version is 0/1.
pub fn encode_v3(local: &V3Side<'_>, remote: &V3Side<'_>, version: u8) -> Result<Vec<u8>> {
    if version > 1 {
        return Err(Error::invalid(
            "fingerprint.qr_version",
            "native QR version must be 0 or 1",
        ));
    }
    fn serialized(side: &V3Side<'_>, version: u8) -> Result<Individual> {
        let mut value = known(side, version)?;
        // 37.71 serializer 713cf8..713e00 retains supplied LID full string
        // and PN regardless of username. Do not apply V4 local PN suppression
        // or the scanned converter's zero-device normalization while exporting.
        value.lid_identifier = side.lid_jid.map(ascii).transpose()?;
        Ok(value)
    }
    Ok(Combined {
        // 713660 writes both presence=1 and version, including zero;
        // 713664/713674 pass the same version to local and remote serializers.
        version: Some(u32::from(version)),
        local: Some(serialized(local, version)?),
        remote: Some(serialized(remote, version)?),
    }
    .encode_to_vec())
}

fn convert(side: &mut Individual, version: u32) -> std::result::Result<(), i32> {
    // Native V3 converter 713e00: PN ASCII, LID ASCII/parse, username ASCII,
    // address presence, hosted enum, then version-selected key. Order matters.
    if side
        .pn_identifier
        .as_ref()
        .is_some_and(|value| !ascii_bytes(value))
    {
        return Err(-7);
    }
    if let Some(lid) = &side.lid_identifier {
        if !ascii_bytes(lid) {
            return Err(-1);
        }
        side.lid_identifier = Some(primary_lid(lid).ok_or(-6)?);
    }
    if side
        .username_identifier
        .as_ref()
        .is_some_and(|value| !ascii_bytes(value))
    {
        return Err(-23);
    }
    if side.pn_identifier.is_none() && side.lid_identifier.is_none() {
        return Err(-3);
    }
    if side
        .hosted_state
        .is_some_and(|value| !matches!(value, 0 | 1))
    {
        return Err(-38);
    }
    if (if version == 0 {
        &side.public_key
    } else {
        &side.hashed_public_key
    })
    .is_none()
    {
        return Err(-4);
    }
    Ok(())
}

fn identity_matches(a: &Individual, b: &Individual) -> bool {
    // Native 71401c accepts equal non-null PN OR retained LID; registered
    // JID equality callback 71eda4 compares constructed full ASCII strings.
    a.pn_identifier
        .as_ref()
        .is_some_and(|pn| b.pn_identifier.as_ref() == Some(pn))
        || a.lid_identifier
            .as_ref()
            .is_some_and(|lid| b.lid_identifier.as_ref() == Some(lid))
}

fn core(known: [&Individual; 2], scanned: [&Individual; 2], version: u32, options: u8) -> i32 {
    // 7140fc..714358 and 714408/714450; actual 264-case native corpus.
    if !identity_matches(known[1], scanned[1]) {
        return -12;
    }
    for side in 0..2 {
        fn selected(value: &Individual, version: u32) -> Option<&[u8]> {
            if version == 0 {
                value.public_key.as_deref()
            } else {
                value.hashed_public_key.as_deref()
            }
        }
        let a = selected(known[side], version);
        let b = selected(scanned[side], version);
        if a.is_none() || a != b {
            return if side == 0 { -14 } else { -15 };
        }
    }
    for side in 0..2 {
        match (&known[side].pn_identifier, &scanned[side].pn_identifier) {
            (Some(a), Some(b)) if a != b => return if side == 0 { -17 } else { -19 },
            (Some(_), None) if side == 1 => return -21,
            (None, Some(_)) => return if side == 0 { -17 } else { -22 },
            _ => {}
        }
    }
    if options & 1 != 0 {
        for side in 0..2 {
            match (
                &known[side].username_identifier,
                &scanned[side].username_identifier,
            ) {
                (Some(a), Some(b)) if a != b => return if side == 0 { -24 } else { -29 },
                (Some(_), None) => return if side == 0 { -26 } else { -31 },
                (None, Some(_)) => return if side == 0 { -28 } else { -33 },
                _ => {}
            }
        }
    }
    if options & 2 != 0 && !scanned.iter().all(|value| value.hosted_state.is_none()) {
        for side in 0..2 {
            if known[side].hosted_state != scanned[side].hosted_state {
                return if side == 0 { -34 } else { -36 };
            }
        }
    }
    0
}

fn remap_v3(code: i32) -> i32 {
    // 37.71 remapper 7140b8, mask 100dead5ad/table 302370.
    match code {
        -21 => -20,
        -17 => -16,
        -19 => -18,
        -36 => -37,
        -34 => -35,
        -31 => -32,
        -29 => -30,
        -26 => -27,
        -24 => -25,
        -12 => -11,
        0 | -33 | -28 | -22 | -15 | -14 => code,
        _ => unreachable!("V3 core emitted an untraced status"),
    }
}

/// Compare native V3 protobuf using explicit known identities and key inventories.
/// Username checks are always enabled, as at the APK UI caller; `check_hosted`
/// selects its options 1/3. No feature inference, account access or verification
/// commit. Empty/oversized buffers refuse as local policy instead of native abort.
/// Native core and LID branch corpora use modeled MCF primitives; other parser,
/// allocation/JNI edges and current authenticated input selection remain unverified.
pub fn compare_v3(
    local: &V3Side<'_>,
    remote: &V3Side<'_>,
    payload: &[u8],
    check_hosted: bool,
) -> Result<V3Comparison> {
    let options = if check_hosted { 3 } else { 1 };
    let result = |status, version, orientation| V3Comparison {
        result: comparison(status, version, orientation),
        scheme: "native_v3",
        options,
        lid_parser_authority: "apk_native_branches_with_modeled_primitives_other_edges_unverified",
    };
    if payload.is_empty() || payload.len() > MAX_COMPARISON_PAYLOAD_BYTES {
        return Err(Error::invalid(
            "fingerprint.qr_payload",
            "requires nonempty binary protobuf at most 64 KiB",
        ));
    }
    let Ok(parsed) = decode::combined(payload) else {
        return Ok(result(-8, None, None));
    };
    let Some(version) = parsed.version else {
        return Ok(result(-5, None, None));
    };
    if version > 1 {
        return Ok(result(-10, Some(version), None));
    }
    let (Some(mut first), Some(mut second)) = (parsed.local, parsed.remote) else {
        return Ok(result(-2, Some(version), None));
    };
    for side in [&mut first, &mut second] {
        if let Err(code) = convert(side, version) {
            return Ok(result(code, Some(version), None));
        }
    }
    let known_local = known(local, version as u8)?;
    let known_remote = known(remote, version as u8)?;
    let (ordered, orientation) = if identity_matches(&known_local, &first) {
        ([&first, &second], ScanOrientation::LocalFirst)
    } else if identity_matches(&known_local, &second) {
        ([&second, &first], ScanOrientation::RemoteFirst)
    } else {
        return Ok(result(-13, Some(version), None));
    };
    let status = core([&known_local, &known_remote], ordered, version, options);
    Ok(result(
        if orientation == ScanOrientation::LocalFirst {
            remap_v3(status)
        } else {
            status
        },
        Some(version),
        Some(orientation),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_matches_independent_v3_wire_fixtures_and_roundtrips_both_orientations() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/fingerprint/native-v3-qr-3771.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let keys = |value: &serde_json::Value| {
                value["raw_identity_keys_hex"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|key| {
                        hex::decode(key.as_str().unwrap())
                            .unwrap()
                            .try_into()
                            .unwrap()
                    })
                    .collect::<Vec<[u8; 32]>>()
            };
            let local_keys = keys(&case["local"]);
            let remote_keys = keys(&case["remote"]);
            fn side<'a>(value: &'a serde_json::Value, keys: &'a [[u8; 32]]) -> V3Side<'a> {
                V3Side {
                    keys,
                    lid_jid: value["lid_jid"].as_str(),
                    pn_identifier: value["pn_identifier"].as_str(),
                    username_identifier: value["username_identifier"].as_str(),
                    hosted_state: match value["hosted_state"].as_i64() {
                        Some(0) => Some(HostedState::E2ee),
                        Some(1) => Some(HostedState::Hosted),
                        _ => None,
                    },
                }
            }
            let local = side(&case["local"], &local_keys);
            let remote = side(&case["remote"], &remote_keys);
            for version in [0, 1] {
                let bytes = encode_v3(&local, &remote, version).unwrap();
                assert_eq!(
                    hex::encode(&bytes),
                    case[format!("version_{version}_hex")],
                    "{}",
                    case["case"]
                );
                for (a, b, orientation) in [
                    (&local, &remote, ScanOrientation::LocalFirst),
                    (&remote, &local, ScanOrientation::RemoteFirst),
                ] {
                    let result = compare_v3(a, b, &bytes, true).unwrap().result;
                    assert_eq!(result.status, 0, "{}", case["case"]);
                    assert_eq!(result.orientation, Some(orientation));
                }
            }
        }
    }

    #[test]
    fn export_refuses_unknown_version_missing_identity_invalid_ascii_and_empty_keys() {
        let keys = [[1u8; 32]];
        let mut side = V3Side {
            keys: &keys,
            lid_jid: None,
            pn_identifier: Some(""),
            username_identifier: Some(""),
            hosted_state: None,
        };
        let parsed = decode::combined(&encode_v3(&side, &side, 0).unwrap()).unwrap();
        assert_eq!(parsed.version, Some(0));
        let individual = parsed.local.unwrap();
        assert_eq!(individual.pn_identifier, Some(vec![]));
        assert_eq!(individual.username_identifier, Some(vec![]));
        assert_eq!(individual.lid_identifier, None);
        assert_eq!(individual.hosted_state, None);
        assert!(encode_v3(&side, &side, 2).is_err());
        side.pn_identifier = None;
        assert!(encode_v3(&side, &side, 0).is_err());
        side.pn_identifier = Some("private\0sentinel");
        assert!(
            !encode_v3(&side, &side, 0)
                .unwrap_err()
                .to_string()
                .contains("sentinel")
        );
        side.pn_identifier = Some("synthetic-pn");
        side.lid_jid = Some("123:1@lid");
        assert!(encode_v3(&side, &side, 0).is_err());
        side.lid_jid = None;
        side.keys = &[];
        assert!(encode_v3(&side, &side, 0).is_err());
    }

    #[test]
    fn port_matches_every_non_abort_native_core_observation() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../docs/protocol/fingerprint-native-v3-comparison-cases.json"
        ))
        .unwrap();
        let side = |value: &serde_json::Value| {
            let bytes = |field: &str| value[field].as_str().map(|text| text.as_bytes().to_vec());
            Individual {
                public_key: bytes("public"),
                hashed_public_key: bytes("hash"),
                pn_identifier: bytes("pn"),
                lid_identifier: bytes("lid"),
                username_identifier: bytes("username"),
                hosted_state: match value["hosted"].as_i64().unwrap() {
                    -1 => None,
                    n => Some(n as i32),
                },
            }
        };
        let mut checked = 0;
        for case in fixture["cases"].as_array().unwrap() {
            if case["native_abort"] == true {
                continue;
            }
            let known = [side(&case["known"][0]), side(&case["known"][1])];
            let scanned = [side(&case["scanned"][0]), side(&case["scanned"][1])];
            let code = core(
                [&known[0], &known[1]],
                [&scanned[0], &scanned[1]],
                case["version"].as_u64().unwrap() as u32,
                case["options"].as_u64().unwrap() as u8,
            );
            assert_eq!(
                i64::from(code),
                case["code"].as_i64().unwrap(),
                "{}",
                case["case"]
            );
            assert_eq!(
                i64::from(remap_v3(code)),
                case["remapped_code"].as_i64().unwrap(),
                "{}",
                case["case"]
            );
            checked += 1;
        }
        assert_eq!(checked, 263);
    }

    #[test]
    fn lid_conversion_matches_native_branch_corpus() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../docs/protocol/fingerprint-native-v3-lid-cases.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let input = case["synthetic_input"].as_str().unwrap();
            let actual =
                primary_lid(input.as_bytes()).map(|bytes| String::from_utf8(bytes).unwrap());
            assert_eq!(
                actual.as_deref(),
                case["normalized_jid"].as_str(),
                "{input}"
            );
        }
        // Merged split 531978 appends even zero-length pieces at every
        // delimiter and after the last; malformed components cannot vanish.
        for input in [
            "@lid",
            "123@@lid",
            "123@lid@",
            "123:@lid",
            "123::0@lid",
            "123@",
            "",
        ] {
            assert_eq!(primary_lid(input.as_bytes()), None, "{input}");
        }
    }

    fn sample<'a>(keys: &'a [[u8; 32]], lid: Option<&'a str>, pn: Option<&'a str>) -> V3Side<'a> {
        V3Side {
            keys,
            lid_jid: lid,
            pn_identifier: pn,
            username_identifier: None,
            hosted_state: Some(HostedState::E2ee),
        }
    }

    #[test]
    fn binary_wrapper_orients_by_pn_or_lid_and_keeps_v3_specific_rules() {
        let local_keys = [[3; 32]];
        let remote_keys = [[6; 32]];
        let mut local = sample(&local_keys, Some("123@lid"), Some("local-pn"));
        let mut remote = sample(&remote_keys, Some("456@lid"), Some("remote-pn"));
        local.username_identifier = Some("Local");
        remote.username_identifier = Some("Remote");
        for version in 0..=1 {
            let mut sides = [
                known(&local, version).unwrap(),
                known(&remote, version).unwrap(),
            ];
            let payload = |sides: &[Individual; 2]| {
                Combined {
                    version: Some(u32::from(version)),
                    local: Some(sides[0].clone()),
                    remote: Some(sides[1].clone()),
                }
                .encode_to_vec()
            };
            assert_eq!(
                compare_v3(&local, &remote, &payload(&sides), true)
                    .unwrap()
                    .result
                    .status,
                0
            );
            sides[0].lid_identifier = Some(b"123:00@lid".to_vec());
            sides[0].pn_identifier = None;
            let result = compare_v3(&local, &remote, &payload(&sides), true).unwrap();
            assert_eq!(result.result.status, 0);
            assert_eq!(result.result.orientation, Some(ScanOrientation::LocalFirst));
            sides[0].username_identifier = Some(b"local".to_vec());
            assert_eq!(
                compare_v3(&local, &remote, &payload(&sides), true)
                    .unwrap()
                    .result
                    .status,
                -25
            );
            sides.swap(0, 1);
            let result = compare_v3(&local, &remote, &payload(&sides), true).unwrap();
            assert_eq!(result.result.status, -24);
            assert_eq!(
                result.result.orientation,
                Some(ScanOrientation::RemoteFirst)
            );
        }
        // PN-only known sides can match scanned sides by PN; LID is not
        // mandatory in V3 and username absence is not V4's -39/-41.
        let local = sample(&local_keys, None, Some("local-pn"));
        let remote = sample(&remote_keys, None, Some("remote-pn"));
        let bytes = Combined {
            version: Some(0),
            local: Some(known(&local, 0).unwrap()),
            remote: Some(known(&remote, 0).unwrap()),
        }
        .encode_to_vec();
        assert_eq!(
            compare_v3(&local, &remote, &bytes, false)
                .unwrap()
                .result
                .status,
            0
        );
    }

    #[test]
    fn converter_error_order_and_resource_policy_are_explicit() {
        let base = Individual {
            public_key: Some(vec![]),
            pn_identifier: Some(b"pn".to_vec()),
            ..Individual::default()
        };
        let mut bad = base.clone();
        bad.pn_identifier = Some(vec![0]);
        bad.lid_identifier = Some(vec![0]);
        assert_eq!(convert(&mut bad, 0), Err(-7));
        bad.pn_identifier = None;
        assert_eq!(convert(&mut bad, 0), Err(-1));
        bad.lid_identifier = Some(b"123:1@lid".to_vec());
        assert_eq!(convert(&mut bad, 0), Err(-6));
        bad.lid_identifier = None;
        bad.username_identifier = Some(vec![0]);
        assert_eq!(convert(&mut bad, 0), Err(-23));
        bad.username_identifier = None;
        bad.hosted_state = Some(2);
        assert_eq!(convert(&mut bad, 0), Err(-3));
        bad.pn_identifier = Some(vec![]);
        assert_eq!(convert(&mut bad, 0), Err(-38));
        bad.hosted_state = None;
        bad.public_key = None;
        assert_eq!(convert(&mut bad, 0), Err(-4));
        let keys = [[3; 32]];
        let known = sample(&keys, Some("123@lid"), None);
        assert!(compare_v3(&known, &known, &[], false).is_err());
        assert!(
            compare_v3(
                &known,
                &known,
                &vec![0; MAX_COMPARISON_PAYLOAD_BYTES + 1],
                false
            )
            .is_err()
        );
        assert_eq!(
            compare_v3(&known, &known, &[0xff], false)
                .unwrap()
                .result
                .status,
            -8
        );
        assert_eq!(
            compare_v3(&known, &known, &[0x20, 0], false)
                .unwrap()
                .result
                .status,
            -5
        );
        assert_eq!(
            compare_v3(
                &known,
                &known,
                &Combined {
                    version: Some(2),
                    ..Combined::default()
                }
                .encode_to_vec(),
                false
            )
            .unwrap()
            .result
            .status,
            -10
        );
        assert_eq!(
            compare_v3(
                &known,
                &known,
                &Combined {
                    version: Some(0),
                    ..Combined::default()
                }
                .encode_to_vec(),
                false
            )
            .unwrap()
            .result
            .status,
            -2
        );
        let output =
            serde_json::to_string(&compare_v3(&known, &known, &[0xff], true).unwrap()).unwrap();
        assert!(!output.contains("123@lid"));
        assert!(!format!("{known:?}").contains("123@lid"));
    }
}
