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
