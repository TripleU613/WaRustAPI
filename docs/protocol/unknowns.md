# Unknowns and blockers (G1/G2)

Ordered by what blocks the first decisive result (docs/design/README.md section 0).

PQ remains a normal-client integration gap: the unmodified locked backend has no KEM
bundle/session construction. Selected pqkey replies now fail explicitly instead
of silently deriving classic state. Original-byte signature diagnostics are implemented.
The [native JNI trace](native-kem.md) resolves ML-KEM dispatch, byte lengths and
the unused Java seed. [Selected native ML-KEM-1024 vectors](native-kem-vectors.md)
now match NIST. [Rust primitive support](rust-kem.md) is implemented; full
primitive/format coverage and application hybrid integration remain open.
[Initial hybrid KDF composition](hybrid-kdf.md) and
[incoming paired KEM metadata](prekey-kem-wire.md) are implemented. Native
version-4 output is 0x44, while the backend emits 0x43.
[Low-level native wire encoders](signal-wire.md) now implement 0x44/MAC and KEM
fields. [Staged hybrid session records and encrypted exchange](hybrid-session.md)
now work offline. [Held version-4 shared sending](held-hybrid-send.md) is integrated;
[Fresh signed bundle staging](hybrid-bundle.md) is implemented offline;
[Normal primary-peer hybrid outbox adoption](hybrid-outbox.md) now works offline.
Same-key classic migration, auth/attestation and companion admission,
[Staged incoming hybrid bootstrap](hybrid-incoming.md) now authenticates offline and
[the normal direct/group inboxes use it](kem-prekey-inventory.md) with a local
KEM row lookup and atomic consumption. KEM prekey generation, ID allocation,
signing and the inner `pq_list`/`pq_last_resort_key` upload shape are untraced,
so no KEM key is produced or uploaded; own-device/companion KEM admission is
unported. [Primary identity refresh](hybrid-identity-refresh.md) now derives
hybrid sessions from verified `pqkey` replies; companion and attestation-bearing
PQ refresh remain deferred. [Device-key batches and group fan-out](hybrid-fanout.md)
now derive hybrid sessions from verified PQ rows. Whether the server returns
`pqkey` without the native `pqsupport` request attribute is unknown. See
[the boundary and source audit](pq-bundle-boundary.md).

Own-device ordinary text edits are now projected with loaded parent kind,
terminal and sender-time ordering checks. Peer edits still require the effective
native integer property 2983; a baked default or guessed 15-minute window is not
proof. Own image/video/GIF/document captions now preserve original attachment references
and enforce nonempty ordinary edits. Generic local video references still lack
native subtype and the 64 KiB inspector bound remains stricter than native Unicode
caps. Other media/caption, gated poll/event, future-proof wrapper, orphan and expiry
lifecycles remain open. See [the receive evidence](message-edit.md).

1. **Android registration protocol** (`registration.*`): payload encoding,
   URL assembly, required prerequisites and their generation timing,
   verification steps, cooldown semantics, returned credential material.
   *Progress 2026-09-17:* host `v.whatsapp.net`, path fragments and the
   parameter vocabulary are client-observed (ledger `registration.host`,
   `registration.vocabulary`). A separately pinned newer build (2.26.37.70)
   statically confirms the full base/path assembly, URL-encoded form body and
   JSON response vocabulary, but the exact 2.26.36.75 artifact and a live
   capture are still needed before changing the active profile.
   `RegistrationService::begin` still returns `Blocked(ProfileIncomplete)`
   because response meanings and the complete flow are unobserved.
   The comparison build's common identifier fields and public key-bundle byte
   encodings are now mapped and implemented offline. The opt-in
   [persisted lab factory](registration-lab.md) generates keys and signs using
   the upstream Signal convention; accepted ID ranges, Android signing
   semantics and mandatory-field timing remain unobserved. Default registration
   is still blocked; mock success is not evidence of server acceptance.
2. **Transport endpoint and layer** (`transport.endpoint`): port, TCP vs
   WebSocket, whether TLS wraps Noise on Android. *Progress:* hosts
   `g.whatsapp.net`, `g-fallback`, `e1`–`e16.whatsapp.net` are client-observed.
   The 2.26.37.70 comparison statically selects 443/5222 with a port-80
   fallback. Its built-in Java path is not WebSocket: primary/fallback
   gateways use raw TCP on 443/5222, while numbered edges use TLS on 443 and
   raw TCP on 5222. Port 80 wraps raw TCP with bounded HTTP/1.1 chunked streams
   using `POST /chat` and host `c.whatsapp.net`. Active remote configuration
   and pinned-profile equivalence remain open.
2b. **Handshake pattern negotiation** (`noise.client_patterns`): the client
   contains classic XX/IK **and nine ML-KEM512 hybrid** Noise patterns. If the
   server requires a hybrid pattern for new primary registrations, `snow`
   cannot be the backend (no such patterns) and a pure-Rust ML-KEM512
   implementation would need review. The newer comparison's selector map is
   known and its compiled defaults select classic mode (`NONE`, variant
   `IKKEM`), but its remotely configured effective mode/variant must be
   observed at runtime.
3. **Login payload** (`login.client_payload`): primary-device markers,
   version fields, startup requests, resume behaviour.
4. **Noise record sizes** (`noise.record_limit`) to accept or reject `snow`.
   The 2.26.37.70 comparison frame writer uses a three-byte big-endian
   length and rejects bodies at or above 16,777,216 bytes. This framing
   bound is much larger than `snow`'s 65,535-byte limit; it does not establish
   the largest record emitted or accepted by the server.
5. ~~**Android token dictionary**~~ *Resolved 2026-09-17 for content and
   order*: `xtask apk-tokens` recovers the ordered dictionary from the APK
   bytecode and it is identical to the web v3 list. Still open: the
   dictionary **version byte** (`binary.tokens_version`) and the structural
   tag bytes remain `ReferenceOnly` for the pinned profile. Version byte 3 is
   statically observed only in the separate 2.26.37.70 comparison.
6. **Signal backend** (ADR 0005): licence decision or clean-room plan.
7. **TLS provider** (ADR 0003) for registration/media HTTPS.
8. **Wire message-ID format** (`messaging.id_format`).

## How to resolve

Evidence must come from the pinned APK (static analysis in the lab) and from
dated captures with a controlled test number. Do not copy web-profile
constants into `Observed` rows. Record each observation with date, APK
version, and whether the server accepted the reference client that day.

## APK access in cloud sessions (2026-10-13)

The documented comparison APK path (`~/Downloads/WhatsApp.apk`) is on
the owner's machine. A Claude Code cloud container has no copy, and no raw APK
is committed (by design). In such a session only the committed bundles under
`docs/protocol/apk/` and `docs/protocol/rea/` are available, so new static
traces (new methods, callers, builders) cannot be made; only retained evidence
can be reused. Work in that state is limited to evidence already retained.
Supplying the APK to a session (or running the session on the owner's machine)
removes this blocker; no other source was used to substitute for it.

**No private account state either (checked 2026-10-13).** The container has no
ignored `lab/` directory, no account store and no profile, so no live
validation (sends, receipts, group actions, registration) is possible from it,
independent of authorization. Work in such a session is limited to offline
implementation, tests, documentation and analysis of retained evidence. What
remains of the parity goal after that is listed with its blocker in
[feature-parity](../feature-parity.md): APK traces (KEM prekey upload,
`pqsupport`, revoke/edit `edit` attribute, document/video `mediatype`, group
poll stanza, privacy setters, app-state index/version for mute/pin/archive,
retry-answer policy, calls, communities, profile picture), live checks on the
owner's account, and owner decisions (HTTP routes that expose addresses,
content or sends; ban appeal).

## Outgoing revoke and edit (2026-10-13)

`MessageContent::Edit` and `MessageContent::Revoke` exist in the model, but the
outbox has no sender for them (`begin_selected` treats them as unsupported).
Retained evidence covers the protobuf side: `D9M.AD7` builds the revoke
`ProtocolMessage` with type REVOKE (`C4F.A0T`) or MESSAGE_UNSCHEDULE (`C4F.A0O`)
and the target key ([revoke writer](apk/2.26.37.71-sender-revoke/revoke-writer.txt)),
`D9B.A00` writes an edit's replacement Message to protocol field 14 and the
sender timestamp to field 15, with ordinary/future-proof wrapper selection
gated by property 8784 ([image-caption-edit.md](image-caption-edit.md)), and
`DEy.A00` parses received stanza `edit` values 1/2/3/7/8
([sender-revoke.md](sender-revoke.md)). The outgoing request bundle
`1Dz.A0H` carries a caller-supplied `edit` string beside `addressing_mode` and
`counter_abuse_token` ([retained](apk/2.26.37.71-core/group-mode-outgoing-bundle.txt)
`00c0..00ca`). **Not retained:** the producers that choose that `edit` value
(and any companion attributes) for a sender revoke or message edit, the effective value of property 8784, and the native send-side
time window. Receive-side parsing does not establish what a sender must emit,
so no sender is implemented. Next step: trace `SendE2EMessageJob`/`0y0.A00`
for the outgoing `edit` attribute in 2.26.37.71.

## Outgoing disappearing context (2026-10-13)

**Negative finding (code search):** no outgoing content message this client
builds (text, reply, media, poll, reaction, group text) sets
`ContextInfo.expiration` (25). The receive path reads it (`BWx`, confirmed),
and timer changes themselves are sent (`ProtocolMessage` type 3). So in a chat
with a disappearing timer, our own messages are not marked to disappear on the
recipient's side. **Not retained:** the native sender-side producer that copies
the chat's timer (and any `ephemeralSettingTimestamp`) into outgoing contexts,
and whether it applies to every message kind. Next step: trace where 2.26.37.71
`SendE2EMessageJob` or the message builders set `BWx.expiration` for outgoing
messages before adding it.

## Document and video sending (2026-10-13)

**What retained 2.26.37.71 evidence covers:** the document and video schemas
(`BZB`, video descriptor), their key domains (`WhatsApp Document Keys`,
`WhatsApp Video Keys`) and the `1qB` media-type table, whose constructor takes
`("document", "DOC", key, …)`
([document evidence](document-media-evidence.txt)). The first argument is the
`/mms/<segment>/` upload path for image and ptt, and those two were confirmed
live, so `/mms/document/` is a strong inference.

**Not retained (blocks sending):** the stanza's `enc mediatype` value and any
other attributes for document and video sends. The image and ptt values came
from `LX/9QH` and live sends. No committed bundle maps document or video, and a
code search of `docs/protocol/apk` and `rea` finds no such table. Thumbnail
generation and upload (`thumbnailDirectPath`, `jpegThumbnail`) are also untraced.
Sending is therefore not implemented, rather than guessing `mediatype` from
general protocol knowledge. Next step: trace the `9QH` equivalent in 2.26.37.71
for all media types, then reuse the existing upload, encryption and outbox paths.

## ACK and confirmation encodings since 2026-10-11 (live check pending)

Three 2026-10-11 commits changed the wire form of stanzas this client had
already sent live, from APK 2.26.37.71 evidence: `5c45218` (receipt ACKs send a
zero-device AD JID as a plain user pair), `fb11db3` (the same for notification
ACKs) and `1b88f56` (delivery confirmations to a LID primary are AD-encoded).
Each is **confirmed static**, but no live window since has recorded how the
server treated the new forms: the only later live session (2026-10-11 evening,
sender revoke) logged nothing about confirmations. A server that silently
ignored them would show only as the same messages and receipts arriving again,
classed `Duplicate`, at every login. Check that in the next live receive window
(`listen` incoming lines: `confirmed=true` followed by no redelivery) before
treating these encodings as verified. Found by the
[live-shape regression sweep](replay-safety-audit.md#live-shape-regression-sweep-2026-10-13).

## Comparison token dictionary: double-byte page 0 index 1

**Resolved 2026-10-07, test completed 2026-10-13.** The checked 37.71
`1Rl.<clinit>` evaluator replaced the heuristic 37.70 fixture; the four refuted
page slot-1 entries are listed in [token-extraction.md](token-extraction.md).
`every_checked_entry_decodes_from_its_wire_index_and_encodes_back` now decodes
all 235 single and 1,024 double entries from index-constructed wire and encodes
each back to the same bytes. The historical note follows.

Observed 2026-09-20 on a block-list fetch reply: an `item` attribute name
decoded as `B3b` where the official parser (`LX/8nc.A02`, 2.26.37.71) reads
`active`, and the web-md-v3 fixture places `active` at double-byte page 0
index 1. The comparison dictionary (`TokenDictionary::comparison_2_26_37_70`)
therefore has at least one wrong double-byte entry; encoding still works
because unknown names fall back to raw strings. Re-derive the double-byte
pages from the APK and add a decode/encode round-trip test for every entry.
