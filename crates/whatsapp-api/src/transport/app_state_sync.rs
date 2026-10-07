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
//! This module builds the request only. It never mints keys, never chooses a
//! collection for an action, and never persists the post-patch state — the
//! caller owns all three, because the post-patch state must not be saved until
//! the server accepts the patch.
use crate::{Result, binary::Node, model::jid::Jid};

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
    let children: Vec<Node> = collections
        .iter()
        .map(|c| {
            Node::new("collection")
                .attr("name", c.name())
                .attr("order", c.order().to_string().as_str())
                .attr("version", "0")
                .attr("return_snapshot", "true")
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

/// Queue a full app-state fetch through the durable one-shot action queue.
/// Returns the action id; nothing is sent until a session drains the queue.
/// Read-only: no patch, no mutation.
pub fn queue_fetch(store: &crate::store::ProtocolStore) -> Result<String> {
    super::actions::queue(store, FETCH_ACTION_KIND, "app-state fetch", |id| {
        fetch_request(id, &ALL_COLLECTIONS)
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
    // Mint on demand: a primary with no companions still mints its own key
    // (LX/0oK->A05, empty-device-collection branch), purely locally.
    let key_id = store.write(|tx| {
        match app_state::select_active_key(
            tx,
            raw_id,
            device_index.into(),
            registered,
            now_ms,
            i64::MAX,
        ) {
            Ok(Some(selected)) => Ok(*selected.id()),
            _ => app_state::mint_key(tx, device_index, raw_id, registered, now_ms),
        }
    })?;
    let keys = store.write(|tx| {
        app_state::select_active_key(
            tx,
            raw_id,
            device_index.into(),
            registered,
            now_ms,
            i64::MAX,
        )?
        .ok_or_else(|| crate::Error::invalid("app_state_sync", "minted key is not selectable"))
    })?;
    // Build on whatever the server last accepted. Starting from `empty` every
    // time would claim version 1 for ever and hash over nothing.
    let state = held_state(store, collection)?;
    let held = state.version();
    let built = state.build_patch(keys.keys(), key_id, device_index.into(), mutations)?;
    let saved = built.state.to_saved();
    let id = super::actions::queue(store, PATCH_ACTION_KIND, label, |id| {
        // The attribute is the version we *hold*, omitted when we hold none
        // (confirmed, both `CSe.A01` call sites). The patch's own claimed
        // version is never on the wire — `SyncdPatch.version` is read-only in
        // the APK and sending it is what three refusals had in common.
        patch_request(
            id,
            collection,
            (held > 0).then_some(held),
            &built.bytes,
            false,
        )
    })?;
    // Stashed against the action, promoted only once the server accepts it: a
    // refused patch must not advance the version we claim to hold.
    store.write(|tx| {
        tx.put_json(
            crate::store::Table::Checkpoints,
            pending_key(&id).as_bytes(),
            1,
            &saved,
        )
    })?;
    Ok(id)
}

const HELD_PREFIX: &str = "app-state-collection:";
const PENDING_PREFIX: &str = "app-state-pending:";

fn held_key(collection: Collection) -> String {
    format!("{HELD_PREFIX}{}", collection.name())
}
fn pending_key(action_id: &str) -> String {
    format!("{PENDING_PREFIX}{action_id}")
}

/// The state the server last accepted for a collection, or an empty one.
fn held_state(
    store: &crate::store::ProtocolStore,
    collection: Collection,
) -> Result<crate::signal::app_state_collection::CollectionState> {
    use crate::signal::app_state_collection::{CollectionState, SavedCollectionState};
    let saved: Option<SavedCollectionState> = store.read(|tx| {
        tx.get_json(
            crate::store::Table::Checkpoints,
            held_key(collection).as_bytes(),
            1,
        )
    })?;
    match saved {
        Some(saved) => CollectionState::from_saved(&saved),
        None => CollectionState::empty(collection.name()),
    }
}

/// Promote the stashed state of every accepted patch, so the next one builds on
/// it. Returns how many were applied.
///
/// The server's `result` names the version it now holds
/// (`<collection name=… version=…/>`); that is compared with what our patch
/// claimed, and a mismatch leaves the state unpromoted rather than recording a
/// version the server does not agree with.
pub fn ingest(store: &crate::store::ProtocolStore) -> Result<usize> {
    use crate::signal::app_state_collection::SavedCollectionState;
    use crate::store::Table;
    let mut applied = 0;
    for (outcome, reply) in super::actions::settled(store, PATCH_ACTION_KIND)? {
        let pending = pending_key(&outcome.id);
        let Some(saved): Option<SavedCollectionState> =
            store.read(|tx| tx.get_json(Table::Checkpoints, pending.as_bytes(), 1))?
        else {
            continue;
        };
        let accepted = outcome.status == super::actions::ActionStatus::Ok;
        let server_version = reply
            .as_ref()
            .and_then(|r| r.child("sync"))
            .and_then(|s| s.child("collection"))
            .and_then(|c| c.get_attr("version"))
            .and_then(|v| v.as_text())
            .and_then(|v| v.parse::<u64>().ok());
        // Each stash names its own collection. Promoting into a fixed one would
        // overwrite, say, `critical_unblock_low` with `critical_block`'s state
        // the first time a second collection is written.
        let collection = Collection::from_name(&saved.collection).ok_or_else(|| {
            crate::Error::invalid("app_state_sync", "stashed state names no known collection")
        })?;
        if accepted && server_version == Some(saved.version) {
            store.write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    held_key(collection).as_bytes(),
                    1,
                    &saved,
                )?;
                tx.delete(Table::Checkpoints, pending.as_bytes())
            })?;
            applied += 1;
        } else {
            // Refused, or accepted at a version we did not claim. Either way the
            // stash is dropped rather than promoted; the next patch rebuilds
            // from the last state the server and we agreed on.
            store.write(|tx| tx.delete(Table::Checkpoints, pending.as_bytes()))?;
            // A conflict is not a dead end: the server returns the patches we
            // are behind on so a client can catch up. Applying them is what
            // turns a repeat conflict into a successful next write.
            if let Some(reply) = &reply {
                applied += catch_up(store, reply)?;
            }
        }
    }
    Ok(applied)
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
fn catch_up(store: &crate::store::ProtocolStore, reply: &Node) -> Result<usize> {
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
    let Some(patches) = conflicted.child("patches") else {
        return Ok(0);
    };
    // The collection the conflict is about, as the server names it.
    let Some(collection) = conflicted
        .get_attr("name")
        .and_then(|v| v.as_text())
        .and_then(Collection::from_name)
    else {
        return Err(crate::Error::invalid(
            "app_state_sync",
            "conflict names no known collection",
        ));
    };
    let mut state = held_state(store, collection)?;
    let mut applied = 0;
    for node in patches.child_nodes() {
        if node.description != "patch" {
            continue;
        }
        let NodeContent::Bytes(bytes) = &node.content else {
            continue;
        };
        let decoded = CollectionPatch::decode(bytes)?;
        let verified = store.write(|tx| {
            let keys = app_state::keys_for(tx, &decoded.key_id)?.ok_or_else(|| {
                crate::Error::invalid("app_state_sync", "conflict names a key we do not hold")
            })?;
            state.verify_patch(&decoded, |id| {
                if id == &decoded.key_id {
                    Ok(&keys)
                } else {
                    Err(crate::Error::invalid("app_state_sync", "unexpected key id"))
                }
            })
        })?;
        state = verified.state;
        applied += 1;
    }
    if applied > 0 {
        let saved = state.to_saved();
        store.write(|tx| {
            tx.put_json(
                Table::Checkpoints,
                held_key(collection).as_bytes(),
                1,
                &saved,
            )
        })?;
        tracing::info!(
            patches = applied,
            version = state.version(),
            "app state: caught up from a conflict"
        );
    }
    Ok(applied)
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
    fn patch_request_refuses_invalid_inputs() {
        assert!(patch_request("", Collection::Regular, Some(1), &[1], false).is_err());
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
