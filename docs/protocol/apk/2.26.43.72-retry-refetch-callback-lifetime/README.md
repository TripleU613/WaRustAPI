# Retry refetch callback lifetime — APK 2.26.43.72

**Confirmed static / local transport correction.** Build code264307238, APK
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected checked-DEX exports retain signatures and exception tables;
these local exports are not REA Evidence envelopes. **Observed:** no APK/runtime
or account traffic. **Inferred:** per-connection, one-shot callback authority is
the compatibility boundary for normal Rust refetch completion. **Unknown:**
exact IQ timeout/elapsed deadline, full reader/writer callback-source binding,
job retry/recovery parity and all other query families. No timeout is invented.

## Pending future registration and cancellation

The retained [BulkGetPreKeyJob.A0L](../2.26.43.72-retry-key-query-wire/bulk-prekey-job-run.txt)
allocatesFfK and inserts it by query ID into **0Th.A05** at0294..02b4. It dispatches
through0Sf.A05, calls0ln.A0E(id), thenFfK.get. The newly retained0ln.A0E body
**removes** an entry from0ln.A0I under lock. It is not future registration or a
newly established callback timer. The prior wire bundle's registration wording
has been corrected; this negative result must not be turned into a deadline.

The five-method0Th class is retained completely. Its A03(Exception) locksA05,
invokes1ue.BsR(exception) for every pending future, clears the map, then unlocks.
[0YB.A05 disconnected handler](../2.26.43.72-iq-shutdown-eligibility/disconnected-handler.txt)
constructsIOException and calls0Th.A03 at01f4..01fa. Thus a persisted send job is
not by itself proof that an IQ future remains active after disconnect.

The retained [0YU.handleMessage](../2.26.43.72-iq-shutdown-eligibility/message-handler.txt)
selected completion branch0456..0494 locksA05, removes the message.obj entry,
unlocks and invokesBsT(null) only for a found callback. Removal precedes that
invocation. Full producer authority for every such message is outside this
chunk. 0ln.A01 separately admits exact type=result/error toCK4/Bvj; other types
log/assert rather than dispatch a result/error callback. Its corruption handlers
are not evidence of retry authorization.

0Th.A02 was inspected as a candidate registration method. It manages **A04**,
wrapping ACK callbacks with2Gc and elapsedRealtime metadata; it is not the bulk
key query's **A05** future insertion. Retaining the complete class makes that
separate map and bounded negative result inspectable.

## Rust correction

Previously normal receive invoked complete_refetch for any IQ. Its persisted
Refetch/checkpoint guards could match a query from a previous connection or
process. ConnectionEpoch is a process-local counter and starts again after
restart, so persisting only that number would not fix the lifetime mismatch.

Each drive invocation now owns a freshActiveRefetches set. A query is registered
only after send_retry_answer returns a successful **local write** outcome; the
pending job and fetching marker already committed before ACK/query writes.
The sequential driver records this before processing the next queued inbound
node, so a fast response cannot overtake registration in the receive loop.
Only IQ result/error with a registered exact ID can invoke complete_refetch.
That ID is removed before awaiting completion. Wrong tags/types/IDs do not consume
it; duplicate or locally failed completions cannot invoke it again.

Disconnect/error drops the set; a new drive/process does not repopulate it from
saved jobs. Pending durable reservations remain held/uncertain without replay,
reset, new fetch or session/pin changes. Existing pending-job ownership/original/
peer-state guards still apply after the live callback check. This is not full
native job cancellation/retry recovery. No guessed timeout, sender provenance,
server acceptance or ban-prevention guarantee is added.

The public low-level complete_refetch API remains available for callers that
already own a live query; normal runtime now supplies that authority through
the private registry. Calling that low-level helper directly is not the normal
network ingestion path and does not itself establish connection authority.

## Regression and reproduction

The synthetic store/callback regression covers durable-only requests, unrelated
messages/IDs, same-ID get/set/missing type, live error settlement exactly once,
failed local ownership settlement without a second callback, and disconnect/
reopen with even the same numeric epoch. Checkpoint/outbox/session/pin snapshots
are preserved for unowned replies. It runs no network/Android method; the normal
driver uses the tested registry directly.

Reproduce with `target/debug/xtask apk-smali <pinned-apk> --class 'LX/0ln;'
--method A0E|A01 --signatures --exceptions` (one method per invocation), and the
same tool with exact classLX/0Th; and no method filter for its five methods.
Prior linked bundles retain insertion/disconnection/completion callsites.
Checksums cover every other file here.
