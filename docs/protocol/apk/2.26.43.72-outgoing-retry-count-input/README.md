# Retry count input cannot be reset by minimal construction

**Confirmed selected static plus Rust regression**, APK2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX fallback exports; method PCs are bytes and branch
units16-bit. No APK execution, new REA Evidence ID or live account operation.

## Input binding and wire increment

The retained [EuN.A01](../2.26.43.72-encrypted-child-metadata/chat-stanza-parser.txt)
reads each enc count with Java i32 parsing/default0, checks against an already
positive aggregate, and stores the current value in Ewa.A0E at093e or0ce8.
[Ewa.A01](../2.26.43.72-encrypted-child-metadata/chat-context-builder.txt) loads
A0E at00e6 into v27 and passes it as the11th Dxs constructor argument at01ae.
[Dxs constructor](../2.26.43.72-encrypted-child-metadata/chat-context-constructor.txt)
receives that argument as v24 and stores it in A0C at008c.
[Fresh F1Z.A04](incoming-count-getter.txt) returns Dxs.A0C via3q5.A0J; its Dxr
branch instead reads Enw.A0D, whose upstream producer is not traced here.
[3q5.A0J](nullable-count-default.txt) defaults a missing Number to0, otherwise
returns intValue unchanged.

The retained [SendRetryReceiptJob constructor](../2.26.43.72-outgoing-retry-timestamp/job-constructor.txt)
stores F1Z.A04 into retryCount at0106. Its
[job run](../2.26.43.72-outgoing-retry-timestamp/job-run.txt) adds1 before EYI
construction on BOTH selected branches (0136 and02aa). The
[writer](../2.26.43.72-outgoing-retry-timestamp/writer.txt) emits the envelope's
count. This is progression from the incoming aggregate, not a new constant1
simply because this client has no local receipt row yet. Full native signed
addition overflow/range policy is not ported by this change.

Positive incoming retryCount also selects the native key-load branch, as traced
in [stateless/PQ inputs](../2.26.43.72-enc-state-pq-retry-inputs/README.md) and
[prekey direct-distribution ownership](../2.26.43.72-retry-prekey-distribution/README.md).
Incrementing the integer alone would still omit native preparation. Runtime
carried-key integration therefore remains required work, not completed here.

## Confirmed wrong construction and bounded correction

Before the fix, receipts::queue_retry_for_incoming_with_cause always passed1 to
the minimal builder, regardless of incoming enc count. The new real-store
regression failed on that code: incoming count1 returned one queued request,
whose construction would reset the count to1 without carried keys.

inbox::incoming_retry_count now exposes the same ordered signed-Java-i32 parsing
already used for whole-child admission. The final aggregate is retained; an
already positive aggregate must match the next child. Missing count defaults0;
no immediate enc remains0 for the existing diagnostic API. This helper does NOT
replace ciphertext type/body validation or claim successful authentication.

The currently bounded minimal retry queue now handles only aggregate0. Other
counts and malformed/out-of-i32/inconsistent counts return Ok(0) with a redacted
unsupported-progression/key-preparation warning, before any new reservation or
wire construction. The raw capture remains caller-owned; no new ACK/delivery
confirmation is authorized. This is an explicit interim local coverage boundary,
not a claim that native rejects all nonzero counts or that count progression is
fully implemented. Negative inputs are native parser-valid but outside this
minimal supported retry scope. Full carried-key/count/stateless integration is
still the next implementation requirement.

The regression covers1,+01,Arabic-digit1,-1,i32-max,overflow,malformed counts and
ordered pairs0→2/2→0. It preserves captures, checks unchanged checkpoints and
journal after each non-queue outcome, reopens the store, queues one aggregate0
request with wire count1, reserves it, and verifies changed-count/zero redelivery
cannot rewrite or reset that existing request. Existing immutable historical
wire is not reconstructed by this guard; historical pending receipts without
input provenance are not retroactively reclassified. No local retry receipt
limit is promoted to a native limit.

Reproduce apk-smali with signatures/exceptions for F1Z.A04 and3q5.A0J. Referenced
same-build exports retain their own checksum manifests.
[checksums.json](checksums.json) pins all files here. No server acceptance or
account-restriction causality is established.
