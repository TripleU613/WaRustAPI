# DeviceSentMessage admission, destination and context

2026-10-11. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-device-sent-wrapper.json),
[DEX signatures/register flow/exception tables](apk/2.26.37.71-device-sent-wrapper/targets.json),
[complete REA validator envelope](rea/2.26.37.71-device-sent-wrapper/wrapper-validation.json),
and [exact-field reference census](rea/2.26.37.71-device-sent-wrapper/wrapper-field-references.txt).
Offsets below are method-relative bytes. This extends
[recipient routing](recipient-routing.md); it does not establish complete
own-device receive or transmission support.

## Wire shape and outer context

The generated descriptors, independently decoded by `apk-proto-fields`, give
Message/BZc.deviceSentMessage **31** and BUs fields destinationJid **1**,
message **2**, phash **3**. Presence matters: BUs bits 1 and 2 distinguish
an absent destination/body from a present empty value. The reference census
contains 14 exact field accesses; the individual methods below are traced,
not every downstream consumer in that census.

`Cla.A01` constructs the wrapper: a nonnull message supplies BUs.message and
bit 2; nonnull destination and phash strings supply bits 1 and 4. It retrieves
the original Message context, clears it in the nested clone through `CmC.A07`,
and attaches it to the outer Message. CmC clears the context field and its
presence bit, not the entire nested message.

`CjY.A00` parses Message and unwraps field 31 once, selecting default empty
messages when the wrapper or inner object is null. `CjY.A01` instead returns
both the outer and selected inner Message. If the outer context exists, it
attaches that context to the inner builder before returning. This is one-level
unwrapping; recursive diagnostic inspection is not evidence of recursive native
admission. It cannot be replaced by simply discarding the outer Message.

## Own-device and wrapper gates

`08i.BP3(device)` is true only for nonnull device, BO0-own user, and a device
number different from Apt(). Apt reads saved `registration_device_id` with
default zero. BP3 is not just BO0, a nonzero-device test, or `category="peer"`.
BO0's stored PN/LID authority is established in [own identity](own-identity.md).

`Cm3.A07` obtains Bp2's normalized key chat, or the other stanza's normalized
remote chat. Its ordinary-type predicate `0DE.A0l` accepts native JID types
0, 11, 18, 22, 26 and 7. In that branch:

- BP3 of the actual `CjN.A00` source must be true.
- A present outer protocol message with any tested exemption bit makes A07
  false: bitField0 masks 0x20, 0x40, 0x100, 0x80, 0x10, 0x1000, 0x2000,
  0x40000, 0x80000, or bitField1 masks 1 and 2. The names and full semantics
  of these exceptions remain untraced here; they must not be collapsed into
  a generic exemption for every protocol message.
- If CjN.A04 passes `0DE.A0Q` (types 3, 5 or 6), A07 returns wrapper presence;
  otherwise it returns true after the preceding checks.

The type-3 branch (`0DE.A0R`) requires BP3 and wrapper presence. A separate
0c3 branch derives a device from CjN.A04, requires BP3, absent A07 encryption
holder and present A06 encryption holder. Those special JID/holder semantics
are retained but not ported. Other branches return false.

`Cm3.A02` parses outer/inner through CjY.A01, then calls A07. True without
wrapper throws code **112**, `Peers must use DeviceSentMessage proto`.
False with wrapper throws code **113**, `DeviceSentMessage proto only allowed
from peer device`. It reports recovery through 0z9.A05 before those throws;
the recovery side effects are not reconstructed by this trace.

## Destination validation and lost Java branch

With a wrapper, A02 requires BUs message and destination presence bits
(`00dc..00f0`) and parses destination through 0D7.A01. For Bp2, the expected
chat is its recipient A06 when present, else the normalized key's chat
(`0180..01a2`); other stanzas use normalized CjN.A05. Equality with that chat
jumps to the successful return at `0296`. Additional success branches allow
a type-3/5/6 destination equal to CjN.A04, a converted bot destination, or
equality with `0Xw.A0I(expected UserJid)`. Bot conversion and directory alias
authority remain untraced; a port must not manufacture such equivalence.

The REA Java rendering loses this successful control flow and appears to log
and throw even after equality. The retained DEX branches at `01ce..020e`
and return at `0296..0298` resolve that rendering error. Missing/unparseable
or unequal destinations reach code **114**, `invalid DeviceSentMessage`.
Phash is optional and is not a destination-admission condition in this method.

## Implemented inspection and remaining runtime work

`signal::direct::payload_summary` now recognizes field 31, reports only the
presence of destination/message/phash, and recursively inspects nested field
numbers within the existing depth and byte bounds. It preserves absent versus
present-empty fields, validates protobuf string encoding, and prints
`admission not checked`. Destination, phash and payload values are never printed.
The existing CLI saved-inbox inspection uses this helper without a new option.
Inspection does not import keys, open attachments, resolve a destination,
reconcile an outgoing message, or enqueue traffic.

Independent hand-encoded tests cover all three present fields with hidden
values, all absent fields, all present-empty fields, invalid UTF-8, malformed
nested Message and excessive nesting. The regression failed on missing wrapper
inspection before the implementation.

[Own-device text receive](own-device-text.md) now implements the held-session,
unambiguous-recipient text subset with outer-context replacement.
[Own-device media](own-device-media.md) extends this route and attachment
retrieval; [sender confirmations](own-device-confirmation.md) are also implemented.

**Unported:** complete Cm3.A07/A02 runtime admission and recovery, effective
recipient namespace mapping, own-device session bootstrap, complete synchronized
content effects and primary echo reconciliation, and native read-self receipt
selection. Unsupported recipient-qualified
stanzas remain raw captures. No fresh live wrapper
sample or ban-immunity claim follows from this offline trace.

2026-10-11 validation: all 939 workspace/all-feature tests, strict workspace
Clippy, warning-denying Rustdoc, default CLI tests, experimental CLI build and
formatting passed. The 14-topic / 19-file / 90,304-byte bundle reproduces exactly.
Fifteen complete REA envelopes retain method renderings and class inventories;
three census files retain exact references and decoded descriptor fields.
