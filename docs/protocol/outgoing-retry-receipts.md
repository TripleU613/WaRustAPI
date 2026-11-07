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

## Classic incoming local-prekey retry error and lazy lookup — 2026-11-05

**Confirmed static / inferred provider equivalence / implemented offline:**
[matching-session reuse, local EDR -> wire3, adapter ordering and tests](apk/2.26.43.72-incoming-local-prekey-retry/README.md).
Classic receive now decodes local prekey records only on backend key-store lookup,
so corrupt unused records do not block a matching-session follow-up. Actual
selected local key failures retain SignalIncomingLocalPrekey / IncomingLocalPrekey
through normal retry/error3, including classic pairwise group processing.

Real-crypto tests cover required missing/corrupt records and successful held-session
reuse with corrupt unused records. Normal-loop cases assert error3 and unchanged
prekeys/no repeated retry or delivery ACK after uncertain write. Three claimed
MAC fixtures were corrected to modify the nested MAC, preserving outer selectors.
Factory signed-material gates, storage failures, hybrid/KEM and later key-pair
integrity classification remain separate open work. Other typed causes/count/key/
routing/ACK parity remain incomplete; no live traffic.

Validation:1329 workspace/all-feature tests passed after correcting the nested
MAC fixtures. Formatting, strict workspace/all-target/all-feature Clippy, CLI
reference regeneration, APK coverage, staged whitespace and research checksum/
link hygiene passed. Offline only.

## Newer unknown sender-key versions retain retry error4 — 2026-11-05

**Confirmed static / inferred provider equivalence / implemented offline:**
[version gate, cause refinement and backend identity](apk/2.26.43.72-sender-key-unknown-version/README.md).
Only skmsg version>3 typed rejection becomes SignalUnknownSenderKeyVersion /
UnknownSenderKeyVersion and normal retry/error4. Native Unknown version is not
EtY's Unknown type refinement. Lower legacy versions keep no-retry handling;
empty-record and short-input precedence remain intact. Other formats are unchanged.

Inbox tests cover high nibbles4/15, legacy0/1/2 and short2/4, with unchanged keys/
journal. The production-loop fixture checks error4 before signature verification,
original timestamp, durable uncertain reservation and no second retry/delivery ACK
on reconnect/redelivery. Offline CLI replay sends nothing. Native short-input,
other formats and full key/count/routing/ACK parity remain unfinished. No live traffic.

Validation:1329 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Offline only.

## Application-padding count and retry admission corrected — 2026-11-05

**Confirmed static / implemented offline:** [EHq unsigned count rule, retry
exclusion and separate CBC failure exploration](apk/2.26.43.72-application-padding-retry/README.md).
Receive now shares the native count-only rule0<count<plaintext length. It no longer
requires count<=16 or repeated padding bytes, and it rejects entire-buffer padding
rather than returning empty plaintext. Existing crypto authentication/block padding
and sender-side padding are unchanged.

SignalInvalidApplicationPadding covers empty/invalid decrypted application framing;
direct/group inbox continues as Unsupported without automatic retry, delivery ACK
or Rust atomic inbox/ratchet commit. Native Signal-store call order before EHq
trimming is retained as a separate durability/transaction tracing gap. Tests cover
unsigned counts through255, nonrepeating tails, actual authenticated ciphertext
and production-loop no-retry/no-confirmation/no-state-event behavior. CBC/provider
errors, full native commit/ACK policy and other causes remain open; no live traffic.

Validation:1332 workspace/all-feature tests passed on the final run. Formatting,
strict workspace/all-target/all-feature Clippy, CLI reference regeneration,
regenerated APK coverage, staged whitespace and research checksum/link hygiene
passed. The invalid table fixture and inventory-test clock were corrected before
that final run. Offline only.

## Stateless/PQ retry preparation inputs (2026-11-07)

**Confirmed selected static:** exact incoming enc state=false and session_type=pq
produce native isStateless/isPqSession retry-job inputs. They affect conditional
key preparation at dispatch and job time, with a native minimal-key branch too;
they are not blanket invalid/decrypt gates. Rust's bounded minimal retry does not
implement this policy. Feature/enum/key lifecycle and wire bindings remain open.
[Evidence](apk/2.26.43.72-enc-state-pq-retry-inputs/README.md). No runtime change,
key regeneration, new request or replay was added.

## Retry prekeys: direct-distribution ownership (2026-11-07)

**Confirmed selected static:** native optional Kyber retry preparation selects an
unsent ordinary non-directly-distributed key, marks direct_distribution and its
local publication timestamp, and requires exactly one updated row before return.
Ordinary prekey preparation has the corresponding ownership transition. Constructor
mapping binds signed/ordinary/Kyber envelopes to keys/skey,key,pq respectively.
Rust has KEM inventory/upload support but no carried-key retry/direct-distribution
state; do not add bytes without durable selection, upload exclusion and no-replay
ownership. [Evidence](apk/2.26.43.72-retry-prekey-distribution/README.md). This is
research only; no key mutation, runtime request or server-acceptance claim.

## Incoming count cannot reset the minimal retry (2026-11-07)

**Confirmed construction gap closed within the current supported scope:** native
aggregate enc count reaches F1Z.A04 and retryCount unchanged, then the job adds1;
positive input also selects carried-key preparation. Rust previously always
built count1. The shared ordered Java-i32 parser now gates the bounded minimal
queue to aggregate0; other/invalid counts stay preserved without new wire or
reservation, and historical once-only/uncertain requests remain immutable. Full
carried-key/count progression and stateless-policy integration remain unfinished.
[Evidence](apk/2.26.43.72-outgoing-retry-count-input/README.md). Real-store
regression reproduces the old queue result and verifies no mutation/reopen/reset.
No new retry/ACK family or live operation.

Verification: the real-store count1 regression first queued a request on old
code, then returned zero without durable changes after the fix. All1,591
workspace/all-feature tests, strict all-target/all-feature Clippy, formatting,
CLI reference, OpenAPI library-symbol check, APK coverage, retained evidence
checksums and repository hygiene passed. No account traffic was generated.

## Stateless retry configuration authority (2026-11-07)

**Confirmed selected static:** the two extra retry-key controls use effective
00E accessors. Selected compiled entries insert0 for21477 and false for23884;
both concrete initial MobileConfig specifiers are0 after a complete2065-write
lookup check. These are not account-effective values or a complete final-map
audit: buildKeepingLast, preferences, cache and readiness remain separate. The
selected SDK integer-zero path disables the extra >0 predicate, while legacy
overrides can require key preparation. Rust policy selection remains unported.
[Evidence and reproducible derivation](apk/2.26.43.72-retry-config-inputs/README.md).
No runtime behavior, key mutation or live operation changed.

## Retry preparation account binding (2026-11-07)

**Confirmed Rust ownership gap fixed.** The retry queue froze registration bytes
from its supplied AccountRecord snapshot, while receipt account ownership was
first established at reserve_pending_owned. A stale snapshot could therefore
prepare old registration bytes and later have them reserved under current
authority. A pending request could likewise outlive an account change before
its first reservation. The real-store regression reproduced stale preparation
on the previous code. This is a local ownership defect, not proof of a native
server rejection or restriction trigger.

When a saved account exists, queue_retry_for_incoming_with_cause now verifies
that the supplied snapshot has the current generation and the existing receipt
authority fingerprint (phone, own JID, registration status/id, identity public/
private material and store generation). It performs that comparison in the same
write transaction that freezes the retry row. New owned rows retain an internal
prepared_account_authority_sha256 marker. It contains no raw identity material.

reserve_pending_owned rechecks that preparation marker against current account
authority before changing pending status or building callback reservations.
Mismatch, malformed marker or missing account refuses the transaction and leaves
wire and reservations unchanged; the existing final pre-write authority fence
continues to protect changes after reservation. No keys, IDs or account state
are regenerated or reset. The encoded retry receipt itself is unchanged.

The regression covers changed registration ID, identity private bytes, generation,
own JID, registration status and account disappearance before preparation or reservation. It verifies stale
preparation refusal without checkpoint mutation, then valid preparation followed
by changed account and database reopen. Reservation refuses without replacing
the pending row. Restoring original authority permits exactly one reservation;
repeating cannot replay the uncertain request. No account operation is performed.

**Explicit compatibility boundary:** unregistered account-less offline queue
calls retain their previous unbound format. A registered snapshot with a missing
saved account refuses preparation instead of falling back to that adapter. Historical rows without the preparation
marker are not retroactively bound to a guessed account; their existing reserve/
write checks remain. Thus this does not claim complete historical provenance or
all receipt-family preparation ownership. The runtime no-queue diagnostic also
now says existing reservation OR unsupported input; zero queued does not prove
a retry row already exists (count-admission refusals create none).

Related native evidence remains the pinned2.26.43.72
[retry key callable](apk/2.26.43.72-enc-state-pq-retry-inputs/retry-key-callable.txt),
which obtains current local registration/key material under its selected Signal
lock, and [retry ownership trace](apk/2.26.43.72-retry-prekey-distribution/README.md).
This Rust fence is an integration invariant, not a newly inferred wire field or
universal server requirement. Stateless policy, carried keys and full count
progression remain unfinished. No new ACK/retry family or live account traffic.

The existing fake-link retry/write-cut regression initially exposed a fixture
with AccountRecord's default generation0 inside a generation1 store. The fixture
now records its actual store generation, matching production registered setup.
Its focused multi-cause retry/write-cut/no-replay run passes with the ownership
check intact. The group recovery fixture also now saves its registered account
with the actual store generation instead of using an account-less registered
snapshot. Unregistered account-less offline compatibility remains separately tested.

Verification: stale preparation and registered-account disappearance regressions
failed before their respective guards, then passed. After correcting the two
registered fixtures, all1,592 workspace/all-feature tests passed. Strict
all-target/all-feature Clippy, formatting, CLI reference, OpenAPI library-symbol
check, APK coverage and repository hygiene passed. No account traffic occurred.

## Prepared retry wire integrity before reservation (2026-11-07)

**Confirmed Rust ownership gap fixed.** Account preparation binding protected
which account supplied a new retry, but reserve_pending_owned still adopted the
then-current pending wire as its callback original. A real-store regression
changed only the pending receipt's registration bytes (31→32), leaving account
authority unchanged; old code reserved the changed bytes. This was not a decode
failure: the replacement was a valid receipt encoding.

New retries prepared with saved account state now retain prepared_wire_sha256
alongside the preparation account fingerprint, in the same queue transaction.
Reservation checks that digest before decoding/adopting callback ownership or
changing status. A mismatched or malformed digest refuses the transaction, leaving
the pending row and original reservation state unchanged. The existing reserved
wire/account checks continue after this earlier stage. No wire-format field or
network request was added; legitimate retry bytes are unchanged.

The regression reproduces acceptance before the fix, then changes registration
bytes, reopens the encrypted store, and verifies reservation refusal without row
replacement. Restoring original persisted wire allows one reservation; repeating
cannot replay it. This is an integrity consistency check, not cryptographic
authentication against rewriting both wire and digest. Rows without a preparation
digest (including earlier account-bound or account-less records) retain their
explicit legacy checks; no hash is backfilled as inferred historical proof. Full
historical preparation provenance and carried-key/count/configuration integration
remain open. No live account operation or server-restriction causality inferred.

Verification: the valid registration-byte substitution was accepted before the
fix; the reopen/refusal/restoration/once-only regression now passes. All1,593
workspace/all-feature tests, strict all-target/all-feature Clippy, formatting,
CLI reference, OpenAPI library-symbol check, APK coverage and repository hygiene
passed. No live account traffic was generated.

## Carried-key byte binding narrowed

**Confirmed selected static:** the pinned retry PQ getter's concrete byte-array
KEMKeyPair overload stores public bytes unchanged through KEMPublicKey.A00; the
classic signed getter instead reaches F3k.A02, requiring type0x05 and at least33
stored bytes and taking exactly the next32. The common ID helper corroborates
existing low24 big-endian research. [Method traces and boundaries](apk/2.26.43.72-retry-key-envelope-bytes/README.md).
This resolves a constructor ambiguity in the earlier retry distribution bundle,
not the full signing/generation/policy/lifecycle integration. No retry enabled.

## Retry job callback ownership is not the bulk-query map

**Confirmed selected static:** retry job0Sf.A0B registers a fresh FfK in0Th.A04
before dispatch, then the job waits on untimed get. Wrapper mode0 forwards null
on success; duplicate registration reports then replaces on normal continuation.
The previously traced bulk disconnect method clears A05, so it does not prove
retry-receipt cleanup. [Exact methods, normalization and remaining consumers](apk/2.26.43.72-retry-receipt-callback-handoff/README.md).
Rust durable no-replay ownership remains unchanged; no automatic retry added.

## Retry callback consumers: ACK completion versus disconnect failure

**Confirmed selected static:** exact A04 field-reference scan binds ordinary
matched ACK completion to BsT(node), with retry wrapper forwarding null even
for an error-bearing ACK. Selected fail-all handling snapshots/clears A04 then
calls BsR(IOException(disconnected)); the status error-by-ID consumer is separate.
[Methods and remaining producer/retry/restart scope](apk/2.26.43.72-retry-receipt-callback-consumers/README.md).
No Rust success/refusal or no-replay policy change; no live operation.

## Retry job failure eligibility is conditional on the runner

**Confirmed selected static:** SendRetryReceiptJob.A0N returns true for supplied
exceptions on normal continuation. The shared runner excludes RuntimeException,
counts attempts, then rechecks requirements for loop/defer. This constructor is
persistent/grouped with default job budget100, distinct from incoming protocol
retryCount. ChatConnectionRequirement's concrete predicate is0AI.A04==2; full
state writers and scheduling remain open. [Exact binding and no-replay boundary](apk/2.26.43.72-retry-receipt-job-failure-policy/README.md).
No runtime retry/defer behavior changed; no live account operation.

## Connection requirement invalidation

**Confirmed selected static:** all five direct writes of0AI.A04 now bind initial/
server-disconnect state3, connecting state1, ready state2 and separate state4.
Only2 satisfies ChatConnectionRequirement. Existing readiness work is reused;
state clearing and the distant connecting-handler join are now classified.
[Exact bodies and concurrent-ordering limits](apk/2.26.43.72-retry-connection-requirement-state/README.md).
No Rust reconnect/replay behavior or live account operation changed.

## Queue wake provider construction boundary

**Confirmed selected static:** the queue facade combines two injected sets,
passes an RF1 snapshot to a per-facade cached NhR, whose retained constructor
associates each observer via Cgm. RNL's concrete masked factory case245 is now
located; set-key membership/provider lifecycle remains unresolved. Existing wake
and requirement traces are reused. [Provider evidence](apk/2.26.43.72-jobqueue-observer-provider/README.md).
No runtime replay/reconnect behavior changed.
