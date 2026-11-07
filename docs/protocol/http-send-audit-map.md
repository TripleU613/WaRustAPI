# Concrete HTTP send audit map

**Confirmed Rust source inventory**, baseline `86cde074`, reviewed offline.
This is a map of concrete reqwest send sites in whatsapp-api, not a completeness
percentage or proof that the APK's network behavior has been ported. Source
symbols are stable search anchors; pinned APK evidence remains in the linked
area documents. No live request is made by this inventory.

| Send site | Construction and response boundary | Evidence and remaining scope |
| --- | --- | --- |
| registration/wire.rs, ExperimentalHttpsTransport::send | POST to the comparison registration origin; encoded body/type/length; configured byte cap; distinguishes connector/builder failures from potentially written requests | [Registration safety](registration-safety-audit.md). Endpoint-specific construction and saved-attempt policy remain separate from this generic HTTP adapter. |
| transport/ban_appeal.rs, submit_ban_appeal | Fixed ACS GraphQL POST; optional saved login Authorization; bounded structural-only summary | [Appeal evidence/unknowns](ban-appeal.md). Operation/doc_id validity, historical token/account provenance and full outcome handling remain unverified. No durable submission reservation is established here. |
| transport/image_upload.rs, upload_prepared | Route-derived media POST; durable upload reservation; response header/body observation before result promotion; fixed 64 KiB JSON ceiling | [Pinned upload HTTP evidence](rea/2.26.43.72-media-upload-http/README.md), [response evidence](rea/2.26.43.72-media-upload-response/README.md). Selected static traces do not prove complete server parity. Smaller configured response budgets were omitted and are corrected below. |
| transport/media.rs, open | Ordinary media GET; owner/source/reservation checks; exact encrypted-length bound; authenticated decryption before publication | [Replay and persistence audit](replay-safety-audit.md). Exact descriptor length is a media-content bound, distinct from a small JSON response budget; no policy to apply that JSON budget to media bodies is inferred here. |
| transport/image_download.rs, download_view_once | View-once GET; owner/source reservation and settlement; encrypted-length bound | [Replay and persistence audit](replay-safety-audit.md). Download completion is distinct from viewer consumption or opened receipts. |

Inventory method: search all whatsapp-api Rust sources for reqwest usage and
`.send()` calls, inspect each builder/response consumer, and exclude channel sends.
The five concrete HTTP calls above are the source inventory at this baseline.
This does not enumerate every stanza carried over the messaging connection.
`transport/socket.rs::connect_comparison` separately opens canonical raw/direct
TCP transport; it rejects unimplemented TLS/pseudo-HTTP wrappers. Noise/login,
queued actions, receipts, ACKs and notifications need their own protocol audits.

## Upload response budget correction

**Confirmed local configuration omission, corrected offline.** The upload response
loop previously used only 65536 bytes even when TransportSettings requested less.
It now checks every streamed chunk against min(max_response_bytes, 65536) before
appending, with saturating size arithmetic. An over-budget chunk leaves the
accumulator unchanged and returns the existing redacted uncertain-outcome error.
The existing response observation/reservation stages are not reset or retried.

The offline accumulator regression covers multiple chunks exactly filling a
three-byte configured budget, a one-byte overflow with unchanged accumulated
bytes, and the hard ceiling even with a larger supplied limit. This test does
not simulate a server or establish the entire upload settlement lifecycle.
Existing upload lifecycle tests remain separate evidence. No request body,
header, identity, retry or server requirement is changed by this local limit.

## Selected inbound cross-check

`receipts::validate` and `record` share aggregate expansion and direct/group target
parsers. `record_in` rechecks unsupported self-user routes, and recipient-selected
aggregates are validated without ordinary fact projection. This bounded source
review found no new confirmed routing mismatch; it is not an exhaustive inbound
coverage claim. [Receipt grammar/evidence](offline-aggregated-receipts.md) and
[direct receipt evidence](direct-receipts.md) retain their explicit unknowns.

## Upload budget failure survives restart

**Offline integration verification**, baseline `f5831e02`; no runtime change.
The new regression stages an owner-bound upload reservation for a synthetic PNG,
retains the HTTP header observation/status using the actual response-stage helpers,
then runs the actual bounded accumulator through exact-budget acceptance and a
one-byte overflow. Both HTTP200 and HTTP503 header cases are exercised.

After closing/reopening the encrypted store, the exact header-stage row and its
independent observation remain present. Calling both prepare and prepare_outbox
with the same message returns the existing uncertain/refused no-replay error,
with checkpoint rows and journal unchanged. These entry calls use valid local
configuration/account inputs, so the assertion establishes saved-outcome refusal
rather than an earlier preflight failure. They exit before client/route/new POST
construction. This closes the previous accumulator-only test boundary for restart
and the two message-bound entry paths. It does not emulate response streaming over
a socket, prove server acceptance or cover the separate public content-upload
producer. Final upload promotion's account/exact-row checks remain separate tests.
