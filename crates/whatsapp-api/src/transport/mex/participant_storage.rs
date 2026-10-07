//! APK 37.71 0yw.A00 member storage scalars, not whole membership reconciliation.
//! Evidence: docs/protocol/mex-participant-storage-evidence.txt and mex.md.

/// Private participant storage metadata. Debug omits values; no implicit export.
#[derive(PartialEq, Eq)]
pub struct ParticipantStorageMetadata {
    /// 1uU.A01 milliseconds, after raw join seconds/previous state/clock selection.
    pub join_time_ms: i64,
    /// 1uU.A02 integer state; unknown higher/nonzero prior states are preserved.
    pub history_state: Option<i32>,
}
impl std::fmt::Debug for ParticipantStorageMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParticipantStorageMetadata")
            .finish_non_exhaustive()
    }
}

/// APK member-storage selection with caller-supplied previous state and clock.
/// Raw join_time is the nullable Long in 31c, not an already scaled timestamp.
/// The normal ADv producer supplies positive optInt widened to Long. This helper
/// also reproduces the consumer's handling of absent/nonpositive/wider Longs.
/// No wall-clock read, account access, alias reconciliation or durable write.
pub fn participant_storage_metadata(
    join_time: Option<i64>,
    group_history_sent: Option<bool>,
    previous_join_time_ms: Option<i64>,
    previous_history_state: Option<i32>,
    clock_ms: i64,
) -> ParticipantStorageMetadata {
    // 0yw.A00+0112..013e: positive 31c.A05 Long multiplied by literal 1000.
    // +0336..035a: otherwise preserve positive 1uU.A01, else 08B.A00 clock.
    // Selection occurs before multiplication; overflow does not trigger fallback.
    let join_time_ms = join_time
        .filter(|value| *value > 0)
        .map(|value| value.wrapping_mul(1000))
        .unwrap_or_else(|| {
            previous_join_time_ms
                .filter(|value| *value > 0)
                .unwrap_or(clock_ms)
        });
    // 0bA.A00: true raises null/<2 to 2; false raises null/0 to 1;
    // null returns previous unchanged. False preserves even negative states.
    let history_state = match group_history_sent {
        Some(true) => Some(previous_history_state.unwrap_or(2).max(2)),
        Some(false) => Some(
            previous_history_state
                .filter(|state| *state != 0)
                .unwrap_or(1),
        ),
        None => previous_history_state,
    };
    ParticipantStorageMetadata {
        join_time_ms,
        history_state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Inputs {
        join_time: Option<i64>,
        group_history_sent: Option<bool>,
        previous_join_time_ms: Option<i64>,
        previous_history_state: Option<i32>,
        clock_ms: i64,
    }
    #[derive(Deserialize)]
    struct Expected {
        join_time_ms: String,
        history_state: Option<i32>,
    }
    #[derive(Deserialize)]
    struct Case {
        input: Inputs,
        expected: Expected,
    }
    #[derive(Deserialize)]
    struct Fixture {
        cases: Vec<Case>,
    }

    #[test]
    fn storage_selection_matches_executed_java_branch_reference() {
        let fixture: Fixture = serde_json::from_str(include_str!(
            "../../../fixtures/mex/2.26.37.71-participant-storage-reference.json"
        ))
        .unwrap();
        assert_eq!(fixture.cases.len(), 432);
        for (index, case) in fixture.cases.into_iter().enumerate() {
            let input = case.input;
            let actual = participant_storage_metadata(
                input.join_time,
                input.group_history_sent,
                input.previous_join_time_ms,
                input.previous_history_state,
                input.clock_ms,
            );
            assert_eq!(
                actual.join_time_ms.to_string(),
                case.expected.join_time_ms,
                "case {index}"
            );
            assert_eq!(
                actual.history_state, case.expected.history_state,
                "case {index}"
            );
            assert_eq!(format!("{actual:?}"), "ParticipantStorageMetadata { .. }");
        }
    }
}
