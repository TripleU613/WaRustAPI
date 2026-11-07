# Peer retry parser and separate responder — APK 2.26.43.72

**Confirmed selected static / implemented error-response distinction**, build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
xtask fallback, complete selected method exports; no target execution, live account
operation or REA Evidence IDs.

## Registration and common parser

[14h constructor caller census](base-constructor-callers.txt) finds seven direct
callers:14i,14k,15H,160,17t,181,19t. Selected complete initializers show
[14i221](14i-registration.txt), [14k212](14k-registration.txt),
[1602](160-registration.txt), [17t263](17t-registration.txt),
[18169](181-registration.txt). Previously retained15H1 and19t277 complete the
selected census; the census is not proof of dynamic provider/runtime coverage.

The [upstream1rU router](../2.26.43.72-receipt-router-boundary/receipt-route.txt)
selects263 for type=retry/category=peer, unless a leading participants/message_id
selects ordinary/status forwarding first. [17t.A04](peer-handler.txt) calls the
SAME F10.A04 retry parser at0030 as160 does. The fresh
[direct-call census](parser-callers.txt) finds exactly160.A04 and17t.A04.
The [complete parser](../2.26.43.72-retry-receipt-keys/receipt-parser.txt) therefore
still owns required retry/registration, version, count, time and original-ID
admission on the peer route; peer category does not bypass that grammar.

17t.A04 has no catch table. A null parser result returns false at0378, which the
[14h complete base](../2.26.43.72-retry-ack-dispatch/base-handler.txt) routes to2Cx
at016e (copied-context error487). An escaping LIL converges through the same
legacy/queued error machinery to fresh1rg error487; the
[complete policy](../2.26.43.72-ordinary-receipt-parse-error-dispatch/README.md)
retains wrapper flags and downstream write boundaries. [BLn](wrapper-entry.txt)
checks the registered code and delegates; it is not the internal exception body.

## Correction to the previous exclusion

The earlier upstream-routing fix correctly excluded peer receipts from ordinary
resend/refetch but also suppressed their proven schema/version error responses.
That temporary assumption is now refuted by17t's shared-parser call. Rust splits
retry_parser_route (ordinary OR peer) from ordinary_retry_route (ordinary only).
Error classification uses the former; public parse_retry, successful handled ACK
and ordinary resend/refetch remain behind the latter. Leading message_id and
nontext-selector guards still apply before both. No malformed ordinary/peer
schema is treated as an authorization to resend or mutate identities.

Regression cases cover missing retry/registration, malformed count/time, missing
nested ID, unsupported version, valid peer schema and early aggregate precedence.
Schema errors use fresh root-ID487 without recipient/edit/category; unsupported
version uses the existing copied487 preserving recipient/edit. Valid peer schema
still gets no ordinary success ACK, RetryRequest or resend. Existing saved-session
workflow tests preserve all ordinary outbox/checkpoint/session/identity rows for
peer receipts. Full peer success handling is not implemented here.

## Separate peer response policy, still unported

After parsing,17t uses the root from converted to DeviceJid and peer-message
lookup0g9.A04(device,id), not the ordinary outgoing-message lookup. Sender
eligibility branches call Exz.A01 with0jL.A01 or09F.BRq plus membership in
0jD.A0A's set. Exact predicate meaning and current state are not established.
Ineligible senders return true without a resend. Missing peer records and messages
older than now minus5184000000ms (60 days) likewise return true. The age comparison
is strict: equality proceeds. It updates retry metadata via0fz.A0L before testing
count>4; high counts then return true without enqueueing a send.

Eligible lower-count handling converts the root sender again, optionally validates
ADV through10N.A0D, prepares peer retry bytes via163.A05 and enqueues
SendPeerMessageJob through0fW.A01. ADV failure returns true without send;
missing required converted device at0368 throws IllegalStateException, not LIL.
These paths must not be replaced by the ordinary responder. True/false return,
queue completion, descriptor construction and actual wire/server acceptance are
separate facts. Full predicate bodies, crypto/peer-job construction, current
provider/feature/readiness state and completion/no-replay policy remain open.
No blanket success ACK or peer resend is added by this change.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-callers "$APK" --class 'LX/14h;' --method '<init>'
# Each retained constructor:14i,14k,160,17t,181 (15H/19t previously retained).
target/debug/xtask apk-smali "$APK" --class 'LX/17t;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/17t;' --method A04 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/14h;' --method BLn --signatures --exceptions
target/debug/xtask apk-callers "$APK" --class 'LX/F10;' --method A04
```

checksums.json covers every other file. No APK/private state or bulk tree is included.
