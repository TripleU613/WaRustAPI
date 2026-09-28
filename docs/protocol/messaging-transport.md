# Messaging transport

The opt-in `experimental-transport` CLI feature now supports a comparison-build
connection, primary login and one durable direct text send. A live test received
server acknowledgement and operator-confirmed handset receipt. The operator
reported replying; two inbound messages have now been authenticated/decrypted.
Offline classification identified one extended-text reply and one protocol
`EPHEMERAL_SYNC_RESPONSE`, not two user replies.
Protocol delivery/read receipt handling remains unverified. Default
CLI `connect`/`send` and the ordinary outbox actor remain unconnected to this lab path.

## Implemented

- Canonical comparison-APK raw/direct endpoints, with bounded DNS/TCP connection
  establishment and no application retry or fallback loop.
- One `WA 06 03` header per connection, then three-byte big-endian frame lengths.
- Bounded fragmented/coalesced frame reads and complete frame writes.
- Connection epochs, operation deadlines, redacted I/O errors and socket closure
  after EOF, transport errors or canceled I/O. A partially written record is
  never retried on the same socket. Local oversize rejection performs no I/O.
- Certificate-verified classical handshake, primary login from the saved numeric
  registration login and Noise identity, and comparison-token stanza encoding.
- Recipient signed-prekey verification, pinned peer identity, upstream Signal
  encryption, atomic session/send-intent commit before I/O, and matching message
  acknowledgement persistence. No retries of refused or uncertain sends.

Wire choices come from the **2.26.37.70 comparison** mapping in
`transport/comparison.rs`; they do not change the pinned 2.26.36.75 profile's
evidence status. The default 65,535-byte body limit and 15/30-second deadlines
are configurable local limits, not claims about server requirements.
TLS-edge and pseudo-HTTP port-80 endpoints are explicitly rejected until their
wrappers are implemented, never silently downgraded.

Tests exercise exact wire bytes, a real loopback TCP round trip, coalesced
maximum-size frames, malformed/truncated input, stale epochs, cancellation,
deadlines and local validation. Additional tests cover real-signature certificate
checks, withheld login for an untrusted root, encrypted records, primary login
construction and a local Signal encrypt/decrypt round trip with APK padding.

## Explicit lab usage

Build `cargo build --locked -p whatsapp-cli --features experimental-transport`.
Use the same encrypted store, account, private registration JSON and master-key
environment as the completed registration. The optional message file is separate
from frozen registration inputs; it contains `to` (E.164 digits), `text`, and a
stable `request_id` (1–64 ASCII letters/digits/hyphens). Keep it private, mode 600.

```sh
whatsapp-cli --store <saved.redb> --account <account> lab-transport \
  --config <private-registration.json> --allow-live-request --login \
  --message <private-message.json>
```

Without `--login`, this checks only the server certificate. Without `--message`,
login sends no chat. Reusing an acknowledged message's request ID reports the
saved outcome without a connection; changing its recipient/text is rejected.
An uncertain/refused send is preserved and blocked, not re-encrypted or replayed.
The lab checkpoints retain encrypted key replies, pending stanzas, ciphertext,
intent and acknowledgement. Do not remove them to retry.

## Evidence and observed results (2026-09-18)

All APK offsets below are method-relative, for comparison **2.26.37.70**:

- `LX/1N7` constructor/A03/A04: protobuf handshake envelope, `WA 06 03`
  prologue, remote key/certificate verification and encrypted client finish.
  `LX/Aw1.<clinit>` provides the committed `WhatsAppLongTerm1` root.
- `LX/1NZ.A06` at 0x000e–0x001c hashes the first ephemeral only. There is no
  empty payload hash after ClientHello. Standard `snow::HandshakeState` adds
  that hash; its first live attempt failed decryption. `transport/classic.rs`
  reproduces APK sequencing using existing X25519/AES-GCM primitives from snow's
  resolver, SHA-256 and HKDF. No certificate check was relaxed.
- `LX/1LC.AAU` 0x003e–0x0068 supplies numeric identity; `LX/1Ed.AAU`
  0x0032–0x0150 and 0x03fa–0x0478 supply Android/version/locale. `LX/1IC.AAU`
  supplies passive/short-connect/reason. Unknown hardware fields are omitted,
  not fabricated. This minimal 34-byte login received `success` live.
- `LX/0y0.A00` 0x1786–0x193a constructs `iq/get`, `xmlns=encrypt`,
  `key/user/jid`. One live result supplied identity, signed key and one-time key.
- `LX/0jc.A0Q` calls `LX/Cb1.A00` before encryption; the latter appends 1–16
  copies of a random padding-count byte. `LX/C6f.A00` parses `enc`, `v=2`,
  `type=pkmsg/msg`. The comparison token dictionary comes from the committed
  APK bundle, separately packaged as a library fixture.
- The first direct encrypted text received matching `ack`, `class=message`,
  with no `error`. The same request was then replay-tested locally: no network
  connection and no second send. Recipient and keys are intentionally omitted.

Wire message IDs currently use random 128-bit uppercase hex: a locally chosen
opaque identifier whose acceptance was observed, not a recovered APK ID recipe.

## Remaining work

The initial path targets the specified recipient's primary phone JID only.
Companion-device discovery/fanout, general LID routing, robust session
reconciliation and post-quantum (Kyber) prekeys remain incomplete; continuous
inbound processing, delivery/read receipts, legacy prekey replenishment and the
ordinary outbox/runtime integration are implemented and documented below. The classic
branch worked in this experiment; this does not prove hybrid modes unnecessary.

The low-level frame API is for encoded handshake/encrypted records, not plaintext
chat or raw credentials. It serializes operations through one mutable connection;
a long-lived concurrent reader/writer actor remains future integration work.

## Bounded inbound capture and offline decryption

Using the same saved account/store/key environment:

```sh
whatsapp-cli --store <saved.redb> --account <account> lab-transport \
  --config <private-registration.json> --allow-live-request --login --receive
whatsapp-cli --store <saved.redb> --account <account> lab-inbox
```

Receive mode conflicts with sending a message. After login it sends one
`iq/set xmlns=passive` with an `active` child (comparison `LX/Avw.A00`,
called with true by primary ConnectionThread `LX/0iS.A0e`). It correlates the
result while persisting every incoming frame, including frames before the result
and on timeout. It captures at most 64 stanzas within a 30-second window.
Primary direct messages already authenticated and committed by `lab-inbox`
receive delivery receipts on redelivery. Supported incoming delivery/read
receipts receive `ack/class=receipt` (comparison `LX/Cns.A00/A05`). These are
not our read/opened receipts. Raw captures alone never qualify for message
delivery receipts; unsupported routing and notification handling stay untouched.
Each confirmation is reserved before writing, with no automatic retry. A written
confirmation is not proof the server processed it. The client then disconnects.
`lab-inbox` makes no network requests. It decrypts `enc/v=2/type=msg` only with an
existing pinned session; unsupported envelopes remain captured without guessing.
The two live messages arrived from primary LID senders with explicit `sender_pn`
attributes. Comparison `LX/CmF.A01` at 0x0096–00c6 parses this attribute as a
`PhoneUserJid`. We use that mapping only to select an existing pinned PN session;
successful Signal authentication is still required. No global LID map is inferred.

Decrypted protobuf and updated ratchet are committed in one transaction, with a
session comparison guard and sender/message-ID deduplication. Plaintext is not
printed. Live processing decrypted two messages (47 and 73 protobuf bytes), and
a second offline invocation skipped both. Tests cover reply authentication,
tampering, changed identity, duplicate ciphertext, invalid padding, routing and
mutually exclusive CLI modes. Inbox text rendering, new inbound prekey sessions,
general multi-device/LID routing and immediate delivery after decryption remain
future work. The bounded receive path can now confirm previously committed
primary direct messages as described above.

### Saved-payload inspection

`lab-inbox --inspect` inspects already decrypted checkpoints locally without
advancing ratchets or printing text/identifiers. The observed 73-byte payload has
Message fields 6 and 35; nested extended-text fields are 1, 10, 17 and 29. The
47-byte payload has Message field 12 and ProtocolMessage type 4. Comparison
`LX/BXF.dynamicMethod` 0x0578 supplies the Message schema (extended text=6,
protocol=12); `LX/BXE.dynamicMethod` 0x01c4 supplies the protocol schema (type=2).
`LX/C1B.<clinit>` 0x0028–003c maps 4 to `EPHEMERAL_SYNC_RESPONSE`. Inspection
reports unknown fields numerically rather than inventing their meaning. The
protocol payload is not a second user-authored reply.

## Image sending (comparison lab)

### View-once image extension

Set `view_once: true` alongside `image` in the private message JSON. Text and
voice-note combinations are currently rejected. The flag is bound into the
saved intent fingerprint, so toggling it cannot repurpose an existing send ID.
Comparison `LX/BWp.dynamicMethod` maps `viewOnce` to image field 25;
`LX/D7s.AD8` 0x007a–00d8 sets it and constructs the V2 wrapper in its wrapped
branch. `LX/BXF` maps that wrapper to field 55 and `LX/BM5` embeds Message at
field 1. Tests include an independent byte fixture and flag/intent separation.

One live test image with these markers received a matching server acknowledgement.
The operator subsequently confirmed viewing it once and being blocked on a
second opening. The two receive
windows for the operator's reported view-once image contained only previously
captured messages. Redacted inspection recognizes wrappers 37/55/59 and image
flag 25 with a nesting bound, without downloading or opening attachments.
There is now a bounded local viewer (below), but no server opened receipt;
do not treat descriptor decryption or inspection as a successful first/second open.

`lab-download-view-once --config <private-registration.json> --allow-live-request`
uses the same global store/account/key settings. It requires exactly one saved
authenticated view-once image; ordinary images and ambiguous selections are
rejected. It makes one bounded HTTPS GET without redirects/retries, verifies
encrypted length, SHA-256 and truncated HMAC, and keeps only ciphertext in
encrypted `downloaded_unopened` storage. There is no plaintext export or opened
receipt. Repeat calls reuse that state; uncertain/refused/consumed states block.
This is prefetch, not an implementation of viewing or expiry. Live invocation
found no qualifying descriptor and made no GET. After the receive activation and
confirmation correction above, four additional authenticated payloads arrived.
One direct ImageMessage carried flag 25. A single live GET downloaded 91,562
encrypted bytes with valid size/SHA-256/MAC, saved as `downloaded_unopened`.
No attachment plaintext was exported or viewed, and no opened receipt was sent.

`lab-open-view-once` subsequently adds local one-use display of the single
downloaded image. It verifies encrypted and plaintext hashes, MAC, padding and
length, then atomically replaces the downloaded ciphertext and key-bearing
inbox protobuf with consumed markers **before** returning plaintext. The CLI
pipes it to mpv, without a plaintext file, for 30 seconds. mpv configuration,
scripts, default keybindings and playback history are disabled. This is not OS
screenshot prevention. Closing/failing after consumption never restores the
image; database-page secure erasure and cross-device consumption sync are not
claimed. No server opened receipt is currently sent. Offline tests verify the
consumed transition, removal of accessible content and rejection of reopening.
`Message.associatedChildMessage` (91, `LX/BXF`) embeds `LX/BM5.message` (1);
inspection now identifies the formerly unclassified payload as an ordinary-image
child, with no view-once flag.

### Voice-note extension

Private message JSON also accepts `voice_note` with an absolute mono Ogg/Opus
file path instead of `text` or `image`; exactly one is allowed. It uses the same
`--message` command, saved upload reservation and atomic Signal send. The file
hash is media-type-bound, preventing a request ID from changing image to audio.
Duration is derived from the final Ogg granule minus Opus pre-skip, rounded up
to whole seconds. The local parser checks bounded page layout, one stream,
sequence, mono Opus header and EOS; it is not a decoder or CRC validator. Local
limits are 8 MiB and ten minutes. Waveform and streaming sidecar are omitted.

Comparison APK `LX/1qB.<clinit>` 0x000e–0028 establishes `WhatsApp Audio Keys`;
0x01d8–01f2 maps PTT to `ptt` and that key domain. `LX/BWZ.dynamicMethod`
0x00b6 defines AudioMessage: URL=1, MIME=2, SHA256=3, length=4, seconds=5,
PTT=6, media key=7, encrypted SHA256=8, direct path=9, key timestamp=10.
`LX/BXF` selects audio at Message field 8. Upload uses `/mms/ptt/`; the encrypted
stanza uses `type=media` and `enc/mediatype=ptt`. This is comparison-only evidence.

Live test: a generated two-second beep (mono Ogg/Opus, 8,721 bytes) uploaded and
received a matching server message acknowledgement. Repeating the same private
request returned locally without networking. The operator subsequently confirmed
handset receipt; playback was not separately confirmed.

Incoming-image follow-up: inbox processing also includes `message:*:pending:*`
and `message:*:send-response:*` captures, with the same sender/message-ID
deduplication. `--inspect` labels Message field 3 as an image and lists nested
field numbers without exposing keys, URLs or content. After the operator corrected
the destination, a new bounded receive and offline decryption identified one
ImageMessage (Message fields 3/35, 1,227 protobuf bytes). A second new payload has
fields 35/91 (1,296 bytes), not yet classified; it is not counted as another image.
The image descriptor includes URL, MIME, hashes, length, dimensions, media key,
direct path and thumbnail fields. Media downloading remains unimplemented, so
this verifies the authenticated image message, not attachment decryption/display.

`signal::image` implements bounded file encryption using existing crypto
primitives, without uploading or exposing the key in diagnostics. Comparison
`LX/1qB.<clinit>` 0x0028–003e supplies `WhatsApp Image Keys`;
`LX/LG2.AKp` 0x0010–0028 derives 80 bytes; `LX/Kr5.A01` splits them into
16-byte IV, 32-byte AES key and 32-byte MAC key. `LX/Kr5.A02/A03` and
`LX/Jq3.read(ILII)` implement CBC/PKCS padding, MAC over IV plus ciphertext,
and a ten-byte MAC suffix. Tests verify a local roundtrip, MAC coverage,
redaction, fresh per-file keys and local size limits; this is not live media
interoperability evidence. The 8 MiB limit is local, not a server limit.

Upload construction: `LX/0ip.A07` builds `iq/set`, namespace `w:m`, with a
`media_conn` child. `LX/1q4.C8h` reads auth and route lifetimes from its response.
`LX/BWp.dynamicMethod` 0x0186 describes the image protobuf; Message field 3
selects it. `LX/IS4.A01` builds the HTTPS `/mms/image/<encrypted-hash>` URL;
`LX/Gzi.A00` hashes the persisted 32-byte upload seed concatenated with the
encrypted file hash for its token. `LX/AvM.A14/A12` and `StringUtils.A0B` establish
SHA-256 and padded URL-safe Base64 (padding is retained). `LX/I1K.A03/A05`
adds server auth to the URL; `A06` has a single-file MMS4 raw-body branch.
`LX/GzU.A04` sets the User-Agent, identity encoding and length. `LX/9QH`
includes `enc/mediatype=image`; the outer stanza uses `type=media`.

Keep a private JSON with `to`, a stable `request_id`, and `image` containing an
absolute PNG path instead of `text`. Use the same `--message` command as text.
If there is no current route, fetch one first:

```sh
whatsapp-cli --store <saved.redb> --account <account> lab-transport \
  --config <private-registration.json> --allow-live-request --login --media-route
whatsapp-cli --store <saved.redb> --account <account> lab-transport \
  --config <private-registration.json> --allow-live-request --login \
  --message <private-image-message.json>
```

The uploader checks route expiry, uses one authenticated HTTPS host, verifies
TLS, disables redirects/retries/proxies, bounds the response, and stores its
reservation/key/body before POST. Success is reused; refused/ambiguous uploads
are not replayed. Exact source hash and recipient bind the request ID. Signal
ratchet and send intent commit atomically with a session-change guard before
the message write. The local PNG header/dimension checks are not a full decoder.
Only PNG is currently supported; thumbnail generation, other media types,
automatic route refresh and general outbox integration remain future work.

Live result (2026-09-18): a 1024×576 PNG (195,430 original bytes, 195,450 encrypted
bytes) uploaded with HTTP 200 and returned validated media URL/direct path.
One encrypted image received a matching `ack/class=message` without error.
An identical invocation returned the saved acknowledgement without networking.
The operator subsequently confirmed receiving the image on the handset, verifying
this image-send path end to end. The handset confirmation is operator-reported;
receipt fact handling added later is described below.

## Persistent receiving and ordinary incoming media

Build the CLI with `--all-features`. Reuse the encrypted account and private
registration JSON; supply its master key through the existing local environment.
Never put keys, real phone numbers or OTPs in tracked files.

```sh
whatsapp-cli --store <saved.redb> --account <account> listen \
  --config <private-registration.json> --allow-live-request --seconds 75
whatsapp-cli --store <saved.redb> --account <account> media-list
whatsapp-cli --store <saved.redb> --account <account> media-open \
  --config <private-registration.json> --id <opaque-media-id> --allow-live-request
```

`listen` without `--seconds` runs until Ctrl-C or connection failure. It verifies
the certificate, authenticates, activates, sends bounded keepalives, captures
incoming frames, decrypts supported direct messages and commits ratchet/inbox
state before delivery confirmation. After a classified recoverable loss (peer
close at a frame boundary, unanswered keepalive, socket I/O or deadline) the
runtime waits with capped jittered backoff and reconnects with a fresh epoch and
full handshake, bounded by the private JSON `transport.reconnect` settings
(`max_attempts`, default 5; `base_delay_ms`, default 2000; `max_delay_ms`,
default 60000; `max_attempts: 0` disables). Refusals, `stream:error`/`failure`,
revocation, missing registration, certificate failures, worker panics and an
uncertain saved login outcome never reconnect automatically. Reconnecting never
replays a reserved send or receipt: durable reservations survive every epoch,
and a genuine server redelivery is handled as a duplicate. `listen` keeps
running through `Connecting`/`Backoff` and ends only on a terminal state.
It does not drain chat sends unless `--send-outbox` is given. That flag turns
the listener into the normal account runtime: every eligible queued intent from
any idempotency namespace (for example one enqueued with `send`) is transmitted
through the normal outbox, unsupported content fails locally, and receipts flow
back into `status`. Before connecting it prints each intent it may send.
Inspect the same list offline with `outbox-pending` and remove anything
unwanted with `cancel --local-id <id>` first; both never touch the network. Receipt facts and supported trusted-contact tokens
are persisted; unknown notifications remain captured, not falsely applied.

`media-list` is offline and shows opaque selectors, kinds and lengths, not
message content. `media-open` authenticates one ordinary attachment and pipes it
to mpv. Verified ciphertext is cached encrypted at rest. No plaintext file is
created. Ordinary access rejects view-once content; use the explicit one-use
viewer for that. Voice notes require Ogg/Opus; incoming playback still needs a
live sample. Successful one-use viewing queues `played` for the next connection;
download alone does not. Uncertain receipt writes are not replayed.

The public API uses `.comparison_transport(config)` on `WhatsAppClient::builder()`
with the `comparison-client` feature. Building is offline; `connect` opens the
connection. `.comparison_outbox(true)` additionally enables queued primary direct
texts and uploaded image/voice-note references, including previously queued eligible
intents. Leave it off for a receive-only session. Groups, quotes, captions and
unsupported media fail locally. Queries and sends are reserved
durably before network writes, without automatic retries. Delivery/read/played
facts reach normal `message_status` and the durable event journal.
`client.send_receipt(receipt)` with `ReceiptKind::Read` durably queues one
`read` receipt per authenticated incoming direct message (up to sixteen IDs,
all-or-nothing, idempotent); the next active connection sends each once and
settles it by matching server acknowledgement. Unknown, unauthenticated, group
or device-addressed messages are refused before any write, and nothing is sent
without an explicit request (`ReceiptPolicy::ExplicitOnly`). The CLI wrapper is
`mark-read --config <json> --chat <jid> --id <msgid>...` (offline queueing; the
next `listen` session transmits; `receipts` lists outcomes). The first live read receipt was server-acknowledged on 2026-09-19.

Server clock: each authenticated incoming `message`/`receipt` stanza with a
plausible whole-second `t` refreshes one saved (server, local) sample; older
server times never regress it. `client.resolve_chat_mute(&chat, mentions)`
converts a saved mute deadline with the APK's `server + local_now - server_now`
rule against that sample and returns `None` without a mute or a sample. The
APK's offset *source* was not traced; sampling stanza `t` is our documented
local inference, not a claim about the official client. This outbox
adapter now has one live direct-text server acknowledgement through the public
client/runtime; handset receipt remains unconfirmed.

To send only one private-JSON intent, without draining unrelated queued work:

```sh
whatsapp-cli --store <saved.redb> --account <account> outbox-send \
  --config <private-request.json> --message <private-message.json> \
  --allow-live-request
```

This uses `.comparison_outbox_ticket(id)`. Keep the JSON `request_id` stable:
acknowledged repeats return the saved result without connecting, and reserved or
uncertain outcomes refuse replay. A request ID previously used by a lab send is
rejected, not retransmitted through a different path. It accepts direct text,
PNG images (including the existing view-once wrapper), and Ogg/Opus voice notes.
For media it reuses the one-shot upload module and requires an unexpired saved
media route; no hidden route refresh or uncertain upload retry is added. Successful
uploads become stable sealed references bound to this account generation, request
ID and recipient. The normal outbox resolves them only against the matching saved
upload, then uses the existing Signal reservation/ACK path. Public `upload_media`
and `download_media` convenience methods remain unsupported; this integration is
the private-JSON command and already-uploaded reference path. A normal-outbox PNG
test now has its own matching live server ACK. The initial local attempt stopped
at an expired media route before upload; one explicit route refresh enabled the
same saved test intent. Repeating its JSON returned locally with no upload or
connection. A bounded receive-only session subsequently saved recipient delivery
for the image (no read/played). A new two-second synthetic Ogg/Opus voice note also
received a matching ACK through this outbox; repeating its JSON returned locally.
A second 35-second receive-only session saved voice-note delivery (no read/played).
Handset display/playback remains unverified; earlier lab ACKs are not counted as
this adapter's results.

Send-path ownership is now symmetric and durable: the lab command also rejects
normal-outbox IDs, including historical entries from other outbox namespaces.
Claims happen before upload/connect/query work and do not authorize retries.
The saved normal-text test was checked through both commands: the original path
returned its ACK locally; the lab path failed before connection. No retransmission
occurred. Do not change IDs to get around this protection.

The pinned default profile and its capability matrix have not been promoted by
these comparison-only experiments. See the [completion checklist](../messaging-completion.md)
for the exact live checks and remaining cross-device work.

Incoming direct text is now projected into the normal durable event journal in
the same transaction as its ratchet/inbox commit. `lab-inbox` also projects
historical authenticated records from their original captures, without repeating
decryption. Consumed view-once records are excluded. `events --summary` prints
only event kinds; omit `--ack` to inspect without moving the consumer cursor.
Unsupported/media payloads produce opaque references, not key-bearing journal
copies. Extended-text context follows comparison `LX/BWx.dynamicMethod`:
stanza ID field 1, quoted message field 3, expiry field 25. Quotes stay unsupported;
text expiry is preserved and already-expired history is not exposed as fresh text.
Event consumers must honor expiry; automatic journal expiry is not implemented.

### Additional comparison APK evidence

All references below concern **2.26.37.70**, APK SHA-256
`b2090b0207e9b97449a5b48e98eb4d1649ae8e5e738e9b7431d6242717fee732`.

- `LX/EIY.<init>(VLI)` branch 15, `006a..009e`: keepalive `iq/get`,
  `xmlns=w:p`, server JID and ID; there is no `ping` child.
- `SendPlayedReceiptJobV2.A0G`, `0042..006e` and `019a`: policy chooses
  `played` versus `played-self`; `LX/0y0` `0c1e..0c9e` invokes `LX/Cns.A09`.
  Its direct receipt contains to/id/type and optional participant, no timestamp.
  `LX/15A.A07` accepts played status for PTT or view-once. The client queues only
  explicitly requested successful viewing, not background downloads.
- `LX/Cns.A09`, `0000..0044`: client receipt builder with `to`, `id`, `type`
  and optional `participant`/`category`, no timestamp or children. `LX/CcH.<clinit>`
  `0000..0036` lists `read`, `read-self`, `played`, `played-self` as the client
  status-receipt vocabulary; `LX/ClU.A00` `004e..0062` maps `read` to its own
  status code. Our explicit read receipt uses that builder with `type=read`.
  **Build attribution:** the `LX/Cns` and `LX/CcH` class names above hold for
  2.26.37.70 only; both were re-checked against the 2.26.37.71 APK on disk on
  2026-09-22. `LX/Cns` **does not exist** in 37.71 — its outbound `<receipt>`
  builder is `LX/Cp2->A00(LLLLLLLLLIJ)` (element name `"receipt"` at `00fe`,
  confirmed), and the ack-to-an-incoming-receipt counterpart is `LX/Cp2->A05`,
  which echoes `participant`/`recipient` through from the incoming stanza and
  omits `type` when that stanza is `class="receipt"` with `type="delivery"`.
  `LX/CcH` in 37.71 is a media-database cursor mapper (`_id`, `local_path`,
  `encryption_iv` → `LX/CLd`), not a receipt vocabulary class. Re-anchor by
  string constant before reusing any class name in this section against 37.71.

  `LX/Cp2->A00` emits, all **confirmed**: `to` and `id` always; `type` only when
  non-null; `participant` only when non-null **and different from `to`**;
  `recipient` only when non-null; `sts` only when its long argument is positive;
  plus an optional `<meta mode="<int>"/>` child gated on a flag and on the `type`
  value being a member of the `LX/CBs->A00` set. The `participant != to` rule is
  why a direct receipt never carries the attribute and a group receipt does.
  This is an inference from the vocabulary: the read-receipt job itself and
  any multi-ID `list` children were not traced, so one ID per receipt is sent.
- `LX/11P.A09`, `001a..00ee`: notification `tokens/token`, `trusted_contact`,
  from/sender_lid, token bytes and timestamp (token `t` defaults to envelope `t`).
  `LX/0q2.A0P`, `00b2..012a`: update only for a strictly newer timestamp.
  `LX/0yO.A01` processes before acknowledgement; `LX/Cns.A05` serializes ACK
  id/to/class/type. This is narrow trusted-contact handling, not app-state sync.
- `LX/BWZ.dynamicMethod`: AudioMessage field 21 is view-once; audio uses its
  separate `WhatsApp Audio Keys` HKDF domain. Ordinary access rejects this flag.

### Prekey replenishment (comparison 2.26.37.71)

Traced from a second comparison build, **2.26.37.71**, APK SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`, same
signing certificate as 2.26.37.70. Its obfuscated class names differ from the
2.26.37.70 references elsewhere in this document; do not mix them.

- `LX/0ko.A06` `0008..0056`: with fewer than 812 (`0x32c`) unsent keys,
  generate `min(812 - unsent, 50)` (`0x32`) new ones; ids are
  `((next_prekey_id + i) % 0xFFFFFE) + 1` (`00c0..00da`).
- `LX/0kf.A01`: unsent batch query `sent_to_server = 0 AND direct_distribution
  = 0 LIMIT 812` (`LX/0jc.A1D` passes `0x32c`).
- `LX/0y0.A00` case 85 (`12d0..1690`): `iq id/xmlns=encrypt/type=set/to=server`
  with `identity` (raw 32 bytes), `registration` (4-byte big endian,
  `LX/1Lz.A03`), `type` (`[0x05]`), `list` of `key{id,value}` where `id` is a
  3-byte big-endian id (`LX/1Lz.A04`) and `value` the raw public key without
  its type byte (`LX/0km.A02` `0070..0082`), then `skey{id,value,signature}`.
  `pq_list`/`pq_last_resort_key` follow only when Kyber material exists;
  `LX/13c.A03` `00d6..00e4` logs "skipping kyber prekeys upload" otherwise. A
  conditional `op` element (`13f0..1474`, flag `0x5bf2`, coexistence reason and
  token) is not reproduced. This client sends neither.
- `LX/14z.A09` `001a..008a`: `notification type=encrypt` with `count/@value`
  (optional `pq_count`) is acknowledged; `LX/14z.A02` `0154..01d4` generates
  when nothing is unsent and uploads legacy keys (`LX/13c.A0U`); a
  `pq_count`-only notification takes no action (`01a0`).
- `LX/DNq.run` `011a..0160`: at start, existing unsent keys are uploaded and a
  store without any generates. `LX/Bor.A03`: the result's `list/@count_low`
  ("true") schedules another upload (`LX/1Dz.A0c`, `needsMoreLegacyPrekeys`);
  errors enter `LX/13c.A02` backoff.

Implementation (`transport::prekeys`): keys and per-key upload state live in
the encrypted store; a batch is reserved durably before its socket write and
keys in an uncertain batch are never re-sent. `iq type=result` marks the batch
acknowledged and keys uploaded; `count_low` triggers a bounded follow-up (20 per
session); an `iq type=error` records the sanitized code and returns the keys to
unsent without automatic retry. Local deviations, stated: a fresh store
generates and uploads in the same session (the APK does these on separate
runs), and no backoff/retry is implemented. `whatsapp-cli prekeys` prints
redacted counts offline. The first live upload (50 keys) was server-acknowledged on 2026-09-19.

### Reactions (comparison 2.26.37.71)

Protobuf-lite descriptors: `LX/BWI.dynamicMethod` (ReactionMessage: `key`=1
message, `text`=2 string, `groupingKey`=3 string, `senderTimestampMs`=4 int64)
and `LX/BZa.dynamicMethod` (MessageKey: `remoteJid`=1, `fromMe`=2 bool, `id`=3,
`participant`=4). The stanza kind `reaction` is in the message-kind table of
`LX/0zO.A01` (`2bec`) and was observed live: an incoming reaction arrived as
`message type=reaction` with `enc` and `meta` children, and its authenticated
payload carried only `Message.reactionMessage` (field 46). `fromMe` is relative
to the reacting sender, so an incoming `fromMe=false` targets one of our
messages and `fromMe=true` one of theirs; an empty `text` removes a reaction.

Implementation (`transport::reactions`): incoming reactions project into
`SyncEvent::MessageUpdated` with `MessageChange::Reaction { by, emoji }` on the
target key (never a new message); `whatsapp-cli reproject` re-runs projection
over saved authenticated captures so earlier opaque placeholders upgrade.
Outgoing `MessageContent::Reaction` goes through the normal outbox as an
encrypted `reactionMessage` payload in a `message type=reaction` envelope with
a `meta content_type="add_on"` child (observed on the official client's live
reaction; `LX/9QF.<init>(VLI)` carries the `add_on` constant for add-on
message kinds). A first reaction sent without that child was server-accepted
but not shown on the handset; a second with the child but a phone-number
`remoteJid` was also not shown. The official client's own reaction key carried a
`@lid` remoteJid, so when a contact's messages arrive LID-addressed
(`from=<lid>` with `sender_pn`), our reaction key now uses that LID
(`inbox::lid_for`); the receiving client stores reactions as add-ons under a
parent message and drops them when the parent is not found
(`reaction_no_parent`). The LID-keyed 👍 was displayed on the handset on 2026-09-20, confirming the cause;
`whatsapp-cli react --chat <jid> --id <msgid> --emoji <text> [--ours]` enqueues
it (`--remove` clears). Live on 2026-09-19: a contact's 👍 on our test text was
decrypted and projected, and our 👍 on their message was server-accepted.
Group and companion-addressed reactions are not implemented.

### Polls (comparison 2.26.37.71)

Descriptors: `LX/BYa` (PollCreationMessage: encKey=1, name=2, options=3 of
`LX/BSo` optionName=1/optionHash=2, selectableOptionsCount=4, contextInfo=5,
pollContentType=6, pollType=7, correctAnswer=8), `LX/BWH` (PollUpdateMessage:
pollCreationMessageKey=1, vote=2 PollEncValue encPayload=1/encIv=2, metadata=3,
senderTimestampMs=4), `LX/BOq` (PollVoteMessage: selectedOptions=1 repeated
bytes), `LX/BQ2` (messageContextInfo.messageSecret=3). A live poll arrived as
`message type=poll` with `meta polltype=creation`, payload fields 35 and 64
(`pollCreationMessageV3`); `Message.pollUpdateMessage` is 50 (public WA6
numbering, consistent with the observed 46 and 64). `polltype` values in the
APK: creation, vote, add_option, result_snapshot, quiz_creation.

Vote encryption: `LX/0ZG.A01` requires a 32-byte secret and 12-byte IV, builds
the AAD `pollId || 0x00 || voterJid`, and passes secret, ciphertext, IV, poll
id, creator JID, voter JID and the label "Poll Vote" through `LX/C9s.A00` →
`LX/CmM.A02` into the native `JniBridge`. The derivation is therefore not
visible in DEX; the implementation uses the publicly documented recipe with the
same inputs (`HKDF-SHA256(secret, info = pollId || creatorJid || voterJid ||
"Poll Vote")`, AES-256-GCM, option hashes = SHA-256 of option names) and is
confirmed or refuted live. Identities follow the chat's addressing: the
contact's LID and our LID from the login `success` reply (`pn`/`lid`, now saved
as `login-identity`) when known, else phone-number JIDs.

Implementation (`transport::polls`): incoming creations project as
`MessageContent::Poll`; incoming votes project as `MessageChange::PollVote`
when the store holds the poll secret (our own polls keep it in
`poll-secret:<id>`; a sender's own poll carries it in the authenticated
plaintext), otherwise they stay opaque. `whatsapp-cli poll` and `vote` enqueue
creations and votes through the normal outbox in `message type=poll` envelopes
with `meta polltype=creation` for polls and `meta polltype=vote content_type=add_on`
plus `enc decrypt-fail=hide` for votes, which also carry an empty
`PollUpdateMessageMetadata` (all observed on the official client's live vote
stanzas; a vote sent without them was server-accepted but never delivered).
Live on 2026-09-20: the contact's poll
(two options, unlimited selection) decoded; our vote for both options was
server-accepted and our own poll was delivered; the contact's vote on their own
poll and three vote updates on ours decrypted and projected, confirming the
derivation and identity choice on the receive side.

### Quoted replies (comparison 2.26.37.71)

Descriptors: `LX/BVL` (Message: conversation=1, extendedTextMessage=6,
messageContextInfo=35), `LX/BVK` (stanzaId=1, participant=2, quotedMessage=3)
and `LX/BWx` (ContextInfo: stanzaId=1, quotedMessage=3, expiration=25);
`ExtendedTextMessage.contextInfo` is 17 as already used for disappearing-timer
context. A live reply from the handset to one of our messages arrived as an
ordinary `message type=text` whose payload had fields 6 and 35: text plus a
`contextInfo` with `stanzaId` = our message's wire id, `participant` = our LID
(not our phone number) and `quotedMessage` = the referenced message (fields
`[3]`, the quoted image). The chat's disappearing timer (field 25) was carried
as before.

Implementation (`transport::quotes`): incoming `conversation` and
`extendedTextMessage` payloads decode through one parser; a `stanzaId` projects
`MessageContent::Text { quoted: Some(MessageRef) }` whose direction is
`Outgoing` when `participant` is one of our identities (`login-identity`
phone-number or LID) and `Incoming` from the sender otherwise; a quote without a
`stanzaId` stays opaque. `reproject` upgraded the earlier opaque placeholder.
Outgoing `Text { quoted: Some(..) }` is validated (same chat), encoded as
`extendedTextMessage{text, contextInfo{stanzaId, participant, quotedMessage}}`
with `participant` = the contact's LID (or phone number when no LID was seen)
for their messages and our LID for ours, and `quotedMessage` = the referenced
message when the store holds it (the authenticated incoming payload with
`messageContextInfo` stripped so a quoted poll never re-sends its secret, or our
own text intent found by wire id), else an empty `conversation`. It travels in
the plain text envelope. `whatsapp-cli reply --chat --id --text [--ours]`
enqueues one. Live on 2026-09-20: our quoted reply to the handset's reply was
server-accepted, delivered and read.

### Contact device-list synchronization (comparison 2.26.37.71)

Same build and hash as the prekey section. The cross-device class names
(`LX/11Q`, `LX/11T`, `LX/17V`, `LX/0bN`, `LX/101`) match the 2.26.37.70
references in docs/protocol/cross-device-sync.md; `LX/0u7` and `LX/0xX`/`0xY`/
`0xb` are cited from 2.26.37.71.

- Query `LX/0u7.A06` `0e52..0fd4`: `iq xmlns=usync/id/type=get` (no `to`) with a
  `usync` child carrying `sid`, `index`, `last`, `mode` (`LX/0xb`: full, delta,
  query, download), `context` (`LX/0xY`: interactive, background, notification,
  message, …) and optional `allow_mutation`; children `query` (`devices
  version="2"`, `0d8c..0db2`), `list` of `user jid=…`, optional `side_list`.
  `SyncDevicesJob.A0G` picks `LX/0xX` presets; the default is
  `INTERACTIVE_MULTI_PROTOCOL_QUERY` (mode `query`, context `interactive`) and
  notification-driven syncs use `NOTIFICATION_MULTI_PROTOCOL_QUERY`.
- Result `LX/0u7.C8f` `0a54..0bb8`: `usync/list/user[jid]/devices/device-list/
  device[id,key-index,is_hosted]` and `devices/key-index-list[ts,expected_ts]`
  whose bytes are the signed ADV key-index list; `devices/error[code]` marks a
  per-user failure. `LX/0bN.A0H` refuses hosted devices for E2EE.
- Verification `LX/11T.A07`: the contact's pinned primary identity ("local")
  verifies the list; the ADV-provided-key branch ("adv") is not implemented, so
  contacts without a pinned identity keep only their primary device. Companion
  filtering is `LX/11T.A08` (`VerifiedIndexList::reconcile_devices`).
- Notification `LX/11Q.A09`: `notification type=devices` with `add`/`remove`
  (`device[jid,key-index,lid]`, `device_hash`, `device_lid_hash`,
  `key-index-list[ts]`) records the user for a sync (`LX/101.A02`). Own-account
  changes go through `LX/17V.A09`/`LX/1Tc.A01` account sync, not implemented:
  captured and acknowledged only.

Implementation (`transport::devices`): pending users are recorded durably; a
session queries up to 64 of them interactively at start and immediately after a
notification; replies are verified, reconciled and stored per user in one
transaction, clearing that user's pending marker. A raw-ID change or stale
timestamp keeps the saved list; refusals record the code; unanswered queries
leave users pending for a later session (queries are reads and safe to repeat).
`client.device_list(&chat)` / `request_device_sync(&chat)` and `whatsapp-cli
devices --chat <jid> [--request]` are offline. Device lists are not yet used
for send fan-out. `sid` is client-chosen. The first live query (2026-09-19) returned one primary device and no signed list for a single-device contact.

### Public ordinary-media download

With the `comparison-client` feature and explicit `comparison_transport(config)`
builder configuration, `client.received_media().await?` returns sealed references
for authenticated ordinary images and voice notes already in the encrypted inbox.
Listing is offline. Pass a selected reference to
`client.download_media(&reference, destination).await?` for a bounded, verified
download and **explicit plaintext export**. This does not require opening the
messaging socket. It reuses verified cached ciphertext and does not retry unknown
GET outcomes. No read or played receipt is sent by downloading.

The destination must be a new file; existing files and symlinks are refused.
On Unix, creation requests mode 0600. After an export I/O failure the owned file
descriptor is truncated on a best-effort basis; a private file can remain and
should be inspected rather than overwritten by a retry. Export is an explicit
opt-in alternative to the existing CLI's memory-only player, not its new default.
References are account-generation-bound; modified metadata, outgoing references
and view-once content are rejected by the download API.

### Public ordinary-media upload

`client.upload_media(source).await?` now accepts bounded PNG images or Ogg/Opus
voice notes for an explicitly configured, registered comparison client. It reuses
the existing TLS-verified one-shot uploader and requires a saved unexpired media
route. Route refresh remains explicit. Uploading does not enqueue a chat or
connect the messaging socket.

The returned account-bound `MediaReference` can be placed in
`MessageContent::Media { reference, caption: None, file_name: None }` and passed
to `send_message` with the caller's stable message request ID. Actual transmission
still requires the separate normal-outbox opt-in. Public upload references are
recipient-independent; message intent/idempotency and Signal encryption remain
recipient-bound at send time. Incoming/download references cannot be sent.

Public upload checkpoints use content plus image/PTT kind, in a namespace separate
from private-JSON lab uploads. Repeating the same encoded file returns the same
sealed reference across restart; moving the file does not create another upload.
Uncertain/refused outcomes are retained and never retried automatically. Editing
the bytes creates different content, not a reset of the old checkpoint. References
bind the exact uploaded protobuf and reject changed metadata or saved payloads.
This API does not expose view-once uploads; the existing explicit JSON view-once
path remains separate. Offline tests verify confirmed reuse, restart, uncertainty
and entry into the normal outbox; a new public-API network upload is not yet a
verified live result.


## Asking for a resend (2026-09-24)

When we cannot decrypt an incoming message we now send a `receipt type="retry"`,
which is the mirror of the ones we answer. Not sending them is why several of
the operator's messages simply vanished: the stanza was preserved, nothing was
acknowledged, and the sender was never told to try again.

`IncomingResult` gained `Undecryptable`, separate from `Unsupported`, because
the two want opposite answers — content this client does not render has arrived
and should be acknowledged, while a message we could not open should make us ask
for it again. Conflating them silently drops the second kind.

The receipt's shape is mirrored from the handset's own
(`<receipt to id [participant] type="retry"><retry count id t v/><registration/>`).
The `<registration>` child carries our registration id, which is what lets the
sender choose between reusing their session and fetching a fresh bundle.

Offline replay counts an undecryptable capture rather than answering it: a
replay sends nothing.

## One bad message must not end the session (2026-09-24)

`inbox::process` propagated a decryption failure straight out, so a single
undecryptable `pkmsg` tore down the whole connection and every stanza queued
behind it went unprocessed. From the outside this looked like the server hanging
up on us — `listen` reported "connection ended; reconnect bounds exhausted",
repeatedly, and it was our own error doing it.

The group path had degraded like this since the first live group stanza. The
direct path now does too, but **only for the Signal layer's own failures**,
matched by `field == "direct_message"`. A transactional failure *after* a
successful decryption also surfaces as `InvalidInput`, and that one keeps
propagating: it means the advanced session and the inbox write rolled back
together, which the caller must see rather than have reported as an ordinary
undecryptable message. An existing rollback test caught exactly that distinction
when the first, broader version of this change swallowed it.

**Live 2026-09-25:** a `--receive` window on profile `l` hit one undecryptable
`msg` and wrote the retry receipt. Nothing in that window shows whether the
sender resent it.

## Read receipts for group messages (2026-09-25)

`mark-read --chat <group>@g.us --id <id>` now queues a read receipt for a group
message. It is the group delivery receipt — `<receipt to=<group> id=<id>
participant=<sender>/>`, confirmed from `LX/Cp2` and live since group receive
— with `type="read"`. That the read receipt uses the same builder is
**inferred**: the read-receipt job was not traced, which is also why direct
read receipts carry no `list` children. `receipts::record_ack` used to ignore
any receipt ack carrying `participant`; it now settles a receipt only when the
ack names the same participant the receipt did, since in a group an id alone
does not say which member's message was read.

**Live 2026-09-25 (profile `l`):** one group read receipt for the operator's
latest Test-group message, queued with `mark-read` and sent by `listen`, was
settled `server_acknowledged` by a matching ack. Whether the operator's
"Read by" list shows it is the handset check.

### Automatic read receipts (`transport.mark_read`)

With `"mark_read": true` in the private JSON's `transport` section, `listen`
queues and sends a read receipt for each incoming message it decrypts whose
payload carries readable content — `Message` field 1, 3, 6, 7, 8, 9, 26 or 66.
Payloads that are only a sender-key distribution (2), a protocol message (12) or
a reaction (46) get none. It uses the same durable, once-only receipt queue as
`mark-read`, so a redelivered message is not marked twice. A refusal is logged
and the session continues. Off by default: the official client decides "read"
from a chat being on screen, which a headless client cannot infer, so this is
the operator saying so. `lab-transport --receive` applies it too, and now
settles receipt acks there as `listen` does.

### Seeing what a session received

Without `RUST_LOG`, the CLI now logs warnings plus one `incoming` line per
stanza from `listen`: element, `type`, direct/group/broadcast/server, the `enc`
types, the inbox outcome, and whether a delivery confirmation and a read
receipt went back — no identities, ids or content. Keepalive replies are
omitted. The previous empty default printed nothing at all, so on 2026-09-28 a
three-minute `listen` window in which the operator's two messages never arrived
looked identical to one that dropped them; only a store inspection afterwards
showed that nothing but keepalive replies and two notifications had come in.

## LID senders decrypt with the LID session (2026-09-28)

**Observed:** one direct `msg` from the operator's LID (`from=<lid>`, with
`sender_pn`) failed with "Signal key/session operation failed" on its first
delivery and on two resends that answered our retry receipt (`enc count=1`),
across four days. The inbox mapped the LID to the phone JID and decrypted with
`comparison-direct:<phone>`. But a LID-addressed chat encrypts under the LID
(`LX/CPQ->A00`, confirmed), and our group fan-outs had established the session
with that person's LID device under `comparison-direct:<lid>`, so the phone
reply rode a ratchet we were not looking at.

`inbox::direct_session_peer` now picks the LID session when the stanza is
LID-addressed and we hold one (or when a `pkmsg` starts one), falling back to
the phone session otherwise. The inbox record stays keyed by phone JID, so
receipts and events are unchanged. Replaying the preserved captures offline
decrypted the message — an extended text, fields `[6, 35]` — and left zero
undecryptable captures on profile `l`. Resending retries alone could not have
fixed it: the sender correctly reused the session it held.

## Receipt acknowledgements mirror the receipt (2026-09-28)

**Confirmed** from 2.26.37.71 `LX/Cp2->A05` (`WriterThread/write/send-message-ack`):
the ack for an incoming receipt is built from the receipt itself — `id`; `to` =
its `from`, as given (a group, a device); `class="receipt"`; `type` unless the
receipt's is absent or `delivery`; `participant` and `recipient` when present;
`edit` when present and not `"0"`.

`received_confirmation` refused any receipt with `participant` or `recipient`,
or from a group or a companion device, so no group receipt was ever acked and
the server redelivered them at every login — visible in the new `incoming` log
as a burst of `receipt chat=group confirmed=false`. Receipts are now acked in
the APK's shape. Only the types this client processes (delivery, read, played)
are acknowledged; the APK acks every type after handling it, and an ack for a
type we only preserve would claim it was handled.
