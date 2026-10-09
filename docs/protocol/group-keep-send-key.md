# Outgoing keep original-message key

2026-10-09. **Confirmed static** for APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [22-topic recipe](apk/recipes/2.26.37.71-group-keep-send-key.json)
reproduces [the selected methods](apk/2.26.37.71-group-keep-send-key/targets.json)
with register flow and exception tables. This extends the
[incoming keep evidence](group-keep.md); it adds no send capability.

## Parent key and serialized fields

**Confirmed:** [Bai.AD7](apk/2.26.37.71-group-keep-send-key/keep-outbound.txt)
requires a BnF message. `0016..0026` calls 1iZ.A0s and A0r. Their
[parent-key](apk/2.26.37.71-group-keep-send-key/parent-key.txt) and
[parent-author](apk/2.26.37.71-group-keep-send-key/parent-author.txt) accessors
read CUq.A01 (1gc) and CUq.A00 (0Cy), respectively, from the 1iZ.A05 container. Both return
null when that container is absent. The values are the parent reference carried
by the keep message, rather than the new keep envelope's key.

Bai.AD7 passes author, parent key and BZa builder to
[CSZ.A01](apk/2.26.37.71-group-keep-send-key/key-serializer.txt) at `007e..0084`.
The next argument is false; the final argument is Cn6.A09. It then assigns the
built key to BV3.key, chooses KEEP_FOR_ALL or UNDO_KEEP_FOR_ALL, and copies
BnF.A02 to timestampMs. The [existing descriptor evidence](group-keep.md#protocol-construction)
confirms Message field 51 and BV3 fields key=1, keepType=2, timestampMs=3.
The native timestamp producer was not traced at this checkpoint. The subsequent
[normal keep outbox](group-keep-outbox.md#closed-static-branches) identifies
11x.A01 / 08B.A00 and preserves original creation time; adjusted-clock parity
remains unported.

CSZ preserves 1gc.A01 as id (`0080..008a`) and 1gc.A02 as fromMe
(`008a..0094`). The corresponding BOE setters assign
[id_](apk/2.26.37.71-group-keep-send-key/key-id-set.txt) and
[fromMe_](apk/2.26.37.71-group-keep-send-key/key-direction-set.txt), with their
presence bits. The author setter assigns
[participant_](apk/2.26.37.71-group-keep-send-key/key-author-set.txt) at
`0022..0026`; it is not a remoteJid setter. The
[remote wrapper](apk/2.26.37.71-group-keep-send-key/key-remote-wrapper.txt)
uses getRawString and delegates to the
[remoteJid_ setter](apk/2.26.37.71-group-keep-send-key/key-remote-set.txt).
Neither serializer branch changes fromMe according to whether an author was
found.

## Author selection branches

**Confirmed:** CSZ.A01 reads config selector `0x2ffb` through 00D.A10 at
`0006..0016`. Its effective value/default is **unknown**. With that gate off,
`00d4..00de` selects the later inclusion branch from the parent's fromMe;
the supplied author is retained. For an ordinary group, the inclusion checks
allow a nonnull author for both incoming and outgoing parent keys. A null
author is omitted. Normal g.us type 1 and the 0DE.A0m type predicate are
[separately established](group-reaction-mapper.md#ordinary-non-own-envelope-and-group-branches).

With the gate on, a supplied author or an outgoing group parent enters
`003a..0066`: the author is resolved again from the parent. Ayo.A0R delegates
to [1hQ.A02](apk/2.26.37.71-group-keep-send-key/parent-lookup.txt), which returns
a selected message's key. 1xo.A0T then looks up that key through
[0V9.Aon](apk/2.26.37.71-group-keep-send-key/message-lookup.txt). If that message
lookup returns null, `0056..005a` branches to `00d0..00d4`, clearing the author.
Thus even a caller-supplied nonnull author is not guaranteed to survive. This
is native omission behavior, not evidence that a server accepts an omitted
group author.

The parent normalizer first tries the message store and then
[1hQ.A00](apk/2.26.37.71-group-keep-send-key/parent-edit-fallback.txt).
The latter queries message_edit_info joined to message by original key ID,
chat row and fromMe. If both searches miss, 1hQ.A02 looks for `__part__` at a
positive position in the parent ID, constructs a key from the preceding ID
substring with the same chat/fromMe, and repeats those two lookups. It returns
the found message's key, or null. This does not authorize stripping `__part__`
from an outgoing wire ID or changing a saved Rust request ID.

[0VA.Aon](apk/2.26.37.71-group-keep-send-key/message-store.txt) checks a cache
through A01 before querying message by chat_row_id, from_me and key_id, using
moveToLast. The selected method does not add a participant SQL predicate.
**Unknown:** cache selection, cursor-to-message conversion and complete native
collision semantics; A01 and A05 were not traced in this chunk. This SQL is
not a reason to weaken Rust's authenticated participant-scoped parent guard.

For a found parent, [0Z1.A01](apk/2.26.37.71-group-keep-send-key/selected-author.txt)
returns incoming message B1V directly. For an outgoing parent it first reads
D5q's DeviceJid through
[C9r.A00](apk/2.26.37.71-group-keep-send-key/device-origin.txt), returns its
UserJid when present, then tries
[0Z1.A00](apk/2.26.37.71-group-keep-send-key/author-device.txt). That fallback
reads author_device_jid from message_details by message row ID and resolves
it through 0VT.A09 and 0DK.A00. If no device is resolved, A01 calls the account
interface's CMn. **Unknown:** CMn's effective PN/LID namespace and the complete
incoming B1V producer for every message kind. No alias, account identity or
device value was fabricated from these interfaces.

## Remote key and remaining boundaries

**Confirmed:** CSZ.A00 runs before writing remoteJid (`0068..0080`). With the
final Cn6.A09 argument false it returns the original parent remote. With it
true, it invokes 0Yo.A02 and uses a nonnull result, otherwise falling back to
the original remote. [0Yo.A02](apk/2.26.37.71-group-keep-send-key/remote-context-wrapper.txt)
constructs 1wk selector 2 and invokes it. **Unknown:** selector 2's mapping and
the producer of Cn6.A09; the lambda was not traced at this checkpoint. The subsequent
[normal keep outbox](group-keep-outbox.md#closed-static-branches) proves that
ordinary group remote keys are unchanged; full non-group mapping and the context
producer remain outside that port. A nullable mapping wrapper alone is not
evidence for choosing an arbitrary PN/LID alias.

**Inferred implementation direction:** ordinary group keep sends need an
original participant, independent of the keeper, and frozen own identity for
supported outgoing targets. Existing retained-parent guards can supply those
facts without copying, opening or consuming media. Native optional lookup and
namespace selection must be resolved or explicitly bounded before claiming
serializer parity. These are research conclusions, not implemented behavior.

**Historical local limitation at this research checkpoint:** the direct Rust
keep encoder omitted participant, and the normal group adapter refused
KeepInChat. The subsequent [normal keep outbox](group-keep-outbox.md) now provides
supported group dispatch through a separate codec; native policies remain open. APK send permission/expiry checks, edit conflict
handling, native timestamp ordering, outgoing stanza attributes, complete
namespace normalization and live group keep/unkeep remain **unknown/unported**.
No chat message, group draft, registration request or account mutation was
needed for this static investigation.

## Reproduction and verification

Existing offline CLI commands expose the research without account access:

```sh
cargo run --locked -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk --recipe docs/protocol/apk/recipes/2.26.37.71-group-keep-send-key.json --output docs/protocol/apk/2.26.37.71-group-keep-send-key --check
cargo run --locked -p xtask -- apk-research-diff docs/protocol/apk/2.26.37.71-group-keep-send-key docs/protocol/apk/2.26.37.71-group-keep-send-key
```

The recipe maps each topic to chat_settings.rs, group_references.rs and
outgoing_group.rs. Cached topic diffs therefore identify the affected Rust
files. A new APK still requires its own hash and class/method re-anchoring;
this self-comparison does not measure new-release extraction or prove a
cross-release protocol change. No API/CLI capability changed in this research
chunk; the existing offline exporter/differ is the reproduction surface.

**Confirmed verification:** re-export/check reproduces 22 topics / 27 files /
75,556 bytes, and the checksum-verified self-diff reports all topics and four
metadata artifacts unchanged. All 63 xtask tests, 50 experimental CLI tests
(including CLI-reference regeneration and OpenAPI checks), and the API-library
symbol test passed. Hygiene checked 1,206 local links and 19 APK checksum
bundles totaling 6,188,509 bytes. No account connection or send was performed.
