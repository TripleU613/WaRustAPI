# Ordinary direct outbox account ownership

## Reproduced reply-admission gap (2026-11-07)

**Implemented/offline:** Pending retained local intent/query IDs and deadline,
but no account owner. prepare_with_policy used its caller's AccountRecord while
its store transactions checked intent/peer/query state only. A regression queued
a directory read under saved registration ID31, replaced it with32, and supplied
a same-ID outer error. The old implementation accepted it rather than returning
an ownership refusal. The pre-fix expected-error assertion failed. Its source
path could mark the queued intent failed under a different saved account.

A controlled local harness ran that regression against the previous committed
outgoing/connection source, then restored the changed files and retained the
test. No APK/server execution, account traffic or private fixture data was used.

## Frozen query owner and transactional checks

New directory query reservations capture the shared transport account fingerprint
in the same write transaction as their outcome_unknown row. Pending holds that
fingerprint. The durable private query metadata also retains account_authority;
identity suspension keeps it so a prospective selected resume cannot silently
adopt a new owner. This is local metadata, not a new stanza field or a MAC
against rewriting both metadata and local state.

prepare_with_policy checks saved account/store ownership before processing the
reply; when a saved account exists it also checks the supplied cryptographic
account snapshot. Refusal, identity-suspension and successful ratchet/outbox
reservation transactions recheck current authority before mutation. Original
intent, peer identity/session and query-state checks remain independent. Query
matches still require exact IQ ID and outer result/error.

Normal queued directory writes now use Pending.check_write_authority: it checks
account, original query selector/uncertain status and deadline at the existing
writer encoding boundary. Normal direct message writes (including explicit
selected identity resumption) check the connection account fingerprint again
there. This final driver fence binds the connection account generation to the
current store generation; stale-generation accounts are not repaired or reset.
A failure leaves reserved/uncertain state, not permission to replay the query,
advance a second session, or regenerate a send ID. No wire contents change.

## Explicit legacy/offline boundaries

Account-less offline preparation remains bound to the original absence. Creating
a saved account changes that ownership. This preserves existing offline adapters,
not live transmission authority: the normal message writer requires its saved
connection account binding. Explicit identity resume reads a prospective stored
authority when present. A historical suspension without that field binds current
local authority for that selected operation only; no historical preparation or
old live callback is retroactively proven. Complete frozen-intent provenance,
non-normal returned-node adapters, group sends and retry resends remain separate
audit work. A local guard does not hold a lock over a remote socket operation.

## Validation and related source audit

Tests: direct_reply_cannot_change_intent_after_account_replacement proves the
old refusal gap and now checks refusal with unchanged checkpoints, original
outbox row, journal head and outbox state after reopen.
queued_direct_query_and_message_recheck_account_before_encoding exercises both
real NodeLink queue guards with unchanged and replaced account states. Valid
owner admits one write; changed owner refuses before encoding, returns an error,
preserves query checkpoints and never reselects the uncertain reservation.
Existing direct identity-policy, crypto, media, ACK and no-replay tests remain.

**Confirmed source/local:** outgoing.rs begin_selected, Pending preparation and
suspension, selected resume; connection.rs Outbound.prepare and normal direct
query/message handoffs. This local ownership defect needed no new APK
instruction exports. Existing native directory/ACK/timing policies remain;
APK2.26.43.72 is still pinned for protocol research. No server restriction cause
or acceptance is established.

The turn's bounded app-state review confirmed that ingest_action's mutable
patch/error/catch-up path obtains actions::view_in_write before applying state.
The held-fetch binding/inspect_response path inspected here is read-only
structural provenance, not a newly implemented inline fetch-state application.
That did not demonstrate a new mutable fetch adoption defect. Full native fetch
application and independent downstream adapters remain open; this is not a
whole-app-state parity claim.

[Generic action ownership](action-account-ownership.md), [group projection](group-projection-ownership.md)
and [identity refresh deferral](identity-refresh-account-deferral.md) retain
other lifecycle boundaries. No live request/account operation or identity reset.

Verification: **1,602 workspace/all-feature tests passed** and strict workspace/
all-target/all-feature Clippy passed. Formatting, CLI/OpenAPI, coverage freshness
and staged repository hygiene are checked before commit. Offline tests do not
prove server acceptance or complete network parity.


## Original query reservation hold (2026-11-07)

New direct key-query preparation also saves an independent encrypted original-reservation marker in its initial transaction. Either that marker or the mutable outcome row suppresses another selection for the same local intent. A lost outcome row alone therefore remains held across restart; inspection reports a redacted local fault instead of silently treating it as fresh work. No historical state is backfilled or replayed. [Regression and limits](replay-safety-audit.md#direct-key-query-original-reservation-survives-checkpoint-loss-2026-11-07).
