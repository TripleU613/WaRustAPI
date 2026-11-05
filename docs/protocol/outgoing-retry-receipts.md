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

## Version gates and no-retry limits — 2026-11-05

**Confirmed static:** [selected native version gates and backend comparison](apk/2.26.43.72-message-version-error-paths/README.md).
ED6 is used for legacy versions by four selected message constructors; it does
not establish identity-refusal semantics. Direct Fj3 admits version nibble2..4,
while the current backend admits3..4. Full version2 crypto support is not proven.
Prekey gate3..4 and sender-key gates exactly3 have different high-version exception
paths. Ordinary direct/group ED6 -> -1007 is excluded, while EDY -> -1005 is eligible.
Prekey outer construction has a separate catch region; later ED6 handlers cannot
be applied indiscriminately to it. These differences rule out a blanket backend
unsupported-version -> no-retry mapping. Identity policy, full range/primitive
parity and producer-specific retry causes remain open. Research only; no new
wire code, runtime behavior or live request.

Validation:31 filtered retry tests, APK coverage, staged whitespace and research
checksum/link hygiene passed. No Rust source changed; the full workspace suite
was not rerun for this research-only chunk.

## Legacy sender-key message retry exclusion — 2026-11-05

**Confirmed static:** [Fj2 legacy version gate -> ED6 and selected -1007 exclusion](apk/2.26.43.72-message-version-error-paths/README.md),
with [0Yl.A00 group coordinator catch table](apk/2.26.43.72-group-counter-error-paths/README.md).
**Inferred provider mapping / implemented offline:** only receive_group's skmsg
decryption maps backend LegacyCiphertextVersion to the redacted typed error
SignalLegacySenderKeyMessage. Group inbox preserves capture and continues as
Unsupported, with no automatic retry or delivery ACK and no ratchet/event commit.
This neither authenticates the input nor creates a durable duplicate marker.

The classification stays scoped to sender-key message decrypt. SKDM ingestion,
ordinary/prekey direct parsing, newer unknown versions and short ciphertext
retain their existing policy; their full native result semantics are not inferred
from the legacy skmsg branch. Missing sender-key record is still classified
before message parsing and uses the existing MissingSession/error1 path.

Offline inbox regression mutates real synthetic sender-key ciphertext to version
nibbles0/1/2 under fresh IDs and verifies Unsupported with unchanged journal and
sender keys. Version4 and a64-byte version2 payload remain Undecryptable, showing
that newer-version and length admission are not swept into this exclusion.
Authenticated durable duplicates, including consumed content, remain Duplicate.
Other producer-specific retry code/key/count/routing/ACK parity remains unfinished.
No live account operation or server acceptance claimed.

Validation:1328 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Offline only.

## Sender-key signature retry error preserved — 2026-11-05

**Confirmed static / inferred provider equivalence:** [message signature verifier,
status refinement, wire5 and backend source identity](apk/2.26.43.72-sender-key-signature-retry/README.md).
Only skmsg decryption maps SignatureValidationFailed to the fixed redacted
Error::SignalInvalidSenderKeySignature. Inbox returns InvalidSenderKeySignature;
normal receive queues retry/error5. The private RetryCause enum now distinguishes
missing session (wire1) from sender-key message signature (wire5), with no guessed
generic fallback. Other signature producers are unchanged.

Production-loop regression tampers a real synthetic group ciphertext signature,
checks error5, original timestamp and reservation before failed write; reconnect
redelivery emits neither another retry nor delivery ACK. Sender-key rows remain
unchanged. Existing direct/group missing-session cases retain error1. Offline CLI
replay sends nothing. Count/key progression, other typed causes, full routing/ACK
policy and server acceptance remain unverified. No live account operation.

Validation:1328 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Offline only.

## Future-counter typed mapping prerequisites — 2026-11-05

**Confirmed static:** [sender-key signature-before-counter order, strict2000
boundary and backend limits](apk/2.26.43.72-sender-key-future-counter/README.md).
Native exact future-counter message refines to wire6. Backend also uses2000,
but returns generic InvalidMessage shared with CBC failures. Its selected-chain
accessor is private, front state may be wrong, and into_components advances the
front chain to its lease ceiling rather than passively exposing current position.
No provider-string matching or partial front-state mapping was added.

Native signed-int versus backend unsigned-counter comparison also differs at
high-bit inputs; this arithmetic inference prevents claiming universal cause
parity from a matching normal-range constant. Full typed cause/state access,
integer-domain compatibility, precedence and remaining retry/ACK policy are open.
Research only; no runtime behavior or live request changed.

Validation:31 filtered retry tests, APK coverage, staged whitespace and research
checksum/link hygiene passed. No Rust source changed; the full workspace suite
was not rerun for this research-only chunk.

## Sender-key future-counter retry error preserved — 2026-11-05

**Confirmed source / inferred provider equivalence / implemented offline:**
[effective matching-state metadata view and typed future-counter retry](apk/2.26.43.72-sender-key-future-retry/README.md).
Loaded-record serialization provides a passive view after reservation recovery,
without component-export advancement or front-state guessing. After backend
InvalidMessage(SenderKey), the adapter classifies only a matching-chain jump
strictly above2000 with both counters in native nonnegative signed-int range.
It uses typed fields, not provider strings, and zeroizes temporary record bytes.

The fixed typed error/outcome reaches normal retry/error6; signature precedence,
missing-session error1, signature error5, original timestamp and once-only durable
reservation remain intact. Actual backend tests cover2000/2001, retained older
chain, reservation ceiling, bad signature and high-bit nonclassification. The
production loop checks error6 and no replay/state change after uncertain write.
Offline CLI replay sends nothing. High-bit/native arithmetic equivalence, other
primitives and full retry key/count/routing/ACK parity remain open. No live traffic.

Validation:1329 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Offline only.

## Empty sender-key records and missing chains — 2026-11-05

**Confirmed static + platform contract / inferred equivalence:** [native empty
record versus wrapped missing-chain paths](apk/2.26.43.72-sender-key-chain-retry/README.md).
Empty record is checked before message parsing and selects error1. Unknown chain
inside a nonempty record constructs EDR then EDY(Throwable); Android's documented
class-prefixed cause-message behavior prevents the bare No keys prefix refinement,
retaining -1005 -> wire4. Full classes, platform references and a local OpenJDK
corroboration fixture are retained; no APK runtime observation is claimed.

receive_group now preserves empty-record precedence. Nonempty-record typed
NoSenderKeyState becomes SignalMissingSenderKeyChain / MissingSenderKeyChain and
normal retry/error4. Diagnostics are fixed/redacted. Production-loop fixtures
check error1 for malformed input with an empty record, error4 for a parsed unknown
chain, and unchanged keys/no repeated retry or delivery ACK after uncertain write.
Existing signature/future/missing-session cases retain5/6/1. Offline CLI replay
sends nothing. Other causes, count/key/routing/ACK parity and live acceptance
remain incomplete. No live account operation.

Validation:1329 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Local platform-contract
fixture passed separately; no APK or live account execution.
