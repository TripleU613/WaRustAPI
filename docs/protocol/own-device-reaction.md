# Own-device ordinary reactions

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Held authenticated own-device sessions now synchronize ordinary reaction set
and removal into MessageUpdated/Reaction. Target ownership follows the embedded
fromMe flag; the actor is the authenticated own source's normalized primary
PN/LID. No send intent or send-status fact is manufactured.

## Source and ownership

The [recipe](apk/recipes/2.26.37.71-own-device-reaction.json) and
[DEX bundle](apk/2.26.37.71-own-device-reaction/targets.json) retain eight topics:
Bal.A01/A00, BWI and Message descriptors, BoQ.A00, Ayo.A0S, context-aware unwrap
and wrapper validation. The complete independent
[Bal builder](rea/2.26.37.71-reaction-text/reaction-builder.json) and prior
[reaction-key evidence](reactions.md) are reused for the same pinned bytes.
The [text predicate](reaction-text.md) remains the 30-code-point boundary.

Bal.A01 constructs BnG through A00, then checks the envelope 1gc.A02 at
0206..020e. For an own envelope it jumps to 025e, sets actor v7=null and jumps
back to the common 0212 path. That path calls BoQ.A00 at 024a..0250 and assigns
the resulting target before validation/return. The Java rendering has duplicate
block warnings; the DEX branches establish that the call is not non-own-only.
BoQ's ordinary own branch retains envelope chat and embedded fromMe; Ayo.A0S
constructs the selected chat/id/flag key. The status singleton branch is distinct,
as documented in [own-device vote mapping](own-device-vote.md).

The ordinary builder does not gate model construction on parent presence. This
supports a typed unloaded-parent reference, not invented history, a secret or
native orphan-store/canonical-author equivalence. Full add-on storage, actor
canonicalization, ordering/edit/timestamp effects and encrypted reaction paths
remain incomplete. The API represents the normalized authenticated source user;
it does not claim to reproduce every native sender-row choice.

## Receive and saved parent selection

The wrapper accepts type=reaction and exactly one field 46 plus optional
Message context. Raw reaction bytes survive outer-context replacement. Existing
registered account, own-source/recipient, held pin/session, original checkpoint,
source namespace and atomic commit checks remain effective. Projection independently
rechecks the source route before assigning the actor. Foreign sources, malformed
or overlong text, participant-bearing/special/device keys and mixed content do
not produce an ordinary direct reaction. Edit attributes and new-device admission
retain their existing refusal. An ordinary embedded remote does not retarget the
envelope chat; group/status/interop routes remain outside this port.

For fromMe=true, an attributed local outgoing intent retains its original key.
If none exists, the shared authenticated normal own-message lookup retains the
synchronized parent chat. Otherwise the update carries a typed outgoing reference
in the envelope chat. For fromMe=false, the existing authenticated direct-record
selector retains the original incoming chat/author, including consumed records;
an unloaded parent similarly receives a typed incoming reference. No parent
plaintext is copied into the event or fetched from the network.

The shared own selector was moved from poll-only lookup into own_device::record_in.
It pages the received-own prefix, validates decrypted status, current registered
own PN/LID source, exact wrapper destination and a supported normal content body.
Reaction/vote update records are not selected as normal parents. Directory-only
aliases cannot change the original target namespace. Two attributed normal
parents refuse with ambiguous synchronized parent. This counts parent identity
before inspecting poll type or secret availability; the former poll-only path
filtered secret-bearing polls first. That stronger local ambiguity guard is not
claimed as native migration policy. Poll lookup still returns local reservations
first and preserves its public fixed ambiguity diagnosis. It imports no cache.
Pagination remains a local scan rather than native indexed SQL; large synchronized
histories may need an index. Namespace migration, cross-source reconciliation,
cleared-parent layout and native orphan/edit policy remain incomplete.

The receiver atomically commits the advanced session, original wrapper/source/
route and message_updated marker/event. Existing sender confirmation and same-source
redelivery deduplication apply, including after reopen. Clearing strips reaction
plaintext like other own rows; historical events retain their existing semantics.
A fresh update referencing an unloaded/cleared parent does not recreate its body.
Outbox contents, attempts, uncertain facts, IDs, idempotency and server waits stay
unchanged. No automatic read, retry, history fetch or key adoption is introduced.
Existing listen/event APIs and saved-inbox inspection expose the update; no new
route, option, schema variant or dependency is added.

## Verification and remaining work

The real Signal regression failed at Unsupported before reaction admission.
It now receives set and removal on a synchronized own poll with no local send
intent, checks target/actor, sender confirmations, reopen deduplication, clearing,
continued ratchet use and unchanged original in-flight/DeliveryUnknown outbox
and idempotency records. The existing vote/text/media/echo branches remain covered.

An encrypted-store matrix covers PN/LID recipients and own/peer targets, local
outbox and synchronized parents, unloaded references, own PN/LID actors, removal,
ordinary embedded-remote fallback, original keys through authenticated namespace
association, directory-only refusal to remap, ambiguity, foreign source,
registration, participant/status keys, mixed content and native text limit.
Projection itself leaves all tables/journal unchanged. Existing synchronized
poll vote/secret tests cover the moved source, binding, paging and clearing guards;
the paging fixture now places padding within the scanned own prefix.

No live account or server request was changed. Fresh live own-device reaction
interoperability, encrypted reactions, special/group/status synchronization,
full canonical author/orphan/ordering policy, bootstrap and complete APK parity
remain unfinished or unverified. No ban immunity is established.

Validation passed 951 workspace/all-feature tests, strict workspace Clippy,
warnings-denied Rustdoc, the real Signal regression with only TCP/Noise/Signal
enabled, 26 default CLI tests and the experimental CLI build. The eight-topic
pinned recipe reproduced all 13 files / 97,611 bytes. API-symbol, formatting and
repository link/evidence hygiene checks passed. No live account was opened.
