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

/// Build one patch request. `patch` is `SyncdPatch` wire bytes from
/// `CollectionState::build_patch`, and `version` is the version that patch
/// claims, i.e. the collection's current version plus one.
///
/// `return_snapshot` is honoured only when `version` is 0, because `CSe.A01`
/// emits the attribute solely when its boolean argument is true **and** the
/// version argument is null or zero (**confirmed**). A patch always claims a
/// nonzero version, so in practice this attribute belongs to a fetch rather
/// than to a patch send; asking for it here is silently not emitted rather
/// than producing a shape the APK never sends.
pub fn patch_request(
    id: &str,
    collection: Collection,
    version: u64,
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
        .attr("order", collection.order().to_string().as_str())
        .attr("version", version.to_string().as_str());
    if return_snapshot && version == 0 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::NodeContent;

    #[test]
    fn patch_request_matches_the_traced_shape() {
        let node = patch_request("ABC123", Collection::RegularHigh, 4, &[1, 2, 3], false).unwrap();
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
        let asked = patch_request("X", Collection::Regular, 1, &[9], true).unwrap();
        assert!(
            asked.child_nodes()[0].child_nodes()[0]
                .get_attr("return_snapshot")
                .is_none()
        );
        let at_zero = patch_request("X", Collection::Regular, 0, &[9], true).unwrap();
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

    #[test]
    fn patch_request_refuses_invalid_inputs() {
        assert!(patch_request("", Collection::Regular, 1, &[1], false).is_err());
        assert!(patch_request("A", Collection::Regular, 1, &[], false).is_err());
    }

    /// The encoder must be able to carry this node, since a patch is raw bytes
    /// inside an element whose name is not a dictionary token in every build.
    #[test]
    fn the_request_round_trips_through_the_comparison_codec() {
        use crate::binary::{Decoder, Encoder, TokenDictionary};
        let node = patch_request("RT1", Collection::RegularHigh, 2, &[7; 64], false).unwrap();
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
