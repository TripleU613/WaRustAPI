//! Participant-scoped local group records. APK 37.71 BZa/Bal retain the
//! original participant; this hashed selector is local storage, not a wire ID.
use crate::{
    Error, Result,
    binary::{Decoder, Node, TokenDictionary},
    model::jid::{Jid, Server},
    profile::Limits,
    store::{Table, WriteTx},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
fn invalid(reason: &'static str) -> Error {
    Error::invalid("group_inbox", reason)
}
fn legacy(chat: &Jid, id: &str) -> String {
    format!("received:{}:{id}", chat.to_non_ad())
}
/// Local selector binds exact group namespace, participant user namespace and
/// agent, and original wire ID. Device/kind representations of that same user
/// do not rename its logical message; the original route retains the device.
pub fn key(chat: &Jid, participant: &Jid, id: &str) -> Result<String> {
    if chat.server != Server::Group
        || !matches!(participant.server, Server::User | Server::Lid)
        || id.is_empty()
        || id.len() > 128
    {
        return Err(invalid("invalid group record scope"));
    }
    let scope = serde_json::to_vec(&(
        participant.user.as_str(),
        participant.server.as_str(),
        participant.agent,
        id,
    ))
    .expect("fixed tuple");
    Ok(format!(
        "received:{}:group-{}",
        chat.to_non_ad(),
        hex::encode(Sha256::digest(scope))
    ))
}
fn parse(raw: &[u8]) -> Result<Value> {
    serde_json::from_slice(raw).map_err(|_| invalid("invalid saved group record"))
}
fn route(saved: &Value, get: &mut impl FnMut(&[u8]) -> Result<Option<Vec<u8>>>) -> Result<Node> {
    let wire: Option<Vec<u8>> = if !saved["route"].is_null() {
        serde_json::from_value(saved["route"].clone()).ok()
    } else if let Ok(capture) = serde_json::from_value::<Vec<u8>>(saved["capture"].clone()) {
        get(&capture)?
            .map(|raw| {
                serde_json::from_slice(&raw).map_err(|_| invalid("invalid saved group capture"))
            })
            .transpose()?
    } else {
        None
    };
    let wire = wire.ok_or_else(|| invalid("saved group record lacks original route"))?;
    let limits = Limits::default();
    let raw = crate::binary::unpack_frame_payload(&wire, &limits)?;
    Decoder::new(&raw, TokenDictionary::comparison_2_26_37_71(), &limits)
        .decode_node()
        .map_err(Into::into)
}
fn scope(node: &Node) -> Option<(Jid, Jid, &str)> {
    if node.description != "message" {
        return None;
    }
    let (chat, participant) = super::inbox::group_sender(node)?;
    Some((chat, participant, node.get_attr("id")?.as_text()?))
}
/// Verify a canonical selector or an unchanged legacy group/id selector against
/// the original route. This function does not authorize decryption or receipt.
pub(super) fn matches(record_key: &[u8], node: &Node) -> bool {
    let Some((chat, sender, id)) = scope(node) else {
        return false;
    };
    record_key == legacy(&chat, id).as_bytes()
        || key(&chat, &sender, id).is_ok_and(|k| record_key == k.as_bytes())
}
pub(super) struct Selection {
    pub key: String,
    pub saved: Option<Value>,
    pub legacy_collision: bool,
    legacy_key: String,
    legacy_raw: Option<Vec<u8>>,
}
impl Selection {
    pub fn validate_in(&self, tx: &WriteTx<'_>) -> Result<()> {
        if tx.get(Table::Checkpoints, self.legacy_key.as_bytes(), 1)? != self.legacy_raw {
            return Err(invalid(
                "legacy group attribution changed during preparation",
            ));
        }
        Ok(())
    }
}
/// Read new and historical rows together. A different legacy participant does
/// not suppress a newly authenticated message; equal historical rows stay put.
pub(super) fn select(
    node: &Node,
    mut get: impl FnMut(&[u8]) -> Result<Option<Vec<u8>>>,
) -> Result<Selection> {
    let (chat, sender, id) =
        scope(node).ok_or_else(|| invalid("group message requires participant and id"))?;
    let canonical = key(&chat, &sender, id)?;
    let old = legacy(&chat, id);
    let new = get(canonical.as_bytes())?.map(|v| parse(&v)).transpose()?;
    if let Some(saved) = &new {
        let original = route(saved, &mut get)?;
        let (c, p, i) =
            scope(&original).ok_or_else(|| invalid("saved group route has no participant"))?;
        if key(&c, &p, i)? != canonical {
            return Err(invalid("saved canonical group attribution differs"));
        }
    }
    let legacy_raw = get(old.as_bytes())?;
    let historical = legacy_raw.as_ref().map(|raw| parse(raw)).transpose()?;
    let had_legacy = historical.is_some();
    let matching = if let Some(saved) = historical {
        let original = route(&saved, &mut get)?;
        let (c, p, i) =
            scope(&original).ok_or_else(|| invalid("legacy group route has no participant"))?;
        if c.to_non_ad() != chat.to_non_ad() || i != id {
            return Err(invalid("legacy group route differs from its selector"));
        }
        (key(&c, &p, i)? == canonical).then_some(saved)
    } else {
        None
    };
    if new.is_some() && matching.is_some() {
        return Err(invalid(
            "multiple records for the same group participant and id",
        ));
    }
    let legacy_collision = had_legacy && matching.is_none();
    let (key, saved) = if matching.is_some() {
        (old.clone(), matching)
    } else {
        (canonical, new)
    };
    Ok(Selection {
        key,
        saved,
        legacy_key: old,
        legacy_raw,
        legacy_collision,
    })
}
/// Select one explicit participant, or one unambiguous group/id pair for the
/// existing read API. Ambiguous user-level reads never choose the first row.
pub(super) fn read_target(
    tx: &WriteTx<'_>,
    chat: &Jid,
    id: &str,
    participant: Option<&Jid>,
) -> Result<(String, Value)> {
    if let Some(participant) = participant {
        let node = Node::new("message")
            .attr("from", chat.clone())
            .attr("participant", participant.clone())
            .attr("id", id);
        let selected = select(&node, |k| tx.get(Table::Checkpoints, k, 1))?;
        return selected
            .saved
            .map(|v| (selected.key, v))
            .ok_or_else(|| invalid("unknown incoming group participant target"));
    }
    let prefix = format!("received:{}:", chat.to_non_ad());
    let old = legacy(chat, id);
    let mut found = None;
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        if rows.is_empty() {
            break;
        }
        cursor = rows.last().map(|(k, _)| k.clone());
        for (k, raw) in rows {
            if !k.starts_with(prefix.as_bytes()) {
                continue;
            }
            let saved = parse(&raw)?;
            if k != old.as_bytes() && saved["wire_id"].as_str() != Some(id) {
                continue;
            }
            let original = route(&saved, &mut |k| tx.get(Table::Checkpoints, k, 1))?;
            let Some((c, _, i)) = scope(&original) else {
                return Err(invalid("saved group target lacks participant"));
            };
            if c.to_non_ad() != chat.to_non_ad() || i != id || !matches(&k, &original) {
                return Err(invalid("saved group target attribution differs"));
            }
            if found.is_some() {
                return Err(invalid("ambiguous group message id; specify participant"));
            }
            found = Some((
                String::from_utf8(k).map_err(|_| invalid("invalid group selector"))?,
                saved,
            ));
        }
    }
    found.ok_or_else(|| invalid("unknown incoming group target"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binary::Encoder,
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource},
    };
    #[test]
    fn legacy_attribution_is_preserved_and_concurrent_changes_roll_back() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProtocolStore::open(OpenOptions {
            path: dir.path().join("legacy.redb"),
            account_id: AccountId::new("fixture").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([29; 32]))),
            create: true,
        })
        .unwrap();
        let chat = Jid::parse("120363000000000001@g.us").unwrap();
        let alice = Jid::parse("15555550100:3@s.whatsapp.net").unwrap();
        let bob = Jid::parse("15555550101@lid").unwrap();
        let node = |p: &Jid| {
            Node::new("message")
                .attr("from", chat.clone())
                .attr("participant", p.clone())
                .attr("id", "SAME")
        };
        let route = Encoder::new(TokenDictionary::comparison_2_26_37_71())
            .encode(&node(&alice))
            .unwrap();
        // A saved legacy route is a selector fixture, not proof that a server
        // authenticated it. Real group decryption is exercised by inbox tests.
        let saved = serde_json::json!({"status":"decrypted", "route":route});
        let old = legacy(&chat, "SAME");
        store
            .write(|tx| tx.put_json(Table::Checkpoints, old.as_bytes(), 1, &saved))
            .unwrap();
        let before = store
            .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
            .unwrap();
        let same = store
            .read(|tx| {
                select(&node(&alice.with_device(0)), |k| {
                    tx.get(Table::Checkpoints, k, 1)
                })
            })
            .unwrap();
        assert_eq!(same.key, old);
        assert_eq!(same.saved, Some(saved.clone()));
        let other = store
            .read(|tx| select(&node(&bob), |k| tx.get(Table::Checkpoints, k, 1)))
            .unwrap();
        assert!(other.saved.is_none() && other.legacy_collision);
        assert_eq!(other.key, key(&chat, &bob, "SAME").unwrap());
        assert_ne!(
            key(&chat, &alice, "SAME").unwrap(),
            key(&chat, &bob, "SAME").unwrap()
        );
        assert_eq!(
            store
                .read(|tx| tx.scan(Table::Checkpoints, 1, 100))
                .unwrap(),
            before
        );
        let changed = serde_json::json!({"status":"consumed", "route":saved["route"]});
        store
            .write(|tx| tx.put_json(Table::Checkpoints, old.as_bytes(), 1, &changed))
            .unwrap();
        assert!(
            store
                .write(|tx| {
                    tx.put_json(Table::Checkpoints, b"must-rollback", 1, &true)?;
                    other.validate_in(tx)
                })
                .is_err()
        );
        assert!(
            store
                .read(|tx| tx.get(Table::Checkpoints, b"must-rollback", 1))
                .unwrap()
                .is_none()
        );
        // Duplicate historical and canonical claims are refused, never picked
        // by scan order or rewritten to erase uncertainty.
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    key(&chat, &alice, "SAME")?.as_bytes(),
                    1,
                    &saved,
                )
            })
            .unwrap();
        assert!(
            store
                .read(|tx| select(&node(&alice), |k| tx.get(Table::Checkpoints, k, 1)))
                .is_err()
        );
        // A historical row without the original route cannot establish sender.
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    old.as_bytes(),
                    1,
                    &serde_json::json!({"status":"decrypted"}),
                )
            })
            .unwrap();
        assert!(
            store
                .read(|tx| select(&node(&bob), |k| tx.get(Table::Checkpoints, k, 1)))
                .is_err()
        );
    }
}
