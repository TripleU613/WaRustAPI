# Status history (to 2026-09-24)

The layered status log as it stood before `STATUS.md` was consolidated on
2026-09-25. Entries are newest first and many are superseded by later ones;
kept verbatim so no finding, negative result or dead end is lost. For what is
true now, read [`STATUS.md`](../STATUS.md).


**Two-step verification can be set, and one attempt has an uncertain outcome.**
The request shape is confirmed from `LX/0lo->A06`
(`TwoFactorXmppMethods/sendSetTwoFactorAuth`): `<iq
xmlns="urn:xmpp:whatsapp:account" type="set"><2fa><code/><email/></2fa>`, each
child present only when supplied — so setting a PIN cannot clear the recovery
address as a side effect, and an empty code turns 2FA off.

**A PIN is set on the working account**, confirmed by `iq type=result`. The PIN
itself is in private lab storage, mode 0600, outside Git.

Getting to that confirmation is the interesting part. The first attempt was
transmitted and its reply was lost — an undecryptable incoming message was ending
the whole session (see below), and IQ replies are not redelivered, so the outcome
was genuinely unrecoverable: the id does not appear among any of the 46 preserved
`iq` frames. There is no server-side read to check against either, since WhatsApp
keeps the code in local preferences (`two_factor_auth_code`) rather than on the
server.

What resolved it was **idempotence**, not a replay. Setting the same PIN to the
same value leaves the same state whether or not the first attempt landed, so
issuing it again could not produce a second effect — which is the actual reason
the no-replay rule exists. A fresh action was queued with the identical value
over the now-stable connection and answered `result`. The first attempt stays
recorded as `Sent`/unknown rather than being rewritten, because that is what it
was.

**A single undecryptable message was ending the connection.** `inbox::process`
propagated the Signal failure instead of degrading, so every stanza queued behind
it went unprocessed and `listen` reported the server hanging up — our own error
doing it. Now degraded to `Undecryptable`, but only for the Signal layer's own
failures; a post-decryption transactional rollback still propagates, which an
existing test caught when the first version of the fix swallowed it.


**APP-STATE WRITES WORK (2026-09-24).** A `contactAction` patch was accepted:
`iq type=result` with `<collection name="critical_unblock_low" version="1"/>`.

`SyncdPatch.version` **was** the cause of the original `400 bad-request` after
all. The third attempt that supposedly cleared it had run a **stale release
binary** built before the fix — the saved request's patch bytes begin
`0a 02 08 01`, so the correction was never on the wire. A live result only tests
the code that was actually running.

A second patch then returned `409 conflict` at the **collection** level inside an
`iq type=result`, with the `<patches>` we are behind on. That exposed a second
bug: `actions::record_reply` read only the outer type and recorded the conflict
as `Ok` — an error reported as success. It now reads collection-level errors.
`CollectionState` is persisted between patches and promoted only when the server
accepts and reports the version we claimed. The catch-up is implemented and the
whole loop is verified live: accepted at version 1, conflicted, caught up from
the returned patches, then accepted again at version 2. `app_state::keys_for`
verifies a server-sent patch by the key it names, which is a different question
from "which key should I write with" and must not inherit that policy.


**STATUS POSTING IS LIVE-VERIFIED (2026-09-24).** A styled text status — font,
background and text colour — was posted from this client and appeared on the
operator's handset. It uses exactly the sender-key fan-out a group message uses,
aimed at `status@broadcast`, with no `addressing_mode` and no `phash` (neither
appears on any incoming status we hold).

The audience is named in the private JSON rather than inferred: a status has no
membership list, and the APK's `E2eStatusJobParams.A02()` is the retry list, not
the audience. The payload is `Message.extendedTextMessage` (field **6**) with
`textArgb`/`backgroundArgb` as fixed32 and `font` 9, checked against the receive
path that was live-verified against a real status long before this encoder
existed.

`E2eStatusJobParams` also settles an old unknown: `participantUsersHash` and
`participantDevicesHash` identify `groupParticipantUserHash` as the users hash
and `groupParticipantHash` as the devices hash.


**GROUP SENDING IS LIVE-VERIFIED (2026-09-24).** The operator confirmed a
sender-key fan-out from profile `l` reads correctly on their handset. The
stanza, the participant hash, the per-device distribution, the `skmsg` and the
sender-key chain all work.

The cause of every earlier failure was ours: `direct::prepare_payload` has always
applied the E2E padding itself and the group send padded the distribution
**again**, leaving 1..=16 junk bytes on the recipient's protobuf after their
single strip. Protobuf tolerates or rejects those depending on what the random
bytes spell, which is why it presented as intermittent and device-specific:
"waiting for this message" when the leftovers hit the SKDM, "your version doesn't
support it" when they hit the payload, and two-of-three devices succeeding on the
same send. `addressing_mode`, the version 2 `phash` and the AD-form `<to>` were
all real corrections and none of them was the cause — the envelope was being
fixed while the contents were corrupt.

Padding is now the encrypting layer's job on both paths, with a round-trip test
that fails if a second layer returns.


**The group fan-out works.** From the new account into the operator's group, two
of the three addressed devices decrypted the message and said nothing; only the
operator's handset returned `receipt type="retry" count=1 v=1`. A device that
cannot decrypt sends a retry, so silence from the other two is success. The
stanza, the participant hash, the per-device distribution and the `skmsg` are all
sound — five sends from the previous account looked like total failure only
because the operator's phone was the single reporter we had.

The retry receipt was caught by the post-send window added yesterday. Every
earlier attempt closed the connection on the `ack`, so the receipts were being
sent to a socket we had already dropped. **We have never answered a retry**,
which is what keeps that one device unable to read the message — and is itself
un-clientlike behaviour worth fixing before it is read as something worse.


**App-state patch: third refusal, one cause eliminated (2026-09-22).** Reading
`LX/CRA->A00`, the site that actually assembles an outgoing `SyncdPatch`, showed
it sets `mutations`, `keyId`, `snapshotMac`, `patchMac`, `deviceIndex` and
`clientDebugData` — and **never** `version`. All nine references to
`LX/BYK->version_` in the APK are reads. We had been sending it. Removing it
(and asserting its absence on the encoded bytes) did not change the answer: the
third live attempt was refused with the same `400 bad-request`. So the version
field is eliminated as the cause, and live attempts stop again. The leading
remaining candidate is the snapshot precondition — this account has no snapshot
for any collection, and no amount of correct patch bytes can supply one.
Details: [chat actions](protocol/chat-actions.md).


**Group sending reaches the server but not yet the reader (2026-09-22).** One
sender-key fan-out was transmitted and the server acknowledged it, but the
operator reports **"waiting for this message"** — a device stored the stanza and
could not decrypt it. The `ack` carried a version 2 `phash` and
`count="2"`: the count matched what we addressed, and the hash was one our
version 1 implementation could not produce.

The operator confirmed the placeholder is on the phone itself — a device we did
address — so the stanza was re-checked against the APK and every structural
doubt closed: `<participants>`, `<to jid=…>` and the outgoing `<enc>` attribute
set are all confirmed now, the last two from the unobfuscated
`SendLiveLocationKeyJob`.

That left `phash`, and the ack had supplied the server's own value for this
group. Searching locally for a construction reproducing those six bytes found
exactly one, and it settles three questions carried as unknown since the hash
was first implemented: the set is **every member including the sender**, the JID
form is the **AD form** with `.0:0` written out, and version `2:` is **SHA-256**
where version `1:` is SHA-1. `participant_hash_v2` implements it against
independently computed synthetic vectors, and the fan-out now sends it.

**The server validated that derivation.** A second fan-out carried our own
computed hash and was acknowledged with `count="2"` and **no `phash`** — the
correction present on the first ack is gone, which is the server agreeing our
claimed membership matches its own.

**It still would not decrypt, and the cause looks like the `<to>` JID encoding.**
Every `participant` the server stamps on an incoming group stanza decodes as an
**AD-JID** with `device=0`, while the group `from` beside it is a plain JID pair.
Our `Jid` only uses the AD encoding when the agent or device is non-zero, so a
member's primary device was going out as a plain *user* JID — nothing for the
server to resolve to a device, so nowhere to deliver that distribution enc, while
the `skmsg` still arrived and the ack stayed clean because its `count` is the
server's own tally rather than an echo of our list. `sender_key_stanza` now
renders every `<to>` in AD form.

Because the earlier sends created our half of sessions the peer never created,
the private message JSON also takes `fresh_sessions`, which replaces the saved
ratchet from a fresh prekey bundle while keeping — and still enforcing — the
pinned peer identity.

**That did not fix it either.** Three fan-outs, three clean acks, three
placeholders. `addressing_mode`, the version 2 `phash` and the AD-form `<to>`
are all corrections worth keeping — the stanza is closer to what the APK builds
than it was — but **each is now eliminated as the cause**. Constant throughout:
a clean `ack` with `count="2"`, and **no retry receipt across five receive
windows**, which a phone that cannot decrypt should be sending.

Two hypotheses remain. LID Signal sessions have never been proven with this
recipient — the one live-verified direct message to them was PN-addressed, and
`LX/CPQ->A00` confirms a LID-addressed chat encrypts under a LID address, but
"correct" and "working" are different claims. And the retry-receipt silence
probably has the same mechanical cause. The discriminating experiment avoids
groups entirely: a **direct** message to their **LID** over the path already
verified for their PN, changing only the addressing form.
`MessageRequest::recipient` now accepts a LID for exactly that.

**LID sessions work; out-of-band distribution did not help.** A direct message to
the recipient's LID arrived readable, which proves our prekey fetch, session
establishment, `pkmsg` and padding are all correct for that handset. Delivering
the sender key as separate acknowledged direct messages — a real shape, since
`includeSenderKeysInMessage` is a per-send flag — still left the group message
unreadable.

Reading the receive side (`LX/Cm3->A05`) eliminated the distribution message
itself: `LX/DRv` accepts version `3` exactly with all four fields, which is
precisely what we emit and what WhatsApp's own serializer writes. What remains is
whether the `<participants>` enc reaches the device at all, and whether
`LX/COU->A00` can resolve a sender address from a non-group stanza — if it
cannot, the out-of-band test was a no-op rather than a refutation. Both need
visibility on **both** ends of a group, which no spare lab account can currently
provide. Original result: the fan-out was
transmitted and acknowledged: `addressing_mode=lid`, two
recipients, two devices, both device lists fetched in the same connection, both
sessions established from the same batched `encrypt` key query. Handset display
is not yet confirmed by the operator. The blocker was never the crypto — it was not knowing who to
address. `SendE2EMessageJob` turns out not to compute the recipient set at all:
`targetDeviceRawJids`, `groupParticipantHash`, `useParticipantUserHash`,
`includeSenderKeysInMessage`, `useLidForEncryption` and
`forceSenderKeyDistribution` are all constructor parameters supplied by
`LX/DNu->run`. A **negative result worth keeping**:
`messageRecipientDevicesProvider` (`LX/CQl`) is *not* the resolver — it reads
the `receipt_device` tables, i.e. devices that already hold a receipt row for
this message, and is reached only inside the counter-abuse-token branch.

So the set is the ordinary one: the chat's participants expanded to their
devices. `lab-transport --login --message <json>` with `group` in place of `to`
does that on one connection — `w:g2` info query, `usync devices`, an
`encrypt` key query for the devices without a session, per-device SKDM, `skmsg`,
one atomic commit of every session and our own sender key with the send intent,
then one write and one `ack`. It refuses when a member has no device list rather
than reaching only part of the group.

Two things that had been guesses are now settled: `addressing_mode` is
**confirmed** in `A06` and **observed** as `"lid"` on every stanza from the
operator's group (`lab-inbox --inspect` prints it), and the `<success>` login
stanza carries this account's own LID (**confirmed** `LX/0iS->A12`), which a
LID-addressed group names us by and which we now record. `phash` is still
omitted: it is `groupParticipantHash` verbatim and the membership set behind it
is still unknown, so an absent attribute is honest where a wrong six-byte prefix
would not be. One send without it was accepted, for a group whose membership had
just been fetched — that is not evidence that it is generally optional. Details: [groups](protocol/groups.md).


**A WhatsApp Status update was received and decrypted (2026-09-22).** The first
one this client has ever seen: it projected as `ChatId::Broadcast` with the
poster in `MessageKey::sender`, after the poster's SKDM installed a sender key
under `sk:status@broadcast:<poster>` and the `skmsg` decrypted through the same
sender-key path group messages use.

Getting there ruled out four hypotheses with evidence rather than guesswork — a
recipient performs no subscribe step (`w:status` has zero references), privacy
tokens do not gate delivery, the poster's privacy list was not the cause, and
nothing was dropped on our receive side. The decisive instrument was the
server's own `<ib><offline_preview/></ib>`, whose per-class counts reported
`status=0` in every session and proved the gap was upstream of our socket.

What changed is that the account stopped looking unlike a normal client: it now
registers its address book (`usync mode=full context=interactive`), announces
`<presence type="available"/>` after connecting — which it had never done — and
performs a read-only app-state fetch. A status posted after those landed
arrived. **Which of the three was operative is not established**; they were
introduced together. Details and the honest boundaries:
[status](protocol/status.md).



**Group message receive LIVE-VERIFIED (2026-09-22).** Incoming
group stanzas (`from`=group, `participant`=sender) now decrypt through the
Signal sender-key protocol. A sender-key distribution message (`Message` field
2) installs the member's key, and that stanza's `skmsg` child decrypts with it;
the member's direct session, pinned identity, consumed prekey, sender key and
inbox row commit together, and the result projects as a `NewMessage` keyed by
`ChatId::Group`. An unreadable stanza degrades to Unsupported rather than
dropping the connection. A delivery receipt
(`<receipt to=<group> id=<id> participant=<sender>/>`) is sent for a decrypted
group message; both its attribute set and the device-addressed participant form
are confirmed from `LX/Cp2->A00`/`A06`, which echoes the sender JID without
stripping the device or converting between phone JID and LID. Live: the operator created a group on their handset, added
the lab number and sent to it; two group stanzas arrived and both decrypted,
installing the sender key and projecting as group `NewMessage` events.

Two defects only the live stanza exposed: `senderKeyDistributionMessage` is
**field 2**, not field 18 (18 is `liveLocationMessage`) — the wrong tag matched
nothing, so no sender key was ever installed; and the combined stanza really does
carry a member's `msg` and its `skmsg` together, which APK evidence had left
inferred. Because the server does not redeliver a queued stanza, both messages
were recovered from their preserved captures with the new
`lab-inbox --replay-captures`, which re-runs preserved captures through the
normal receive path offline. Group *sending* (per-device SKDM fan-out) is not
implemented. The participant hash it requires is now traced and implemented
(`transport::groups::participant_hash`: sorted raw JIDs, SHA-1, first six bytes,
Base64, `"1:"` prefix — tested against independently computed vectors), but
which membership set feeds it remains unknown.
Details: [groups](protocol/groups.md).

**API reference added:** an OpenAPI 3.1 document now describes every CLI command,
HTTP route, library capability, request/response field and recognized server
response. It is served at `/openapi.json` from the control plane and can be viewed
at `docs/api/openapi.json` in the repository. Most operations today are
`x-transport: cli`; the HTTP routes beyond the inspection routes (health, profile,
capabilities, wire-profile) are contracts for future implementation, not served yet.
See `docs/api/README.md` for usage and `docs/api/openapi.json` for the full spec.


**Onboarding is two commands:** `accounts new` with the 2.26.37.70 token material
creates the account config, master key and profile automatically. Then
`lab-signup begin` and `verify-code` complete registration. Reference: [configuration](configuration.md),
[working recipe and token material](protocol/registration-lab.md#working-recipe),
[accounts](accounts.md).
The CLI reference (`docs/cli.md`), the OpenAPI document (`docs/api/openapi.json`, with `x-cli` and `x-library` per operation) and the configuration reference (`docs/configuration.md`) are kept current by tests.


**Chat actions landed (2026-09-20):** block/unblock/block-list fetch, spam
report (not sent live), disappearing timer, keep in chat, advanced chat privacy
(limit sharing), default disappearing timer IQ, local contacts with a `usync`
existence check, and local clear-chat, all on a new durable one-shot IQ action
queue (`whatsapp-cli actions`). Live on the lab account: block and unblock
answered, the contact check answered, and the three E2E setting messages were
delivered (two also read). Evidence and open points: `docs/protocol/chat-actions.md`.


**Live checks passed on the saved account:** a combined `listen --send-outbox`
session uploaded the first 50 one-time prekeys (server acknowledged) and sent one
queued text that was accepted, delivered and read; a device query for the test
contact returned its single primary device; an explicit read receipt was sent
and acknowledged; reactions now round-trip (a contact's 👍 was decrypted and
projected, ours displayed once keyed by the contact's LID). Polls decode and
send: our vote was server-accepted and our own poll delivered; incoming votes
decrypt. Quoted text replies project with the quoted reference and our quoted
reply was delivered and read. Checklist item
10 is now live-verified; item 8 keeps only
own-account sync, snapshot projection and companion fan-out open.

**Prekey replenishment implemented (offline verified):** from the restored
comparison APK, which is build 2.26.37.71, the client now generates one-time
prekeys, uploads unsent ones at session start and after the server's low-count
notification, and follows `count_low` with bounded extra batches. Post-quantum
keys are not sent. The first live upload has not run yet. Contact device lists
are now fetched with `usync devices`, verified against pinned identities and
stored; `devices` and `encrypt` notifications are acknowledged. Fan-out to
companion devices and own-account device sync remain open.

**Bounded reconnect added (offline verified):** the account runtime now retries
after recoverable connection loss with capped jittered backoff and a fresh
handshake, configurable via `transport.reconnect` in the private JSON (default
five attempts; `0` disables). Refusals, revocation, missing registration and
uncertain logins never retry, and reserved sends/receipts are never replayed.
`listen` survives transient loss. Live reconnect behavior is not yet observed.
The public `send_receipt` now queues explicit `read` receipts for authenticated
incoming direct messages through the durable receipt outbox (offline verified;
no live read receipt sent yet). `listen --send-outbox` is the first combined
receive-and-send CLI session; `outbox-pending` and `cancel` inspect and withdraw
queued intents offline before it runs. No live combined session yet. The session
loop now runs over an injectable node link, with socket cut-point tests at
activation, receipt, delivery-ack, key-query and keepalive boundaries. `mark-read`
queues read receipts from the CLI, and saved mute timers can now be resolved
locally against a server clock sample taken from authenticated stanzas.

**Public ordinary-media download integrated (offline verified):** comparison
clients can list sealed incoming references with `received_media()` and explicitly
export authenticated ordinary images/voice notes with `download_media()` to new
private files. View-once content, foreign references, metadata changes and
file/symlink overwrite are refused. A cached-download client test stays offline.
Public ordinary upload now also returns stable reusable references for the normal
outbox, with offline reuse/restart/uncertainty tests. A fresh public-API network
upload and full cross-device synchronization remain unverified/incomplete.

**Normal-outbox image and voice note server-acknowledged:** private JSON image
and voice-note intents now reuse the bounded upload module and enter the public
outbox through stable account-bound sealed references. Text and media share Signal
reservation and ACK tracking. One new image test to the existing test contact
received a matching server ACK through the public outbox after an explicit expired
media-route refresh; a receive-only session then saved recipient delivery. A new
two-second synthetic voice-note test also received a matching normal-outbox ACK
and subsequent recipient delivery (no read/played receipt for either media test).
Repeating either private JSON returned its saved ACK without uploading or
connecting again. Handset display/playback confirmation and public upload/download
convenience APIs remain incomplete.

**Ten-item messaging work in progress:** the persistent listener now runs through
the public account actor (explicit comparison opt-in). A 75-second authenticated
live check completed cleanly; ordinary incoming image download/display also
passed live. Incoming primary prekey support, receipt fact tracking, a durable
post-view opened-receipt queue and ordinary voice-note playback are implemented
with offline tests. Opened-receipt, incoming voice and new-sender live checks
remain pending. Trusted-contact token notification handling and atomic key-share
storage from an already pinned own-device session are implemented and tested;
device linking, key activation and general cross-device synchronization are not.
Normal direct-text outbox sending
has a separate explicit opt-in: one selected private-JSON text received a matching
server acknowledgement through the public client/runtime. A subsequent receive
session persisted a recipient delivery receipt (no read/played receipt). Repeating
the same JSON returns locally without sending again. Human handset confirmation,
broader media/outbox and cross-device work remain incomplete. See the
[ten-item checklist and handoff](messaging-completion.md).

The entries below are the historical experiment sequence; newer results above
and in the checklist supersede their then-current limitations.

**Local view-once viewer added:** `lab-open-view-once` authenticates/decrypts the
downloaded image and atomically consumes it before passing plaintext through a
pipe to a 30-second desktop viewer. No plaintext file is created. Reopening is
blocked, including after a viewer failure; accessible media body/key-bearing
protobuf records are replaced with consumed markers. Secure erasure of old
database pages is not claimed. Server opened receipts remain unimplemented.
The requested image was passed to the desktop viewer, which exited successfully
after its bounded display. It is now locally consumed, not available to reopen.

**Incoming view-once prefetch verified:** adding the APK's post-login active IQ
received a matching result, but alone produced only old messages. Delivery
receipts for previously authenticated/stored messages plus acknowledgements of
incoming receipts were then written; four additional payloads arrived and
authenticated offline. One contains ImageMessage flag 25 (`view_once=true`).
One HTTPS GET downloaded 91,562 encrypted bytes; size, hash and MAC passed.
The ciphertext is stored encrypted at rest as `downloaded_unopened`; no plaintext
export, viewing or opened receipt occurred. This sequence supports the missing
receive confirmations as the queue-progress issue, without proving a particular
server-side policy. No chats were resent or account identities changed.

**View-once sending confirmed:** the operator confirmed opening the outgoing test
picture once and being unable to reopen it. Incoming view-once download now has
an explicit ciphertext-only command, with HTTPS bounds, hash/MAC verification
and encrypted unopened storage. The initial invocation found no authenticated
descriptor; the receive correction above subsequently enabled a verified GET.
Local viewing/consumption is now available above; server opened receipts remain
unimplemented.

**First voice note server-acknowledged:** a locally generated two-second mono
Ogg/Opus beep was encrypted/uploaded and sent as `ptt` through the saved Signal
session. A matching server message acknowledgement was persisted. Repeating its
private JSON returned locally without another send. The operator confirmed
handset receipt; playback was not separately confirmed. Voice-note input is data
(`voice_note` path), not recipient-specific code.

**Incoming image message received:** after the operator corrected the recipient,
one bounded receive captured two new authenticated payloads. Offline decryption
identified one ImageMessage (Message fields 3/35, 1,227 protobuf bytes); the other
has fields 35/91: an associated-child wrapper containing another ordinary-image
descriptor, now classified from the APK. The attachment itself has not been
downloaded or decrypted. The inbox
now also scans pending/send-response captures from outgoing-message operations;
it previously scanned only dedicated receive captures. Image protobufs are now
explicitly labelled by redacted inspection. Inbound media download is not yet
implemented for ordinary images; the view-once prefetch above has not had a live
download to verify. Outbound success does not establish inbound media support.

**First image server-acknowledged:** the requested PNG was encrypted and uploaded
once (HTTP 200), then sent through the existing pinned Signal session. A matching
message acknowledgement was saved. Repeating the same private JSON returned
locally without another upload or connection. The operator confirmed receiving
the image on the handset. This is the explicit comparison lab path, not the default media
API. See [image usage](protocol/messaging-transport.md#image-sending-comparison-lab).

**Inbound decryption verified:** one bounded authenticated receive connection
captured two encrypted `msg` stanzas. Offline processing used the explicit
`sender_pn` mapping on their LID envelopes and the existing pinned Signal session
to authenticate/decrypt both. Plaintext protobufs and advanced sessions were
committed atomically in encrypted storage. A second offline run skipped both
without changing the ratchet or contacting the server. No chats or receipts were
sent. Continuous receiving, inbox display and receipt handling remain incomplete.

**Payload classification correction:** those two encrypted payloads are one
extended-text message (fields 6/35) and one protocol message (field 12), type 4
`EPHEMERAL_SYNC_RESPONSE`. They are not two user replies. Offline `lab-inbox
--inspect` reads the saved plaintext structure without advancing sessions or
printing message content. Comparison APK `LX/BXF`, `LX/BXE` and `LX/C1B` establish
these field/type mappings.

**First live encrypted text message server-acknowledged:** the comparison lab
verified the server certificate (intermediate serial 3 / leaf serial 361),
received authenticated login `success`, fetched the selected recipient's key
bundle, verified its signature, committed Signal session/send intent atomically,
and sent one encrypted text. A matching `ack` / `class=message` with no error
was persisted. Repeating the same private message request returned the saved
acknowledgement without connecting or sending again. The operator subsequently
confirmed handset receipt and reported replying. Inbound decryption is now
verified above; protocol delivery/read receipt handling remains unverified. This explicit lab
flow is not the default CLI/outbox runtime.
See [transport progress and usage](protocol/messaging-transport.md).

**Live registration completed:** the operator supplied the requested DOB locally
and one `/v2/consent` submission returned HTTP 200 / `ok`, with no pending branch
or wait. The same saved attempt transitioned to Registered; returned credentials
were committed in encrypted storage. Messaging authentication and server-side
message acceptance are now verified above. No DOB, phone or credential values
are recorded here.

**Consent follow-up:** an OTP submission returned HTTP 200 / `fail` / `consent`.
Static comparison-APK analysis found separate consent-result and age-verification
request shapes. The client now supports one-shot context inspection and explicit
private-JSON DOB/youth-consent submissions using saved keys and intermediate login.
Pending parent/app-store/unknown branches remain blocked. No choice is invented.
Live context refresh confirmed `pending=dob`, an intermediate login, no consent
ID/version, and no waits. The subsequent explicit DOB submission completed
registration as described above.
See the [consent trace](protocol/http-post-analysis.md).

**Post-OTP hardening:** library-side answer validation preserves challenges on
local errors; PIN transitions require successful HTTP and the expected status;
PIN waits are enforced; credential provenance distinguishes register/security.
Offline response tests cover refusal families and unknown outcomes without
answer replay. [Branch coverage and remaining work](protocol/after-otp.md).
Device-confirmation/second-code continuation remains missing; the later live
messaging result is recorded above.

**New onboarding patch (live dispatch verified):** new JSON templates enable an
existence preflight before code requests. The builder reuses saved identity;
typed recommendations and waits gate the selected method. Unknown/recovery
branches stop explicitly, and tests verify that they do not trigger SMS.
The fallback-only preflight response now permits the configured SMS method when
recommendations are absent and waits are zero. Resuming the saved attempt with
unchanged identity returned HTTP 200 / `sent` from `/v2/code`, with SMS wait 125
seconds and voice wait 65 seconds. Handset receipt and fresh-account signup
remain unverified. The local guard fix and continuation rule are in AGENTS.md.
One later code submission reached `device_confirm_or_second_code`, not a
registered session. No OTP or account PIN is retained in these docs.

**Latest comparison-lab result:** the operator-selected historical number was
retried once using the corrected JSON while preserving its saved keys/attempt.
HTTP 200 / `too_many_all_methods` imposed a 32417-second retry/SMS/voice wait.
The cooldown is persisted and no further send was made.

**Previous number's result:** correcting one miscopied byte in the APK-derived
token salt changed HTTP 200 / `bad_token` to HTTP 200 / `sent` on the same
attempt. The 1123-byte ENC/empty-H request returned 347 bytes in 3219 ms,
with SMS wait 125 seconds and voice wait 65 seconds. Registration now awaits
the SMS code. This confirms server-reported dispatch, not handset receipt,
completed signup, or messaging. Only the token changed; keys/history are intact.

**Previous number:** one APK-derived unknown-build-properties
User-Agent experiment changed the response from HTTP 200 / `bad_param` to
HTTP 200 / `format_wrong` (64-byte response). The saved attempt and account
keys are unchanged. This is a different refusal, not accepted signup or OTP
delivery. See the [experiment sequence](protocol/http-post-analysis.md).
APK-only follow-up traced this refusal to the official client's invalid-mobile-
number-for-country dialog (resource `0x7f123741`). The request already separates
country code and national number correctly. No further live request was sent;
a justified input correction remains necessary before another test.

**Historical offline baseline:** G0 complete (repository, dependency decisions, baseline).
G1 partially complete offline (codec, framing, Noise adapter, certificate
binding, schemas, reference manifest, **static APK observations**). **G2 not
attempted**: the Android registration payload encoding, transport port/layer
and login payload values are `Unknown` (see `docs/protocol/unknowns.md`). No
live server contact had been made and no OTP had been requested at that baseline.

**Owner-approved integration update:** registration HTTPS now selects
rustls/ring instead of the alpha RustCrypto provider. The optional
`experimental-signal-crypto` feature supplies an upstream-backed XEdDSA
certificate-signature verifier. This is not a complete Signal messaging
backend; trust-root evidence, key provisioning, sessions and live interop
remain unfinished. The historical offline inventory below predates this update.

The [persisted registration lab](protocol/registration-lab.md) now connects
key generation/signing, encrypted persistence and comparison request assembly.
It is explicitly lab-profile-only; default registration and messaging remain blocked.
The opt-in CLI feature `experimental-signup` now exposes `lab-signup` for
single-action operator-authorized HTTPS attempts. Ordinary commands remain
blocked. Two prior operator-authorized attempts returned `ServerRefused` without
a recorded HTTP reason; neither established signup or OTP delivery. The editable
[request profile](../crates/whatsapp-api/src/registration/request.rs) now exports to
private JSON, with offline preview and redacted HTTP diagnostics for future
explicit actions. [Outbound POST analysis](protocol/http-post-analysis.md)
has started; it is not a complete traffic audit. Server acceptance is unverified.

One ownership-confirmed, one-shot legacy diagnostic on 2026-09-18 UTC reused
the saved identity and returned HTTP 200 / `fail` / `bad_param`. The actual
reason now survives restart. The offending field is unknown; no OTP delivery
or successful signup is confirmed. Known refusals and cooldowns remain ineligible
for that legacy-only action, and HTTP-library retries are explicitly disabled.

**Key static findings (2026-09-17):** the pinned client contains classic
`Noise_XX`/`Noise_IK` patterns **and nine ML-KEM512 hybrid Noise patterns**.
If the server requires a hybrid handshake for primary devices, the current
`snow`-based adapter cannot be used (ADR 0004 addendum). A separately pinned,
WhatsApp-signed 2.26.37.70 comparison statically confirms `WA 06 03` and the
registration URL/form-body construction and JSON response vocabulary. Its
compiled Noise selector defaults to classic mode, though remote configuration
can select ML-KEM and remains unobserved. Its built-in socket path uses raw
TCP for primary/fallback gateways on 443/5222, and TLS only for numbered-edge
port 443; the port-80 fallback is raw TCP with a bounded HTTP/1.1 chunked
`POST /chat` wrapper. These comparison findings do not replace evidence for
the active 2.26.36.75 profile.

## Verified in this repository (offline)

| Area | Evidence |
| --- | --- |
| Toolchain and dependency contract | `rust-toolchain.toml` 1.98.1; `cargo run -p xtask -- dependency-audit`: no denied crates in `whatsapp-api`, `whatsapp-api +experimental-noise`, `whatsapp-cli` (report: `docs/protocol/dependency-report.md`) |
| Reference APK pinned | `com.whatsapp` 2.26.36.75 / 263607533, sha256 `ce7468…922e9`, signing cert `39:87:D0:…:5D:44`; reproduced by `xtask reference-manifest --check` |
| APK static observations | `xtask apk-strings`: 449,216 dex strings in pinned 2.26.36.75; hosts, 12 Noise pattern names, registration vocabulary, ClientPayload field constants. `apk-xref`/`apk-callers`/`apk-smali`: structural method and caller analysis, validated on separately hashed 2.26.37.70. |
| Token dictionary | `xtask apk-tokens --check`: ordered dictionary recovered from `classes2.dex` bytecode is identical to the committed fixture → `binary.tokens` is client-observed; version byte still ReferenceOnly |
| Store | encryption at rest, AAD row binding, wrapped data keys with wrong-key detection, master-key rotation, second owner rejected, generation fencing, atomic rollback, journal replay/ack/trim |
| Crash matrix (docs/design/README.md 19.2) | `tests/crash_matrix.rs`: rows 1, 2, 3, 9, 10 (rotation) and the second-process ownership test pass; rows 4–8 and 11 need a transmission path / Signal backend / I/O fault injection |
| Codec | independent hand-written fixture, JID kinds, packed strings, bounds, duplicate attributes, zlib bomb; `tests/codec_props.rs` property round trips (found and fixed an odd-length hex padding overflow), decoder never panics on arbitrary bytes, framing under arbitrary fragmentation |
| Transport | framing fragmentation/cancellation tests; Noise XX handshake + records against a snow responder; complete version-scoped classic/ML-KEM selector map with tested classic compiled defaults; typed comparison-only `WA 06 03` header and built-in gateway/edge raw-TCP/TLS map; cert chain binding incl. "valid-looking chain that does not bind to the remote key"; opt-in upstream Signal signature verifier; feature-gated HTTPS with explicit rustls/ring provider, validation on, no redirects/retries/proxies, deadlines and bounded streaming responses |
| Registration | state table enforced; handles bound to attempt/generation/step; answers never persisted; prerequisite persistence is atomic with the pre-request state; elapsed cooldowns resume without regenerating keys; ambiguous outcomes have a non-repeating reconciliation seam; successful account credentials commit atomically with `Registered`; version-scoped 2.26.37.70 endpoint catalog, deterministic common-field, `/v2/code`, `/v2/register`, `/v2/security`, public key-bundle encoding, APK-backed `/v2/code` token derivation, and fresh-install UUID/token prerequisite generation; conservative comparison HTTP flow including the account-PIN branch is tested end-to-end with a mock transport (not installed for the active profile) |
| Outbox | idempotent enqueue, conflict, limits, cancel rules, uncertain delivery representation |
| Client | durable enqueue → status → events → ack → restart over the same store |
| CLI / HTTP control plane | `keygen`, `store init/inspect`, `send`, `status`, `events --ack`, `connect`; loopback `serve` with health/profile/capability and comparison transport/registration wire-profile endpoints plus structured gated-operation errors |

Test count: 227 (`cargo test --locked --workspace --all-features`). Formatting,
all-feature/all-target Clippy with `-D warnings`, strict rustdoc, package
verification, and protobuf regeneration checks pass on the continuation
machine (Rust 1.98.1). The repository and CI pin Rust
1.98.1 as of 2026-09-22, raised from 1.94.1 so that the pinned toolchain and the
development machine are the same version and local verification speaks for CI
(ADR 0001 addendum).
The newer-toolchain chunk-iterator warnings were resolved without changing
encoding behavior.

## Blocked / not started

- Registration driver, transport connect, login: need Android-profile
  evidence (`docs/protocol/evidence-ledger.md` rows marked `Unknown`).
- Signal backend: owner approved AGPL-component evaluation (ADR 0005);
  implementation and interoperability gates remain open.
- HTTPS now uses an explicit rustls/ring provider under the owner-approved
  native TLS exception (ADR 0003). Registration remains experimental behind
  `experimental-registration-https`; no live success is claimed.
- Inbox, devices, groups, media, message operations, bridge adapter: gates
  G3–G6; modules intentionally not created yet.
- Data-key re-encryption, backup/restore, and I/O-fault injection for the
  remaining crash-matrix rows.
- `crates/tzibbur-api` and `crates/bridge` are not in this repository; the
  adapter contract could not be inspected.

## Suggested next steps (in gate order)

The [compact APK research snapshot](protocol/apk/README.md) now preserves
21 mapped protocol classes (149 methods), the comparison token dictionary,
and provenance/checksums in 564,315 bytes. The media-free export is bounded
to 2 MiB and can be regenerated or verified with `xtask apk-research-bundle`.
It is comparison-only structural disassembly, not a complete decompilation
or proof of live protocol behavior.

Continuation (2026-09-17): the supplied local APK was verified as the exact
2.26.37.70 comparison hash. Its frame writer uses a three-byte big-endian
length and rejects bodies at 16 MiB; server Noise record sizes remain open.
The Rust bytecode tool now displays `const/high16` values, with a regression
test covering this bound and signed bit patterns. No extra tools were needed.

1. Static: obtain exact pinned 2.26.36.75 and reproduce the comparison
   build's dictionary-version / `WA` header and registration findings; then
   continue into handshake selection and the trust root
   (`docs/protocol/apk-analysis.md`).
2. Integrate and validate the approved Signal candidate (ADR 0005); it also gates real XEdDSA
   certificate verification, for which no independent vectors are available
   offline yet.
3. With a controlled test number and operator consent: capture one
   registration and one handshake to move `registration.*`, `transport.*`
   and `noise.*` rows to `Observed`; only then implement the registration
   driver and `connect`.
4. Resolve whether the server requires the ML-KEM512 hybrid handshake
   before investing further in the `snow` adapter.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo run --locked -p xtask -- proto-check
cargo run --locked -p xtask -- dependency-audit
cargo package --locked -p whatsapp-api --allow-dirty
```
