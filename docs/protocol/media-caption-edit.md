# Native ordinary media caption edits

Pinned APK: WhatsApp **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.

Authenticated held own-device sessions now project supported image, ordinary
video/GIF and document caption edits while preserving the original attachment
reference. This also corrects the earlier image-caption removal claim: ordinary
edit decoding requires a nonempty caption. Empty or absent captions decline.

## Native decoder and context

The [18-topic recipe](apk/recipes/2.26.37.71-media-caption-edit.json) retains full
signatures, register flow and catch tables in the
[reproducible bundle](apk/2.26.37.71-media-caption-edit/targets.json).
Complete REA Evidence independently retains the
[image](rea/2.26.37.71-media-caption-edit/image-builder.json),
[video](rea/2.26.37.71-media-caption-edit/video-builder.json) and
[document](rea/2.26.37.71-media-caption-edit/document-builder.json) builders.
JADX duplicates several blocks; exact DEX branches in the bundle disambiguate
them. Static evidence does not establish Android execution or full native policy.

`D9B.CF4` sets replacement `CoT.A00=1`. `CiA.A03`, `Cmr.A05` and `CSF.A01`
take their caption-only branch when edit mode is one and the extra context flag
is false. Each rejects an empty caption with `Bp0.A00`; whitespace is nonempty.
The raw image/video/document caption fields are respectively **3 / 7 / 20**.
They do not require replacement attachment keys on this ordinary edit branch.

`CoT.A07` returns `A0V || A0W`. `CRD` initializes corresponding `A0M/A0O`
to false. `Cna.A01` builds the ordinary received context without changing them;
`CRD.A00` maps them into CoT, and `CoT.A05` preserves them when replacing the
Message body. Authenticated envelope ownership resides in its key and does not
enable these extra context flags. Quoted/history and other extra-context paths
remain separate; this implementation does not grant their bypass to own messages.

`StringUtils.A0F` caps strings by **65,536 Unicode code points**, not bytes or
UTF-16 units. The existing payload inspector's 64 KiB byte bound is stricter,
so accepted edit captions cannot reach that native cap. Large Unicode payloads
still need a separately verified receive-bound expansion; no larger frame
limits or pretend truncation are introduced here.

## Original attachment and native kinds

`152.A00` clones original media and copies caption through `1iB.Ao1/A0s`.
For documents it additionally copies **caption** through `72p.A0w/A0x`.
`CSF.A01` shows that `Ao1/A0s` holds document title, `A0w/A0x` holds document
caption, and `CTi` sets filename. The clone/edit block does not copy filename.
The existing API projects document caption and preserves its original filename;
document title is not yet modeled as a public event field.

At `0b76..0bd6`, ordinary kinds must agree, with one explicit exception:
replacement video **3** may edit original GIF **13**. The reverse is rejected.
The retained constructors establish image **1**, video **3**, GIF **13** and
document **9**. `D9C.CF4` excludes GIF and view-once; `D9F.CF4` selects GIF from
`BZE.gifPlayback` (field **8**). The receiver reads the original synchronized
payload's subtype instead of inferring it from generic public `MediaKind::Video`.
PTV has a separate native type and remains unported here.

Attributed local image/document intents retain their known kind and reference.
Generic local video references do not prove video/GIF/PTV subtype, so this path
declines them until their saved wire kind can be resolved. Missing, ambiguous,
cleared or otherwise unsupported parents are not manufactured or fetched.

## Preservation and verification

Account/source/recipient, held identity/session, chat attribution, terminal and
sender-time ordering guards remain active. Expiring, quoted, view-once, mixed
and special replacements decline. Accepted edits change only projected caption:
replacement URL/key/hash and document filename do not select or replace the
original attachment. The original reference remains resolvable through existing
media list/open/export and clear paths. Parent bytes, historical events, identities,
send facts, IDs, idempotency records and server waits remain unchanged.

The regression initially failed with an empty image caption accepted. The
corrected decoder now rejects missing/empty captions across all three families
while preserving whitespace and Unicode within the existing bound. An eight-case
encrypted-store projection matrix verifies same-kind edits, both GIF/video
directions, image/document mismatches, original selectors and filename preservation
without writing any table. The existing image reference test now refuses removal.
The real Signal regression rejects an authenticated empty-caption wrapper with
all saved tables/journal unchanged and no sender confirmation, then continues
normal receive through the skipped message. Supported image edit receipt,
confirmation, reference binding, ordering, reopen and clear tests remain effective.

Fresh live video/document edits, native media hooks, modeled document title,
PTV/audio/sticker and special/orphan/expiry lifecycles, peer property 2983 and a
second APK comparison remain unfinished or unverified. Historical events from
the earlier permissive build are not erased or automatically reclassified.
No live account operation ran for this change; no ban-immunity claim follows.

Validation passed 969 workspace/all-feature tests, strict all-target Clippy,
warnings-denied Rustdoc, the real Signal receive regression with TCP/Noise/Signal
features, 26 default CLI/API-symbol tests and the experimental CLI build.
The 18-topic APK bundle reproduced all 23 files / 240,483 bytes exactly.
