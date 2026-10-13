# Post-quantum key bundle boundary

2026-10-13: identity preview/application no longer defer every PQ reply;
verified primary replies derive hybrid sessions
([hybrid identity refresh](hybrid-identity-refresh.md)). The consumable classic
bundle parser below still refuses selected PQ input. Device-key batches and group
fan-out now accept verified PQ rows ([hybrid fan-out](hybrid-fanout.md)); the
`invalid_bundle` batch classification below now applies only to invalid PQ material.

Pinned APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.

The shared directory parser now refuses a selected user's `pqkey` instead of
silently dropping it and returning a consumable classic Signal bundle. This is
an explicit backend limitation, **not implemented PQ support or a native ban on
post-quantum keys**. Classic replies and unrelated devices retain existing behavior.

## Native processing and backend gap

The [four-topic recipe](apk/recipes/2.26.37.71-pq-bundle-boundary.json) reproduces
[nine files / 94,712 bytes](apk/2.26.37.71-pq-bundle-boundary/targets.json) with
full signatures, register flow and catch tables. Complete
[CQc.A00 REA Evidence](rea/2.26.37.71-pq-bundle-boundary/bundle-processing.json)
independently retains its signature checks and encapsulation path.

`Bot.A03` reads selected PQ id/value/signature into CIv. `14J.A04` passes
admitted metadata to `0jc.A0L`, which constructs `KEMPublicKey` and optional
PQ fields in C6P when the required material is available. `CQc.A00` verifies
the signed EC key and, when its KEM field is present, the Kyber signature.
It encapsulates the KEM key and incorporates the result in session construction.
Discarding PQ data before this path is not an equivalent hybrid session.

`0jc.A0L` also has a logging/fallback path for incomplete nullable PQ material.
Native parsing, effective flags, all such fallback cases and complete KEM
algorithms/record/wire compatibility remain unported. Refusing every selected
`pqkey` is this client's conservative unsupported-input boundary, not a claim
that WhatsApp rejects all those nodes.

The [backend capability audit](pq-backend-capability.json) verifies the relevant
installed **wacore-libsignal 0.7.0** source files against the cached package archive
and its [Cargo.lock](../../Cargo.lock) checksum. Its PreKeyBundle fields are EC
only; pending-prekey construction sets Kyber fields to None, and legacy pending
conversion rejects present Kyber fields. Storage proto fields do not establish
KEM cryptographic support. This audit describes the locked package, not future
upstream releases. No dependency or crypto primitive was changed.

## Shared callers and preservation

`prekey_bundle` / `prekey_bundle_for` select and validate the exact response user
before refusing its PQ presence. Their existing root/list/user, duplicate, JID,
error, classic key width/type and signature boundaries remain unchanged.
PQ on an unselected device does not block a classic recipient. Direct send,
device key batches and group fan-out use this shared consumable parser, so they
cannot construct a new classic bundle by deleting selected PQ input.

A crate-private classic metadata projection shares the exact selector/parser
for identity inspection, read-only preview and prior-read guards. It validates
only the classic signed prekey, not the PQ key/signature. Existing PQ presence
diagnostics and `pq_session_unported` preview/application deferral remain intact;
no session derivation or pin adoption uses this projection on a PQ candidate.
`signature_valid` continues to mean classic signed-prekey validity only.

(Superseded 2026-10-13: a PQ reply the outbox cannot use now suspends its
intent with a reason, see [hybrid-outbox.md](hybrid-outbox.md).) Direct preparation fails with a fixed unsupported error before crypto/session
use or an outgoing message reservation. Device batches retain their exact reply
and existing `invalid_bundle` terminal classification, blocking later pages
without changing keys. This does not manufacture a server refusal or retry.
Account/peer keys, sessions, message IDs, prior requests, uncertain send facts,
server waits and journal history are preserved. Existing already prepared or
committed packets/history are not automatically rewritten or reclassified.

## Verification and remaining work

The regression failed before the change with selected PQ input returned as a
classic bundle. It covers empty and populated placeholder PQ nodes, classic
metadata diagnostics, selected versus unrelated devices and existing classic
validation. Placeholder bytes are not a valid native KEM fixture.
The direct outbox integration test verifies no table/journal changes, no message
reservation and no replay across reopening. The batch regression verifies a
selected PQ response blocks later pages and preserves crypto state. Existing
identity-refresh PQ deferral, classic metadata/signature inspection, pin/session
application and preview tests remain effective.

Full post-quantum directory parsing, signature validation, native KEM algorithm
identification, hybrid derivation, key inventory/publication, incoming ciphertext,
record migration and fresh live interoperability still need implementation.
Some recipients requiring new PQ sessions therefore remain unsupported. The
low-level encryptor can still use an established classic session without a new
bundle; any path requesting a consumable bundle from a PQ reply now refuses,
including legacy callers holding a session. Per-send directory query policy
is not changed by this guard. No live account
operation or new key lookup ran for this change; no ban-immunity claim follows.

Validation passed 970 workspace/all-feature tests, strict all-target Clippy,
warnings-denied Rustdoc, the outbox refusal/reopen test with TCP/Noise/Signal
features, 26 default CLI/API-symbol tests and the experimental CLI build.
The four-topic APK bundle reproduced exactly, and backend source files matched
the locked package archive. No PQ cryptographic compatibility was verified.

2026-10-12 follow-up: [PQ signature diagnostics](pq-signature.md) now verify the
original value bytes separately from the classic signed prekey. KEM validation
and session support remain unported; no guard or adoption permission is lifted.

## The PQ predicate `Can.A04` (2026-10-13, retained evidence only)

A cross-bundle search of the retained 2.26.37.71 evidence (no APK in the
session) finds one Boolean predicate, `LX/Can;->A04(Jid)Z`, deciding PQ work
for a JID at five distinct call sites. Each offset was re-checked by an
independent verification pass the same day, which corrected the first version
of this section (noted below).

- **`SyncDeviceAndResendMessageJob.A0G`**
  ([sync-resend/job-run](apk/2.26.37.71-sync-resend/job-run.txt)): **confirmed
  static**, `Can.A04` of the job's chat JID (`018e..0194`, `move-result v16`)
  is the last argument of `DFC.<init>(…, Z)` (`029e..02a4`), and an
  `iget-boolean DFC.A04` adds `key_fetch pqsupport="true"` (`03be..03d4`).
  **Inferred:** the constructor stores that last argument in `A04`.
  `DFC.<init>` is not retained, so this follows from the type order only
  ([sync-resend](sync-resend.md)).
- **`DLp` identity-notification worker**
  ([identity-worker-run](apk/2.26.37.71-identity-notifications/identity-worker-run.txt)):
  `Can.A04(user)` (`02a8..02b4`), together with `0Fs.A0U(user)` (`028c..0294`),
  gates the second prekey fetch `14L.A08` (`02d0..02d6`), whose last Boolean
  is the constant 1 ([identity-notifications](identity-notifications.md)).
- **`DNe` message worker**
  ([message-worker](apk/2.26.37.71-read-receipt-persistence/message-worker.txt)):
  `Can.A04(jid)` (`04a2..04aa`, into `v18`) is the last Boolean argument of the
  prekey-fetch scheduler `14L.A08(0Cy, DeviceJid[], int, int, int, Z, Z)`
  (`04b6..04bc`).
- **`By0` broadcast-receipt decrypt owner**
  ([decrypt-owner](apk/2.26.37.71-broadcast-receipt/decrypt-owner.txt)):
  `Can.A04(CjN.A05)` (`01d6..01de`) guards a key-store branch (`01de..01f2`).
- **`DNu` send-message runnable** (retained in
  [history-identity-worker](apk/2.26.37.71-token-import/history-identity-worker.txt)
  and [group-send-runnable](apk/2.26.37.71-groups/group-send-runnable.txt)):
  after "send message runnable creating e2e message job" (`2c82`), the result
  (`2f1c..2f24`, `v63`) is passed to `SendE2EMessageJob.<init>` (`3040..3046`)
  as the 9th of its 10 trailing Booleans. **Corrected:** the first version
  placed this in the history identity worker with its use "not traced". The
  receiving field name is not retained; the job has a `fetchPqPrekey:Z` field
  ([group-send-job-hash-check](apk/2.26.37.71-groups/group-send-job-hash-check.txt)
  `046c..0470`), a plausible but **unconfirmed** match.

`Can` has other PQ predicates too: `Can.A01()`/`A02()` gate the
`remainingPqPreKeys` path of the prekey count worker
([count-notification-worker](apk/2.26.37.71-prekey-count-notifications/count-notification-worker.txt)
`0056..0076`), and `Can.A03()` gates the `pqSessionUpgradeManager` block of
`SendE2EMessageJob` (group-send-job-hash-check `1750`, `178e`).

**Unknown:** the bodies of `Can.A01`..`A04` (feature properties, per-JID state
or both) are in no retained bundle, so whether, when and for whom the native
client asks for PQ keys is not established. **The `Can` predicate bodies are
the next trace** that resolves the `pqsupport` producer for all of the above.
Until then this client sends no `pqsupport` and never claims PQ support;
adding it on a guess would change live requests without evidence.
