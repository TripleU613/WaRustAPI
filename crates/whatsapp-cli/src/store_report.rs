//! The `store inspect` report, shared by the CLI and the opt-in
//! `GET /v1/store` store route so both print the same redacted JSON.

use serde_json::{Value, json};
use whatsapp_api::Error;
use whatsapp_api::session::AccountRecord;
use whatsapp_api::store::{ProtocolStore, Table};

/// Longest checkpoint kind printed by name.
const MAX_KIND_LEN: usize = 48;

/// Row counts per table, the account's registration status, and the
/// Checkpoints table broken down by record kind. Checkpoints holds many
/// unrelated record kinds under one table, so a bare row count cannot answer
/// "is there an app-state key / a login identity / any captures". Only the
/// leading key segment is reported, and only when it looks structural
/// ([`structural_kind`]); the rest of a key is a JID, an ID or a digest and is
/// never printed. Secret-free: no key material, address, ID or content.
pub(crate) fn report(store: &ProtocolStore) -> Result<Value, Error> {
    store.read(|tx| {
        let mut counts = serde_json::Map::new();
        for t in Table::ALL {
            counts.insert(format!("{t:?}"), json!(tx.count(t)?));
        }
        let account = AccountRecord::load(tx)?;
        let mut kinds: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        let mut cursor = None;
        loop {
            let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 512)?;
            let done = rows.len() < 512;
            cursor = rows.last().map(|(key, _)| key.clone());
            for (key, _) in rows {
                let head = key.split(|byte| *byte == b':').next().unwrap_or_default();
                *kinds.entry(structural_kind(head).to_owned()).or_default() += 1;
            }
            if done {
                break;
            }
        }
        Ok(json!({
            "account_id": store.account_id().as_str(),
            "generation": store.generation().0,
            "journal_head": tx.journal_head()?.0,
            "registration": account
                .as_ref()
                .map(|a| format!("{:?}", a.registration))
                .unwrap_or_else(|| "no account record".into()),
            "tables": counts,
            "checkpoint_kinds": kinds,
        }))
    })
}

/// A leading key segment printed by name only when it is a lowercase record
/// name (`[a-z][a-z0-9_-]*`, bounded) that is not hex-like; anything else, for
/// example a key without a `:` separator whose whole text is an address or a
/// digest, is counted as `other`. The kind list is code-defined, so this never
/// hides a real kind: it guards against a key shape nobody audited.
fn structural_kind(head: &[u8]) -> &str {
    let Ok(text) = std::str::from_utf8(head) else {
        return "other";
    };
    let named = text.len() <= MAX_KIND_LEN
        && text.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_');
    let hex_like = text.bytes().all(|b| b.is_ascii_hexdigit());
    if named && !hex_like { text } else { "other" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_structural_kinds_are_named() {
        for kind in [
            "login-identity",
            "received",
            "inbound-undecodable",
            "receipt-outbox",
            "app_state",
        ] {
            assert_eq!(structural_kind(kind.as_bytes()), kind);
        }
        for raw in [
            "15551234567@s.whatsapp.net",
            "authenticated-lid.12345@lid",
            "deadbeef",
            "0123",
            "Upper",
            "",
            "a".repeat(MAX_KIND_LEN + 1).as_str(),
        ] {
            assert_eq!(structural_kind(raw.as_bytes()), "other", "{raw}");
        }
        assert_eq!(structural_kind(&[0xff, 0xfe]), "other");
    }
}
