# Voting on synchronized own polls

Follow-up: [own-device ordinary vote updates](own-device-vote.md)
adds supported wrapped own votes on our or a received peer poll. It supersedes
the own-device-update gap below; special/edit/group/status paths remain unfinished.

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The shared own-poll lookup now resolves a supported authenticated synchronized
creation when no local poll-secret reservation exists. Incoming peer vote
context, explicit outgoing vote encryption and the existing own_chat_binding
diagnostic use it. No automatic vote or synchronized vote-update port is added.

## Current native parent and secret path

The [recipe](apk/recipes/2.26.37.71-synchronized-poll-secret.json) and
[DEX bundle](apk/2.26.37.71-synchronized-poll-secret/targets.json) retain six
topics: Ban.A01, Clc.A02/A04, BoQ.A00, 1hN.A02 and 0Z1.A01. Full independent
REA envelopes retain [decryption](rea/2.26.37.71-synchronized-poll-secret/decryption.json),
[helper field inventory](rea/2.26.37.71-synchronized-poll-secret/helper-class.json),
[secret store](rea/2.26.37.71-synchronized-poll-secret/secret-store.json) and
[creator selection](rea/2.26.37.71-synchronized-poll-secret/creator.json).

Ban.A01's immediate path resolves a parent through 1hQ.A02/Ayn.A0T and requires
a poll model before submitting CZg to Clc.A04. The latter maps the parent key
through Clc.A02, loads its message, then calls its A07/1hN secret store with
that message's A0j row ID. 1hN.A02 selects message_secret where
message_row_id equals the supplied row ID. A missing parent returns null;
missing secret or non-32-byte secret refuses decryption. This is parent-relative
lookup, not authority from a bare stanza ID or guessed contact alias.

0Z1.A01 uses B1V for a non-own parent. For an own parent it selects the retained
D5q device, otherwise A00's device, otherwise saved own CMn; a selected device
contributes its user JID. Clc.A04 validates creator/voter, 12-byte IV and
nonempty encrypted payload before calling its existing crypto path. Its Java
rendering contains duplicate/PHI warnings in the namespace retry closure; that
rendering does not prove every mapping/retry branch. The old 0ZG class name
in historical messaging notes is not present under that exact name in this
pinned REA class view. Current Clc/1hN evidence supersedes it for this lookup
boundary. Cryptographic derivation/JNI parity is not newly established here.

Full native parent-store hydration, creator metadata, effective immediate/
deferred property, edits/end-time/quiz policy and namespace migration remain
incomplete. The existing [parent-key trace](poll-votes.md) and
[ordinary synchronized creation](own-device-poll.md) constrain this port.

## Rust authority and unchanged state

polls::own_record_in first returns an existing poll-secret record unchanged.
A malformed, stale, foreign or legacy-unbound local reservation is not repaired
or replaced by synchronized content; its existing caller checks remain decisive.
When absent, the helper pages all encrypted Checkpoints and selects a
received-own record with the exact poll ID and an exact/authenticated primary
chat association. Directory aliases and raw sender metadata do not supply that
association. This scan is a local representation choice; native SQL uses the
selected parent row's ID. Indexing is deferred until large histories require it.

The fallback requires registered saved account, primary own PN, supported
nonzero own source device matching that PN or validated saved own LID,
decrypted status, exact wrapped destination and a supported ordinary poll with
a 32-byte secret. Cleared, malformed, foreign-source or secretless records do
not qualify. Two qualifying parents refuse with the fixed local diagnostic
ambiguous synchronized poll; they are never merged. This ambiguity refusal and
current-own-source check are local guards, not reconstructed native policy.

The return value is transient: original chat, secret, question/options,
selectable count and saved own creator identities. No poll-secret import,
checkpoint replacement, pin/session mutation, alias creation, outbox operation,
request-ID change, wait or journal event occurs during lookup. Incoming votes
still require authenticated vote decryption in the existing projection path.
Outgoing vote encryption uses the synchronized selectable count, refusing too
many selected options. Historical local reservations without that field retain
the existing unknown/unlimited behavior. Existing candidate creator/voter pairs
remain guarded by authenticated decryption; complete native mapping is unported.

Existing vote API/CLI and binding inspection gain support without a new route,
option, dependency or schema variant. clear_chat removes the wrapper plaintext,
so the fallback disappears. A matching local reservation continues to retain its
original independent lifecycle. No lookup manufactures a synchronized update,
network send, delivery fact or retry permission.

## Checks and remaining work

The real Signal fixture initially failed at unknown_poll. After authenticated
own creation, binding inspection matches and incoming peer vote context uses
the restored outer-context secret to decrypt/resolve the selected option without
changing checkpoints. Clearing makes the fallback unknown again.

An encrypted-store outgoing fixture covers pagination beyond 256 rows, exact
chat, authenticated PN/LID association in both directions, directory-only and
foreign-chat refusal, foreign/primary source refusal, missing secret, wrong ID,
cleared/malformed wrapper, duplicate attributed parents, unregistered account,
local reservation precedence and malformed local reservation refusal. Successful
outgoing votes decrypt with the original secret; synchronized selection count
is enforced. All tables and journal stay equal, including after reopen.
Existing local-reservation binding, malformed mapping, legacy state and regular
poll decoder/projection tests remain shared.

No saved live account or request was changed. Own-device vote updates, edits,
quiz/timed/anonymous/add-option/results lifecycle, special/group routing and
complete native parent/migration parity remain unfinished. Fresh synchronized
poll/vote interoperability and ban immunity are unverified.

Validation: 947 workspace/all-feature tests, strict workspace Clippy,
warnings-denied Rustdoc, 26 default CLI tests, experimental CLI build and the
minimal TCP/Noise/Signal regression passed. The six-topic pinned recipe
reproduced all 11 files / 72,291 bytes. No live account was opened.
