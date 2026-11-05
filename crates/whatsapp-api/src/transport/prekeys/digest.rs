//! Offline 43.72 digest inspection: 1ru.A02/291(1), Fdf.run(selector1).
//! Parsing/matching is not request correlation or authority to resynchronize.
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    session::AccountRecord,
    store::{ProtocolStore, ReadTx, Table},
};
use serde::Serialize;
use sha2::Digest;
use wacore_libsignal::protocol::{GenericSignedPreKey, PreKeyRecord, SignedPreKeyRecord};

fn invalid(reason: &'static str) -> Error {
    Error::invalid("prekey_digest", reason)
}
fn required<'a>(node: &'a Node, name: &str) -> Result<&'a Node> {
    node.child(name)
        .ok_or_else(|| invalid("required digest child missing"))
}
fn bytes<const N: usize>(node: &Node) -> Result<[u8; N]> {
    match &node.content {
        NodeContent::Bytes(b) => b
            .as_ref()
            .try_into()
            .map_err(|_| invalid("digest field has incorrect byte length")),
        _ => Err(invalid("digest field requires byte content")),
    }
}
fn ids(node: &Node) -> Result<Vec<u32>> {
    node.child_nodes()
        .iter()
        .map(|n| {
            if n.description != "id" {
                return Err(invalid("digest list requires direct id children"));
            }
            Ok(id(bytes::<3>(n)?))
        })
        .collect()
}
fn id(b: [u8; 3]) -> u32 {
    u32::from_be_bytes([0, b[0], b[1], b[2]])
}
struct Reply {
    registration: [u8; 4],
    kind: u8,
    signed: [u8; 3],
    legacy: Vec<u32>,
    pq: Option<Vec<u32>>,
    lr: Option<u32>,
    hash: Vec<u8>,
}
fn parse(node: &Node) -> Result<Reply> {
    if node.description != "iq" || node.get_attr("type").and_then(|v| v.as_text()) != Some("result")
    {
        return Err(invalid(
            "requires an IQ result; offline inspection does not correlate it",
        ));
    }
    let d = required(node, "digest")?;
    let lr = d
        .child("pq_last_resort_key")
        .map(|n| bytes::<3>(required(n, "id")?).map(id))
        .transpose()?;
    let hash = required(d, "hash")?;
    let hash = if lr.is_some() {
        bytes::<32>(hash)?.to_vec()
    } else {
        bytes::<20>(hash)?.to_vec()
    };
    Ok(Reply {
        registration: bytes(required(d, "registration")?)?,
        kind: bytes::<1>(required(d, "type")?)?[0],
        signed: bytes(required(required(d, "skey")?, "id")?)?,
        legacy: ids(required(d, "list")?)?,
        pq: d.child("pq_list").map(ids).transpose()?,
        lr,
        hash,
    })
}

/// Read-only comparison outcome; none authorizes upload, reset or key release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    /// All supplied digest fields match the inspected local material.
    Matches,
    /// Type is not the native value5.
    TypeMismatch,
    /// Registration differs from local account.
    RegistrationMismatch,
    /// Signed prekey ID differs from local registered key.
    SignedKeyMismatch,
    /// A named ordinary legacy row is absent.
    MissingLegacyKey,
    /// No unique uploaded LR matches the supplied ID.
    LastResortMismatch,
    /// More than one uploaded LR exists; native row-order selection is unported.
    AmbiguousLastResort,
    /// A named ordinary KEM row is absent.
    MissingPqKey,
    /// Frozen acknowledged upload signature is unavailable or ambiguous.
    UnavailableLastResortSignature,
    /// Local key material does not reproduce the server hash.
    HashMismatch,
}
/// Redacted inspection only; contains no IDs, hashes, addresses or key bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Inspection {
    /// Comparison result, not an authenticated server disposition.
    pub disposition: Disposition,
    /// Number of legacy IDs in supplied order, preserving repeats.
    pub legacy_ids: usize,
    /// None means pq_list absent; Some(0) means present but empty.
    pub pq_ids: Option<usize>,
    /// Whether an LR ID was supplied.
    pub last_resort_present: bool,
    /// Algorithm chosen by saved gate and LR presence, independently of wire size.
    pub sha256: bool,
}

/// Inspect an IQ result against local material without writes or network calls.
/// The saved PQ gate is read separately from the account/key snapshot; this is
/// provisional offline evidence, never authority for state changes or recovery.
pub fn inspect(store: &ProtocolStore, node: &Node) -> Result<Inspection> {
    let gate = super::super::abprops::pq_keys_upload_enabled(store)?;
    store.read(|tx| {
        let account =
            AccountRecord::load(tx)?.ok_or_else(|| invalid("local account unavailable"))?;
        let signed = crate::registration::factory::registered_signed_prekey_in(tx)?;
        inspect_in(tx, &account, &signed, gate, node)
    })
}

pub(super) fn inspect_in(
    tx: &ReadTx<'_>,
    account: &AccountRecord,
    signed: &[u8],
    gate: bool,
    node: &Node,
) -> Result<Inspection> {
    let reply = parse(node)?;
    let mut out = Inspection {
        disposition: Disposition::Matches,
        legacy_ids: reply.legacy.len(),
        pq_ids: reply.pq.as_ref().map(Vec::len),
        last_resort_present: reply.lr.is_some(),
        sha256: gate && reply.lr.is_some(),
    };
    macro_rules! mismatch {
        ($d:ident) => {{
            out.disposition = Disposition::$d;
            return Ok(out);
        }};
    }
    if reply.kind != 5 {
        mismatch!(TypeMismatch);
    }
    let registration = account
        .signal_registration_id
        .ok_or_else(|| invalid("local registration unavailable"))?;
    if reply.registration != registration.to_be_bytes() {
        mismatch!(RegistrationMismatch);
    }
    let signed =
        SignedPreKeyRecord::deserialize(signed).map_err(|_| invalid("invalid local signed key"))?;
    if reply.signed.as_slice()
        != super::be3(u32::from(
            signed
                .id()
                .map_err(|_| invalid("invalid local signed key ID"))?,
        ))
    {
        mismatch!(SignedKeyMismatch);
    }
    let identity = account
        .signal_identity
        .as_ref()
        .ok_or_else(|| invalid("local identity unavailable"))?;
    let mut input = identity.0.public.to_vec();
    input.extend_from_slice(
        signed
            .public_key()
            .map_err(|_| invalid("invalid local signed public key"))?
            .public_key_bytes(),
    );
    input.extend_from_slice(
        &signed
            .signature()
            .map_err(|_| invalid("invalid local signed signature"))?,
    );
    // Fdf hashes in supplied ID order; no sorting or deduplication.
    for key_id in reply.legacy {
        let raw: Option<Vec<u8>> =
            tx.get_json(Table::Prekeys, super::record_key(key_id).as_bytes(), 1)?;
        let Some(raw) = raw else {
            mismatch!(MissingLegacyKey);
        };
        let key =
            PreKeyRecord::deserialize(&raw).map_err(|_| invalid("invalid local legacy key"))?;
        input.extend_from_slice(
            key.public_key()
                .map_err(|_| invalid("invalid local legacy public key"))?
                .public_key_bytes(),
        );
    }
    if gate {
        if let Some(lr_id) = reply.lr {
            // Current local producer creates one LR. Multiple uploaded rows need
            // native insertion-order metadata, not a guessed highest numeric ID.
            let rows = super::scan_prefix::<super::super::kem_prekeys::StoredKemPrekey>(
                tx,
                Table::Prekeys,
                super::super::kem_prekeys::RECORD_PREFIX,
            )?;
            let mut sent = rows.iter().filter(|(_, r)| r.last_resort && r.sent);
            let Some((_, lr)) = sent.next() else {
                mismatch!(LastResortMismatch);
            };
            if sent.next().is_some() {
                mismatch!(AmbiguousLastResort);
            }
            if lr.id != lr_id {
                mismatch!(LastResortMismatch);
            }
            let _ = super::super::kem_prekeys::load(tx, lr_id)?
                .ok_or_else(|| invalid("local LR disappeared"))?;
            // New reservations retain their exact publication signature. Historical
            // rows may recover it from an acknowledged frozen SET; never re-sign.
            let mut signature = lr.signature.clone();
            if let Some(sig) = &signature {
                let public = wacore_libsignal::protocol::PublicKey::from_djb_public_key_bytes(
                    &identity.0.public,
                )
                .map_err(|_| invalid("invalid local identity public key"))?;
                if !public.verify_signature(&lr.public, sig) {
                    return Err(invalid("invalid stored LR signature"));
                }
            } else {
                for (batch_key, batch) in
                    super::scan_prefix::<super::Batch>(tx, Table::Checkpoints, super::BATCH_PREFIX)?
                {
                    if batch.status != "server_acknowledged" || !batch.kem_ids.contains(&lr_id) {
                        continue;
                    }
                    let iq_id = batch_key
                        .strip_prefix(super::BATCH_PREFIX)
                        .ok_or_else(|| invalid("invalid local batch key"))?;
                    let Some(request) = tx.get_json::<serde_json::Value>(
                        Table::Checkpoints,
                        super::request_key(iq_id).as_bytes(),
                        1,
                    )?
                    else {
                        continue;
                    };
                    if request["mode"] != "set" || request["dictionary"] != "comparison_2_26_37_71"
                    {
                        continue;
                    }
                    let wire: Vec<u8> = serde_json::from_value(request["wire"].clone())
                        .map_err(|_| invalid("invalid frozen upload bytes"))?;
                    let limits = crate::profile::Limits::default();
                    let unpacked = crate::binary::unpack_frame_payload(&wire, &limits)
                        .map_err(|_| invalid("invalid frozen upload frame"))?;
                    let node = crate::binary::Decoder::new(
                        &unpacked,
                        crate::binary::TokenDictionary::comparison_2_26_37_71(),
                        &limits,
                    )
                    .decode_node()
                    .map_err(|_| invalid("invalid frozen upload node"))?;
                    if node.description != "iq"
                        || node.get_attr("id").and_then(|v| v.as_text()) != Some(iq_id)
                        || node.get_attr("type").and_then(|v| v.as_text()) != Some("set")
                        || node.get_attr("xmlns").and_then(|v| v.as_text()) != Some("encrypt")
                    {
                        return Err(invalid("frozen upload does not match saved reservation"));
                    }
                    let Some(entry) = node.child("pq_last_resort_key") else {
                        continue;
                    };
                    if id(bytes::<3>(required(entry, "id")?)?) != lr_id {
                        continue;
                    }
                    if bytes::<1568>(required(entry, "value")?)?.as_slice() != lr.public {
                        continue;
                    }
                    let sig = bytes::<64>(required(entry, "signature")?)?.to_vec();
                    let public = wacore_libsignal::protocol::PublicKey::from_djb_public_key_bytes(
                        &identity.0.public,
                    )
                    .map_err(|_| invalid("invalid local identity public key"))?;
                    if !public.verify_signature(&lr.public, &sig) {
                        return Err(invalid("invalid frozen LR signature"));
                    }
                    if signature.as_ref().is_some_and(|prior| prior != &sig) {
                        mismatch!(UnavailableLastResortSignature);
                    }
                    signature = Some(sig);
                }
            }
            let Some(signature) = signature else {
                mismatch!(UnavailableLastResortSignature);
            };
            input.extend_from_slice(&lr.public);
            input.extend_from_slice(&signature);
        }
        if let Some(pq) = reply.pq {
            for key_id in pq {
                let Some(key) = super::super::kem_prekeys::load(tx, key_id)? else {
                    mismatch!(MissingPqKey);
                };
                input.extend_from_slice(&key.public);
            }
        }
    }
    let computed = if out.sha256 {
        sha2::Sha256::digest(&input).to_vec()
    } else {
        sha1::Sha1::digest(&input).to_vec()
    };
    if computed != reply.hash {
        out.disposition = Disposition::HashMismatch;
    }
    Ok(out)
}
