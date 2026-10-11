# Own-device keep/unkeep updates

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Held authenticated own-device sessions synchronize ordinary keep and unkeep
into MessageUpdated/KeepInChat. This reports the action; native keep permission,
conflict ordering and physical retention/expiry application remain unported.

## Native evidence

The [recipe](apk/recipes/2.26.37.71-own-device-keep.json) reproduces the
[DEX bundle](apk/2.26.37.71-own-device-keep/targets.json): D9L.A01, BV3,
C7G.A00, BoQ.A00, Ayo.A0S, C2L, Message and context-aware unwrap.
The complete selected instructions retain signatures, register flow and exception
tables for future APK comparison. Eight topics reproduce 13 files / 91,027 bytes.
Existing [group keep evidence](group-keep.md) supplies the independent native trace.
A fresh post-restart [REA validator result](rea/2.26.37.71-own-device-keep/key-validator.json),
Evidence `ev_523d4e5950c33d7c0c9e5118a0b7280aa6a12c2cbaf201b307810d9e6c01ac2c`,
confirms nonempty present id/remote and successful native remote parsing.
Rust admits its supported key subset; broader native JID parity is not established.

D9L.A01 validates BV3.key with C7G.A00 at 05cc..05e4. BV3 carries key at 1,
keepType at 2 and timestampMs at 3; C2L defines UNKNOWN=0, KEEP_FOR_ALL=1,
UNDO_KEEP_FOR_ALL=2. At 0682..0692 the receiver selects its actor; an own
envelope branches to 06b8, nulls the actor and returns to the common 0696 path.
BoQ.A00 at 0696..06b0 maps the parent, then the receiver assigns the target.
The ordinary own branch preserves envelope chat and embedded fromMe. The status
singleton is a separate path. These are static observations, not live acceptance.

## Receive behavior

The own wrapper admits one field 51 plus optional context with type=text.
Supported full parent-key validation is now shared by keep decode, so malformed
or missing remote keys decline in existing receive callers too. Ordinary remote
PN/LID values do not override the authenticated envelope chat. The own update
mapper requires a primary plain direct remote and no participant; group, status,
interop, device, mixed content, unknown keep kinds and edited routes remain
outside this port.

Keep and reaction projection reuse one private parent/source helper. It rechecks
the registered own source and recipient, preserves embedded own/peer ownership,
retains an attributed outgoing intent or authenticated saved parent in its
original namespace, and otherwise produces an unloaded typed reference.
Directory-only aliases do not remap it. The actor is the authenticated source's
normalized primary PN/LID. This does not claim full native author-row policy.

The event retains sender timestamp and stanza update ID; no local clock is
invented. Original wrapper/source, atomic session commit, sender confirmation,
reopen deduplication and plaintext clearing use existing paths. Normal own-parent
selection excludes keep updates as well as timers, reactions and votes.
No history is fetched, cache imported, send intent created, send status changed,
read issued or identity key adopted. Historical events retain existing semantics.

## Verification and limits

The real Signal receive regression checks both keep and unkeep against a
synchronized own parent, original target, own actor, sender timestamp, update ID,
sender confirmation, reopen deduplication, clearing and continued ratchet use.
The encrypted-store matrix now covers both keep and reaction with PN/LID,
own/peer, saved/outbox/unloaded parents, namespace association, ambiguity,
foreign sources, registration, participant/status, mixed and invalid kind cases.
Projection leaves saved tables and the journal unchanged. An independent decoder
regression covers missing, malformed and unsupported remote keys.

Fresh live interoperability, native permission/expiry/conflict enforcement,
special/group/status own synchronization, bootstrap and complete APK parity
remain unfinished or unverified. No live account was changed and no ban immunity
is established.

Validation passed 956 workspace/all-feature tests, strict workspace Clippy
and warnings-denied Rustdoc, the Signal regression with only TCP/Noise/Signal
enabled, 26 default CLI tests, API-symbol checks and experimental CLI build.
Formatting and repository link/evidence hygiene checks passed.
The pinned eight-topic APK bundle check reproduced
13 files / 91,027 bytes.
