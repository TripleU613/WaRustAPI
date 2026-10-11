//! APK 37.71 attestation verification and state-independent primary admission.
use crate::binary::{Node, NodeContent};
use serde::Serialize;
use wacore_libsignal::protocol::IdentityKey;

/// BP2.accountType's protobuf enum, distinct from stored C0F state values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationAccountType {
    /// C29.A01 / protobuf 0; CPB rejects this as an attestation account type.
    E2ee,
    /// C29.A02 / protobuf 1.
    HybridE2ee,
    /// C29.A03 / protobuf 2.
    NonE2ee,
}
impl AttestationAccountType {
    fn from_value(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::E2ee),
            1 => Some(Self::HybridE2ee),
            2 => Some(Self::NonE2ee),
            _ => None,
        }
    }
}
/// Fixed redacted verification result; not CPB admission or a pin policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationStatus {
    /// No selected-device non_e2ee_attestation child.
    Absent,
    /// Recognized account types agree and the original bytes' signature verifies.
    Verified,
    /// account_type is absent, nontext or outside A61's two permitted strings.
    InvalidAccountType,
    /// Required child data/64-byte signature is absent, invalid or ambiguous.
    MalformedAttestation,
    /// Payload exceeds this inspector's local 64 KiB bound, not an APK limit.
    PayloadTooLarge,
    /// BP2 protobuf cannot be parsed by the supported bounded parser.
    MalformedProtobuf,
    /// Parsed/default E2EE is not a valid attestation account type.
    InvalidPayloadAccountType,
    /// Declared account_type and recognized protobuf type differ.
    AccountTypeMismatch,
    /// Signature fails against the returned identity, prefix and original body.
    InvalidSignature,
}
/// Redacted verification of a selected device's attestation, without its bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttestationInspection {
    /// Fixed verification result, never native acceptance or stored-state update.
    pub status: AttestationStatus,
    /// Recognized stanza type only; arbitrary server strings are never echoed.
    pub declared_account_type: Option<AttestationAccountType>,
    /// Parsed/default protobuf type, null when parsing was not reached or failed.
    pub payload_account_type: Option<AttestationAccountType>,
    /// Null until signature verification, true only with matching recognized types.
    pub signature_valid: Option<bool>,
}
fn singleton<'a>(node: &'a Node, name: &str) -> Option<&'a Node> {
    let mut matches = node
        .child_nodes()
        .iter()
        .filter(|node| node.description == name);
    let value = matches.next()?;
    matches.next().is_none().then_some(value)
}
fn bytes(node: &Node) -> Option<&[u8]> {
    match &node.content {
        NodeContent::Bytes(bytes) => Some(bytes),
        _ => None,
    }
}

// CPB admits changed primary keys even after failed verification (E2EE fallback),
// and verified same-key HYBRID/NON candidates regardless of stored state/flags.
pub(super) fn primary_admitted(
    user: &Node,
    same_identity: bool,
    inspection: Option<&AttestationInspection>,
) -> bool {
    let Some(node) = singleton(user, "non_e2ee_attestation") else {
        return false;
    };
    // Bot/CWS require byte bodies, but permit nullable type and empty bytes.
    // Duplicate refusal is our existing local ambiguity guard.
    singleton(node, "value").and_then(bytes).is_some()
        && singleton(node, "signature").and_then(bytes).is_some()
        && (!same_identity || inspection.is_some_and(|v| v.status == AttestationStatus::Verified))
}

pub(super) fn inspect(user: &Node, identity: &IdentityKey) -> AttestationInspection {
    let mut result = AttestationInspection {
        status: AttestationStatus::Absent,
        declared_account_type: None,
        payload_account_type: None,
        signature_valid: None,
    };
    if user.child("non_e2ee_attestation").is_none() {
        return result;
    }
    result.status = AttestationStatus::MalformedAttestation;
    // Duplicate nodes are a local ambiguity refusal, not a claimed APK rule.
    let Some(node) = singleton(user, "non_e2ee_attestation") else {
        return result;
    };
    // A61.<clinit> 0008..001c: only these account_type strings are supported.
    result.declared_account_type = match node.get_attr("account_type").and_then(|v| v.as_text()) {
        Some("hybrid_e2ee") => Some(AttestationAccountType::HybridE2ee),
        Some("non_e2ee") => Some(AttestationAccountType::NonE2ee),
        _ => {
            result.status = AttestationStatus::InvalidAccountType;
            return result;
        }
    };
    let Some(body) = singleton(node, "value").and_then(bytes) else {
        return result;
    };
    let Some(signature) = singleton(node, "signature").and_then(bytes) else {
        return result;
    };
    // CPB.A00 0112..011c requires exactly 64 signature bytes.
    if signature.len() != 64 {
        return result;
    }
    if body.len() > 65536 {
        result.status = AttestationStatus::PayloadTooLarge;
        return result;
    }
    let Ok(account_type) = parse_account_type(body) else {
        result.status = AttestationStatus::MalformedProtobuf;
        return result;
    };
    result.payload_account_type = Some(account_type);
    if account_type == AttestationAccountType::E2ee {
        result.status = AttestationStatus::InvalidPayloadAccountType;
        return result;
    }
    if result.declared_account_type != Some(account_type) {
        result.status = AttestationStatus::AccountTypeMismatch;
        return result;
    }
    // 08F.<clinit> 0150..015a: A0J = [06,07]. CPB.A00 0172..0190
    // signs prefix + ORIGINAL value bytes. Re-encoding BP2 would discard or
    // reorder unknown fields and change the signature input.
    let mut signed = Vec::with_capacity(2 + body.len());
    signed.extend_from_slice(&[0x06, 0x07]);
    signed.extend_from_slice(body);
    let valid = identity.public_key().verify_signature(&signed, signature);
    result.signature_valid = Some(valid);
    result.status = if valid {
        AttestationStatus::Verified
    } else {
        AttestationStatus::InvalidSignature
    };
    result
}

// BP2's closed enum verifier ignores unknown numeric values, retaining the last
// recognized one or default 0. Plain prost enum decoding keeps the last unknown
// instead. Java's raw varint32 also truncates high bits (including tags/lengths)
// and consumes trailing continuation bytes; the independent descriptor oracle
// exercises those differences. This parser only projects BP2.accountType.
fn varint32(bytes: &mut &[u8]) -> Result<u32, ()> {
    let mut value = 0;
    let mut index = 0;
    loop {
        let (&byte, rest) = bytes.split_first().ok_or(())?;
        *bytes = rest;
        if index < 5 {
            value |= u32::from(byte & 127) << (index * 7);
        }
        if byte < 128 {
            return Ok(value);
        }
        index += 1;
    }
}
fn skip(bytes: &mut &[u8], key: u32, depth: usize) -> Result<(), ()> {
    if key >> 3 == 0 {
        return Err(());
    }
    let length = match key & 7 {
        0 => {
            varint32(bytes)?;
            0
        }
        1 => 8,
        2 => {
            let length = varint32(bytes)?;
            if length > i32::MAX as u32 {
                return Err(());
            }
            length as usize
        }
        3 => {
            // The pinned Java descriptor oracle counts its root toward the
            // default recursion limit: 99 unknown groups pass, 100 refuse.
            // The APK protobuf runtime version/boundary is not established.
            if depth >= 99 {
                return Err(());
            }
            loop {
                let nested = varint32(bytes)?;
                if nested & 7 == 4 {
                    if nested >> 3 != key >> 3 {
                        return Err(());
                    }
                    break;
                }
                skip(bytes, nested, depth + 1)?;
            }
            0
        }
        5 => 4,
        _ => return Err(()),
    };
    *bytes = bytes.get(length..).ok_or(())?;
    Ok(())
}
fn parse_account_type(mut body: &[u8]) -> Result<AttestationAccountType, ()> {
    let mut result = AttestationAccountType::E2ee;
    while !body.is_empty() {
        let key = varint32(&mut body)?;
        // BP2.dynamicMethod 0032..0040: field 1, enum; Cu0/C29 verifier 0..2.
        if key == 8 {
            if let Some(value) = AttestationAccountType::from_value(varint32(&mut body)?) {
                result = value;
            }
        } else {
            skip(&mut body, key, 0)?;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wacore_libsignal::protocol::IdentityKeyPair;

    #[test]
    fn account_type_matches_descriptor_java_oracle() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/prekeys/2.26.37.71-attestation-proto-oracle.json"
        ))
        .unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let body = hex::decode(case["hex"].as_str().unwrap()).unwrap();
            let expected = case["account_type"]
                .as_u64()
                .map(|n| AttestationAccountType::from_value(n as u32).unwrap());
            assert_eq!(
                parse_account_type(&body).ok(),
                expected,
                "case {}",
                case["hex"]
            );
        }
    }
    fn user(identity: &IdentityKeyPair, declared: &str, body: &[u8], prefix: &[u8]) -> Node {
        let mut signed = prefix.to_vec();
        signed.extend_from_slice(body);
        let signature = identity
            .private_key()
            .calculate_signature(&signed, &mut rand::rng())
            .unwrap();
        Node::new("user").children(vec![
            Node::new("non_e2ee_attestation")
                .attr("account_type", declared)
                .children(vec![
                    Node::new("value").bytes(body.to_vec()),
                    Node::new("signature").bytes(signature.to_vec()),
                ]),
        ])
    }
    #[test]
    fn verification_binds_declared_enum_identity_prefix_and_original_bytes() {
        let identity = IdentityKeyPair::generate(&mut rand::rng());
        for (declared, body) in [
            ("hybrid_e2ee", vec![8, 1]),
            ("non_e2ee", vec![8, 2]),
            ("hybrid_e2ee", vec![8, 1, 16, 7]),
            ("hybrid_e2ee", vec![8, 1, 8, 3]),
            ("non_e2ee", vec![8, 1, 8, 2]),
            ("hybrid_e2ee", vec![8, 0x81, 0]),
        ] {
            let node = user(&identity, declared, &body, &[6, 7]);
            assert_eq!(
                inspect(&node, identity.identity_key()).status,
                AttestationStatus::Verified
            );
            assert_eq!(
                inspect(
                    &user(&identity, declared, &body, &[6, 2]),
                    identity.identity_key()
                )
                .status,
                AttestationStatus::InvalidSignature
            );
            let other = IdentityKeyPair::generate(&mut rand::rng());
            assert_eq!(
                inspect(&node, other.identity_key()).status,
                AttestationStatus::InvalidSignature
            );
        }
        assert_eq!(
            inspect(
                &user(&identity, "hybrid_e2ee", &[8, 2], &[6, 7]),
                identity.identity_key()
            )
            .status,
            AttestationStatus::AccountTypeMismatch
        );
        for body in [vec![], vec![8, 0], vec![8, 3], vec![8, 255, 1]] {
            assert_eq!(
                inspect(
                    &user(&identity, "hybrid_e2ee", &body, &[6, 7]),
                    identity.identity_key()
                )
                .status,
                AttestationStatus::InvalidPayloadAccountType
            );
        }
        assert_eq!(
            inspect(
                &user(&identity, "hybrid_e2ee", &[8], &[6, 7]),
                identity.identity_key()
            )
            .status,
            AttestationStatus::MalformedProtobuf
        );
        assert_eq!(
            inspect(
                &user(&identity, "private-account-type", &[8, 1], &[6, 7]),
                identity.identity_key()
            )
            .status,
            AttestationStatus::InvalidAccountType
        );
        assert_eq!(
            inspect(&Node::new("user"), identity.identity_key()).status,
            AttestationStatus::Absent
        );
    }
    fn fields(node: &mut Node) -> &mut Vec<Node> {
        let NodeContent::Children(children) = &mut node.content else {
            panic!("fixture user")
        };
        let NodeContent::Children(fields) = &mut children[0].content else {
            panic!("fixture attestation")
        };
        fields
    }
    #[test]
    fn malformed_and_ambiguous_fields_refuse_without_echo_or_reencoding() {
        let identity = IdentityKeyPair::generate(&mut rand::rng());
        for case in [
            "short_signature",
            "missing_signature",
            "duplicate_signature",
            "duplicate_value",
            "text_value",
            "duplicate_attestation",
            "changed_encoding",
            "oversize",
        ] {
            let mut node = user(&identity, "hybrid_e2ee", &[8, 1, 16, 7], &[6, 7]);
            match case {
                "short_signature" => {
                    fields(&mut node)[1] = Node::new("signature").bytes(vec![0; 63])
                }
                "missing_signature" => {
                    fields(&mut node).remove(1);
                }
                "duplicate_signature" => {
                    let field = fields(&mut node)[1].clone();
                    fields(&mut node).push(field);
                }
                "duplicate_value" => {
                    let field = fields(&mut node)[0].clone();
                    fields(&mut node).push(field);
                }
                "text_value" => {
                    fields(&mut node)[0].content =
                        NodeContent::Text("private-payload-marker".into())
                }
                "duplicate_attestation" => {
                    let NodeContent::Children(children) = &mut node.content else {
                        panic!("fixture user")
                    };
                    children.push(children[0].clone());
                }
                "changed_encoding" => fields(&mut node)[0] = Node::new("value").bytes(vec![8, 1]),
                _ => fields(&mut node)[0] = Node::new("value").bytes(vec![0; 65537]),
            }
            let result = inspect(&node, identity.identity_key());
            assert_eq!(
                result.status,
                match case {
                    "changed_encoding" => AttestationStatus::InvalidSignature,
                    "oversize" => AttestationStatus::PayloadTooLarge,
                    _ => AttestationStatus::MalformedAttestation,
                }
            );
            assert!(
                !serde_json::to_string(&result)
                    .unwrap()
                    .contains("private-payload-marker")
            );
        }
    }
}
