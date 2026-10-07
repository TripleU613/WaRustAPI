# Evidence ledger

### Polls — 2026-09-20

`LX/BYa`, `LX/BWH`, `LX/BOq`, `LX/BQ2`, `LX/BSo` descriptors give the poll
protobufs; a live poll used top-level fields 35 and 64 with `meta
polltype=creation`. `LX/0ZG.A01` shows the vote AAD (`pollId || 0 || voterJid`)
and the inputs handed to native code for key derivation; the derivation itself
is not in DEX. Our vote and our poll were server-accepted; the poll was
delivered; incoming votes decrypted with the documented recipe.

### Quoted replies — 2026-09-20

Comparison 2.26.37.71 descriptors `LX/BVL` (Message: conversation=1,
extendedTextMessage=6, messageContextInfo=35), `LX/BVK` (stanzaId=1,
participant=2, quotedMessage=3) and `LX/BWx` (ContextInfo expiration=25). A live
handset reply to our image arrived as `type=text`, payload fields 6+35, with
`contextInfo.participant` = our LID and `quotedMessage` fields `[3]`. Our quoted
reply (same shape, participant = the contact's LID, quoted = the stored
authenticated payload) was server-accepted, delivered and read.

### Reactions — 2026-09-19

`LX/BWI`/`LX/BZa` descriptors give ReactionMessage (key=1, text=2,
groupingKey=3, senderTimestampMs=4) and MessageKey (remoteJid=1, fromMe=2,
id=3, participant=4); `LX/0zO.A01` lists the `reaction` stanza kind. A live
incoming reaction (`message type=reaction`, payload field 46) decrypted and
projected as a message update; a reaction sent back was server-accepted.

### Live confirmation of 2.26.37.71-derived requests — 2026-09-19

On the saved comparison account: the `iq/encrypt/set` prekey upload built from
2.26.37.71 evidence was accepted (`iq type=result`, 50 keys, no `count_low`); the
`usync devices version=2` query was answered with a `device-list` of one primary
device and no `key-index-list`, consistent with a single-device contact; an
explicit `receipt type=read` received a matching `ack class=receipt type=read`.
These are server-reported outcomes, not handset observations.

### Comparison 2.26.37.71 contact device-list sync — 2026-09-18

Same build. `LX/0u7.A06` builds the `usync` query (`sid`, `index`, `last`,
`mode`, `context`, `query/devices version=2`, `list/user`), `LX/0u7.C8f` parses
`device-list/device[id,key-index,is_hosted]` and `key-index-list[ts,
expected_ts]`, `LX/11T.A07` verifies the signed list with the pinned primary
identity (or an ADV-provided key, not implemented), `LX/11Q.A09` parses
`notification type=devices` and `LX/101.A02` records the user for a sync.
Implemented for contacts with tests; own-account sync and send fan-out remain
open. No live query has run.

### Comparison 2.26.37.71 prekey replenishment — 2026-09-18

A second comparison build, 2.26.37.71 (APK SHA-256 `1755111a…7d5c`, same
signing certificate as 2.26.37.70, different obfuscated class names), provides
the one-time prekey path: generation of `min(812 - unsent, 50)` keys with ids
`((next + i) % 0xFFFFFE) + 1` (`LX/0ko.A06`), the 812-key unsent batch
(`LX/0kf.A01`), the `iq/encrypt/set` upload layout with 3-byte key ids and a
4-byte registration id (`LX/0y0.A00` case 85, `LX/1Lz.A03/A04`), the
`notification type=encrypt count/@value` trigger and its acknowledgement
(`LX/14z.A09/A02`), startup upload of unsent keys (`LX/DNq.run`) and the
`count_low` follow-up in the result (`LX/Bor.A03`). Kyber (`pq_*`) children are
optional in the APK and are not sent. Offline tests reproduce the layout; no
live upload has run. Details: docs/protocol/messaging-transport.md.

### Comparison receive activation and view-once prefetch — 2026-09-18

Derived from 2.26.37.70: `LX/Avw.A00` 0x0008–0068 builds
`iq/id/xmlns=passive/type=set/to=server` with `active` for true;
primary ConnectionThread `LX/0iS.A0e` 0x0014–004a generates the ID and
calls that true branch. `LX/Cns.A00` 0x007c–0108 constructs delivery
receipts with optional type/participant/recipient; `A05` 0x0000–0074
constructs receipt acknowledgements, omitting type for delivery. Our bounded
implementation supports primary direct routes only; read/opened receipts are
emitted only on explicit request (viewer completion or `send_receipt`), never
automatically. Messages must already be authenticated and committed before delivery
confirmation; all incoming frames are encrypted at rest before acknowledgement.

Observed on our comparison lab client: activation result accepted. Activation
alone yielded the five old messages. Adding delivery/receipt confirmations was
followed by four additional authenticated payloads; one had direct ImageMessage
flag 25. A single HTTPS prefetch downloaded 91,562 ciphertext bytes and passed
size/hash/MAC verification. Encrypted state is `downloaded_unopened`, with no
plaintext export or opened receipt. Queue progress followed the confirmations;
the exact internal server queue policy remains unknown. This is not a capture
from an official Android device or evidence for the pinned build.

### Comparison image upload/send — 2026-09-18

Comparison 2.26.37.70: media route query returned nine hosts and auth. The first
route expired before use and was explicitly refreshed; no upload used expired
auth. One encrypted PNG POST returned HTTP 200 with URL/direct_path, followed
by authenticated login and a matching encrypted-image message acknowledgement.
Reusing the private request returned locally without uploading or sending again.
The operator subsequently confirmed receiving the image. APK mappings, limits and commands are in
[messaging transport](messaging-transport.md#image-sending-comparison-lab).
No route credentials, media keys, phone numbers or returned private URLs are
included here. This observation is comparison-only, not the pinned APK profile.

### Comparison inbound messages — 2026-09-18

One bounded authenticated connection captured two `enc/v=2/type=msg` messages.
Both used a primary LID `from` and explicit `sender_pn`; comparison APK
`LX/CmF.A01` 0x0096–00c6 reads `sender_pn` as `PhoneUserJid`. Selecting the saved
pinned PN session allowed authenticated offline Signal decryption of both.
Plaintext/session were committed atomically and a repeated offline run skipped
both. This verifies this mapped established-session case, not general LID or
prekey receiving. No message content, sender values or credentials are recorded.

Subsequent offline protobuf inspection established one extended-text payload
(Message fields 6/35) and one protocol payload (field 12, type 4), not two user
replies. Comparison `LX/BXF.dynamicMethod` 0x0578 and `LX/BXE.dynamicMethod`
0x01c4 supply the field schemas; `LX/C1B.<clinit>` 0x0028–003c labels type 4
`EPHEMERAL_SYNC_RESPONSE`. No new connection or Signal ratchet mutation was needed.

Labels (docs/design/README.md 6.1): `Observed` · `Derived` · `ReferenceOnly` · `Unknown`.
Every important wire constant used by the code points here through a
`ledger_ref` string. **No `Observed` transport fact exists yet**: nothing has
been captured from the pinned Android client or a live server.

Reference client: `com.whatsapp` 2.26.36.75 (code 263607533), APK sha256
`ce746890…922e9`, signing cert sha256 `39:87:D0:43:…:5D:44`. Full record:
`crates/whatsapp-api/fixtures/reference-manifest.json`.

Reference implementations (web multi-device profile, **not** Android primary):
`tulir/whatsmeow` @ 2375e175 (MPL-2.0); `oxidezap/whatsapp-rust` @ 4a0721b (MIT
headline; see ADR 0005 for AGPL files).

| Ledger ref | Fact | Value in code | Label | Source / notes |
| --- | --- | --- | --- | --- |
| `framing.length_prefix` | Frame length prefix width | 3 bytes big-endian | ReferenceOnly | whatsmeow `socket/constants.go` `FrameLengthSize`; whatsapp-rust `framing.rs`. Web uses WebSocket; Android transport (direct TCP vs WS) is `Unknown` (`transport.endpoint`). |
| `framing.max_frame_len` | Maximum frame body | `1 << 24` | ReferenceOnly | whatsmeow `FrameMaxSize`. Local product limit defaults to 1 MiB (`Limits`). |
| `framing.connection_header` | One-time header before first frame | `WA 0x06 <dict_version>` | ReferenceOnly | whatsmeow `WAConnHeader`; whatsapp-rust `WA_CONN_HEADER`. |
| `binary.frame_flags` | Payload flag byte; bit 1 = zlib | `0x02` | ReferenceOnly | whatsmeow `binary/unpack.go`. |
| `binary.tags` | Structural tag bytes | LIST_EMPTY=0, DICT_0..3=236..239, INTEROP_JID=245, FB_JID=246, AD_JID=247, LIST_8=248, LIST_16=249, JID_PAIR=250, HEX_8=251, BINARY_8/20/32=252/253/254, NIBBLE_8=255 | ReferenceOnly | whatsmeow `binary/token/token.go`; whatsapp-rust `token.rs`. |
| `binary.tokens` | Token dictionary content and order | 235 single + 4×256 double | **Observed (client, static)** | Recovered in order from the `const-string` initialiser in `classes2.dex @ 0x126a96` of APK 2.26.36.75 by `cargo run -p xtask -- apk-tokens <apk> --check fixtures/tokens/web-md-v3.json` (2026-09-17); identical to the web v3 list from whatsapp-rust/whatsmeow. |
| `binary.tokens_version` | Dictionary version byte sent in header/prologue | 3 | ReferenceOnly | Not established from the APK (an integer constant, not a string). |
| `jid.servers` | Server names | `s.whatsapp.net`, `g.us`, `lid`, `broadcast`, `newsletter`, `c.us`, `hosted`, `hosted.lid`, `msgr`, `interop` | ReferenceOnly | whatsmeow `types/jid.go`. |
| `jid.ad_domain_types` | AD_JID first byte → server | 0→user, 1→lid, 128→hosted, 129→hosted.lid, else agent | ReferenceOnly | whatsmeow `NewADJID`. |
| `noise.pattern` | Handshake pattern | `Noise_XX_25519_AESGCM_SHA256` (IK resume also used by web) | ReferenceOnly | whatsmeow `NoiseStartPattern`; whatsapp-rust `consts.rs`. docs/design/README.md 8.3: hypothesis until confirmed. **APK 2.26.36.75 also contains twelve pattern names including ML-KEM512 hybrids** (`noise.client_patterns`). |
| `noise.client_patterns` | Pattern names compiled into the client | 12 names: XX, IK, XXfallback and nine `*kem*MLKEM512*` hybrids | Observed (client, static) | `xtask apk-strings`, 2026-09-17. Which one the server negotiates/accepts is `Unknown`. `snow` has no MLKEM512 hybrid patterns (ADR 0004). |
| `noise.prologue` | Prologue | `WA 0x06 <dict_version>` | ReferenceOnly | Same bytes as the connection header in the references. |
| `noise.trust_root` | Root public key for cert chain | `1423 7557 4d0a … a8ee 6b` | ReferenceOnly | whatsmeow `handshake.go` `WACertPubKey`; whatsapp-rust `WA_CERT_PUB_KEY`. Signature scheme XEdDSA. |
| `noise.issuer_serial` | Intermediate issuer serial | 0 | ReferenceOnly | whatsmeow `WACertIssuerSerial`. |
| `noise.cert_chain` | Chain structure and check order | `CertChain{intermediate,leaf}`; details/signature(64) | ReferenceOnly | whatsmeow `verifyServerCert`; schema `WACert.proto`. |
| `noise.record_limit` | Max Noise record size on the wire | — | Unknown | Needed to accept `snow` (65,535-byte limit); ADR 0004. |
| `schemas.*` | Protobuf schemas | `WACert`, `WAWebProtobufsWa6`, `WACommon`, `WAAdv` | ReferenceOnly | `proto/whatsmeow/` @ 2375e175. |
| `transport.endpoint` | Chat endpoint (port, TCP vs WebSocket, TLS layer) | — | Unknown | Active pinned profile remains unresolved. Comparison APK 2.26.37.70 uses Java sockets: built-in primary/fallback gateway branches are raw TCP on 443/5222; numbered edges use TLS on 443 and raw TCP on 5222; port 80 uses raw TCP wrapped in bounded HTTP/1.1 chunked streams (`POST /chat`, host `c.whatsapp.net`). |
| `transport.hosts` | Chat gateway hostnames in the client | `g.whatsapp.net`, `g-fallback.whatsapp.net`, `e1`–`e16.whatsapp.net` | Observed (client, static) | `xtask apk-strings`, 2026-09-17. |
| `registration.host` | Registration hostname in the client | `v.whatsapp.net` | Observed (client, static) | `xtask apk-strings`, 2026-09-17. |
| `registration.vocabulary` | Path fragments and parameter names in the client | `v2`, `code`, `exist`, `register`, `security`, `verify`; params `cc`, `in`, `lg`, `lc`, `mcc`, `mnc`, `method`, `token`, `id`, `e_regid`, `e_keytype`, `e_ident`, `e_skey_id`, `e_skey_val`, `e_skey_sig`, `fdid`, `expid`, `backup_token`, `authkey`, … | Observed (client, static) | Presence only; assembly, encoding, order and required subset are `Unknown` (`registration.payload_encoding`). |
| `login.client_payload_fields` | Protobuf field constants matching web `ClientPayload`/`HandshakeMessage` | `CLIENT_HELLO`, `CLIENT_FINISH`, `HANDSHAKE_MATERIAL`, `CONNECT_TYPE`, `CONNECT_REASON`, `DEVICE_PROPS`, `LID_DB_MIGRATED`, … plus new `HN_EPOCH1_EPH_PUB`, `EPOCH1_TRANSCRIPT_HASH` | Observed (client, static) | Corroborates the vendored `WAWebProtobufsWa6` schema as a starting point; primary-device field values remain `Unknown`. |
| `login.client_payload` | `ClientPayload` fields for a primary Android device | — | Unknown | whatsmeow builds a WEB/companion payload; primary markers, version fields and startup requests are not observed. |
| `registration.endpoints` | HTTPS registration endpoints | — | Unknown | Not observed. |
| `registration.payload_encoding` | Request/response encoding | — | Unknown | Not observed. |
| `registration.prerequisites` | Keys/ids needed before first request | — | Unknown | Section 7 requires generating them before the first request that needs them. |
| `registration.verification_steps` | OTP / PIN / other steps and cooldowns | — | Unknown | Not observed. |
| `messaging.id_format` | Wire message-ID format | — | Unknown | Web references use uppercase hex of random bytes; tickets carry `wire_id: None` until observed. |
| `e2ee.*` | Session/message versions, prekey upload, padding, retry | — | Unknown | Gate ADR 0005. |
| `devices.*` | Device list discovery/invalidation | — | Unknown | |
| `media.*` | Media construction | — | Unknown | |
| `store.*` | Local store facts (not wire) | see ADR 0002 | Observed | Tested offline in this repository. |

## Dated client observations (static)

2026-09-17 — APK `com.whatsapp` 2.26.36.75, string tables of `classes*.dex`
extracted with `cargo run -p xtask -- apk-strings <apk>` (449,216 unique
strings). Findings are in `crates/whatsapp-api/fixtures/reference-manifest.json`
under `client_observations`. Presence proves the client contains a constant,
not how it is used; nothing here changes a wire fact from hypothesis to fact
without a capture. Native `libs.so` carries no protocol strings.

2026-09-17 — a separately hashed, WhatsApp-signed newer build
(`com.whatsapp` 2.26.37.70, code 263707033, sha256 `b2090b…ee732`) was used as
a comparison because the official `/current/` URL had advanced and the pinned
APK was unavailable. Structural bytecode analysis observed `WA 06 03`, the
`https://v.whatsapp.net` base plus 17 `/v2/*` paths, UTF-8 URL-encoded form
POST bodies, construction of the `authkey`/`e_*` key bundle, the same
`WhatsAppLongTerm1` root and issuer serial used by the reference code, and
chat connection candidates on ports 443/5222 (with a port-80 fallback). Its
token dictionary differs from the pinned fixture. Its registration bridge
also parses the JSON response status/reason and cooldown vocabulary recorded
in the comparison fixture. The key bundle uses the client Noise public key,
key type `05`, big-endian 4-byte/3-byte identifiers, and unpadded URL-safe
Base64. The `/v2/code` token is derived locally with PBKDF2-HMAC-SHA1 over the
package/logo material and HMAC-SHA1 over the signing certificate, primary-DEX
MD5, and national number. Fresh fallback prerequisites are persisted UUIDv4
values for `fdid`/`expid`/`access_session_id` and 20 random bytes apiece for
the recovery and backup tokens; the exact recipes are in the comparison fixture.
The built-in socket path does not construct a WebSocket: it supplies a Java
`SSLSocketFactory` for numbered-edge port 443, but uses raw TCP for primary or
fallback gateways even on port 443 and for all built-in port-5222 branches.
Dedicated port-80 states wrap those raw sockets in a pseudo-HTTP/1.1 chunked
stream: `POST /chat`, host `c.whatsapp.net`, an 8,192-byte output chunk buffer,
and 1,024-byte response/chunk-line bounds.
These are version-scoped comparative observations only: the
pinned-profile labels above remain unchanged. Full details and hashes are in
`fixtures/comparisons/whatsapp-2.26.37.70.json` and `decompilation-log.md`.

Additional comparison framing observation (2026-09-17): `LX/1NX;->write(VLII)`
rejects lengths at or above `0x1000000`; `LX/1Lz;->A04(LI)` encodes the
accepted length as three big-endian bytes. This establishes a static writer
bound of 16,777,215 body bytes for 2.26.37.70 only, not an observed server
Noise record limit. See the continuation entry in `decompilation-log.md`.

## Dated server observations

The active pinned reference manifest still has `server_observation: null`.

2026-09-18 UTC, comparison messaging lab: the standard snow XX message adapter
received a server hello but failed decryption. APK sequencing showed an extra
empty-payload transcript hash in the adapter, absent from `LX/1NZ.A06`. After
correcting that sequencing over existing cryptographic primitives, the client
verified the WhatsApp root/intermediate/leaf chain and its binding to the Noise
remote static (intermediate serial 3, leaf serial 361). A 34-byte minimal primary
login using saved registration credentials returned encrypted `success`.

One recipient-key query returned identity, signed and one-time keys. The client
verified the signed key, established Signal encryption, atomically committed
session/send intent and transmitted one direct text. An encrypted matching
`ack`, `class=message`, without `error` established server acceptance. A local
repeat of the same request read the persisted acknowledgement and sent nothing.
Handset delivery/read is unconfirmed. No phone, plaintext, key or private stanza
is recorded here. These observations apply to the comparison lab only, not the
pinned default profile. [Implementation and static sources](messaging-transport.md).

2026-09-18 UTC, comparison lab only: one operator-authorized diagnostic
`POST https://v.whatsapp.net/v2/code` reused an existing legacy attempt's
configuration and keys. The server returned HTTP 200 and parsed JSON
`status=fail`, `reason=bad_param`; the bounded response body was 58 bytes.
Parsed retry/SMS/voice waits were absent. No successful code delivery,
registration, or specific offending parameter was established. The new
HTTP status and recognized reason were verified in the encrypted saved state.
No further request followed, and the durable one-shot diagnostic latch remains.
Phone, tokens, keys and raw request/response bodies are intentionally not published.

2026-09-18 UTC, follow-up comparison: an explicitly authorized diagnostic
repeat on the separately supplied comparison number reused its saved keys and
unchanged 612-byte form. HTTP 200 / `fail` / `bad_param` was unchanged; the
response was again 58 bytes. The updated parser retained no recognized `param`
hint (`invalid_parameter: null`). This does not prove that no unknown hint was
present. No new optional fields were populated and no successful signup or OTP
delivery was established. The dedicated bad-parameter diagnostic latch is used;
no automatic retry or identity rotation occurred.

2026-09-18 UTC, APK-only controlled continuation: empty-H, seven fallback
fields, and post-failure reason/metrics experiments each still returned
HTTP 200 / fail / bad_param. The next experiment used the APK's
unknown-build-properties User-Agent and advanced the checked preparation count
to 7. The 1167-byte ENC/H request returned HTTP 200 / fail / format_wrong
(64-byte response, 189 ms). The new refusal is persisted on the same attempt
and account keys. This is changed behavior, not successful registration.
No recognized offending-field hint or wait was supplied. Detailed inputs and
static offsets are recorded in [the HTTP analysis](http-post-analysis.md).

Subsequent APK-only analysis resolved `format_wrong` through response enum 29,
`VerifyPhoneNumber.A19`, and dialog 53 to resource `0x7f123741`: the official
client says the mobile number is invalid for the country. This identifies the
client's interpretation, not the server's internal rule. No additional live
attempt, identity change, or successful signup occurred during this analysis.

Next operator-requested test used a different supplied number and its own
private JSON/store, leaving the previous attempt untouched. The JSON selected
`enc_empty_h`, the APK unknown-build-properties User-Agent, the seven documented
fallback fields, and `client_metrics` preparation count 1; no prior-failure
reason was copied. A phone-bound token was derived from the same comparison
APK. One 1126-byte POST returned HTTP 200 / fail / bad_token (61 bytes, 559 ms),
with no recognized parameter hint or wait. This is a different refusal, not
accepted registration, proven protocol progress, or confirmed OTP delivery.
No retry followed. Because number, fresh identity, and request context differ,
this is not a controlled single-variable comparison with the previous attempt.

Token audit subsequently found a single-byte transcription error in the
PBKDF2 salt: offset 62 was `0xfc`, not the APK's `0xf8`. After correcting it,
the private profile changed only its token. One explicitly authorized retry
preserved the same attempt/account material and all other inner fields
(including the previous metrics value for this token-only comparison).
The fresh encrypted envelope was 1123 bytes. The server returned HTTP 200 /
sent (347 bytes, 3219 ms), retry/SMS wait 125 seconds and voice wait 65 seconds.
The saved state is now an SMS challenge. This is server-reported dispatch;
receipt, OTP verification, completed registration and messaging are not yet
confirmed. No further request or OTP submission followed.

The operator then selected the original historical number again. Its saved
opaque refusal was migrated once through corrected JSON: fixed phone-bound
token, APK unknown-build-properties header, ENC/empty-H encoding, documented
fallback fields and history-checked preparation count 2. Original account keys,
attempt, country/locale and registration IDs were retained. The 1126-byte POST
returned HTTP 200 / fail / too_many_all_methods (329 bytes, 1017 ms), with
32417 seconds for retry, SMS and voice. The service persisted a cooldown;
no further request or delivery-method switch followed. No SMS dispatch or
successful registration was established for this number.

## 2026-10-07: comparison 37.71 sticker descriptor and key domain

**Confirmed/static:** `BZC.dynamicMethod` (`00de..00ec`), with field-name
array helpers `6al.A1W` and `Ayq.A11`, establishes all 22 StickerMessage
fields, including skipped wire tags and integer `premium`.
`1qB.<clinit>` (`0028..0034`, `02a0..02be`) passes the image-key label to
the sticker type. **Negative/tooling:** automatic descriptor extraction refuses
the incomplete directly visible name array; exact-string sticker-key-domain
xref has no references. **Offline:** ordinary URL-bearing WebP receive media
uses the existing bounded authenticated retrieval and sealed references.
**Unknown:** live sticker receive/display, Lottie and directPath-only retrieval.
Full evidence and reasoning: [stickers](stickers.md).

## 2026-10-07: video and PTV receive media

**Confirmed/static, 37.71:** Message fields 9 and 66 reference BZE; its
`dynamicMethod` descriptor (`01a4..01b2`) plus `Ayq.A12` establishes the
30-field shared schema, including caption=7 and viewOnce=20. `1qB.<clinit>`
(`0054..006a`, `02e4..031a`) passes the video-key label to both types.
**Negative/tooling:** automatic field extraction refuses helper-filled names.
**Offline:** ordinary URL-bearing MP4 retrieval and event/reference projection
use the distinct domain and existing bounded downloader. An independent
Python/OpenSSL fixture checks domain separation and journal/retrieval behavior.
**Unknown:** live incoming/playback, directPath-only retrieval, HD variants,
circular rendering, sending and view-once video consumption.
[Full video evidence](video-media.md).

**Observed/negative live check:** a rebuilt saved-account 60-second receive-only
window authenticated/activated and ended cleanly. No new event or receipt was
recorded, so it does not verify incoming video/PTV or automatic reads. The
complete 20-event journal and nine acknowledged receipts were unchanged; the
existing uncertain action was preserved. Private evidence paths are in the
video document.

## 2026-10-07: safety-number schemes and packed native boundary

**Confirmed/static, 37.71:** Java `0jc.A0V` / `CjU` uses sorted serialized
identity-key sets, normalized UTF-8 identifiers, 5200 SHA-512 iterations and
six five-digit decimal groups per side. Its legacy QR uses version zero and
raw concatenated public identities. **Refuted:** A0V is not a native wrapper;
one diagnostic SHA-256 pin is not its safety-number input. **Unknown:** native
A0T/A0U algorithms, active UI scheme, inventory producers, modern QR comparison
and human verification. The native ELF is an outer superpack archive container;
ordinary strings/export inspection cannot establish absence of fingerprint code.
The disassembler now names the seven previously opaque opcodes used in these
traces, with literal DEX fixtures. [Full evidence](safety-number.md).

## 2026-10-07: native decoder and loader baseline

**Confirmed/static, 37.71:** the APK includes separate ARM64/x86_64 Superpack
decoders; the earlier scoped name search missed them. One bounded XZ stream
reveals V3/V4 fingerprint creation/comparison anchors, but does not reconstruct
an ELF (all four ELF-magic matches have invalid header bytes). x86_64 JNI
relocations and entry-point disassembly establish the `spo` branch and archive
start/end resolution; reconstruction internals remain unknown.
**Observed/local:** a pinned four-file public Qiling rootfs and Python 3.12
environment run hello to `exit_group(0)` with a narrow proc-exe stat adapter.
Python 3.14, single-threaded futex and proc-exe stat failures are preserved,
along with remaining loader warnings. No APK decoder or live account request
was executed. [Hashes, offsets, reproducer and limits](native-unpacking.md).

**Follow-up, confirmed/static:** ARM64 JNI relocations identify range entry
`0x9a50` and simple entry `0x9ec8`; wrapper `0xa964` has an exclusive upper
range bound. **Observed/local:** own-source, fail-closed asset/JNI adapters
allow the exact decoder to load and register 19 methods, returning `0x10006`
and guest exit zero. First-entry reconstruction reaches the work queue but
returns native `9/2` / `AssetDecompressionException`, exit 7, no file. Real
host meminfo removes one warning but not this failure. The cause is unknown;
neither equal printed byte counts nor initialization prove extraction.
**Refuted/fixed:** Qiling's threaded runner ignores its ordinary timeout/count
arguments. Parent-process wall limits and shared block charging now enforce
bounds, tested with actual hello/infinite ARM64 guests. An initial C shadowed
JNI pointer and proc-mapper API owner error were our harness bugs, corrected.
No live account request occurred. [Full follow-up evidence](native-unpacking.md).

**Observed/confirmed, worker follow-up:** stream 6's produced/declared sizes
are 125448, but requested bytes are zero. It reaches the oversized-file branch
`0x1efa4`, not a traced checksum mismatch. Producer `0x1e544..0x1e564` and
worker `0x1ef00..0x1f078` distinguish the requested and declared fields; the
diagnostic prominently prints the latter. Partial-range activation of the
indexed requested-size selection remains inferred. Full `decompress_from_so`
checks first timed out with no output/refusal; this refutes generalizing the
range's `9/2` result to a demonstrated full-extraction failure. Read-only
hash-pinned tracing and a normal full-entry probe are now reproducible, with
tests for preserving partial files and rejecting unknown decoder layouts.

**Observed/local positive:** QEMU 11.1.1 completes ordinary full extraction
through the same unmodified decoder and own-source JNI probe. Executable-mode
and read-only mount-point setup failures are corrected reproducibly. The
isolated checked-in wrapper returns process zero and reproduces all 79 unique
ELF hashes (80 names, duplicate libsodium.so), totaling 62743576 bytes. All
parse as little-endian ARM64 ET_DYN with bounded file segment extents.
**Confirmed/static:** reconstructed libwhatsapp.so contains the four V3/V4
fingerprint anchors and identifiable ADRP/ADD references/unwind ranges.
Dynamic exports/direct relocation references are absent for those names;
input/crypto algorithms and dispatch mapping remain untraced. The redundant
Qiling long run was explicitly terminated after QEMU success. No account
state was opened or live request sent. [Complete evidence](native-unpacking.md).


**Confirmed/static, reconstructed 37.71 libwhatsapp.so:** shared V3/V4 hash
helper `0x713938..0x7139e4` finishes/restarts SHA-512 once plus 5199 further
rounds, hashing previous output plus its second data argument. Both create
paths concatenate sorted key data and call H(K,K); their display routines
construct a separate identifier-bound seed with two zero bytes. V3/V4 params
place PN/LID inputs differently; JNI-backed normalization remains untraced.
Sort slot `0xd1c2d0` resolves to `0x712ef0`, which requires equal-length
representations and uses memcmp. Modern QR field mapping/current scheme and
human verification remain unfinished. **Observed/local:** two bounded native
control-flow slices with host-modeled SHA/MCF calls agree on 5200 finishes and
10400 updates. This does not execute native crypto or verify an account.
[Full source addresses and model limitations](safety-number.md).


**Confirmed/static, same 37.71 reconstructed ELF:** stable PN/LID adapters
`0x6b7a04` / `0x6b7a94` call Java `JniBridge.jnidispatchOO` selectors 1/2,
resolved through initializer method slot `0xd36ae8`. The PN/type-zero branch
uses HNy shortening; type-18 LID branches return the parsed JID user and native
conversion rejects non-ASCII output. Key constructors resolve raw 32-byte sort
data versus full 33-byte serialized hashing data (raw adapter prefix 0x05).
Java V4 creation selector 2 now connects through the registered dispatch entry
and context wrapper to the V4 native create routines. Its result selector 4
passes QR serialization version zero; alternate selector 5 passes one.
This refutes inferring QR version 4 from the V4 algorithm name. Current UI
scheme/input inventory and individual QR fields/comparison semantics remain
unfinished. Exact-class getType searches missed the obfuscated LID class;
0ZY.getType confirms type 18. No live request or account verification occurred.
[Complete callback/relocation/DEX evidence](safety-number.md#native-to-java-identifier-callback-and-dispatch-mapping).


**Confirmed/static, 37.71 native QR:** libwhatsappmerged.so initializer and
unpack wrappers resolve two protobuf-C message descriptors and HostedState.
FingerprintData has six optional fields: public_key, PN, LID, username,
hosted_state and hashed_public_key. The compact committed schema artifact
records actual tags/types/offsets/addresses; upstream descriptor ABI matching
is not proof of the APK's protobuf-C version. Serializer version zero carries
concatenated serialized keys, version one H(K,K), not the display digest.
V4 can deliberately omit local PN when a username exists. Comparison resolves
orientation by LID and requires selected key equality plus identifier conditions;
matching LID/key alone is insufficient. Hosted enum is E2EE=0/HOSTED=1; unknown
is params sentinel -1, not an invented enum. Username comparison flag semantics,
complete error remapping and verified-state action remain unfinished. Fourteen
local tooling checks pass; no live scan or account verification occurred.
[Schema and comparison evidence](safety-number.md#native-qr-schema-serialization-and-v4-comparison).


**Confirmed/static, 37.71 fingerprint comparison caller:** KXj.A01 invokes
V4 selector 4 with a native holder and feature 0x1d2d; 1xm.A1P accepts exactly
zero. NFC/camera/async callers and success callbacks reach presentation through
A18. No persisted verification action is established; delayed callbacks and
writer chain remain unknown. Other same-dispatch operations are not fingerprint
evidence. [Selection and action trace](safety-number.md#java-comparison-selection-and-scan-result-action).


**Confirmed/static, 37.71 verification writers:** Cg3.A00 checks rendered
identity then rechecks serialized bytes under the address lock before updating
verification metadata and sequence. Companion peer actions require a LID,
32-byte key and fresh positive sequence; primary inbound actions drop.
DNu history applies a distinct BT5 path. BWE descriptor tags 1..4 and
ProtocolMessage field 32 are recovered. Delayed scan callbacks are presentation.
No live verification/metadata update occurred. [Writer evidence](peer-identity.md#verification-metadata-writers-and-rotation-guards-3771).


**Confirmed/static, 37.71 local verification control:** DUm selector 15
binds primary LID, rendered key and saved verified bit; 6Ac installs 5XZ.
Its marked branch schedules unmark, while unmarked opens confirmation whose
positive callback schedules mark. Runtime feature/exclusion predicates and
complete fingerprint inventory remain unknown. Two missing DEX field-write
mnemonics are fixed with literal APK bytes and actual APK re-rendering.
530 all-feature tests and strict Clippy pass; no account state changed.
[Control and decoder evidence](peer-identity.md#local-verification-control-and-captured-identity-3771).

### comparison.binary.tokens.37_71 — confirmed static

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
`1Rl.<clinit>`: typed indexed stores publish 236 single and 4×256 double tokens.
Four historical comparison page/index-1 values are refuted by actual stores;
this establishes an extraction defect, not an upstream dictionary change.
[Token extraction](token-extraction.md) carries offsets, negative results and
runtime fixture provenance. All 1,260 entries equal the separately held web-v3
fixture. Live acceptance of the four corrected slots remains unknown.
[Core release review](core-release-review.md) records 22-topic coverage and seven
manual relocation decisions; text differences do not establish behavior changes.

### comparison.group.subject.typed_37_71 — confirmed static

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
0hs.A09 calls signed Long.parseLong; A08 is the separate optional-default path.
0Fs.A0A reads cached 0Ze.A04; A02 ties it to database group_type at 047c/0aa4.
0yv.A09 branches on types 1/3 at 477c..478c; typed message constructions and
Bo3/BoJ getters/setters are preserved in [source evidence](group-subject-types-evidence.txt).
1bh.<init> stores the supplied system kind in A00. Full-signature constructor
resolution, message storage and receive parity remain unknown/unimplemented;
see [interpretation](groups.md#subject-typed-branches-and-signed-timestamp-contract-2026-10-07).
The long parser diagnostic also matches integer/reference-valued methods;
method_shorty JLL distinguishes them without class-name fallback. VLL yields
no_matching_shorty and no proposal. Two string-free helpers remain unanchored.
0Xx.A02+00ba raw 16 02 ff ff 31 01 09 02 occurs once in classes.dex at 0x5cbc2e:
-1 insert-result comparison, not 65535. Corrected display retains hex bits and
signed meaning. No server request or saved-account access occurred in this work.

### comparison.group.info.recovery_37_71 — confirmed static

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
0Xm.A0H constructs false/false A9r arguments for normal mode 2; 5/6 alone use
its in-flight-set branch. A0F consults 8km.A02's 60,000-ms local monotonic guard,
then normal-JID dispatch submits 0Xo.A02's MEX QueryGroupInfo. That context
builder maps ten explicit reasons, unknown reasons to UNKNOWN, and a private
set membership override to MISSING_PARTICIPANT_IDENTIFICATION. Our working
classic IQ info path is not this recovered APK path. 9Np selector-0 callbacks
and numeric error 429 handling are partial; no retry duration was established.
[Evidence](group-info-recovery-evidence.txt) and [interpretation](groups.md#subject-recovery-mex-dispatch-and-local-guard-2026-10-07)
carry offsets, reasoning and remaining unknowns. No client MEX route, handler,
server wait bypass, identity reset or live request was added by this research.

### comparison.mex.group_query_37_71 — confirmed static, offline Rust port

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
QueryGroupInfo server ID 37759635623635554 is selected from the namespace's
persisted-ID asset (SHA-256 a528a786209342b3e1d8cf8a78dc046599731c3842ea4bc3b316c64647bdfec0).
164.CFb reads doc_id; the separate AFf client_doc_id feature path is not this
JSON/XMPP ID. Lae/15t/161 resolve it; Lai constructs queryId/variables and passes
it to BpQ selector 3 and 9QZ(VLLLL). The same ID appears in trace/flow_id,
query_id and JSON. 1S7.getServer, 8kY.A1H, 8kX.A0T and 8ka.A0a close the
server-only destination/type/id helper trace. GraphQlCallInput.A00/A01 confirms
recursive input objects and 1xg.A1P JSONObject string serialization. Missing
registration returns null, not a guessed identifier. Source assertion semantics,
actual namespace-registration filename producer, Pando/Argo feature selection,
timeout units and complete response/error conversion remain unknown.
[Interpretation and release/CLI workflow](mex.md),
[request evidence](mex-request-evidence.txt) and
[partial response evidence](mex-response-evidence.txt) preserve reasoning and
negative results. LaT chooses result/update; LaZ distinguishes data/errors
parse failures; 2FU itself has no schema methods. Initial 0Xo.A03 reads
xwa2_group_query_by_id through 2FT/90x/ADv, not a proven complete projection.
Rust's offline builder and selected-ID exporter are implemented and invokable;
no durable MEX action, response application, HTTP route or live verification
is claimed. Existing account identities, reservations and snapshots untouched.

### comparison.mex.json_response_37_71 — confirmed static, offline response-parts port

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
00S/1UE provider 0x1763 resolves through the checked packed-switch entry
(classes.dex 0x909e70 fd010000, target A0E+03fe) to 0yN.A00/O4r selector 0.
Lab/LaZ constructors resolve selectors 2/1/4 for nullable object data, nullable
array errors, and malformed-errors exception conversion. Independent failures
preserve the other field. LZw stores data/errors; LZv's nonempty flags do not
establish group success. 90x.A0J confirms subject/8yq rather than the earlier
inferred getter interpretation. Complete callback, per-error model, Argo/Pando,
Android lexical JSON acceptance and group projection remain unverified.
[Interpretation](mex.md#json-response-field-port-2026-10-07),
[source evidence](mex-json-parser-evidence.txt) and
[raw switch entry](mex-json-provider-switch.json) preserve offsets and reasoning.
Rust response-parts parsing and a counts-only CLI inspector are implemented,
including explicit invalid states, preserved counterparts and redacted Debug.
Sixteen synthetic getter-derived combinations exercise result/update; no
server response, live request or durable group-state update is claimed.

### comparison.mex.group_callback_37_71 — confirmed static, offline gate port

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
Lac/LaT typed model conversion dispatches Okj.C2c; 9Np directly extends
BaseMexCallback, whose A03 consults A06 before nonempty-data A04. Group
selector-0 A06 always returns false, including 429 handling: partial data with
errors cannot reach success callback. I1H requires every error entry to be a
JSONObject. 2FU requires nonnull data; A04's 0yw.A0x gate can skip conversion.
[Source/interpretation](mex.md#group-callback-gate-and-inherited-dispatch-2026-10-07)
include IWF extensions/root code precedence, backoff/severity/path/message,
selected-error ranking boundary and partial ADv/LbE getter findings. Full
transport classification, exception/coercion/ranking, native/Pando/Argo,
converter and durable application remain unfinished. Rust group disposition
and the optional offline --group-info inspector are implemented; no live
request or account/state change is claimed. apk-smali's checked superclass
header exposes inherited ownership rather than guessing absent functionality.

## comparison.mex.callback_source_coverage_37_71 — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [eighteen-topic recipe](apk/recipes/2.26.37.71-mex.json) now reproduces
Lac.BEE, LaT.A00, BaseMexCallback.A03, 9Np.A06, IWF.AYS and I1H.A06 as
separate versioned methods with Rust ownership, on top of twelve unchanged
request/parser topics. Exact export/check: 23 files/76,908 bytes.
[Coverage diff](apk/mex-callback-coverage-diff.json) compares old Git cdd4056
against this change, using the same APK hash: six added, twelve unchanged;
targets metadata changed, other three artifacts unchanged, 0.0069 seconds.
This is cached source coverage, not a new release or six protocol changes.

[Actual remap](apk/remap-evidence/2.26.37.71-mex.json): eight unique, seven
unanchored, three ambiguous. IWF.AYS code/error_code plus shorty I is unique;
is_retryable plus shorty Z still matches I1H.A06 and 916.BGC/BQ2. Unknown
new-release mapping remains explicitly unresolved; no full recipe proposal.
[Interpretation](mex.md#versioned-parserrequest-files) distinguishes source-only
getters and callback admission from unfinished converter/live application.

## comparison.mex.group_converter_branches_37_71 — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Selected source](mex-group-converter-evidence.txt) preserves full 0Xo.A03,
90x getter fields, required properties getters, 8kY.A0B/0Gv.A01, group id parser,
creator-id getter, feature-gated 0Xo.A01/A00 and required LbE.A05. Four
__typename hash branches construct A4m with distinct integer/flag inputs;
unknown hashes/null group object throw, subject is nullable, participants and
properties are required. Exact wire type names and A4m slot mapping remain
unknown. Searching 453,954 escaped DEX strings found none of the four hashes;
this does not exhaust runtime assembly/unescaped UTF-16 names.

Group/subject times multiply parsed long by 1000; announcement_version does
not. Raw classes6.dex 0x26374c bytes `16 00 e8 03 bd 02 10 02` independently
prove 8kY.A0B's opcode/operands; xtask now names bd mul-long/2addr with an
independent regression. Parser catch types/coercion/storage units remain
unknown. Creator is creator.id, with feature 0x36fe controlling additional
LID/country maps. [Interpretation and boundaries](mex.md#group-converter-branches-and-time-arithmetic-2026-10-07)
record source facts without claiming a live or completed Rust converter.

## comparison.mex.constructor_and_catches_37_71 — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[A4m/catch source](mex-group-constructor-catch-evidence.txt) and
[literal code_item](mex-long-parser-catch-fixture.json) resolve group type
A03, distinct creation/subject/version/appeal long slots and ordered map/flag
inputs. Wide arguments occupy two words; interpreting every argument as one
register would mislabel later fields. [Full mapping/boundaries](mex.md#result-constructor-and-explicit-exception-inspection-2026-10-07)
retain unknown enum/getter meanings and unfinished state application.

0Gv.A01 catches NumberFormatException over code units 7..12, handler 12 returns
the supplied default (zero at traced callers). This closes the previous
unknown parse-failure path, not all Java numeric lexing/coercion. LbE.A04
catches four reflection exceptions at one null-return handler; it has no
arbitrary Throwable catch. 0Xo.A03 has no local try range; caller handling is
not refuted. Reusable apk-smali --exceptions adds checked table inspection,
synthetic malformed/overflow/catch-all tests and independent APK regression.
Source format AOSP DEX code_item/try_item/encoded_catch_handler_list;
no live account/server validation or full group converter claimed.

Inspection failure correction: apk_smali previously warned on a later DEX
parse error and returned earlier methods. It now propagates the named file
error before rendering any method result; two-DEX tests cover malformed bytes
and invalid catch tables following valid methods. This changes failure
reporting, not valid default output or protocol state.

Core rendering correction: the full 37.71 check found noise-handshake
+045e..0460 op=bd had retained ?? after the preceding multiplication-name fix.
Searching for CLI-style ??(0xbd) missed bundle op=bd syntax. Exact replacement
shows no other method-content change; same operands/offset/opcode. Only that
file/checksum refreshed, core 27 files/731,494 bytes. [Cached diff](apk/core-mul-long-rendering-diff.json)
against Git 34b5ab5: 0.0328 seconds, one changed/21 unchanged topics, all four
artifacts unchanged, same APK. This is presentation, not a protocol delta.

## comparison.mex.numeric_projection_37_71 — 2026-10-07

**Confirmed static APK calls**, build 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
ADv.A04/A05 -> 8kY.A0B -> 0Gv.A01 typed NumberFormatException/default zero,
then signed multiplication 1000; ADv.A03 unscaled; A4m A0A/A0B/A08 slots.
Rust GroupNumericMetadata now projects the three numbers after callback gate
and required group-object checks; it does not validate/apply a complete group.
CLI --group-numbers (requires --group-info) explicitly prints private numeric
values as signed decimal strings. Default summaries/Debug remain redacted;
blocked callbacks/wrong group shape refuse without partial output.

**Executed host reference / inferred platform compatibility:** independent
[Java oracle](MexJavaNumericOracle.java), Temurin 21.0.12.1+1 LTS, produces
[479 cases](../../crates/whatsapp-api/fixtures/mex/2.26.37.71-numeric-reference.json)
and the complete UTF-16 digit-domain enumeration (370 digits/37 BMP blocks).
Tests compare parsing/default/wrapping and all 65,536 code units. AOSP
JSONObject/JSON String.valueOf supports inferred numeric coercions; strict
JSON lexical acceptance and live Android SDK/Unicode behavior remain unverified.
[Interpretation/reference links](mex.md#numeric-group-projection-into-rust-2026-10-07)
separate APK evidence, host reference, inferred coercion and unknown runtime.
No account opened, saved identity/state mutated or live request sent.

## comparison.mex.participant_roles_37_71 — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
2hu enum ordinals, LbE.A0C -> 4R7.A00 Locale.US/no-trim/typed fallback,
3c5.A0C ordinal and both ADv.A0D stored-role branches. Rust nullable-string
role helper now preserves member versus unrecognized; CLI mex-role-inspect
prints only recognized output. **Executed independent host reference:** 91
Java cases, Temurin 21.0.12.1+1 LTS; complete Android Unicode parity unknown.
[Findings and boundaries](mex.md#participant-role-conversion-and-member-map-boundaries-2026-10-07),
[attributable selected bytecode](mex-participant-role-evidence.txt).

Additional **confirmed static prerequisites**, unported: matching phash returns
null/retain; Ayj.A1R true only for integer 1; BotJid branch/domain bot/type26,
1..20 ASCII digits/no leading zero; separate strict UserJid branch and optional
aliases; 32q alias type filtering. Unknown wire typename hash preimages are
preserved as unknown; complete list/coercion/device/result-field mapping and
member application are unfinished. No saved account or live server touched.
