//! Redacted offline inspection of the library's APK-derived row selector.
use std::{collections::BTreeSet, fs::File, io::Read, path::Path};

use serde::Deserialize;
use whatsapp_api::transport::jid_map::{JidMapRow, select_account_row};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    jid_row_id: i64,
    rows: Vec<JidMapRow>,
}

fn inspect(reader: impl Read) -> Result<serde_json::Value, String> {
    // Local inspection budgets, not APK/server limits.
    const MAX_BYTES: u64 = 64 * 1024;
    let mut bytes = Vec::new();
    reader
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read JID-map input".to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("JID-map input exceeds local 64 KiB limit".into());
    }
    let input: Input = serde_json::from_slice(&bytes).map_err(|error| {
        // Name only known schema fields, never private unknown keys or values.
        let message = error.to_string();
        for field in ["jid_row_id", "rows", "lid_row_id", "sort_id"] {
            for kind in ["missing", "duplicate"] {
                if message.starts_with(&format!("{kind} field `{field}`")) {
                    return format!("{kind} JID-map field: {field}");
                }
            }
        }
        "invalid JID-map input: expected jid_row_id and rows; each row needs signed 64-bit jid_row_id, lid_row_id and sort_id; unknown fields are rejected".to_string()
    })?;
    if input.rows.len() > 1024 {
        return Err("JID-map input exceeds local 1024-row limit".into());
    }
    let matched: Vec<_> = input
        .rows
        .iter()
        .filter(|row| row.jid_row_id == input.jid_row_id)
        .collect();
    let selected = select_account_row(&input.rows, input.jid_row_id);
    Ok(serde_json::json!({
        "input_rows": input.rows.len(),
        "matched_rows": matched.len(),
        "distinct_sort_ids": matched.iter().map(|row| row.sort_id).collect::<BTreeSet<_>>().len(),
        "selected": selected.is_some(),
        "selected_lid_row_nonnegative": selected.map(|id| id >= 0),
        "selected_lid_row_positive": selected.map(|id| id > 0),
        "runtime_mapping_applied": false
    }))
}

pub fn run(input: &Path) -> Result<(), String> {
    let file = File::open(input).map_err(|_| "cannot open JID-map input".to_string())?;
    let summary = inspect(file)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspection_selects_before_validating_and_does_not_echo_private_columns() {
        let summary = inspect(br#"{"jid_row_id":9234567,"rows":[{"jid_row_id":9234567,"lid_row_id":8765432,"sort_id":1},{"jid_row_id":9234567,"lid_row_id":-1,"sort_id":2}]}"#.as_slice()).unwrap();
        assert_eq!(summary["matched_rows"], 2);
        assert_eq!(summary["selected"], true);
        assert_eq!(summary["selected_lid_row_positive"], false);
        assert!(!summary.to_string().contains("9234567"));
        assert!(!summary.to_string().contains("8765432"));
        assert_eq!(summary["runtime_mapping_applied"], false);
    }

    #[test]
    fn missing_and_zero_candidates_are_distinct() {
        let missing = inspect(br#"{"jid_row_id":1,"rows":[]}"#.as_slice()).unwrap();
        assert_eq!(missing["selected"], false);
        assert!(missing["selected_lid_row_nonnegative"].is_null());
        let zero = inspect(
            br#"{"jid_row_id":1,"rows":[{"jid_row_id":1,"lid_row_id":0,"sort_id":0}]}"#.as_slice(),
        )
        .unwrap();
        assert_eq!(zero["selected"], true);
        assert_eq!(zero["selected_lid_row_nonnegative"], true);
        assert_eq!(zero["selected_lid_row_positive"], false);
    }

    #[test]
    fn inspection_refuses_malformed_unknown_and_oversized_inputs_without_echo() {
        for input in [
            r#"{"jid_row_id":1,"rows":[],"private-secret":true}"#,
            r#"{"jid_row_id":1,"rows":[{"jid_row_id":1,"lid_row_id":2,"sort_id":0,"private-secret":true}]}"#,
            r#"{"jid_row_id":1,"rows":[{"jid_row_id":1,"lid_row_id":9223372036854775808,"sort_id":0}]}"#,
            r#"{"jid_row_id":1,"rows":[{"jid_row_id":1,"lid_row_id":2,"sort_id":1.5}]}"#,
        ] {
            let error = inspect(input.as_bytes()).unwrap_err();
            assert!(error.starts_with("invalid JID-map input"));
            assert!(!error.contains("private-secret"));
        }
        assert_eq!(
            inspect(br#"{"jid_row_id":1}"#.as_slice()).unwrap_err(),
            "missing JID-map field: rows"
        );
        assert_eq!(
            inspect(br#"{"jid_row_id":1,"rows":[{"jid_row_id":1,"lid_row_id":2}]}"#.as_slice())
                .unwrap_err(),
            "missing JID-map field: sort_id"
        );
        assert_eq!(
            inspect(br#"{"jid_row_id":1,"jid_row_id":2,"rows":[]}"#.as_slice()).unwrap_err(),
            "duplicate JID-map field: jid_row_id"
        );
        assert!(
            inspect(vec![b' '; 65537].as_slice())
                .unwrap_err()
                .contains("64 KiB")
        );
        let rows = vec![serde_json::json!({"jid_row_id":1,"lid_row_id":2,"sort_id":0}); 1025];
        let bytes = serde_json::to_vec(&serde_json::json!({"jid_row_id":1,"rows":rows})).unwrap();
        assert!(inspect(bytes.as_slice()).unwrap_err().contains("1024-row"));
    }
}
