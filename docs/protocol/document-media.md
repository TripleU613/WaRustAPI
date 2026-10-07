# Ordinary incoming documents

**Confirmed/static**, APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
BZc Message descriptor establishes documentMessage=7. Exact DEX field IDs
classes.dex[24818], classes7.dex[15637], classes8.dex[1810] establish its
nested type LX/BZB; these references prove type, not wire number.

BZB.dynamicMethod's object array at 0036..011e and descriptor at 011e..012c,
with 6al.A1W (0000..000c) bookkeeping and Ayp.A1K (0000..0012) title/hash
slots, establish these fields:

| Tag | Field | Type |
| --- | --- | --- |
| 1 | url | string |
| 2 | mimetype | string |
| 3 | title | string |
| 4 | fileSha256 | bytes |
| 5 | fileLength | uint64 |
| 6 | pageCount | uint32 |
| 7 | mediaKey | bytes |
| 8 | fileName | string |
| 9 | fileEncSha256 | bytes |
| 10 | directPath | string |
| 11 | mediaKeyTimestamp | int64 |
| 12 | contactVcard | bool |
| 13 | thumbnailDirectPath | string |
| 14 | thumbnailSha256 | bytes |
| 15 | thumbnailEncSha256 | bytes |
| 16 | jpegThumbnail | bytes |
| 17 | contextInfo | message |
| 18 | thumbnailHeight | uint32 |
| 19 | thumbnailWidth | uint32 |
| 20 | caption | string |
| 21 | accessibilityLabel | string |

There is no viewOnce field in this descriptor. The client's supported view-once
wrappers still exclude the entire payload from ordinary retrieval. That absence
is not evidence that arbitrary unseen wrappers are replayable. Complete context,
thumbnail and contactVcard handling remain unknown/unimplemented here.

**Confirmed key domain:** 1qB.<clinit> loads `WhatsApp Document Keys` into v4
at 0080..0096 and passes v4 to the document/DOC constructors at 03c2..03e6.
The existing CBC/HKDF/HMAC media primitives now expose `decrypt_document` with
that distinct domain. Image, audio and video labels do not authenticate the
independent document fixture. [Scoped static evidence](document-media-evidence.txt)
retains the descriptor/helpers/key anchors.

**Negative tooling result:** apk-proto-fields --class LX/BZB; refused with
“decoded 21 fields against 20 names; not guessing”. Our heuristic does not
expand Ayp.A1K. The missing name is fileSha256_, confirmed by that helper;
the APK schema is not missing a field. Bulk raw DEX and original scoped dumps
remain in ignored lab/apk-2.26.37.71, with no media inventory committed.

**Implemented/offline:** ordinary URL-bearing documents join media-list,
authenticated retrieval, sealed account-bound references and durable receive
projection as MediaKind::Document. Caption=20 and fileName=8 are retained as
message metadata; title is not silently substituted for either. Sender filenames
are never used as filesystem paths. Quoted projection remains unsupported;
existing context expiration handling applies. The same inbox/event transaction
and once-only markers are used, with no re-decryption or marker deletion.

`media-save --config <private-request.json> --id <opaque-selector>
--output <new-private-file> --allow-live-request` explicitly exports any
supported ordinary attachment. No HTTP route exists. Missing live opt-in refuses
before config/store access. Retrieval checks the registered saved account,
approved HTTPS media host, bounded descriptor, encrypted hash, MAC, CBC/padding,
plaintext hash and length. One-shot download reservations and encrypted ciphertext
cache preserve uncertain/refused outcomes across restart; no automatic retry
is added. Availability is rechecked before export. Existing output/symlink
refuses before retrieval and create_new rechecks atomically after retrieval.
Output is mode 0600. I/O failure truncates the owned descriptor; an empty private
file can remain. Stdout reports only byte count, never content/path/labels/keys.

Document MIME is export metadata, not a viewer/executable selector. The local
MIME token/256-byte and existing 8 MiB bounds are implementation policies, not
APK/server limits. No MIME is invented. media-open refuses documents before
retrieval and directs the operator to media-save --output; the library retrieval
API returns authenticated bytes without executing/opening them. DirectPath-only,
large documents, thumbnails, sending and document presentation remain unfinished.

**Independent offline validation:** tests/fixtures/document-media.json was
produced with Python stdlib HMAC/SHA256, explicit RFC5869 expansion and OpenSSL
AES-256-CBC. Key is 32 synthetic bytes of 0x0c; plaintext is synthetic text.
Extract uses zero 32-byte salt; expand uses WhatsApp Document Keys; bytes 0–15
are IV, 16–47 AES key, 48–79 MAC key. CBC uses PKCS padding; append first ten
HMAC-SHA256 bytes over IV+ciphertext and hash the complete body separately.
Literal wire-tag tests exercise schema/domain, per-byte tampering, MIME syntax,
view-once wrapper exclusion and mixed-media refusal. A real encrypted-store
fixture commits one event, refuses duplicate publication, reopens, retrieves
cached ciphertext without a connection, validates the sealed reference, preserves
caption/filename and saves only to the operator path. Existing-file/symlink
refusals preserve output bytes and mode 0600. Quote projection is excluded.

**Live unknown:** no fresh document message/download has been observed. Offline
fixture success does not prove handset display, real media interoperability or
full messaging completion. No saved real account or live request was changed
by these tests. Current peer inventory/feature selection and companion/account
synchronization gaps remain recorded separately.

Validation: 567 workspace/all-feature tests and 252 default-feature workspace
tests passed, plus strict all-target/all-feature Clippy, warning-free Rustdoc,
generated CLI reference, formatting and repository hygiene (370 local links).
The first Rustdoc run rejected an unescaped <clinit> in the new API comment;
after formatting that name as code, its rerun passed. No runtime change was
needed. Focused document/schema/crypto/restart/export tests passed.

**Observed/live receive check (2026-10-07):** the rebuilt all-feature CLI
completed a single 45-second receive-only window using the preserved profile l
and its existing request-authenticated-receive.json (incoming apk_authenticated,
outgoing pinned by default), without --send-outbox. It authenticated/activated
and stopped cleanly with exit zero. Before/after redacted summaries were
byte-identical: 20 journal events, nine acknowledged receipts, zero eligible
outbox intents and twelve historical actions with no new queued action; the
existing Sent/uncertain action remained preserved. Traffic comprised ib,
notifications encrypt/mex/psa/w:gp2 and a group retry receipt. Those generic
incoming log labels are not proof that every notification was fully handled;
no new event or encrypted message arrived. No document entry appeared, so no
live media download/export or document interoperability was validated.

An encrypted database backup, private logs and before/after summaries are kept
under ignored mode-0700 lab/accounts/l/document-receive-check-2026-10-07
(files mode 0600, backup before.redb). Store opening performs normal metadata
initialization, so unchanged summaries are not a byte-identical database claim.
No prior send ID was changed or acknowledged send repeated. The built CLI also
refused a synthetic media-save invocation without live opt-in before accessing
config, store or output. Current notification/account-sync and identity-input
integration work remains open; the goal is not complete.
