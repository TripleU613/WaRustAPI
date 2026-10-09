# Incoming group keep/unkeep references

2026-10-09. **Confirmed static** for APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [six-topic recipe](apk/recipes/2.26.37.71-group-keep.json) reproduces
[bounded register/exception evidence](apk/2.26.37.71-group-keep/targets.json).
The [field/descriptor inventory](group-keep-inventory.txt) preserves the
field-reference search and independent number extraction.

## Protocol construction

**Confirmed:** `apk-proto-fields` decodes BZc.dynamicMethod's descriptor as
keepInChatMessage=51. BV3.dynamicMethod encodes key=1, keepType=2 and
timestampMs=3. C2L.<clinit> constructs UNKNOWN=0 (`0000..0014`), KEEP_FOR_ALL=1
(`0014..0028`) and UNDO_KEEP_FOR_ALL=2 (`0028..003c`). These are descriptor/enum
values, not guessed positions in the field-name array. BZa's key layout is
pinned in [reaction-key evidence](apk/2.26.37.71-reaction-key/targets.json).

The six BZc.keepInChatMessage_ field references locate outbound Bai.AD7,
incoming D9L.A01 and two SendE2EMessageJob methods. In D9L.A01, the Bai branch
begins at `055e`; it tests the keep field's presence (`0564..0574`). Its
edit-version/KEEP_FOR_ALL conflict checks are `0574..05cc`; those edit paths
are not implemented by the ordinary field-51 receive projection.

**Confirmed:** D9L.A01 validates the parent key through C7G (`05cc..05e4`),
constructs BnF using the envelope key/time (`05f0..060c`), then initially
constructs a target using raw remoteJid, payload fromMe and nullable author
(`060c..0656`). It records keep type/time at `0656..0682`.

It then reads CoT.A06, the account interface and the envelope own flag
(`0682..0692`). An own envelope nulls the source (`06b8..06bc`); both paths
call BoQ.A00 with BV3.key and false/false final gates (`0696..06b0`) and
replace the initial parent target (`06b0..06b6`). The explicit author remains
independent of the keeper. For normal groups, BoQ's group branch does not
depend on the hosted-participant gate used by the reaction caller.

The [group mapper](group-reaction-mapper.md) and
[network envelope producer](group-envelope-own.md) establish ordinary
group target scope, exact known own PN/LID classification, own-envelope flag
production and true/null-author fallback. Those supported rules also apply
here because the incoming keep handler calls the same BoQ method with the
same group key/source context. A target's presence does not establish keep
permission or downstream acceptance.

## Rust behavior

**Confirmed local correction:** prior field-51 projection used the direct chat
as both keeper and target author. In a group this could fail to construct a
DeviceAddress from the group JID, leaving a supported keep update opaque;
it also could not identify a third member's message. The new
`chat_settings::group_keep_target` passes the actual BZa key to the shared
group mapper without constructing a substitute reaction payload.

Group event projection now distinguishes the target author and actual keeper.
An authenticated remote sender with true fromMe and absent/empty author falls
back to its user identity; false/default requires an explicit supported author.
Known own senders retain payload direction, including false references to an
explicit incoming author. Status, hosted, agent/device-qualified target authors
and unsupported sender agents stay opaque. Unknown keep type is a local
unsupported condition; this does not claim the native handler rejects every
unknown enum in the same way.

The shared group parent resolver retains canonical/legacy participant scope,
matching original devices and exact-group outgoing intent provenance. It never
copies or opens a parent body. Parent ambiguity or conflicting outgoing group
scope refuses the write atomically. Mixed field-46/51 or other top-level content
is not projected by choosing one update arbitrarily. Previously published
MessageUpdated markers remain historical; existing unsupported markers may
upgrade once through the existing projection mechanism. Direct keep behavior
retains its existing mapping.

`lab-inbox --inspect` reports group keep target_validation with fixed redacted
labels, loading and validating the original saved route/capture before deriving
sender context. Inspection does not resolve parents, authenticate ciphertext,
advance ratchets, apply keep state or repair old event markers. Unsupported
target errors remain diagnosable without printing keys, addresses or content.

**Unknown/unported:** native downstream keep authorization and parent-expiry
enforcement, edit-version keep handling, hosted/status targets, full own-device
synchronization and native zero-device textual subtype parity. A durable
KeepInChat event is not proof that an expiry scheduler or remote handset has
applied it. Fresh live group keep/unkeep and handset rendering remain unverified.

**Subsequent confirmed local send implementation:** the normal group actor
now accepts supported KeepInChat references with the original participant and
sender-relative fromMe. The direct encoder remains separate. See the
[outgoing-key trace](group-keep-send-key.md) and
[normal keep outbox](group-keep-outbox.md) for exact native evidence, invocation,
commit-time reference guard and remaining permission/clock/expiry limits.
Fresh live group keep/unkeep and handset rendering remain unverified.

## Verification and negative results

**Confirmed offline:** literal BV3/BZa descriptor fixtures exercise keep/unkeep,
third-member attribution, null-author fallback, own-envelope direction, unknown
types and missing identity/author refusals. Twenty-four reaction/keep projection
cases cover remote/own keepers, original PN/LID parents, actual stored devices,
consumed body/outbox preservation, exact group scoping and once-only markers
after reopening the store. Existing shared group fixtures retain ambiguous
parent refusal and unknown original-route behavior.

The earlier `BoJ` candidate scan contained no keep/BV3/BoQ references; it did
not establish a keep receive path. The exact BZc field scan located D9L instead.
Do not infer a required participant from the descriptor alone: the supported
sender-context rules determine when missing authors can be resolved. Native
later permission/expiry handling was not traced here and remains unknown.

**Confirmed local representation limit:** IncomingKeep and the KeepInChat
message change omit the payload timestampMs. The event's server timestamp
comes from stanza `t`; the original protobuf remains in encrypted saved state.
Native timestamp-based conflict ordering/application has not been traced or
ported. Do not describe projected keep/unkeep facts as full expiry enforcement.

**Confirmed verification:** 780 workspace/all-feature tests passed (653 API,
4 property, 9 crash, 50 CLI, 1 API-symbol and 63 xtask tests), together with
strict all-target/all-feature Clippy, Rustdoc with warnings denied, default CLI,
API/library-symbol checks, CLI-reference regeneration and formatting. Export
and checksum revalidation reproduce six topics / 11 files / 84,246 bytes.
Repository hygiene validates 18 APK bundles totaling 6,112,953 bytes.

**Observed receive validation (2026-10-09):** one bounded 45-second window
authenticated and stopped cleanly with chat sends and automatic reconnect
disabled. Original captures were retained; capture rows grew from 487 to 492.
Peer keys, actions, receipts, outbox and durable events stayed unchanged. Two
retained group reaction targets remain valid; no retained group keep payload
was available to validate live attribution. No fresh group keep/unkeep, group
send or human handset rendering is claimed. Native keep timestamp ordering,
authorization, expiry enforcement and outgoing group keep remain open gaps.
