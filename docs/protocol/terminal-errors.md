# Terminal stream errors and ACK diagnostics

## Saved-capture attribution (2026-10-08)

**Confirmed implementation:** `transport::inbox::preserve` saves encoded nodes
under `inbound:sha256:<content digest>` without a capture time or connection
epoch. Repeated identical nodes share a key. `lab-inbox --inspect` scans all
saved captures and deduplicates printed shapes; it is not a latest-session log.
The 2026-10-08 receive stdout/stderr proves authenticated activation and early
terminal closure, but contains no terminal tag or numeric reason. The stored
`stream:error`/ack shape cannot independently be attributed to that session.
There was no before-session capture census. Earlier wording identifying it as
*the* live terminal ACK is too strong; its origin remains **unknown**.

**Observed offline:** a saved stream:error has no outer attributes; one ack
child has class notification, an ID attribute and no to attribute. The fixed-label inspection identifies type w:gp2; no ID or address is exposed.
This does not establish what we transmitted, that an ACK was rejected during
the recent window, a ban, a server wait or causation by the AB-property query.
The query remains sent/outcome_unknown with its original reservation.

**Implemented:** `lab-inbox --inspect` includes terminal_ack_class,
terminal_ack_type, terminal_ack_id_present and terminal_ack_to_present when
one ack child occurs under stream:error. Only fixed class/type labels are
printed; unsupported values become unknown, absent values become absent.
Two or more ack children report terminal_ack=ambiguous. It never echoes an ID,
address or unknown field value. This is offline inspection; it sends nothing,
changes no capture key and applies no retry/recovery policy.

Tests cover known labels, echoed-ID/address/unknown-value redaction, unrelated
stanzas, missing children and duplicate-ack ambiguity. Presence flags say only
that an attribute exists, not that it is valid or that the APK requires it.

## APK anchors and remaining scope

**Confirmed static 2.26.37.71:** exact stream:error anchors locate 1R7.A02/A03
and 1Rl.<clinit>; WriterThread/write/send-message-ack locates 0y0.A00.
1R7.A02 is the companion interceptor path: it recognizes stream:error and,
under a separate 0GD.A02 predicate, parses an integer code with default zero
before routing to 1Dz.A1D. Its separate error branch reads class/id/code/text.
This method is not proof of the primary client's required closure handling or
an interpretation of this saved ACK. Primary routing, the active predicate,
0y0's complete ACK construction and attribution/correlation of this echoed group-notification ACK remain untraced.

[Scoped raw evidence](terminal-ack-evidence.txt) retains the exact anchor scan
and complete A02 disassembly with exception tables. No live capture, raw APK,
private number, key or echoed identifier is included.


**Validation:** 47 experimental CLI checks passed (46 unit and one API-symbol),
including two new diagnostic regressions. Strict CLI/all-target Clippy,
regenerated CLI reference, experimental build and formatting passed on system
Rust 1.99.0. Rebuilt offline inspection identifies the saved ACK type as w:gp2;
no network session was opened. Complete workspace tests were not repeated for
this CLI-only diagnostic chunk. Historical capture origin remains unknown.

Repository hygiene passed: 915 local links, tracked-file budgets, package
licenses and 3,840,464 bytes across five checksum-verified APK bundles.


## Network terminal provenance (2026-10-08)

**Implemented local diagnostic policy:** the comparison login reader and active
receive loop call `inbox::preserve_in_epoch`. The encoded capture, terminal
observation metadata and latest-terminal pointer commit in one transaction.
Only failure and stream:error get metadata. Existing content-addressed capture
keys and their wire data remain unchanged; no identity/session is migrated.

`lab-inbox --inspect` includes terminal_observation for those stanzas:
first_observed_at and last_observed_at are local Unix milliseconds; last_epoch
is actor-local and can repeat after process restart; observations counts newly
instrumented network observations, saturating at u64::MAX; latest_terminal
identifies the most recent instrumented terminal commit by commit order. The
wall clock can move backwards. These are local observations, not APK/server
fields, globally unique session identifiers or a complete per-session journal.
They establish latest-observed provenance when compared with the current run's
start/end, not the origin of every prior occurrence of the same content.

Old and offline-only captures report unknown. `preserve` and offline replay do
not add, increment or overwrite metadata. A later real network re-observation
of an old content hash records its current observation, preserving the wire;
first_observed_at then means first **instrumented** observation, not the first
time that content ever arrived. Inspection applies no recovery policy and does
not replay the uncertain AB-property request.

**Tested offline:** first/repeated observations, epoch reset/backward wall clock,
latest marker supersession, nonterminal exclusion, private-content redaction,
reopen persistence, unchanged content keys/wire and offline non-backfill. Invalid
stored observation metadata aborts the transaction: even the preceding capture
write and latest-marker update roll back. Failure diagnosis is preserved rather
than silently replacing corrupt metadata. Older capture origin remains unknown.


### Validation and bounded saved-account check

**Verification:** all 702 workspace/all-feature tests passed (583 API unit,
four codec integration, nine crash-matrix, 46 CLI unit, one API-symbol and
59 xtask). The existing real session-loop terminal fixture additionally now
asserts one recorded observation, matching epoch/time and latest marker; its
targeted rerun passed. Strict workspace/all-target Clippy passed again after
that test change. Warning-free Rustdoc, 24 default CLI checks, regenerated CLI
reference, experimental CLI build and formatting passed on system Rust 1.99.0.
Pinned Rust 1.98.1 proof is not claimed.

**Observed live:** preflight found no eligible outbox, queued action, pending
receipt or selected-chat refresh. The existing initial AB-property read was
already outcome_unknown and was not requeued. One 32-second receive-only
listen authenticated, became active, completed its window and stopped cleanly
(exit 0), without --send-outbox. No instrumented terminal observation occurred;
offline inspection still has one historical terminal shape with unknown
provenance and zero known terminal observations. This clean session does not
explain the previous closure or live-test terminal persistence; the session-loop
fixture is the verification for the terminal code path. It also does not
establish fresh incoming-message decryption or long-term account reliability.

**Observed postflight:** actions, selected devices, selected pins, receipts and
outbox matched preflight; zero new actions. The initial read remains
outcome_unknown with the same action ID; no query was queued or replayed.
Login/captures can persist, so these comparisons do not claim all store tables
are unchanged. Private command output remains in ignored private local storage.

Repository hygiene passed: 917 local links, tracked-file budgets, package
licenses and 3,840,464 bytes across five checksum-verified APK bundles.
