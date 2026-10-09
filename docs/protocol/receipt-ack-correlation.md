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

Follow-up: [generic ACK registration/dispatch](receipt-ack-dispatch.md) closes
BaE forwarding and full-key map removal for the generic path, with separate
ID queues and bounded normalization. Complete special-case/error/timeout and
queue ownership semantics remain unknown; no local guard was weakened.

## Saved receipt inspection diagnostics (2026-10-09)

**Confirmed local:** receipts::outcomes, exposed by the existing offline
`receipts` CLI, previously used filter_map/as-u8 casts for saved JSON wire bytes
and converted decoder/ID failures into an empty message_id. This could silently
drop non-numbers, truncate integers above 255, or show an unknown reservation as
an apparently ordinary outcome with no ID. A failing-before regression reproduced
Ok(outcome_unknown, empty ID) for a missing saved wire. This was our inspection
bug, not a WhatsApp response or an APK-imposed account requirement.

Inspection now decodes JSON as Vec<u8>, requires a receipt node and a nonempty
text ID, and returns fixed redacted diagnostics for missing wire, invalid bytes,
decode failure, wrong node kind or missing/invalid ID. It does not include the
raw decoder/serde error, recipient, route, contents or secret-bearing JSON.
Successful outputs retain their existing shape. The read-only inspection never
rewrites a reservation or changes pending/unknown/acknowledged/refused status.
One malformed row fails the inspection rather than reporting a partial list;
there is no automatic repair, deletion, retransmission or account migration.

This is local storage validation, separate from the APK 37.71 Cb1 correlation
source above. It adds no wire grammar or server reason. The regression covers
missing wire, an integer outside byte range, non-number data, undecodable bytes,
a receipt without an ID and a non-receipt node; each fixed error survives reopening
and the encrypted saved row remains unchanged. A valid reservation reopens with
its original ID and outcome_unknown. These synthetic fixtures do not establish
live ACK matching, receipt_user membership or primary identity cleanup parity.

Inspection correction validation: the new regression failed before the fix on
Ok(outcome_unknown, empty message_id), then passed with corruption/reopen
coverage. All **823 workspace/all-feature tests** passed (687 API unit, 4 codec,
9 crash, 52 CLI, 1 OpenAPI symbol and 70 xtask). Strict workspace/all-target/
all-feature Clippy, warning-free workspace rustdoc, default CLI tests,
CLI-reference regeneration, formatting, whitespace and repository hygiene
passed (1,913 local links; unchanged 84 checksum bundles / 18,046,975 bytes).
No real account was opened or changed and no live request was sent.

## Original saved-account inspection restored (2026-10-09)

**Observed offline:** the preceding missing-environment result was corrected
using the existing private key file beside profile `l`'s original store. File
permissions were private; the key stayed in memory and the child process's
configured environment variable. No value was printed, committed or replaced.
The rebuilt `receipts --json` CLI opened the existing account and returned
**nine** reservations: **eight read**, **one retry**, all **server_acknowledged**,
with nonempty message IDs. Two further inspections across reopening returned
identical full logical outcome lists. No message IDs or account numbers are
included in this research note. No socket, live request or retransmission ran.

The database file's byte digest changed on opening. **Confirmed source:**
ProtocolStore::open uses a metadata/table-validation write transaction and
commits it even for an existing schema before returning the store; outcomes
itself uses store.read. This is why byte identity cannot establish read-only
logical account behavior here. No first-run full database semantic snapshot was
captured, so this observation does not claim whole-account byte/row equivalence.
The verified narrower result is stable logical receipt outcomes across reopen,
plus the inspection path's lack of reservation mutation. Keys, identities and
original receipt IDs were not regenerated. The earlier missing-input checkpoint
remains preserved in [key lookup](receipt-key-lookup.md#offline-account-check-and-reproduction).

A scoped directory inventory found **19 profile JSON files**. This is a count
of configured files, **not** a claim of 19 distinct phone numbers, usable server
accounts or registered identities; copies/test profiles and account state were
not classified. No additional number is needed for this offline validation.
