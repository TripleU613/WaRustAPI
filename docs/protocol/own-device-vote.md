# Own-device ordinary poll votes

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
A held authenticated own-device session can synchronize a supported ordinary
poll vote into MessageUpdated/PollVote. The update targets our poll when the
embedded key's fromMe is true, and the received peer poll when it is false.
The voter must authenticate as a saved own primary PN/LID identity.

## Native ownership and the status singleton

The [recipe](apk/recipes/2.26.37.71-own-device-vote.json) and
[DEX bundle](apk/2.26.37.71-own-device-vote/targets.json) retain Message/unwrap,
receiver, parent/key mapping, key construction, parent resolution/edit fallback
and status-singleton methods. Complete independent REA envelopes include
[key mapping](rea/2.26.37.71-own-device-vote/key-mapper.json),
[key construction](rea/2.26.37.71-own-device-vote/key-constructor.json),
[parent resolution](rea/2.26.37.71-own-device-vote/parent-resolution.json) and
[edited-parent lookup](rea/2.26.37.71-own-device-vote/edited-parent.json).

Clc.A02 passes a null actor to BoQ.A00 for an own envelope. In BoQ.A00,
the ordinary direct branch retains the envelope chat and the embedded fromMe
flag. Ayo.A0S constructs 1gc from the chosen chat, embedded id and that flag.
This differs from incoming peer projection, whose ownership is interpreted
relative to the peer.

The apparent sentinel in the valid remote parsing branch is specifically
8oR.A00, the **status@broadcast** singleton: its constructor supplies user
status, getServer returns broadcast and getType returns 5. Its full
[class inventory](rea/2.26.37.71-own-device-vote/status-class.json),
[constructor](rea/2.26.37.71-own-device-vote/status-constructor.json),
[type](rea/2.26.37.71-own-device-vote/status-type.json) and
[server](rea/2.26.37.71-own-device-vote/status-server.json) are retained.
BoQ requires that subtype for its special parse result. An ordinary PN/LID
remote is not that subtype, so its branch falls back to the envelope chat.
A different ordinary embedded remote therefore does not retarget the update.
It must not be described as a generic valid-chat sentinel.

Ayo.A0R delegates to 1hQ.A02. The latter first loads the exact message,
then tries A00's edited-parent lookup, then an id prefix before __part__ when
present, returning the resolved message's concrete key. A00's SQL binds
original_key_id, chat_row_id and from_me. Exact cache hydration and its
transitive providers remain incomplete. Java failed for A02; complete smali
and the independent DEX export retain the branch targets. Current Rust does
not implement edited/suffixed-parent fallback or full special namespace mapping.
The [parent-secret trace](synchronized-poll-secret.md) remains the authority
for row-relative secret selection and creator/voter prerequisites.

## Receive, parent selection and API

The one-level wrapper preserves raw field 50 through outer-context replacement.
Its existing own source, exact recipient, held pin/session, original checkpoint
and account transaction checks remain effective. Own vote projection uses the
same decoded vote and shared parent-context selection, translating fromMe only
in memory to the peer-relative context convention. Original plaintext and route
are retained unchanged.

An own parent uses the existing local reservation or authenticated synchronized
parent fallback. A peer parent uses its saved authenticated receive record and
concrete original chat. Incoming parents must remain decrypted and have an
ordinary field-64 body; the shared supported-field check excludes quiz/timed/
anonymous/add-option fields 6..11. Local normal reservations keep their existing
priority and binding checks. Missing, cleared, ambiguous or unsupported parents
do not authorize a vote. Full expiry, edit, extended context and native parent
hydration policy remain unfinished.

Only saved account PN and validated saved own LID are candidate voters. AEAD
must succeed for an independently selected parent creator and one of those own
identities. The event's by is that authenticated primary identity, not the
companion device or the chat partner. Empty selection withdraws a vote; selected
hashes and unresolved hashes use the existing resolver. A foreign voter's valid
ciphertext under the same secret still refuses. Participant-bearing, group/
status/device keys, nonempty edit metadata, mixed content and malformed votes
remain unsupported on this direct route. These supported-subset guards do not
claim full native subtype/lifecycle parity.

The advanced receiving ratchet, original wrapper and MessageUpdated marker
commit atomically. Unsupported inner AEAD leaves the held session untouched
and earns no sender confirmation. A valid committed update uses the existing
sender confirmation and same-source deduplication, including after reopen.
There is no send-status mutation, new outbox intent, request ID, secret-cache
import, automatic read or retry reservation. Existing listen/event APIs and
saved-inbox inspection expose the update without a new route, option or schema.
Local clearing retains its existing plaintext/marker and historical-event
semantics; no app-state patch is sent to companions.

## Checks and limits

The real Signal regression failed at Unsupported before wrapper admission was
extended. It now receives a vote on a synchronized own poll, verifies outgoing
parent ownership and saved-own voter, confirms it, deduplicates after reopen,
clears it and continues the ratchet. A Signal-authenticated wrapper carrying an
inner vote by the peer instead of us refuses without checkpoint/session/pin
mutation or delivery confirmation; the next valid vote still decrypts.

An encrypted-store matrix covers PN/LID chats, own/peer parents, vote withdrawal,
ordinary embedded-remote fallback, missing parent, foreign voter/creator,
nonempty metadata, participant, group/status keys, mixed content, registration
and special incoming parent refusal. Projection leaves all saved tables and
journal unchanged until the normal receiver commits it. Existing ordinary
peer-vote behavior and owned parent-secret precedence remain shared.

Fresh live own-device vote interoperability is unverified. Edited/suffixed
parents, special poll lifecycles, own group/status routes, cross-source duplicate
reconciliation, bootstrap and complete APK/native crypto parity remain unfinished.
No live account or request was changed; no ban-immunity claim is made.

Validation: 948 workspace/all-feature tests, strict workspace Clippy and
warnings-denied Rustdoc passed. The real Signal regression also passed with
only TCP/Noise/Signal enabled. All 26 default CLI tests, experimental CLI build
and API-symbol check passed. The 12-topic pinned bundle reproduced all 17 files
/ 95,539 bytes. Formatting and repository link/evidence hygiene passed.
