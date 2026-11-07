# Whole image/video MIME admission — APK 2.26.43.72

**Confirmed selected static / implemented offline preflight correction**,
build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports via xtask fallback plus previously checksum-pinned
current parser exports; no target execution, new REA Evidence IDs, live requests
or account operations. PCs below are method-relative bytes; branch displacements
use16-bit code units.

## Image comparison is whole-string with context-specific WebP

The [complete current EwA.A03](../2.26.43.72-media-image-ingress/image-projector.txt)
at043a..04ac uses whole-string String.equalsIgnoreCase for image/jpeg and
image/png. It does NOT trim leading/trailing whitespace or split parameters.
Its image/webp branch additionally requires config flag0x38f9 and a BUZ or1g9
message subtype. Caller update/context predicates can skip MIME adoption.
Thus WebP is not proven universally native-valid for every ordinary image.

Rust already supports explicit image WebP retrieval; this pass retains that
supported-format policy rather than inventing unavailable native feature/subtype
inputs. Exact native WebP profile binding remains unknown. For the supported
ASCII image MIME values, whole-string ASCII-insensitive matching avoids claiming
Java Unicode case-fold parity while matching ordinary JPEG/PNG case behavior.
No original metadata is rewritten or normalized.

## Video comparison is exact, not parameter-tolerant

The [complete current F1B.A05](../2.26.43.72-media-image-ingress/video-projector.txt)
at01a0..01ca calls0jZ.A09 and throws on null in the selected parse path.
[0jZ.A09](video-mime-selector.txt) first handles empty MIME, then calls
[0jZ.A0E](video-mime-eligibility.txt). A0E uses String.equals on video/mp4 OR
video/3gpp; A09 maps those exact strings to mp4/3gp. There is no case folding,
trimming or parameter stripping in these leaves. Logging flag affects diagnostics,
not admission. Update-mode/caller conditions are independent.

Rust currently supports MP4 only. Native3GPP recognition does not add3GPP support
or prove every native video path handles it. The video-note receiver uses the
existing same Rust descriptor/crypto policy, but native PTV dispatch remains
separately untraced. No complete PTV operation parity is inferred from this leaf.

## Confirmed Rust gap and implementation

media_url previously stripped semicolon parameters and trimmed the MIME base
for image/video, admitting contradictory/padded metadata for its supported formats.
Descriptor admission did not check MIME, so completed cached ciphertext could
also produce plaintext without the URL-stage format check. Conversely image
ASCII case variants were refused by the case-sensitive URL comparison.

A shared supported_mime helper now supplies descriptor and URL preflight for
image, voice note, sticker, video and video-note formats. Image uses whole-string
ASCII-insensitive jpeg/png/webp; video/video-note use exact video/mp4. This means
unsupported visual MIME cannot become a selector/reference, reuse cached plaintext
or reserve a new GET. Voice-note exact Opus and sticker whole-WebP rules retain
the behavior fixed in the preceding changes. Document export MIME handling is
unchanged: its local token/size policy still trims/splits only within the document
branch and is not represented as a native visual-parser rule.

No cryptographic material, account owner, saved source, uncertainty checkpoint,
HTTP header, format parameter or timestamp is synthesized or reset. Existing
view-once exclusions, size/hash checks and no-replay state remain in force.
Unsupported historical metadata becomes unselectable without deleting ciphertext.
No native parser error/ACK/retry-code mapping is added by a local admission fix.

## Regression and scope

`image_video_mime_admission_precedes_cached_plaintext_and_get_reservation` uses
actual per-kind image/video ciphertext with MAC and both hashes in encrypted
stores. It checks image, video and video-note selection/reference/open APIs.
Canonical formats decrypt cached content and preserve their kind; ASCII image
case variants work. Parameters/padding, unsupported image GIF, uppercase MP4,
and locally unsupported3GPP cannot become ordinary selectors or open the cache.
Checkpoint rows and journal remain unchanged by read/refusal. After cache removal
and reopen, unsupported metadata still creates no reservation. Uncached refused
fixtures use an unapproved host as a no-network backstop. Synthetic crypto
plaintext does not claim JPEG/PNG/WebP/MP4 renderer validity or live HTTP behavior.
Existing audio/sticker/document tests cover shared-helper compatibility.

**Unknown:** full native update/error/envelope dispatch, exact WebP config/subtype
binding, generic3GPP/PTV/native codec behavior, Java Unicode image matching,
URL/range/status recovery, account/output authority and live interoperability.
No206/416 relaxation, retry/resume, new field/header, native-global allowlist or
restriction-cause/zero-ban claim is introduced.

Reproduce `apk-smali <apk> --class 'LX/0jZ;' --method A09 --signatures --exceptions`
and A0E for the new leaves. Earlier complete EwA.A03/F1B.A05 exports are linked
above with their own bundle checksum manifests. [checksums.json](checksums.json)
pins all new retained files, including this report.
