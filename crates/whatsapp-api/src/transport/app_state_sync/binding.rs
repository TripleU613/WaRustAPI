//! Pure agreement checks for prospective fetch admission claims. This does not
//! prove store origin, check current transactional state, verify MACs or apply data.
//! Authority boundary: docs/protocol/app-state-inline-fetch-application-spec.md.
use super::{
    Collection,
    response::{self, Category},
};
use crate::{
    Error, Result,
    binary::Node,
    model::ids::AccountGeneration,
    signal::app_state_collection::{CollectionState, SavedCollectionState},
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

fn invalid(message: &'static str) -> Error {
    Error::invalid("app_state_fetch_binding", message)
}

/// Caller-supplied admission claim, not a proof of authenticated store origin.
/// A future wrapper must capture/persist it under the reservation transaction.
pub struct FetchBaselinePlan {
    generation: AccountGeneration,
    action_id: String,
    request_digest: [u8; 32],
    baselines: Vec<SavedCollectionState>,
}
impl FetchBaselinePlan {
    /// Freeze the digest of exact original encoded bytes and claimed baselines.
    /// Do not create this retrospectively from present state for a historical send.
    pub fn new(
        generation: AccountGeneration,
        action_id: String,
        original_wire: &[u8],
        baselines: Vec<SavedCollectionState>,
    ) -> Result<Self> {
        if action_id.is_empty()
            || action_id.len() > 128
            || original_wire.len() > 16 * 1024 * 1024
            || baselines.is_empty()
            || baselines.len() > super::ALL_COLLECTIONS.len()
        {
            return Err(invalid("unsupported admission claim dimensions"));
        }
        Ok(Self {
            generation,
            action_id,
            request_digest: Sha256::digest(original_wire).into(),
            baselines,
        })
    }
}
impl fmt::Debug for FetchBaselinePlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FetchBaselinePlan")
            .field("baseline_count", &self.baselines.len())
            .finish_non_exhaustive()
    }
}

/// Bound structural candidates, not verified or applicable mutation transitions.
pub struct BoundFetch<'a> {
    /// Local generation supplied by the admission claim and current transaction.
    pub generation: AccountGeneration,
    /// Exact original bytes; never rebuilt or normalized by this binder.
    pub original_request_wire: &'a [u8],
    /// Original structural response, retaining unsupported content.
    pub response: &'a response::Response<'a>,
    /// Requested collections in original request order, including missing replies.
    pub collections: Vec<BoundCollection<'a>>,
}
/// One requested collection and its unchanged claimed baseline.
pub struct BoundCollection<'a> {
    /// Exact known requested collection.
    pub collection: Collection,
    /// Explicit requested version equal to the validated claimed baseline.
    pub requested_version: u64,
    /// Original baseline claim; current store equality must be rechecked before commit.
    pub baseline: &'a SavedCollectionState,
    /// Original response collection, absent when not returned.
    pub response: Option<&'a response::CollectionReply<'a>>,
    /// Original response position; this request-order vector is not application order.
    pub response_index: Option<usize>,
    /// Whole selected collection's structural headers; never an integrity verdict.
    pub headers: Option<super::headers::CollectionHeaders<'a>>,
    /// No disposition authorizes state advancement on its own.
    pub disposition: Disposition<'a>,
}
/// Local structural outcomes; original response metadata remains preserved.
pub enum Disposition<'a> {
    /// Inline headers passed; supported decoding, cryptography and current-state checks remain.
    DataCandidate(&'a response::CollectionData<'a>),
    /// External mutation records require authenticated retrieval, never omission.
    UnsupportedExternalMutations(&'a response::CollectionData<'a>),
    /// Snapshot/bootstrap processing is outside this incremental contract.
    UnsupportedSnapshot(&'a response::CollectionData<'a>),
    /// A pure-fetch acknowledgement supplies no prepared outgoing hash authority.
    NoAdvanceVersionAck(i64),
    /// No synthetic new version/hash is inferred from an empty category.
    NoAdvanceAlreadySynced,
    /// Error processing, including 409 data, remains separate.
    CollectionRefusal,
    /// Missing requested response remains unresolved, not successful.
    MissingResponse,
}
impl fmt::Debug for BoundFetch<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoundFetch")
            .field("collection_count", &self.collections.len())
            .finish_non_exhaustive()
    }
}
impl fmt::Debug for BoundCollection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoundCollection")
            .field("disposition", &self.disposition)
            .finish_non_exhaustive()
    }
}
impl fmt::Debug for Disposition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::DataCandidate(_) => "DataCandidate",
            Self::UnsupportedExternalMutations(_) => "UnsupportedExternalMutations",
            Self::UnsupportedSnapshot(_) => "UnsupportedSnapshot",
            Self::NoAdvanceVersionAck(_) => "NoAdvanceVersionAck",
            Self::NoAdvanceAlreadySynced => "NoAdvanceAlreadySynced",
            Self::CollectionRefusal => "CollectionRefusal",
            Self::MissingResponse => "MissingResponse",
        })
    }
}

fn supported_attributes(node: &Node, allowed: &[&str]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for attr in &node.attributes {
        if !allowed.contains(&attr.name.as_str()) || !seen.insert(attr.name.as_str()) {
            return Err(invalid(
                "ambiguous or unsupported original request attribute",
            ));
        }
    }
    Ok(())
}

/// Bind exact stored bytes and structural response to admission claims. This
/// validates agreement only; the caller still owes authenticated capture and a
/// full current-baseline/generation recheck in each application transaction.
pub fn bind_inline_fetch_plan<'a>(
    original_wire: &'a [u8],
    response: &'a response::Response<'a>,
    admitted: &'a FetchBaselinePlan,
    current_generation: AccountGeneration,
) -> Result<BoundFetch<'a>> {
    if original_wire.len() > 16 * 1024 * 1024
        || current_generation != admitted.generation
        || <[u8; 32]>::from(Sha256::digest(original_wire)) != admitted.request_digest
    {
        return Err(invalid(
            "generation or original request fingerprint differs from admission claim",
        ));
    }
    let request = super::super::actions::decode(original_wire)
        .map_err(|_| invalid("cannot decode admitted original request"))?;
    if response::classify_response(response.original)? != *response {
        return Err(invalid(
            "structural response model differs from its preserved original",
        ));
    }
    super::require_original_fetch(&request, &admitted.action_id)?;
    if response.original.get_attr("id").and_then(|v| v.as_text())
        != Some(admitted.action_id.as_str())
    {
        return Err(invalid("response identifier differs from admitted request"));
    }
    supported_attributes(&request, &["id", "type", "xmlns", "to"])?;
    let sync = request.child("sync").expect("validated original fetch");
    supported_attributes(sync, &["data_namespace"])?;
    let mut baseline_map = BTreeMap::new();
    for baseline in &admitted.baselines {
        if Collection::from_name(&baseline.collection).is_none()
            || baseline_map
                .insert(baseline.collection.as_str(), baseline)
                .is_some()
        {
            return Err(invalid(
                "admission baselines have duplicate or unknown collection identity",
            ));
        }
        if !CollectionState::from_saved(baseline)?.has_logical_indexes() {
            return Err(invalid(
                "admission baseline lacks authenticated logical-index provenance",
            ));
        }
    }
    let mut replies = BTreeMap::new();
    for (index, reply) in response.collections.iter().enumerate() {
        if replies.insert(reply.name, (index, reply)).is_some() {
            return Err(invalid(
                "duplicate response collection is ambiguous for request binding",
            ));
        }
    }
    let mut collections = Vec::new();
    for node in sync.child_nodes() {
        supported_attributes(node, &["name", "order", "version", "return_snapshot"])?;
        if node.get_attr("return_snapshot").is_some() {
            return Err(invalid(
                "snapshot/bootstrap request is outside incremental binding",
            ));
        }
        let name = node
            .get_attr("name")
            .and_then(|v| v.as_text())
            .expect("validated fetch name");
        let collection = Collection::from_name(name).expect("validated fetch collection");
        let requested_version = node
            .get_attr("version")
            .and_then(|v| v.as_text())
            .and_then(super::super::mex::java_decimal_long)
            .and_then(|v| u64::try_from(v).ok())
            .ok_or_else(|| {
                invalid("incremental binding requires an explicit nonnegative signed-long version")
            })?;
        if node
            .get_attr("order")
            .and_then(|v| v.as_text())
            .and_then(super::super::mex::java_decimal_long)
            != Some(i64::from(collection.order()))
        {
            return Err(invalid(
                "original collection order differs from supported producer",
            ));
        }
        let baseline = baseline_map
            .remove(name)
            .ok_or_else(|| invalid("requested collection has no admitted baseline"))?;
        if baseline.version != requested_version {
            return Err(invalid(
                "request version differs from admitted held baseline",
            ));
        }
        let entry = replies.remove(name);
        let response_index = entry.map(|(index, _)| index);
        let reply = entry.map(|(_, reply)| reply);
        let data = match reply.map(|r| &r.category) {
            Some(Category::Data(data)) => Some(data),
            Some(Category::Error { conflict_data, .. }) => conflict_data.as_ref(),
            _ => None,
        };
        let headers = data
            .map(super::headers::inspect_collection_headers)
            .transpose()
            .map_err(|_| invalid("selected fetch data fails structural header admission"))?;
        let disposition = match reply.map(|r| &r.category) {
            None => Disposition::MissingResponse,
            Some(Category::VersionAck { version }) => Disposition::NoAdvanceVersionAck(*version),
            Some(Category::AlreadySynced) => Disposition::NoAdvanceAlreadySynced,
            Some(Category::Error { .. }) => Disposition::CollectionRefusal,
            Some(Category::Data(data)) if data.snapshot_ref.is_some() => {
                Disposition::UnsupportedSnapshot(data)
            }
            Some(Category::Data(data))
                if headers.as_ref().is_some_and(|h| {
                    h.patches
                        .iter()
                        .any(|p| p.source == super::headers::PatchSource::ExternalDeferred)
                }) =>
            {
                Disposition::UnsupportedExternalMutations(data)
            }
            Some(Category::Data(data)) => Disposition::DataCandidate(data),
        };
        collections.push(BoundCollection {
            collection,
            requested_version,
            baseline,
            response: reply,
            response_index,
            headers,
            disposition,
        });
    }
    if !baseline_map.is_empty() || !replies.is_empty() {
        return Err(invalid(
            "response or admission claim enlarges requested collection domain",
        ));
    }
    Ok(BoundFetch {
        generation: admitted.generation,
        original_request_wire: original_wire,
        response,
        collections,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{Encoder, TokenDictionary};
    fn baseline(collection: Collection, version: u64) -> SavedCollectionState {
        let mut saved = CollectionState::empty(collection.name())
            .unwrap()
            .to_saved();
        saved.version = version;
        saved
    }
    fn wire(entries: &[(Collection, Option<u64>)]) -> Vec<u8> {
        Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&super::super::fetch_request_at_versions("READ", entries, false).unwrap())
            .unwrap()
    }
    fn reply(children: Vec<Node>) -> Node {
        Node::new("iq")
            .attr("id", "READ")
            .attr("type", "result")
            .children(vec![Node::new("sync").children(children)])
    }
    fn claim(wire: &[u8], baselines: Vec<SavedCollectionState>) -> FetchBaselinePlan {
        FetchBaselinePlan::new(AccountGeneration(1), "READ".into(), wire, baselines).unwrap()
    }

    #[test]
    fn binding_preserves_response_positions_and_never_advances_acknowledged_versions() {
        let bytes = wire(&[
            (Collection::Regular, Some(7)),
            (Collection::RegularLow, Some(2)),
        ]);
        let admitted = claim(
            &bytes,
            vec![
                baseline(Collection::Regular, 7),
                baseline(Collection::RegularLow, 2),
            ],
        );
        let node = reply(vec![
            Node::new("collection").attr("name", "regular_low"),
            Node::new("collection")
                .attr("name", "regular")
                .attr("version", "999"),
        ]);
        let response = response::classify_response(&node).unwrap();
        let bound =
            bind_inline_fetch_plan(&bytes, &response, &admitted, AccountGeneration(1)).unwrap();
        assert_eq!(bound.original_request_wire, bytes);
        assert_eq!(bound.collections[0].response_index, Some(1));
        assert_eq!(bound.collections[1].response_index, Some(0));
        assert!(matches!(
            bound.collections[0].disposition,
            Disposition::NoAdvanceVersionAck(999)
        ));
        assert_eq!(bound.collections[0].baseline.version, 7);
        assert!(matches!(
            bound.collections[1].disposition,
            Disposition::NoAdvanceAlreadySynced
        ));
        let missing = reply(vec![]);
        let response = response::classify_response(&missing).unwrap();
        let bound =
            bind_inline_fetch_plan(&bytes, &response, &admitted, AccountGeneration(1)).unwrap();
        assert!(
            bound
                .collections
                .iter()
                .all(|c| matches!(c.disposition, Disposition::MissingResponse))
        );
    }

    #[test]
    fn frozen_wire_generation_and_exact_domains_cannot_be_reinterpreted() {
        let bytes = wire(&[(Collection::Regular, Some(7))]);
        let admitted = claim(&bytes, vec![baseline(Collection::Regular, 7)]);
        let node = reply(vec![Node::new("collection").attr("name", "regular")]);
        let response = response::classify_response(&node).unwrap();
        assert!(
            bind_inline_fetch_plan(&bytes, &response, &admitted, AccountGeneration(2)).is_err()
        );
        let mut changed = bytes.clone();
        changed.push(0);
        assert!(
            bind_inline_fetch_plan(&changed, &response, &admitted, AccountGeneration(1)).is_err()
        );
        let malformed = b"\x00private-sensitive-wire";
        let invalid_claim = claim(malformed, vec![baseline(Collection::Regular, 7)]);
        let error =
            bind_inline_fetch_plan(malformed, &response, &invalid_claim, AccountGeneration(1))
                .unwrap_err()
                .to_string();
        assert!(!error.contains("private-sensitive-wire"));
        for children in [
            vec![Node::new("collection").attr("name", "private-unknown")],
            vec![Node::new("collection").attr("name", "regular_low")],
            vec![
                Node::new("collection").attr("name", "regular"),
                Node::new("collection").attr("name", "regular"),
            ],
        ] {
            let node = reply(children);
            let response = response::classify_response(&node).unwrap();
            let error = bind_inline_fetch_plan(&bytes, &response, &admitted, AccountGeneration(1))
                .unwrap_err()
                .to_string();
            assert!(!error.contains("private-unknown"));
        }
        for version in [None, Some(0)] {
            let zero = wire(&[(Collection::Regular, version)]);
            let admitted = claim(&zero, vec![baseline(Collection::Regular, 7)]);
            assert!(
                bind_inline_fetch_plan(&zero, &response, &admitted, AccountGeneration(1)).is_err()
            );
        }
        let zero = wire(&[(Collection::Regular, Some(0))]);
        let admitted = claim(&zero, vec![baseline(Collection::Regular, 7)]);
        assert!(
            bind_inline_fetch_plan(&zero, &response, &admitted, AccountGeneration(1))
                .unwrap_err()
                .to_string()
                .contains("request version differs"),
            "zero fixture has no snapshot flag masking the version guard"
        );
    }

    #[test]
    fn admission_provenance_and_domain_are_validated_without_inventing_empty_state() {
        let bytes = wire(&[(Collection::Regular, Some(7))]);
        let node = reply(vec![Node::new("collection").attr("name", "regular")]);
        let response = response::classify_response(&node).unwrap();
        let saved = baseline(Collection::Regular, 7);
        let mut bad_hash = saved.clone();
        bad_hash.hash[0] ^= 1;
        let mut legacy = saved.clone();
        legacy.values = vec![(vec![1; 32], vec![2; 32])];
        legacy.hash = crate::signal::app_state_hash::StateHash::default()
            .updated(&[], &[[2; 32]])
            .unwrap()
            .as_bytes()
            .to_vec();
        legacy.logical_indexes = None;
        for baselines in [
            vec![bad_hash],
            vec![legacy],
            vec![saved.clone(), saved.clone()],
            vec![baseline(Collection::RegularLow, 7)],
            vec![saved.clone(), baseline(Collection::RegularLow, 2)],
        ] {
            let admitted = claim(&bytes, baselines);
            assert!(
                bind_inline_fetch_plan(&bytes, &response, &admitted, AccountGeneration(1)).is_err()
            );
        }
        assert!(
            FetchBaselinePlan::new(AccountGeneration(1), "READ".into(), &bytes, vec![]).is_err()
        );
        let mut legacy_empty = saved;
        legacy_empty.logical_indexes = None;
        let admitted = claim(&bytes, vec![legacy_empty]);
        let bound =
            bind_inline_fetch_plan(&bytes, &response, &admitted, AccountGeneration(1)).unwrap();
        assert_eq!(
            bound.collections[0].baseline.version, 7,
            "valid empty legacy baseline preserves its version"
        );
    }

    #[test]
    fn all_selected_headers_preflight_before_candidates_and_external_data_stays_deferred() {
        let bytes = wire(&[(Collection::Regular, Some(7))]);
        let admitted = claim(&bytes, vec![baseline(Collection::Regular, 7)]);
        for external in [false, true] {
            let mut patch = vec![0x0a, 2, 8, 8];
            if external {
                patch.extend([0x1a, 0]);
            }
            let node = reply(vec![
                Node::new("collection")
                    .attr("name", "regular")
                    .children(vec![
                        Node::new("patches").children(vec![Node::new("patch").bytes(patch)]),
                    ]),
            ]);
            let response = response::classify_response(&node).unwrap();
            let bound =
                bind_inline_fetch_plan(&bytes, &response, &admitted, AccountGeneration(1)).unwrap();
            assert_eq!(
                bound.collections[0].headers.as_ref().unwrap().patches[0].version,
                8
            );
            assert_eq!(bound.collections[0].baseline.version, 7);
            if external {
                assert!(matches!(
                    bound.collections[0].disposition,
                    Disposition::UnsupportedExternalMutations(_)
                ));
            } else {
                assert!(matches!(
                    bound.collections[0].disposition,
                    Disposition::DataCandidate(_)
                ));
            }
        }
        let node = reply(vec![
            Node::new("collection")
                .attr("name", "regular")
                .children(vec![Node::new("patches").children(vec![
                    Node::new("patch").bytes(vec![0x0a, 2, 8, 8]),
                    Node::new("patch").bytes(vec![0x0a]),
                ])]),
        ]);
        let response = response::classify_response(&node).unwrap();
        assert!(
            bind_inline_fetch_plan(&bytes, &response, &admitted, AccountGeneration(1)).is_err()
        );
        assert_eq!(
            admitted.baselines[0].version, 7,
            "late malformed header exposes no application plan"
        );
    }

    #[test]
    fn modified_public_response_models_cannot_replace_the_preserved_original() {
        let bytes = wire(&[(Collection::Regular, Some(7))]);
        let admitted = claim(&bytes, vec![baseline(Collection::Regular, 7)]);
        let node = reply(vec![
            Node::new("collection")
                .attr("name", "regular")
                .attr("has_more_patches", "true")
                .children(vec![
                    Node::new("patches").children(vec![Node::new("patch").bytes(vec![1])]),
                ]),
        ]);
        let rogue = Node::new("private-rogue");
        for case in 0..6 {
            let mut response = response::classify_response(&node).unwrap();
            match case {
                0 => response.collections[0].name = "regular_low",
                1 => response.collections[0].known_collection = None,
                2 => response.collections[0].category = Category::VersionAck { version: 7 },
                3 => response.collections[0].original = &rogue,
                _ => {
                    if let Category::Data(data) = &mut response.collections[0].category {
                        if case == 4 {
                            data.has_more_patches = false;
                        } else {
                            data.patches[0] = b"private-replacement";
                        }
                    }
                }
            }
            let error = bind_inline_fetch_plan(&bytes, &response, &admitted, AccountGeneration(1))
                .unwrap_err()
                .to_string();
            assert!(!error.contains("private"));
        }
        assert!(!format!("{admitted:?}").contains("READ"));
    }
}
