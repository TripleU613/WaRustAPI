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
default 60000; `max_attempts: 0` disables). The attempt count resets only after
an activated connection stayed up for `max_delay_ms` (see
[reconnect budget stability](#reconnect-budget-stability-2026-10-13)). Refusals, `stream:error`/`failure`,
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
viewer for that. Ordinary URL-bearing WebP stickers also use this path and the
image-key domain; Lottie and directPath-only retrieval remain unsupported.
[Sticker schema and evidence](stickers.md). Ordinary MP4 video/video notes
also use this downloader with the distinct video-key domain and audio/video
playback enabled. [Video schema and limits](video-media.md). Voice notes require Ogg/Opus; incoming playback still needs a
live sample. Successful one-use viewing queues `played` for the next connection;
download alone does not. Uncertain receipt writes are not replayed.

The public API uses `.comparison_transport(config)` on `WhatsAppClient::builder()`
with the `comparison-client` feature. Building is offline; `connect` opens the
connection. `.comparison_outbox(true)` additionally enables queued primary direct
texts and uploaded image/voice-note references, including previously queued eligible
intents. Leave it off for a receive-only session. Groups, quotes, captions and
unsupported media fail locally. (Superseded: groups go through the normal group
outbox, and quoted text and quoted image/voice-note replies are sent
([quoted media](quoted-media.md#sending-2026-10-13)); captions and other media
kinds still fail locally.) Queries and sends are reserved
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
  The [37.71 read-job trace](read-receipt-job.md) now confirms its use of
  Cp2 and extra-ID list construction. Rust still sends one ID per receipt;
  full native settings/read-self selection and specialized routing are unported.
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
- `LX/14z.A09` `001a..008a`: first-child `count` selects legacy work
  independently of its optional signed value; sibling PQ is strict/defaulted.
  `LX/14z.A02` `0154..01d4` generates when nothing is unsent and uploads
  legacy keys (`LX/13c.A0U`), then acknowledges (`023a..0244`). PQ-only
  takes no legacy action (`01a0`). Rust keeps its local ACK-before-upload
  order. [Count parsing evidence](prekey-count-notifications.md) records the
  corrected first-child/value distinctions. The separate first `identity`
  branch remains unported; its [worker trace](identity-notifications.md)
  checks a stored identity before scheduling reason-8 bulk prekey fetching.
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
Group and companion-addressed reactions were not implemented then (2026-10-13:
normal group reactions now exist, see [normal group outbox](normal-group-outbox.md);
they still do not render on the handset).

Current reaction-key validation, historical target stability and corrected
status-singleton scope: [2026-10-09 evidence and limits](reactions.md).

### Polls (comparison 2.26.37.71)

2026-10-11 follow-up: [own-device ordinary poll creation](own-device-poll.md)
connects synchronized creations to events. Its current BYa descriptor also has
endTime=9, hideParticipantName=10 and allowAddOption=11, omitted by the historical
eight-field list below. Special poll lifecycles and synchronized vote lookup
remain unfinished.

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

Implementation (`transport::devices`): pending users/revisions are durable;
each query selects at most 64 users. The receive actor continues eligible
batches and clears only the revision settled by that user's verified/reconciled
reply. Refused, omitted, errored and unanswered work remains pending and is
deferred for the connection. See the current [revision/continuation safeguards](#background-device-revisions-and-continuation-2026-10-07)
and [scoped send-preparation guards](#scoped-device-batches-and-completeness-2026-10-07).
`client.device_list(&chat)` / `request_device_sync(&chat)` and `whatsapp-cli
devices --chat <jid> [--request]` are offline. Device lists feed the explicit
group/status send preparation; full public-outbox group fan-out and own-account
inventory remain unfinished. `sid` is client-chosen. The first live query
(2026-09-19) returned one primary and no signed list for a single-device contact.

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

### Runtime parity correction (2026-10-06)

**Confirmed by source inspection:** the earlier implementation below existed
only in `whatsapp-cli::transport`'s lab receive window. The public-client
`transport::connection::drive` classified decryption failures but never called
the retry builder. Both paths now use
`receipts::queue_retry_for_incoming` and the existing durable receipt queue.
This corrects a receive-path difference; it does **not** explain the historical
window where no incoming message stanzas were captured.

**Observed historical evidence:** the retry shape remains the operator handset's
2026-09-24 receipt, recorded below and in `messaging::retry_receipt`. This change
adds no new APK claim or traced retry policy. The existing APK **2.26.37.71**
`LX/Cp2.A05` evidence concerns acknowledgement shape, not the outbound retry
count or limit; those producer branches remain **unknown** here.

Our policy is one request, count 1, per exact sender/chat, participant and
message ID. Timestamp or ciphertext changes on redelivery do not reset it.
The saved Signal registration ID is required; no default ID is invented.
Broadcast routing is unsupported. The capture is stored before processing,
then the request is queued and reserved as `outcome_unknown` before any write.
Write failure, reconnect and process restart do not replay a reservation.
An ordinary receipt ACK matching the queued route, ID and type settles its
outcome through the existing queue; a matching retry ACK settled the live request on 2026-10-06. Acceptance of a retry request is not delivery or
decryption of the replacement message.

`whatsapp-cli receipts` exposes `kind=retry` and its durable outcome offline.
The OpenAPI receipt schema now matches the actual CLI fields (`message_id`,
`kind`, `status`) and status vocabulary instead of the unrelated action states.
Missing registration/routing yields a redacted diagnostic and leaves receive
running; storage failures propagate rather than masquerading as absent input.
The lab receive path shares this policy instead of repeatedly writing count 1.
Tests exercise an actual undecryptable message through the session loop, a
socket write cut, no replay next epoch, continuation without registration data,
queue persistence, sender/participant separation and ACK correlation.

**Observed recovery:** the fresh checkout initially lacked `lab/`. A search
including ignored files found the former checkout under the desktop Trash at
`~/.local/share/Trash/files/TeleWhatsApp/lab`. It was copied back
with private permissions; the original was preserved and all restored files
remain Git-ignored. Offline inspection of profile `l` opened the saved encrypted
store: registered, last login checkpoint `success`, zero eligible outbox
intents, no queued actions (nine `ok`, one `error`, one already `sent`), six
acknowledged explicit receipts, and 49 uploaded prekeys with none unsent or
uncertain. `transport.mark_read` was already true; request data was unchanged.
This establishes recoverable local state, not a fresh successful server login.
The first filename search respected ignore rules and missed this state; use
`rg --files --hidden --no-ignore` when looking for private recovery artifacts.

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
— with `type="read"`. The [37.71 read-job trace](read-receipt-job.md)
now **confirms** that shared builder and native extra-ID list construction.
Rust retains one ID per receipt and does not port full native policy/routing. `receipts::record_ack` used to ignore
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
(authenticated offline recovery; the older unversioned `LX/CPQ.A00`
citation is not established for current 2.26.37.71), and our group fan-outs had established the session
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


## Recovered-account receive window (2026-10-06 local / 2026-10-07 UTC)

**Observed:** the rebuilt release CLI ran one 90-second `listen` window for
profile `l`, with the existing `transport.mark_read=true` and without
`--send-outbox`. Certificate-checked login and connection activation succeeded.
Queued traffic included a direct `pkmsg` and a group `pkmsg`/`skmsg`; neither
decrypted. The direct failure queued one retry, a matching ACK settled it as
`server_acknowledged`, and its redelivery failed too. The saved marker suppressed
a second request. Six prior read receipts remained acknowledged; the only new
receipt outcome was the acknowledged retry. The session stopped cleanly.
Private log and receipt snapshots are under `lab/accounts/l/` with prefix
`receive-2026-10-06-runtime-retry`; no content or identities are reproduced here.

The former `signal::direct::crypto` erased the failure variant into one generic
message. It now maps typed `wacore-libsignal` 0.7.0 errors to fixed, redacted
reasons (missing prekey/session, untrusted identity, bad MAC, invalid encoding,
duplicate counter, etc.). It never formats address/string/backend payloads.
A regression test checks those redactions. This is an implementation diagnostic
correction, not new APK evidence or a relaxation of cryptographic checks.
Identity pins, keys and retry reservations stay intact during offline diagnosis.

`lab-inbox --inspect` now includes `prekey_pin_matches` for the first `pkmsg`
child against stored `comparison-direct` pins for its `from`, `sender_pn`,
`participant` and `participant_pn` aliases. `Some(true)` is a matching public
identity, `Some(false)` a mismatch, and `None` no saved pin; malformed inputs
are labelled `invalid_input` (or `invalid_ciphertext` for a non-byte payload). `signal::direct::incoming_prekey_identity_matches`
only parses and compares public keys; neither inspection nor a match authenticates
the ciphertext or permits replacing a pin. It prints no keys or full addresses
and never advances sessions. This exposes the inputs needed to distinguish a
PN/LID routing issue from an identity difference.

**Confirmed by offline inspection of the preserved captures:** both direct
prekey envelopes differ from both existing `from` (LID) and `sender_pn` pins.
The group envelope differs from both `participant` (LID) and `participant_pn`
pins. All four alias types had saved pins (`Some(false)`, not `None`). Switching
only from the saved LID session to the saved PN session therefore cannot fix
these identity comparisons. Offline replay decrypted nothing new, skipped 33
already committed captures and left three unsupported captures; it sent no
receipts. The keys and pins were preserved.

**Unknown:** whether this is a legitimate peer identity change, incorrect
historical pins or another upstream identity/routing issue. A prekey-envelope
key comparison alone does not authenticate the new key. The existing blanket
TOFU/pin guard has no verified identity-change workflow; that is this client's
limitation, not a server requirement or delivery refusal. Do not replace pins
or create a fresh account merely to make these messages decrypt.

Validation after the diagnostic and inspection changes: 496 workspace/all-feature
tests passed, strict Clippy, rustdoc with warnings denied, formatting, generated
CLI-reference verification and repository hygiene passed. The receipt OpenAPI
schema was corrected to the actual CLI fields and states. Handset ticks and
fresh-message automatic-read behavior remain unconfirmed in this run.


## Authenticated duplicate auto-read correction, 2026-10-07

**Observed live:** after canonical recovery, a 45-second receive-only session
logged in and activated successfully. The server redelivered one direct and
one group message; both were `Duplicate`, delivery-confirmed, and not marked
read. No fresh message arrived. The durable queue still held six acknowledged
reads and one acknowledged retry after the window.

**Confirmed in our code / corrected:** `connection::drive` and lab receive
only invoked the automatic-read helper on `Decrypted`, suppressing an eligible
authenticated duplicate that had been recovered offline. Both now also invoke
it on `Duplicate` when `transport.mark_read` is enabled. The helper still
requires a saved decrypted, readable payload and uses the original authenticated
route. Cleared/consumed records and non-readable protocol content are skipped.
Its durable once-only queue prevents repeats after ACK or uncertain writes.
This does not mark arbitrary historical messages read merely at startup; it
responds to a delivered authenticated duplicate under the explicit read policy.

A real node-loop regression starts from an offline-authenticated text row,
observes delivery confirmation plus one read receipt, cuts the read write,
and confirms that another epoch/redelivery sends no second read. The reserve
remains `outcome_unknown` pending actual server acknowledgement.

For bounded validation, the two recovered Text events were selected from the
journal in memory and queued with the existing `mark-read` command. The private
selector JSON contains only their chat/message IDs, no message content. No
network request occurred while selecting/queuing. The subsequent short session
transmits these receipts through the normal durable actor without sending
queued chats; results are recorded after completion. This is an explicit saved-
message read test, not proof of a fresh incoming automatic-read result.


**Observed live, saved-message read test:** the subsequent 20-second receive-
only session logged in/activated successfully and obtained matching server
ACKs for both selected recovered text reads. Final durable totals are eight
`read/server_acknowledged` and one `retry/server_acknowledged`. No queued chat
send was enabled. This verifies direct/group saved-message read receipt
acceptance; it does not confirm handset blue ticks or a fresh incoming automatic
read. Private selectors and redacted outcomes remain under ignored account
storage. No automatic retry of these acknowledged receipts is permitted.


### Scoped device batches and completeness (2026-10-07)

**Confirmed static protocol substrate, APK 2.26.37.71:** the selected
`0u7.A06` request layout and C8f user/device/error parsing, `11T.A07/A08`
signed-list reconciliation, and SyncDevicesJob query presets remain the
sources cited in the contact-device section above. This correction changes
local selection, accounting and send preparation; it does not establish a new
APK/server limit or claim that the APK applies our completeness policy.

**Observed local bugs:** `transport_group_send::fetch_devices` recorded every
recipient as pending, then called global `begin_query` once. That selected only
64 pending users, including unrelated contacts that sorted before recipients.
Groups larger than the local batch size could remain unfetched. Worse, the
caller printed accepted=false/per-user errors and returned success; existing
cached device lists could then let planning proceed despite the failed refresh.
A matching IQ result with omitted users had the same problem. These were our
plumbing failures, not a WhatsApp requirement or evidence that a server refusal
could be bypassed by cached state.

**Implemented:** `devices::begin_query_for_users(store, context, users)` selects
only the explicit pending batch, normalizes to user addresses and deduplicates
model identities. At most 64 supplied entries is a **local bound**, not a server
maximum. Pending validation and durable query reservation share the write
transaction; missing/invalid/oversized inputs create no query or state changes.
An empty selection returns None. It uses the same usync v2 node builder as global
begin_query: a fresh opaque sid/ID, index=0/last=true, mode=query, and explicit
context. Each batch is an independent read, not an unimplemented multi-index
protocol stream. Background global selection remains available.

Group/status lab send preparation captures and deduplicates its recipient
snapshot, records those recipients, and processes fixed chunks once in message
context. It never loops until the global pending queue empties. The reservation
ID and batch/user count are logged before the write; socket uncertainty or
refusal exits without another batch or message reservation. Matching replies
are committed before their completion guard runs. A later explicit read may
query pending users again under the established read policy; the failed query's
ID/status is never reset and this preparation has no automatic retry.

`IngestSummary` now counts distinct ingested and errored requested users and
reports **omitted_users**: requested identities with no recognized user row.
Duplicate recognized rows cannot inflate success to cover a missing identity;
row reconciliation itself still follows the existing loop. A requested user
represented by devices/error is a partial error rather than an omission.
Unrequested or invalid-owner rows do not settle a requested user. A malformed
requested device row still aborts the transaction, preserving unknown outcome,
old lists and pending markers. A top-level refusal represents no user rows and
records the corresponding omitted count; that does not claim “missing field”
is the server's refusal reason.

`IngestSummary::require_complete` stops send preparation on accepted=false,
any per-user errors, or any omissions, with content-free count/category
messages. It does not promote server_acknowledged to complete: an IQ result
can be saved successfully while selected users remain unresolved. Old cached
lists cannot satisfy this guard. Earlier complete batches and successfully
reconciled users in a partial batch stay saved; errored/omitted users keep
pending markers and prior lists. No message/session reservation or outbound
chat occurs after that preparation failure. Own-account device sync is still
separate and unimplemented; no own identity or companion set is fabricated.
No SMS/voice wait, saved refusal, primary key or chat request ID is reset.

QueryRecord persists nullable omitted_users with serde default. **Historical
and unsettled records retain null/unknown**, even when old aggregate counts
look complete; no identity/store migration or inferred retroactive success.
The existing refusal regression has two pending users, including an earlier
per-user failure; the new count honestly reports two rather than the old
Default summary's absent accounting. This also illustrates why a send's
selected recipients must not silently drain global pending users.

**Implemented offline inspection:**

```sh
whatsapp-cli --profile <saved-profile> device-query --id <existing-query-id>
```

No HTTP handler exists. It reads an existing query without login, queuing,
projection, reservation or reset. Output always includes offline_only/found;
found records include status, requested_users, ingested, partial_errors,
omitted_users (nullable unknown) and a bounded numeric code or null. It omits
addresses, supplied ID, timestamps and raw response text. Unknown IDs return
found=false, invalid IDs refuse without echo. A nonnumeric saved code is not
printed or replaced by an invented recognized value. Existing `devices`
inspection/request behavior remains available. Receive logs now include accepted
and omitted counts instead of describing every settled refusal as “lists updated”.
Background partial-success synchronization keeps its existing behavior.

**Observed offline regression:** 130 synthetic requested users settle as
64/64/2 disjoint batches; one unrelated pending contact remains untouched.
Every query is durably unknown before reply and settles once. Missing selection
and oversized/invalid input leave all checkpoints unchanged. A cached-list
503 refusal blocks preparation and preserves both old lists/pending users;
a duplicate-row result counts one ingested user and one omission, still blocks,
and survives encrypted-store reopen. A later explicit 404 per-user-error read
preserves its prior list and blocks without rewriting the earlier refused record.
Historical omission remains null. CLI summary tests hide identifiers/unknown
codes and preserve the null distinction. These are synthetic injected-node
checks, not live large-group or socket-cut batching validation.

**Unknown / unfinished:** full APK fresh/retry device-source selection, own
account-sync inventory/fan-out, active hash feature, full public-outbox group
sending, large-group live batch behavior and complete incoming ADV-key branch.
The selected APK bundles stay unchanged in this correction; it reuses their
existing attributable protocol substrate, not new upstream source differences.


**Observed live, 2026-10-07 21:30:49 UTC:** preflight found zero eligible outbox
intents, queued actions, pending receipts and device users. One existing peer's
device read was recorded; an authenticated 20-second receive-only session
activated and stopped cleanly with chat sending disabled. Saved query inspection
reported server_acknowledged, requested_users=1, ingested=1, partial_errors=0,
omitted_users=0, code=null. Pending device users returned to zero. The retained
list contains one primary, companions_verified=false, reason “no signed index
list”; this does not establish companion coverage. No new chat intent was
created/transmitted. Action counts stayed error=1/ok=15/sent=1 (including the
unresolved sent action); all nine existing receipts remained acknowledged.
Original identities, keys, send IDs, refusals and waits were preserved.
This validates the shared one-user reservation/parser/accounting path and saved
inspector against a real reply, **not selected multi-batch or group-send parity**.

**Observed diagnostic limitation:** the first validation harness expected the
settlement INFO line in default stderr and falsely failed its log assertion.
The CLI defaults to `warn,whatsapp_api::incoming=info`, hiding connection INFO
counter events. No request was restarted because of this observation failure;
the same session had already stopped normally. Offline canonical-store inspection
found the single new peer query, and device-query established its saved complete
counts; pending/action/receipt state was rechecked. Connection INFO can be
explicitly selected through RUST_LOG; the offline saved inspector is independent
of log filters. Raw private outputs and the query ID remain in ignored lab data.

**Observed verification:** 644 workspace all-feature tests and 24 default CLI
tests passed; strict workspace/all-target Clippy, warning-free Rustdoc, formatting
and CLI reference regeneration passed. Actual device-query CLI probes on an
isolated encrypted synthetic store covered acknowledged-with-omission, refused,
outcome_unknown, historical null, absent ID, invalid ID without value echo, and
unchanged refused status on repeat. The live saved query was also inspected by
the actual CLI. Repository hygiene checked 641 local links and 2405965 bytes across all four
independent APK checksum bundles; cached APK snapshots remain unchanged.


### Background device revisions and continuation (2026-10-07)

**Confirmed substrate, APK 2.26.37.71:** this correction uses the existing
`LX/0u7.A06` usync request and `C8f` response traces, `SyncDevicesJob`
context presets, `LX/11Q.A09` notification parsing and `LX/11T.A07/A08`
signed-index verification/reconciliation. The exact instruction evidence remains
in this document's device-sync section and [cross-device analysis](cross-device-sync.md).
`LX/Cp2.A05` establishes ACK shape, not retry policy. No APK snapshot or wire
field changes are introduced. The 64-user bound and revision bookkeeping below
are **local implementation safeguards**, not inferred server requirements.

**Observed local defects:** the receive actor previously queried only the first
startup batch. An in-flight old reply could delete a newer notification's pending
marker. A failed notification ACK write could terminate the connection before
pending work was recorded. These are our implementation defects; none establishes
a server refusal or ban condition.

Pending work now carries a monotonically increasing local generation. Its
separate durable counter survives marker deletion, preventing deletion/recreation
from reusing an old revision. Overflow refuses without changing state. A plain
refresh preserves an existing pending notification hash; no hint is manufactured.
Reservation atomically snapshots the generations beside the selected users.
A reconciled reply stores its list but clears pending only for the matching
revision. Newer work remains pending and successful superseded users are counted.
`require_complete` refuses that count before group/status send preparation.
These generations are not on-wire fields, keys, account identities or APK values.
Historical pending rows default generation zero; historical query generations and
superseded counts remain null, rather than guessing completion.

The per-connection `DeviceSyncCursor` reserves remaining eligible batches after
settlement or timeout. Each unchanged revision is attempted once. Exact refused,
errored, omitted or unanswered users are deferred for the rest of the connection,
including after a newer notification; their pending work remains saved. Successful
users with newer work remain eligible even if another user failed. A later
connection may perform its normal read with a fresh query ID; it never retransmits
an old reservation. Startup uses interactive context; after a notification trigger
continuation uses notification context. This scheduling is local behavior over the
known presets, not a claim of exact APK job grouping.

Contact notification pending work commits before ACK socket writing. Own-account
notifications retain the existing capture/ACK path; full own-account/LID sync is
still **unknown/unimplemented**. Notification-ID duplicate/conflict semantics and
durable deduplication across restart remain **unknown/untraced**. A repeated
notification can still produce a new local revision; unchanged-revision eligibility
does not prove deduplication of repeated upstream notifications.

**Observed offline regression:** injected-link receive tests cover 130 startup
users across 64/64/2 queries with a refused first batch retained; an ACK write cut
followed by encrypted-store reopen and recovery without rewriting the account;
an unanswered batch followed by another batch and an ignored late old reply; and
a new notification during an in-flight query requiring a distinct follow-up query.
Library tests cover durable counter non-reuse, hash-hint preservation, exact
failed-user deferral alongside eligible successful newer work, and omitted-user
holding despite newer notifications. Historical null and redacted CLI summaries
are tested. `device-query --id` exposes nullable `superseded_users`; private
aligned generations and addresses are not exported. No new command or HTTP route.

**Unknown live boundaries:** large-batch continuation, notification race, ACK
socket-cut recovery and timeout/late-reply cases are offline verified only. The
previous one-user live read predates this correction and is not proof of these
new branches. No canonical account or live request was touched while developing
these regressions. Full own-device inventory, ADV-key verification and public
outbox group fan-out remain unfinished.


**Observed live, 2026-10-07 21:57:48 UTC:** after zero eligible outbox,
queued-action, pending-receipt and pending-device preflight, one existing peer's
device read was queued and a 20-second authenticated receive-only session ended
cleanly. Actual saved-query CLI inspection reported server_acknowledged,
requested_users=1, ingested=1, partial_errors=0, omitted_users=0,
superseded_users=0, code=null. Pending users returned to zero; outbox, all
17 actions (1 error/15 ok/1 sent) and nine acknowledged receipts were unchanged.
One primary remains, companions_verified=false, reason “no signed index list”.
No chat sends, identity rotation, reset or refusal replay occurred. This proves
the new cursor/reservation/settlement path for one live user; it does not prove
live large-batch, notification-race, ACK-cut or timeout behavior.

**Observed verification:** 651 workspace/all-feature tests and 24 default CLI
tests passed. Strict workspace/all-target Clippy passed after removing four
unnecessary clones in test assertions; warning-free Rustdoc, formatting and CLI
reference regeneration passed. Six actual offline CLI probes on an isolated
encrypted synthetic store covered omission, refusal, unknown, historical null,
superseded and absent records; no private generations, addresses or query IDs
were exported. The live saved query was inspected by the actual CLI. Repository
hygiene checked 645 local links and 2405965 bytes across four independent APK
checksum bundles, unchanged by this correction.


Device synchronization now preserves rejected signed candidates and records fixed
local verification reasons, separately from server per-user errors.
[Current retention/inspection behavior](cross-device-sync.md#rejected-device-candidate-retention-2026-10-07).


Device work queued after authenticated primary replacement is picked up on
the receive heartbeat; saved list authority is bound to its actual verifier.
[Current authority/refresh behavior](cross-device-sync.md#signed-device-authority-and-authenticated-refresh-2026-10-07).


### Missing group sender-key recovery (2026-10-07)

**Observed source defect / refuted local assumption:** supported group `skmsg`
with no stored/distributed sender key returned `Unsupported`. The normal actor
and comparison receive loop only queue retries for `Undecryptable`, so that
message silently missed the existing recovery request. A prior regression even
required this outcome. Separately, a stored sender key could make an empty or
structured `skmsg` body fall through to `Decrypted` without plaintext, allowing
an incorrect delivery confirmation. The same fallthrough affected an empty
pairwise-only group child with an old stored sender key.

**Implemented:** supported nonempty `skmsg` ciphertext without a usable key now
returns `Undecryptable`, preserves its capture and reaches the existing durable
once-only retry queue. Empty/structured bodies remain `Unsupported`, regardless
of stored key, and cannot commit group state or earn a delivery confirmation.
An old stored key alone is never sufficient: without authenticated plaintext or
a newly applied SKDM, a pairwise-only group stanza stays unsupported as well.
The shared direct/group ciphertext byte view accepts both Bytes and unchanged
UTF-8 Text. `binary::decoder::read_content` uses read_value(false), so raw binary
bodies remain Bytes even when UTF-8-valid; token/string values can yield Text.
This is not a universal UTF-8 conversion of binary ciphertext. This is an
implementation correction, not an APK-required text encoding or permission to
reinterpret structured content as bytes.

**Evidence/status:** the retry stanza remains the **observed** 2026-09-24 handset
shape with group participant routing recorded in [groups](groups.md#the-fan-out-works-one-device-asks-for-a-retry-2026-09-24).
Existing APK **2.26.37.71 Cp2.A05** supports ACK construction only; the exact
missing-sender-key retry producer and APK count/limit branches remain **unknown**.
One count-1 request per exact chat/participant/message ID is the existing local
policy, unchanged here. Broadcast retry routing stays unsupported; no registration
ID is invented. This change does not broaden retry scheduling after a saved
uncertain/refused request or reset a consumed/send/event marker.

**Offline evidence:** a real sender-key ciphertext first returns Undecryptable,
then decrypts the same original capture after a trusted late test SKDM is installed.
A repeat becomes Duplicate and leaves the ratchet unchanged. The already reserved
retry remains outcome_unknown; no new request is queued or reserved. This does
not test authentication of that imported distribution, which is covered separately
by the pairwise receive path. Empty and structured skmsg bodies with a stored key
stay Unsupported and do not advance it. Byte/Text missing-key bodies yield the
same recovery classification. The actual receive-loop write-cut test now runs
both direct and group cases: capture persists before a retry write, socket failure
retains its reservation, next-epoch redelivery emits neither another retry nor a
delivery ACK, and a later receipt proves receiving continues. Initial new harness
compile failed converting Zeroizing ciphertext directly to Bytes; corrected to
an explicit fixture copy before passing.

**CLI/API:** existing `listen` and comparison receive use the shared result and
queue; `receipts` inspects the same retry outcome. No new command/field/HTTP route
or server status is introduced. `lab-inbox` processing uses the corrected local
classification but does not itself transmit a retry. **Unknown live:** fresh
missing-group-key arrival and recovery; an ordinary receive window alone cannot
prove these branches ran.


Validation: 668 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target Clippy, warning-free Rustdoc and CLI reference generation
passed for the final classification/body guards. APK research artifacts and
wire builders were unchanged; no new claim of APK retry-limit parity is made.


**Observed live regression check:** rebuilt client, 20-second receive-only
window after queue preflight. Session ended cleanly; peer/device/action/outbox/
receipt inspections were unchanged, with no added query or chat intent. Private
evidence index: `lab/releases/group-key-live/active.json`. No fresh missing-key
group message was established, so live recovery/late-SKDM decryption remain
unverified; the window establishes existing receive compatibility only.


**Next source-audit boundary:** `inbox::process_group_inner` currently writes
its `received:` checkpoint only when group_plaintext is present. An SKDM-only
group control can return Decrypted after committing its key without that
plaintext checkpoint. The new late-key fixture covers a subsequent skmsg, not
SKDM-only redelivery. Whether all such controls get durable duplicate handling
elsewhere is **unknown/unverified**; trace/test that path before claiming full
group-control idempotency. This finding is not fixed by the missing-key guard.


### Durable sender-key-only group controls (2026-10-07)

**Observed / reproduced local bug:** a real independently established Signal
session delivers an SKDM-only group control, which commits the new sender key
and pairwise session. The prior implementation wrote no `received:` marker when
there was no user plaintext. After store reopen, redelivery therefore returned
**Undecryptable** (real regression failure), rather than Duplicate, potentially
entering the once-only retry queue for an already accepted distribution. The
preceding missing-key guard did not solve this distinct deduplication gap.

**APK context / confirmed existing schema:** Message field **2** carries the
sender-key distribution; its inner group ID/axolotl fields are **1/2**, recorded
in [groups](groups.md) with the 2.26.37.71 descriptor/call evidence and historical
live decrypts. These are key exchange inputs, not user conversation text.
**Unknown:** exact APK internal SKDM-only storage/duplicate marker policy. The
checkpoint added below is a **local atomicity/idempotency correction**, not an
APK-derived new field or outbound wire requirement.

Newly authenticated SKDM-only group controls now commit a private
`status=sender_key_distributed` marker with their key, pairwise session, applicable
prekey consumption and original capture/route. The marker has **no protobuf**,
publishes no user event, and earns no automatic read receipt. Redelivery returns
Duplicate before decryption and preserves the original marker, key and ratchet,
even if the redelivered ciphertext bytes differ. Existing delivery-confirmation
handling can acknowledge the already stored control; this does not mean a user
message was received or read. Event backfill and read scheduling already skip
statuses other than decrypted, so they do not parse missing control plaintext.

**CLI/API:** `lab-inbox --inspect` recognizes the marker and prints only
`sender-key distribution committed; no user payload or read receipt`. Without
this explicit branch, inspection would fail trying to parse its absent protobuf.
The existing command remains offline and CLI-only. `listen` and comparison receive
share the corrected Duplicate outcome; no new command/flag/HTTP route or retry
reset exists. Aggregate decrypted outcomes still describe processing, not proof
of a human reply; use the event journal for user-message projection.

**Offline regression:** independently generated accounts/signed prekey establish
both sides of a real pairwise session; a subsequent Signal msg distributes a
fresh own group key. The first process succeeds and the marker lacks plaintext;
reopen/redelivery returns Duplicate, stored keys/sessions remain unchanged,
read scheduling returns zero, and no receipt or user journal event is created.
Changed-ciphertext redelivery preserves the original marker/capture/route. The
initial test failed with Undecryptable instead of Duplicate, confirming the gap
before the correction; the corrected regression passes.

**Historical boundary / unknown live:** do not synthesize markers for past
controls based solely on unauthenticated captures or rewind ratchets to replay
them. Prior controls without an authenticated durable marker remain unclassified;
no migration/reset is implemented. Fresh live SKDM-only arrival/redelivery is
not established by these offline tests.


Validation: 669 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target Clippy, warning-free Rustdoc and regenerated CLI reference
passed. API/CLI coverage is included in workspace checks. Existing APK bundles
and all outbound wire builders remain unchanged.


**Observed live receive regression:** rebuilt client, 20-second receive-only
window after protocol queue preflight. Clean exit, no added query/chat intent,
and peer/device/action/outbox/receipt inspections unchanged. Private evidence:
`lab/releases/skdm-control-live/active.json`. No fresh SKDM-only control or its
redelivery was established; this checks existing receive compatibility only.


### Retry reservation identity across JID encodings (2026-10-07)

**Observed / reproduced local bug:** the retry selector serialized the entire
Jid tuple, including `kind` (wire encoding). A regression changed only a saved
sender's Plain/AgentDevice representation after an uncertain reservation and
obtained **1** new retry instead of **0**. The peer, server, agent, device and
message ID were unchanged. Receipt ACK matching also compared encoding kind,
so equivalent representations could leave a valid acknowledgement unmatched.

**Implemented local correction:** logical comparisons retain **user, server,
agent and device**, while ignoring only `JidKind`. Never use `to_non_ad` here:
that would merge distinct devices. New retry selectors retain the predecessor's
JSON tuple spelling with canonical Plain kind; ordinary historical Plain
selectors therefore retain their spelling. Before reserving, the same transaction
probes every predecessor enum-kind selector (at most 16, or 4 without a participant).
Any existing row blocks another reservation, including pending, outcome_unknown,
refused and acknowledged. No row is deleted, rewritten, renamed or assigned a
new outcome; original request IDs, wire and waits remain intact. Outbound retry
encoding still uses the original validated input route, not the lookup-normalized
JID. Enum-kind probing does not establish that all those formats are used by the
APK or broaden accepted server families.

ACK matching now compares the same logical address fields for destination and
any named participant. Existing class/type/message ID, recipient/edit exclusions
and participant-presence policy are unchanged; a different device, agent or
server still cannot settle the receipt. This applies to the shared receipt queue,
not only retries. Previously created duplicate historical rows are **not** merged
or given invented ACK outcomes; retrospective reconciliation remains unknown.

**Evidence boundary:** observed handset retry construction from 2026-09-24 and
APK **2.26.37.71 Cp2.A05** ACK construction remain the wire substrate. The APK's
internal duplicate selector and retry limits remain **unknown**. This corrects
Rust model/guard behavior, not a new APK-derived wire attribute, count or policy.

**Offline tests:** the original regression fails before the fix and passes after.
An independent Python hashlib fixture pins the a1d4c05 historical JSON selector
for a synthetic group AD/0 + LID AD/3 + message ID. Across store reopen and all
four saved statuses, all 16 kind combinations return zero with byte-identical
protocol checkpoint records; device 4 remains a separate request. ACK tests
refuse foreign device/agent/server and accept encoding-only differences. The
actual direct/group receive-loop write-cut regression changes encoding on the
next epoch's redelivery and still emits no second retry or delivery ACK, while
continuing to handle later receipts.

**CLI/API:** `receipts` exposes the original outcomes and selectors; no new
command/flag/HTTP route exists. `listen` and comparison receive share this guard.
No new live retry is requested merely to test it. **Unknown live:** an actual
encoding-altered retry/redelivery or ACK from the server; saved-state inspection
cannot prove that branch occurred.

**Validation:** 671 workspace/all-feature tests, 24 default CLI tests, strict
Clippy, warning-free Rustdoc and regenerated CLI/API checks passed. No live
retry was manufactured, settled request replayed or account identity changed.

### Retry once-only across PN and LID (2026-10-13)

**Found by code audit, not observed live:** the selector above is keyed by the
stanza's raw `from`/`participant`. Read receipts resolve a PN/LID redelivery
through the stored inbox record (`inbox::direct_record_in`, group selection),
but the retry probe only varied encoding kind. Message X delivered as
`from=<PN>` and undecryptable reserved one retry; a redelivery as
`from=<LID> sender_pn=<PN>`, still undecryptable, missed that row and would
queue a second count-1 retry for the same message.

**Correction:** inside the same write transaction, the probe now also covers
the address's other spelling, keeping its device and agent:

- a LID address adds the stanza's `sender_pn` (direct) or `participant_pn`
  (group) and the authenticated `authenticated-lid:<lid>` LID-to-PN mapping;
- a PN address adds the stanza's `sender_lid` / `participant_lid`.

All four attribute names are read by **2.26.37.71** `LidMessageStanzaParser`
(**confirmed**, [metadata-parser.txt](apk/2.26.37.71-primary-lid/metadata-parser.txt)).
Extra probes can only suppress a request, never add one. The new reservation
keeps the canonical raw-address selector; no row is renamed or merged. A PN
redelivery of a LID-first message with no `sender_lid`/`participant_lid` is
still a separate request: there is no PN-to-LID index, and scanning mappings is
not justified for a suppression probe. A saved mapping that is not a phone JID
refuses (fail closed, nothing queued), matching the inbox's treatment.

**Test:** `retry_queue_is_once_across_pn_and_lid_redelivery` covers direct and
group, attribute and mapping aliases in both directions, companion devices, and
controls (another sender, another device, unmapped address) that still queue.
**Unknown:** whether the server redelivers under the other address in practice,
and the APK's own duplicate-retry selector.

## Direct LID sessions across attribute representations (2026-10-07)

**Confirmed local bug, corrected offline:** direct contact routing accepted
both textual and typed `from`/`sender_pn` JIDs, but lid_sender only accepted
AttributeValue::Jid. Consequently a textual LID `from` supplied a valid PN
contact while selecting the PN session instead of an existing LID ratchet.
A real Signal handshake/reply fixture reproduced Undecryptable before the
correction. The shared receipts::jid parser now supplies the LID session
selector for both representations, preserving the existing namespace/device
filter and normalized storage key. No identity policy or PN fallback rule changed.

**APK evidence and its limit:** APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
[DEw.CF0](apk/2.26.37.71-core/group-mode-incoming-parser.txt) resolves
sender/participant PN/LID attributes through typed JID parsing. This source
supports treating addresses as semantic identities; it does not establish
that every server sends both representations or the entire direct-session
producer graph. The ratchet selection itself is grounded in the earlier
**observed** 2026-09-28 LID-session recovery. This change fixes our in-memory
representation mismatch without deriving new cache-mode or alias authority.

The regression establishes a real LID-keyed Signal session, encrypts a reply,
and checks each first-delivery representation in separate synthetic stores.
Both decrypt, keep exactly the LID session with no PN session created, and
return Duplicate for the opposite representation after closing/reopening the
store. Saved session and identity rows remain byte-identical on redelivery.
No ratchet is rewound in a saved account, no marker is deleted, and no failed
or uncertain outbound reservation becomes eligible through this correction.

Direct-LID correction validation: 676 workspace/all-feature tests and 24 default
CLI tests passed, along with strict all-target/all-feature Clippy, warning-free
Rustdoc, CLI reference/API checks and the experimental CLI build. The first
local preflight ran before that build was terminal and encountered an older
CLI without outbox-pending; it stopped before opening an account connection.
A subsequent preflight uses the completed build.

**Observed live follow-up:** the completed experimental CLI passed queue preflight
and authenticated for one 20-second receive-only window, then stopped cleanly.
Outbox, actions, receipts, retained devices and peer pins matched their preflight
inspections afterward. No fresh encrypted user-message stanza was logged, so
live text/typed-LID decryption is not established by this window. Private evidence
index: lab/releases/direct-lid-live/active.json. No queued chat transmission,
new identity query, reservation reset or manufactured recipient message.

## Receipt ACK participant presence (2026-10-09)

[Confirmed APK/local guard correction](receipt-ack-correlation.md): group
read/retry ACKs must carry the same participant presence and address as the
saved receipt. Incomplete success/error ACKs leave outcome_unknown intact,
including after restart, without replay. Cb1 is the 37.71 parser-produced
stanza key; the old CZH comment was not correctly attributed to that key.
Full native callback lifecycle and fresh live validation remain unverified.

### Concrete trusted-contact peers (2026-10-09)

**Confirmed local/static-backed:** [token notification peer guard](token-notification-peer.md)
records the empty-user receive correction against APK 37.71 `11P.A09` typed
UserJid inputs. Server-only PN/LID values do not create tokens or ACKs; invalid
supplied sender_lid reports a fixed diagnostic. No native fallback-policy or
real-account live validation is claimed.

## Reconnect budget stability (2026-10-13)

**Found by code audit:** the runtime reset its consecutive-attempt count as
soon as the server activated a connection. A server that activates and then
drops at once (peer close at a frame boundary is recoverable) therefore gave an
unbounded cycle: every 1.5-2 s a fresh Noise handshake, login, active IQ,
presence, ping and prekey/device work. Nothing reserved was replayed, but
repeated logins are exactly the avoidable client-induced load this project
must not generate.

**Correction (client policy, not an APK value):** the count now resets only
when a connection that reached Ready ends after staying up for at least
`transport.reconnect.max_delay_ms` (default 60 s), or on an explicit `connect`.
An activate-then-drop loop exhausts `max_attempts` with the usual doubling
delays and then stops as `Disconnected`. A flaky network whose connections
last under a minute now ends `listen` after the budget instead of retrying
forever; restart it deliberately. The APK's own reconnect schedule is
**untraced**; this is a conservative local bound.

**Tests:** `recoverable_loss_reconnects_with_fresh_epochs_bounded_attempts_and_kept_reservation`
now shows a stable connection resetting to attempt 1, then three activated but
immediately dropped connections exhausting the budget;
`waiters_span_backoff_while_nudge_connect_and_disconnect_control_the_wait`
shows a brief activation continuing the count (attempt 2).
