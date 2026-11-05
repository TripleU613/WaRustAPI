# Trusted-contact successful issue state: captured time and monotonic updates

**Confirmed static**, APK 2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No live account operations or target execution. The current A0V export was
rechecked byte-identical to the retained [success-store](../2.26.43.72-privacy-token-callback/success-store.txt).
[8R8.CK4 selector3](../2.26.43.72-privacy-token-callback/result.txt) captures
user and request timestamp, calls A0V and then completes the job future.
It does not substitute response attributes or current wall time.

## Native persistence selection

0ht.A0V's first conditional UPDATE matches sent_tc_token_timestamp <= captured.
It sets sent_tc_token_timestamp=captured and real_issue_timestamp=NULL. If no
row matches, a second UPDATE matches nonnull real_issue_timestamp <= captured
and sets only that real timestamp. Otherwise INSERT with conflict mode4
(IGNORE) supplies JID and sent timestamp, with no real timestamp. A newer
schedule must therefore survive an older successful callback, while its
nonnull real timestamp can advance. A newer real timestamp or already-null
value survives an old callback; duplicate action settlement adds no callback.
A0V then evicts the canonical cache and may delete a counterpart row separately.
This port covers the selected direct normalized-user timestamp state; alias
canonicalization, counterpart deletion and full cache policy remain separate.

Current FKt.ATc declares the send-table columns. The retained 0Me.A00 shows
column-builder state/default fields reset after each column; the real column
is supplied without an explicit value/default in the A0V insertion. The SQL
fixture uses a nullable integer column, and is a focused selected-statement
oracle, not an Android schema migration or runtime acceptance test.

0ht.A0W, retained here in full, does not change an equal/newer saved schedule.
On a new schedule it carries a nonnegative decoded issue timestamp. Current
0hv.A08 projects raw SQL NULL to the previous sent timestamp, raw zero to no
issue timestamp, and nonzero values to themselves. A0W carries the nonnegative
projection; absent/negative projection or a missing row becomes explicit zero.
Thus a known completed NULL row preserves the prior sent time on rescheduling,
whereas an explicit zero stays unissued. row-decoder.txt retains this distinction.
The current scheduling fallback and configured-admission gaps still apply.

## Rust change and inspection

**Implemented offline:** SendState now carries nullable real_issue metadata.
Missing state schedules use Some(0). Advancing known state carries the decoded
nonnegative issue timestamp, including prior sent time for known raw NULL.
real_issue_known distinguishes this from old Rust records that never persisted
completion metadata. A legacy record without the fields is unknown history;
its next schedule initializes zero without claiming historical completion. Historical Ok rows are not retroactively
converted into success-store mutations or reconstructed live callbacks.

A correlated typed privacy-token outer result uses the frozen reserved request's
user and timestamp. Conditional state changes, saved completion effect, reply
and action settlement commit in the same encrypted-store transaction. No write
is performed for an unchanged timestamp row. Missing state is inserted from
captured data. Old callbacks never replace the newer schedule owner or refresh
its request bytes. Refusal/expiry/disconnect do not execute the success update.
A local failure after a timestamp update rolls the entire transaction back;
it leaves the uncertain Sent reservation in place without replay.

Existing actions plain/JSON includes optional token_issue_completion with:
- scheduled_timestamp_confirmed: captured time confirms/advances schedule and clears real metadata;
- earlier_issue_timestamp_advanced: older callback advances real time only;
- inserted_completed_state: captured completion inserts missing state;
- no_timestamp_change: no supported change; newer/completed or unknown legacy state is preserved.
These are local selected-state effects, not a server-delivery claim, raw timestamp,
user identifier, effective bucket policy or authorization to retry. Unrelated,
legacy and unsuccessful actions omit the field. No new command or HTTP route.

## Verification and remaining gaps

derive-timestamp-vectors.py independently executes the selected SQL statements
against synthetic in-memory SQLite rows. timestamp-vectors.json covers missing,
equal/older/newer scheduled times, null/zero/negative/equal/newer real times.
Rust tests feed these independent vectors through real action reservation and
reply settlement for PN/LID, verify frozen wire, duplicate no-op and disk reopen.
A reordered pair preserves a newer queued owner, records the older real time,
then clears it when the current request completes. Scheduling vectors separately replay the cursor CASE projection and carry rule,
including known NULL-to-prior-sent, zero, negative and positive raw values.
Legacy unknown history stays distinct. Fault tests cover damaged
state and a later settlement-sequence failure: neither can partially commit a
completion. The existing32s callback test confirms expiry/refusal do not record
completion, while outer result with nested error still follows native success.

Reproduce current methods with existing xtask apk-smali APK --class CLASS
--method METHOD --signatures --exceptions: 0ht.A0W, 0hv.A08, FKt.ATc, 0Me.A00.
Previous bundle retains full 0ht.A0V and callback. Python fixture regeneration
must match committed JSON exactly. Checksums cover every other file here.

Effective configured expiry/scheduling, batch/counterpart mappings, native job
retry/cancellation and complete success-cache behavior remain unfinished. Saving
this direct timestamp state does not establish full native token policy.
