# Ordinary direct sender revocations

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Authenticated ordinary peer and held own-device sessions now project supported
sender revocations into MessageUpdated/Revoked. The event reports a received
action; native timing/type eligibility, receipt/cache cleanup and physical
message/media deletion remain unported. Outgoing revoke/edit intents and
admin/group/status revocation are not implemented by this change.

## Native selection and parent ownership

The [recipe](apk/recipes/2.26.37.71-sender-revoke.json) reproduces
[13 method topics](apk/2.26.37.71-sender-revoke/targets.json), including full
signatures, register flow and exception tables: Bua/Bub receive, Cbf selector,
C4F enum, BZb/BZa descriptors, D9M writers, 153 store, DEy edit parser, 1hR factory,
CoT constructor and context-aware unwrap. The export is 18 files / 179,139 bytes.
Complete independent [REA sender builder](rea/2.26.37.71-sender-revoke/revoke-sender-receive.json),
[admin builder](rea/2.26.37.71-sender-revoke/revoke-admin-receive.json),
[selector](rea/2.26.37.71-sender-revoke/revoke-selector-A01.json),
[enum](rea/2.26.37.71-sender-revoke/revoke-enum.json),
[edit parser](rea/2.26.37.71-sender-revoke/revoke-edit-parse.json) and
[factory](rea/2.26.37.71-sender-revoke/revoke-factory.json) retain their Evidence IDs
and limitations. These are static observations, not APK execution.

Cbf.A01 requires Message protocol presence and BZb type presence, so absent type
must not default into REVOKE. C4F.A0T is REVOKE=0; A0O is MESSAGE_UNSCHEDULE=32.
Bua.CF4 accepts those two types except edit-class 8, constructs Bpt with the
envelope CoT.A0A key and time, and stores BZb.key.id as 1ii.A01. It separately
handles a parsed special participant through C0hd. Rust admits only REVOKE and
an absent/empty participant; unschedule and special participant classes remain
outside this port. Unlike keep decoding, this builder does not call C7G's full
parent-key validator or use the embedded remote/fromMe to select ownership.
Do not substitute the keep/reaction parent policy here.

The [retained 153.A00 store](apk/2.26.37.71-sender-revoke/revoke-store.txt),
003e..0042 and 009a..00c6, constructs the ordinary target from envelope chat,
old id and envelope ownership. Later branches include parent presence, author
association, special/admin types and payment/old/terminal drops. These broader
effects are not replaced by a received API event. Bub.CF4 is a separate
edit-class-8 admin path with remote chat, participant and group checks.

DEy.A00 returns zero for absent/empty edit, otherwise uses Integer.parseInt and
allows 1/2/3/7/8. Rust reuses its existing Java decimal parser for the non-admin
1/2/3/7 values; nonempty zero, whitespace, overflow, unknown values and nontext
attrs decline. Bua excludes 8. The factory iterates DbV decoders, but its optional
ProtobufValidator/property 8556 and subsystem precedence remain partly unported.
A successful local projection does not prove every native configured validator
would admit the same payload. The Java factory has duplicate-block warnings;
complete DEX is retained rather than treating its Boolean reconstruction as proof.

## Receive, keys and recovery

The supported payload is one Message field 12 plus optional context, a present
REVOKE type, key and bounded nonempty original target id. Embedded remote/fromMe
cannot move a peer action to our outgoing message or move an own action to a peer
message. Mixed content, duplicate protocol fields, missing/unknown type and
special participants do not produce a revoke event. Group, broadcast/status,
recipient-qualified peer and unsupported agent routes remain outside this port.

Peer direct revokes use the authenticated incoming envelope. A matching saved
parent retains its original chat namespace and sender device through the saved
route/capture, including consumed records. Route/id/chat inconsistency refuses.
Without a loaded parent, projection carries a typed incoming reference rather
than fetching or inventing history. Directory-only aliases do not create an
attributed legacy parent; existing authenticated namespace selection applies.
The acting device remains distinct from the original parent sender device.

Own wrappers use the held own-device source and exact recipient authority.
Reactions/keep's existing own-parent selector resolves the outgoing target or an
unloaded typed outgoing reference; its ambiguity/outbox/source guards remain
unchanged. Editable headers now reach this route, but non-revoke edited bodies
remain unsupported without saving an advanced ratchet. Edited redelivery and
sender confirmation additionally require authenticated saved revoke content:
a new header cannot turn a previously saved text id into a supported revoke.

Original wrapper/capture and advanced session commit atomically with the event
and marker. Sender confirmation, reopen deduplication and clearing of own rows
use existing paths. Peer delivery confirmation still retains its existing edited
header refusal; full native edited-confirmation policy remains unported. No
parent body, media, receipt ledger, send intent, status, request id or wait is
rewritten. No automatic history fetch, read, retry or identity admission is added.
Existing listen/journal APIs expose the variant; no new route, option, schema
variant, dependency or configurable admission bypass is added.

## Tests and limits

An independent literal regression failed at the missing sender-revoke event.
It now proves embedded false/true/absent ownership and a foreign/special embedded
remote cannot retarget the peer action; unsupported routes/types and participants
stay opaque. Edit-parser boundaries include signed/zero-padded/BMP Unicode digits.
The encrypted-store test retains a consumed legacy PN parent through an
attributed LID route without deleting its row, checks once-only publication,
and retains a canonical saved companion sender device. The existing PN/LID
own-target matrix now includes revoke, embedded flag reversal, normal/outbox/
unloaded/mapped targets, ambiguity, source/registration and mixed-content guards.
Projection leaves all saved tables and journal unchanged.

The real Signal fixture receives a peer revoke and an own wrapped revoke, checks
target and actor, own confirmation, redelivery/reopen deduplication, clearing and
continued ratchet use. Original uncertain outbox/idempotency rows remain equal.
An encrypted edited-text wrapper is rejected without any saved-table/journal
change; the next supported counter still decrypts. A forged edited redelivery
of an already-saved ordinary text initially reproduced Duplicate; the new saved
content guard refuses it and does not confirm it.

Native physical deletion, time/type/terminal policy, orphan reconciliation,
mutable receipt/cache cleanup, full ProtobufValidator/participant/namespace
policy, outgoing revoke/edit and group/admin/status receive remain incomplete.
Fresh live revocation and official-client rendering are unverified. No ban
immunity or complete APK parity is established.

Validation passed 959 workspace/all-feature tests, strict Clippy and
warnings-denied Rustdoc, the real Signal regression with only TCP/Noise/Signal
enabled, 26 default CLI tests, the TCP/Noise build without Signal, API-symbol
checks and the experimental CLI build. The hash-pinned 13-topic bundle check
reproduced all 18 files / 179,139 bytes.

## Bounded retained-account validation

**Observed live:** one 45-second receive window (58.19 seconds including
connection/shutdown) authenticated and exited successfully. Chat/outbox sends,
automatic reads and reconnects were disabled in a temporary private config;
the original profile/config/key stayed in place. Preflight had no queued action,
pending receipt or eligible outbox send. Existing uncertain outcomes were kept.

The before/after CLI action, receipt and eligible-outbox views compared equal,
as did generation, registration and journal head. Only Checkpoints count rose
1,139→1,149, all ten in the inbound category; other table counts stayed equal.
No new journal event was published. These are count/summary comparisons, not
proof of byte equality for all encrypted rows or live revocation acceptance.
[Redacted aggregate evidence](sender-revoke-live.json) contains no address, key,
request/message identifier or original private capture.

Postflight initially hit a local CLI build-feature mismatch after the default
API-symbol test replaced the experimental executable. Rebuilding the experimental
CLI restored offline inspection; the same successful receive run was assessed.
No second window, replay, new account, key change or manual message was used to
manufacture a positive result. Fresh live revocation remains unverified.

## Peer receive-window follow-up — 2026-10-11

[Native peer window rejection](sender-revoke-window.md) now declines supported
ordinary direct updates whose authenticated parent and explicit times prove the
signed difference exceeds three days. This supersedes only that part of the
previous timing-unported boundary. Own-device revokes retain their separate
path; absent timestamps are not guessed. Rejected markers remain final, and
published history is preserved. Full eligibility, outgoing revoke and physical
body/media/receipt/cache deletion remain unfinished.
