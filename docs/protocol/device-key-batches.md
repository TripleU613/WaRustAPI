# Frozen directory-key batches — 2026-10-09

2026-10-13 follow-up: rows carrying a verified `pqkey` now answer; a failed KEM
signature is `invalid_signature` and other PQ defects `invalid_bundle`.
See [hybrid fan-out](hybrid-fanout.md).

**Confirmed local implementation:** the comparison group/status sender now
queries every device needing a bundle in durable bounded batches and uses only
complete validated results for [shared fanout preparation](group-fanout.md).
A 130-device fixture produces five queries (32/32/32/32/2). This removes the
prior single-query width failure; it does not complete normal group outbox
integration, own-device sync or native directory policy.

## APK evidence and unknown native policy

**Confirmed static, comparison 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
`0y0.A00(VL)` builds per-target `user` nodes at `1786..17fc`, then constructs
`iq/get`, `xmlns=encrypt`, `to=server`, `key/user@jid` at `18a2..193a`.
The next diagnostic names this `get-pre-key-batch` (`193a..1960`). The existing
[complete writer method](apk/2.26.37.71-groups/group-ack-writer-dispatch.txt)
and [recipe](apk/recipes/2.26.37.71-groups.json) retain full operands/catches.
This re-anchors the current builder comment to 37.71; the older transport
history's 37.70 attribution remains historical, not cross-build proof.

**Confirmed static / unknown runtime:** the loop can add `reason=identity`
for a subset held in a supplied HashSet (`17a0..17e2`). A supplied boolean can
add `pqsupport=true` (`1806..1824`); feature 0x6530 and context predicates gate
`context_jid` (`1824..1886`). The producer values, complete subset semantics,
effective flag and native batching/retry policy are **unknown / unported**.
Those branches do not establish universally mandatory fields. Rust retains its
existing supported classic bare query and does not invent those inputs.

**Confirmed static:** `B7H.A06` preserves device/category in Signal address
construction, and `CQc.A00` verifies the signed device key before session
creation. See the [identity evidence](peer-identity.md) and retained
[sender-key address method](apk/2.26.37.71-core/sender-key-address-build.txt).
Rust's existing `prekey_bundle_for` selects the exact PN/LID user, agent and
device, rejects duplicate matching rows and parses key widths. Batch completion
also verifies signed-key signatures before claiming a usable directory answer.
This is not peer pin replacement or human identity verification.

**Local bounds:** 32 devices per query is the existing Rust builder's width,
now shared as `messaging::KEY_QUERY_MAX_DEVICES` /
`device_key_batches::QUERY_WIDTH`. A frozen plan has at most 4,096 targets,
matching the local binary child bound. Neither is a confirmed WhatsApp server,
group-size or native scheduler limit.

## Reservation, response and assembly

`reserve_next` validates ordinary PN/LID target devices, preserving user,
namespace, agent and device. Unsupported/hosted/interop/nonzero-agent routes
refuse. Ordinary Plain/AD representation is normalized for stable comparison;
input order is sorted and duplicate logical devices refuse. A reordered equal
set therefore reuses the same plan, while missing/new devices or a changed
namespace do not silently begin a new lookup.

The plan is saved in the original account's encrypted checkpoints under a
request-ID-derived key. Each page transitions from ready to `outcome_unknown`
with its unique query ID **before** the caller writes a node to the socket.
Reservation is atomic. Unknown, refused and invalid pages block; this is a
conservative local no-retry policy, not a native/server requirement. A process
that stops after reservation keeps the same unknown outcome on reopen. It
cannot create a new query merely because no response was saved.

`settle` accepts only an authenticated IQ with the reserved ID, as supplied by
the caller. It preserves the exact encoded matched response privately before
recording answered/refused/invalid. Unrelated IDs and terminal duplicates make
no mutation. A top-level error is `query_refused`; a matching user error is
`device_refused`. These are local classifications, not the server's detailed
reason. Raw code/text/addresses/key material remain in encrypted response wire.
No server wait or challenge is cleared to continue the batch.

A result must contain one supported list, every expected device exactly once,
no foreign/aliased/missing device, valid key widths and valid signed-key
signatures. Structural coverage is checked before bundle parsing so a duplicate
is diagnosed as `duplicate_device` rather than a generic parser failure.
Other fixed classifications are `invalid_reply`, `invalid_list`,
`missing_device`, `foreign_device`, `invalid_bundle` and `invalid_signature`.
Strict completeness/foreign-row refusal is **our local guard**, not a traced
unconditional native parser rule. Remaining pages stay unreserved after failure;
there is no partial fanout fallback or repeated directory query.

`assembled` requires every page answered, re-decodes saved original wire,
checks its query ID, validates exact coverage and verifies signatures again,
then composes a private synthetic `iq/result/list` for Snapshot preparation.
That synthetic node is **local composition**, not another server response or
network request. Existing pin checks remain in the staged fanout encryption;
a valid signature alone cannot adopt a changed peer identity. Directory
completion is neither a send reservation nor message acceptance/delivery.

The sender keeps its captured crypto snapshot across the batch sequence.
A concurrent ratchet/pin/own-key/exposure change still refuses at final send
reservation. No chat bytes are written until shared crypto and the original send
intent have committed atomically. Its comparison-only post-send retry path is
separate; no retry behavior is widened by this lookup change.

## Historical and diagnostic boundaries

The old CLI wrote `message:<request_id>:keys` without frozen page provenance.
Those records remain untouched. If such an outcome exists without the new
batch plan, reservation refuses with an explicit provenance diagnostic; it
cannot manufacture a fresh query or pretend the old attempt has no history.
Inspection explains that frozen counts are unknown and the original is preserved.
An evidence-backed legacy migration remains **unimplemented**, not a server
constraint. This guard also applies when changed settings would make the new
required-bundle set empty. Old acknowledged sends still return locally through
their original preflight and are never resent to create batch records.

```sh
whatsapp-cli --profile NAME group key-queries --request-id EXISTING_REQUEST_ID
```

The offline command prints only frozen device/page counts, unreserved,
`outcome_unknown`, answered, refused and invalid counts, `complete`, and the
nullable fixed `blocking_reason`. If neither a batch nor a historical outcome is saved, the command returns
JSON null; legacy outcomes lacking a plan produce the fixed local error above.
It never prints devices, query IDs, signatures, key bytes, raw responses or
message content, and never queries, reserves, reconnects or sends. Since
2026-10-13 `GET /v1/groups/key-queries?request_id=` is also served read-only by
`serve --store-routes` ([store routes](../api/README.md#store-routes-opt-in)),
with the same JSON; a record the summary cannot count answers 422 with its
fixed reason.

## Verification and remaining work

**Confirmed local:** generated signed-key fixtures cover 130 unique primary/
companion PN/LID targets across all five actual encoded/decoded response pages.
Every device is queried once; page reservation blocks another call until a
response is saved. Complete responses survive reopen and reordered inputs reuse
them without another query. Unknown reservations survive restart. Changed target
sets, empty-set bypass, foreign/namespace-mismatched/duplicate/missing devices,
whole/per-device refusals, malformed types/lists/key widths/signatures, corrupt
saved reply/ID and unattributed legacy outcomes retain original state with
redacted diagnostics. These operations do not change any peer pin, ratchet,
sender key, outbox entry or event. CLI subprocess coverage verifies offline
key-query summary/null behavior, address redaction and unchanged encrypted state.

Fresh live large-group lookup and sending remain **unverified**. No new group or
status message is sent solely to validate batching. Normal actor group query
state-machine integration, identity policy composition, complete own inventory,
optional native query fields and historical migration remain unfinished. Full
checks and saved-account inspection are recorded after verification.

Directory-batch validation: 758 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and formatting pass.
Offline projection upgraded zero events; saved peer keys, action outcomes,
receipt ACKs, outbox, events and normalized retained inspection stayed unchanged.
Twelve existing request files were inspected offline: no new batch records,
eleven with neither batch nor historical key outcome, one with a preserved
legacy outcome and unknown frozen counts. No server request or message was sent.

The existing groups APK recipe now anchors get-pre-key-batch and maps its writer
to messaging.rs and device_key_batches.rs for future release diffs. Exact
re-export changed only targets/checksums metadata, not native method bodies;
all 44 files / 928,943 bytes / 43 selected methods reproduce. Cached self-diff
is 0.047877 seconds, not a claim of cross-release equivalence. Large-group live
lookup, native optional-field gates and normal group actor wiring remain pending.
