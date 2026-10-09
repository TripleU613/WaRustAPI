# Receipt ACK participant presence

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Three-topic recipe](apk/recipes/2.26.37.71-receipt-ack-correlation.json),
[complete selected methods](apk/2.26.37.71-receipt-ack-correlation/targets.json).
Existing [ACK parser](apk/2.26.37.71-message-ack/ack-wire-parser.txt),
[builder](apk/2.26.37.71-message-ack/ack-attributes-constructor.txt) and
[constructor](apk/2.26.37.71-message-ack/ack-attributes-storage.txt) are reused.
Offsets are method-relative bytes. This fixes local explicit-receipt queue
correlation, not per-user receipt admission or primary identity cleanup.

## APK attribution and limits

**Confirmed:** 1Qy.A02 reads from, participant, recipient, id, class, type and
edit into CSu at 0018..00a6, then calls CSu.A00. That builder forwards its
nullable participant A01 as constructor argument 2; Cb1.<init> stores it in
Cb1.A01. Cb1.equals compares the nullable participant fields at 0086..00a6:
null matches null, present versus null returns false, and two present values
must compare equal. Other equality fields include from, ID, class, recipient,
type and edit, plus additional metadata. This is not ID-only equality.

1Dz.A1E logs ConnectionThreadRequestsImpl/onAckReceived; stanzaKey and forwards
0hs/Cb1 to 1Dw.BaE. Complete BaE implementation/registration, pending-request
matching, native receipt-special-case lifecycle and JID equality internals are
**unknown** in this scoped trace. The local correction uses the explicit saved
receipt route, not a claim of complete Android callback parity.

**Corrected attribution / negative:** the previous Rust comment named CZH.equals
alongside 1Qy.A02. In this build CZH.equals compares BYJ, 1gc, 0Cy and other
fields; it is not the Cb1 produced by this ACK parser. The separate retained
method records that mismatch rather than deleting the negative result. No claim
about every other CZH caller is made.

## Local correction and durable behavior

**Confirmed locally:** record_ack formerly tested participant equality only when
the incoming ACK carried the attribute. A group read/retry receipt with a saved
participant could therefore settle on matching from/ID/type but no participant.
The new retry regression failed on that exact case before the guard correction.

The guard now first requires equal participant **presence**, then a valid equal
logical address when present. Malformed, missing, extra or foreign participants
cannot settle a reservation. Existing user/server/agent/device comparison ignores
wire-encoding kind only; it does not drop devices, merge PN/LID or invent aliases.
Recipient/edit ACK routes remain unsupported. Error-bearing ACKs require the
same route match before recording refusal.

The group-read regression leaves the saved outcome_unknown status intact across
restart after an incomplete ACK; it also rejects an incomplete error ACK,
confirms no replay, and accepts a later complete matching ACK. The retry regression
checks missing participant plus agent/device/server mismatches and equivalent
encoding acceptance. Existing direct receipt fixtures test extra participant,
recipient and edit rejection. Original wire, IDs and reservations stay unchanged.
These are offline fixtures, not a new server-acceptance or live-receive result.
No private account migration/reset, message send or live request was performed.

Existing receipt-outcomes inspects pending/outcome_unknown/server_acknowledged/
refused states; no command, flag or HTTP route was added. Full receipt_user
admission, primary cleanup and verified peer recovery remain unfinished.

```sh
cargo test -p whatsapp-api --all-features transport::receipts::tests
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-ack-correlation.json \
  --output docs/protocol/apk/2.26.37.71-receipt-ack-correlation --check
```

Validation: 812 workspace/all-feature tests passed (including all 19 receipt
tests, CLI reference regeneration and API library symbols), strict all-target/
all-feature Clippy, warnings-denied Rustdoc, formatting and diff hygiene. The
three-topic source bundle reproduces exactly: eight files / 37,422 bytes; cached
self-diff verifies three unchanged topics and four indexed artifacts. No live
validation is claimed.
