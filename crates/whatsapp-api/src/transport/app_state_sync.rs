//! Outgoing `w:sync:app:state` patch requests.
//!
//! Comparison 2.26.37.71 `CSe.A00`/`CSe.A01` (**confirmed** shape, recorded in
//! `docs/protocol/chat-actions.md` section 6):
//!
//! ```text
//! <iq to="s.whatsapp.net" type="set" xmlns="w:sync:app:state" id=…>
//!   <sync>
//!     <collection name=<collection> order=<n> [version=<v>] [return_snapshot="true"]>
//!       <patch>{SyncdPatch bytes}</patch>
//!     </collection>
//!   </sync>
//! </iq>
//! ```
//!
//! Low-level builders coexist with durable publication, held-key selection and
//! authenticated patch settlement. Preparation and original wire are frozen
//! together; held state advances only after a verified correlated transition.
//! Current-build evidence and unported fetch lifecycle: docs/protocol/app-state-fetch-completion.md.
use crate::{Result, binary::Node, model::jid::Jid};
pub(crate) mod action_journal;
pub mod binding;
pub mod headers;
pub mod held_fetch;
pub mod response;

/// The five app-state collections (`LX/0oP.A03`, **confirmed** names).
///
/// Which collection a given action belongs to is a separate question from the
/// name list, and is **not** encoded here: callers pass the collection they
/// have evidence for. Guessing it would produce a patch the server may accept
/// while companions look for the mutation in the wrong place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collection {
    /// `regular`
    Regular,
    /// `regular_low`
    RegularLow,
    /// `regular_high`
    RegularHigh,
    /// `critical_block`
    CriticalBlock,
    /// `critical_unblock_low`
    CriticalUnblockLow,
}

impl Collection {
    /// Wire name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::RegularLow => "regular_low",
            Self::RegularHigh => "regular_high",
            Self::CriticalBlock => "critical_block",
            Self::CriticalUnblockLow => "critical_unblock_low",
        }
    }

    /// `order` attribute (**confirmed**, `CSe.A01`):
    /// ```text
    /// 0014  "critical_unblock_low".equals(name) -> v0
    /// 0020  xor-int/lit8 v3, v0, #0x1          ; order = !isCriticalUnblockLow
    /// 003c  0hq("order", v3)
    /// ```
    /// So every collection emits `order=1` **except** `critical_unblock_low`,
    /// which emits `0`. Note this is the inverse of the "0 normally, 1 for
    /// critical_unblock_low" claim an earlier trace left in
    /// `docs/protocol/chat-actions.md`; that note was wrong and is corrected.
    pub fn order(self) -> u8 {
        u8::from(self != Self::CriticalUnblockLow)
    }

    /// The collection a wire or saved name refers to.
    pub fn from_name(name: &str) -> Option<Self> {
        ALL_COLLECTIONS.into_iter().find(|c| c.name() == name)
    }
}

/// Build an app-state **fetch**: ask the server for each collection's current
/// snapshot. This is a read — it carries no `<patch>` child and mutates
/// nothing.
///
/// `CSe.A01` adds the `<patch>` child only when its patch argument is non-null,
/// and emits `return_snapshot="true"` only when its boolean argument is true
/// **and** the version argument is null or zero (**confirmed**). A collection at
/// version 0 asking for a snapshot is therefore exactly the shape a client sends
/// when it has no app state yet — which is this account's situation: its
/// `offline_preview` has always reported `appdata=0`, so it has never received
/// app-state data, and it has never asked.
///
/// Sending this is worthwhile independently of any status question: the reply
/// carries the server's authoritative version for each collection, which patch
/// writing needs, since both our verifier and the server require patches to be
/// strictly sequential from the current version.
pub fn fetch_request(id: &str, collections: &[Collection]) -> Result<Node> {
    let versions: Vec<_> = collections.iter().map(|c| (*c, Some(0))).collect();
    fetch_request_at_versions(id, &versions, true)
}

/// Build a patch-free fetch from each collection's held, verified version.
///
/// **Confirmed, 2.26.43.72 `EfK.A01`:** `None` omits `version`; a held
/// version is emitted unchanged. `return_snapshot` is emitted only when
/// requested and the held version is absent or zero. See
/// `docs/protocol/app-state-versioned-fetch.md` for current producer evidence.
/// These are applied versions, never a `server_sync` notification's advertised
/// dirty versions. This builder does not schedule work, reserve an IQ or ACK a
/// notification; those require the lifecycle in `server-sync-lifecycle.md`.
pub fn fetch_request_at_versions(
    id: &str,
    collections: &[(Collection, Option<u64>)],
    return_snapshot: bool,
) -> Result<Node> {
    if id.is_empty() || id.len() > 128 {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "invalid request id",
        ));
    }
    if collections.is_empty() {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "a fetch needs at least one collection",
        ));
    }
    // EfK.A01 passes a Java Long unchanged to 0Tl(String,long), which formats
    // its decimal value. Do not wrap an unsigned version into a negative Long.
    if collections
        .iter()
        .any(|(_, version)| version.is_some_and(|v| v > i64::MAX as u64))
    {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "held collection version exceeds the APK signed-long range",
        ));
    }
    // The current producer (0aS.A0Q -> Dxc.run(4) -> FJU.accept(12))
    // prepares one result per collection-keyed map entry. EfK itself appends
    // children; enforce that producer invariant at this slice-based boundary.
    for (index, (collection, _)) in collections.iter().enumerate() {
        if collections[..index].iter().any(|(c, _)| c == collection) {
            return Err(crate::Error::invalid(
                "app_state_sync",
                format!("duplicate fetch collection: {}", collection.name()),
            ));
        }
    }
    let children: Vec<Node> = collections
        .iter()
        .map(|(c, version)| {
            let mut node = Node::new("collection")
                .attr("name", c.name())
                .attr("order", c.order().to_string().as_str());
            if let Some(version) = version {
                node = node.attr("version", version.to_string().as_str());
            }
            if return_snapshot && version.is_none_or(|v| v == 0) {
                node = node.attr("return_snapshot", "true");
            }
            node
        })
        .collect();
    Ok(Node::new("iq")
        .attr("id", id)
        .attr("type", "set")
        .attr("xmlns", "w:sync:app:state")
        .attr(
            "to",
            Jid::parse("s.whatsapp.net").expect("constant server JID"),
        )
        .children(vec![
            Node::new("sync")
                .attr("data_namespace", "3")
                .children(children),
        ]))
}

/// Every collection, for a first full fetch.
pub const ALL_COLLECTIONS: [Collection; 5] = [
    Collection::Regular,
    Collection::RegularLow,
    Collection::RegularHigh,
    Collection::CriticalBlock,
    Collection::CriticalUnblockLow,
];

/// Build one patch request. `patch` is `SyncdPatch` wire bytes from
/// `CollectionState::build_patch`, and `version` is the version that patch
/// claims, i.e. the collection's current version plus one.
///
/// `version` is **the version the client currently holds for this collection**,
/// not the version the patch claims — and it is `None` for a collection that
/// has never been synced (**confirmed**, both `CSe.A01` call sites). `LX/D44`
/// looks it up by name in the `CollectionVersionsTable` (`LX/0rP->A02`, a
/// `SELECT ... WHERE collection_name = ?` returning the stored `version`
/// column), and `LX/DUo` likewise sources it from stored state; neither reads
/// it from the patch. `A02` returns null when no row matches, and `CSe.A01`'s
/// `if-eqz` on that argument then omits the attribute altogether. The patch's
/// own version lives inside the `SyncdPatch` protobuf and is a separate value.
///
/// Sending `version="1"` on a first patch — the patch's claimed version rather
/// than the held one — is what the server answered with `400 bad-request`.
///
/// `return_snapshot` is honoured only when `version` is `None` or 0, matching
/// `CSe.A01`, which emits it when its boolean is true **and** the version
/// argument is null or zero.
pub fn patch_request(
    id: &str,
    collection: Collection,
    version: Option<u64>,
    patch: &[u8],
    return_snapshot: bool,
) -> Result<Node> {
    if version.is_some_and(|v| v > i64::MAX as u64) {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "held collection version exceeds the APK signed-long range",
        ));
    }
    if id.is_empty() || id.len() > 128 {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "invalid request id",
        ));
    }
    if patch.is_empty() || patch.len() > 16 * 1024 * 1024 {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "invalid patch size",
        ));
    }
    let mut node = Node::new("collection")
        .attr("name", collection.name())
        .attr("order", collection.order().to_string().as_str());
    if let Some(version) = version {
        node = node.attr("version", version.to_string().as_str());
    }
    if return_snapshot && version.is_none_or(|v| v == 0) {
        node = node.attr("return_snapshot", "true");
    }
    let collection_node = node.children(vec![Node::new("patch").bytes(patch.to_vec())]);
    Ok(Node::new("iq")
        .attr("id", id)
        .attr("type", "set")
        .attr("xmlns", "w:sync:app:state")
        .attr(
            "to",
            Jid::parse("s.whatsapp.net").expect("constant server JID"),
        )
        // CSe.A00 emits data_namespace as a numeric attribute with the literal
        // value 3 (confirmed). The optional <bot><coex mode="sync"/> child it
        // can append is not sent.
        .children(vec![
            Node::new("sync")
                .attr("data_namespace", "3")
                .children(vec![collection_node]),
        ]))
}

/// Durable action kind for an app-state fetch.
pub const FETCH_ACTION_KIND: &str = "app-state-fetch";
/// Prospective held-version fetch with a sealed preparation/admission record.
pub const HELD_FETCH_ACTION_KIND: &str = "app-state-held-fetch";

/// Queue a full app-state fetch through the durable one-shot action queue.
/// Returns the action id; nothing is sent until a session drains the queue.
/// Read-only: no patch, no mutation.
pub fn queue_fetch(store: &crate::store::ProtocolStore) -> Result<String> {
    ingest_fetch_backoff(store)?;
    store.write(|tx| {
        require_server_ready_in(tx)?;
        super::actions::queue_in(tx, FETCH_ACTION_KIND, "app-state fetch", |id| {
            fetch_request(id, &ALL_COLLECTIONS)
        })
    })
}

/// Durable action kind for an app-state patch write.
pub const PATCH_ACTION_KIND: &str = "app-state-patch";

/// Queue a `contactAction` patch for every saved contact, so companion devices
/// receive this account's address book.
///
/// Routing is **confirmed** (`LX/Azs->A00`): contactAction goes to
/// `critical_unblock_low`, not a "regular" collection. The index and schema
/// version come from `signal::app_state_actions::contact_mutation`.
///
/// Mints this device's app-state key if it has none. The post-patch collection
/// state is deliberately **not** persisted here: it must only be saved once the
/// server accepts the patch, otherwise the local version desynchronizes from the
/// server's and every later patch is refused as non-sequential.
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub fn queue_contact_patch(
    store: &crate::store::ProtocolStore,
    device_index: u16,
    raw_id: u32,
    registered: &std::collections::BTreeSet<u32>,
    now_ms: i64,
) -> Result<(String, usize)> {
    use crate::signal::app_state_actions;
    use crate::transport::contacts;

    // Promote anything the server accepted since the last call, so this patch
    // builds on it rather than re-claiming a version the server has moved past.
    ingest(store)?;
    let saved = contacts::list(store)?;
    if saved.is_empty() {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "no saved contacts to publish",
        ));
    }
    let mut mutations = Vec::new();
    for contact in &saved {
        let digits = contact.phone.strip_prefix('+').unwrap_or(&contact.phone);
        let jid = Jid::parse(&format!("{digits}@s.whatsapp.net"))?;
        let action = app_state_actions::contact_mutation(
            &jid,
            &contact.full_name,
            contact.first_name.as_deref(),
            true,
            now_ms,
        )?;
        mutations.push(
            crate::signal::app_state_collection::patch::OutgoingMutation {
                operation: crate::signal::app_state_crypto::MutationOperation::Set,
                index: action.index,
                action: Some(action.value),
                version: action.version,
            },
        );
    }
    let count = mutations.len();
    let id = queue_patch(
        store,
        device_index,
        raw_id,
        registered,
        now_ms,
        Collection::CriticalUnblockLow,
        &mutations,
        "contact app-state patch",
    )?;
    Ok((id, count))
}

/// Queue a `pushNameSetting` patch, so this account's other devices show the
/// same display name. Mutation, index and version come from
/// `signal::app_state_actions::push_name_mutation`; the collection is
/// `critical_block` (`LX/B66.<clinit>` routes `LX/Azq.A1A` through
/// `LX/Azs->A00`, **confirmed**).
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
pub fn queue_push_name_patch(
    store: &crate::store::ProtocolStore,
    device_index: u16,
    raw_id: u32,
    registered: &std::collections::BTreeSet<u32>,
    now_ms: i64,
    name: &str,
) -> Result<String> {
    ingest(store)?;
    let action = crate::signal::app_state_actions::push_name_mutation(name, now_ms)?;
    let mutation = crate::signal::app_state_collection::patch::OutgoingMutation {
        operation: crate::signal::app_state_crypto::MutationOperation::Set,
        index: action.index,
        action: Some(action.value),
        version: action.version,
    };
    queue_patch(
        store,
        device_index,
        raw_id,
        registered,
        now_ms,
        Collection::CriticalBlock,
        std::slice::from_ref(&mutation),
        "push-name app-state patch",
    )
}

/// Build one patch over the state the server last accepted for `collection`
/// and queue it, stashing the resulting state until the server accepts it.
#[cfg(all(
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
#[allow(clippy::too_many_arguments)]
fn queue_patch(
    store: &crate::store::ProtocolStore,
    device_index: u16,
    raw_id: u32,
    registered: &std::collections::BTreeSet<u32>,
    now_ms: i64,
    collection: Collection,
    mutations: &[crate::signal::app_state_collection::patch::OutgoingMutation],
    label: &str,
) -> Result<String> {
    use crate::signal::app_state;
    store.write(|tx| {
        // Native sync is single-flight; the explicit Rust action queue must at
        // least prevent two writes from claiming the same held collection state.
        ensure_collection_idle(tx, collection)?;
        let keys = match app_state::select_active_key(
            tx,
            raw_id,
            device_index.into(),
            registered,
            now_ms,
            i64::MAX,
        )? {
            Some(keys) => keys,
            None => {
                // APK 0oK.A05 permits local generation when no eligible key exists.
                // A malformed stored key is an error, never a reason to mint.
                app_state::mint_key(tx, device_index, raw_id, registered, now_ms)?;
                app_state::select_active_key(
                    tx,
                    raw_id,
                    device_index.into(),
                    registered,
                    now_ms,
                    i64::MAX,
                )?
                .ok_or_else(|| {
                    crate::Error::invalid("app_state_sync", "minted key is not selectable")
                })?
            }
        };
        let state = held_state_in(tx, collection)?;
        let held = state.version();
        let built = state.build_patch(keys.keys(), *keys.id(), device_index.into(), mutations)?;
        if built.state.version() > i64::MAX as u64 {
            return Err(invalid_patch(
                "prepared collection version exceeds the APK signed-long range",
            ));
        }
        let id = super::actions::queue_in(tx, PATCH_ACTION_KIND, label, |id| {
            patch_request(
                id,
                collection,
                (held > 0).then_some(held),
                &built.bytes,
                false,
            )
        })?;
        // One commit binds the sendable wire to its exact baseline and result.
        tx.put_json(
            crate::store::Table::Checkpoints,
            pending_key(&id).as_bytes(),
            1,
            &PendingState::Bound {
                base: state.to_saved(),
                prepared: built.state.to_saved(),
            },
        )?;
        Ok(id)
    })
}

const HELD_PREFIX: &str = "app-state-collection:";
const PENDING_PREFIX: &str = "app-state-pending:";
const SERVER_BACKOFF: &[u8] = b"app-state-server-backoff-until";

/// Redacted structural classification; this does not authorize state application.
#[derive(Debug, serde::Serialize)]
pub struct ResponseInspection {
    /// Original immutable action identifier.
    pub action_id: String,
    /// Structural classification never verifies snapshot or mutation integrity.
    pub integrity_verified: bool,
    /// Exact prospective store provenance and structural binding, for successful
    /// held fetches only. Never current-state equality or MAC verification.
    pub admission_bound: Option<bool>,
    /// Redacted summaries in original collection order; unknown names are omitted.
    pub collections: Vec<CollectionReplyInspection>,
}

/// Structural summary with no raw names, bodies, reference keys or error text.
#[derive(Debug, serde::Serialize)]
pub struct CollectionReplyInspection {
    /// Known static collection name, absent for an unknown server-provided name.
    pub known_collection: Option<&'static str>,
    /// Fixed structural category.
    pub category: &'static str,
    /// Native signed version acknowledgement, separate from applied state.
    pub version_ack: Option<i64>,
    /// Parsed error code; -1 is the native absent-code sentinel.
    pub error_code: Option<i32>,
    /// Whether opaque error text was present; its content is not printed.
    pub has_error_text: bool,
    /// Whether opaque external-reference bytes were present, not validated.
    pub snapshot_ref_present: bool,
    /// Selected inline patch bodies, not protobuf/MAC verified.
    pub patch_count: usize,
    /// Literal native continuation flag only for consumed data categories.
    pub has_more_patches: Option<bool>,
    /// Structural header outcome only for consumed data, never MAC verification.
    pub header_status: Option<&'static str>,
    /// Native parser fatal reason when established; not a server error code.
    pub native_header_reason: Option<u32>,
    /// Zero-based selected patch index for native header failures, absent for snapshot/local failures.
    pub failed_patch_index: Option<usize>,
    /// Structurally parsed patch versions; never adopted as held state.
    pub header_patch_versions: Vec<u64>,
    /// External mutation patches requiring authenticated retrieval.
    pub external_mutation_patch_count: Option<usize>,
}

/// Classify a saved correlated app-state result without keys, downloads or writes.
pub fn inspect_response(
    store: &crate::store::ProtocolStore,
    id: &str,
) -> Result<ResponseInspection> {
    store.read(|tx| {
        let (outcome, reply) = super::actions::view_in(tx, id)?
            .ok_or_else(|| invalid_patch("missing saved app-state action"))?;
        if !matches!(
            outcome.kind.as_str(),
            FETCH_ACTION_KIND | HELD_FETCH_ACTION_KIND | PATCH_ACTION_KIND
        ) {
            return Err(invalid_patch("selected action is not app-state sync"));
        }
        let request = super::actions::request_in(tx, id)?
            .ok_or_else(|| invalid_patch("missing original app-state request"))?;
        let reply = reply.ok_or_else(|| invalid_patch("selected action has no preserved reply"))?;
        if request.description != "iq"
            || request.get_attr("type").and_then(|v| v.as_text()) != Some("set")
            || request.get_attr("xmlns").and_then(|v| v.as_text()) != Some("w:sync:app:state")
            || request.get_attr("id").and_then(|v| v.as_text()) != Some(id)
            || reply.get_attr("id").and_then(|v| v.as_text()) != Some(id)
        {
            return Err(invalid_patch(
                "saved result is not correlated with its app-state request",
            ));
        }
        let classified = response::classify_response(&reply)?;
        let admission_bound = if outcome.kind == HELD_FETCH_ACTION_KIND
            && outcome.status == super::actions::ActionStatus::Ok
        {
            Some(held_fetch::binding_in(tx, id, &classified)?)
        } else {
            None
        };
        let collections = classified
            .collections
            .into_iter()
            .map(|collection| {
                use response::Category;
                let mut summary = CollectionReplyInspection {
                    known_collection: collection.known_collection.map(Collection::name),
                    category: "already_synced",
                    version_ack: None,
                    error_code: None,
                    has_error_text: false,
                    snapshot_ref_present: false,
                    patch_count: 0,
                    has_more_patches: None,
                    header_status: None,
                    native_header_reason: None,
                    failed_patch_index: None,
                    header_patch_versions: Vec::new(),
                    external_mutation_patch_count: None,
                };
                let data = match collection.category {
                    Category::Error {
                        code,
                        text,
                        conflict_data,
                    } => {
                        summary.category = "error";
                        summary.error_code = Some(code);
                        summary.has_error_text = text.is_some();
                        conflict_data
                    }
                    Category::VersionAck { version } => {
                        summary.category = "version_ack";
                        summary.version_ack = Some(version);
                        None
                    }
                    Category::AlreadySynced => None,
                    Category::Data(data) => {
                        summary.category = "data";
                        Some(data)
                    }
                };
                if let Some(data) = data {
                    summary.snapshot_ref_present = data.snapshot_ref.is_some();
                    summary.patch_count = data.patches.len();
                    summary.has_more_patches = Some(data.has_more_patches);
                    match headers::inspect_collection_headers(&data) {
                        Ok(headers) => {
                            let external = headers
                                .patches
                                .iter()
                                .filter(|p| p.source == headers::PatchSource::ExternalDeferred)
                                .count();
                            summary.header_status =
                                Some(if headers.snapshot_ref.is_some() || external > 0 {
                                    "external_data_deferred"
                                } else {
                                    "inline_headers"
                                });
                            summary.header_patch_versions =
                                headers.patches.iter().map(|p| p.version).collect();
                            summary.external_mutation_patch_count = Some(external);
                        }
                        Err(failure) => {
                            summary.native_header_reason = failure.native_reason();
                            if let headers::HeaderFailure::NativeFatal { patch_index, .. } = failure
                            {
                                summary.failed_patch_index = patch_index;
                            }
                            summary.header_status =
                                Some(if summary.native_header_reason.is_some() {
                                    "native_fatal_header"
                                } else {
                                    "local_header_refusal"
                                });
                        }
                    }
                }
                summary
            })
            .collect();
        Ok(ResponseInspection {
            action_id: id.to_owned(),
            integrity_verified: false,
            admission_bound,
            collections,
        })
    })
}

/// Redacted durable collection state; this is not native scheduler eligibility.
#[derive(Debug, serde::Serialize)]
pub struct CollectionInspection {
    /// Known collection name.
    pub collection: &'static str,
    /// Saved verified version, absent when no checkpoint exists.
    pub held_version: Option<u64>,
    /// Number of retained integrity entries.
    pub mutation_count: usize,
    /// Whether authenticated logical-index metadata covers every entry.
    pub logical_indexes_complete: bool,
}

/// Inspect validated saved collections without fetching, reserving or migrating.
pub fn inspect_collections(
    store: &crate::store::ProtocolStore,
) -> Result<Vec<CollectionInspection>> {
    use crate::{
        signal::app_state_collection::{CollectionState, SavedCollectionState},
        store::Table,
    };
    store.read(|tx| {
        ALL_COLLECTIONS
            .iter()
            .map(|collection| {
                let saved: Option<SavedCollectionState> =
                    tx.get_json(Table::Checkpoints, held_key(*collection).as_bytes(), 1)?;
                let state = match &saved {
                    Some(saved) if saved.collection == collection.name() => {
                        CollectionState::from_saved(saved)?
                    }
                    Some(_) => {
                        return Err(invalid_patch("held state names a different collection"));
                    }
                    None => CollectionState::empty(collection.name())?,
                };
                Ok(CollectionInspection {
                    collection: collection.name(),
                    held_version: saved.map(|s| s.version),
                    mutation_count: state.len(),
                    logical_indexes_complete: state.has_logical_indexes(),
                })
            })
            .collect()
    })
}

/// Reconstruct only logical-index metadata from a matching authenticated local
/// snapshot. Never advance a collection, apply actions, fetch or clear pending work.
pub fn reconstruct_logical_indexes(
    store: &crate::store::ProtocolStore,
    collection: Collection,
    snapshot_bytes: &[u8],
) -> Result<bool> {
    use crate::{
        signal::{
            app_state,
            app_state_collection::{CollectionSnapshot, CollectionState, SavedCollectionState},
        },
        store::Table,
    };
    let snapshot = CollectionSnapshot::decode(snapshot_bytes)?;
    store.write(|tx| {
        ensure_no_pending_collection_in(tx, collection)?;
        let saved: SavedCollectionState = tx
            .get_json(Table::Checkpoints, held_key(collection).as_bytes(), 1)?
            .ok_or_else(|| invalid_patch("reconstruction requires an existing held checkpoint"))?;
        if saved.collection != collection.name() {
            return Err(invalid_patch("held state names a different collection"));
        }
        let baseline = CollectionState::from_saved(&saved)?;
        let mut keys = std::collections::BTreeMap::new();
        for id in snapshot.key_ids() {
            if let std::collections::btree_map::Entry::Vacant(entry) = keys.entry(id) {
                entry.insert(
                    app_state::keys_for(tx, &id)?
                        .ok_or_else(|| invalid_patch("snapshot names a key we do not hold"))?,
                );
            }
        }
        let restored = baseline.reconstruct_logical_indexes(&snapshot, |id| {
            keys.get(id)
                .ok_or_else(|| invalid_patch("snapshot references an unheld key"))
        })?;
        let changed = !baseline.has_logical_indexes();
        if changed {
            tx.put_json(
                Table::Checkpoints,
                held_key(collection).as_bytes(),
                1,
                &restored.to_saved(),
            )?;
        }
        Ok(changed)
    })
}

fn held_key(collection: Collection) -> String {
    format!("{HELD_PREFIX}{}", collection.name())
}
fn pending_key(action_id: &str) -> String {
    format!("{PENDING_PREFIX}{action_id}")
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum PendingState {
    Bound {
        base: crate::signal::app_state_collection::SavedCollectionState,
        prepared: crate::signal::app_state_collection::SavedCollectionState,
    },
    // Existing encrypted stores retain their original prepared-state format.
    Legacy(crate::signal::app_state_collection::SavedCollectionState),
}
impl PendingState {
    fn prepared(&self) -> &crate::signal::app_state_collection::SavedCollectionState {
        match self {
            Self::Bound { prepared, .. } | Self::Legacy(prepared) => prepared,
        }
    }
}

fn ensure_collection_idle(tx: &crate::store::WriteTx<'_>, collection: Collection) -> Result<()> {
    require_server_ready_in(tx)?;
    ensure_no_pending_collection_in(tx, collection)
}

fn require_server_ready_in(tx: &crate::store::WriteTx<'_>) -> Result<()> {
    if let Some(until) = active_backoff_in(tx)? {
        return Err(crate::Error::invalid(
            "app_state_sync",
            format!(
                "server app-state backoff is active until {} ms since Unix epoch",
                until.0
            ),
        ));
    }
    Ok(())
}

pub(crate) fn server_waiting_in(tx: &crate::store::WriteTx<'_>) -> Result<bool> {
    Ok(active_backoff_in(tx)?.is_some())
}

fn ensure_no_pending_collection_in(
    tx: &crate::store::WriteTx<'_>,
    collection: Collection,
) -> Result<()> {
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(crate::store::Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        let done = rows.len() < 256;
        cursor = rows.last().map(|(key, _)| key.clone());
        for (key, _) in rows {
            if !key.starts_with(PENDING_PREFIX.as_bytes()) {
                continue;
            }
            let pending: PendingState = tx
                .get_json(crate::store::Table::Checkpoints, &key, 1)?
                .ok_or_else(|| {
                    crate::Error::invalid("app_state_sync", "pending patch disappeared")
                })?;
            if pending.prepared().collection == collection.name() {
                return Err(crate::Error::invalid(
                    "app_state_sync",
                    "collection has an unsettled patch; inspect its original action",
                ));
            }
        }
        if done {
            return Ok(());
        }
    }
}

pub(crate) fn server_waiting_read_in(tx: &crate::store::ReadTx<'_>) -> Result<bool> {
    let until = tx.get_json(crate::store::Table::Checkpoints, SERVER_BACKOFF, 1)?;
    Ok(active_backoff_until(until)?.is_some())
}

fn active_backoff_in(
    tx: &crate::store::WriteTx<'_>,
) -> Result<Option<crate::model::time::Timestamp>> {
    let until: Option<crate::model::time::Timestamp> =
        tx.get_json(crate::store::Table::Checkpoints, SERVER_BACKOFF, 1)?;
    active_backoff_until(until)
}

fn active_backoff_until(
    until: Option<crate::model::time::Timestamp>,
) -> Result<Option<crate::model::time::Timestamp>> {
    if until.is_some_and(|v| v.0 < 0) {
        return Err(invalid_patch("invalid stored app-state backoff"));
    }
    Ok(until.filter(|until| *until > crate::model::time::Timestamp::now()))
}

fn retain_backoff_in(tx: &mut crate::store::WriteTx<'_>, id: &str, error: &Node) -> Result<()> {
    // APK 43.72 FUc.Bvj reads integral seconds from the outer error. Anchor
    // at its preserved receipt time, never a later repeated inspection.
    let Some(raw) = error.get_attr("backoff") else {
        return Ok(());
    };
    let seconds = raw
        .as_text()
        .and_then(super::mex::java_decimal_long)
        .ok_or_else(|| invalid_patch("invalid server app-state backoff"))?;
    if seconds == -1 {
        return Ok(());
    }
    if seconds < 0 {
        return Err(invalid_patch("negative server app-state backoff"));
    }
    let received = super::actions::settled_at_in(tx, id)?
        .ok_or_else(|| invalid_patch("server backoff has no preserved receipt time"))?;
    if received.0 < 0 {
        return Err(invalid_patch("invalid preserved backoff receipt time"));
    }
    let deadline = seconds
        .checked_mul(1000)
        .and_then(|ms| received.0.checked_add(ms))
        .map(crate::model::time::Timestamp)
        .ok_or_else(|| invalid_patch("server app-state backoff overflows local time"))?;
    let prior: Option<crate::model::time::Timestamp> =
        tx.get_json(crate::store::Table::Checkpoints, SERVER_BACKOFF, 1)?;
    if prior.is_some_and(|p| p.0 < 0) {
        return Err(invalid_patch("invalid stored app-state backoff"));
    }
    tx.put_json(
        crate::store::Table::Checkpoints,
        SERVER_BACKOFF,
        1,
        &prior.map_or(deadline, |p| p.max(deadline)),
    )
}

fn definite_error(node: &Node) -> Result<&Node> {
    let mut errors = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "error");
    let error = errors
        .next()
        .ok_or_else(|| invalid_patch("app-state refusal has no error detail"))?;
    let code = error
        .get_attr("code")
        .and_then(|v| v.as_text())
        .and_then(super::mex::java_decimal_long)
        .and_then(|v| i32::try_from(v).ok());
    if errors.next().is_some() || code.is_none_or(|v| v == -1) {
        return Err(invalid_patch("ambiguous or malformed app-state refusal"));
    }
    Ok(error)
}

/// Read and validate the verified state inside the same transaction as its use.
fn held_state_in(
    tx: &crate::store::WriteTx<'_>,
    collection: Collection,
) -> Result<crate::signal::app_state_collection::CollectionState> {
    use crate::signal::app_state_collection::{CollectionState, SavedCollectionState};
    let saved: Option<SavedCollectionState> = tx.get_json(
        crate::store::Table::Checkpoints,
        held_key(collection).as_bytes(),
        1,
    )?;
    match saved {
        Some(saved) if saved.collection == collection.name() => CollectionState::from_saved(&saved),
        Some(_) => Err(crate::Error::invalid(
            "app_state_sync",
            "held state names a different collection",
        )),
        None => CollectionState::empty(collection.name()),
    }
}

fn invalid_patch(reason: &'static str) -> crate::Error {
    crate::Error::invalid("app_state_sync", reason)
}

fn one_collection(node: &Node, collection: Collection) -> Result<&Node> {
    let mut syncs = node
        .child_nodes()
        .iter()
        .filter(|n| n.description == "sync");
    let sync = syncs
        .next()
        .ok_or_else(|| invalid_patch("patch reply/request has no sync"))?;
    if syncs.next().is_some() {
        return Err(invalid_patch("ambiguous sync envelope"));
    }
    let mut collections = sync
        .child_nodes()
        .iter()
        .filter(|n| n.description == "collection");
    let target = collections
        .next()
        .ok_or_else(|| invalid_patch("missing patch collection"))?;
    if collections.next().is_some()
        || target.get_attr("name").and_then(|v| v.as_text()) != Some(collection.name())
    {
        return Err(invalid_patch(
            "patch collection does not match original action",
        ));
    }
    Ok(target)
}

fn original_patch<'a>(request: &'a Node, id: &str, collection: Collection) -> Result<&'a Node> {
    if request.description != "iq"
        || request.get_attr("id").and_then(|v| v.as_text()) != Some(id)
        || request.get_attr("type").and_then(|v| v.as_text()) != Some("set")
        || request.get_attr("xmlns").and_then(|v| v.as_text()) != Some("w:sync:app:state")
    {
        return Err(invalid_patch(
            "original action is not the prepared app-state write",
        ));
    }
    one_collection(request, collection)
}

// Legacy preparation omitted logical-index metadata. Only authenticated current
// or rederived state may supply it; every frozen integrity field must agree.
fn matches_frozen_state(
    actual: &crate::signal::app_state_collection::SavedCollectionState,
    frozen: &crate::signal::app_state_collection::SavedCollectionState,
) -> Result<bool> {
    use crate::signal::app_state_collection::CollectionState;
    let mut actual = CollectionState::from_saved(actual)?.to_saved();
    let frozen = CollectionState::from_saved(frozen)?.to_saved();
    if frozen.logical_indexes.is_none() {
        actual.logical_indexes = None;
    }
    Ok(actual == frozen)
}

fn verify_prepared_in(
    tx: &crate::store::WriteTx<'_>,
    base: &crate::signal::app_state_collection::SavedCollectionState,
    prepared: &crate::signal::app_state_collection::SavedCollectionState,
    request: &Node,
) -> Result<crate::signal::app_state_collection::SavedCollectionState> {
    use crate::signal::{
        app_state,
        app_state_collection::{CollectionPatch, CollectionState},
    };
    if base.collection != prepared.collection
        || base.version.checked_add(1) != Some(prepared.version)
    {
        return Err(invalid_patch("prepared patch has no sequential baseline"));
    }
    let held_version = request.get_attr("version").and_then(|v| v.as_text());
    if match held_version {
        Some(v) => v.parse::<u64>().ok() != Some(base.version),
        None => base.version != 0,
    } {
        return Err(invalid_patch(
            "original request version differs from its baseline",
        ));
    }
    let mut patches = request
        .child_nodes()
        .iter()
        .filter(|n| n.description == "patch");
    let patch = patches
        .next()
        .ok_or_else(|| invalid_patch("original request has no patch"))?;
    if patches.next().is_some() {
        return Err(invalid_patch("original request has multiple patches"));
    }
    let crate::binary::NodeContent::Bytes(bytes) = &patch.content else {
        return Err(invalid_patch("original patch has no binary payload"));
    };
    // An old pending row held only the result. Reverify its immutable original
    // request against the actual baseline instead of guessing or resetting it.
    let patch = CollectionPatch::decode_prepared(bytes, prepared.version)?;
    let mut keys = std::collections::BTreeMap::new();
    for id in std::iter::once(&patch.key_id).chain(patch.mutations.iter().map(|m| &m.key_id)) {
        if !keys.contains_key(id) {
            keys.insert(
                *id,
                app_state::keys_for(tx, id)?
                    .ok_or_else(|| invalid_patch("prepared patch key is not held"))?,
            );
        }
    }
    let verified = CollectionState::from_saved(base)?.verify_patch(&patch, |id| {
        keys.get(id)
            .ok_or_else(|| invalid_patch("prepared patch references an unheld key"))
    })?;
    let verified = verified.state.to_saved();
    if !matches_frozen_state(&verified, prepared)? {
        return Err(invalid_patch(
            "prepared state does not match its authenticated original patch",
        ));
    }
    Ok(verified)
}

/// Recheck a queued patch in the same transaction as its socket reservation.
/// Orphaned legacy actions cannot leave without their frozen/reverified state.
pub(crate) fn validate_patch_reservation_in(
    tx: &mut crate::store::WriteTx<'_>,
    id: &str,
) -> Result<bool> {
    use crate::store::Table;
    if active_backoff_in(tx)?.is_some() {
        return Ok(false);
    }
    let pending: PendingState = tx
        .get_json(Table::Checkpoints, pending_key(id).as_bytes(), 1)?
        .ok_or_else(|| invalid_patch("patch action has no prepared state; no request reserved"))?;
    let prepared = pending.prepared();
    let collection = Collection::from_name(&prepared.collection)
        .ok_or_else(|| invalid_patch("prepared state names no known collection"))?;
    crate::signal::app_state_collection::CollectionState::from_saved(prepared)?;
    let current = held_state_in(tx, collection)?.to_saved();
    let request = super::actions::request_in_write(tx, id)?
        .ok_or_else(|| invalid_patch("missing original patch action"))?;
    let target = original_patch(&request, id, collection)?;
    let base = match &pending {
        PendingState::Bound { base, .. } => base,
        PendingState::Legacy(_) => &current,
    };
    if !matches_frozen_state(&current, base)? {
        return Err(invalid_patch(
            "held collection changed before patch reservation",
        ));
    }
    let verified = verify_prepared_in(tx, &current, prepared, target)?;
    if matches!(pending, PendingState::Legacy(_)) || *base != current || *prepared != verified {
        tx.put_json(
            Table::Checkpoints,
            pending_key(id).as_bytes(),
            1,
            &PendingState::Bound {
                base: current,
                prepared: verified,
            },
        )?;
    }
    Ok(true)
}

/// Recover explicit fetch waits and settle supported patch replies against
/// their frozen baselines in per-action transactions.
/// Malformed or stale replies preserve prepared state for inspection; no replay.
pub fn ingest(store: &crate::store::ProtocolStore) -> Result<usize> {
    ingest_fetch_backoff(store)?;
    let mut applied = 0;
    for (outcome, _) in super::actions::settled(store, PATCH_ACTION_KIND)? {
        applied += ingest_action(store, &outcome.id)?;
    }
    Ok(applied)
}

fn ingest_fetch_backoff(store: &crate::store::ProtocolStore) -> Result<()> {
    for kind in [FETCH_ACTION_KIND, HELD_FETCH_ACTION_KIND] {
        for (outcome, _) in super::actions::settled(store, kind)? {
            ingest_action(store, &outcome.id)?;
        }
    }
    Ok(())
}

/// Recover explicit fetch waits after settlement/application cuts. Unsupported
/// server replies remain inspectable; only durable store faults stop receiving.
pub(crate) fn recover_fetch_backoff(store: &crate::store::ProtocolStore) -> Result<()> {
    for kind in [FETCH_ACTION_KIND, HELD_FETCH_ACTION_KIND] {
        for (outcome, _) in super::actions::settled(store, kind)? {
            match ingest_action(store, &outcome.id) {
                Ok(_) => {}
                Err(crate::Error::InvalidInput { .. }) => {
                    tracing::warn!(
                        "app-state fetch refusal retained without adopting unsupported wait"
                    );
                }
                Err(error) => return Err(error),
            }
        }
    }
    Ok(())
}

fn require_original_fetch(request: &Node, id: &str) -> Result<()> {
    if request.description != "iq"
        || request.get_attr("id").and_then(|v| v.as_text()) != Some(id)
        || request.get_attr("type").and_then(|v| v.as_text()) != Some("set")
        || request.get_attr("xmlns").and_then(|v| v.as_text()) != Some("w:sync:app:state")
        || super::receipts::jid(request, "to") != Some(Jid::parse("s.whatsapp.net")?)
    {
        return Err(invalid_patch(
            "original action is not a supported app-state fetch",
        ));
    }
    let syncs: Vec<_> = request
        .child_nodes()
        .iter()
        .filter(|n| n.description == "sync")
        .collect();
    if syncs.len() != 1
        || request.child_nodes().len() != 1
        || syncs[0].child_nodes().is_empty()
        || syncs[0]
            .get_attr("data_namespace")
            .and_then(|v| v.as_text())
            != Some("3")
    {
        return Err(invalid_patch(
            "original fetch has no unique collection envelope",
        ));
    }
    let mut names = std::collections::BTreeSet::new();
    for collection in syncs[0].child_nodes() {
        let name = collection.get_attr("name").and_then(|v| v.as_text());
        if collection.description != "collection"
            || !matches!(collection.content, crate::binary::NodeContent::Empty)
            || name.and_then(Collection::from_name).is_none()
            || !names.insert(name)
        {
            return Err(invalid_patch(
                "original fetch has an unsupported collection shape",
            ));
        }
    }
    Ok(())
}

pub(crate) fn ingest_action(store: &crate::store::ProtocolStore, id: &str) -> Result<usize> {
    use crate::{signal::app_state_collection::CollectionState, store::Table};
    store.write(|tx| {
            let view = super::actions::view_in_write(tx, id)?;
            if let Some((actual, reply, request)) = &view
                && matches!(actual.kind.as_str(), FETCH_ACTION_KIND | HELD_FETCH_ACTION_KIND)
                && actual.status == super::actions::ActionStatus::Error
            {
                let reply = reply.as_ref().ok_or_else(|| invalid_patch("settled fetch has no preserved reply"))?;
                if reply.get_attr("type").and_then(|v| v.as_text()) == Some("error") {
                    require_original_fetch(request, id)?;
                    if reply.description != "iq" || reply.get_attr("id").and_then(|v| v.as_text()) != Some(id) {
                        return Err(invalid_patch("fetch refusal does not match original action"));
                    }
                    retain_backoff_in(tx, id, definite_error(reply)?)?;
                }
                return Ok(0);
            }
            let pending_key = pending_key(id);
            let pending: Option<PendingState> = tx.get_json(Table::Checkpoints, pending_key.as_bytes(), 1)?;
            let Some(pending) = pending else { return Ok(0); };
            let Some((actual, reply, request)) = view else {
                return Err(invalid_patch("prepared patch has no original action"));
            };
            if actual.kind != PATCH_ACTION_KIND || !matches!(actual.status, super::actions::ActionStatus::Ok | super::actions::ActionStatus::Error) {
                return Err(invalid_patch("patch action is not settled"));
            }
            let prepared = pending.prepared();
            if prepared.version > i64::MAX as u64 { return Err(invalid_patch("prepared version exceeds the APK signed-long range")); }
            let collection = Collection::from_name(&prepared.collection).ok_or_else(|| invalid_patch("stashed state names no known collection"))?;
            CollectionState::from_saved(prepared)?;
            let target = original_patch(&request, &actual.id, collection)?;
            let reply = reply.ok_or_else(|| invalid_patch("settled patch has no preserved reply"))?;
            if reply.description != "iq" || reply.get_attr("id").and_then(|v| v.as_text()) != Some(actual.id.as_str()) {
                return Err(invalid_patch("patch reply does not match original action"));
            }
            let kind = reply.get_attr("type").and_then(|v| v.as_text());
            if kind == Some("error") {
                // A definite outer refusal cannot promote any prepared state.
                let error = definite_error(&reply)?;
                retain_backoff_in(tx, &actual.id, error)?;
                tx.delete(Table::Checkpoints, pending_key.as_bytes())?;
                return Ok(0);
            }
            if kind != Some("result") { return Err(invalid_patch("patch response is not result or error")); }
            let response = one_collection(&reply, collection)?;
            if response.get_attr("type").and_then(|v| v.as_text()) == Some("error") || response.child("error").is_some() {
                if response.get_attr("type").and_then(|v| v.as_text()) != Some("error") {
                    return Err(invalid_patch("unsupported collection error shape"));
                }
                definite_error(response)?;
                // Conflict catch-up and pending release commit/roll back together.
                let (request_wire, reply_wire) = super::actions::wire_pair_in_write(tx, &actual.id)?
                    .ok_or_else(|| invalid_patch("conflict has no preserved wire pair"))?;
                use sha2::{Digest, Sha256};
                let source = CatchUpSource {
                    action_id: &actual.id,
                    request_digest: Sha256::digest(&request_wire).into(),
                    reply_digest: Sha256::digest(&reply_wire).into(),
                };
                let changed = catch_up_in(tx, &reply, &source)?;
                tx.delete(Table::Checkpoints, pending_key.as_bytes())?;
                return Ok(changed);
            }
            // APK 43.72 F2T version acknowledgements use signed Java decimal
            // parsing (app-state-fetch-completion.md). Accept its spelling only;
            // the value still must equal the authenticated prepared version.
            if actual.status != super::actions::ActionStatus::Ok
                || response.get_attr("type").is_some_and(|v| v.as_text() != Some("result"))
                || response.get_attr("version").and_then(|v| v.as_text()).and_then(super::mex::java_decimal_long).and_then(|v| u64::try_from(v).ok()) != Some(prepared.version)
            { return Err(invalid_patch("accepted patch version/shape does not match prepared state; inspect original action")); }
            let current = held_state_in(tx, collection)?.to_saved();
            if matches_frozen_state(&current, prepared)? {
                tx.delete(Table::Checkpoints, pending_key.as_bytes())?;
                return Ok(0);
            }
            let base = match &pending { PendingState::Bound { base, .. } => base, PendingState::Legacy(_) => &current };
            if !matches_frozen_state(&current, base)? { return Err(invalid_patch("held collection changed before accepted patch promotion")); }
            let verified = verify_prepared_in(tx, &current, prepared, target)?;
            tx.put_json(Table::Checkpoints, held_key(collection).as_bytes(), 1, &verified)?;
            tx.delete(Table::Checkpoints, pending_key.as_bytes())?;
            Ok(1)
    })
}

/// Apply the `<patches>` a `409 conflict` returns, advancing our held state to
/// what the server already has.
///
/// **Observed** 2026-09-24: `<collection type="error" name=…><error code="409"
/// text="conflict"/><patches><patch>…</patch></patches></collection>`. Each is a
/// `SyncdPatch` carrying its own `version` — server-sent patches do, outgoing
/// ones must not — so it goes through exactly the verification an inline patch
/// from a fetch does. Nothing is applied that does not verify: a conflict is not
/// permission to trust bytes, and the key a patch names must be one this account
/// already holds.
struct CatchUpSource<'a> {
    action_id: &'a str,
    request_digest: [u8; 32],
    reply_digest: [u8; 32],
}

fn catch_up_in(
    tx: &mut crate::store::WriteTx<'_>,
    reply: &Node,
    source: &CatchUpSource<'_>,
) -> Result<usize> {
    use crate::binary::NodeContent;
    use crate::signal::{app_state, app_state_collection::CollectionPatch};
    use crate::store::Table;
    let Some(conflicted) = reply
        .child("sync")
        .and_then(|sync| sync.child("collection"))
        .filter(|c| c.get_attr("type").and_then(|v| v.as_text()) == Some("error"))
    else {
        return Ok(0);
    };
    let mut patch_lists = conflicted
        .child_nodes()
        .iter()
        .filter(|node| node.description == "patches");
    let Some(patches) = patch_lists.next() else {
        return Ok(0);
    };
    if patch_lists.next().is_some() {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "ambiguous conflict patch lists",
        ));
    }
    let mut errors = conflicted
        .child_nodes()
        .iter()
        .filter(|node| node.description == "error");
    if errors
        .next()
        .and_then(|error| error.get_attr("code"))
        .and_then(|code| code.as_text())
        .and_then(super::mex::java_decimal_long)
        != Some(409)
        || errors.next().is_some()
    {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "patch catch-up requires an explicit conflict",
        ));
    }
    let collection = conflicted
        .get_attr("name")
        .and_then(|v| v.as_text())
        .and_then(Collection::from_name)
        .ok_or_else(|| {
            crate::Error::invalid("app_state_sync", "conflict names no known collection")
        })?;
    // APK 43.72 F2T.A00 preflights the whole selected collection before
    // FeN/SyncResponseHandler application. Reuse that structural stage here;
    // an accompanying external snapshot must not be silently discarded.
    let classified = response::classify_response(reply)?;
    if classified.collections.len() != 1 {
        return Err(invalid_patch(
            "conflict requires one unambiguous collection",
        ));
    }
    let response::Category::Error {
        code: 409,
        conflict_data: Some(data),
        ..
    } = &classified.collections[0].category
    else {
        return Err(invalid_patch("conflict has no supported patch data"));
    };
    let headers = headers::inspect_collection_headers(data)
        .map_err(|_| invalid_patch("conflict fails structural header preflight"))?;
    if headers.snapshot_ref.is_some()
        || headers
            .patches
            .iter()
            .any(|patch| patch.source != headers::PatchSource::Inline)
    {
        return Err(invalid_patch(
            "conflict requires unsupported authenticated external retrieval",
        ));
    }
    // APK 43.72 SyncResponseHandler.A07 -> 0mj.A00 commits verified state
    // inside its application transaction (server-sync-lifecycle.md). Read the
    // baseline and every key in this same transaction: a separately loaded
    // baseline could otherwise overwrite a concurrent accepted patch.
    let mut state = held_state_in(tx, collection)?;
    let mut applied = 0;
    for node in patches.child_nodes() {
        if node.description != "patch" {
            continue;
        }
        let NodeContent::Bytes(bytes) = &node.content else {
            return Err(crate::Error::invalid(
                "app_state_sync",
                "conflict patch has no byte payload",
            ));
        };
        let decoded = CollectionPatch::decode(bytes)?;
        // APK 43.72 SyncResponseHandler.A07 collects record IDs separately from
        // its patch/snapshot ID before 0fI.A08 resolves the combined set. See
        // app-state-catchup-atomicity.md. Resolve only held keys in this tx.
        let mut keys = std::collections::BTreeMap::new();
        for id in std::iter::once(&decoded.key_id)
            .chain(decoded.mutations.iter().map(|mutation| &mutation.key_id))
        {
            if !keys.contains_key(id) {
                keys.insert(
                    *id,
                    app_state::keys_for(tx, id)?
                        .ok_or_else(|| invalid_patch("conflict names a key we do not hold"))?,
                );
            }
        }
        let verified = state.verify_patch(&decoded, |id| {
            keys.get(id)
                .ok_or_else(|| invalid_patch("conflict references an unheld key"))
        })?;
        // Retain authenticated action plaintext before committing its integrity
        // checkpoint. This closes data loss in the existing whole-chain 409
        // utility; it does not claim native per-patch commit/projection parity.
        action_journal::retain_authenticated_mutations_in(
            tx,
            &action_journal::JournalContext {
                action_id: source.action_id,
                collection,
                patch_version: decoded.version,
                generation: tx.generation(),
                request_digest: source.request_digest,
                reply_digest: source.reply_digest,
            },
            &verified.mutations,
        )?;
        state = verified.state;
        applied += 1;
    }
    if applied > 0 {
        tx.put_json(
            Table::Checkpoints,
            held_key(collection).as_bytes(),
            1,
            &state.to_saved(),
        )?;
    }
    Ok(applied)
}

#[cfg(test)]
mod catch_up_atomic_tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        signal::app_state_collection::{CollectionPatch, CollectionState, SavedCollectionState},
        store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
    };

    fn test_source() -> CatchUpSource<'static> {
        CatchUpSource {
            action_id: "synthetic-conflict-test",
            request_digest: [1; 32],
            reply_digest: [2; 32],
        }
    }

    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("catchup-test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap()
    }

    fn held(store: &ProtocolStore) -> Option<SavedCollectionState> {
        store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    held_key(Collection::Regular).as_bytes(),
                    1,
                )
            })
            .unwrap()
    }

    fn journal(store: &ProtocolStore) -> Vec<serde_json::Value> {
        store
            .read(|tx| {
                tx.scan(Table::Checkpoints, 1, 1000)?
                    .into_iter()
                    .filter(|(key, _)| key.starts_with(b"app-state-fetch-action-journal:v1:"))
                    .map(|(_, bytes)| {
                        serde_json::from_slice(&bytes)
                            .map_err(|_| invalid_patch("invalid test journal"))
                    })
                    .collect()
            })
            .unwrap()
    }

    fn install_key(store: &ProtocolStore) {
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"app-state-key:000100000002",
                    1,
                    &serde_json::json!({"data":(0u8..32).collect::<Vec<_>>(),
                "fingerprint":{"raw_id":7,"current_index":1,"device_indexes":[1]},
                "timestamp":1}),
                )
            })
            .unwrap();
    }

    fn patch(remove: bool) -> Vec<u8> {
        // Independent Python/OpenSSL SET/REMOVE vectors already exercised in
        // signal::app_state_collection's literal-wire tests; handwritten tags.
        let (body, snapshot, mac) = if remove {
            (
                "000102030405060708090a0b0c0d0e0f2192b6a9dde3b0f3712d6c99dc98976b3a3b4d03d17330a34a3211156d7360a31dc6482aacb38b0e0be76016dc6f7b7673e949ff5098ce118e05a12e44b8fa67",
                "372fa102e3b780fd0fac7ca9f521ebf00480b92e1e812fc867a6b751fde6f168",
                "81f911830690a8c2f52881a8eae26e8de2875da64ffece1cab25db631fff648f",
            )
        } else {
            (
                "000102030405060708090a0b0c0d0e0f2192b6a9dde3b0f3712d6c99dc98976b69e1b9f25155fa8ca9e9f0e206925fcc5f5fd8469f020699365e4750632c893bc24096315f6739aaeba497a176823909",
                "0e326b26aa4969d98e2fcf482d2790bf9933a9e11f093b7c18a17f0da491775d",
                "0960e911ee5e9d0e27b9e9e10b665dee186429aae47f0ed7bd0ade5c00081e8e",
            )
        };
        let id = [0, 1, 0, 0, 0, 2];
        let mut bytes = vec![
            0x0a,
            2,
            8,
            if remove { 2 } else { 1 },
            0x12,
            0x87,
            1,
            8,
            u8::from(remove),
            0x12,
            0x82,
            1,
            0x0a,
            0x22,
            0x0a,
            0x20,
        ];
        bytes.extend(
            hex::decode("3d47311df4bc7aa8bd5ef2c16cbe7fc1ea07f18c86a2fb34078d1d416edb22d8")
                .unwrap(),
        );
        bytes.extend([0x12, 0x52, 0x0a, 0x50]);
        bytes.extend(hex::decode(body).unwrap());
        bytes.extend([0x1a, 8, 0x0a, 6]);
        bytes.extend(id);
        bytes.extend([0x22, 32]);
        bytes.extend(hex::decode(snapshot).unwrap());
        bytes.extend([0x2a, 32]);
        bytes.extend(hex::decode(mac).unwrap());
        bytes.extend([0x32, 8, 0x0a, 6]);
        bytes.extend(id);
        bytes
    }

    fn reply(patches: Vec<Node>) -> Node {
        Node::new("iq")
            .attr("type", "result")
            .children(vec![Node::new("sync").children(vec![
            Node::new("collection").attr("name", "regular").attr("type", "error").children(vec![
                Node::new("error").attr("code", "409"),
                Node::new("patches").children(patches),
            ])
        ])])
    }

    #[test]
    fn saved_response_inspection_redacts_payloads_and_never_changes_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("response-inspection.redb"));
        let id = queue_fetch(&store).unwrap();
        assert!(inspect_response(&store, &id).is_err());
        super::super::actions::reserve_next(&store)
            .unwrap()
            .unwrap();
        let response = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("sync").children(vec![
                    Node::new("collection")
                        .attr("name", "regular")
                        .attr("version", "3")
                        .children(vec![
                            Node::new("snapshot").bytes(b"private-ignored-snapshot".to_vec()),
                        ]),
                    Node::new("collection")
                        .attr("name", "private-unknown-name")
                        .attr("type", "error")
                        .children(vec![Node::new("error").attr("text", "private-error-text")]),
                    Node::new("collection")
                        .attr("name", "regular_high")
                        .attr("has_more_patches", "true")
                        .children(vec![
                            Node::new("snapshot").bytes(b"private-external-reference".to_vec()),
                            Node::new("patches").children(vec![
                                Node::new("patch").bytes(b"private-patch".to_vec()),
                                Node::new("future").bytes(b"private-opaque".to_vec()),
                            ]),
                        ]),
                    Node::new("collection").attr("name", "regular"),
                ])]);
        super::super::actions::record_reply(&store, &response).unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
            .unwrap();
        let head = store.journal_head().unwrap();
        let report = inspect_response(&store, &id).unwrap();
        assert!(!report.integrity_verified);
        assert_eq!(
            report
                .collections
                .iter()
                .map(|c| c.category)
                .collect::<Vec<_>>(),
            vec!["version_ack", "error", "data", "already_synced"]
        );
        assert_eq!(report.collections[0].version_ack, Some(3));
        assert!(!report.collections[0].snapshot_ref_present);
        assert_eq!(
            report.collections[0].header_status, None,
            "ignored acknowledgement payload is not parsed"
        );
        assert_eq!(report.collections[1].known_collection, None);
        assert_eq!(report.collections[1].error_code, Some(-1));
        assert!(report.collections[1].has_error_text);
        assert_eq!(report.collections[2].patch_count, 1);
        assert!(report.collections[2].snapshot_ref_present);
        assert_eq!(report.collections[2].has_more_patches, Some(true));
        assert!(matches!(
            report.collections[2].header_status,
            Some("native_fatal_header" | "local_header_refusal")
        ));
        assert!(report.collections[2].header_patch_versions.is_empty());
        assert!(!serde_json::to_string(&report).unwrap().contains("private"));
        assert!(!format!("{report:?}").contains("private"));
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                .unwrap(),
            before
        );
        assert_eq!(store.journal_head().unwrap(), head);
    }

    #[test]
    fn inspection_reports_legacy_provenance_without_changing_checkpoints() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("inspect.redb"));
        let empty = inspect_collections(&store).unwrap();
        assert_eq!(empty.len(), ALL_COLLECTIONS.len());
        assert!(empty.iter().all(|s| s.held_version.is_none()
            && s.mutation_count == 0
            && s.logical_indexes_complete));
        install_key(&store);
        let response = reply(vec![Node::new("patch").bytes(patch(false))]);
        store
            .write(|tx| catch_up_in(tx, &response, &test_source()))
            .unwrap();
        let complete = inspect_collections(&store).unwrap();
        assert!(
            complete
                .iter()
                .find(|s| s.collection == "regular")
                .unwrap()
                .logical_indexes_complete
        );
        store
            .write(|tx| {
                let mut saved: serde_json::Value = tx
                    .get_json(
                        Table::Checkpoints,
                        held_key(Collection::Regular).as_bytes(),
                        1,
                    )?
                    .unwrap();
                saved.as_object_mut().unwrap().remove("logical_indexes");
                tx.put_json(
                    Table::Checkpoints,
                    held_key(Collection::Regular).as_bytes(),
                    1,
                    &saved,
                )
            })
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let legacy = inspect_collections(&store).unwrap();
        let regular = legacy.iter().find(|s| s.collection == "regular").unwrap();
        assert_eq!(regular.held_version, Some(1));
        assert_eq!(regular.mutation_count, 1);
        assert!(!regular.logical_indexes_complete);
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        let patch = CollectionPatch::decode(&patch(false)).unwrap();
        let record = &patch.mutations[0];
        // Same independent literal record in the snapshot protobuf envelope.
        let mut snapshot = vec![0x0a, 2, 8, 1, 0x12, 0x82, 1, 0x0a, 0x22, 0x0a, 0x20];
        snapshot.extend(record.index_mac);
        snapshot.extend([0x12, 0x52, 0x0a, 0x50]);
        snapshot.extend(&record.body);
        snapshot.extend([0x1a, 8, 0x0a, 6]);
        snapshot.extend(record.key_id);
        snapshot.extend([0x1a, 32]);
        snapshot.extend(patch.snapshot_mac);
        snapshot.extend([0x22, 8, 0x0a, 6]);
        snapshot.extend(patch.key_id);
        let mut corrupt = snapshot.clone();
        corrupt[20] ^= 1;
        assert!(reconstruct_logical_indexes(&store, Collection::Regular, &corrupt).is_err());
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        let original = held(&store).unwrap();
        let mut future = snapshot.clone();
        future[3] = 2;
        assert!(reconstruct_logical_indexes(&store, Collection::Regular, &future).is_err());
        let mut unheld = snapshot.clone();
        *unheld.last_mut().unwrap() ^= 1;
        assert!(reconstruct_logical_indexes(&store, Collection::Regular, &unheld).is_err());
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    pending_key("inspect-pending").as_bytes(),
                    1,
                    &PendingState::Legacy(original.clone()),
                )
            })
            .unwrap();
        assert!(reconstruct_logical_indexes(&store, Collection::Regular, &snapshot).is_err());
        assert_eq!(held(&store).unwrap(), original);
        store
            .write(|tx| {
                tx.delete(
                    Table::Checkpoints,
                    pending_key("inspect-pending").as_bytes(),
                )
            })
            .unwrap();
        assert!(reconstruct_logical_indexes(&store, Collection::Regular, &snapshot).unwrap());
        let restored = held(&store).unwrap();
        assert_eq!(restored.version, original.version);
        assert_eq!(restored.hash, original.hash);
        assert_eq!(restored.values, original.values);
        assert!(
            inspect_collections(&store)
                .unwrap()
                .iter()
                .all(|s| s.logical_indexes_complete)
        );
        let after = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        assert!(!reconstruct_logical_indexes(&store, Collection::Regular, &snapshot).unwrap());
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            after
        );
    }

    #[test]
    fn conflict_resolves_record_key_separately_from_patch_key() {
        use crate::signal::app_state_crypto::MutationKeys;
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("mixed-keys.redb"));
        install_key(&store);
        let mut bytes = patch(false);
        let decoded = CollectionPatch::decode(&bytes).unwrap();
        let original_keys = MutationKeys::derive(&(0u8..32).collect::<Vec<_>>()).unwrap();
        let expected = CollectionState::empty("regular")
            .unwrap()
            .verify_patch(&decoded, |_| Ok(&original_keys))
            .unwrap()
            .state;
        // Keep the independent literal record untouched. Only its enclosing
        // patch/snapshot MACs are signed by a different, synthetic held key.
        let signing_keys = MutationKeys::derive(&[42; 32]).unwrap();
        let snapshot = signing_keys
            .snapshot_mac(expected.hash().as_bytes(), 1, "regular")
            .unwrap();
        let value_mac = decoded.mutations[0].body[decoded.mutations[0].body.len() - 32..]
            .try_into()
            .unwrap();
        let mac = signing_keys
            .patch_mac(&snapshot, &[value_mac], 1, "regular")
            .unwrap();
        bytes.truncate(bytes.len() - 78);
        bytes.extend([0x22, 32]);
        bytes.extend(snapshot);
        bytes.extend([0x2a, 32]);
        bytes.extend(mac);
        bytes.extend([0x32, 8, 0x0a, 6]);
        bytes.extend([0, 1, 0, 0, 0, 3]);
        let response = reply(vec![Node::new("patch").bytes(bytes)]);
        assert!(
            store
                .write(|tx| catch_up_in(tx, &response, &test_source()))
                .is_err()
        );
        assert!(
            held(&store).is_none(),
            "missing signing key cannot advance state"
        );
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    b"app-state-key:000100000003",
                    1,
                    &serde_json::json!({"data":vec![42u8;32],
                "fingerprint":{"raw_id":7,"current_index":1,"device_indexes":[1]},
                "timestamp":1}),
                )
            })
            .unwrap();
        assert_eq!(
            store
                .write(|tx| catch_up_in(tx, &response, &test_source()))
                .unwrap(),
            1
        );
        assert_eq!(held(&store).unwrap(), expected.to_saved());
    }

    #[test]
    fn settled_conflict_journals_exact_wire_context_and_releases_pending_atomically() {
        use sha2::{Digest, Sha256};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settled-conflict.redb");
        let store = open(&path);
        install_key(&store);
        let base = CollectionState::empty("regular").unwrap();
        let keys =
            crate::signal::app_state_crypto::MutationKeys::derive(&(0u8..32).collect::<Vec<_>>())
                .unwrap();
        let prepared = base
            .verify_patch(&CollectionPatch::decode(&patch(false)).unwrap(), |_| {
                Ok(&keys)
            })
            .unwrap()
            .state
            .to_saved();
        let id = store
            .write(|tx| {
                let id = super::super::actions::queue_in(
                    tx,
                    PATCH_ACTION_KIND,
                    "synthetic conflict",
                    |id| patch_request(id, Collection::Regular, Some(0), &patch(false)[4..], false),
                )?;
                tx.put_json(
                    Table::Checkpoints,
                    pending_key(&id).as_bytes(),
                    1,
                    &PendingState::Bound {
                        base: base.to_saved(),
                        prepared: prepared.clone(),
                    },
                )?;
                Ok(id)
            })
            .unwrap();
        super::super::actions::reserve_next(&store)
            .unwrap()
            .unwrap();
        let mut response = reply(vec![Node::new("patch").bytes(patch(false))]);
        response = response.attr("id", id.as_str());
        assert!(super::super::actions::record_reply(&store, &response).unwrap());
        let (request_wire, reply_wire) = store
            .write(|tx| super::super::actions::wire_pair_in_write(tx, &id))
            .unwrap()
            .unwrap();
        assert_eq!(ingest_action(&store, &id).unwrap(), 1);
        let retained = journal(&store);
        assert_eq!(retained.len(), 1);
        assert_eq!(retained[0]["action_id"], id);
        assert_eq!(
            retained[0]["request_digest"],
            serde_json::json!(Sha256::digest(&request_wire).to_vec())
        );
        assert_eq!(
            retained[0]["reply_digest"],
            serde_json::json!(Sha256::digest(&reply_wire).to_vec())
        );
        assert_eq!(held(&store).unwrap(), prepared);
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, pending_key(&id).as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        drop(store);
        let store = open(&path);
        assert_eq!(ingest_action(&store, &id).unwrap(), 0);
        assert_eq!(journal(&store), retained);
        assert_eq!(held(&store).unwrap(), prepared);
    }

    #[test]
    fn conflict_never_discards_snapshot_or_later_header_failure() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("preflight.redb"));
        install_key(&store);
        // Even a syntactically valid empty external-reference message is not
        // permission to omit the snapshot and apply just the inline portion.
        let snapshot =
            Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("sync").children(
                    vec![Node::new("collection")
                .attr("name", "regular").attr("type", "error").children(vec![
                    Node::new("error").attr("code", "409"),
                    Node::new("snapshot").bytes(vec![0x0a, 0]),
                    Node::new("patches").children(vec![Node::new("patch").bytes(patch(false))]),
                ])],
                )]);
        let duplicate = reply(vec![
            Node::new("patch").bytes(patch(false)),
            Node::new("patch").bytes(patch(false)),
        ]);
        let mut external = patch(true);
        external.extend([0x1a, 0]);
        let external = reply(vec![
            Node::new("patch").bytes(patch(false)),
            Node::new("patch").bytes(external),
        ]);
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
            .unwrap();
        for response in [snapshot, duplicate, external] {
            assert!(
                store
                    .write(|tx| catch_up_in(tx, &response, &test_source()))
                    .is_err()
            );
            assert_eq!(
                store
                    .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
                    .unwrap(),
                before
            );
            assert!(journal(&store).is_empty());
        }
    }

    #[test]
    fn conflict_code_uses_java_integer_spelling() {
        for code in ["+409", "٠٤٠٩", "４０９"] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("java-code.redb"));
            install_key(&store);
            let response =
                Node::new("iq")
                    .attr("type", "result")
                    .children(vec![Node::new("sync").children(
                        vec![Node::new("collection")
                    .attr("name", "regular").attr("type", "error").children(vec![
                        Node::new("error").attr("code", code),
                        Node::new("patches").children(vec![Node::new("patch").bytes(patch(false))]),
                    ])],
                    )]);
            assert_eq!(
                store
                    .write(|tx| catch_up_in(tx, &response, &test_source()))
                    .unwrap(),
                1
            );
            assert_eq!(held(&store).unwrap().version, 1);
            assert_eq!(journal(&store).len(), 1);
        }
    }

    #[test]
    fn conflict_chain_commits_together_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chain.redb");
        let store = open(&path);
        install_key(&store);
        let response = reply(vec![
            Node::new("patch").bytes(patch(false)),
            Node::new("patch").bytes(patch(true)),
        ]);
        assert_eq!(
            store
                .write(|tx| catch_up_in(tx, &response, &test_source()))
                .unwrap(),
            2
        );
        let expected = held(&store).unwrap();
        assert_eq!(expected.version, 2);
        assert!(expected.values.is_empty());
        assert_eq!(expected.hash, vec![0; 128]);
        let retained = journal(&store);
        assert_eq!(retained.len(), 2);
        for row in &retained {
            assert_eq!(row["action_id"], "synthetic-conflict-test");
            assert_eq!(row["request_digest"], serde_json::json!([1; 32].to_vec()));
            assert_eq!(row["reply_digest"], serde_json::json!([2; 32].to_vec()));
            assert_eq!(row["actions"].as_array().unwrap().len(), 1);
        }
        assert!(
            retained
                .iter()
                .any(|row| row["actions"][0]["operation"] == 1)
        );
        assert!(
            retained
                .iter()
                .any(|row| row["actions"][0]["operation"] == 2)
        );
        drop(store);
        let store = open(&path);
        assert_eq!(held(&store).unwrap(), expected);
        assert_eq!(journal(&store), retained);
        let before = held(&store);
        assert!(
            store
                .write(|tx| catch_up_in(tx, &response, &test_source()))
                .is_err()
        );
        assert_eq!(
            held(&store),
            before,
            "old conflict cannot regress verified state"
        );
    }

    #[test]
    fn bad_later_patch_rolls_back_caller_writes_and_never_adopts_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollback.redb");
        let store = open(&path);
        install_key(&store);
        let mut wrong_key = patch(true);
        *wrong_key.last_mut().unwrap() ^= 1;
        let mut bad_mac = patch(true);
        let mac_tail = bad_mac.len() - 12;
        bad_mac[mac_tail] ^= 1;
        for late in [
            Node::new("patch").bytes(wrong_key),
            Node::new("patch").bytes(bad_mac),
            Node::new("patch").bytes(vec![]),
            Node::new("patch"),
        ] {
            let response = reply(vec![Node::new("patch").bytes(patch(false)), late]);
            assert!(
                store
                    .write(|tx| {
                        tx.put_json(Table::Checkpoints, b"catchup-parent-write", 1, &true)?;
                        catch_up_in(tx, &response, &test_source())
                    })
                    .is_err()
            );
            assert!(held(&store).is_none());
            assert!(
                journal(&store).is_empty(),
                "failed chain must retain no partial actions"
            );
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, b"catchup-parent-write", 1))
                    .unwrap()
                    .is_none()
            );
            assert!(
                store
                    .write(|tx| crate::signal::app_state::keys_for(tx, &[0, 1, 0, 0, 0, 3]))
                    .unwrap()
                    .is_none()
            );
        }
        drop(store);
        let store = open(&path);
        assert!(held(&store).is_none());
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, b"catchup-parent-write", 1))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn conflict_reads_the_callers_transaction_baseline_and_requires_409() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("baseline.redb"));
        install_key(&store);
        let initial = CollectionState::empty("regular").unwrap();
        let set = CollectionPatch::decode(&patch(false)).unwrap();
        let staged = store
            .write(|tx| {
                let keys = crate::signal::app_state::keys_for(tx, &set.key_id)?.unwrap();
                Ok(initial.verify_patch(&set, |_| Ok(&keys))?.state.to_saved())
            })
            .unwrap();
        let response = reply(vec![Node::new("patch").bytes(patch(true))]);
        assert_eq!(
            store
                .write(|tx| {
                    // This state is not visible to an independent read transaction yet.
                    tx.put_json(
                        Table::Checkpoints,
                        held_key(Collection::Regular).as_bytes(),
                        1,
                        &staged,
                    )?;
                    catch_up_in(tx, &response, &test_source())
                })
                .unwrap(),
            1
        );
        assert_eq!(held(&store).unwrap().version, 2);
        let untrusted_route = Node::new("iq").children(vec![Node::new("sync").children(vec![
            Node::new("collection").attr("type", "error").attr("name", "regular").children(vec![
                Node::new("error").attr("code", "500"),
                Node::new("patches").children(vec![Node::new("patch").bytes(patch(false))]),
            ])
        ])]);
        let before = held(&store);
        assert!(
            store
                .write(|tx| catch_up_in(tx, &untrusted_route, &test_source()))
                .is_err()
        );
        assert_eq!(held(&store), before);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::NodeContent;

    /// A second patch has to continue the first. Round-tripping the state is
    /// what makes that possible, and a mis-sized hash or MAC must be refused
    /// rather than silently restarting from a wrong baseline.
    #[test]
    fn collection_state_round_trips_and_refuses_corruption() {
        use crate::signal::app_state_collection::{CollectionState, SavedCollectionState};
        let state = CollectionState::empty("critical_unblock_low").unwrap();
        let saved = state.to_saved();
        assert_eq!(saved.version, 0);
        assert_eq!(saved.hash.len(), 128);
        let back = CollectionState::from_saved(&saved).unwrap();
        assert_eq!(back, state);

        let short = SavedCollectionState {
            hash: vec![0; 127],
            ..saved.clone()
        };
        assert!(CollectionState::from_saved(&short).is_err());
        let bad_mac = SavedCollectionState {
            values: vec![(vec![0; 31], vec![0; 32])],
            ..saved
        };
        assert!(CollectionState::from_saved(&bad_mac).is_err());
    }

    /// The `version` attribute is the version we hold, so it is absent before
    /// the first accepted patch and present after — the distinction that three
    /// live refusals turned on.
    #[test]
    fn the_held_version_attribute_appears_only_once_we_hold_one() {
        let first = patch_request("A", Collection::CriticalUnblockLow, None, &[1], false).unwrap();
        let collection = &first.child_nodes()[0].child_nodes()[0];
        assert!(collection.get_attr("version").is_none());
        let second =
            patch_request("B", Collection::CriticalUnblockLow, Some(1), &[1], false).unwrap();
        let collection = &second.child_nodes()[0].child_nodes()[0];
        assert_eq!(collection.get_attr("version").unwrap().as_text(), Some("1"));
    }

    #[test]
    fn patch_request_matches_the_traced_shape() {
        let node = patch_request(
            "ABC123",
            Collection::RegularHigh,
            Some(4),
            &[1, 2, 3],
            false,
        )
        .unwrap();
        assert_eq!(node.description, "iq");
        assert_eq!(node.get_attr("type").unwrap().as_text(), Some("set"));
        assert_eq!(
            node.get_attr("xmlns").unwrap().as_text(),
            Some("w:sync:app:state")
        );
        assert_eq!(node.get_attr("id").unwrap().as_text(), Some("ABC123"));
        let sync = &node.child_nodes()[0];
        assert_eq!(sync.description, "sync");
        assert_eq!(
            sync.get_attr("data_namespace").unwrap().as_text(),
            Some("3")
        );
        let collection = &sync.child_nodes()[0];
        assert_eq!(collection.description, "collection");
        assert_eq!(
            collection.get_attr("name").unwrap().as_text(),
            Some("regular_high")
        );
        assert_eq!(collection.get_attr("version").unwrap().as_text(), Some("4"));
        // return_snapshot is omitted unless requested.
        assert!(collection.get_attr("return_snapshot").is_none());
        let patch = &collection.child_nodes()[0];
        assert_eq!(patch.description, "patch");
        assert!(matches!(&patch.content, NodeContent::Bytes(b) if b.as_ref() == [1, 2, 3]));

        // Requested with a nonzero version it is still omitted: CSe.A01 emits it
        // only when the version argument is null or zero, so a patch send never
        // carries it.
        let asked = patch_request("X", Collection::Regular, Some(1), &[9], true).unwrap();
        assert!(
            asked.child_nodes()[0].child_nodes()[0]
                .get_attr("return_snapshot")
                .is_none()
        );
        let at_zero = patch_request("X", Collection::Regular, Some(0), &[9], true).unwrap();
        assert_eq!(
            at_zero.child_nodes()[0].child_nodes()[0]
                .get_attr("return_snapshot")
                .unwrap()
                .as_text(),
            Some("true")
        );
    }

    #[test]
    /// `CSe.A01` computes order as `!isCriticalUnblockLow`, so the special case
    /// is the one that gets 0. An earlier note in chat-actions.md had this
    /// backwards; this test pins the corrected direction.
    fn only_critical_unblock_low_takes_order_zero() {
        assert_eq!(Collection::CriticalUnblockLow.order(), 0);
        for c in [
            Collection::Regular,
            Collection::RegularLow,
            Collection::RegularHigh,
            Collection::CriticalBlock,
        ] {
            assert_eq!(c.order(), 1, "{} should take order 1", c.name());
        }
    }

    /// A fetch is a read: every collection at version 0 asking for a snapshot,
    /// and crucially no `<patch>` child, since CSe.A01 adds that only when a
    /// patch argument is present.
    /// The version attribute is the version the client *holds*, so a collection
    /// it has never synced emits no `version` at all. Sending the patch's own
    /// claimed version instead is what the server refused with 400 bad-request.
    #[test]
    fn a_first_patch_omits_the_version_attribute_entirely() {
        let node =
            patch_request("P1", Collection::CriticalUnblockLow, None, &[1, 2], false).unwrap();
        let collection = &node.child_nodes()[0].child_nodes()[0];
        assert!(
            collection.get_attr("version").is_none(),
            "a never-synced collection sends no version"
        );
        // Name, order and the patch child are still present.
        assert_eq!(
            collection.get_attr("name").unwrap().as_text(),
            Some("critical_unblock_low")
        );
        assert_eq!(collection.get_attr("order").unwrap().as_text(), Some("0"));
        assert_eq!(collection.child_nodes()[0].description, "patch");
        // A held version is emitted verbatim, not incremented.
        let held = patch_request("P2", Collection::RegularHigh, Some(7), &[1], false).unwrap();
        assert_eq!(
            held.child_nodes()[0].child_nodes()[0]
                .get_attr("version")
                .unwrap()
                .as_text(),
            Some("7")
        );
    }

    #[test]
    fn fetch_request_asks_for_snapshots_and_carries_no_patch() {
        let node = fetch_request("F1", &ALL_COLLECTIONS).unwrap();
        assert_eq!(node.description, "iq");
        assert_eq!(node.get_attr("type").unwrap().as_text(), Some("set"));
        assert_eq!(
            node.get_attr("xmlns").unwrap().as_text(),
            Some("w:sync:app:state")
        );
        let sync = &node.child_nodes()[0];
        assert_eq!(
            sync.get_attr("data_namespace").unwrap().as_text(),
            Some("3")
        );
        let collections = sync.child_nodes();
        assert_eq!(collections.len(), 5);
        for c in collections {
            assert_eq!(c.description, "collection");
            assert_eq!(c.get_attr("version").unwrap().as_text(), Some("0"));
            assert_eq!(
                c.get_attr("return_snapshot").unwrap().as_text(),
                Some("true")
            );
            assert!(
                c.child_nodes().is_empty(),
                "a fetch must not carry a patch child"
            );
        }
        assert!(fetch_request("F1", &[]).is_err());
        assert!(fetch_request("", &ALL_COLLECTIONS).is_err());
    }

    #[test]
    fn incremental_fetch_preserves_held_versions_and_native_snapshot_condition() {
        // Current 43.72 EfK.A01 has separate null/zero/positive version arms.
        // A notification's dirty version is not an input to this fixture.
        for snapshot in [false, true] {
            let node = fetch_request_at_versions(
                "F2",
                &[
                    (Collection::Regular, None),
                    (Collection::CriticalUnblockLow, Some(0)),
                    (Collection::CriticalBlock, Some(7)),
                ],
                snapshot,
            )
            .unwrap();
            assert_eq!(node.get_attr("id").unwrap().as_text(), Some("F2"));
            let sync = &node.child_nodes()[0];
            assert_eq!(sync.description, "sync");
            assert_eq!(
                sync.get_attr("data_namespace").unwrap().as_text(),
                Some("3")
            );
            let children = sync.child_nodes();
            assert_eq!(children.len(), 3);
            for (index, (version, order)) in [(None, "1"), (Some("0"), "0"), (Some("7"), "1")]
                .into_iter()
                .enumerate()
            {
                let c = &children[index];
                assert_eq!(c.get_attr("version").and_then(|v| v.as_text()), version);
                assert_eq!(c.get_attr("order").unwrap().as_text(), Some(order));
                assert_eq!(
                    c.get_attr("return_snapshot").and_then(|v| v.as_text()),
                    (snapshot && index < 2).then_some("true")
                );
                assert!(c.child_nodes().is_empty());
            }
        }
    }

    #[test]
    fn incremental_fetch_retains_large_integer_precision_through_wire_codec() {
        use crate::binary::{Decoder, Encoder, TokenDictionary};
        let node = fetch_request_at_versions(
            "F3",
            &[
                (Collection::RegularHigh, Some(9_007_199_254_740_993)),
                (Collection::CriticalBlock, Some(i64::MAX as u64)),
            ],
            true,
        )
        .unwrap();
        let dict = TokenDictionary::comparison_2_26_37_71();
        let encoded = Encoder::new(dict).encode(&node).unwrap();
        let decoded = Decoder::new(&encoded[1..], dict, &crate::profile::Limits::default())
            .decode_node()
            .unwrap();
        assert_eq!(decoded, node);
        let children = decoded.child_nodes()[0].child_nodes();
        assert_eq!(
            children[0].get_attr("version").unwrap().as_text(),
            Some("9007199254740993")
        );
        assert_eq!(
            children[1].get_attr("version").unwrap().as_text(),
            Some("9223372036854775807")
        );
        assert!(
            children
                .iter()
                .all(|c| c.get_attr("return_snapshot").is_none())
        );
    }

    #[test]
    fn incremental_fetch_rejects_missing_targets_bad_ids_and_unsigned_overflow() {
        let target = [(Collection::Regular, None)];
        assert!(fetch_request_at_versions("F4", &[], true).is_err());
        assert!(fetch_request_at_versions("", &target, false).is_err());
        assert!(fetch_request_at_versions(&"x".repeat(129), &target, false).is_err());
        assert!(
            fetch_request_at_versions(
                "F4",
                &[(Collection::Regular, Some(i64::MAX as u64 + 1))],
                false
            )
            .is_err()
        );
    }

    #[test]
    fn incremental_fetch_refuses_duplicate_collections_even_with_matching_versions() {
        for versions in [
            (None, None),
            (Some(7), Some(7)),
            (Some(7), Some(8)),
            (None, Some(0)),
        ] {
            let error = fetch_request_at_versions(
                "F5",
                &[
                    (Collection::Regular, versions.0),
                    (Collection::Regular, versions.1),
                ],
                true,
            )
            .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("duplicate fetch collection: regular")
            );
        }
        assert!(fetch_request("F5", &[Collection::Regular, Collection::Regular]).is_err());
    }

    #[test]
    fn patch_request_refuses_invalid_inputs() {
        assert!(patch_request("", Collection::Regular, Some(1), &[1], false).is_err());
        assert!(
            patch_request(
                "P",
                Collection::Regular,
                Some(i64::MAX as u64 + 1),
                &[1],
                false
            )
            .is_err()
        );
        assert!(patch_request("A", Collection::Regular, Some(1), &[], false).is_err());
    }

    /// The encoder must be able to carry this node, since a patch is raw bytes
    /// inside an element whose name is not a dictionary token in every build.
    #[test]
    fn the_request_round_trips_through_the_comparison_codec() {
        use crate::binary::{Decoder, Encoder, TokenDictionary};
        let node = patch_request("RT1", Collection::RegularHigh, Some(2), &[7; 64], false).unwrap();
        let dict = TokenDictionary::comparison_2_26_37_71();
        let encoded = Encoder::new(dict).encode(&node).unwrap();
        // The encoder prepends a flag byte; the decoder reads the node after it.
        let decoded = Decoder::new(&encoded[1..], dict, &crate::profile::Limits::default())
            .decode_node()
            .unwrap();
        assert_eq!(decoded, node, "the request must survive the wire codec");
        let patch = &decoded.child_nodes()[0].child_nodes()[0].child_nodes()[0];
        assert!(matches!(&patch.content, NodeContent::Bytes(b) if b.len() == 64));
    }
}

#[cfg(all(
    test,
    feature = "experimental-signal-crypto",
    feature = "experimental-noise",
    feature = "experimental-transport"
))]
mod per_collection_tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        signal::app_state_collection::SavedCollectionState,
        store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
    };

    /// An accepted patch is promoted into the collection it was built for.
    /// `ingest` used to promote every stash into `critical_unblock_low`, which
    /// the first write to a second collection would have overwritten.
    #[test]
    fn an_accepted_push_name_patch_lands_in_critical_block_only() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("p.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([9; 32]))),
            create: true,
        })
        .unwrap();
        let registered = [0u32].into_iter().collect();
        let id = queue_push_name_patch(&store, 0, 0, &registered, 1_000, "Lab").unwrap();
        crate::transport::actions::reserve_pending(&store).unwrap();
        let reply = Node::new("iq")
            .attr("id", id.as_str())
            .attr("type", "result")
            .children(vec![Node::new("sync").children(vec![
                Node::new("collection")
                    .attr("name", "critical_block")
                    .attr("version", "1"),
            ])]);
        assert!(crate::transport::actions::record_reply(&store, &reply).unwrap());
        assert_eq!(ingest(&store).unwrap(), 1);
        let held = |c: Collection| -> Option<SavedCollectionState> {
            store
                .read(|tx| tx.get_json(Table::Checkpoints, held_key(c).as_bytes(), 1))
                .unwrap()
        };
        let block = held(Collection::CriticalBlock).expect("promoted into critical_block");
        assert_eq!(block.version, 1);
        assert_eq!(block.collection, "critical_block");
        assert!(held(Collection::CriticalUnblockLow).is_none());
    }
}

#[cfg(test)]
mod settlement_atomic_tests {
    use super::*;
    use crate::{
        model::ids::AccountId,
        signal::app_state_collection::SavedCollectionState,
        store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
    };
    use std::{path::Path, sync::Arc};

    fn open(path: &Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("settlement-test").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn queue(store: &ProtocolStore) -> Result<String> {
        queue_push_name_patch(store, 0, 0, &[0].into_iter().collect(), 1_000, "Fixture")
    }
    fn snapshot(store: &ProtocolStore) -> Vec<(Vec<u8>, Vec<u8>)> {
        store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
            .unwrap()
    }
    fn prepared(store: &ProtocolStore, id: &str) -> SavedCollectionState {
        store
            .read(|tx| {
                tx.get_json::<PendingState>(Table::Checkpoints, pending_key(id).as_bytes(), 1)
            })
            .unwrap()
            .unwrap()
            .prepared()
            .clone()
    }
    fn success(id: &str, name: &str, version: &str) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "result")
            .children(vec![Node::new("sync").children(
                vec![Node::new("collection").attr("name", name).attr("version", version)],
            )])
    }

    #[test]
    fn settled_app_state_outcomes_cannot_be_adopted_by_a_changed_account_after_reopen() {
        use crate::session::{AccountRecord, RegistrationStatus};
        for fetch_error in [false, true] {
            for change in ["same", "observation", "jid", "logout", "registration-id"] {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("settled-owner.redb");
                let store = open(&path);
                let original = AccountRecord {
                    generation: store.generation(),
                    registration: RegistrationStatus::Registered,
                    own_jid: Some(Jid::parse("100@s.whatsapp.net").unwrap()),
                    ..Default::default()
                };
                store.write(|tx| original.save(tx)).unwrap();
                let id = if fetch_error {
                    queue_fetch(&store).unwrap()
                } else {
                    queue(&store).unwrap()
                };
                super::super::actions::reserve_next(&store)
                    .unwrap()
                    .unwrap();
                let reply = if fetch_error {
                    Node::new("iq")
                        .attr("id", id.as_str())
                        .attr("type", "error")
                        .children(vec![
                            Node::new("error").attr("code", "503").attr("backoff", "60"),
                        ])
                } else {
                    success(&id, "critical_block", "1")
                };
                assert!(super::super::actions::record_reply(&store, &reply).unwrap());
                let mut replacement = original.clone();
                match change {
                    "observation" => {
                        replacement.last_authenticated_at = Some(crate::model::time::Timestamp(123))
                    }
                    "jid" => replacement.own_jid = Some(Jid::parse("200@s.whatsapp.net").unwrap()),
                    "logout" => replacement.registration = RegistrationStatus::Unregistered,
                    "registration-id" => replacement.signal_registration_id = Some(42),
                    _ => {}
                }
                store.write(|tx| replacement.save(tx)).unwrap();
                drop(store);
                let store = open(&path);
                let before = snapshot(&store);
                let journal = store.journal_head().unwrap();
                let result = ingest_action(&store, &id);
                if matches!(change, "same" | "observation") {
                    assert_eq!(result.unwrap(), usize::from(!fetch_error));
                    if fetch_error {
                        assert!(
                            store
                                .read(|tx| tx.get(Table::Checkpoints, SERVER_BACKOFF, 1))
                                .unwrap()
                                .is_some()
                        );
                    } else {
                        assert!(
                            store
                                .read(|tx| tx.get(
                                    Table::Checkpoints,
                                    held_key(Collection::CriticalBlock).as_bytes(),
                                    1
                                ))
                                .unwrap()
                                .is_some()
                        );
                    }
                } else {
                    assert!(result.is_err());
                    assert_eq!(snapshot(&store), before);
                    assert_eq!(store.journal_head().unwrap(), journal);
                    if fetch_error {
                        recover_fetch_backoff(&store).unwrap();
                        assert_eq!(snapshot(&store), before);
                        assert_eq!(store.journal_head().unwrap(), journal);
                    }
                }
            }
        }
    }
    #[test]
    fn accepted_versions_follow_java_decimal_without_weakening_prepared_authority() {
        // Handwritten oracle cases for Java Long.parseLong's signed BMP digits;
        // no whitespace, separators, surrogate pairs or out-of-range numbers.
        for (version, accepted) in [
            ("+1", true),
            ("٠١", true),
            ("００１", true),
            ("-0", false),
            (" 1", false),
            ("1 ", false),
            ("-1", false),
            ("2", false),
            ("9223372036854775808", false),
            ("𝟙", false),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("java-version.redb"));
            let id = queue(&store).unwrap();
            let expected = prepared(&store, &id);
            super::super::actions::reserve_next(&store)
                .unwrap()
                .unwrap();
            assert!(
                super::super::actions::record_reply(
                    &store,
                    &success(&id, "critical_block", version)
                )
                .unwrap()
            );
            let before = snapshot(&store);
            let result = ingest_action(&store, &id);
            if accepted {
                assert_eq!(result.unwrap(), 1);
                let held: SavedCollectionState = store
                    .read(|tx| {
                        tx.get_json(
                            Table::Checkpoints,
                            held_key(Collection::CriticalBlock).as_bytes(),
                            1,
                        )
                    })
                    .unwrap()
                    .unwrap();
                assert_eq!(held, expected);
                assert_eq!(ingest_action(&store, &id).unwrap(), 0);
            } else {
                assert!(result.is_err());
                assert_eq!(snapshot(&store), before);
            }
        }
    }

    #[test]
    fn queue_and_prepared_state_are_atomic_single_flight_and_reopen_safe() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("one.redb");
        let store = open(&path);
        let id = queue(&store).unwrap();
        let pending = store
            .read(|tx| {
                tx.get_json::<PendingState>(Table::Checkpoints, pending_key(&id).as_bytes(), 1)
            })
            .unwrap()
            .unwrap();
        assert!(
            matches!(pending, PendingState::Bound { base, prepared } if base.version == 0 && prepared.version == 1)
        );
        let before = snapshot(&store);
        assert!(queue(&store).is_err());
        assert_eq!(snapshot(&store), before);
        assert_eq!(
            super::super::actions::reserve_pending(&store)
                .unwrap()
                .len(),
            1
        );
        let sent = snapshot(&store);
        assert!(queue(&store).is_err());
        assert_eq!(snapshot(&store), sent);
        drop(store);
        let store = open(&path);
        assert!(
            super::super::actions::reserve_pending(&store)
                .unwrap()
                .is_empty()
        );
        assert!(
            super::super::actions::record_reply(&store, &success(&id, "critical_block", "1"))
                .unwrap()
        );
        assert_eq!(ingest(&store).unwrap(), 1);
        assert_eq!(ingest(&store).unwrap(), 0);
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, pending_key(&id).as_bytes(), 1))
                .unwrap()
                .is_none()
        );
        let next = queue(&store).unwrap();
        assert_ne!(id, next);
        assert_eq!(prepared(&store, &next).version, 2);
    }

    #[test]
    fn concurrent_publishers_cannot_claim_the_same_collection_baseline() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(open(&dir.path().join("concurrent.redb")));
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let jobs: Vec<_> = (0..2)
            .map(|_| {
                let store = store.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    queue(&store)
                })
            })
            .collect();
        let results: Vec<_> = jobs.into_iter().map(|j| j.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
        assert_eq!(super::super::actions::outcomes(&store).unwrap().len(), 1);
        assert_eq!(
            super::super::actions::reserve_pending(&store)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn queue_errors_roll_back_key_generation_and_do_not_mask_corrupt_keys() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("rollback.redb"));
        let mut bad = crate::signal::app_state_collection::CollectionState::empty("critical_block")
            .unwrap()
            .to_saved();
        bad.hash.pop();
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    held_key(Collection::CriticalBlock).as_bytes(),
                    1,
                    &bad,
                )
            })
            .unwrap();
        let before = snapshot(&store);
        assert!(queue(&store).is_err());
        assert_eq!(snapshot(&store), before);
        store
            .write(|tx| {
                tx.delete(
                    Table::Checkpoints,
                    held_key(Collection::CriticalBlock).as_bytes(),
                )?;
                tx.put(
                    Table::Checkpoints,
                    b"app-state-key:000000000001",
                    1,
                    b"not-json",
                )
            })
            .unwrap();
        let before = snapshot(&store);
        assert!(queue(&store).is_err());
        assert_eq!(snapshot(&store), before);
    }

    #[test]
    fn mismatched_or_ambiguous_patch_results_keep_the_original_pending_state() {
        for case in 0..4 {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("mismatch.redb");
            let store = open(&path);
            let id = queue(&store).unwrap();
            super::super::actions::reserve_pending(&store).unwrap();
            let reply = match case {
                0 => success(&id, "regular", "1"),
                1 => success(&id, "critical_block", "2"),
                2 => Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result"),
                _ => Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
                    .children(vec![Node::new("sync").children(vec![
                    Node::new("collection").attr("name","critical_block").attr("version","1"),
                    Node::new("collection").attr("name","regular").attr("version","1")])]),
            };
            super::super::actions::record_reply(&store, &reply).unwrap();
            let before = snapshot(&store);
            assert!(ingest_action(&store, &id).is_err());
            assert_eq!(snapshot(&store), before);
            drop(store);
            let store = open(&path);
            assert_eq!(snapshot(&store), before);
            assert!(
                store
                    .read(|tx| tx.get(
                        Table::Checkpoints,
                        held_key(Collection::CriticalBlock).as_bytes(),
                        1
                    ))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn accepted_response_cannot_regress_a_later_crypto_state() {
        use crate::signal::{
            app_state, app_state_actions,
            app_state_collection::{CollectionState, patch::OutgoingMutation},
            app_state_crypto::MutationOperation,
        };
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("stale.redb"));
        let id = queue(&store).unwrap();
        super::super::actions::reserve_pending(&store).unwrap();
        let old = prepared(&store, &id);
        let action = app_state_actions::push_name_mutation("Later", 2_000).unwrap();
        let mutation = OutgoingMutation {
            operation: MutationOperation::Set,
            index: action.index,
            action: Some(action.value),
            version: action.version,
        };
        store
            .write(|tx| {
                let key = app_state::select_active_key(
                    tx,
                    0,
                    0,
                    &[0].into_iter().collect(),
                    2_000,
                    i64::MAX,
                )?
                .unwrap();
                let next = CollectionState::from_saved(&old)?
                    .build_patch(key.keys(), *key.id(), 0, &[mutation])?
                    .state
                    .to_saved();
                assert_eq!(next.version, 2);
                tx.put_json(
                    Table::Checkpoints,
                    held_key(Collection::CriticalBlock).as_bytes(),
                    1,
                    &next,
                )
            })
            .unwrap();
        super::super::actions::record_reply(&store, &success(&id, "critical_block", "1")).unwrap();
        let before = snapshot(&store);
        assert!(ingest(&store).is_err());
        assert_eq!(snapshot(&store), before);
    }

    #[test]
    fn legacy_rows_are_cryptographically_reverified_with_original_wire() {
        for (already_sent, bound, existing_baseline) in
            (0..8).map(|case| (case & 1 != 0, case & 2 != 0, case & 4 != 0))
        {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("legacy.redb"));
            if existing_baseline {
                let first = queue(&store).unwrap();
                super::super::actions::reserve_pending(&store).unwrap();
                super::super::actions::record_reply(
                    &store,
                    &success(&first, "critical_block", "1"),
                )
                .unwrap();
                assert_eq!(ingest(&store).unwrap(), 1);
            }
            let id = queue(&store).unwrap();
            let wire = super::super::actions::request(&store, &id).unwrap();
            if already_sent {
                super::super::actions::reserve_pending(&store).unwrap();
            }
            let expected = prepared(&store, &id);
            let mut post = expected.clone();
            post.logical_indexes = None;
            store
                .write(|tx| {
                    let pending = if bound {
                        let mut base = held_state_in(tx, Collection::CriticalBlock)?.to_saved();
                        base.logical_indexes = None;
                        PendingState::Bound {
                            base,
                            prepared: post.clone(),
                        }
                    } else {
                        PendingState::Legacy(post.clone())
                    };
                    tx.put_json(Table::Checkpoints, pending_key(&id).as_bytes(), 1, &pending)
                })
                .unwrap();
            if !already_sent {
                super::super::actions::reserve_pending(&store).unwrap();
            }
            assert_eq!(super::super::actions::request(&store, &id).unwrap(), wire);
            super::super::actions::record_reply(
                &store,
                &success(&id, "critical_block", &expected.version.to_string()),
            )
            .unwrap();
            assert_eq!(ingest(&store).unwrap(), 1);
            assert_eq!(
                store
                    .read(|tx| tx.get_json::<SavedCollectionState>(
                        Table::Checkpoints,
                        held_key(Collection::CriticalBlock).as_bytes(),
                        1
                    ))
                    .unwrap()
                    .unwrap(),
                expected
            );
            assert_eq!(ingest(&store).unwrap(), 0);
        }
    }

    #[test]
    fn legacy_preparation_never_invents_baseline_or_ignores_supplied_indexes() {
        for (sent, missing_baseline) in (0..4).map(|case| (case & 1 != 0, case & 2 != 0)) {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("legacy-refusal.redb"));
            let first = queue(&store).unwrap();
            super::super::actions::reserve_pending(&store).unwrap();
            super::super::actions::record_reply(&store, &success(&first, "critical_block", "1"))
                .unwrap();
            ingest(&store).unwrap();
            let id = queue(&store).unwrap();
            if sent {
                super::super::actions::reserve_pending(&store).unwrap();
            }
            store
                .write(|tx| {
                    let mut post = prepared_from_tx(tx, &id)?;
                    if missing_baseline {
                        post.logical_indexes = None;
                        let mut baseline = held_state_in(tx, Collection::CriticalBlock)?.to_saved();
                        baseline.logical_indexes = None;
                        tx.put_json(
                            Table::Checkpoints,
                            held_key(Collection::CriticalBlock).as_bytes(),
                            1,
                            &baseline,
                        )?;
                    } else {
                        post.logical_indexes.as_mut().unwrap()[0].0 = br#"["wrong"]"#.to_vec();
                    }
                    tx.put_json(
                        Table::Checkpoints,
                        pending_key(&id).as_bytes(),
                        1,
                        &PendingState::Legacy(post),
                    )
                })
                .unwrap();
            if sent {
                super::super::actions::record_reply(&store, &success(&id, "critical_block", "2"))
                    .unwrap();
            }
            let before = snapshot(&store);
            if sent {
                assert!(ingest_action(&store, &id).is_err());
            } else {
                assert!(super::super::actions::reserve_pending(&store).is_err());
            }
            assert_eq!(snapshot(&store), before);
        }
    }

    #[test]
    fn orphaned_or_corrupt_prepared_actions_refuse_before_reservation() {
        for orphan in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("orphan.redb"));
            let id = queue(&store).unwrap();
            store
                .write(|tx| {
                    if orphan {
                        tx.delete(Table::Checkpoints, pending_key(&id).as_bytes())?;
                        Ok(())
                    } else {
                        let mut post = prepared_from_tx(tx, &id)?;
                        post.hash[0] ^= 1;
                        tx.put_json(Table::Checkpoints, pending_key(&id).as_bytes(), 1, &post)
                    }
                })
                .unwrap();
            let before = snapshot(&store);
            assert!(super::super::actions::reserve_pending(&store).is_err());
            assert_eq!(snapshot(&store), before);
            assert_eq!(
                super::super::actions::outcomes(&store).unwrap()[0].status,
                super::super::actions::ActionStatus::Queued
            );
        }
    }
    fn prepared_from_tx(tx: &crate::store::WriteTx<'_>, id: &str) -> Result<SavedCollectionState> {
        Ok(tx
            .get_json::<PendingState>(Table::Checkpoints, pending_key(id).as_bytes(), 1)?
            .unwrap()
            .prepared()
            .clone())
    }
}

#[cfg(test)]
mod server_backoff_tests {
    use super::*;
    use crate::{
        model::{ids::AccountId, time::Timestamp},
        store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
    };
    use std::{path::Path, sync::Arc};
    fn open(path: &Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("backoff").unwrap(),
            key_source: Arc::new(StaticKeySource::new(MasterKey::from_bytes([8; 32]))),
            create: true,
        })
        .unwrap()
    }
    fn queue(store: &ProtocolStore) -> String {
        queue_push_name_patch(store, 0, 0, &[0].into_iter().collect(), 1_000, "Backoff").unwrap()
    }
    fn error(id: &str, backoff: &str) -> Node {
        Node::new("iq")
            .attr("id", id)
            .attr("type", "error")
            .children(vec![
                Node::new("error")
                    .attr("code", "503")
                    .attr("backoff", backoff),
            ])
    }
    fn snapshot(store: &ProtocolStore) -> Vec<(Vec<u8>, Vec<u8>)> {
        store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 1000))
            .unwrap()
    }

    #[test]
    fn prospective_held_fetch_keeps_shared_refusal_backoff_and_safe_inspection() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("held-fetch-wait.redb"));
        let mut baseline = crate::signal::app_state_collection::CollectionState::empty("regular")
            .unwrap()
            .to_saved();
        baseline.version = 7;
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    held_key(Collection::Regular).as_bytes(),
                    1,
                    &baseline,
                )
            })
            .unwrap();
        let id = held_fetch::queue(&store, &[Collection::Regular]).unwrap();
        let original = super::super::actions::request(&store, &id).unwrap();
        super::super::actions::reserve_next(&store)
            .unwrap()
            .unwrap();
        super::super::actions::record_reply(&store, &error(&id, "3600")).unwrap();
        assert_eq!(ingest_action(&store, &id).unwrap(), 0);
        let info = held_fetch::inspect(&store, &id).unwrap().unwrap();
        assert_eq!(info.phase, "admitted");
        assert_eq!(info.status, super::super::actions::ActionStatus::Error);
        assert_eq!(info.error_code, Some(503));
        assert!(held_fetch::queue(&store, &[Collection::Regular]).is_err());
        assert!(queue_fetch(&store).is_err());
        assert_eq!(
            super::super::actions::request(&store, &id).unwrap(),
            original
        );
        let current: crate::signal::app_state_collection::SavedCollectionState = store
            .read(|tx| {
                tx.get_json(
                    Table::Checkpoints,
                    held_key(Collection::Regular).as_bytes(),
                    1,
                )
            })
            .unwrap()
            .unwrap();
        assert_eq!(current, baseline);
    }

    #[test]
    fn fetch_error_wait_recovers_after_cut_and_defers_only_app_state_work() {
        for live in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("fetch-wait.redb");
            let store = open(&path);
            let id = queue_fetch(&store).unwrap();
            let later = queue_fetch(&store).unwrap();
            let patch = queue(&store);
            let other = super::super::actions::queue(&store, "fixture", "fixture", |id| {
                Ok(Node::new("iq").attr("id", id).attr("type", "get"))
            })
            .unwrap();
            let original = super::super::actions::request(&store, &id).unwrap();
            store
                .write(|tx| super::super::actions::reserve_one_in(tx, &id))
                .unwrap();
            super::super::actions::record_reply(&store, &error(&id, "3600")).unwrap();
            if live {
                assert_eq!(ingest_action(&store, &id).unwrap(), 0);
            }
            drop(store);
            let store = open(&path);
            recover_fetch_backoff(&store).unwrap();
            let until: Timestamp = store
                .read(|tx| tx.get_json(Table::Checkpoints, SERVER_BACKOFF, 1))
                .unwrap()
                .unwrap();
            assert!(until > Timestamp::now());
            let before = snapshot(&store);
            recover_fetch_backoff(&store).unwrap();
            assert_eq!(
                snapshot(&store),
                before,
                "recovery cannot extend original receipt-time wait"
            );
            assert!(queue_fetch(&store).is_err());
            let ordinary = super::super::actions::reserve_next(&store)
                .unwrap()
                .unwrap();
            assert_eq!(
                ordinary.get_attr("id").and_then(|v| v.as_text()),
                Some(other.as_str())
            );
            assert!(
                super::super::actions::reserve_next(&store)
                    .unwrap()
                    .is_none()
            );
            for id in [later, patch] {
                assert_eq!(
                    super::super::actions::outcomes(&store)
                        .unwrap()
                        .iter()
                        .find(|a| a.id == id)
                        .unwrap()
                        .status,
                    super::super::actions::ActionStatus::Queued
                );
            }
            assert_eq!(
                super::super::actions::request(&store, &id).unwrap(),
                original
            );
            assert!(
                store
                    .read(|tx| tx.get(
                        Table::Checkpoints,
                        held_key(Collection::CriticalBlock).as_bytes(),
                        1
                    ))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn malformed_fetch_wait_preserves_reply_and_refuses_new_explicit_fetch() {
        for raw in ["private-invalid", "0.5", "-2", "9223372036854775807"] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("fetch-invalid.redb"));
            let id = queue_fetch(&store).unwrap();
            super::super::actions::reserve_next(&store)
                .unwrap()
                .unwrap();
            super::super::actions::record_reply(&store, &error(&id, raw)).unwrap();
            let before = snapshot(&store);
            let failure = ingest_action(&store, &id).unwrap_err().to_string();
            assert!(!failure.contains("private-invalid"));
            recover_fetch_backoff(&store).unwrap();
            assert!(queue_fetch(&store).is_err());
            assert_eq!(snapshot(&store), before);
        }
    }

    #[test]
    fn fetch_wait_is_not_inferred_from_unrelated_or_inner_error_paths() {
        for case in 0..3 {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("fetch-scope.redb"));
            let id = super::super::actions::queue(
                &store,
                if case == 0 {
                    "fixture"
                } else {
                    FETCH_ACTION_KIND
                },
                "fixture",
                |id| {
                    if case == 2 {
                        Ok(Node::new("iq")
                            .attr("id", id)
                            .attr("type", "set")
                            .attr("xmlns", "unrelated"))
                    } else {
                        fetch_request(id, &ALL_COLLECTIONS)
                    }
                },
            )
            .unwrap();
            super::super::actions::reserve_next(&store)
                .unwrap()
                .unwrap();
            let reply = if case == 1 {
                Node::new("iq")
                    .attr("id", id.as_str())
                    .attr("type", "result")
                    .children(vec![Node::new("sync").children(vec![
                        Node::new("collection").attr("type", "error").children(
                            vec![Node::new("error").attr("code", "503").attr("backoff", "3600")],
                        ),
                    ])])
            } else {
                error(&id, "3600")
            };
            super::super::actions::record_reply(&store, &reply).unwrap();
            let before = snapshot(&store);
            let result = ingest_action(&store, &id);
            assert_eq!(result.is_err(), case == 2);
            assert_eq!(snapshot(&store), before);
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, SERVER_BACKOFF, 1))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn server_backoff_survives_reopen_and_defers_only_never_sent_patches() {
        use crate::signal::{
            app_state_actions, app_state_collection::patch::OutgoingMutation,
            app_state_crypto::MutationOperation,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wait.redb");
        let store = open(&path);
        let id = queue(&store);
        let contact = app_state_actions::contact_mutation(
            &Jid::parse("15555550100@s.whatsapp.net").unwrap(),
            "Fixture",
            None,
            true,
            1_000,
        )
        .unwrap();
        let other = queue_patch(
            &store,
            0,
            0,
            &[0].into_iter().collect(),
            1_000,
            Collection::CriticalUnblockLow,
            &[OutgoingMutation {
                operation: MutationOperation::Set,
                index: contact.index,
                action: Some(contact.value),
                version: contact.version,
            }],
            "contact fixture",
        )
        .unwrap();
        let fetch = queue_fetch(&store).unwrap();
        store
            .write(|tx| super::super::actions::reserve_one_in(tx, &id).map(|_| ()))
            .unwrap();
        super::super::actions::record_reply(&store, &error(&id, "60")).unwrap();
        let received = store
            .write(|tx| super::super::actions::settled_at_in(tx, &id))
            .unwrap()
            .unwrap();
        assert_eq!(ingest_action(&store, &id).unwrap(), 0);
        let until: Timestamp = store
            .read(|tx| tx.get_json(Table::Checkpoints, SERVER_BACKOFF, 1))
            .unwrap()
            .unwrap();
        assert_eq!(until.0, received.0 + 60_000);
        let before = snapshot(&store);
        assert_eq!(ingest_action(&store, &id).unwrap(), 0);
        assert_eq!(snapshot(&store), before);
        assert!(
            queue_push_name_patch(&store, 0, 0, &[0].into_iter().collect(), 2_000, "Too early")
                .is_err()
        );
        assert!(queue_fetch(&store).is_err());
        assert_eq!(snapshot(&store), before);
        drop(store);
        let store = open(&path);
        assert!(
            super::super::actions::reserve_pending(&store)
                .unwrap()
                .is_empty()
        );
        assert_eq!(snapshot(&store), before);
        let control = super::super::actions::queue(&store, "fixture", "control", |id| {
            Ok(Node::new("iq").attr("id", id).attr("type", "get"))
        })
        .unwrap();
        let writes = super::super::actions::reserve_pending(&store).unwrap();
        assert_eq!(writes.len(), 1);
        assert_eq!(
            writes[0].get_attr("id").and_then(|v| v.as_text()),
            Some(control.as_str())
        );
        let outcomes = super::super::actions::outcomes(&store).unwrap();
        assert_eq!(
            outcomes.iter().find(|o| o.id == other).unwrap().status,
            super::super::actions::ActionStatus::Queued
        );
        // Simulate expiration without a wall-clock sleep or replaying sent work.
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    SERVER_BACKOFF,
                    1,
                    &Timestamp(Timestamp::now().0 - 1),
                )
            })
            .unwrap();
        let writes = super::super::actions::reserve_pending(&store).unwrap();
        assert_eq!(writes.len(), 2);
        let ids: std::collections::BTreeSet<_> = writes
            .iter()
            .map(|n| n.get_attr("id").unwrap().as_text().unwrap())
            .collect();
        assert_eq!(ids, [other.as_str(), fetch.as_str()].into_iter().collect());
        assert!(
            super::super::actions::reserve_pending(&store)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn malformed_or_overflowing_backoff_preserves_pending_preparation() {
        for raw in ["private-invalid", "0.5", "-2", "9223372036854775807"] {
            let dir = tempfile::tempdir().unwrap();
            let store = open(&dir.path().join("invalid.redb"));
            let id = queue(&store);
            super::super::actions::reserve_pending(&store).unwrap();
            super::super::actions::record_reply(&store, &error(&id, raw)).unwrap();
            let before = snapshot(&store);
            let failure = ingest_action(&store, &id).unwrap_err().to_string();
            assert!(!failure.contains("private-invalid"));
            assert_eq!(snapshot(&store), before);
        }
    }
}
