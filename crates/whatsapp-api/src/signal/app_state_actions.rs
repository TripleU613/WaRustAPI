//! Building app-state mutations for specific actions.
//!
//! All numbering and routing here is read from comparison APK 2.26.37.71.
//! Each value class builds its index as a Java `String[]` in its constructor,
//! which `LX/Azp->A0B` returns and `LX/C8y->A00` renders with
//! `JSONArray(String[]).toString()` — compact, no spaces between elements. That
//! is why the index is assembled as a string here rather than through a JSON
//! serializer that might pretty-print or reorder.
use crate::{Error, Result};
use prost::Message;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("app_state_action", reason)
}

/// `SyncActionValue` (`LX/BZV`): timestamp is field 1, and each action occupies
/// its own field number.
#[derive(Message)]
struct SyncActionValue {
    #[prost(int64, optional, tag = "1")]
    timestamp: Option<i64>,
    #[prost(message, optional, tag = "3")]
    contact_action: Option<ContactAction>,
    /// `pushNameSetting` is field **7** (`xtask apk-proto-fields` on `LX/BZV`).
    #[prost(message, optional, tag = "7")]
    push_name_setting: Option<PushNameSetting>,
}

/// `PushNameSetting` (`LX/BQd`): one field, `name` = 1.
#[derive(Message)]
struct PushNameSetting {
    #[prost(string, optional, tag = "1")]
    name: Option<String>,
}

/// `contactAction` (`LX/BXZ`). `LX/B6M->A05` sets `fullName_` (presence bit
/// 0x1) and `saveOnPrimaryAddressbook_` (bit 0x8, i.e. the fourth field), so
/// those are the two a real client populates for a saved contact.
#[derive(Message)]
struct ContactAction {
    #[prost(string, optional, tag = "1")]
    full_name: Option<String>,
    #[prost(string, optional, tag = "2")]
    first_name: Option<String>,
    #[prost(string, optional, tag = "4")]
    save_on_primary_addressbook: Option<String>,
}

/// One action's collection, index and schema version, as the APK routes it.
pub struct ActionMutation {
    /// Exact index JSON bytes, MAC'd verbatim.
    pub index: Vec<u8>,
    /// `SyncActionValue` protobuf.
    pub value: Vec<u8>,
    /// `SyncActionData.version` (field 4).
    pub version: i32,
}
impl std::fmt::Debug for ActionMutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ActionMutation(<redacted>)")
    }
}

/// Render an index array exactly as `JSONArray(String[]).toString()` does:
/// `["a","b"]` with no spaces. Only the escapes JSON requires are applied; a
/// JID or name containing a quote or backslash would otherwise break the array.
fn index_json(parts: &[&str]) -> Result<Vec<u8>> {
    if parts.is_empty() || parts.iter().any(|p| p.is_empty()) {
        return Err(invalid("index needs nonempty parts"));
    }
    let mut out = String::from("[");
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('"');
        for c in part.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                c if (c as u32) < 0x20 => return Err(invalid("control character in index")),
                c => out.push(c),
            }
        }
        out.push('"');
    }
    out.push(']');
    Ok(out.into_bytes())
}

/// Build a `contactAction` mutation for a saved contact.
///
/// Routing and numbering, all **confirmed**:
/// * Collection is **`critical_unblock_low`** — `LX/Azs->A00` is a packed
///   switch on the action ordinal, and contactAction (ordinal 1) maps to
///   `LX/Azr.A05`. This is counter-intuitive (a contact is not a block-list
///   entry), which is exactly why it is read rather than assumed.
/// * Index is two elements, `["contact", <jid raw string>]`
///   (`LX/B6M.<init>` `007a..0098`: it reads `Azq.A0O.value` = `"contact"` and
///   `Jid.getRawString()` into a two-element `String[]`).
/// * `SyncActionData.version` is **2** (`LX/B6M.<init>` `const/4 v9, #0x2`).
///
/// **Inferred, not confirmed:** that the JID in the index is the contact's
/// phone JID. The field is typed `UserJid`, so it is user-level rather than
/// device-addressed, but `LX/BXZ` carries both `lidJid` and `pnJid`, and the
/// enqueue call site that decides which one is passed was not located. The
/// phone JID is used because that is how this client's address book identifies
/// a contact. If it turns out to be the LID, the mutation lands under a
/// different index and a later SET corrects it — it does not corrupt anything.
pub fn contact_mutation(
    jid: &crate::model::jid::Jid,
    full_name: &str,
    first_name: Option<&str>,
    save_on_primary_addressbook: bool,
    timestamp_ms: i64,
) -> Result<ActionMutation> {
    if full_name.is_empty() || full_name.len() > 512 || timestamp_ms < 0 {
        return Err(invalid("invalid contact action inputs"));
    }
    let raw = jid.to_string();
    Ok(ActionMutation {
        index: index_json(&["contact", &raw])?,
        value: SyncActionValue {
            timestamp: Some(timestamp_ms),
            push_name_setting: None,
            contact_action: Some(ContactAction {
                full_name: Some(full_name.to_owned()),
                first_name: first_name.map(str::to_owned),
                // The APK stores this as a set field rather than a bool; a real
                // client sets it alongside fullName.
                save_on_primary_addressbook: save_on_primary_addressbook.then(|| "true".to_owned()),
            }),
        }
        .encode_to_vec(),
        version: 2,
    })
}

/// Build a `pushNameSetting` mutation: the display name as the account's other
/// devices read it.
///
/// All **confirmed** from comparison 2.26.37.71:
/// * Index is one element, `["setting_pushName"]` — `LX/B66.<init>` puts
///   `LX/Azq.A1A.value` into a one-element `String[]`, and `LX/Azq.<clinit>`
///   names that entry `PushNameSetting` / `"setting_pushName"` (ordinal 5).
/// * `SyncActionData.version` is **1** (the `const/4 v8, #0x1` passed as
///   `LX/Azp.<init>`'s int argument).
/// * The value sets only `PushNameSetting.name_` (`LX/B66->A05`, presence bit
///   0x1) and stores it as `SyncActionValue.pushNameSetting_`.
/// * Collection `critical_block` (`LX/B66.<clinit>` → `LX/Azs->A00`).
pub fn push_name_mutation(name: &str, timestamp_ms: i64) -> Result<ActionMutation> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 || timestamp_ms < 0 {
        return Err(invalid("invalid push name inputs"));
    }
    Ok(ActionMutation {
        index: index_json(&["setting_pushName"])?,
        value: SyncActionValue {
            timestamp: Some(timestamp_ms),
            contact_action: None,
            push_name_setting: Some(PushNameSetting {
                name: Some(name.to_owned()),
            }),
        }
        .encode_to_vec(),
        version: 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::jid::Jid;

    /// Field numbers checked against a hand-assembled encoding rather than
    /// the struct above: SyncActionValue{1: ts, 7: {1: "Lab"}}.
    #[test]
    fn push_name_mutation_matches_the_traced_value_class() {
        let action = push_name_mutation("  Lab ", 5).unwrap();
        assert_eq!(action.index, br#"["setting_pushName"]"#);
        assert_eq!(action.version, 1);
        assert_eq!(
            action.value,
            [0x08, 0x05, 0x3a, 0x05, 0x0a, 0x03, b'L', b'a', b'b']
        );
        assert!(push_name_mutation(" ", 5).is_err());
        assert!(push_name_mutation("x", -1).is_err());
    }

    #[test]
    fn index_renders_like_java_jsonarray_tostring() {
        assert_eq!(
            index_json(&["contact", "a@b"]).unwrap(),
            br#"["contact","a@b"]"#
        );
        // No spaces, in order.
        assert_eq!(index_json(&["a", "b", "c"]).unwrap(), br#"["a","b","c"]"#);
        // Quotes and backslashes escape; control characters are refused.
        assert_eq!(index_json(&[r#"a"b"#]).unwrap(), br#"["a\"b"]"#);
        assert_eq!(index_json(&[r"a\b"]).unwrap(), br#"["a\\b"]"#);
        assert!(index_json(&["a\nb"]).is_err());
        assert!(index_json(&[]).is_err());
        assert!(index_json(&["a", ""]).is_err());
    }

    #[test]
    fn contact_mutation_uses_the_traced_index_and_version() {
        let jid = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let m = contact_mutation(&jid, "Alice", Some("Al"), true, 1_700_000_000_000).unwrap();
        assert_eq!(m.index, br#"["contact","15551234567@s.whatsapp.net"]"#);
        assert_eq!(m.version, 2, "LX/B6M.<init> passes 2");
        // The value must carry a timestamp, or the reader refuses the SET.
        let fields = crate::signal::direct::payload_fields(&m.value).unwrap();
        assert!(fields.contains(&1), "timestamp is required for a SET");
        assert!(fields.contains(&3), "contactAction is field 3");
        assert!(contact_mutation(&jid, "", None, true, 1).is_err());
        assert!(contact_mutation(&jid, "Alice", None, true, -1).is_err());
    }
}
