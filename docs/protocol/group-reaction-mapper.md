# Ordinary group reaction mapper

2026-10-09. **Confirmed static** for APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [hash-pinned recipe](apk/recipes/2.26.37.71-group-reaction-mapper.json)
exports an [18-topic bounded bundle](apk/2.26.37.71-group-reaction-mapper/targets.json).
These are structural disassemblies, not recovered compilable Java. Method offsets
below are bytes. The [definition/field/caller inventory](group-reaction-mapper-inventory.txt)
records searches as well as the limits of their interpretation.

## Own-user predicate

A BO0 method-definition scan across the APK returned exactly three definitions:
abstract 08Z, [08a forwarding](apk/2.26.37.71-group-reaction-mapper/own-forwarder.txt)
and [08i.BO0](apk/2.26.37.71-group-reaction-mapper/own-user-predicate.txt).
08a delegates at `0000..000e`. 08i returns false for null; calls A09, compares the
participant with `A0I:PhoneUserJid` at `000a..001a`, then calls A04 and compares
`A0H:0ZY` at `001a..0030`. Either match returns true. This **confirms** the prior
own-PN/LID interpretation of the immediate predicate; it does not establish
signed device membership, a directory alias or the current values of its fields.

[Jid.equals](apk/2.26.37.71-group-reaction-mapper/jid-equality.txt), `0010..0058`,
compares user, server and getType. [DeviceJid.equals](apk/2.26.37.71-group-reaction-mapper/device-equality.txt)
additionally compares runtime class, device and embedded UserJid. Equal numeric
PN/LID users are therefore not an own-identity association. Our supported
ordinary PN/LID subset uses exact known own user/server identities and never a
contact/directory alias. Full native factory/subtype equality remains unported.

[08i.A04](apk/2.26.37.71-group-reaction-mapper/own-lid-loader.txt) reads
`self_lid` preferences (`0026..0048`), accepts a LidUserJid or extracts one from
a LidDeviceJid (`0062..0098`), and also examines Me.lid (`00aa..00b6`). It carries
coherence/fallback branches. The native current preference/Me values and the
complete A09 phone producer are **unknown**. Rust uses the existing authenticated
login-identity checkpoint; it does not fabricate preferences or Android state.

## Ordinary non-own envelope and group branches

[Bal.A01](apk/2.26.37.71-group-reaction-mapper/incoming-caller.txt), `01dc..0256`,
passes the source `CoT.A06`, account interface, envelope key, reaction key,
`0hd.A02(parsed participant)` and literal false as BoQ's arguments. The own-envelope
key boolean at `0206..020e` nulls the source argument through `025e..0262`.

[CoT constructor](apk/2.26.37.71-group-reaction-mapper/envelope-constructor.txt)
assigns its first reference argument to A06 (`001e..0022`). The
[CRD builder](apk/2.26.37.71-group-reaction-mapper/envelope-builder.txt) forwards
CRD.A02 through v23/v17 (`0034..003c`, `0108..010c`, `0130..0136`). The inventory
found one A06 write, six constructor call sites and seventeen CRD.A02 writes.
**Unknown:** all source-producer operand roles and canonicalization. A first
constructor argument is not itself proof of raw stanza participant authority.
The Rust projection supplies the already authenticated group sender and remains
scoped to ordinary incoming group references, not full native own-envelope sync.

[1bn.getType](apk/2.26.37.71-group-reaction-mapper/group-type.txt) returns 1.
[0DE.A0m](apk/2.26.37.71-group-reaction-mapper/group-predicate.txt) accepts type
1 or 2 after excluding type 28 via
[A0W](apk/2.26.37.71-group-reaction-mapper/group-predicate-exclusion.txt).
[A0i](apk/2.26.37.71-group-reaction-mapper/status-predicate.txt) accepts type 5.
Thus a normal group takes BoQ's group branch. The caller's v11 participant-kind
gate is not needed to select a non-own third author in that branch; full
non-group/hosted gate behavior remains outside this port.

[BoQ.A00](apk/2.26.37.71-group-reaction-mapper/ordinary-target-mapper.txt) starts
with a nullable UserJid participant. For a normal non-status envelope it selects
the envelope chat (`00fe..0106`). The group predicate sets v1 at `00a0..00a2`.
With a source argument present, the supported present-author cases are:

| Payload fromMe | Participant | Native path / Rust target |
|---|---|---|
| absent/false | known own PN/LID | BO0 true (`00dc..00e8`), native key boolean true, no author: outgoing |
| absent/false | another ordinary PN/LID | BO0 false, group selects participant (`00e8..00f6`), native key boolean false: incoming author |
| true | present ordinary PN/LID, including own | group branch `00ce..00dc` selects participant, reaches `00b8` and clears native key boolean: incoming author |

**Confirmed correction:** the previous helper classified a true-fromMe key with
an own author as outgoing without following that branch. It now invokes own
classification only for absent/false fromMe. This also removes its unnecessary
own-checkpoint requirement on the true/present-author branch. Tests preserve the
literal flag and author combinations independently of the payload builder.

## Nullable author and representation limits

[0D8.A02](apk/2.26.37.71-group-reaction-mapper/nullable-user-parser.txt) returns
null for empty/absent or a caught parse failure.
[A01](apk/2.26.37.71-group-reaction-mapper/required-user-parser.txt) requires an
actual UserJid (`000c..001e`); it does not normalize a DeviceJid into one.
**Confirmed correction:** nonzero device-qualified reaction authors now report
`unsupported_participant_device` rather than being normalized into own-user
comparison. Actual device routing from a selected saved parent remains intact.

**Negative / local parser boundary:** the Rust textual parser collapses an
explicit `:0` suffix to a plain JID. A fixture that expected it to remain a
device failed for that reason. Native factory handling of zero-device text has
not been traced; the public helper does not claim raw-form or subtype parity.

**Confirmed static, unported:** true fromMe with a null participant can select
the source through `00d6..00b8`; false fromMe with null can produce a key without
an author. The current helper refuses missing/unrepresentable authors. Full
source-producer authority, native own-envelope mapping, hosted participants,
nullable-author reconciliation and status targets remain **unknown/unported**.
**Confirmed correction:** the prior exact payload/envelope-group guard was a
local limitation. BoQ's ordinary branch selects the envelope chat, so the helper
now accepts supported plain PN/LID/group remote keys while keeping the update
in the authenticated envelope group. It never adopts the payload's other chat
or borrows its parent body. Status and unsupported ChatJid subtypes stay
separate (`unsupported_group_remote_jid`). Mandatory participant requirements
and unported nullable-author fallback remain local support limits, not native
C7G validation.

## Reproduction and validation

`xtask apk-smali APK --class '' --method BO0 --exceptions` reproduces the
method-definition inventory. `apk-field-refs` with writes-only records A06 and
CRD.A02, and `apk-callers` records CoT constructor callers. The recipe reproduces
selected bytecode, register operands and exception tables without committing
raw APKs, media or bulk trees. Cached research diff can compare these named topics
without scanning either APK again.

**Confirmed offline:** literal descriptor-grounded fixtures cover native
true/false/default flag decisions and device-author refusal. Projection fixtures
cover legacy/new PN/LID parents, consumed body preservation, actual parent devices,
own true/false references, equal numeric users across distinct PN/LID
namespaces, ordinary remote-chat substitution without retargeting, immutable
outbox records, duplicate-claim rollback and once-only event markers across reopen. Fresh live group targeting and native
current account preference values remain **unverified**.

**Confirmed static / remaining producer boundary:** BoQ's envelope-own key
boolean at `0076..007e` takes `00f6..00fe`, preserving the supplied reaction
fromMe flag and participant instead of the ordinary remote-group decisions
above. Its native branch is now known; producing and exposing that authenticated
envelope-own boolean in the Rust receive path is still unported. Do not infer
that the current helper has full own-device echo parity merely because BO0 is
traced. Native g.us-to-1bn selection is pinned by
[0D1.A00](apk/2.26.37.71-reaction-key/jid-parser.txt), `012a..0144`.

**Confirmed verification:** all 778 workspace/all-feature tests passed, as did
strict all-target/all-feature Clippy, Rustdoc with warnings denied, default CLI
tests, API/library-symbol checks, CLI-reference regeneration and formatting.
The final API description edits were followed by five passing OpenAPI tests
and an experimental CLI rebuild. Export and checksum revalidation reproduced
23 files / 86,053 bytes. Repository hygiene validates 16 APK bundles totaling
5,901,337 bytes. A cached checksum-verified self-comparison returned all eighteen
topics and four metadata artifacts unchanged in 0.008 seconds. This measures
cached self-diff only; it does not benchmark extracting or re-anchoring a new
APK release. No new release was compared in this change.

**Observed receive validation (2026-10-09):** two bounded 45-second windows
authenticated and stopped cleanly; the second ran the final ordinary-remote
correction. Both disabled chat sends and automatic reconnect. Original captured
records were retained; capture rows grew from 471 to 481. Peer keys, actions,
receipts, outbox and durable events were unchanged. Two retained group reaction
keys still inspected as target_validation=valid. These are retained-key and
receive-runtime checks, not fresh live group reaction/own-envelope proof.
