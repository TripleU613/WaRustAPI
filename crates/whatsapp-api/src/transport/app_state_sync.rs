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
    use crate::signal::{app_state, app_state_actions, app_state_collection::CollectionState};
    use crate::transport::contacts;

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
    // This account has never synced, so its collections start at version 0; the
    // server's empty <sync/> reply to a fetch confirmed that.
    let state = CollectionState::empty(Collection::CriticalUnblockLow.name())?;
    let built = state.build_patch(keys.keys(), key_id, device_index.into(), &mutations)?;
    let id = super::actions::queue(store, PATCH_ACTION_KIND, "contact app-state patch", |id| {
        // Never synced, so no held version exists and the attribute is omitted.
        patch_request(
            id,
            Collection::CriticalUnblockLow,
            None,
            &built.bytes,
            false,
        )
    })?;
    Ok((id, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::NodeContent;

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
        let dict = TokenDictionary::comparison_2_26_37_70();
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
