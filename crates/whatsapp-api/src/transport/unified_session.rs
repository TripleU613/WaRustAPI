//! Unified-session startup stanza.
//!
//! APK 2.26.43.72 `X.0Sb.A02` (called from the selected startup block after the
//! push-config GET) builds `<ib><unified_session id="…"/></ib>` with no IQ
//! wrapper, request ID, namespace, type or destination. The `id` is `String`
//! from the app's offset-aware clock:
//!
//! ```text
//! String.valueOf((clock_ms + 0xf731400) % 0x240c8400)
//! ```
//!
//! `0xf731400` is three days and `0x240c8400` seven days in milliseconds
//! (`X.0Sb.A00`). `X.08q.A00` returns `elapsedRealtime + offset` when a saved
//! offset exists, otherwise `currentTimeMillis - offset`; this client uses the
//! same corrected-server-time concept through [`crate::transport::clock`] and
//! falls back to the local wall clock when no sample exists.
//!
//! The APK caches the value and refreshes it on its own producers; this client
//! computes it once per startup, which is the selected startup call. A null
//! value returns no node in the APK (uninitialized account); after login the
//! node is always produced. Server meaning, acknowledgement and refresh cadence
//! are not established by this trace.

use crate::{Result, binary::Node, model::time::Timestamp, store::ProtocolStore};

/// Three days in milliseconds (`0xf731400`).
pub const LEAD_MS: i64 = 259_200_000;
/// Seven days in milliseconds (`0x240c8400`).
pub const WINDOW_MS: i64 = 604_800_000;

/// The `id` value for a clock reading, matching the APK's signed long
/// arithmetic (`+` then `%`).
pub fn id_at(now_ms: i64) -> String {
    now_ms
        .saturating_add(LEAD_MS)
        .rem_euclid(WINDOW_MS)
        .to_string()
}

/// `<ib><unified_session id="…"/></ib>` for a clock reading.
pub fn node_at(now_ms: i64) -> Node {
    Node::new("ib").children(vec![Node::new("unified_session").attr("id", id_at(now_ms))])
}

/// The startup node using saved server-clock authority when available.
pub fn startup_node(store: &ProtocolStore) -> Result<Node> {
    let local_ms = Timestamp::now().as_millis();
    let now_ms = store
        .read(|tx| super::clock::server_now(tx, local_ms))?
        .unwrap_or(local_ms);
    Ok(node_at(now_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_matches_the_traced_formula() {
        // Independent evaluation of (t + 0xf731400) % 0x240c8400.
        assert_eq!(id_at(0), "259200000");
        assert_eq!(id_at(1), "259200001");
        assert_eq!(id_at(WINDOW_MS - LEAD_MS), "0");
        assert_eq!(id_at(WINDOW_MS - LEAD_MS + 1), "1");
        // Java's remainder is signed; a negative instant must not produce a
        // negative id, and the value always stays inside one window.
        for t in [-1, -LEAD_MS, i64::MIN + 1, 1_700_000_000_000] {
            let id = id_at(t);
            let value: i64 = id.parse().unwrap();
            assert!((0..WINDOW_MS).contains(&value), "{t} -> {id}");
        }
    }

    #[test]
    fn node_shape_has_no_iq_wrapper_or_extra_attributes() {
        let node = node_at(1_700_000_000_000);
        assert_eq!(node.description, "ib");
        assert!(node.attributes.is_empty());
        let children = node.child_nodes();
        assert_eq!(children.len(), 1);
        let session = &children[0];
        assert_eq!(session.description, "unified_session");
        assert_eq!(
            session.get_attr("id").and_then(|v| v.as_text()),
            Some(id_at(1_700_000_000_000).as_str())
        );
        // Only `id`; the APK's builder adds nothing else.
        assert_eq!(session.attributes.len(), 1);
    }
}
