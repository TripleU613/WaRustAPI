//! Offline selection of APK JID-map rows. Mapping authority and runtime writes
//! remain separate: a row selector does not authenticate a PN/LID relationship.

use serde::{Deserialize, Serialize};

/// Signed SQLite/Java-long columns from an existing `jid_map` row.
/// `sort_id` is an ordering value; its timestamp semantics are not established.
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JidMapRow {
    /// Phone-user row referenced by the APK `jid_row_id` predicate.
    pub jid_row_id: i64,
    /// Account-user row referenced by the APK `lid_row_id` column.
    pub lid_row_id: i64,
    /// Signed ordering key; its timestamp meaning is unestablished.
    pub sort_id: i64,
}

impl std::fmt::Debug for JidMapRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JidMapRow([REDACTED])")
    }
}

/// Select the account row for one phone row, matching APK 2.26.37.71
/// `0Xx.A01`/`A00` SQL and `0eU.<init>`/`A00`.
///
/// SQL visits descending sort/LID IDs, but the reverse TreeMap stores only one
/// LID per sort and overwrites ties. The winner therefore has the greatest sort
/// and the *smallest* LID at that sort. Preserve zero/negative candidates here:
/// `0Xw.A0C` and bulk `A08` apply different validity checks afterward. Dropping
/// an invalid winning row and selecting a lower-ranked row would change behavior.
/// No cache, address materialization, mapping freshness or identity mutation is
/// implied. Input order is irrelevant to the returned row ID.
pub fn select_account_row(rows: &[JidMapRow], jid_row_id: i64) -> Option<i64> {
    rows.iter()
        .filter(|row| row.jid_row_id == jid_row_id)
        .min_by_key(|row| (std::cmp::Reverse(row.sort_id), row.lid_row_id))
        .map(|row| row.lid_row_id)
}

#[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
/// Match primary direct namespaces using only saved authenticated association.
/// This does not adopt a pin, change an alias or prove outgoing delivery.
pub(super) fn authenticated_direct_chat(
    tx: &crate::store::WriteTx<'_>,
    from: &crate::model::jid::Jid,
    original: &crate::model::jid::Jid,
) -> crate::Result<bool> {
    if from.user.is_empty()
        || original.user.is_empty()
        || !matches!(
            from.kind,
            crate::model::jid::JidKind::Plain | crate::model::jid::JidKind::AgentDevice
        )
        || !matches!(
            original.kind,
            crate::model::jid::JidKind::Plain | crate::model::jid::JidKind::AgentDevice
        )
        || from.agent != 0
        || from.device != 0
        || !matches!(
            from.server,
            crate::model::jid::Server::User | crate::model::jid::Server::Lid
        )
        || original.agent != 0
        || original.device != 0
        || !matches!(
            original.server,
            crate::model::jid::Server::User | crate::model::jid::Server::Lid
        )
    {
        return Ok(false);
    }
    // Compare address components, accepting both ordinary and zero AD encoding.
    // This local association guard is not the APK's gated mapping policy.
    let same = |a: &crate::model::jid::Jid, b: &crate::model::jid::Jid| {
        use crate::model::jid::JidKind;
        matches!(a.kind, JidKind::Plain | JidKind::AgentDevice)
            && matches!(b.kind, JidKind::Plain | JidKind::AgentDevice)
            && a.user == b.user
            && a.server == b.server
            && a.agent == b.agent
            && a.device == b.device
    };
    let mut attributed = same(original, from);
    let alias_pair = match (&from.server, &original.server) {
        (crate::model::jid::Server::Lid, crate::model::jid::Server::User) => Some((from, original)),
        (crate::model::jid::Server::User, crate::model::jid::Server::Lid) => Some((original, from)),
        _ => None,
    };
    if !attributed
        && let Some((lid, expected_phone)) = alias_pair
        && let Some(value) = tx.get_json::<serde_json::Value>(
            crate::store::Table::Checkpoints,
            format!("authenticated-lid:{}", lid.to_non_ad()).as_bytes(),
            1,
        )?
    {
        let phone: crate::model::jid::Jid = serde_json::from_value(value)
            .map_err(|_| crate::Error::invalid("inbox", "invalid authenticated LID mapping"))?;
        if phone.user.is_empty()
            || phone.server != crate::model::jid::Server::User
            || phone.agent != 0
            || phone.device != 0
            || !matches!(
                phone.kind,
                crate::model::jid::JidKind::Plain | crate::model::jid::JidKind::AgentDevice
            )
        {
            return Err(crate::Error::invalid(
                "inbox",
                "invalid authenticated LID mapping",
            ));
        }
        attributed = same(expected_phone, &phone);
    }
    Ok(attributed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Fixture {
        cases: Vec<Case>,
    }

    #[derive(Deserialize)]
    struct Case {
        name: String,
        jid_row_id: i64,
        rows: Vec<JidMapRow>,
        selected_lid_row_id: Option<i64>,
    }

    #[test]
    fn selection_matches_independent_sqlite_cursor_and_reverse_map_fixtures() {
        let fixture: Fixture = serde_json::from_str(include_str!(
            "../../../../docs/protocol/jid-map-row-selection-fixtures.json"
        ))
        .unwrap();
        assert!(fixture.cases.len() >= 30);
        for case in fixture.cases {
            assert_eq!(
                select_account_row(&case.rows, case.jid_row_id),
                case.selected_lid_row_id,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn row_debug_omits_private_columns() {
        let row = JidMapRow {
            jid_row_id: 9234567,
            lid_row_id: 8765432,
            sort_id: 9876543,
        };
        assert_eq!(format!("{row:?}"), "JidMapRow([REDACTED])");
    }
}
