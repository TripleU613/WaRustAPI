# Outgoing retry receipts — 2026-11-05

**Confirmed static / timestamp correction:** APK2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Retained parser/envelope/job/writer chain, exact fields and limits](apk/2.26.43.72-outgoing-retry-timestamp/README.md).
No live traffic or APK execution occurred.

Rust's automatic undecryptable-message retry previously set nested retry.t to
local queue-creation time. Native selected FVv/Ewu message paths preserve the
original root t through Dyc/Dyb seconds, Edc/F1Z milliseconds,
SendRetryReceiptJob.timestamp andEYI.A06; F3e.A08 divides by1000 for retry/@t.
The normal queue now uses the original message's seconds. Missing/unusable
values are refused with a redacted field/reason and no queue write, rather
than being replaced with the local clock. Values that cannot safely convert
to milliseconds are a local unsupported boundary, not a claimed native error.

The public retry builder's explicit seconds argument is now named
message_seconds; its wire field meaning is documented. Registration/counter,
addressing and reservations are unchanged. Valid redelivery with a different
timestamp cannot reset an already-held logical retry request.

A new queue/reopen regression verifies exact original seconds, invalid-input
no-write behavior and no replay. The real normal receive-loop direct/group
write-cut test asserts the emitted original timestamp and retains the existing
uncertain reservation across a failed write/restart. Older synthetic retry
fixtures now include a usable timestamp; malformed-input cases remain explicit.

Native retry/@error is also confirmed unconditional inF3e.A08, but its numeric
producer classification is unported. It must not be guessed from generic
Undecryptable or confused with receipt ACK error487. Conditional extra keys,
retry-count progression, full native time admission/ranges and special routing
remain open. This is not a complete outgoing retry receipt port.

Validation:1327 workspace/all-feature tests passed, strict workspace/all-target/
all-feature Clippy and formatting passed, CLI reference regenerated, and APK
coverage plus research checksum/link hygiene passed. Offline verification only.

## Error domains and eligibility recovered — 2026-11-05

[Native crypto-status → failure-kind → wire-error tables and producer limits](apk/2.26.43.72-outgoing-retry-error-map/README.md)
now distinguish the three number domains. EtY's selected eligible crypto set
maps throughF2q/DxB; several nonpersistent/device paths supply wire errors
withoutDxB. Its default0 and maximum explicit output11 are not universal policy.

EtY's guarded status=-1005 refinements identify native signature, key-type,
future-counter, MAC and no-valid-session categories. Exception class identity
for other primitive-specific failures remains open; generic obfuscated class
constructors do not establish their semantic names. Rust's redacted typed
Signal errors are merged before its generic Undecryptable result reaches the
retry queue. Preserve that typed cause and eligibility before serializing an
error; do not add a blanket default or parse arbitrary provider error strings.

This is a research chunk, with no production behavior change. The native
selected eligibility predicate also excludes several nonzero crypto statuses,
so a complete fix must cover **whether to request a retry**, not only its error
attribute. Typed cause propagation, other primitives/producer paths, key/count
policy and full routing remain unfinished. No live request.

Validation for this research chunk:29 filtered retry tests passed; APK coverage,
staged whitespace and research checksum/link hygiene passed. No Rust source
changed, so the full workspace suite was not rerun for the mapping-only change.

## Old counters no longer request retries — 2026-11-05

**Confirmed static:** [ED4 old-counter producers and -1001 mapping](apk/2.26.43.72-old-counter-no-retry/README.md)
close one typed cause: selected native retry eligibility excludes this status.
**Inferred compatibility mapping / offline implementation:** libsignal's
DuplicatedMessage now survives as Error::SignalOldMessageCounter. Direct and
pairwise SKDM inbox handling preserves capture and continues as Unsupported,
without retry or delivery ACK. Durable authenticated same-ID duplicates keep
the existing Duplicate result. No counter/address/backend data enters the error.

Other retry error codes, native old-counter confirmation policy, sender-key
primitive classification and full route/key/count policy remain open. Existing
group crypto error formatting is also still an uncorrected redaction gap; no
primitive equivalence is inferred from direct cipher evidence. No live request.

Validation:1327 workspace/all-feature tests passed; formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage and
research checksum/link hygiene passed. Offline only.

## Group counter and producer-specific errors — 2026-11-05

**Confirmed static:** [sender-key ED4/-1001 and key-error producer paths](apk/2.26.43.72-group-counter-error-paths/README.md).
Sender-key old counters use the same excluded retry status as direct messages.
**Inferred mapping, implemented offline:** group DuplicatedMessage now retains
SignalOldMessageCounter, preserving capture without retry, delivery ACK or state
commit. Group provider diagnostics now use fixed variant-only messages. Durable
authenticated duplicates retain their prior confirmation behavior.

EDR identifies missing/invalid local prekeys in F22.A04, but group Eyj.A01 wraps
EDR as EDY before result classification. Session builder signature failures
also use a different exception from the exact “Invalid signature!” refinement.
Global exception/error-name -> wire-code mappings would lose these distinctions.
No other retry wire-code policy changed. Full native ACK policy and remaining
producer/primitive/key/count/routing parity remain open; no live request.

Validation:1328 workspace/all-feature tests passed, as did formatting, strict
workspace/all-target/all-feature Clippy, CLI reference regeneration, APK coverage
and research checksum/link hygiene. Offline only.

## Direct coordinator attribution and cause aggregation — 2026-11-05

**Correction / confirmed static:**0WJ.A0P is fast-ratchet group decrypt, not
ordinary direct decrypt. Ordinary direct0WJ.A0N -> F22.A05/A01/A02 independently
confirms ED4 -> -1001, supporting the already implemented no-retry behavior.
[Selected methods, missing-session chain and signature producer](apk/2.26.43.72-direct-retry-cause-paths/README.md).

F22.A05 missing session throws EDS -> A0N -1008 -> wire1. F22.A01 aggregates
EDY failures into EDY(List), whose outer No valid sessions. message refines to
wire8; raw BadMac/future-counter mappings cannot ignore this wrapper. Sender-key
Fj2.A00 exact Invalid signature! directly throws EDY, supporting selected group
wire5; bundle key signatures use a separate producer. No new wire code/runtime
behavior added. Remaining cause propagation, provider equivalence and full
retry/ACK policy are open. No live account operation.

Validation:31 filtered retry tests, formatting, APK coverage, staged whitespace
and research checksum/link hygiene passed. This chunk changes research and a
source attribution comment only; the full workspace suite was not rerun.

## Missing-session retry error preserved — 2026-11-05

**Confirmed static:** [ordinary direct parser/session order and -1008 -> wire1](apk/2.26.43.72-direct-retry-cause-paths/README.md),
and [group absent sender-key EDS -> -1008](apk/2.26.43.72-group-counter-error-paths/README.md).
A0N constructs Fj3 beforeF22.A05 tests session presence. Eyj.A01 checks empty
sender-key record before parsing its message. These orderings differ.

**Implemented / offline tested:** IncomingResult::MissingSession preserves the
specific absence through normal receive. Direct requires no saved session and
successful SignalMessage syntax admission; a present session with missing
identity remains generic Undecryptable. No sender-key record yields MissingSession
on the group path. Normal receive queues retry with error=1 using the existing
count1/original timestamp and durable selector/reservation. Malformed direct
input does not inherit this cause. Syntax admission does not authenticate MAC
or plaintext without a session.

The existing public low-level queue_retry_for_incoming and retry_receipt helpers
retain their unclassified behavior; callers without cause evidence do not gain
a guessed default. Private construction accepts the confirmed code from normal
receive. Existing reservation lookup keeps old held/uncertain/acknowledged wire
unchanged; introducing a cause never resets retry count or creates a second
attempt. Offline CLI replay recognizes the new outcome and sends nothing.

Production-loop regression uses a structurally valid synthetic Signal message
for the direct absent-session case and absent group key for skmsg; asserts
error1/original timestamp, pending reservation before failed write, and no repeat
or delivery ACK after reconnect/redelivery. This does not prove server acceptance.
Generic failures still lack full typed eligibility/error classification, and
native key/count/routing/ACK parity remains unfinished. No live account traffic.

Validation:1328 workspace/all-feature tests passed. The final historical-wire
regression then passed separately; strict workspace/all-target/all-feature Clippy,
formatting, CLI reference regeneration, APK coverage, staged whitespace and
research checksum/link hygiene passed. Offline only.
