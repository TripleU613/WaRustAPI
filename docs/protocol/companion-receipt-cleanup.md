# Authenticated companion receipt cleanup

2026-10-09. **Implemented/offline tested.** Evidence is APK
2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
No new native claim or undocumented server prerequisite is used.

## Confirmed APK behavior and exact port

D4x.A00 082a..0848 supplies the changed device singleton to 0Z4.A07;
its concrete ordinary/add-on stores call 0Z5.A0B. A0B 0034..0040 excludes
primary devices. Its DELETE 008c..0116 selects the device-row set and
receipt_device_timestamp IS NULL, without a message-state or user-row
intersection. It therefore preserves timestamped rows. See the
[existing full branch](apk/2.26.37.71-identity-receipts/device-receipt-store-cleanup.txt)
and [cleanup predicates](identity-receipts.md#counterparts-versions-and-companion-cleanup).
Primary cleanup is a different query and remains unported.

0Z5.A0A 031c..0366 updates an existing row's timestamp; only zero matching
rows causes INSERT with the current nullable user version. See
[version evidence](receipt-version.md#updates-preserve-an-existing-rows-version)
and [earliest-positive clock](receipt-clock.md#earliest-positive-timestamp-and-columns).
Thus deletion cannot be represented as permanent refusal of later receipts,
or late insertion as restoration of the original at-send version.

**Implemented:** archive_identity calls the shared companion cleanup hook only
after a previous key differs from the authenticated replacement. Incoming
message and signed-directory sources use the same hook, retaining their distinct
history authentication labels. First-seen and unchanged keys do not clean rows.
The hook normalizes wire encoding, requires nonempty PN/LID user, agent zero
and nonzero device, and removes only pending exact-device membership of existing
ordinary direct/group outbox rows. It leaves other companions, primary devices,
received rows and other user namespaces alone. It does not consult an arbitrary
contact alias. Explicit status/broadcast/coexistence branches remain separate.

## Membership is separate from immutable send provenance

Original outbox intent, request/wire IDs, attempts, payload, progress and facts
are unchanged. outbox-primary-versions remains immutable. Every original target
keeps a provenance cell in outbox-device-receipts, with present=false after the
pending companion row is removed. This is local ledger representation, not a
new wire field or a native SQL tombstone. Absent cells have null timestamp and
are excluded from current row membership. No current resend consumer is enabled.

Old instrumented rows without present/reinserted metadata default to actually
present/non-reinserted: their old writer always created a full original set.
Historical missing instrumentation remains absent and is never reconstructed.
The decoder still requires exact full original device coverage, with no
unplanned/duplicate/primary removed cells. Non-reinserted versions must match
original capture. Reinsertion metadata requires a present companion with a
positive timestamp; it allows the current nullable version to differ from the
original. Malformed records refuse with fixed redacted reasons, not repair.

A later authenticated, attributed supported positive root receipt for a removed
original companion inserts membership again with the then-current nullable
exact-namespace user version and its positive server timestamp. Further earlier
timestamps can improve that row without replacing its inserted version. Invalid
or nonpositive times leave removed membership absent. Other devices cannot
create rows merely by sending an unrelated receipt. This port does not change
existing native-status admission limits documented in device-receipt-rows.

All changes share the identity/session/archive transaction, so failure rolls
back cleanup, history and journal. There is no additional cache or post-commit
callback in Rust's encrypted ledger representation. Native selected-row/full
cache effects remain separately documented; no FIFO guarantee is invented.
Checkpoint scans use bounded 256-record pages within the same transaction, with
no network query. A malformed known ledger can refuse an identity commit;
that is our local consistency refusal, not a WhatsApp challenge or refusal.

## Offline inspection and unported scope

```sh
whatsapp-cli --profile l --json outbox-versions --local-id ORIGINAL_LOCAL_ID
```

Each device_receipts item exposes present and version alongside immutable
at_send, original hash/component and nullable server-millisecond timestamp.
version is null when absent; present distinguishes a removed row from a present
row with unknown version. No raw address/key/content is printed. This existing
CLI/library contract remains x-live=false, with no served HTTP route.

**Unknown/unported:** primary pending/user-row intersection, PN/LID counterpart
migration and row authority, status/coexistence cleanup, all per-user admission,
full revoke projection, new-device receipts beyond captured targets, native
cache-mode integration and sync/key-fetch/final resend recovery. The exact
ordinary companion branch is one required part of that lifecycle; it does not
establish full native receipt or identity parity. No original reservation is
cleared and no automatic chat resend is introduced.

## Live regression and terminal diagnostics

**Observed:** a bounded receive-only regression authenticated successfully, with
chat sends and reconnects disabled, then ended before the requested 45-second
window completed. Exit code was 1. The previous generic CLI message said only
that reconnect bounds were exhausted or the failure was not recoverable; it
lost the runtime's actual reason by shutting down before reporting state. That
wording cannot establish recoverable loss, refusal, revocation or a protocol
regression. No retry was made on this unknown outcome.

Offline inspection found unchanged selected peer state, 20 actions, nine receipt
records, zero eligible outbox intents and unchanged event summary. Only six
additional inbound checkpoints were counted (798 to 804; total checkpoints
1,039 to 1,045). Eleven redacted incoming log lines were Unsupported ib or
psa/encrypt/mex notifications, with no delivery or read confirmation. Repeated
capture shapes may share one checkpoint; neither count means user replies.
The saved stream:error shape has a notification/w ACK child, ID presence and
no to attribute, but terminal_observation=unknown. It is historical and does
not prove the cause of this window's ending. No fresh terminal reason or
qualifying companion transition was established.

**Corrected locally:** listen now snapshots terminal state before shutdown and
reports fixed terminal_state/cause labels, with an offline inspection hint.
Known actor reason labels distinguish server-ended session, peer close, transport
failure, timeout and cancellation/worker stop; unknown reason text is omitted.
Needs-registration, revoked and stopped remain distinct states. This changes
neither runtime recoverability classification nor reservations/retry bounds.
Tests cover the labels and secret-bearing unknown reason redaction. A future
bounded session can now diagnose its actual terminal class; no live result is
claimed for this diagnostic correction, and this turn did not retry the loss.

Validation: all 810 workspace/all-feature tests passed (API 677, property 4,
crash matrix 9, CLI 52, API-symbol 1, xtask 67). Strict workspace/all-target/
all-feature Clippy, warning-denying Rustdoc, formatting, default CLI tests,
CLI-reference regeneration, API/library symbols and experimental CLI build
passed. Strict Clippy initially rejected redundant test qualifications; those
were removed and lint plus the focused diagnostic test reran successfully.
Hygiene checked 1,447 local links and 36 bundles / 8,929,026 bytes.

The group lifecycle regression and direct legacy-row regression cover both
ordinary chat branches, paged lookup past unrelated checkpoints, first-seen /
unchanged-key exclusion, both authenticated archive sources, exact namespace
selection, received/primary/agent preservation, rollback, unchanged original
outbox/snapshots and uncertain receipt reservation, late insertion with current
version, earliest-positive subsequent time without version replacement, corrupt
state refusal and restart. The live session authenticated but ended early with
unknown cause, so neither a clean full window nor a live companion transition
is claimed. No new intent, resend, identity reset or wait bypass was performed.
