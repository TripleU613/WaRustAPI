# Normal group keep/unkeep outbox

2026-10-09. **Confirmed static/local implementation** for APK 2.26.37.71,
SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [33-topic recipe](apk/recipes/2.26.37.71-group-keep-outbox.json) reproduces
[selected code](apk/2.26.37.71-group-keep-outbox/targets.json) and maps it to
chat_settings.rs, group_references.rs and outgoing_group.rs. The
[field-write inventory](group-keep-outbox-inventory.txt) records timestamp and
context writers, including negative/untraced results. No live group send has
been performed. This supersedes the earlier local group-send refusal in the
[receive trace](group-keep.md) and [key research](group-keep-send-key.md).

## Closed static branches

**Confirmed:** the [original-key trace](group-keep-send-key.md) establishes
Bai.AD7 -> CSZ.A01, BV3 field numbers, original id/fromMe and participant
serialization. The native key serializer's effective `0x2ffb` parent-lookup
feature gate remains unknown. Supported Rust group references preserve an
explicit original author, validate retained attribution and derive supported
outgoing targets from the frozen own identity. They do not emulate the complete
native edited/partial-ID lookup or native own-companion provenance.

**Confirmed:** [1wk.invoke](apk/2.26.37.71-group-keep-outbox/remote-lambda.txt)
selector 2 (`00b8..00d4`) calls
[0Yo.A0B](apk/2.26.37.71-group-keep-outbox/remote-mapping.txt) with false.
That method requires 0DE.A0O before translation. The
[predicate](apk/2.26.37.71-group-keep-outbox/remote-user-predicate.txt) accepts
types 22, 18 and 26; normal g.us is type 1, as established in the
[group mapper](group-reaction-mapper.md#ordinary-non-own-envelope-and-group-branches).
Thus ordinary group remoteJid returns unchanged (`00c8..00ca`) even when
Cn6.A09 enables the CSZ wrapper. The full non-group mapping and Cn6.A09
producer remain outside this port; no PN/LID alias is guessed from a group key.

**Confirmed:** [11x.A01](apk/2.26.37.71-group-keep-outbox/keep-action.txt)
constructs CUq from the original message's B1V/key (`0280..0292`), creates a
BnF add-on and sets BnF.A01 to the selected keep kind and BnF.A02 from
08B.A00 (`02da..0312`), before handing it to Cgz.A02 (`033a..0354`). The
[add-on constructor](apk/2.26.37.71-group-keep-outbox/add-on-constructor.txt)
uses 1iZ -> 178; BnF.A07 returns its A01 keep kind. A timestamp is a creation
fact, not a value to regenerate each time a preparation query finishes.

[08B.A00](apk/2.26.37.71-group-keep-outbox/keep-clock.txt) prefers A03 plus
elapsedRealtime, then A04 plus elapsedRealtime, then currentTimeMillis minus
A02. **Unknown/unported:** offset producers, synchronization and native clock
calibration. Rust uses the original normal intent's accepted_at for group
keep timestampMs. This preserves local creation time across preparation and
restart without claiming native adjusted-clock parity. The public convenience
group-reference encoder uses local time when invoked without a normal intent;
the low-level group keep codec accepts an explicit timestamp.

**Confirmed:** Bai.AD7 sets BZc.bitField1 bit `0x100` for keep field 51.
[BoR.A02](apk/2.26.37.71-group-keep-outbox/stanza-kind.txt) does not classify
that bit as reaction, poll, event or pay. With a bare keep payload, no media
kind and ordinary flags it reaches `text` at `01d4..01e0`;
[BoR.A04](apk/2.26.37.71-group-keep-outbox/stanza-kind-predicate.txt) tests other
presence bits. Rust uses the existing group text stanza path, without
reaction-only meta or decrypt-fail attributes. Full native optional stanza/job
flags remain unported; this evidence does not classify arbitrary mixed content.

## Invocation and durable behavior

**Confirmed local implementation:**

```sh
whatsapp-cli --profile "$PROFILE" keep --chat "$GROUP" --id "$ORIGINAL_ID" --participant "$ORIGINAL_AUTHOR" --dry-run
whatsapp-cli --profile "$PROFILE" keep --chat "$GROUP" --id "$ORIGINAL_ID" --participant "$ORIGINAL_AUTHOR" --request-id "$KEEP_INTENT_ID"
whatsapp-cli --profile "$PROFILE" keep --chat "$GROUP" --id "$ORIGINAL_ID" --participant "$ORIGINAL_AUTHOR" --undo --request-id "$UNKEEP_INTENT_ID"
```

Use `--ours` instead of `--participant` for a target sent by this account.
Group incoming references require the original PN/LID author; the keeper can
be a different member. Direct targets reject --participant. Existing default
request IDs and queue records are unchanged; a conflicting intent does not
silently acquire another identity. Never change an acknowledged/uncertain
intent's ID merely to transmit it again.

These commands enqueue offline, except --dry-run, which requires
experimental-transport and performs reference preflight without enqueue or
network. Preflight is not membership, key lookup, final own-identity validation
or proof of remote keep permission. Unknown targets retain an explicit supplied
author, not an invented parent body. Existing legacy/canonical target collisions
and wrong-chat outgoing records refuse. `--ours` derives the supported actor's
frozen own identity at final preparation; complete historical own-companion
author provenance is not established.

The normal [group actor](normal-group-outbox.md) handles membership, signed
devices, missing key pages and original atomic fanout reservations. It now
accepts KeepInChat alongside supported quotes/reactions/media. Queued work is
sent only by explicit `listen --send-outbox`; that option drains **all** eligible
intents across namespaces. Inspect `outbox-pending` before opting in. No group
draft or live drain was authorized or performed for this chunk.

## Parent guard correction

**Confirmed local correction:** previously group reference resolution could
read the selected parent and its capture in separate store snapshots. It also
checked attribution before asynchronous fanout encryption, with no reference
recheck in the final reservation transaction. A parent change in that window
could leave a stale quote or a legacy same-ID author selection authorized.

Parent/route/capture resolution now shares one read transaction. The encoded
reference carries an in-memory guard; the actor resolves the same reference
again inside the crypto/outbox reservation transaction, before committing
crypto, the original attempt and wire reservation. Attribution errors or a
changed quote preview block with reference_changed and roll back without a
wire attempt. No new queue, request ID, identity or permission is introduced.
Unrelated delivery progress does not change the selected parent semantics.

Keeps/reactions resolve attribution without parsing/copying a parent text
preview. Consumed parent bodies remain absent; quoting checks consumption again
at commit and refuses a stale previously available preview. Matching group
scope, original participant namespace and legacy route checks are retained.
This optimistic local transaction guard is not a claim about native SQLite
transaction scheduling.

## Verification and outstanding native policy

**Confirmed offline:** independent literal BZc/BV3/BZa fixtures cover keep and
undo bytes, sender-relative fromMe, original author, device normalization and
unchanged direct encoding. The real normal-actor Signal tests cover PN/LID,
incoming/own targets, keep/undo, type=text, absence of reaction-only attributes,
original accepted timestamp, late parent changes and no replay across reopen.
Consumed-parent tests verify no body restoration, checkpoint mutation or new
event. A reservation-transaction regression changes quote body, consumption,
reaction/keep author or malformed parent data after encoding and proves a
fixed reference_changed refusal plus rollback.
CLI parsing covers --participant/--ours conflict and --undo/--dry-run.

**Located, not fully traced/ported:** the bounded
[0Y5.A00 method](apk/2.26.37.71-group-keep-outbox/keep-store.txt) contains native
keep permission, non-ephemeral parent, after-read expiry, member-exit and
existing/new timestamp branches; its keep section starts near `063c`.
The earlier orphan/revoked handling is near `00f0..01b8`. These branches were
located while tracing the timestamp writer, not proven as a complete permission
or expiry specification. 0Z0.A09's write is a cursor field-load path, not the
outgoing creation-time producer. Other field writers Bno.AFm and D1t.Byz remain
untraced. BnF has no matching own <init> in the scan; the action invokes the
retained 1iZ constructor.

Native keep application/conflict ordering, expiration scheduler, member/author
permissions, remote acceptance, adjusted-clock synchronization, full own-device
sync and handset keep rendering remain **unknown/unported/unverified**.
The supported encoder/dispatch does not establish that expired messages are
restored or that every native keep policy is enforced. Server ACKs and group
handset behavior require a separately authorized fresh group test.

**Confirmed validation:** 784 workspace/all-feature tests passed (656 API,
4 property, 9 crash, 51 CLI, one API-library symbol and 63 xtask tests), along
with strict all-target/all-feature Clippy, Rustdoc with warnings denied, default
CLI tests, API/library-symbol checks, CLI-reference regeneration, formatting and
CLI build. Re-export/check reproduces 33 topics / 38 files / 259,783 bytes.
Checksum-verified cached self-diff reports every topic/artifact unchanged in
0.017053 seconds on this machine; new-release extraction/re-anchoring speed
remains unmeasured. Offline incoming-undo and outgoing keep dry-runs reported
no enqueue/transmission; missing incoming participant refused. Saved peer keys,
actions, receipt ACKs, outbox, events and retained inspection stayed unchanged,
with 492 original captures.

**Confirmed hygiene:** 1,237 local links and 20 APK checksum bundles totaling
6,448,292 bytes pass repository checks.

**Observed receive validation:** the completed 45-second receive-only window
authenticated and stopped cleanly with sends and reconnect disabled. Peer keys,
actions, receipts, outbox and events stayed unchanged; original captured records
were retained and captures grew from 492 to 497. Both retained group reaction
targets remain valid; no retained group keep payload was available. This checks
the upgraded runtime and saved state, not fresh group keep acceptance or handset
rendering. The first local launch overlapped an offline inspection and was refused
by the store ownership guard; its logs were preserved. Inspection completion was
verified before the single completed receive window. No account reset, ownership
bypass, fresh group draft or keep/unkeep transmission was performed.

## Undo-keep edit value (2026-10-13, retained evidence)

The native keep action `11x.A01` stores edit value 7 on the keep add-on when
the target message is one's own and the action is an undo, and 0 otherwise;
receivers reject a nonzero edit value only on KEEP. This client's `keep
--undo` sends no `edit` attribute. Peers do not check it on UNDO, so behaviour
is not expected to differ, and the stanza attribute is still inferred; it is
recorded as a divergence, not changed.
[Trace](unknowns.md#outgoing-revoke-and-edit-2026-10-13).
