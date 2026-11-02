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
signing and the content of each `pq_list`/`pq_last_resort_key` element are
untraced (the element shape itself is now read from retained evidence:
[kem-prekey-inventory](kem-prekey-inventory.md#upload-element-shape-2026-10-13-retained-evidence)),
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

## Retained-evidence sweep of the "needs the APK" items (2026-10-13)

Before calling these blocked again, the retained bundles (`docs/protocol/apk`,
`rea`) were searched for each one. **Gained:** the KEM upload element shape
([kem-prekey-inventory](kem-prekey-inventory.md#upload-element-shape-2026-10-13-retained-evidence)),
the PQ predicate `Can.A04(jid)` behind `pqsupport` and its siblings
([pq-bundle-boundary](pq-bundle-boundary.md#the-pq-predicate-cana04-2026-10-13-retained-evidence-only)),
`SendE2EMessageJob.editVersion` as the sender's edit value and the receivers
that distinguish edit values 1 and 2 (below), the correction that `1Dz.A0H` is
the server-ACK hand-off (below), the timer state `0Zi` and its per-message
send-path source (below), and the job's retry bound of four
([retry-answers](retry-answers.md)). An independent verification pass the same
day corrected several of these first readings; the sections say which. **Searched, nothing retained:** the bodies of
`Can.A01`..`A04`; who sets `editVersion` per operation, and `Civ`'s stanza writer;
any `ContextInfo` (`BWx`) field write (past `Cn6`); the incoming retry-receipt handler;
`9QH` (document/video `mediatype`); a profile-picture set (only the
`w:profile:picture` token and received `picture` attributes in usync
parsing); a group poll stanza producer (only `PollCreation` descriptors); and
mute/pin/archive app-state actions (no matches). These need the APK.

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
([sender-revoke.md](sender-revoke.md)). **Corrected 2026-10-13 (confirmed
static):** `1Dz.A0H`, earlier called the outgoing request bundle, is the
server-ACK hand-off, not a send path. Its only retained caller is `1Dz.A10`
([forwarder](apk/2.26.37.71-message-ack/ack-message-forwarder.txt), logged
`ConnectionThreadRequestsImpl/message-received-by-server`, `00d8..00de`),
called by the ACK wire parser `1Qy.A02`
([parser](apk/2.26.37.71-message-ack/ack-wire-parser.txt) `0572..0578`), which
reads `edit` from the incoming `<ack>` (`0094..00a2`). So the `edit` string in
`1Dz.A0H` ([retained](apk/2.26.37.71-core/group-mode-outgoing-bundle.txt)
`00c0..00ca`) is what the server's ACK of our message echoes; it says nothing
about what a sender emits. **Not retained:** the producers that choose that `edit` value
(and any companion attributes) for a sender revoke or message edit, the effective value of property 8784, and the native send-side
time window. Receive-side parsing does not establish what a sender must emit,
so no sender is implemented.

**Sender side, from retained `SendE2EMessageJob` bundles (2026-10-13):**

- **Confirmed static:** the job has an integer `editVersion` field. It is part
  of `CSP(jid, id, editVersion, participant)`
  ([send-job-run](apk/2.26.37.71-send-job-state/send-job-run.txt)
  `113c..114e`), the key of the static map `A1P` that the send-message
  runnable checks as "jobAlreadyScheduled" (`DNu` `2820..283e`). It is a
  scheduled-job dedupe key (corrected from "in-flight key"), so an edit or
  revoke is scheduled separately from the original send under the same message
  ID. In `A06`
  ([group-send-job-hash-check](apk/2.26.37.71-groups/group-send-job-hash-check.txt)),
  the job builds a `CSu` expected-ACK key (`0ba8..0bd4`), the type the ACK
  parser `1Qy.A02` fills (`A02` from, `A01` participant, `A03` recipient, `A08`
  id, `A06` class, `A09` type, `A07` edit), with
  `CSu.A07 = editVersion != 0 ? String.valueOf(editVersion) : null`. A
  nonzero value (`0b98..0b9c`) is converted at `0ba0..0ba6`; a zero one
  branches to `0d44`, sets `v6 = 0` and returns to `0ba8`, and `A07` is stored
  from `v6` at `0bca..0bce`. (An independent verification pass upgraded this
  from "inferred".) `editVersion` is very probably also passed to the request
  object `Civ.<init>` (`0be6..0c28`), but the `move/from16` operands are
  elided there, so that is **inferred**.
- **Inferred:** the stanza's `edit` attribute is `editVersion` in decimal text,
  absent when zero. The server's ACK echoes `edit`, and the job matches the
  ACK on exactly that value.
- **Not retained:** the `SendE2EMessageJob` constructor body (which argument
  becomes `editVersion`) and `Civ`'s stanza writer. **Superseded in part the
  same day** by the message-side trace below, which answers "which value for
  which operation" up to those two links.

**Who sets the edit value (2026-10-13, second retained pass).** The send
runnable does not invent the value; it reads it from the message:

- **Confirmed static:** `DNu.run` builds the jobAlreadyScheduled key
  `CSP(jid, id, v15, participant)` with `v15 = iget LX/178;->A00:I` of the
  message being sent
  ([group-send-runnable](apk/2.26.37.71-groups/group-send-runnable.txt)
  `27fc..2800`, `2820..2826`), the same `CSP` position the job fills with its
  own `editVersion` (`send-job-run` `113c..114e`). `DNu` reads `178.A00` again
  at `2eb8..2ebc` while assembling the constructor arguments, but the
  `move/from16` operands are elided in the retained render.
- **Inferred (strong):** the job's `editVersion` is the message's `178.A00`;
  otherwise the dedupe lookup could never match the job it guards. This also
  assumes `CSP`'s `equals`/`hashCode` include the int field; `CSP` is not
  retained.
- **Confirmed static, retained writers of `178.A00`** (every one found):

  | Writer | Message | Value |
  | --- | --- | --- |
  | `Bpt.<init>(1gc, J)` ([sender-constructor](apk/2.26.37.71-sender-revoke-window/sender-constructor.txt) `000e..0012`; the copy constructor calls it first) | kind 15, sender revoke | **7** |
  | `Bps.<init>(1gc, J)` ([special-message-constructor](apk/2.26.37.71-receipt-ownership/special-message-constructor.txt) `0010..0014`) | kind 64, admin revoke | **8** |
  | `DNh.run` case at `17fe` ([call-start-worker](apk/2.26.37.71-token-issue/call-start-worker.txt) `18e6..18ec`) | copy of a kind-0x42 poll with a new name; the same case writes `poll_edit_snapshot`/`previous_poll_name` (`1938..196a`), so a **poll edit** (meaning inferred from those strings) | **1** |
  | `11x.A01` keep action ([keep-action](apk/2.26.37.71-group-keep-outbox/keep-action.txt) `0320..0334`) | `BnF` keep add-on (kind 0x44) | **7** when the *target* message's key is `fromMe` and the keep flag `v20` is false, else **0** |
  | `SyncDeviceAndResendMessageJob.A00` ([job-original-message](apk/2.26.37.71-sync-resend/job-original-message.txt) `00a2..00a8`) | a `BnB` message re-fetched for resend | 0 |
  | `D7E.Byt` ([message-flag-copy](apk/2.26.37.71-receipt-ownership/message-flag-copy.txt) `00a6..00ae`) | copy constructor | copies |

  `v10` in the keep action is the target message's key (`016c..0170`; every
  path to `0320` passes that write). `v20` is also stored as `BnF.A01`
  (`0018`, `030a..030e`) and as the target's `178.A03` (`001c..0020`), which
  the exception handler reverts to `v20 ^ 1` (`0358..0360`), so `v20` is the
  target's new kept state. **Inferred:** `v20` true means KEEP: receivers throw "edit version and keeptype as kept are both set"
  when the envelope edit version is nonzero on KEEP_FOR_ALL
  ([keep-receiver](apk/2.26.37.71-own-device-keep/keep-receiver.txt)
  `0574..05ca`; `C2L.forNumber` maps 1 to `A01`, [keep-values](apk/2.26.37.71-control-enum/keep-values.txt)),
  so a KEEP never carries 7 and only an undo of one's own message does. The
  action's own telemetry agrees: `JgH.A04` is set to 1 when `v20` is true and
  2 when false (`007e..0128`), matching KEEP_FOR_ALL 1 / UNDO 2 (also
  inferred). The one retained receiver check (`D9L.A01`) does not look at the
  value on UNDO; other receivers are not retained.
- **Written, not proven final (review correction, same day):** right after
  each write, `Bpt` (`0016..001c`) and `Bps` (`0018..001e`) call the
  unretained `178.A0G(0)`, and the `Bpt` copy constructor then calls the
  unretained `Bpt.A0p(178)` (`0052`). The 7 and 8 are confirmed writes; that
  they are still the values when the job reads `A00` assumes neither call
  rewrites it.
- **Inferred:** the stanza `edit` attribute is `editVersion` in decimal,
  absent when 0 (the ACK match above). So, chaining the two inferences: a
  sender revoke sends `edit="7"`, an admin revoke `edit="8"`, a poll edit
  `edit="1"`, an undo-keep of one's own message `edit="7"`, a keep none.
- **Not found in retained writers:** the value for a text or caption edit
  (receivers mark `edit == 1` as an edit; `D9B.CF4` sets the received
  replacement's `CoT.A00 = 1`), for pin-in-chat (receivers take the pin path
  for 2), and for unschedule.
- **Revoke stanza ID (unknown, 2026-10-13):** the retained revoke key writer
  `D9M.A00` ([revoke-key-writer](apk/2.26.37.71-sender-revoke/revoke-key-writer.txt))
  fills the protobuf key's `id` from `1ii.A01` (the target ID) and, when that
  is null, from the revoke message's *own* key ID (`0078..0084`); remoteJid
  goes through `CSZ.A00` (`0026..003c`) and `fromMe` comes from the key
  (`008c..0090`), forced false for a `Bpt` with a participant when
  `0hd.A02(participant)` is true (`0056..0064`; otherwise it comes from the
  key, `005e` to `008c`).
  Both retained `Bpt` constructions are receive-side (`Bua.CF4`, an orphan
  writer), so the send-side caller that picks the revoke's own key is not
  retained: whether a native revoke goes out under a new stanza ID or the
  original message's ID is **unknown**. The `CSP` dedupe key carrying
  `editVersion` would be needed only if the same ID can be scheduled twice,
  which fits reuse but does not prove it. A revoke sender therefore needs
  three facts: this ID choice, the `edit` attribute (inferred 7) and the
  sender's time window.
- **Divergence, recorded, not changed:** this client's group `keep --undo`
  sends no `edit` attribute, where the native sender very probably sends
  `edit="7"` for one's own message. The retained receiver check ignores it on UNDO (above), so
  receipt is not expected to differ; changing what a live account sends waits
  for `Civ`'s stanza writer to confirm the attribute.
- **What the receive side constrains (confirmed static):** `DEy.A00`
  validates the value (8, 7, 3, 2, 1, or absent/empty as 0; anything else
  throws, [edit-parser](apk/2.26.37.71-sender-revoke/edit-parser.txt)
  `002a..005e`). The sender-revoke builder `Bua.CF4` accepts REVOKE or
  UNSCHEDULE with any value except 8
  ([sender-builder](apk/2.26.37.71-sender-revoke/sender-builder.txt)
  `0018..0024`), and the admin builder `Bub` requires 8 (`0010..001c`).
  Receivers **do** distinguish other values: the message factory `1hR.A00`
  sets flag bit `0x8` when `edit == 1`
  ([message-factory](apk/2.26.37.71-sender-revoke/message-factory.txt)
  `033a..0348`), and the keep/pin-in-chat receiver `D9L.A01` takes its pin path
  only when `edit == 2`
  ([keep-receiver](apk/2.26.37.71-own-device-keep/keep-receiver.txt)
  `00f6..0104`). **Corrected:** an earlier version of this note said peers
  never distinguish 1/2/3/7 and that the value mattered only to the server;
  that is refuted. **Inferred from the receivers:** an admin revoke carries
  `edit="8"` and a pin-in-chat message `edit="2"`. Which value a sender's own
  revoke and a message edit use (1, 3 or 7) still needs the sender's producer.

Next step (APK): trace the `SendE2EMessageJob` constructor callers that set
`editVersion`, and `Civ`'s consumer that writes the stanza.

**Pin-in-chat sending (2026-10-13, retained bundles only):** the envelope field
is confirmed — `xtask retained-proto-fields` decodes BZc exactly and gives
`pinInChatMessage` = 63, type `BVD`. Sending still needs, from the APK, the
`BVD.dynamicMethod` descriptor (the receiver `D9L.A01` reads `key_`, `type_`
and `senderTimestampMs_`, so the names are known, not their numbers), the
C2S pin/unpin type enum values, and the `edit` value the sender's producer sets
(receivers take the pin path only for `edit == 2`, above). None of these is
retained; [what the retained files can and cannot decode](apk-analysis.md#field-numbers-from-retained-bundles-retained-proto-fields-2026-10-13).

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

**Retained-evidence search (2026-10-13, corrected the same day by an
independent verification pass):** the timer state is `LX/0Zi`
(`expiration:I`, `ephemeralSettingTimestamp:J`, `disappearingMessagesInitiator:I`,
`afterReadDuration:I`; constructor `(IIJI)`). For a chat it comes from
`0Fs.A0J(UserJid)` (**confirmed static**, e.g.
[timer-builder](apk/2.26.37.71-own-device-timer/timer-builder.txt) `D9s`
`0092..00a2`), read by the timer builder `D9s`, the preprocessed-message store
`0zC` ([own-preprocessed-store](apk/2.26.37.71-own-device-echo/own-preprocessed-store.txt)
`023c`, `0476`, `077a`, `07ee`), the ACK background worker `DMZ`
([server-background-worker](apk/2.26.37.71-message-ack/server-background-worker.txt)
`0a0e`), chat-transfer import `CnI`
([chat-transfer-import](apk/2.26.37.71-token-import/chat-transfer-import.txt)
`0440`), and further retained readers (`ODa`, `DNy`, `DNk`, `0zO`, the `0Fs`
chat-cache loader). **On the send path, confirmed static:** the send-message
runnable `DNu` takes a per-message `0Zi` from `B35.A01(FMessage)`
([history-identity-worker](apk/2.26.37.71-token-import/history-identity-worker.txt)
`2a1a..2a20`) and passes it into `Cn6.<init>(BOG, BOB, 0Zi, …)` (`2a82..2a88`),
then `Ayn.A1C(…, Cn6)` (`2a90..2a96`), right after "building message".
**Negative:** no retained bundle accesses any `LX/BWx` field, so the step from
`Cn6` to `ContextInfo.expiration` (25) is not retained. **The next APK trace is
`Cn6`/`Ayn.A1C` and `B35.A01`** (corrected from "a `0Fs.A0J` caller").

**What is established without that trace:** native senders do set it.
**Observed** live (recorded in
[messaging-transport](messaging-transport.md#quoted-replies-comparison-2263771)):
the handset's messages in a chat with a disappearing timer, including a quoted
reply, carried the chat's timer in `contextInfo` field 25, "as already used for
disappearing-timer context". Our receive path reads that field and keeps the
message's expiry (`transport::quotes`). So without it our messages do not
disappear on the recipient's side, a visible privacy difference. **Why it is
still not sent:** this client keeps no per-chat timer state at all
([own-device-timer](own-device-timer.md): chat and default state are not
reproduced), and the native rule for which timer applies (the chat setting
versus the default timer, and `0Zi.ephemeralSettingTimestamp` ordering
against a peer's change) is untraced. The prerequisite is local per-chat timer
state built from sent and received timer changes, with that precedence traced;
only then can field 25 be set on outgoing content.

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

## Wrong OTP re-entry and `guess_wait` (2026-10-13)

A `/v2/register` `mismatch` answer leaves our attempt blocked with no way back
to code entry. 2.26.37.71 `KotlinRegistrationBridge.A0D` maps `mismatch` to
`HD6.A0A` and reads `guess_wait` as a string into `HdD.A0C` (**confirmed
static**). The consumer of `HD6.A0A`, whether the app returns to code entry, and
the unit and format of `guess_wait` are **not traced**. Trace those before
building a re-entry command; it must honour `guess_wait`.
[Audit](registration-safety-audit.md#plausible-not-verified).

## Where the APK handles an undecodable stanza (2026-10-13)

When a decrypted frame does not decode, `listen` now captures it as
`inbound-undecodable`, keeps receiving, and stops only for a terminal stanza
whose name it can read (or a close right after an unreadable one). That is
**local policy**. Which frame catches the native reader's decode exception, and
whether the native connection survives it, is **not traced**: the retained
`124.A01`/`124.A02` evidence covers the readers, not their callers' exception
handling. [Audit](replay-safety-audit.md#network-input-panic-audit-2026-10-13).

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

## PSA notification ACK ends the session (observed 2026-11-01)

A freshly registered account's first `listen` logged in, received two
`notification type=psa` stanzas (one with `participant`), and ended about 2 s
later on `stream:error` containing only `ack class=notification type=psa id=…`
(`lab-inbox --inspect`: `terminal_ack_to_present=false`). The 2026-10-08
terminal stream:error had the same notification-ACK shape. **Resolved 2026-11-01:** this client sent **no** ACK for `psa` at all; the
stream:error (about 15 s after the PSA arrived, not 2 s) echoes the ACK the
server was waiting for. **Confirmed, 2.26.37.71:** `Fbs` registers `psa`;
its handler `17o` (`PSANotificationHandler`, log strings
`PSANotificationHandler/...`) extends `0yO`
(`BaseNotificationHandler/handleAndAckNotification`), which ACKs every
notification after `A09`, or later via `AAR.A00` when `A09` deferred with
`A07(true)`. `17o.A09` inspects children only when `from` is
`0@s.whatsapp.net`; a `surfaces` child with feature `0xdd0` off, or
`psa_wakeup` with `0x1068` off, is ACKed by `17o.A03` with extra attribute
`error="487"`. `transport::notifications::psa_ack` now ACKs every psa
(id/to/class/type/participant per `Cp2.A05`), adding `error="487"` for those
two children because this client implements neither feature. **Inferred:**
the `campaign` (status PSA) and other children get the plain ACK; their
feature-gated branches were not fully mapped. Live confirmation of a session
that stays up past a PSA is recorded in STATUS.md.

## Other unacknowledged notification types (2026-11-01)

The PSA fix generalized: this client only ACKed notification types it had a
dedicated handler for, while 2.26.37.71 `BaseNotificationHandler` (`0yO`)
ACKs every notification its subclasses dispatch.

* **`w:gp2` (fixed).** `Fbq` registers it; handler `0yv` extends `0yO`, defers
  in 32 places with `A07(true)` and never adds an `error` attribute. Only
  `groups_dirty` was ACKed before; **observed:** an unACKed `w:gp2` ended a
  session with `stream:error` echoing `ack class=notification type=w:gp2`.
  `groups::notification_ack` now ACKs every other `w:gp2` (plain base ACK);
  membership changes are still not applied (capture retained).
* **`disappearing_mode` (fixed).** `11O` (a `0yO`) reads the notification's
  `disappearing_mode` child (`duration`, `t`); **inferred** that it is the
  type's handler from that shape. `notifications::disappearing_mode_ack`
  sends the plain base ACK; the timer is not applied.
* **`business` (fixed 2026-11-01).** `BusinessNotificationHandler` is `17Y`
  (found by its log strings, not the bare `business` literal, which is why the
  first search missed it); it extends `0yO`, does not defer and adds no
  attribute, so `notifications::business_ack` sends the plain base ACK.
* **`mex` (fixed 2026-11-01).** The most frequent type in our captures (82 on
  `l`/`q`). The MEX notification router `0yP` (logs `mexnotificationrouter/…`)
  extends `0yO`, does not defer and adds no attribute, and still reaches the
  base ACK when no handler/operation matches; `notifications::mex_ack` sends
  the plain base ACK. Operations are not applied (captures kept).
* **The routers.** The newer `EGu` registry holds only `w:gp2`, `location` and
  `psa` (`Fbq`/`Fbr`/`Fbs`). The classic type router was not located
  (`0u7.C8f`, which names many types, is the usync reply parser). For an
  unrouted stanza `1SP.run` logs `StanzaReceiver/receiveStanza not handled`
  and continues: it reads an integer stanza attribute (`0hs.A05`) and only when
  it is at least 11 looks up a per-type handler in `A8h`'s map (`1Qp.A01` →
  `0BW.CGQ`, the unhandled-stanza error path); otherwise it returns to the loop
  with nothing sent. **Confirmed negative:** the official client does not
  generically ACK unhandled types, so neither do we.

## Retry-receipt answers (2026-11-01)

Answered in `listen` only for the APK's no-fetch branch
([retry-answers](retry-answers.md)). Still unknown or unported: whether the
resent `enc` carries a `count` attribute (`Civ` writer untraced); the count-2
base-key record and count > 2 comparison (`0jc.A0u`/`A17`); the fresh-bundle
answer after a registration change; sessions built from `<keys>` in the receipt
(`151.A01`); normal-runtime group/broadcast retries. No retry has been answered
live.

## `pqsupport` and Kyber upload gate (resolved 2026-11-01)

`Can.A04` (the `pqsupport` producer) and the Kyber prekey upload both require
A/B property 21198 (`PQ_KEYS_UPLOAD_CODE`), default false; `q`'s A/B read does
not set it. Still open: whether MobileConfig overrides that default, and the
Kyber producer (ID allocation, counts, last resort) for an account where the
property is true. [kem-prekey-inventory](kem-prekey-inventory.md).

