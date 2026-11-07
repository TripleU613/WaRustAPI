# Retry receipt acknowledgement dispatch — 2026-11-05

**Confirmed static / implementation gap:** pinned APK2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Retained exports, exact branches, reproduction and limits](apk/2.26.43.72-retry-ack-dispatch/README.md).
No account traffic or runtime execution was used.

At commit bb6567d Rust did **not** ACK retry receipts: receipt_ack used receipt_kind's
explicit delivery/read/played allowlist. The hypothesis that malformed retries
receive an ordinary success ACK was refuted by the current source and existing
retry-exclusion regressions. The normal receive driver calling the confirmation
helper does not itself prove a confirmation is constructed.

The APK selected base handler distinguishes true completion, false/unhandled,
and LIL parse exceptions. False invokes2Cx.A00, which adds error487; its490 value
is a local diagnostic. LIL on the non-BX2 path reaches1rg.A04, whose allowlist
explicitly contains receipt and which calls its parsing-error helper with487.
Successful handler return can submit a plain confirmation even when retry work
was merely scheduled. BX2 uses separate completion/error queues. Exact final
Message76-to-writer binding and asynchronous ordering still require tracing.
Do not turn Rust's local unsupported keys/count/recipient limits into native
malformed-input errors, or ACK every receipt indiscriminately.

The actual selected base-handler receipt context factory is now located:
14h.BLn callsF0R.A03, which delegates toF0R.A00, reads root from/id/participant/type,
defaults absent type to delivery, and adds recipient/category/edit before freeze.
This supersedes the earlier candidate-only factory search for that entrypoint;
complete registration, alias policy and full dispatch remain open.

The initial research chunk changed no runtime policy. Its next step was to bind Android Message76 to
the final ACK writer, then introduce separately evidenced retry response
classification and ordering without converting local capability refusals to
protocol errors. Existing supported receipt behavior remains unchanged.

Validation: five filtered receipt-ACK tests passed with all features. APK coverage,
research checksums/local-link hygiene and staged whitespace checks passed.
No shared Rust source changed; the workspace suite was not rerun for this
research-only chunk.

## Unsupported-version error response correction — 2026-11-05

[Writer binding, ordering and implementation evidence](apk/2.26.43.72-retry-version-error-ack/README.md)
closes the previously open selected Message76-to-F3e.A05 binding. Both direct
writer and pre-ACK conversion routes use that builder. Readiness/recovery and
actual wire completion remain distinct. The selected160 handler placesFe5 in
14h.A09, and14h submits its ACK before removing/running the deferred callback;
there is no universal resend-before-ACK guarantee.

Shared received_confirmation now emits **error487** for an explicit non-1 text
retry version after four-byte binary registration admission. It mirrors root
receipt id/from/participant/recipient/edit using the existing supported context
rules. F10 returnsnull before reading nested id/t/count/keys in this branch, so
those are not extra prerequisites. Current normal/comparison callers keep their
existing durable-capture/admission sequencing. The response never authorizes
resend, crypto work, identity changes or a query.

This does not ACK ordinary supported retries or map all local parse_retry(None)
results to487. Missing/v1, non-text versions, malformed registration and unsupported
correlation fields remain unacknowledged here. Native LIL error response policy,
successful handler ACKs, complete upstream registration, BX2 and durable ACK
recovery remain unfinished. No live request occurred.

Validation for the correction:1322 workspace/all-feature tests passed, strict
workspace/all-target/all-feature Clippy and formatting passed, CLI reference
regeneration passed, and APK coverage/checksum/link hygiene passed. All checks
were offline.

## Required-field parsing-error responses — 2026-11-05

[Selected exception paths, fresh context, regression table and limits](apk/2.26.43.72-retry-parse-error-ack/README.md).
Shared confirmation now separately classifies missing retry/registration,
non-four-byte binary registration, invalid ASCII signed32 count, missing/invalid
ASCII signed64 t, and missing nested id. These confirmed nativeLIL paths get487
with root id/from/type/participant, **without recipient/edit**, matching1rg.A03's
fresh context. Unsupported-version487 still preserves those fields via2Cx's copy.
Classification follows native read order; registration errors beat version,
unsupported version beats later fields, and count beats t/id.

Default counts/versions, negative/high valid counts, empty/oversize IDs, carried
keys, Unicode and non-text grammar are not generically declared native errors.
Unsupported root context remains locally refused. This does not complete retry
success ACKs, malformed carried-key processing, full routing/BX2 or ACK recovery.
The shared helper remains after existing caller capture/admission sequencing.
No account traffic or server-acceptance claim.

Validation:1323 workspace/all-feature tests passed, including25 filtered retry
tests. Strict workspace/all-target/all-feature Clippy, formatting and CLI
reference regeneration passed. APK coverage and research hygiene checked before
push. This validates local decoded-node behavior, not live server acceptance.

## Reserved direct resend success ACK — 2026-11-05

This initial31273d5 chunk excluded queries; the follow-up below supersedes that
query exclusion after tracing their deferred job requirements.

**Confirmed static / bounded local port.** In pinned43.72,160.A04 buildsFe5 and
puts it into14h.A09 at0668..06ac. On the selected non-BX2 successful-handler
path,14h.A00 calls0Sf.A0O at0156..015c then branches to00fe, removes the Runnable
and invokes it at0112..0118. The previously retained
[dispatch](apk/2.26.43.72-retry-ack-dispatch/base-dispatch.txt),
[handler](apk/2.26.43.72-retry-ack-dispatch/retry-handler.txt) and
[writer binding](apk/2.26.43.72-retry-version-error-ack/README.md)
establish ACK submission before this deferred callback. They do not establish
remote receipt, cross-thread physical wire order or every query/group branch.

Normal direct receive now uses answer_with_confirmation: it first calls the
existing answer entrypoint, which prepares and atomically reserves the advanced
session/re-encryption attempt. Only a returned **message** can attach a private
handled-retry confirmation. Its decoded request must remain parseable with
user-like chat/device and supported root correlation fields. The ACK uses root
receipt id/from/type=retry/participant/recipient/edit, with no error attribute;
its id is independent of the resend's nested original-message id.

NodeLink.send_retry_answer submits the ACK and waits for its local write outcome
before submitting the reserved resend. An ACK write error ends the epoch and
prevents submitting that resend on the failed link; a resend error also ends the
epoch. The existing reservation remains held/uncertain, never rolled back or
replayed. This is a bounded local failure policy, not parity with every native
queue/recovery behavior. Capture/admission and durable reservation still precede
both writes. The generic confirmation helper does not newly ACK valid retries,
so the later generic receive step cannot duplicate this success ACK.

A key query alone, locally refused/unanswered request, duplicate/held answer,
missing root context, group/broadcast retry and comparison group window do not
gain success ACK authority from this change. Full key-query ACK sequencing,
redelivery ACKs for already-held work, count/base-key policy and native ACK
recovery remain unfinished. A crash between ACK and resend can still leave held
uncertain work; this chunk adds no automatic replay or recovery claim.

The existing real saved/fresh Signal/store workflow now exercises the normal
wrapper, verifies the committed reserved checkpoint and same-ID attempt before
observing a root-ID ACK, verifies a duplicate produces no new pair, and checks
fresh-bundle queries have no success ACK. An injected NodeLink test checks the
ACK is first, no resend is queued until its successful write outcome, and
ACK/resend write failures stop without another submission. It executes local
channels and synthetic crypto/store data, not a socket, APK runtime or account.

Validation:1325 workspace/all-feature tests passed, along with strict workspace/
all-target/all-feature Clippy, formatting, CLI-reference regeneration, APK
coverage and repository hygiene. No live request or runtime-native observation.

## Durable retry key-query ACK ordering — 2026-11-05

[Selected session preparation, job requirements, job-added key fetch and limits](apk/2.26.43.72-retry-query-ack/README.md)
now establish the selected query path after ACK submission. FG2.CbE performs
pre-ACK session preparation through163.A05/A04. Registration mismatch can
return the **old base key** after selected cleanup. The post-ACKFe5 callback
constructs the send job with SessionRequirement and, for that old key,
DifferentAliceBaseKeyRequirement. ItsA0J job-added requirement scan invokes
14t.A07 for an unmet device session/base-key requirement.
The selected enqueue hook is bound through0fW.A01 →Qlv selector31 →NhR.A04
→virtualJob.A0J; persistent-job insertion is conditional and precedes that hook.

Normal direct answer_with_confirmation now attaches a handled-retry ACK after a
key IQ's pending job and fetching/query/count checkpoint commit. NodeLink writes
ACK before IQ with the same failure stop rule as a reserved message. The later
IQ reply still must pass ownership/original/peer guards before resend; it does
not create a new receipt ACK. Refusal/duplicate/group/unsupported-context paths,
uncertain holds and no-automatic-replay policy remain unchanged.

0wT.A02 and the send-job constructor'sA02 were ruled out as key-fetch entries:
they are rejection diagnostics and argument validation, respectively. Native
14t.A03's selected backoff reset is recorded, not copied as an unconditional
Rust policy. Manager batching/backoff/final wire serializer, redelivery ACKs,
full recovery and every native job provider remain open. No live traffic.

Validation:1325 workspace/all-feature tests passed with the expanded query
reservation and two-route write-failure checks. Strict workspace/all-target/
all-feature Clippy, formatting, CLI reference, APK coverage and research hygiene
passed. No live server or native-runtime validation.

Follow-up: [selected key-query manager-to-wire construction](retry-key-query-wire.md)
is now traced throughBulkGetPreKeyJob/Message87. Its classic baseline matches
Rust; effective context/PQ/manager timing and recovery remain open. This
supersedes the selected final-serializer gap above, not those broader unknowns.

## Queued key-refetch writes retain reservation ownership (2026-11-06)

**Confirmed Rust defect fixed offline:** the normal runtime durably reserved a
retry key query, but queued its receipt ACK and query as unguarded writes. A
concurrent local change could remove/replace the pending job or settle its answer
while either stanza waited for the writer; the writer still admitted it.

The private answer wrapper now carries the exact query/job snapshot captured at
reservation. Both ACK and key IQ carry that guard through the writer queue.
Before encoding each stanza, the writer requires the unchanged pending job and
the matching fetching answer's query/count. A refusal consumes the queued item,
reports an error and ends this connection; it does not clear uncertainty, repair
state, start another query or authorize replay. The live callback is registered
only after the guarded query write succeeds, as before.

**Confirmed static reference, APK2.26.43.72:**
[14h.A00/Fe5 ACK-before-job ordering](apk/2.26.43.72-retry-ack-dispatch/README.md)
and [bulk future/callback completion](apk/2.26.43.72-bulk-key-job-completion/README.md)
remain the native evidence. This change closes a local queue ownership gap; it
does not claim native uses Rust's durable checkpoint format. Follow-up direct
field/caller searches reproduced the already retained future-map census. They
justify no new deadline: BulkGetPreKeyJob uses untimed FfK.get, while the manager's
60-second suppression and ordinary IQ timers are distinct mechanisms. The shared
legacy callback map and its reader aliases were already retained in the
[about callback audit](apk/2.26.43.72-about-callback-lifetime/README.md#shared-legacy-map-separate-ui-timeout).
Full sender admission, wire-callback expiry and native recovery remain open.

The queue regression holds ACK/query separately, removes/replaces the job or
settles the answer before either encoding boundary, and verifies refusal, no next
write, unchanged checkpoints and reopen. Unchanged control emits ACK then query.
Existing ownership tests also exercise missing/malformed/changed markers against
the writer guard and retain a frozen snapshot through reopen. The real retry
workflow asserts its reserved IQ carries a valid guard.

**Limits:** no lock is held across encoding/socket I/O; a mutation after the
writer check is outside this boundary. Direct message retry writes and callers
of public low-level answer/complete_refetch retain their existing contracts.
No wire grammar, timeout, automatic resend or live account operation changed.

Validation:1,446 workspace/all-feature tests passed, including the new queue
regression. Strict workspace/all-target/all-feature Clippy, formatting, CLI
reference regeneration, OpenAPI symbol validation, APK coverage freshness and
repository hygiene passed. Offline only.

## Ordinary parse-error convergence and corrected send gate (2026-11-06)

**Confirmed selected static / runtime implementation gap:** [ordinary handler, wrapper census and queued error path](apk/2.26.43.72-ordinary-receipt-parse-error-dispatch/README.md)
bind F10.A03 to ordinary15H and status19t handlers. Both selected BX2 variants
reach 1rg.A04 for LIL, which builds a fresh error487 context. The queued path
also creates distinct completion bookkeeping. Required offline aggregate actor
failures are therefore candidates for an evidenced error response; current Rust
silence is not claimed to match a native silence policy.

**Corrected earlier evidence interpretation:** 0Sf.A07(true) logs a bypass then
falls through into the same Message76 construction and enqueue path as false.
It does not return without sending. Fresh bytes match the old retained export;
its README and checksum now carry the correction explicitly. The receipt feature
flag in A0S selects this boolean, not a wire-suppression gate in A07. The later
writer bundle already closes the selected Message76-to-F3e.A05 binding.

No runtime changed in this research chunk. The next implementation must carry
confirmed native parse-error causes separately from local limits/storage faults,
use the fresh descriptor without recipient/edit, preserve capture/write ordering
and forbid resend/query/identity work. Queue recovery, unsupported metadata and
remote acceptance remain unverified; this is not permission to ACK all refusals.

## Ordinary required-device error responses implemented (2026-11-06)

**Confirmed local correction / offline**, grounded in the pinned APK2.26.43.72
[ordinary handler/wrapper/error-dispatch trace](apk/2.26.43.72-ordinary-receipt-parse-error-dispatch/README.md)
and [required DeviceJid accessor](apk/2.26.43.72-offline-required-receipt-actor/README.md).
F10.A03 required-actor failures reach 1rg.A04/A03 and its fresh error487
context. This implements the previously documented ordinary response gap for
proven required-device causes, not every local receipt refusal.

### Cause classification precedes output

The pure parser assigns `receipt.aggregate.native_required_device` only to a
missing required actor, known native-null PN/LID/hosted text, known wrong-class
text or generic classes, and the bounded binary service/nonuser/server-only
cast failures. Ordinary binary `Invalid` remains a generic refusal because its
upstream constructor/recovery error route is not established by a class-cast
proof. Local batch bounds, opaque/unknown representations, other unported
schema/time/type failures, self-user policy, and stored-row/storage failures
are NOT automatically mapped to487. The field is an internal diagnostic cause,
not an HTTP route, user-supplied switch or identity authority.

A supported recipient previously caused validate/record to return before
inspecting offline aggregate grammar. Both now preflight only the explicit
message_id aggregate in that no-fact-projection branch. A valid recipient still
does not grant ordinary fact projection, while a confirmed malformed required
actor exposes the same native cause. Other recipient/aggregate paths retain
existing behavior.

### Fresh root descriptor, all selected receive paths

`messaging::received_parse_error_confirmation` is a public, pure, validation-gated
builder: it returns a Node only for that native cause and supported root output.
It performs no capture, store mutation or I/O. Normal connection.rs, the CLI
comparison receive window and group-send post-send window now emit the response
after preserving the inbound frame. A genuine redelivery may receive another
NACK; a stored capture does not authorize local write replay.

The descriptor uses ROOT id/from/type/participant and error487, without
recipient/edit/category or aggregate children. It never substitutes the inner
message_id for the stanza id. Delivery type is omitted; other textual root types
are copied. Known-null root participants are omitted; known noncompliant or
out-of-range selected writer fields suppress construction. An omitted recipient
cannot make the fresh output unwritable. Existing supported root-id and sender
admission remains: this does not yet port arbitrary nullable sender output,
unknown metadata or every native subtype.

The normal driver sets receipt_recorded=false after the validation failure, so
it does not record facts, invoke retry answers or build an ordinary success ACK.
Its error ACK diagnostic is distinct from receipt success/facts. The comparison
window reuses existing outcome_unknown-before-write/written-after-write markers;
write failure ends the window. Group-send error ACK write failure ends its
post-send window without replay or claiming the original send failed.

### Verification and remaining scope

`offline_required_actor_parse_error_gets_fresh_487_after_capture` first timed out
waiting for the missing NACK. It now checks a real UserJid recipient, exact fresh
root descriptor, capture-before-write, repeated server delivery, unchanged
outbox/peer-identity/session snapshots, exclusion of a 4097-item local bound,
a following ping barrier, a synthetic write cut, preserved capture after reopen
and no automatic NACK replay on the next activation. Existing aggregate tests
retain atomic refusal with a valid sibling and unchanged facts/journal.

`ordinary_native_parse_error_cause_excludes_local_and_binary_refusals` binds the
cause distinction. `ordinary_parse_error_descriptor_copies_only_selected_root_fields`
checks literal LID0 projection, selected writer refusals, null participant omission,
codec output and delivery-type omission. Full CLI typechecking covers the two
additional receive call sites; they are not represented as independently observed
network traffic.

Unknown native constructor/queue/recovery behavior, other parse-error causes,
full nullable sender grammar, all request families and live acceptance remain
open. No new query, resend, identity change or live account operation occurred.
Run `cargo test --locked --workspace --all-features -- --test-threads=4` plus
strict workspace Clippy and the existing CLI/OpenAPI/coverage/hygiene gates.

## Aggregate timestamp native error responses (2026-11-06)

**Confirmed static / fixed offline:** [required/defaulted Long parser chain, branch countercheck and regression table](apk/2.26.43.72-receipt-timestamp-errors/README.md)
bind 0Tp.A07/A0L/A09 and A08/A09 to LIL and the existing ordinary fresh error487
path. Missing required offline item timestamps and malformed supported signed-long
item text now have a distinct native_timestamp validation cause; key item absence
still uses its existing clock default. Opaque representations and local/storage
failures are not automatically NACKed. Normal and selected CLI receive paths use
the same cause predicate/builder. Recipient-bearing participants aggregates also
preflight this grammar without granting new fact projection.

The hypothesis that root delivery gates key-aggregate parsing was refuted by
following 028c ->03aa ->02d0: only optional biz/privacy extraction is skipped.
Existing non-delivery key routing and read/played facts remain unchanged. The
new numeric codec regression and extended actor check fresh response fields,
capture-before-write, no fact/identity/session mutation and default/valid-number
controls. Existing atomic-fact and write-cut/no-replay tests remain. Ordinary root
timestamps, complete binary/native clock/storage behavior and live acceptance
remain open. No account operation or new request family.
