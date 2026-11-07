# Current video ingress and sidecar presence — APK 2.26.43.72

**Confirmed selected static / research only**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports via xtask fallback; no target execution, new REA
Evidence IDs, account traffic or Rust runtime change. Byte PCs below are method
relative; branch displacements are16-bit code units. This follows the current
[image ingress](../2.26.43.72-media-image-ingress/README.md) without treating video
and image sidecars or converters as interchangeable.

## Current schema matches Rust core video tags

[DiM descriptor](video-descriptor.txt) and [exact decoded fields](video-fields.txt)
confirm url1, mimetype2, fileSha2563, fileLength4, mediaKey6, fileEncSha25611,
streamingSidecar18, viewOnce20. Rust's Video descriptor agrees on the core fields
it consumes. Its8-MiB bound and full-ciphertext policy are local choices, not
native maximum-size or range-parity claims. The current native PTV wrapper/dispatch
is not independently traced by finding the shared video descriptor.

The [previous complete F1B.A05](../2.26.43.72-media-image-ingress/video-projector.txt)
reads the actual DiM fields. Its selected fresh/update paths require positive
fileLength at00f2..0108, a32-byte plaintext hash at0140..016a and a32-byte encrypted
hash at0176..01a0. Hashes use [DBg.A0f](hash-string-helper.txt), Base64 flag2, then
Ch4/Ch2; the prior message/model evidence carries those to the consumed download
hash fields. No new Base64 wire requirement follows. Existing update/context
predicates guard field adoption, so no universal absent-field policy is asserted.
The local parser's malformed-hash error argument14 is not a ban/retry code.

## View-once conversion is separate from common field parsing

[View-once field census](view-once-refs.txt) finds FIVE direct DEX read/write sites.
[FOr.CR7](video-converter.txt) gets the videoMessage from DiW, rejects gifPlayback,
and rejects a present-and-true viewOnce before constructing9JU. [FPf.CR7](alternate-video-converter.txt)
requires a videoMessage, false gifPlayback and present-and-true viewOnce before
constructing9JV (constructor integer43). Both call [F2D.A03](projector-entry.txt),
which calls F1B.A05 then F1B.A02. This connects common field and sidecar parsing
to these selected converter paths, not merely an uncalled utility method.

Rust's ordinary descriptor excludes true viewOnce before download and does not
use the view-once content as replayable video. Its treatment of gifPlayback is
not classified as native converter parity here: both selected converters reject
GIFs, but the native GIF converter and full dispatch order remain untraced.
Do not conclude GIFs are forbidden, or change network behavior from that bounded
negative result. No view-once viewer/ACK/receipt behavior was recovered here.

## The sidecar helper tests KEY presence, not sidecar presence

[Sidecar field census](sidecar-refs.txt) finds SIX direct DEX sites. In the
[actual F1B.A02](sidecar-projector.txt), an update-mode early return at0008..0012
precedes required8tC access. At0022..002e it checks bitField0_ &0x20, then reads
streamingSidecar bytes and calls1fW.Cdm only if those bytes are nonempty. That
mask MUST NOT be labelled a streamingSidecar-presence check.

The descriptor's auxiliary presence indexes independently establish:

| Field | Wire tag | Presence index | Mask |
| --- | --- | --- | --- |
| mediaKey | 6 | 5 | 0x20 |
| streamingSidecar | 18 | 15 | 0x8000 |
| viewOnce | 20 | 17 | 0x20000 |

[Offline checker](presence_check.py) decodes this exact retained descriptor's
integer sequence and verifies these selected indexes; [output](selected-presence.json)
is retained. It explicitly refuses continuation-word or different header forms,
rather than pretending to be a general protobuf descriptor decoder. Field names
are paired by the already retained exact apk-proto-fields output; this small
check verifies presence indexes, not arbitrary object-array name recovery.

Thus a nonempty sidecar value alone does not establish Cdm adoption in this leaf;
the key-presence bit and caller mode also matter. Conversely this is not proof
that every absent sidecar is an error: the helper skips the sidecar call and still
processes annotations. Concrete Cdm validation/storage and complete provider/range
geometry remain unknown. Image scan count/index rules must not be pasted onto
video's distinct single-byte-array setter.

## Findings and remaining work

**Confirmed:** current video core wire tags and selected hash/length validation
match the relevant Rust descriptor requirements; selected native ordinary/view-once
converters separate ownership before common projection; sidecar key/mode/payload
conditions differ from the image scan path. No confirmed runtime construction or
response mismatch was repaired in this pass. Rust change is source provenance
comments only, with no new network or storage behavior.

**Unknown:** native GIF/PTV/full envelope dispatch, caller update-mode inputs,
Cdm/provider implementation, current audio/document/sticker ingress, exact native
request profile and range/status adoption, account/cancellation/output authority
and live interoperability. Rust's200-only full-download policy remains a known
shared-leaf difference pending supported-operation binding, not silently labelled
native parity. No206/416 relaxation, guessed sidecar/header, automatic retry/resume,
account operation or restriction-cause claim is introduced.

Reproduce `apk-smali <apk> --class '<descriptor>' --method <name> --signatures
--exceptions` for DiM.dynamicMethod,FOr.CR7,FPf.CR7,F1B.A02,F2D.A03,DBg.A0f;
`apk-field-refs <apk> --class 'LX/DiM;' --field viewOnce_ --type Z` and
`--field streamingSidecar_ --type 'Lcom/google/protobuf/ByteString;'`.
Core field output was obtained with `apk-proto-fields <apk> --class 'LX/DiM;'`
in the prior bundle and copied byte-identically here. Run
`python3 docs/protocol/apk/2.26.43.72-media-video-ingress/presence_check.py`
for the selected presence check. [checksums.json](checksums.json) pins all files.
