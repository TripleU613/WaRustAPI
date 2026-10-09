# Evidence ledger

Token clock producers (2026-10-09, **confirmed static/local audit**):
APK `08B.A07` takes login `success@t` through `1Rd` or selector-4 Bundle time,
sets an elapsed baseline and persists client/server difference. Seven source
topics close those inputs; selector-4 wire provenance and full Rust clock parity
remain unknown/unported. Current stanza clock sampling is still an inference;
no real account or live change.
[Evidence and limits](token-clock-producers.md).

Trusted-contact peer guard (2026-10-09, **confirmed local / static-backed**):
privacy-token receive now rejects empty PN/LID users before storage or ACK;
supplied empty sender_lid keeps a fixed diagnostic. Generic server-only JIDs
previously passed our namespace-only guard. A failing-before regression checks
text/Plain/AD inputs, unchanged tokens/pins/sessions and reopening; valid token
handling remains covered. No real account/live change.
[Evidence and limits](token-notification-peer.md).

Token mapping consumers (2026-10-09, **confirmed static/local**):
selected table readers feed a separate specifier cache and native stable-ID list.
`00D.A0C` compares/logs values; its caller separately chooses native or fallback
return. Readiness and initial preference inputs are traced, effective account
settings remain unknown. Renderer now decodes APK shift opcodes b8/c5 with a
failing-before regression; seven source topics preserve the boundary.
[Evidence and limits](token-feature-inputs.md#mapping-consumers-and-shadow-comparison-2026-10-09).

Token MobileConfig mapping (2026-10-09, **confirmed static**): provider
233 resolves to `09N`; its generated array has nonzero initial specifiers for
15491 (`0x811d83000016e2`) and 14303 (`0x811af9000014f5`). Eight source topics
preserve the binding and indexed lookup. Effective native values/readiness and
array consumers remain unknown; no runtime/live change.
[Evidence and limits](token-feature-inputs.md#property-to-mobileconfig-mapping-2026-10-09).

Token Boolean provider bindings (2026-10-09, **confirmed static**):
`00D` binds `1Y4(4)` and `1Y5(0)` to its Boolean providers. Both resolve IDs
through `09M.B2y`; zero mapping returns false, nonzero calls MobileConfig
`AVn`. Six source topics preserve bindings/selectors/branches; actual mappings,
native values and effective precedence remain unknown. No runtime/live change.
[Evidence and limits](token-feature-inputs.md#boolean-provider-bindings-2026-10-09-continuation).

Token feature inputs (2026-10-09, **confirmed static/local**): APK 37.71
registers properties 15491/14303 as false Booleans; its getter can use caches,
preferences or MobileConfig, so effective values remain unknown. Offline
`abprops-query --show` now reports selected returned token values and invalid
last-value flags, without default fallback or runtime activation. Twelve source
topics and regression/reopen tests preserve the evidence. No real account or
live change. [Evidence and limits](token-feature-inputs.md).

Token canonicalization gates (2026-10-09, **confirmed static**): APK
37.71 normalizes type-zero phone users independently of property 15491, which
suppresses only counterpart lookup/cleanup. Separate property 14303 selects
pending-job namespace. The six-topic snapshot records incoming-byte versus
outbound timestamp getters and four physical counterpart-helper callers. Effective
flags, mapping authority and refresh runtime remain unknown/unported. No account
state or live request changed.
[Evidence and limits](token-canonicalization.md).

Identity-only peer prerequisite (2026-10-09, **confirmed local**): public
query/inspection now refuses an empty peer user before reading or saving action
state; shared primary-address matching also refuses server-only values. The
regression failed before the fix and covers PN/LID, Plain/AD encodings, unchanged
pins/sessions/checkpoints and reopening without replay of an uncertain query.
This is our input guard, not a WhatsApp refusal; no real account or live change.
[Evidence and limits](peer-identity.md#empty-peer-user-refusal-2026-10-09).

Receipt ACK state boundary (2026-10-09, **confirmed static/local**): APK
37.71 DDH preserves ACK status 10/12 and zero time; selected ordinary items
can reach 15A.A07 with a null callback through Cmn. Permanent-failure status
21 skips that update. Deferred jobs preserve key direction/status/time and use
a separate processing budget. The six-topic snapshot closes model/deferred-job
inputs, not complete persistence or primary user-row authority. Rust ACKs still
settle receipt reservations separately from peer facts; no runtime or live change.
[Evidence and limits](receipt-ack-state.md).

Receipt ACK handler bindings (2026-10-09, **confirmed static/local**): APK
37.71 N12 providers resolve read selector 86 to 14c and state selector 262
to 17J. Read processing checks chat type 5; state processing enqueues Cmn
with arguments 0/5000/0. The offline switch renderer now retains every declared
case and rejects malformed tails, closing its former 64-case visibility gap.
The eighteen-topic snapshot preserves concrete worker boundaries; full persistence
and live parity remain unknown. No account state or live request changed.
[Evidence and limits](receipt-ack-handlers.md).

Specialized receipt ACK producers (2026-10-09, **confirmed static**): APK
37.71 read ACKs produce selector 86 bundles; played/server-error produce selector
262 models with arguments 10/12 and zero time. The fourteen-topic snapshot
traces arg1 dispatch to N12 A0d/A0Z or registered/fallback handlers. Concrete
application and persistence remain unknown; these are not peer timestamps or
receipt_user membership. No runtime, account state or live request changed.
[Evidence and limits](receipt-ack-special.md).

Receipt ACK dispatch (2026-10-09, **confirmed static**): APK 37.71 generic
ACKs remove full Cb1 callback keys under a map lock, then separately handle ID
queues. Read-job registration normalizes only device-zero address forms; duplicate
registration replaces callbacks after logging. The ten-topic snapshot retains
null versus original-node callback payloads. Full queue ownership, timeout/error
lifecycle and receive effects remain unknown. No runtime, account state or live
request changed.
[Evidence and limits](receipt-ack-dispatch.md).

Receipt ACK participant guard (2026-10-09, **confirmed static/local**): APK
37.71 Cb1 compares nullable participants. Rust now rejects a participant-less
ACK for a group read/retry reservation; incomplete error ACKs also leave the
saved outcome unknown. Regression coverage checks restart/no replay and complete
matching ACKs. The former CZH attribution is corrected and retained as a negative.
No private state or live request changed; full primary cleanup remains unfinished.
[Evidence and limits](receipt-ack-correlation.md).

Linked metadata observers (2026-10-09, **confirmed static**): APK 37.71
case 26 invokes a no-payload Bjk callback. A physical scan finds thirteen empty
concrete bodies and two state-update consumers; transitive effects/registration
remain unknown. The five-topic snapshot also names the browser/desktop flag
and distinguishes view lookup from raw-cache lookup. No runtime, account state
or live request changed; this is not identity-verification or receive-replay proof.
[Evidence and limits](linked-metadata-observers.md).

Linked-device runnable writes (2026-10-09, **confirmed static**): APK 37.71
DLo case 23, DM7 case 2 and DNh case 20 write history access, last-active time
and nickname. A three-topic anchored snapshot completes immediate classification
of the twenty physical cache stores; transitive authority remains unfinished.
Last-active mutates a record before SQL, and only its separate companion
preference update has a greater-than check. No runtime, account state or live
request changed.
[Evidence and limits](linked-runnable-writes.md).

Linked-device metadata caches (2026-10-09, **confirmed static**): APK 37.71
place/instrumentation writers update SQL, null only the view map and mutate
one existing raw-map record. The five-topic snapshot distinguishes that behavior
from GEu's broader store-reset context. Location requirements, metadata authority,
remaining runnable writers and complete linked-device trust remain unknown.
No runtime, private account state or live request changed.
[Evidence and limits](linked-metadata-cache.md).

Linked-device cache writes (2026-10-09, **confirmed static**): APK 37.71
has twenty physical raw/view cache writes across fourteen methods. The six-topic
snapshot traces metadata writes, logout marking versus row deletion, own-alias
selectors and bounded PN/LID migration resets. Full remaining writer/caller
authority and commit/rollback semantics remain unknown; no cache reset implies
verified identity or removal permission. No runtime, account state or live
request changed; linked-device receipt handling remains unfinished.
[Evidence and limits](linked-cache-writes.md).

Read-job saved linked set (2026-10-09, **confirmed static**): APK 37.71
requires loaded Me and derives its set from saved devices, excluding positive
logout times. The nine-topic snapshot traces SQL loading and bounded own
LID-to-PN device conversion with existing-PN collision preference. Set membership
is not key verification; full ingestion/deletion authority, cache invalidation
and read-self fanout remain unknown. No runtime, account state or live request
changed; primary cleanup remains unfinished.
[Evidence and limits](read-linked-set.md).

Outgoing read-job trace (2026-10-09, **confirmed static**): APK 37.71
SendReadReceiptJob uses Cp2's shared receipt builder and extra-ID list construction.
Its outgoing read/read-self selection differs from the incoming factory policy;
job admission also checks IDs, status age and a saved set. The eleven-topic
snapshot supersedes wire-format inference. Rust still sends one explicit ID
per receipt; complete policy, linked-device scope and scheduling remain unfinished.
No runtime behavior, account state or live request changed.
[Evidence and limits](read-receipt-job.md).

Receipt read-policy classifiers (2026-10-09, **confirmed static**): APK 37.71
uses a configured JID-user list, chat codes -1/-3 and contact-addon presence.
The twelve-topic snapshot retains dependent-account age-state checks and a
predicate that can schedule a refresh before returning true. Effective state,
classifier writers and refresh completion remain unknown; full user-row
admission and primary cleanup remain unfinished. No runtime, account state
or live request changed.
[Evidence and limits](receipt-read-classifiers.md).

Receipt factory read policy (2026-10-09, **confirmed static**): APK 37.71
uses read_receipts_enabled default true plus chat, contact and account predicates.
The eleven-topic snapshot retains literal smb:/ent: prefix checks, premium-message
existence queries, dependent-account preference and the entry thread guard.
Effective preferences/classifiers and settings synchronization remain unknown;
full user-row admission and primary cleanup remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](receipt-read-policy.md).

Receipt state-only suppression (2026-10-09, **confirmed static**): APK 37.71
C8L selects incoming keys and mapped state arguments 13/16/17. Its state-only
caller skips A07 when the chat last-read message row is at least message A0j;
the missing-chat-model getter returns 1. A two-topic snapshot and eleven-write
inventory retain the immediate boundary. Full chat read-state lifecycle,
self/user-row admission and primary cleanup remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](receipt-state-suppression.md).

Receipt state-only admission (2026-10-09, **confirmed static**): APK 37.71
keeps message-key direction separate from receipt status: read-self forces false,
while played-self retains the recipient-presence branch. The ten-topic snapshot
traces A07 advancement, played eligibility and core persistence. A07 true does
not prove every downstream database update succeeded; the base A0V helper tests
chat kind, not a view-once flag. Full self/user-row admission and primary cleanup
remain unfinished. No runtime, account state or live request changed.
[Evidence and limits](receipt-state-only.md).

History mode provenance (2026-10-09, **confirmed static**): APK 37.71
BMv reads positive registration_device_id from preferences; BNm tests loaded
Me presence. These are separate from own-user equality, registration state and
linked-device authentication. An eight-topic snapshot retains mode/getter/setter
and the bounded completion-handler source. Full pairing/crypto and receipt_user
admission remain unfinished. No runtime, account state or live request changed.
[Evidence and limits](history-mode.md).

History queue admission (2026-10-09, **confirmed static**): APK 37.71
stages history chunks in the peer handler after own-user and companion-mode
checks. SQL insertion-constraint recovery updates the original message ID;
queue selection and null/empty binding differences are retained. The four-topic
snapshot preserves full staging/store methods. Upstream crypto, linked-device
trust, media authentication and receipt_user creation remain unfinished.
No runtime, account state or live request changed.
[Evidence and limits](history-queue.md).

Mapping import entry points (2026-10-09, **confirmed static**): APK 37.71's
remaining conversation-mapping callers are chat-transfer import and the initial
history-bootstrap branch. The immediate history inputs are bounded file reads
or inflated inline chunks; nonce presence/storage is not demonstrated nonce
verification. A four-topic snapshot preserves the callers and full worker.
Upstream chunk/transfer authentication and receipt_user admission remain unknown.
No runtime, account state or live request changed.
[Evidence and limits](mapping-admission.md).

Chat counterpart provenance (2026-10-09, **confirmed static**): APK 37.71
uses a HashBiMap populated from distinct positive original chat-JID rows and
from parsed conversation account-LID/phone pairs. Reverse lookup, chat deletion,
cache clearing and a mapping-sync consumer are retained in a twelve-topic snapshot.
The backup restore caller is distinct from direct wire admission; upstream trust,
full sync receiver and receipt_user membership remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](chat-counterparts.md).

Receipt counterpart selection (2026-10-09, **confirmed static**): APK 37.71
8G8.A02 expands selected JID rows only through the migration gate and the
0Yo.A0B(input, false) mapping path; its comparison uses object identity.
JID-row materialization does not establish receipt_user membership. The eleven-topic
snapshot retains the mapping/gate and full untraced chat-cache loader. Mapping
producers, full user-row admission and primary cleanup remain unported.
No runtime, account state or live request changed.
[Evidence and limits](receipt-counterparts.md).

Receipt metadata provenance (2026-10-09, **confirmed static / inferred lifecycle**):
APK 37.71 has four physical route-flag writers and hydrates 1ic mentions from
message_mentions, separately from receipt_user. Clearing its list preserves
lookup bit 1 and the route flag; BotGroupMetadata attachment explicitly sets or
clears a separate bit. The fourteen-topic snapshot preserves these sources and
remaining constructor inventories. Complete user-row admission and primary
cleanup remain unported. No Rust runtime or account state changed.
[Evidence and limits](receipt-metadata.md).

Receipt metadata route (2026-10-09, **confirmed static**): APK 37.71 special
classification requires chat exclusion, flag/first qualifying metadata user,
message A02 and descriptor absence. It does not count recipients; A0z bypasses
metadata selection only. The fourteen-topic snapshot retains the helper chain.
Disassembler opcode-c2 naming is corrected with a literal-byte regression.
Metadata producers, full user-row admission and primary cleanup remain unported.
No messaging runtime, saved identity or reservation changed.
[Evidence and limits](receipt-route.md).

Ordinary cursor status (2026-10-09, **confirmed static**): APK 37.71 0VA.A00
reads status into the virtual message setter and loads server receipt time
without enum conversion or minimum selection. Two physical cursor entry points,
the type factory and one status-1 media preparation source are retained in a
five-topic snapshot. Full outer SQL/cache and user-row creation remain unknown;
no Rust runtime, reservation or account state changed.
[Evidence and limits](message-loader.md).

FStatus conversion (2026-10-09, **confirmed static**): APK 37.71 7z7.A00
maps named 1cE enum ordinals with media/subtype conditions; 7zY.A01 keeps
native 178 integers separate. The ten-topic snapshot retains the transition
matrix and all six direct enum writes. This narrows the earlier restoration
trace: ordinary SQL status loading and universal outgoing initial status remain
unknown. No Rust runtime or account state changed.
[Evidence and limits](status-enum.md).

Outgoing success ACK participant trace (2026-10-09, **confirmed static / tested offline**): APK 37.71
wire parser → device conversion → event 19 → handler proves that a nonnull
converted participant suppresses success acceptance. Rust now guards supported
PN/LID participants, including primary forms, while preserving error ACKs and
original uncertain reservations across restart. The twenty-topic snapshot also
retains native accepted-message SQL, status branches, failure helpers and scoped
poll-path negatives. Native extra user subtypes, complete status/user-row parity
and a qualifying live ACK remain unverified. No sends or account resets ran.
[Evidence and limits](message-ack.md).

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
null (retain intent inferred; downstream application untraced); Ayj.A1R true only for integer 1; BotJid branch/domain bot/type26,
1..20 ASCII digits/no leading zero; separate strict UserJid branch and optional
aliases; 32q alias type filtering. Unknown wire typename hash preimages are
preserved as unknown; complete list/coercion/device/result-field mapping and
member application are unfinished. No saved account or live server touched.

## comparison.mex.exception_aware_release_coverage — 2026-10-07

**Confirmed static/tooling**, same APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
recipe expands to 38 methods / 43 files / 162,435 bytes and opts into explicit
catch tables via include_exception_tables=true. Default false preserves old
formats. [Actual cached diff](apk/mex-participant-coverage-diff.json), Git
78a7218 old paths: 0.023364 seconds, twenty added/eighteen annotations-only
changed; prior instruction content identical, inventory/manifest/tokens same.
Fixture-based catch-only mutation regression proves the diff blind spot fixed.

**Negative relocation results:** [report](apk/remap-evidence/2.26.37.71-mex.json)
14 unique / 17 unanchored / 7 ambiguous; creation_time/JL matches both times,
invalid-bot/LL both parsers, bot/L four methods. No automatic full proposal.
**New static catch facts:** 0hj.A03 local catch-all 0009..000e -> 000e, versus
0D1.A00 NumberFormatException 0106..010e -> 0191, code units. Complete members,
device/alias/feature/state conversion remains unknown/unported; account state
unchanged and no live request. [Reproduce/interpret](mex.md#exception-aware-numeric-and-participant-release-coverage-2026-10-07).

## comparison.mex.participant_model_and_edges — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
LbE.A0A/A09 object-array filter and empty fallback, 8yl constructor, 31c
field slots, 1xf.A17 int-to-long join boxing, 8kW.A0q nullable-history value
boxing, 3c9.A0E optString typename/hash. A0F is not a required field reader.
**Independent literal class_def evidence** proves 0hh -> abstract 0ZX ->
UserJid; 0ZX has no class data, not no class. Strict UserJid is not PN-only or
disjoint from bot. [Full findings/sources](mex.md#participant-model-fields-and-object-edge-projection-2026-10-07).

Rust group_participant_edge_roles and explicit offline --edge-roles now expose
ordered recognized roles of object edges, before phash/model/JID validation.
Eighteen synthetic cases cover both payload children; private diagnostics and
default summary remain redacted. This is not a member map/state application.
Constructor A08 extra string, final join-time unit, device hierarchy and full
member/alias/boolean/feature application remain unknown/unported. No saved
account access, identity reclassification or live request.

## comparison.mex.participant_scalars — 2026-10-07

**Confirmed static** APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
LbE.A0G/Ayj.A1R optInt==1 then optBoolean; ADv.A0D/1xf.A17 positive join int
boxed long, isNull/8kW.A0q nullable history. **Inferred platform / executed
reference:** six hash-pinned AOSP org.json sources commit de876a01b29230b877c9f408348c38a90ee724a5,
Temurin21.0.12.1+1 with compile-only annotation stubs, 408 independent cases.
**Observed local defect fixed:** default serde float parser rounded
0.99999999999999994 to1, falsely setting phash true; float_roundtrip restores
Java-reference below1 result. The exact literal remains a regression.
[Sources/oracle/semantics/unknowns](mex.md#participant-scalar-coercion-and-float-rounding-2026-10-07).

Rust scalar projection/explicit --participant-scalars now exposes private
phash/history/join, with redacted Debug/default output; integer Number wrapping
versus string/floating saturation and decimal/hex/suffix/NaN/Infinity supported.
This is not a member map or state application. Full JSONTokener/Android runtime
parity and remaining member/JID/alias/feature assembly stay unverified/unported.
Four topic owners add the coercion Rust module; 38 unchanged topic texts,
owner-only cached diff0.016635s, 43 files/162671bytes. No account/live action.


## comparison.mex.participant_addresses — 2026-10-07

**Confirmed static** APK 2.26.37.71 / SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
0D2.A02/A04, strict 0ZZ.A00/0D9.A00, PhoneUserJid/0ZY/9Lm constructors,
0D9.A02, 3c8.A1a, 0CM.A0J, DeviceJid and device-subclass constructors.
Thirteen literal class definitions prove DeviceJid is outside UserJid:
strict participant/alias checks reject device wrappers; plain bot/interop qualify.
Phone prefix/length/ASCII rules and plain hosted-domain normalization are traced.
Interop uses last hyphen and signed Java parseInt; no unsigned-only rule inferred.
[Source, reasoning and negative results](mex.md#participant-address-validation-and-strict-type-boundary-2026-10-07).
Remaining unknowns: singleton types, platform parseInt parity, Ble final assertion,
full member assembly and phash downstream application. Existing send policy,
API/CLI capabilities, saved account state and live transport remain unchanged.


## comparison.mex.strict_participant_address_port — 2026-10-07

**Confirmed static** APK 2.26.37.71 / SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
0D2.A00/A02/A03, seven scoped singleton/factory classes, nineteen literal domain
class definitions settle strict UserJid special cases and non-user domains.
Server/0 and bare Me placeholders qualify; gdpr/bare-server/group/broadcast/
newsletter/call/temp/interop.g.us do not. **Implemented** ParticipantAddress
strict primary parsing and redacted offline inspector, separate from send policy.
283 executed host Java Integer.parseInt cases validate platform conversion;
Android Unicode/cache parity and member/alias/durable application remain unknown.
Observed negative-prefix CLI flag bug fixed with regression and actual CLI checks.
[Sources, reasoning and reproduction](mex.md#strict-member-primary-address-parser-in-rust-2026-10-07).
Release recipe now 50 methods / 55 files / 191422 bytes, twelve additions,
38 prior texts unchanged; cached diff0.051814s. No saved account/live request.


## comparison.mex.nullable_ordered_members — 2026-10-07

**Confirmed static** APK 2.26.37.71 / SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
LbE.A0E/A0F nullable/default optString, 8kU.A1A id, 32q.A00 filters and inherited
Jid.equals/hashCode/raw formatting (08p.A00 nullable Object.equals).
**Implemented** ADv.A0D member map with distinct phash-null variant, object-edge
filter, Java typename-hash dispatch, bot skip versus strict UserJid/alias failure,
ordered duplicate replacement and mapped role/join/history/display fields.
**Independent host execution:** eight Java String.hashCode cases; 41 synthetic
APK-branch cases through both children. Wire typename names remain unknown;
fixture preimages are not claimed server spellings and were never sent.
[Source/reasoning/limitations](mex.md#nullable-ordered-participant-map-port-2026-10-07).
Explicit local refusal for unported Java floating/structured optString forms;
this is not an APK field-type requirement. Downstream phash handling, complete
group conversion/Android/cache parity and live/durable application remain open.
New private --participant-members inspection composes with existing projections,
redacts Debug/default output and never emits a partial failed map. No account/live
request, sending-policy change or identity reclassification.

0DE.A0Z/A0e/A0X are numeric type predicates (18/19/25; 0; 22), not
instanceof checks. Strict caller alias parsers prevent device aliases from entering
this port; broader 32q callers remain untraced. Phone/LID types 0/18 confirmed.
59 methods/64 files/204600bytes; nine added topics, seven owners updated,
50 prior method texts unchanged; checksum-verified cached diff0.019005s.

8yk/8yg/8yj constructors confirmed as nonnull JSONObject wrappers only;
no additional field validation found. Refusals identify the offending attribute
without its value; diagnostic fixture checks cover id/lid/pn/display_name/__typename.


### Participant string metadata (2026-10-07)

**Confirmed static (APK 2.26.37.71):** LbE.A0E/A0F delegate nullable/default
strings to JSONObject; ADv.A0D consumes them for display metadata/model dispatch.
**Observed reference:** pinned AOSP org.json on host Temurin 21.0.12.1+1 agrees
with the scoped ordered Rust conversion across 424 raw JSON and 2242 IEEE
references. Original source/token distinctions, duplicate ordering, escaping,
container integral reduction and negative zero are preserved. An independent
exact-rational Java 21 formatter implements the specification; no JDK source
was copied. **Observed corrections:** serde's integer `-0` classification needed
scoped normalization; Python splitlines broke literal Unicode separators in
oracle rows and was replaced with physical-LF splitting. Both negative results
and reproduction evidence are in
[the string conversion trace](mex.md#participant-metadata-string-conversion-2026-10-07).
The existing CLI exposes these conversions privately and keeps default summaries
redacted; API metadata remains CLI-only/x-live false. **Unknown:** Android
runtime-version and full lexical parity, actual typename spellings, full group
application/live MEX settlement. Owner-only re-export: 59 unchanged method texts,
64 files/205234 bytes, cached diff 0.038028 s; baseline Git a737c69. No account
state access/mutation or live network request.


### MEX downstream participant storage (2026-10-07)

**Confirmed static, APK 2.26.37.71:** exact A4m.A0O field search finds one
behavioral reader, 0yw.A0X; it forwards the nullable map to A0U. Null skips
A00 membership reconciliation while other group-field updates continue; empty
nonnull maps still reconcile. A00 scales positive 31c.A05 by 1000, otherwise
preserves positive previous 1uU.A01 or uses 08B.A00. 0bA.A00 maps nullable history
to prior integer state: true minimum 2, false null/zero→ 1 otherwise unchanged,
null preserves. **Inferred:** join storage milliseconds/raw seconds from the
multiplication and clock calls. **Observed reference:** pure Rust metadata helper
and CLI match 432 host-Java transcribed branch cases; not Android/APK execution.
**Unknown:** clock calibration, higher history-state meanings, inner storage
atomicity, full alias/device/own-account reconciliation and live MEX durable
application. Those are not inferred from A0i delegation order. Seven methods
added to the versioned bundle; 59 existing texts unchanged,66 methods / 71 files /
285828 bytes, cached diff 0.035567s, baseline 8295e18.
[Trace, negative boundaries, evidence and reproduction](mex.md#downstream-member-storage-and-null-map-2026-10-07).
No private account state or live operation changed.


**Observed research tooling correction (same chunk):** CI hygiene's checksum
pass covered only the hardcoded 37.70 snapshot. It now verifies every discovered
checksum snapshot separately; four bundles/2029764 bytes and 621 local links
passed. A standalone corruption/size/extra-file regression is wired into CI.
Earlier 564315-byte hygiene output did not establish newer snapshot coverage;
MEX exports were separately checked by apk-research-diff. See
[checksum correction](apk-porting.md#checksum-coverage-correction-2026-10-07).
No new protocol/API capability is inferred from this tooling change.


### Member cache and versioned participant hashes (2026-10-07)

**Confirmed static, APK 2.26.37.71:** `1jL.A04/A0T/A00/A0G/A01/A0U/A0R/A05`
trace cache changes, preserve existing devices and nullable history; separate
locks do not prove a database transaction. `290.A02` plus `28z.A00/2l0.A00`
synchronize alias ordinals. A0R/A02 hash user keys; A0N/A0O gather stored device
values, refuting the earlier device-keyed-map uncertainty. `AzR.A03` selects
V2 literal agent zero or feature-0x415b V3 no-agent rendering; both SHA-256 via
Ayj.A19, first six bytes, NO_WRAP via A18, Java UTF-16 sorting, no rendered dedup.
**Observed host reference:** 114 synthetic cases/684 CLI modes match executed
Java; old Rust rules disagree on 2 V1 and 89 V2-set probes. Six redacted refusal
probes pass. Pure offline module fixes default xtask's feature-gating build
failure; no dependency changes. **Unknown:** active feature, Android charset/
subtype equality, complete own/companion inventory and durable/live application.
Automatic group sends still V2 primary-member hashing before device resolution.
[Sources, reasoning, reproduction and negative boundaries](groups.md#participant-cache-and-versioned-hashes-2026-10-07).
Snapshot: 86 methods/91 files/337088 bytes; all 66 older texts unchanged, cached
diff 0.052231 s against 875ed8f. No account access, new numbers or live sends.


### Saved group device hash inputs and send-job consumers (2026-10-07)

**Confirmed static, APK 37.71:** 1jL.A0I/J/K/L and 0bA.A0H/I select cached
PN/non-PN device/user hashes; SendE2EMessageJob.A06 compares separate hashes
in its gated mismatch resolver and emits groupParticipantHash as phash.
DNu/Ceo receipt/message-specific source branches are distinct from fresh group
inventory; 0Z5.A0O is abstract here, concrete behavior remains unknown. A0W
PN-map replacement skips/logs non-PN entries, does not itself refresh hashes.
**Observed local correction:** the planner hashes saved recipient companion
devices plus known own primary, instead of ignoring addressed companions;
independent Python vector 2:Fhoy7LxL versus primary-only 2:79jEAyGD. Foreign
saved devices refuse. CLI group fanout-inspect exposes counts/own unknown
without addresses or implicit hash; --hash reveals only the requested candidate.
Device preparation uses message context. **Unknown:** full own-device/account
sync, fresh/retry selection, active feature and live companion send parity.
34 group methods/39 files/773068 bytes, 20 prior texts unchanged; cached diff
0.045388 s, baseline 0e216b2. Actual CLI checks used only an isolated encrypted synthetic store: default
redaction, explicit hash, repeated counts and content-free refusal passed. No
canonical account opened or live operation performed. 639 workspace tests and
24 default CLI tests, strict Clippy/Rustdoc/reference/hygiene passed.
[Reasoning, sources, negative results and reproduction](groups.md#saved-device-hash-planning-2026-10-07).


### Scoped send-preparation device queries (2026-10-07)

**Observed local correction:** lab group/status fetch_devices formerly queried
only one global 64-user batch and returned success even on refused/partial
refreshes; cached lists could then permit preparation. Selected pending batches
now reserve atomically, process a fixed recipient set once, and require accepted
complete per-user replies before advancing. Distinct-user accounting prevents
duplicate rows hiding omissions. QueryRecord's new omitted_users is nullable:
historical/unsettled unknown stays null, never inferred from old counts. CLI
device-query exposes saved status/counts and numeric codes only, no HTTP/reset.
**Confirmed substrate:** APK 37.71 0u7.A06/C8f, SyncDevicesJob presets and
11T.A07/A08 previously selected request/parsing/verification traces. The 64-user
bound and completeness guard are local policy, not newly proven APK requirements.
**Observed offline:** 130 targets settle 64/64/2, unrelated pending remains;
503 with old lists and duplicate-row omission block; explicit later 404 keeps
prior state, refused record stays refused; reopen/null/redaction cases pass.
**Unknown:** live large-group batches, own account sync, complete ADV-key/fresh
source selection and public-outbox group parity. No APK snapshot change claimed.
[Reasoning, negative cases and reproduction](messaging-transport.md#scoped-device-batches-and-completeness-2026-10-07).


**Observed live follow-up, same chunk:** a 20-second receive-only session after
zero-send/action/receipt/device-pending preflight completed one existing-peer
device read. Saved query: server_acknowledged, one requested/ingested user,
zero partial errors/omissions, no code; pending users returned to zero. Retained
one primary/no signed index list, companions unverified. Nine ACKed receipts
and action counts (1 error/15 ok/1 sent) stayed unchanged; no chat send occurred.
Default incoming-only logging hid the INFO counters and caused a harness-only
assertion failure; the same completed session was inspected, never restarted.
This proves shared one-user accounting, not live selected multi-batch/group send.
644 workspace tests, default24, strict Clippy/Rustdoc/reference passed; actual
synthetic/query CLI redaction and canonical saved-query inspection passed.


**2026-10-07 — background device revision/continuation safeguards.** Confirmed
substrate: APK 37.71 0u7.A06/C8f, SyncDevicesJob presets, 11Q.A09,
11T.A07/A08 and Cp2.A05. No new upstream trace or APK bundle change. Observed
local lost-work races (old reply/new notification and ACK write cut) and single
startup-batch limit corrected using durable local generations and a per-connection
cursor. Exact failed/omitted/unknown users remain held; successful newer work and
other batches advance. Offline actor/reopen tests establish local behavior;
live races, full own-account sync and notification-ID dedup remain unknown.
[Reasoning and regression scope](messaging-transport.md#background-device-revisions-and-continuation-2026-10-07).

Live follow-up 21:57:48 UTC: one existing-peer device read in a bounded
20-second receive-only session saved server_acknowledged, one requested/ingested,
zero errors/omissions/superseded, code null. Device pending returned to zero;
outbox/actions/nine ACKed receipts unchanged, no chat sends. Primary only,
companions still unverified. Live races/large batches/socket-cut remain unknown.
Validation: 651 workspace/all-feature tests, default CLI24, strict Clippy,
warning-free Rustdoc, CLI reference and formatting passed. Six actual isolated
CLI probes plus live saved-query inspection passed; hygiene645 local links,
2405965 bytes/four unchanged APK bundles.


Contact ADV caller trace (2026-10-07, APK 37.71): eleven exact selected
methods now preserve supplied-key verification, stale/invalid caller branches,
notification addition and primary identity/session application. Saved identity
is not always the APK's selected key. Invalid/stale signed candidates do not
apply through the success path; raw-ID mismatch has separate reset branches.
Our current fallback actually drops companions and keeps only raw-ID/time floor;
prior preservation comments were wrong and are corrected. Runtime retention,
verified device authority after identity change, feature 0x3285 selection and
full supplied-key integration remain unfinished; no account/live request changed.
[Evidence and unknowns](cross-device-sync.md#contact-adv-verification-and-retention-2026-10-07).
Core snapshot: 33 topics/161 methods, 38 files/967529 bytes. Cached comparison
against b058f61 took 0.028928 seconds: eleven added, all twenty-two old topics
unchanged; inventory/manifest/tokens unchanged, targets metadata expanded.

Validation: exact APK re-export/check verified all 38 files/967529 bytes;
checksum-verified cached diff preserved all 22 older topic texts. CLI reference
regeneration, four CLI OpenAPI coverage checks, API symbol check, formatting and
staged whitespace passed. Hygiene verified 659 local links and 2642000 bytes
across all four independent APK bundles. This source/comment-only chunk makes
no runtime behavior change; no new workspace-test or live-interop claim.


Device candidate retention implemented (2026-10-07): stale/invalid supplied
signed candidates and unsigned non-hosted companions now preserve cached lists,
pins/sessions and pending work. Fixed local verification_failures counts remain
separate from server errors; device-query exposes counts/null history and send
preparation blocks rejected reads. The receive cursor defers rejected users while
other users advance. Raw-ID replacement/supplied-key application remain unsupported;
no resets occur. Regression covers cache/pin/session preservation, duplicate rows,
encrypted reopen, fresh valid settlement and mixed missing-pin/other-user success.
[Evidence and limits](cross-device-sync.md#rejected-device-candidate-retention-2026-10-07).

Conflicting duplicate-row correction: a valid first row could previously clear
pending before a later refused row. Selection now preflights all requested-user
rows before mutation; exact duplicates collapse, differing rows refuse with fixed
local conflicting_user_rows, preserving cached/pending state in both orders and
across reopen. This is local ambiguity policy; APK duplicate dispatch is unknown.
The fixed local reason set above additionally includes conflicting_user_rows.


Observed live 2026-10-07 22:13:51 UTC: zero outbox/queued-action/pending-receipt/
pending-device preflight, one existing-peer read and a 20-second authenticated
receive-only session completed cleanly. Saved CLI query: server_acknowledged,
requested_users=1, ingested=1, partial_errors=0, omitted_users=0,
superseded_users=0, verification_errors=0, verification_failures={}, code=null.
Pending users returned to zero; outbox, 17 actions (error1/ok15/sent1) and all
nine ACKed receipts unchanged. No chat sends or resets. This proves the supported
unsigned-primary live path and new diagnostics schema, not live rejected-list or
supplied-key behavior; the primary-only list still has no signed index evidence.

Validation: 654 workspace/all-feature tests and 24 default CLI tests passed;
strict workspace/all-target Clippy, warning-free Rustdoc, CLI reference/API
coverage and formatting passed. Fifteen actual isolated CLI probes covered all
nine local reasons, zero success, historical/unsettled null, absent, unchanged
repeat and invalid-ID redaction. Conflicting duplicates preserve cache/pending
in either order and after encrypted reopen. Hygiene663 local links and 2642000
bytes across four unchanged APK checksum bundles passed. No live malformed,
signature-refusal or raw-ID replacement claim is made.


Signed-device authority binding and authenticated refresh (2026-10-07): successful
signed lists privately bind their actual verifier key; historical rows remain
unknown. devices/peer-identity expose current authority flags without the binding
fingerprint. Group/status plans refuse unbound/changed cached companions while
preserving history. Already-authenticated remote primary changes queue refresh
inside their pin/session/archive/event transaction; own PN/LID and companions
are excluded. Receive heartbeat picks up eligible post-startup work without a
notification/reconnect and keeps existing refusal/unknown deferral. Offline
rollback/reopen/rebinding and queued-during-receive checks cover the local path;
live primary replacement/own inventory/supplied ADV-key adoption remain unknown.
[Evidence and limits](cross-device-sync.md#signed-device-authority-and-authenticated-refresh-2026-10-07).


Observed live 2026-10-07 22:33:56 UTC: a zero-send/action/receipt/device-pending preflight,
one existing-peer read and a 20-second authenticated receive-only session
completed cleanly. Saved query: server_acknowledged, requested/ingested=1,
partial_errors/omitted/superseded/verification_errors=0, verification_failures={},
code=null. Pending devices returned to zero; outbox, all 17 actions
(error1/ok15/sent1) and nine ACKed receipts unchanged, no chat sends. Authority:
binding_known=false, primary_pin_present=true, primary_pin_matches=null,
companions_current=false. This accurately reports the unsigned primary-only
reply with no signed index list; it does not prove live companion binding,
identity replacement, own inventory or a supplied-key branch.

Validation: 658 workspace/all-feature tests and 24 default CLI tests passed.
Strict workspace/all-target Clippy, warning-free Rustdoc, CLI reference/API
coverage, formatting and staged whitespace passed. Thirteen actual isolated
CLI JSON/human/request probes covered bound/current, changed, missing pin,
historical unknown, unsigned primary and absent list; no private verifier
fingerprint was exported and --json --request parsed as one object. Fixture
metadata probes test inspection, not cryptographic signing; genuine signed
baseline/replacement tests cover authority separately. The own-primary hook
additionally passed explicit PN/LID/companion exclusions. Hygiene668 local
links and 2642000 bytes across four unchanged APK checksum bundles passed.

ADV identity follow-up trace (2026-10-07, **confirmed static**, APK 37.71):
DNy case 40 sends an encrypt/identity read after verifier failure; DF6/DNk
separate primary identity saves from companion ADV validation. Feature 0x3285
enabled skips immediate application; its raw-ID callback does not establish
ordinary deferred success. Raw-ID cleanup removes remote companions while
explicitly preserving the primary. Remaining key-type/companion verifier,
continuation and cleanup internals are unknown; current full peer-keys queries
are not identity-only recovery. No account or live request changed.
[Source and limits](cross-device-sync.md#adv-identity-follow-up-and-raw-id-cleanup-2026-10-07).

Primary identity-only read staging (2026-10-07): peer-identity-query queues the
APK 37.71 encrypt/identity read once; --show classifies saved primary PN/LID
public-key evidence with exact namespace pin match and redacted failures.
B7H type 5 / first 32 key bytes and companion dual-signature checks are traced.
Four offline regressions preserve state and uncertain/refused IDs. Automatic
ADV-failure scheduling, key adoption, companion policy and downstream success
event/session recovery remain unfinished; presence is not verification.
[Evidence and usage](cross-device-sync.md#primary-identity-only-query-2026-10-07).

Identity-only read live validation (2026-10-07): one bounded 20-second session
settled the existing-peer query ok / primary_identity_received, null failure/code,
PN pin match false and no device-identity node. Returned public identity matched
an already saved verified full bundle in an offline cross-check. No pin/session,
outbox/receipt/prior-action change or chat send; repeat reuses the original ID.
This validates the read, not adoption or automatic ADV-failure recovery.
662 workspace/all-feature tests, 24 default CLI tests, strict Clippy/Rustdoc and
CLI/API coverage passed; core cached diff is 0.053777 seconds.

Sender-key device separation (2026-10-07, offline): APK Cnr retains recipient
and device fields; Rust previously flattened companions into the primary's
record. Storage/libsignal names now preserve the actual sender device and keep
primary history in place. Real-crypto PN/LID three-device store/reopen coverage
passes; companions never borrow a historical primary record. Primary save
callbacks additionally trace remote companion cleanup and sender-key rotation;
full actor/category/alias variants and identity-triggered rotation remain
unfinished. Identity-only queries still do not adopt keys.
[Evidence and limits](cross-device-sync.md#primary-save-callbacks-and-device-sender-key-storage-2026-10-07).

Validation of this correction: 663 workspace/all-feature tests, 24 default CLI
tests, strict workspace/all-target Clippy, warning-free Rustdoc and regenerated
CLI/API coverage passed. The 13 added source topics expanded the core to 60
topics / 197 methods, 65 files / 1,528,287 bytes. Cached comparison took 0.053107
seconds: all 47 prior topics unchanged; manifest/inventory/tokens unchanged;
checksums verified. A rebuilt-client 20-second receive-only session ended
cleanly with peer pins, device inspection, actions, outbox and receipts unchanged,
no added query or chat sends. Private evidence index:
`lab/releases/sender-key-live/active.json`. Live companion decryption and
identity-triggered rotation remain unverified; no primary key was adopted.


Sender-key cleanup trace (2026-10-07, **confirmed static** APK 2.26.37.71):
0jc.A19 deletes one translated chat/account-type/device/bucket tuple;
0kX.A00 and 0kT.A02 preserve chat, bucket and sender device during translation.
This does not establish blanket remote incoming-key deletion. Account-provider
selection is partly traced; feature/alias authority and distribution-history
parity remain unknown/unimplemented. The comparison CLI distributes an existing
SKDM to every target but has no sent-sender-key history; redistributing is not
identity-triggered rotation. No live request or identity adoption occurred.
See [evidence and next implementation](cross-device-sync.md#scoped-sender-key-removal-and-recipient-translation-2026-10-07).


Own sender-key exposure/rotation (2026-10-07): the comparison sender reserves
exact recipient-device exposure with key/session/send state. An authenticated
remote primary change marks indexed own keys pending in the same archive/event
transaction; the next new authorized group/status intent rotates with OCC and
private old-key archival. Incoming chains and uncertain intents are retained.
`peer-identity` exposes redacted pending_sender_key_rotations counts; zero does
not establish historical coverage. APK 37.71 D4x.A00/0jc.A19 motivate this local
subset; full bucket/alias/companion and normal actor group parity remain
unimplemented, and live rotation is unverified.
See [evidence and limits](cross-device-sync.md#durable-own-sender-key-exposure-and-rotation-2026-10-07).


Directory bundle selection (2026-10-07): shared parser now compares user/server/
agent/device without conflating wire-encoding kinds, refuses duplicate requested
user rows and optional one-time keys, and refuses contradictory outer errors.
These local ambiguity guards precede identity/session preparation; other batch
devices remain valid. Existing peer-keys inspection reports invalid_reply and
blocks refresh while preserving its selector and pins. APK 37.71 B7H.A06/CQc.A00
provide address/signature context; exact duplicate policy is not attributed to
the APK. [Evidence and tests](peer-identity.md#exact-directory-bundle-selection-2026-10-07).


Missing group sender-key correction (2026-10-07): supported nonempty skmsg with
no sender key is now Undecryptable, reaching the existing once-only retry queue;
it is no longer silently Unsupported. Empty/structured skmsg with a stored key
cannot be reported Decrypted or delivery-confirmed. Byte/Text ciphertext views
are consistent across direct/group receive. Real-crypto late-key recovery and
direct/group loop write-cut fixtures preserve captures, ratchets and retry
reservations. Handset retry shape is observed; APK 37.71 Cp2.A05 is ACK evidence,
not a confirmed missing-key retry count policy. Fresh live group recovery remains
unverified. [Evidence and limits](messaging-transport.md#missing-group-sender-key-recovery-2026-10-07).


SKDM-only group control deduplication (2026-10-07): real Signal regression
reproduced Undecryptable after a previously accepted control was redelivered
across reopen. Newly authenticated controls now commit sender_key_distributed
markers with key/session state and return Duplicate without another decrypt.
They carry no user protobuf/event/read receipt; lab-inbox --inspect labels them
redacted. APK Message field 2 is existing schema evidence; this private marker
is a local atomicity fix, and APK internal duplicate policy is unknown. No
historical marker synthesis or ratchet rewind; fresh live control redelivery
remains unverified. [Evidence and regression](messaging-transport.md#durable-sender-key-only-group-controls-2026-10-07).


Retry/ACK encoding identity correction (2026-10-07): a reproduced second-retry
reservation caused only by JidKind differences is fixed. Canonical logical
selectors preserve user/server/agent/device and probe all historical kind keys
in the same transaction, retaining pending/unknown/refused/acknowledged records
and original wire. ACKs ignore encoding kind but keep exact logical routes.
Independent historical-hash, reopen/state-preservation, foreign-route ACK and
actual receive-loop cut/reconnect tests cover the correction. APK 37.71 Cp2.A05
is wire ACK context; APK duplicate/retry-limit policy remains unknown. Historical
duplicate rows are not consolidated; live encoding changes remain unverified.
[Evidence](messaging-transport.md#retry-reservation-identity-across-jid-encodings-2026-10-07).

Primary-change encoding correction (2026-10-07, confirmed offline): reproduced
missed own group-key rotation for a remote AgentDevice/0 primary and accidental
own-PN device refresh. Encoding is normalized only after primary agent/device
checks; PN/LID namespaces and companion boundaries remain exact. Redacted rotation
inspection now ignores kind without folding devices. PN incoming and LID signed
bundle archive/reopen, own exclusion, rollback and stale-reservation tests cover
the local correction. APK 37.71 D4x.A00/B7H.A06 remain source context; live alternate
primary encoding and subsequent group rotation remain unverified.
[Evidence](cross-device-sync.md#encoding-independent-primary-change-follow-ups-2026-10-07).

Own sender account selection (2026-10-07, confirmed static APK 37.71): 08i
initializes Apw from self_lid and Apz from saved Me.jabber_id; both retain the
persisted registration_device_id (default 0). D4x.A00 selects LID versus PN using
0bA.A0k -> 1jL.A0Y (group row.A00 == 2), then wraps the own device via B7H.A04.
Me.lid consistency diagnostics are not an established immediate self_lid fallback;
1XY is a call callback, not run. Group-row mode production, async reconciliation,
complete bucket/actor semantics and Rust own-address parity remain unfinished.
No account state, keys, identity policy or live traffic changed.
[Sources and boundaries](cross-device-sync.md#own-pn-lid-sender-selection-and-persisted-device-index-2026-10-07).

Group-mode producer/storage trace (2026-10-07, confirmed static APK 37.71): five
1jL.A00 writes and its constructor input are retained. Cache-miss mode comes
from B7i.A00 chat predicates, not membership suffixes alone; contact mode is saved
through 0Y9.A0s -> 0gt.A0V -> 9R1.A0J in wa_group_admin_settings. Reload 0lO.A01
normalizes case-insensitive pn to pn, other returned values to lid. B7i lookup
and predicate have different absent-contact defaults; broadcasts have additional
diagnostic behavior. Full mode provenance, category predicates and Rust mode/
own-PN fallback integration remain incomplete. No live/account state changed.
[Evidence and negative results](cross-device-sync.md#group-mode-producers-storage-and-cache-classification-2026-10-07).

Group routing before device sync (2026-10-07, confirmed offline): the comparison
sender's catch-all plan fallback returned device-query targets even after route
refusals. The extracted old logic failed a mixed-membership regression. Shared
recipients_for_group now validates namespace/own membership before sync, allows
initial missing lists and excludes/deduplicates own/remote targets. Final plan
still validates device authority before reservation. APK mode/category state
integration and automatic own-PN selection remain incomplete; no new live send.
[Routing correction](groups.md#group-recipient-validation-before-device-sync-2026-10-07).

Concrete group-mode kinds (2026-10-07, confirmed APK 37.71): 0DE predicates
resolve broadcast-list type 3, interop.g.us type 28, status@broadcast type 5,
and cache-group types 1=g.us/2=temp. A0n additionally includes type 28.
Temporary groups are not established as old PN g.us groups. Existing 0Fs cache
and ADs reply classifications are linked; numeric group-type agreement alone
does not establish complete callback/history provenance. Runtime mode/contact
synchronization and unsupported namespaces remain unported; no account request.
[Kinds and remaining gaps](cross-device-sync.md#concrete-jid-kinds-in-group-mode-classification-2026-10-07).

Group-type persistence continuation (2026-10-07, **confirmed static APK 37.71**):
existing-chat updates write group_type through transactional database helpers;
missing chats are left absent. Selected subgroup/zero-update callers and secondary
type-4 model writes are retained without inventing complete-reply enum values.
A0Y's outer A0n guard is narrowed again by A0A/A0m, excluding interop type 28
from the type-3 branch. Runtime contact/type synchronization remains incomplete;
no account/network request. [Evidence and unknowns](cross-device-sync.md#persisted-group-type-updates-feeding-sender-mode-2026-10-07).

Addressing-mode producer trace (2026-10-07, **confirmed static APK 37.71**):
complete binary group replies do not directly read the mode attribute. Generated
validators use exact pn/lid membership; incoming group-message parsing has a
separate case-insensitive PN check. Five contact-mode setter callers now have
selected paths retained, including gated info-error and PN-conversion branches;
triggers, full callback provenance and runtime integration remain unfinished.
No live request or account change. [Evidence and boundaries](groups.md#addressing-mode-producers-and-separate-parser-rules-2026-10-07).

Direct LID representation correction (2026-10-07, **confirmed offline**):
textual and typed sender JIDs now select the same held LID session. A real
Signal regression failed Undecryptable before the fix; both representations
now decrypt and cross-representation redelivery remains Duplicate after reopen,
with no PN session or identity rewrite. APK DEw.CF0 parsing is linked, while
full direct-session producer provenance remains incomplete.
[Evidence and boundaries](messaging-transport.md#direct-lid-sessions-across-attribute-representations-2026-10-07).

Group pairwise companion correction (2026-10-07, **confirmed offline**):
held and pinned companion msg sessions retain their actual participant device;
primary/alias fallback, unpinned and prekey-bootstrap paths are excluded from
this local subset. Real Signal PN/LID primary/device-3 control fixtures now
install sender keys and retain duplicate markers across reopen. B7H forward/
reverse device construction is linked; CPQ.A00 is refuted as a 37.71 session
selector. Full companion bootstrap/identity policy and live device coverage
remain unfinished. [Evidence and scope](groups.md#group-pairwise-session-device-correction-2026-10-07).


### Ordinary companion ADV and pending refresh — 2026-10-07

**Confirmed static, APK 2.26.37.71:** DEs.CF2 writes the `device-identity`
body to Cih.A0V; Cih.A01 passes constructor argument 26 to Bp2.A0L.
11T.A0D's matched-primary/raw-ID/type branch refuses old/equal timestamp
candidates with absent/different device index. 0bN.A05 preserves LbQ.A05
(list timestamp) while advancing A02 (expected timestamp); 0bp.A00/A05 and
LbQ.<init> independently identify the SQL mappings. Treating the newer message
ADV as a verified list timestamp was refuted. Callback selection, cache callback,
full alias fan-out and atomic runtime admission remain unknown/unimplemented.
[Trace and reproducible sources](peer-identity.md#ordinary-companion-adv-producer-and-baseline-timestamps-2026-10-07).


### Known companion prekey bootstrap — 2026-10-07

**Confirmed static / implemented offline:** APK 37.71 11T.A0D matched-held-
primary/raw-ID/type branch (`04c8..05fa`) accepts an equal actual device index
with a non-newer candidate timestamp. Group/status pkmsg now implements that
branch using DEs.CF2's ordinary device-identity body, both ADV signatures and
nested Signal MAC authentication. Exact companion state, one-time prekey,
SKDM/control/message are committed atomically with current authority rechecks.
Own-device/unknown-own guards are local limitations; newer timestamps require
separate pending refresh and are deferred. Five real-crypto tests cover success,
replacement/refusal, authority rollback, reopen dedupe and sender-key fallback.
Direct companion prekeys and wider admission/reconciliation remain unimplemented;
no fresh companion live sample is established.
[Implementation and remaining gaps](peer-identity.md#known-companion-prekey-bootstrap-2026-10-07).


### Direct companion receive/receipt route — 2026-10-08

**Implemented/offline:** same known-companion ADV branch now handles direct
prekeys; actual-device ratchet/event/SKDM routing is separate from primary chat
identity. Alias disappearance dedupes. **Confirmed static 37.71:** Cp2.A00 ->
CmC.A06 retains supplied DeviceJid as direct receipt destination for the traced
nongroup/non-A0Q branch; A0Q types 3/5/6 are not a companion predicate. The
initial Cp2.A06 ordinary-receipt hypothesis was refuted (Bp1/status only).
A0p/CM8 upstream producers and full direct-control parity remain unknown.
[Trace, tests and remaining work](peer-identity.md#direct-companion-receive-and-receipts-2026-10-08).


### Known companion newer ADV — 2026-10-08

**Confirmed static 37.71:** `101.A03`/`AzX`/`DLx.run` case 14 schedule deduplicated SyncDevicesJob; `Lcr.A00` preserves an unresolved expected timestamp on a lower normal response. `0bN.A0H` persists, rather than constructs, that baseline. **Verified offline:** known-index direct/group admission records the expectation and queues refresh only with authenticated atomic receive; signed responses below/reaching it retain/clear pending work. Boolean reset semantics, new-device/own-account application and live coverage remain unknown. See [reasoning and reproduction](peer-identity.md#known-companion-newer-adv-refresh-2026-10-08).


Companion new-device membership trace (2026-10-08, **confirmed static 37.71**): `11T.A0D` removes already present addresses before add-only application; a newer different index does not itself overwrite existing membership. `0bN.A0G` filters ownership; `1IW.A02/A09` selects gated canonical users; `AzR.A00` rebinds device numbers; `B7j.A03` writes user_device rows separately from the verified LbQ timestamp. Added-member participant/observer calls and caller teardown Boolean are traced only to their boundaries. Runtime gate values, full side effects and Rust new-device application remain incomplete. Full raw methods and reasoning: [new-device application](peer-identity.md#companion-new-device-membership-application-2026-10-08). No live request or saved account mutation occurred.


Companion teardown producer (2026-10-08, **confirmed static 37.71**): ordinary CoU.A04 combines 0bq.A04, the local dead_devices registry and a nonnull parsed offline Integer for its final admission Boolean. Offline metadata is separate from retry count/dhash. 1Uf.A06 persists removal markers; A02 rechecks current membership before cleanup, dropping returned-device markers without key teardown. Removal callbacks, transient A09 set and local worker bounds are distinct. Rust registry/reap and new-device application remain unported; no account mutation occurred. Disassembler b2 multiplication now has its proper label and independent literal regression. [Evidence and limits](peer-identity.md#companion-teardown-producer-and-registry-2026-10-08).


Remote newer-ADV application (2026-10-08, APK 37.71 static/offline): absent non-offline companions now enter authenticated direct/group receive and are added atomically with provenance/refresh, preserving verified timestamp. Existing different indices are retained until sync. devices exposes adv_added_devices and human expected_ts/count. Signed reconciliation retains newer allocations and clears provenance only on confirmation/removal. Absent offline candidates remain locally deferred pending teardown lifecycle; alias/participant/own reconciliation and fresh live validation remain incomplete. [Implementation and limits](peer-identity.md#remote-newer-adv-device-application-in-rust-2026-10-08).


Pending removal registry (2026-10-08, **static/offline**): signed device reconciliation records disappeared companions and clears authenticated returned addresses in nullable pending_removed_devices. Legacy null stays unknown until signed reconciliation; unsigned responses do not initialize/clear it. Offline absent candidates proceed only with known no-marker state; pending_device_teardown refuses a marked address. Complete-list CAS includes removal state. Inspection exposes unknown/known-empty/count. No cleanup, identity/key deletion, reset, alias fan-out or worker parity is implemented. [Evidence and limits](peer-identity.md#namespace-local-pending-removal-registry-2026-10-08).


## Contact LID result parsing (2026-10-08)

**Confirmed**, APK 2.26.37.71 (`1755111a…7d5c`): `0u7.C8f` reads
`user/lid@val` as `0ZY` after `usync/result/lid` protocol success;
`0u7.A01`, `A3p.<init>` and `0hs.A0A` close the success-flag/typed-attribute chain.
The earlier Rust `lid@jid` assumption is **refuted**. Strict plain-address
validation and the result gate are now implemented and offline-tested.
Fresh live replies, complete contact state handling and refresh scheduling remain
**unknown/unverified**. The initial four-topic (now seven-topic)
[recipe](apk/recipes/2.26.37.71-contact-lid.json) preserves a compact deterministic
[build-specific bundle](apk/2.26.37.71-contact-lid/manifest.json); exact APK
re-export with `--check` matches. See [reasoning and boundaries](chat-actions.md#contact-lid-response-correction-2026-10-08).


Contact-type follow-up (2026-10-08, **confirmed APK 37.71**): `0u3.A02` writes
its positive account flag only for `N0z.A04=1` (`in`), while `0u5.BF6` handles
`invalid` separately from `in`/`out`. The prior JID-presence existence rule is
**refuted**. Rust preserves nullable recognized type and historical unknowns;
`contacts status` inspects offline without queuing. Full state-machine parity
and fresh live contact replies remain **unknown**. See
[consumer chain](chat-actions.md#contact-type-and-offline-status-2026-10-08).


## Live contact projection and primary wire LID (2026-10-08)

**Observed live:** one original existing-peer query returned `in` with `lid@val`
and successful LID protocol metadata during a clean 20-second receive-only
session. **Confirmed local bug:** no receive projection, plus rejection of
zero-component AD_JID encoding as if JidKind were an APK Java subtype. Both are
corrected; receive/startup projection is real-loop tested. Guarded offline repair
of the same reply recovered LID once with retained prior status and no action,
receipt, timestamp or outbox changes. No request repeated. **Unknown:** native
wire-to-Java factory, fresh post-fix replies and complete contact sync. The
[factory/caller negative trace](contact-primary-wire-evidence.txt) is scoped,
not native recovery. [Full reasoning and operator corrections](chat-actions.md#live-contact-projection-and-primary-wire-lid-2026-10-08).


Primary wire correlation audit (2026-10-08): **confirmed local defects** in
identity-only full-Jid comparison and key-directory per-user refusal comparison.
**Implemented/offline-tested** exact component matching across Plain/AD encodings,
with foreign namespace/device/agent and duplicate identity-only guards retained.
DF6.C8f typed dispatch, DNk.run primary branching and B7H.A06 address construction
are APK 37.71 evidence;
native wire object construction and fresh identity-only AD replies remain
**unknown/unverified**. Devices/successful bundle paths already have component
behavior; group/manual-verification paths are not blindly normalized. Existing
private PN inspection was already received; LID evidence absent. See
[reasoning and negative audit](peer-identity.md#primary-wire-address-correlation-2026-10-08).

Directory refusal attribution (2026-10-08, offline): `peer-keys --show` now
reports fixed local invalid-reply diagnoses and binds numeric refusals to one
unambiguous selected-user/root context. Foreign first-error codes and duplicate
rows cannot masquerade as selected-peer refusals. Original action history and
retry reservations remain intact. APK 37.71 DF6 identity-only error handling
supports contextual attribution; duplicate guards are local policy and fresh
live refusal validation remains unverified.
[Evidence and limits](peer-identity.md#directory-refusal-attribution-2026-10-08).

Identity-success wait consumer (2026-10-09, **confirmed static APK 37.71**):
the sole DEX Fco constructor caller is a feature-gated extensions/Flows path.
Its 30-second wait wakes callbacks after success or Exception timeout; those
callbacks reverify held keys. This does not establish incoming replay/session
repair. Eight new reproducible identity-wait topics preserve the caller, observer,
timed worker and verification chain; all other event consumers and complete
identity-only application remain unfinished. No account or network changed.
[Evidence and scope](cross-device-sync.md#identity-wait-caller-and-extensions-verification-2026-10-09).

Identity-success consumers (2026-10-09, **confirmed static APK 37.71**): the
full DEX scan preserves 22 BdO/BdR declarations/bodies. Only Fco and AL2 have
nonempty BdO; the second waiter is a 30-second business-profile UI task.
No scanned callback directly decrypts or retries messages; transitive helpers
remain unknown. Primary LID receive's sender_pn prerequisite is a local guard,
not a proven APK requirement, and remains the next receive-routing trace.
No account or network changed. [Evidence and limits](cross-device-sync.md#identity-success-consumer-inventory-2026-10-09).

Primary LID routing trace (2026-10-09, **confirmed static APK 37.71**): missing
sender_pn/username logs join nullable metadata construction rather than throwing.
The parsed username attribute is username; DFc's 13 field bindings are recovered.
Ordinary LID's type 18 selects the message's A05 route for sender-device lookup,
without consulting alias metadata. Rust's unconditional primary alias guard is
a local limitation; historical PN-marker stability and the raw stanza producer
remain next boundaries before a receive change. Six unrelated/special builder
write paths are recorded to avoid repeating them. No account or network changed.
[Evidence and limits](peer-identity.md#primary-lid-route-and-nullable-aliases-2026-10-09).

Primary LID receive (2026-10-09, implemented/offline): new primary messages use
actual LID without requiring a phone alias. Original authenticated capture proofs
retain historical PN rows for duplicate/event/read lookup; no row rename, ratchet
replay or alias adoption from redelivery. Real-crypto msg/pkmsg, reopen/MAC failure,
legacy capture ambiguity and poll-target fixtures cover the correction. Fresh
live alias-less interoperability and full APK storage parity remain unverified.
[Evidence and scope](peer-identity.md#primary-lid-receive-and-legacy-record-stability-2026-10-09).

- Primary LID legacy deduplication also retains consumed view-once markers using
  original route proof; alias-less redelivery cannot recreate plaintext or an event.
- Bounded receive smoke check authenticated and stopped cleanly after 45 seconds
  with reconnect/chat sends disabled; 15 decrypted records remained 15, with no
  fresh DM or alias-less live proof. Original action/receipt/outbox/event summaries
  were unchanged. The later consumed-marker fix has offline validation only.

Reaction receive correction (2026-10-09, implemented/offline): APK 37.71 C7G
requires target ID plus parseable remoteJid; malformed keys now remain opaque
with redacted key_validation diagnostics. Primary direct reactions retain proven
historical PN target keys while the reactor keeps its actual LID; consumed
markers and original records are preserved. Status likes now require the actual
status@broadcast singleton. Own/outgoing/group/companion reconciliation and fresh
live reaction validation remain incomplete. A 21-topic/26-method checksum bundle
preserves the trace and cached diff (0.028679 seconds).
[Evidence, negative findings and limits](reactions.md).
Validation: 727 workspace/all-feature tests, strict Clippy, warning-free Rustdoc,
24 default CLI checks and CLI/API reference checks passed. Retained-account
offline reprojection upgraded zero events; action/receipt/outbox/event and
peer-key summaries stayed unchanged, with two valid saved reaction keys. Capture
inspection now uses one timestamp snapshot and handles overflowing ages; a
subprocess regression proves malformed-key diagnostics remain visible. No new
chat/live request or identity/reset/retry authority was added.

Outgoing reaction attribution (2026-10-09, implemented/offline): a unique normal-
outbox wire reservation plus exact address or existing authenticated PN/LID pair
retains the original sent-message key. Reactor identity remains actual; attempts,
facts, progress and uncertainty are unchanged. Lookup now checks all pages and
uncommitted rows for duplicate wire IDs. Outgoing quote bodies require the actual
saved intent's chat, rather than just the caller's reference. Unknown targets,
legacy lab-only sends, full native alias/companion/group reconciliation and fresh
live reactions remain unfinished.
[Sources, negative findings and fixtures](reactions.md#outgoing-intent-attribution-and-quote-scope-2026-10-09).
Validation: 730 workspace/all-feature tests, strict Clippy, warning-free Rustdoc,
24 default CLI checks and regenerated CLI/API reference checks passed. Offline
reprojection upgraded zero events and kept retained summaries unchanged. A
45-second receive-only live smoke check authenticated and stopped cleanly, with
15 decrypted records remaining 15; no fresh reaction/DM established new target
interoperability. Original uncertain actions, outbox and receipts were preserved.

### 2026-10-09: poll parent-key validation and mapper

**Confirmed**, APK 2.26.37.71 Ban.A01 -> C7G.A00 and Clc.A02 -> BoQ.A00;
BWH/BSp/BZa descriptors; six exact parent-key field references in Ban.A01/AD7.
Seven selected source topics/methods retain operands/exceptions in a checksummed
69,090-byte bundle. Rust checks common supported structure and CLI reports
fixed redacted refusals independently of decoding. Native JID factories,
effective decryption property, AD7 writer, transitive parent/expiry/edit/quiz
policy remain unknown/unported. Own poll secret chat binding is a confirmed
local implementation gap and remains next work. No fresh live poll test.
[Detailed evidence and boundaries](poll-votes.md).

### 2026-10-09: own-poll chat provenance in projection/preparation

**Confirmed, implementation**, existing normal reservation chat now binds own
poll secret lookup and outgoing vote preparation. Primary PN/LID association
uses only exact components or saved authenticated mapping, shared with reaction
attribution. Actual AEAD fixtures verify original target/voter identity and
durable dedupe; missing/foreign/alias-only/companion and corrupt-state boundaries
remain explicit. Read-only binding inspection is exposed by lab-inbox.
APK context remains the prior 37.71 Ban/Clc/BoQ source bundle, not a newly
traced native parent store; full native parity and fresh live poll coverage are
unproven. [Details](poll-votes.md#saved-chat-attribution-2026-10-09).

**Observed**, bounded receive-only 45-second login/active/clean-stop; retained
peer/action/receipt/outbox/event summaries unchanged, fifteen decrypted records
remained fifteen. No fresh poll/vote attribution, replay or identity adoption.
All 734 workspace tests and strict/local feature checks passed.

### 2026-10-09: identity receipt selection, primary version and token refresh

**Confirmed**, APK 2.26.37.71 D4x/0Z5/8G8 row predicates, DNd case-9 cache
operation, 0Z6/0ZE concrete columns, 0Z4 scoped companion/status/coexistence
cleanup, 0Z7 persistent per-user version update and HxT conditional existing-token
refresh. The 23-topic/53-declaration checksummed source bundle is distinct from
the older sender-key/key-removal scope. Runtime counterpart/migration state,
0Z8/A06 cache internals, callback closure and privacy job lifecycle remain unknown.
**Confirmed local gap:** receipts::observe still uses full Jid equality for
normal-outbox fact attribution; the actual incoming receipt authority path must
be traced before fixing PN/LID namespace matching. No live mutation, deletion,
replay or native-parity claim. [Details](identity-receipts.md).

Direct receipt attribution (2026-10-09, **confirmed static / local correction**):
APK 37.71 `CmG`/`Cmk`/`DDH` keep message chat and receipt actor separate;
stale `ClU` attribution is corrected. Primary normal-outbox receipts now bind
the unique saved wire intent to the actual actor through exact PN/LID or an
existing authenticated association in either direction, within one transaction.
Actual actor, original IDs, progress, uncertainty and receipt-send reservations
are preserved. Legacy PN records page history; ambiguous/corrupt matches roll
back. Native canonicalizer gates, identity cleanup and fresh live PN/LID
receipt attribution remain unverified. [Evidence and boundaries](direct-receipts.md).

Receipt correction validation: 739 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and repository hygiene
pass. The 12-topic APK bundle reproduces exactly; cached self-diff is 0.017892
seconds. A 45-second receive-only session authenticated and stopped cleanly;
no fresh user message or qualifying receipt arrived. Saved peer keys, actions,
receipt ACKs, outbox and events remained unchanged. No chat was sent, no
uncertain action replayed. Fresh live namespace attribution remains pending.

Receipt canonicalizer (2026-10-09, **confirmed static / local correction**):
APK 37.71 `0Yo.A07` selects PN/LID by global migration state and exempts
PNH_CTWA chats; `0Xw.A0I` has explicit subtype filters. Bot conversion uses
a separate gated static dictionary. Effective state and complete application
remain unknown/unported. `0hs.A0C/A0M` defaults only for absent attributes;
Rust now refuses present nontext/empty/unknown receipt types without false
delivery mutation, covered through actual wire encode/decode and store reopen.
No account identity, map or send state is reset. [Evidence and limits](receipt-canonicalizer.md).

Canonicalizer validation: 740 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, CLI/API/reference checks and hygiene pass. The 24-method
source bundle reproduces exactly and cached self-diff takes 0.007559 seconds.
Offline projection upgraded zero events; saved keys, actions, receipt ACKs,
outbox, events and retained inspection stayed identical. No new live request
or saved-account mutation; malformed types and native runtime gates remain
unverified live.

Group participant receipts (2026-10-09, **confirmed static / local implementation**):
APK 37.71 `Cmk`/`DDH`/`ReceiptProcessingJob`/`Cmn` retain separate message key,
group chat and participant device through deferred receipt processing. Rust now
tracks ordinary group delivery/read/played against the original addressed-device
set, retaining actual PN/LID actors and companion indices. Frozen targets are a
local guard; historical out-of-band targets remain unknown. Comparison sends
preserve new targets before writing; `group receipts --request-id` inspects only
redacted counts offline. Normal group send preparation, native orphaned receipt
application/aggregation and fresh live participant attribution remain unfinished.
[Evidence and boundaries](group-receipts.md).

Group receipt validation: 749 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and formatting pass.
The 21-method / 15-file / 124,482-byte source bundle reproduces exactly; cached
self-comparison takes 0.019472 seconds. Offline inspection found seven saved
group reservations across twelve request files, with zero tracked participant
receipt devices. Projection upgraded zero events; saved peer keys, action
outcomes, receipt ACKs, outbox, events and normalized inspection were unchanged.
No historical receipt capture was replayed or new group send prepared.

**Observed live regression:** a 45-second receive-only session authenticated,
activated and stopped cleanly with chat sending and reconnects disabled.
Peer keys, action outcomes, receipt ACKs, empty outbox, events and retained
noncapture inspection stayed unchanged. Transport capture rows increased from
442 to 446; these are not four user messages. All seven saved group summaries
stayed unchanged with zero tracked participant receipt devices. No fresh group
receipt established live attribution, and no uncertain send was replayed.

Shared group fanout (2026-10-09, **confirmed local correction**): the comparison
CLI now uses library Snapshot/Prepared staging and atomic crypto reservation.
Fresh mode retains the original ratchet for concurrency checks instead of
allowing a late same-identity advance to be overwritten. Pins remain enforced;
identity-triggered own-key rotation archives the original generation and leaves
uncertain sends intact. Existing fanout-inspect adds nullable missing-bundle and
cached-session counts offline. Native query batching and normal group outbox
integration remain unfinished; no new group/status send is claimed.
[Evidence and port boundaries](group-fanout.md).

Shared fanout validation: 753 workspace/all-feature tests passed, plus the final
CLI inspection regression, strict Clippy, warning-free Rustdoc, default CLI,
API symbol/reference, formatting and hygiene checks (1,117 local links; 15 APK
bundles). Offline projection upgraded zero events. Saved peer keys, action
outcomes, receipt ACKs, outbox, events and normalized retained inspection were
unchanged. No server connection, group/status send, request replay or saved
account crypto mutation occurred in this chunk. Shared staging/rotation is
verified offline; live group/status validation and normal actor wiring remain.

Directory-key batches (2026-10-09, **confirmed static / local implementation**):
APK 37.71 `0y0.A00` confirms batched encrypt/key/user construction and optional
identity-reason/PQ/context branches; effective native producer flags and limits
remain unknown. Group/status lookup now covers all frozen bundle-required
devices in local 32-device pages, durably reserved before write, with complete
signed responses reused only for the same plan. Unknown/refused/invalid/legacy
unattributed outcomes block without resets. New group key-queries inspection
prints redacted progress offline. Normal group actor integration, optional native
fields, own inventory and fresh live large-group coverage remain unfinished.
[Evidence and boundaries](device-key-batches.md).

Directory-batch validation: 758 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and formatting pass.
Offline projection upgraded zero events; saved peer keys, action outcomes,
receipt ACKs, outbox, events and normalized retained inspection stayed unchanged.
Twelve existing request files were inspected offline: no new batch records,
eleven with neither batch nor historical key outcome, one with a preserved
legacy outcome and unknown frozen counts. No server request or message was sent.

The existing groups APK recipe now anchors get-pre-key-batch and maps its writer
to messaging.rs and device_key_batches.rs for future release diffs. Exact
re-export changed only targets/checksums metadata, not native method bodies;
all 44 files / 928,943 bytes / 43 selected methods reproduce. Cached self-diff
is 0.047877 seconds, not a claim of cross-release equivalence. Large-group live
lookup, native optional-field gates and normal group actor wiring remain pending.

Group routing snapshot (2026-10-09, **confirmed local correction**): planning
now reads membership/lists/primary authority together; shared fanout revalidates
the supplied plan and refuses changed routing rows inside the original send
transaction. Fresh/cached crypto and caller reservation roll back together,
with newer rows preserved. The native TEE node-token equality is not evidence
of a membership transaction. Normal group outbox actor wiring, own inventory
and fresh live group fanout remain unfinished.
[Evidence and limits](group-fanout.md#routing-snapshot-correction-2026-10-09).

Routing-guard validation: 760 workspace/all-feature tests passed, alongside
strict Clippy, warning-free Rustdoc, default CLI, API symbols/CLI reference,
formatting and repository hygiene (1,129 local links; 15 APK bundles). Offline
projection upgraded zero events. Saved peer keys, action outcomes, receipt ACKs,
outbox, events and normalized retained inspection stayed unchanged. No server
request, message transmission or saved-account crypto mutation occurred.

Normal group outbox (2026-10-09, **confirmed local implementation**): the actor
now resolves fresh membership, selected message-context device pages and all
missing signed key pages, then reserves shared fanout crypto and the original
normal intent together before write. Existing private JSON group/text/image/voice
uses outbox-send; group uploads bind the chat. Redacted outbox-group-preparations
shows original phase, fixed blockers, counts and scoped batch outcomes offline.
Cold PN/LID real-crypto tests, 130-user batches and injected four-stage write cuts
cover preparation and once-only reservations; simulated late ACK/participant
delivery settles without another attempt. Normal group live acceptance, broader
message kinds, answered-preparation resume, group peer replacement and full own
inventory remain unfinished.
[Evidence, invocation and boundaries](normal-group-outbox.md).

Validation for this chunk: 767 workspace/all-feature tests and strict Clippy,
Rustdoc, default CLI/API/reference/format checks passed. The upgraded saved
account authenticated and stopped cleanly after a 45-second receive-only window,
with chat sends and reconnect disabled. No live outgoing group was tested.

Signed group peer replacement (2026-10-09, **confirmed local implementation**):
normal group fanout now honors the explicit outgoing APK-authenticated policy for
verified signed remote bundles, archives old pins/ratchets and emits identity
events in the original send transaction, and rotates an exposed current chain
while leaving other exposed groups pending. Own identities remain pinned;
changed-primary fanout with old companion authority reports
companion_authority_changed. Interrupted/blocked preparations are not reset or
replayed. Cached-session refresh, companion reconciliation, preparation resume
and live group identity transition remain unfinished.

Signed group policy validation: 770 workspace/all-feature tests and strict
Clippy, Rustdoc, default CLI/API/reference/format and hygiene checks passed.
A 45-second receive-only check authenticated and stopped cleanly; peer keys,
actions, receipts, outbox and events stayed unchanged. Five transport captures
were added. No live outgoing group identity transition was tested.

Typed group quotes/reactions (2026-10-09, **confirmed local implementation**):
reply/react now accept an original --participant for incoming group targets and
--ours derives frozen own identity. Normal fanout carries the existing APK
quote/reaction encodings, checks retained parent attribution, and blocks late
reference changes without a wire attempt. Existing default IDs/queued records
are preserved. Private JSON quote, media quotes, incoming group/id collisions
and fresh live group rendering remain unfinished.

Typed group reference validation: 773 workspace/all-feature tests and strict
Clippy, Rustdoc, default CLI/API/reference/format and hygiene checks passed.
A 45-second receive-only check authenticated and stopped cleanly with sends and
reconnect disabled; peer keys, actions, receipts, outbox and events stayed
unchanged. Five transport captures were added. Group quote/reaction handset
rendering and the participant-aware incoming storage port remain unverified.

Participant-scoped group inbox (2026-10-09, **confirmed local implementation**):
new records distinguish original participant namespace/agent and wire ID, retain
original device routes, and preserve legacy records/event markers. Quotes use the
selected participant's body. `mark-read --participant` selects a group member;
ambiguous unqualified group/ID reads refuse atomically. Automatic reads remain
opt-in and refuse unsupported agent selection. Native SQLite-key parity and live
same-ID group collision behavior remain unknown. See [group inbox](group-inbox.md).

Group inbox validation: 775 workspace/all-feature tests passed, together with
strict Clippy, Rustdoc, default CLI, library-symbol, regenerated CLI reference
and formatting checks. Repository hygiene passed (15 APK research bundles,
5,815,284 bytes). A 45-second receive-only check authenticated and stopped
cleanly with sends/reconnect disabled; peer keys, actions, receipts, outbox and
events were unchanged. Original capture shapes were preserved; captures grew
from 461 to 466. No fresh group send, explicit read request or live same-ID
collision was performed. The prior incoming storage gap is now corrected
locally; native storage parity and fresh group handset rendering remain unknown.

Incoming group reaction authors (2026-10-09, **confirmed local correction**):
BZa participant now selects the original author; a third member's message no
longer becomes an outgoing target merely because fromMe is false. Matching
legacy/new parent routes retain actual devices; consumed bodies stay consumed.
Known own PN/LID selects outgoing targets; unsupported agent/malformed/foreign
group keys remain opaque. Inspection adds fixed target_validation labels. Parent
ambiguities refuse atomically; old event markers are preserved. Fresh live group
reactions and native own/hosted normalization remain unknown. See [reactions](reactions.md#incoming-group-original-author-2026-10-09).

Group reaction validation: 778 workspace/all-feature tests and strict Clippy,
Rustdoc, default CLI, API/library symbols, CLI-reference regeneration and format
checks passed. Hygiene checked the unchanged 15 APK evidence checksum bundles
(5,815,284 bytes). Two retained group keys report target_validation=valid; no
historical events were rewritten. A 45-second receive-only session authenticated
and stopped cleanly with sends/reconnect disabled. Peer keys, actions, receipts,
outbox and events stayed unchanged; original capture shapes were retained and
captures grew from 466 to 471. Fresh live group reaction targeting is unverified.
BoQ.A00's BO0 predicate, effective caller gates and contradictory fromMe/author
normalization remain unported; exact known own PN/LID matching is a local
projection contract. One temporary-store ownership test refusal passed isolated
and on full rerun without changing ownership guards; its cause remains unknown.

Native group reaction predicate/branch (2026-10-09, **confirmed static/local**):
the new 18-topic APK 37.71 bundle closes BO0's own PN/LID comparison and
user/server/type equality. Ordinary groups consult it only for false/default
fromMe; true/present-author stays incoming, correcting our prior unconditional
own classification. Nonzero device authors no longer fold into UserJid equality.
Nullable-author fallback, actor producer authority, zero-device text and
authenticated own-envelope context remain unported. No identity/alias adoption
or historical-marker rewrite is introduced. See [group mapper](group-reaction-mapper.md).

Ordinary remote-key correction (**confirmed APK 37.71 BoQ.A00**, `00fe..0106`):
plain PN/LID/group remote keys may differ from the envelope group. The update
and parent selection now remain in that authenticated envelope group. The
previous exact-match refusal was our local limitation; status/device/unported
remote kinds stay separate. Native nullable-author and own-envelope source
context remain unported; no fresh live group-targeting result is claimed.

**Observed receive validation (2026-10-09):** two bounded 45-second windows
authenticated and stopped cleanly; the second ran the final ordinary-remote
correction. Both disabled chat sends and automatic reconnect. Original captured
records were retained; capture rows grew from 471 to 481. Peer keys, actions,
receipts, outbox and durable events were unchanged. Two retained group reaction
keys still inspected as target_validation=valid. These are retained-key and
receive-runtime checks, not fresh live group reaction/own-envelope proof.

Native mapper verification: 778 workspace/all-feature tests, strict Clippy,
Rustdoc, default CLI, API/library-symbol checks, CLI-reference regeneration and
formatting passed. Final API edits passed five OpenAPI tests and rebuilt the CLI.
The 18-topic APK export/check reproduces 23 files / 86,053 bytes; hygiene validates
16 checksum bundles / 5,901,337 bytes. Cached verified self-diff took 0.008 seconds;
new-release extraction and re-anchoring speed remains unmeasured.

Group envelope direction (2026-10-09, **confirmed static/local**): the APK 37.71
CnN/CgC/Cih/Bp2/Cna producer chain now establishes the ordinary network group's
selected sender and own-envelope flag. Sender-aware reactions preserve payload
direction for a known own sender; remote true/absent-or-empty-author references
fall back to the authenticated sender user. Inspection verifies the saved original
route against its selector. Existing projection markers, consumed bodies, parent
routes, outbox records and trust admission remain preserved. This supersedes the
ordinary own-envelope/source-producer gap above; full own-device synchronization,
hosted/status references, special chat normalization and fresh live own-device
reaction behavior remain unverified/unported. See [envelope direction](group-envelope-own.md).

**Confirmed verification:** 779 workspace/all-feature tests passed (652 API,
4 property, 9 crash, 50 CLI, 1 API-symbol and 63 xtask tests), along with strict
all-target/all-feature Clippy, Rustdoc with warnings denied, default CLI tests,
API/library checks, CLI-reference regeneration and formatting. The final embedded
API/build check also passed. The 24-topic export and revalidation agree on
29 files / 127,370 bytes. Hygiene validates 17 APK checksum bundles totaling
6,028,707 bytes.

**Observed receive check (2026-10-09):** one 45-second window authenticated and
stopped cleanly with chat sends and automatic reconnect disabled. Original
captures were retained; capture rows grew from 481 to 487. Peer keys, actions,
receipts, outbox and durable events stayed unchanged. Both retained group keys
still inspected as target_validation=valid using their original sender routes.
No historical markers were repaired or removed. This checks the receive runtime
and retained references; it does not establish fresh live own-device/group
reaction handling or human handset rendering.

Group keep/unkeep receive attribution (2026-10-09, **confirmed static/local**):
APK 37.71 D9L.A01 maps BV3.key through BoQ, just like ordinary group reactions.
Reception now resolves the original author independently of the keeper, uses
authenticated own-envelope context and retains matching parent devices. Incoming
updates preserve consumed bodies, outbox facts and existing event markers.
`lab-inbox --inspect` exposes fixed group keep target_validation labels using
original saved routes. Direct keep projection retains its existing behavior.
Normal group keep sending, native downstream authorization/expiry enforcement,
and fresh live group keep/handset rendering remain unimplemented or unverified.
See [group keep](group-keep.md).

**Confirmed verification:** 780 workspace/all-feature tests passed (653 API,
4 property, 9 crash, 50 CLI, 1 API-symbol and 63 xtask tests), together with
strict all-target/all-feature Clippy, Rustdoc with warnings denied, default CLI,
API/library-symbol checks, CLI-reference regeneration and formatting. Export
and checksum revalidation reproduce six topics / 11 files / 84,246 bytes.
Repository hygiene validates 18 APK bundles totaling 6,112,953 bytes.

**Observed receive validation (2026-10-09):** one bounded 45-second window
authenticated and stopped cleanly with chat sends and automatic reconnect
disabled. Original captures were retained; capture rows grew from 487 to 492.
Peer keys, actions, receipts, outbox and durable events stayed unchanged. Two
retained group reaction targets remain valid; no retained group keep payload
was available to validate live attribution. No fresh group keep/unkeep, group
send or human handset rendering is claimed. Native keep timestamp ordering,
authorization, expiry enforcement and outgoing group keep remain open gaps.

Outgoing keep key research (2026-10-09, **confirmed static**): APK 37.71
Bai.AD7 serializes the original parent key through CSZ.A01. The new 22-topic
bundle traces explicit id/fromMe/participant setters, gated parent/edit/partial-ID
lookup and retained author-device selection. A supplied author may be omitted
when the gated lookup misses; remote namespace mapping and effective gate values
remain unknown. No Rust send behavior changed. Normal group keep sending,
native permission/expiry checks and fresh live keep/unkeep remain unfinished.
[Evidence and implementation boundaries](group-keep-send-key.md).

**Confirmed research validation:** exact re-export/check reproduces 22 topics /
27 files / 75,556 bytes; checksum-verified self-diff is unchanged. All 63 xtask,
50 experimental CLI and one API-library symbol tests passed, including CLI
reference regeneration. Hygiene validates 19 APK bundles / 6,188,509 bytes.
No account connection, group draft, message send or private state change was
performed. This verifies the research export, not outgoing keep implementation.

Normal group keep/unkeep (2026-10-09, **confirmed static/local implementation**):
APK 37.71 original-key, namespace, creation-clock and stanza traces now ground
a separate group codec and typed keep --participant/--ours/--undo/--dry-run.
Normal fanout dispatch preserves the original accepted timestamp and original
intent reservations. Shared quote/reaction/keep parent selection now reads one
snapshot and revalidates inside the crypto/outbox reservation transaction;
changed references roll back with reference_changed. Consumed bodies stay
consumed and keeps/reactions do not parse parent text previews. Native
permission/expiry enforcement, adjusted-clock synchronization, complete parent
normalization, full own-device sync and fresh group ACK/rendering remain open.
[Evidence and limits](group-keep-outbox.md).

**Confirmed validation:** 784 workspace/all-feature tests (656 API, 4 property,
9 crash, 51 CLI, one API-library symbol, 63 xtask), strict Clippy, warning-free
Rustdoc, default CLI/API checks, regenerated CLI reference, formatting and build
passed. APK re-export/check reproduces 33 topics / 38 files / 259,783 bytes;
cached checksum-verified self-diff is unchanged in 0.017053 seconds, not a
new-release extraction benchmark. Hygiene checks 1,237 local links and 20 APK
bundles / 6,448,292 bytes. Offline keep dry-runs enqueue nothing; missing group
participant refuses. Saved account facts and 492 retained captures were unchanged
before live validation.

**Observed receive validation:** a single completed 45-second window
authenticated and stopped cleanly with sends and automatic reconnect disabled.
Peer keys, actions, receipts, outbox, events and retained original capture shapes
stayed unchanged; captures grew from 492 to 497. Two retained group reaction
references remain valid; no group keep payload was available. No new group draft,
keep/unkeep transmission, group ACK or handset rendering was tested. An earlier
local ownership refusal resulted from overlapping offline inspection and receive
orchestration; its logs were preserved and inspection completion verified before
this window. No server retry, account reset or ownership bypass was added.

2026-10-09 — **Confirmed local/static:** incoming keep events now preserve
optional payload milliseconds and their own envelope ID, independently of the
original target and stanza time. Literal legacy-journal records remain readable
and retain their shape; reopening preserves group update metadata without
republication. APK 37.71 native ordering prefers newer envelope time, then newer
payload time, then the smaller action ID. Permission/expiry enforcement and
fresh live keep reception remain unverified. [Evidence and limits](keep-timestamps.md).

Keep metadata verification: 786 workspace/all-feature tests, strict Clippy,
Rustdoc, default CLI, API-symbol checks, CLI reference and formatting passed.
One 45-second receive-only window authenticated and stopped cleanly with sends
and reconnect disabled. Captures grew 497→502 with originals retained; keys,
actions, receipts, outbox and event summaries were unchanged. No group keep
payload was available, so fresh keep semantics remain unverified.

2026-10-09 — **Confirmed static:** identity-change token refresh now has traced
outbound issue/send storage, bucket expiry formula, request destination/time,
callbacks and explicit completion/cancellation/retry branches. Incoming token
bytes are not its outbound state. Native 4xx IQ refusal suppresses job retry;
base terminal dispatch, active properties and canonical counterpart gates remain
unknown. Rust refresh sending/storage is still unported; account state was
untouched. [Evidence and port boundary](identity-token-refresh.md).

Token-refresh research validation: 26 topics / 31 files / 104,087 bytes reproduce
from the comparison APK; checksum/self-diff, 63 xtask tests, API symbols, CLI
reference, formatting and repository hygiene passed. Source changes are a
comment only. No live refresh or identity replacement was attempted.

2026-10-09 — **Confirmed static/local:** traced the normal server-acceptance
producer of outbound token scheduled timestamps, its raw message exclusions,
namespace transfer and separate call producers. Scheduled timestamp state is
not proof of token server acceptance. New apk-smali --signatures and recipe
include_method_signatures expose complete DEX descriptors, distinguishing two
DNf constructors with identical shorties while preserving legacy defaults.
Token scheduling/refresh runtime remains unported; account state was untouched.
[Evidence, invocation and boundaries](token-issue.md).

Token-issue/full-signature validation: 788 workspace/all-feature tests, strict
Clippy, Rustdoc, default CLI/API/reference/format and build checks passed. New
14-topic evidence and the prior default-mode bundle reproduce; self-diff took
0.022072 seconds. No account operation or live token/identity request was made.

2026-10-09 — **Confirmed local:** manual peer verification now refuses a saved
own LID with no user before writing metadata. A failing-before regression and
reopen checks cover mark/unmark refusals, unchanged pins/sessions/events and
valid remote decisions. The initial zero-device bypass hypothesis was refuted:
valid textual zero forms already normalize; no parser change was needed.
This is our local prerequisite, not a server refusal. Native malformed-login
parity and fresh live identity transitions remain unknown. No real account
verification decision or network request was made.
[Evidence and reproduction](own-verification.md).

Own-prerequisite verification: 790 workspace/all-feature tests (660 API, four
property, nine crash, 51 CLI, one API symbol and 65 xtask), strict Clippy,
warning-free Rustdoc, default CLI/API checks, regenerated CLI reference,
formatting and build passed. Hygiene checked 1,283 local links and 23 APK
bundles / 6,988,623 bytes. Validation used synthetic stores only.

2026-10-09 — **Confirmed static:** the two remaining outbound token timestamp
writers restore imported chat-transfer/companion-history sender state, separately
from received token bytes. The companion key-parse failure can still reach token
restoration; no valid-key admission shortcut is inferred. The hidden descriptor
id-array helper explains the field decoder's honest 59/58 refusal. Manual tag and
presence mapping is preserved with the helper evidence. History source authority,
active gates and Rust import/refresh remain unported. Account state was untouched.
[Evidence and limits](token-import.md).

Token-import research validation: seven topics / 12 files / 412,652 bytes
reproduce from the APK. Cached self-diff reports seven topics and four metadata
artifacts unchanged in 0.021659 seconds; this is not a cross-release benchmark.
All 65 xtask tests, CLI-reference regeneration, API/library-symbol check,
formatting and diff hygiene passed. Repository hygiene checked 1,287 local links
and 24 APK bundles / 7,401,275 bytes. Runtime source changes are comments only;
no account operation or live token/history request was made.

2026-10-09 — **Confirmed local/static:** authenticated primary replacements
now persist the APK-style per-user device version atomically with their archive
and follow-ups, exposed in PeerPinState through existing offline inspection.
Missing old state stays null; first tracked change starts at 1. PN/LID namespaces
stay separate, own/companion exclusions remain, and receipt reservations are
preserved. Native receipt version use and fresh live increments remain unverified.
[Evidence and limits](primary-device-version.md).

Primary-version verification: 792 workspace/all-feature tests (662 API, four
property, nine crash, 51 CLI, one API symbol and 65 xtask), strict Clippy,
warning-free Rustdoc, default CLI/API checks, regenerated CLI reference,
formatting and build passed. Hygiene checked 1,296 local links and 24 APK
bundles / 7,401,275 bytes. Real encrypted direct/group transition fixtures
exercise the receive integration; isolated tests cover both archive sources,
rollback, reopening, namespace separation, signed-long overflow and legacy JSON.

**Observed compatibility:** one completed 45-second receive-only window
authenticated and stopped cleanly, with chat sends and automatic reconnect
disabled. Saved peer facts (apart from the new absent-version null field),
actions, receipts, outbox and event summaries were unchanged. Captures grew
502→508 with all original shapes retained; two retained group reaction targets
remain valid. No fresh identity transition occurred, so live version increments
remain unverified. The original 23-topic / 28-file / 323,094-byte APK receipt
bundle reproduced byte-for-byte.

**Observed inspection clarification:** direct `peer-identity` checks with the
previous and rebuilt binaries match after omitting only the new null counter.
Two inspected primary namespaces have absent historical versions. `peer-keys
--show` exposes directory match diagnostics, not PeerPinState; its original
compatibility comparison remains valid but cannot prove counter output. The API
and invocation description were corrected to name `peer-identity` only.

2026-10-09 — **Confirmed static:** native pending device-receipt rows capture
nullable send-time primary versions; existing-row timestamp updates retain them.
The ordinary sync/resend path compares prior/current values with nullable equality,
plus a gated absent-to-1 exception. The raw hosted branch has additional gates;
0x6ebd does not surround all ordinary comparisons. Historical Rust sends lack
this snapshot and remain unknown. Rust resend/snapshot runtime remains unported;
no saved reservation or account was changed. [Evidence and next input](receipt-version.md).

Receipt-version research validation: 14 topics / 19 files / 135,279 bytes
reproduce exactly from the comparison APK. Cached self-diff reports all topics
and four metadata artifacts unchanged in 0.022983 seconds, not cross-release
equivalence. All 65 xtask tests, CLI-reference regeneration, API/library-symbol
check, formatting and diff hygiene passed. Repository hygiene checked 1,304
local links and 25 APK bundles / 7,536,554 bytes. Runtime changes are comments
only; no account operation, live resend or receipt reservation change occurred.

2026-10-09 — **Confirmed local/static:** normal direct/group original-send
reservations now capture nullable per-user primary versions after authenticated
replacement hooks, atomically with their original state. All group plan devices
are included. Offline outbox-versions exposes hashes/counts and prior/current
values; absent historical snapshots remain unknown. Missing JSON version members
and malformed state refuse without private echoes. Native resend/receipt-cache
runtime remains unported; no new live send was authorized or performed.
[Evidence and limits](send-version-snapshot.md).

Send-version validation: all 795 workspace/all-feature tests, strict Clippy,
warning-denying Rustdoc, 26 default CLI checks, API/library-symbol check,
CLI-reference regeneration, formatting and experimental CLI build passed.
Hygiene checked 1,313 local links and 25 APK bundles / 7,536,554 bytes.
Offline retained-profile CLI checks left store inspection and queued intents
unchanged; invalid IDs and unknown intents refused without input echoes.
There were no retained group preparations to inspect; historical missing-state
coverage is from fixtures. New send capture has no live validation in this chunk.

2026-10-09 — **Confirmed static/local audit:** native sync/resend now has a
22-topic trace of its distinct encrypt/key_fetch request, existing remote
registration encoding, reply/worker completion, original receipt-device
subtraction and post-version target mutation. An empty/errored-device result
can complete true; callback completion is not per-device usability. Expiry
is strictly past, not equal-time rejection. Rust lacks key_fetch and original
per-device receipt membership; user counts cannot reproduce removeAll.
Current nullable user snapshots are preserved; no live worker/resend, account
operation or retry-guard relaxation was added. [Evidence and next port inputs](sync-resend.md).

Sync/resend research validation: all 22 source topics / 27 files / 272,318
bytes reproduced from the exact comparison APK. Cached self-diff verified
22 unchanged topics and four unchanged metadata artifacts in 0.031055 seconds;
this is plumbing verification, not cross-release equivalence. All 65 xtask
tests, CLI-reference regeneration, API/library-symbol check and formatting
passed. Repository hygiene checked 1,324 local links and 26 APK bundles /
7,808,872 bytes. Runtime changes are comments only; no account operation,
key-fetch query or live resend was performed.

2026-10-09 — **Confirmed local/static:** original send snapshots now preserve
exact per-user device components; outbox-versions exposes only component/hash
rows. Historical count-only rows remain unknown and untouched. Group receipts
use the full original plan, validate envelope distributions as a subset and
refuse contradictions atomically. Current fanout sends SKDMs to every target;
omitted-distribution coverage is fixture/native compatibility, not a reproduced
current live failure. Native mutable receipt lifecycle/resend remains unfinished.
[Evidence and limits](send-version-snapshot.md).

Exact-device validation: all 798 workspace/all-feature tests passed (668 API,
4 property, 9 crash, 51 CLI, 1 API-symbol and 65 xtask), alongside strict
Clippy, warning-denying Rustdoc, 26 default CLI checks, CLI-reference/API checks,
formatting and experimental CLI build. Hygiene checked 1,329 local links and
26 APK bundles / 7,808,872 bytes. Native-shaped subset/contradiction fixtures
cover planned companions, unplanned devices, immutable send state, reopening
and legacy wire-only fallback. Old count-only snapshots remain readable;
malformed device IDs refuse with redacted diagnostics.
Offline retained-profile CLI checks left store inspection and queued intents
unchanged. No retained group preparations existed to inspect; no new live
message or native pending receipt cache is claimed from these fixtures.

2026-10-09 — **Confirmed static:** receipt application now has an eight-topic
trace through 14e/Cmn and the DDH device/time/status/key getters. Cmn.A00
selects CNn.A00 for a status predicate and separately 15A.A07 for other
statuses. CNn/CYV row policy and complete clocks/mapping remain untraced;
no mutable receipt ledger or runtime status promotion is claimed.
[Writer boundary and next inputs](receipt-apply.md).

Receipt-application validation: eight source topics / 13 files / 112,789 bytes
reproduced exactly from the pinned comparison APK. Cached self-diff verified
eight unchanged topics and four unchanged metadata artifacts. Repository
hygiene checked 1,336 local links and 27 APK bundles / 7,921,661 bytes; diff
hygiene passed. This follow-up changes research/docs only; the preceding
cdca7ad runtime chunk passed all 798 workspace tests and strict lint/doc/CLI
checks. No new runtime implementation or live receipt application is claimed.

2026-10-09 — **Confirmed static:** CNn is a receipt value factory, not a row
writer. Its settings branch downgrades played/read (8/13) to delivery (5);
1hg.A03 admits six statuses. The downstream 0Z4 dispatch separates message
classes and reaches 0Z5 timestamp storage. Complete 15A admission, clock and
namespace/cleanup policy remain unknown; no runtime or live operation changed.
[Evidence and reproducible recipe](receipt-factory.md).

Receipt-factory validation: seven source topics / 12 files / 137,766 bytes
reproduce exactly from APK 2.26.37.71. Cached self-diff reports seven unchanged
topics and four unchanged metadata artifacts; this is reproducibility, not
cross-release equivalence. Repository hygiene checked 1,340 local links and
28 APK bundles / 8,059,427 bytes; diff hygiene passed. Research/docs only; no
fresh Rust runtime test or live account operation is claimed for this chunk.

2026-10-09 — **Confirmed static:** receipt timestamp branches now distinguish
direct zero defaults, strict aggregated participant parsing and adjusted-clock
defaults for successful-message participant receipts. Wire seconds become
milliseconds. The cache retains earliest positive time without rewriting the
nullable primary version; ordinary concrete device-receipt columns and conditional
device-preserving namespace mapping are traced. Clock calibration, complete
message admission and mutable identity cleanup remain unknown/unported.
[Evidence and comparison recipe](receipt-clock.md); no runtime/account operation changed.

Receipt-clock validation: 18 source topics / 23 files / 82,330 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified 18 unchanged
topics and four unchanged metadata artifacts; this does not establish parity
between different releases. Repository hygiene checked 1,350 local links and
29 APK bundles / 8,141,757 bytes. Diff hygiene passed. Changes are research/docs
only; no new runtime tests, live receipt or account changes are claimed.

2026-10-09 — **Confirmed local/static:** signed const/4 annotations now show
negative decimal values; 0Z8's trimToSize input is -1 (clear), not 255.
Primary selected-row invalidation and companion broad cache clear are traced,
including optional map/callback handling. Incoming SQL fallback selects timestamps
without primary versions; general load/eviction and complete mutable receipt
runtime remain unported. [Evidence and recipe](receipt-cache.md).

Receipt-cache validation: all 66 xtask tests, strict xtask Clippy, rebuilt
research CLI, CLI-reference regeneration, OpenAPI/library-symbol check, formatting
and diff hygiene passed. Eight source topics / 13 files / 57,611 bytes reproduce
exactly with the corrected renderer; cached self-diff verifies eight unchanged
topics and four metadata artifacts. Hygiene checked 1,359 local links and
30 APK bundles / 8,199,368 bytes. No transport code changed; previous 798-test
runtime validation is not claimed as a fresh run. No live account operation ran.

2026-10-09 — **Confirmed static:** the general receipt loader retains device
row membership without a positive timestamp filter; it does not load primary
versions. Cache install rechecks rather than overwrites, and general row mapping
uses 0Yo.A03 rather than the incoming update's A07. Pending row membership,
delivery facts and immutable original targets remain distinct. Mutable runtime
receipt_user/status admission and alias authority are still unported.
[Evidence and next inputs](receipt-load.md); no account or live operation changed.

Receipt-load validation: seven source topics / 12 files / 58,000 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified seven unchanged
topics and four metadata artifacts; this does not prove cross-release parity.
Repository hygiene checked 1,368 local links and 31 APK bundles / 8,257,368
bytes; diff hygiene passed. Research/docs only; no fresh Rust or live tests
are claimed beyond the preceding 66-test tooling chunk.

2026-10-09 — **Confirmed static:** per-user receipt timestamps retain earliest
positive delivery/read/played independently. User rows can be created by read
or played without a delivery timestamp; identity cleanup's existence query
is not equivalent to a Delivered fact check. Writer rejects other statuses,
while complete upstream 15A admission and namespace authority remain unknown.
[Evidence and reproducible recipe](receipt-user.md); mutable Rust receipt runtime remains
unported, with no account or live operation in this chunk.

Receipt-user validation: eight source topics / 13 files / 128,135 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified eight unchanged
topics and four metadata artifacts, not cross-release equivalence. Hygiene
checked 1,381 local links and 32 APK bundles / 8,385,503 bytes; diff hygiene
passed. Research/docs only; no new runtime tests or live operation is claimed.

2026-10-09 — **Confirmed static:** 15A admission rejects nonpositive times
and splits message/user advancement from device timestamp handling. Native
nonadvancing comparison includes equal ranks; its 22-entry ordering was recovered
from the exact APK payload. Lower/equal events can still reach device storage.
State-only status translation, manager exclusions and conditional message
classification are traced with explicit remaining authority/lifecycle unknowns.
[Evidence and next port inputs](receipt-admission.md); no runtime/live changes in this chunk.

Receipt-admission validation: 13 source topics / 18 files / 121,518 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified 13 unchanged
topics and four metadata artifacts; raw 96-byte/22-entry status-order evidence
independently matches classes2.dex at its recorded offset. Repository hygiene
checked 1,393 local links and 33 APK bundles / 8,507,021 bytes; diff hygiene
passed. Research/docs only; no fresh runtime tests or live account operation
is claimed.

2026-10-09 — **Implemented/offline:** APK 37.71 0Z5.A0D/DPo/0Z6.A0L pending
nullable-version rows and Cmk.A03/0Gv.A01/7hf.A01 earliest-positive device times
now have bounded Rust storage/receipt/inspection paths. Duplicate fact admission
no longer suppresses an earlier supported root timestamp. Exact original-device
provenance and atomic rollback are required; historical rows are never backfilled.
[Implementation, sources and remaining unknowns](device-receipt-rows.md).
No new raw APK bundle or live account operation in this chunk.

Receipt-row runtime validation: 802 workspace/all-feature tests passed, plus
strict Clippy, warning-denying Rustdoc, formatting, default CLI/API symbols,
CLI-reference regeneration, experimental CLI build and hygiene (1,405 local
links, 33 bundles / 8,507,021 bytes). Retained-state offline inspection was
unchanged; zero network commands and zero retained group preparation samples.
A separate local owner-drop shared-descriptor regression confirms teardown
release without widening active-owner admission. Historical refusal holder
attribution remains unknown; see [verification](../design/verification.md#temporary-store-ownership-refusal-observed-2026-10-09).

2026-10-09 — **Confirmed static:** APK 37.71 13v constructor/A02 and 8UM44
close the A03 revoke exclusion producer; 15A's earlier device-only pool remains
separate. 0V9.A00 and Cjb.A02 establish deletion-path marking/copying of A0l,
not its full persistence/producer semantics. Raw 1gB.close decrement bytes at
classes2.dex 0x12a6da independently confirm signed -1; 22b/22s renderer output
is corrected with boundary tests and preserved predecessor generations.
[Evidence and unknowns](receipt-manager.md). No Rust revoke/cleanup or live claim.

Receipt-manager validation: 67 xtask tests and strict xtask Clippy passed,
plus rebuilt CLI, CLI reference/API symbols, formatting and hygiene. Eight source
topics / 13 files / 301,312 bytes reproduce exactly; self-diff verifies eight
unchanged topics/four artifacts. Independent raw DEX read matches signed decrement
at 0x12a6da. Hygiene: 1,416 links, 34 bundles / 8,808,333 bytes. Tooling/research
only; no new account/shared-runtime/live test is claimed.

2026-10-09 — **Confirmed static / implemented offline:** 08a delegates BO0/BO1
to 08i; BO0 compares stored A0I PN/A0H LID, BO1 extracts the device user first.
A01/A02 field producers and nullable self_lid initialization are retained. Rust
primary-change hooks now share authenticated login/retained own-LID lookup;
conflict refusal is explicit local policy, not a native/server requirement.
[Evidence, inspector and remaining unknowns](own-identity.md).

Own-LID validation: all 807 workspace/all-feature tests, strict Clippy,
warning-denying Rustdoc, default CLI, reference/API-symbol and formatting checks
passed. Exact bundle reproduction and cached self-diff passed. One 45-second
receive-only session authenticated and stopped cleanly, with sends/reconnects
disabled; peer/actions/receipts/outbox/events remained unchanged. Eight additional
inbound checkpoints remain unclassified; no live own-primary transition is
claimed. See the linked own-identity evidence for counts and limits.

2026-10-09 — **Confirmed static:** revoke receipt callbacks are now traced:
8SJ discriminator 6 removes the selected message-row user cache entry under a
monitor; DLn discriminator 6 delegates selected-row device-cache invalidation.
The user cache constructor uses plain 0D3; base entryRemoved is empty. Existing
transaction-close/rollback and optional device-cache callback evidence remains
separate. Mutable cleanup, revoke projection and resend are unported.
[Worker evidence and boundaries](receipt-callbacks.md).

Callback validation: the five-topic / 10-file / 70,998-byte bundle reproduces
exactly; cached self-diff verifies five unchanged topics and four metadata
artifacts, not cross-release protocol equivalence. CLI-reference regeneration,
OpenAPI/library-symbol checks, formatting and diff hygiene passed. Repository
hygiene checked 1,436 local links and 36 bundles / 8,929,026 bytes. This final
chunk changes static research/docs only: it does not claim a new workspace
runtime or live cleanup test. The earlier own-LID runtime validation is separate.
No retained account was read or changed during this callback research.

2026-10-09 — Companion receipt lifecycle implementation (offline verified):
authenticated changed companion keys now clear only pending exact-device mutable
membership for ordinary direct/group original intents. A later positive supported
receipt reinserts with current nullable user version; original at_send/targets
and held send/receipt reservations are preserved. Primary/counterpart/status/
coexistence cleanup and final resend remain unfinished.
[APK evidence, diagnostics and boundary](companion-receipt-cleanup.md).

Companion lifecycle validation: all 810 workspace/all-feature tests, strict
Clippy, warning-denying Rustdoc, default CLI, CLI reference/API symbols,
formatting and experimental CLI build passed. Hygiene checked 1,447 links and
36 bundles / 8,929,026 bytes. One receive-only session authenticated, then ended
early with unknown cause and no retry. Saved peer/actions/receipts/outbox/events
were unchanged; six new inbound checkpoints remain controls/unknowns rather
than user replies. No qualifying live companion transition occurred. listen now
preserves terminal state before shutdown and prints fixed redacted cause labels;
this diagnostic fix is locally tested, not yet live observed. Primary cleanup,
namespace migration and resend remain unfinished.

2026-10-09 — **Confirmed static:** bulk user receipt restoration uses a distinct
8G8.A05 writer from chat transfer/history sync. Any cache-slot improvement enables
SQL with every positive supplied timestamp; this is not the one-column live
writer or a universal SQL minimum policy. Descriptor field 40 and nested fields
1..6, actual caller branches and unique user-row index are retained. Exact table
xref negatives do not prove absence of generic normal-send seeding. Primary
cleanup still needs actual user-row existence/admission, not synthesized facts.
[Evidence and remaining port inputs](user-receipt-import.md).

Validation: nine source topics / 14 files / 204,931 bytes reproduce exactly from
the pinned APK. Cached self-diff verifies nine unchanged topics and four metadata
artifacts, not another release's behavior. Self-remap selects the correct original
class/method for seven topics; one is ambiguous and one unanchored, so no complete
proposed recipe is emitted. The committed compact report keeps actual candidate
selections and the full report digest; unrelated per-anchor hit lists stay local.
CLI-reference regeneration, API/library-symbol validation, formatting and diff
hygiene passed. Repository hygiene checked 1,463 local links and 37 bundles /
9,133,957 bytes. No Rust runtime changed, no fresh workspace/runtime test is
claimed, and no account or live operation ran. The prior cfda410 functional
chunk's 810-test validation remains separate.

2026-10-09 — **Confirmed static:** the four remaining exact receipt_user
reference roles are classified: backup_changes trigger construction, migration
error classification, whole-table deletion and receipt table metadata. The
trigger INSERT destinations are backup_changes, not receipt_user. Generic
consumers/normal-send user-row seeding remain unknown; primary cleanup still
requires actual user-row existence. No runtime or account behavior changed.
[Complete methods and reasoning](user-receipt-reference-roles.md).

Validation: ten source topics / 15 files / 138,158 bytes reproduce exactly from
the pinned 37.71 APK; cached self-diff verifies ten unchanged topics and four
metadata artifacts. CLI-reference regeneration, API/library-symbol validation,
formatting and whitespace checks passed. No fresh workspace/runtime test or
live operation is claimed; the prior functional validation remains separate.
Repository hygiene checked 1,471 local links and 38 bundles / 9,272,115 bytes.

2026-10-09 — **Confirmed static:** explicit receipt table-descriptor readers
are incremental backup export/restore and JSON metadata serialization. Descriptor
fields now have actual serializer labels, including optional lookup_field.
Restore contains a dynamic table INSERT; its specific receipt input continuity
and complete validation remain untraced. Normal-send user-row seeding and
primary cleanup are still unfinished. Exact scans, scoped source, signatures,
catch tables and complete-dump digests are retained outside the checksum bundle.
[Consumer evidence and limits](user-receipt-reference-roles.md#descriptor-consumers-follow-up).
No Rust/account/network behavior changed.

Consumer follow-up validation: CLI-reference regeneration, OpenAPI/library-symbol
checks, formatting and whitespace checks passed. Hygiene checked 1,475 local
links and the unchanged 38 checksum bundles / 9,272,115 bytes. The new excerpts
are separately attributed, not counted as checksum-bundle coverage. No fresh
workspace/runtime or live test is claimed.

2026-10-09 — **Confirmed static:** receipt ownership now has an exact caller
boundary: true message-key from-me inputs short circuit to ordinary admission
before the A0y helper/foreign-input property. False-key helper logic is
(type 64 + stored-own Bps.A00 + clear flag); later pool gates still apply.
Property 35516 has true/true baked defaults, not a known effective account value.
Ten explicit flag writers are retained with scoped source and full-dump digests;
constructors/receive/copy contexts do not prove universal origin semantics.
[Source, branch arithmetic and remaining prerequisites](receipt-ownership.md).
No Rust user-row projection, primary cleanup, account or network change is claimed.

Ownership validation: eight topics / 13 files / 59,820 bytes reproduce exactly
from APK 37.71. Cached self-diff verifies eight unchanged topics and four metadata
artifacts, not cross-release equivalence. CLI-reference regeneration,
OpenAPI/library-symbol checks, formatting, whitespace and hygiene passed.
Hygiene checked 1,486 local links and 39 bundles / 9,331,935 bytes. Flag-writer
excerpts are separately attributed outside that bundle. No fresh workspace
runtime test or live account operation is claimed.

2026-10-09 — **Confirmed static:** native message status is A08, with direct
getter/reset and a downgrade-logging setter that still assigns. Two subtype
setters always select 6. Five direct writer methods and 111 physical superclass
setter calls are retained. The blank receipt message entry connects to device
pending insertion; failure writes local state 20, not a recognized server reason.
Conditional coexistence rows are separate from receipt_user membership.
[Exact methods, provenance and remaining initialization inputs](message-status.md).
Ordinary outgoing initialization, full admission and primary cleanup remain
unfinished. No Rust runtime, account or network behavior changed.

Message-status validation: eleven topics / 16 files / 79,539 bytes reproduce
exactly from APK 37.71. Cached self-diff verifies eleven unchanged topics and
four metadata artifacts, not cross-release semantic parity. CLI-reference
regeneration, OpenAPI/library-symbol checks, formatting, whitespace and hygiene
passed. Hygiene checked 1,499 local links and 40 bundles / 9,411,474 bytes.
Caller/writer inventories and the no-matching ordinary override body result
remain separately attributed. No fresh workspace/runtime or live test is claimed.
