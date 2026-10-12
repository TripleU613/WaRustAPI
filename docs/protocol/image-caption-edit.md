# Own-device image caption edits

Pinned WhatsApp APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.

Supported authenticated own-device image edits now emit the existing
`MessageUpdated` / `Edited` event with the original image reference and edited
caption. An absent caption removes it. This extends
[ordinary own-device text edits](message-edit.md); it does not send edits.

## APK evidence

The [six-topic recipe](apk/recipes/2.26.37.71-image-caption-edit.json) retains
full signatures, register flow and catch tables for the manager, cloning
subsystem, caption accessors, edit writer and image descriptor.
[The bundle](apk/2.26.37.71-image-caption-edit/targets.json) can be regenerated
and checked when comparing releases. Complete REA records retain the
[clone](rea/2.26.37.71-image-caption-edit/clone.json),
[caption getter](rea/2.26.37.71-image-caption-edit/caption-getter.json),
[caption setter](rea/2.26.37.71-image-caption-edit/caption-setter.json) and
[edit writer](rea/2.26.37.71-image-caption-edit/writer.json).

In `152.A00` at `088c..08d2`, the media handler casts original and edit to
`1iB`, clones the original through `2vT.A00`, then reads `Ao1` from the edit and
writes it to the clone with `A0s`. `Ao1` reads caption `A03`, synchronizing with
media metadata `6b5.A0V`; `A0s` replaces both. The clone therefore retains the
original media attachment. Subsequent native media hooks are visible in the
retained manager but are not fully reconstructed here. `D9B.A00` writes the
replacement Message into protocol field 14 and sender timestamp into field 15;
ordinary/future-proof wrapper selection depends on property 8784.

These are static observations, not runtime proof of the Android client or
complete native media edit behavior.

## Implementation and limits

The existing wrapper, held session/pin, account generation, source/recipient,
parent attribution, terminal and sender-time ordering checks remain in force.
Only ordinary image-to-image caption updates use this extension. A text/image
kind mismatch stays opaque. View-once, mixed, quoted and expiring replacements
decline, as do unloaded/cleared parents. Other media edits require their native
type and metadata paths. The existing HTTPS media feature is required; the
TCP/Noise/Signal-only build retains text-only edit support.

The receiver loads the original supported parent content, preserving its sealed
reference and filename, and replaces only the caption. Edited URLs, hashes and
keys do not choose another attachment. A synchronized parent's reference stays
bound to its original checkpoint; list/open/export continue to use the existing
ordinary-media selector and clear behavior. No new media download, event schema,
configuration option, send intent or send outcome is introduced. Raw authenticated
edit wrappers remain sealed in their own receive checkpoints. Parent bytes,
historical events, account keys, IDs, idempotency records and waits are unchanged.

The reference test sets and removes a caption with conflicting replacement
attachment metadata, verifies the original selector and unchanged parent row,
and checks that listing exposes one attachment and clearing invalidates it.
The real Signal regression receives an image edit, confirms and deduplicates it,
resolves its reference to the original image, and continues the receive ratchet.
Literal guard cases cover view-once, quoted, mixed and malformed replacements.

Fresh live caption-edit delivery, peer configured window 2983, media hooks,
video/document captions, special/orphan/expiry lifecycles and a second APK release
comparison remain unfinished or unverified. No live account operation ran for
this change, and these checks do not establish ban immunity.

Validation passed 967 workspace/all-feature tests, strict all-target Clippy,
warnings-denied Rustdoc, the real Signal regression with TCP/Noise/Signal only,
56 all-feature and 26 default CLI/API-symbol tests, and the experimental CLI
build. The six-topic APK bundle reproduced all 11 files / 159,517 bytes.
The generated CLI reference is current; its generator now emits one final newline.
