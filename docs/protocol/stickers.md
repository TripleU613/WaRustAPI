# Incoming stickers

Evidence applies to APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This does not update the older comparison build's obfuscated names.

## Confirmed schema

`BZc` (Message) field 26 references `BZC` (StickerMessage); capitalization
matters. `BZC.dynamicMethod` allocates a 23-entry name array at `002e..0036`.
`6al.A1W` (`0000..000a`) supplies bookkeeping `bitField0_` at slot zero;
`Ayq.A11` (`0000..0060`) supplies slots 1–9. `BZC.dynamicMethod`
(`0042..00de`) supplies the remaining names. Its descriptor literal at
`00de..00e2`, passed to `newMessageInfo` at `00e6..00ec`, has a ten-integer
header followed by 22 scalar field/type/presence entries. Joining those entries
with the complete helper-produced array confirms this wire layout:

| Field | Name | Type |
| --- | --- | --- |
| 1 | url | string |
| 2 | fileSha256 | bytes |
| 3 | fileEncSha256 | bytes |
| 4 | mediaKey | bytes |
| 5 | mimetype | string |
| 6 | height | uint32 |
| 7 | width | uint32 |
| 8 | directPath | string |
| 9 | fileLength | uint64 |
| 10 | mediaKeyTimestamp | int64 |
| 11 | firstFrameLength | uint32 |
| 12 | firstFrameSidecar | bytes |
| 13 | isAnimated | bool |
| 16 | pngThumbnail | bytes |
| 17 | contextInfo | message |
| 18 | stickerSentTs | int64 |
| 19 | isAvatar | bool |
| 20 | isAiSticker | bool |
| 21 | isLottie | bool |
| 22 | accessibilityLabel | string |
| 24 | premium | int32 |
| 25 | emojis | string |

Fields 14, 15 and 23 are absent from this descriptor. Array positions are not
wire tags: the skipped numbers, integer `premium` and scalar string `emojis`
come from the descriptor, not assumptions about their names. Nested context's
complete current-build type is not traced here; only the existing context
projection's quote/expiration subset is used.

**Confirmed key domain:** `1qB.<clinit>` loads `WhatsApp Image Keys` into
`v7` at `0028..0034`. Its `sticker` / `STK` constructor branch at
`02a0..02be` passes that same `v7`, then saves type `A0t`. Stickers therefore
reuse the image HKDF domain. Exact-string xref for `WhatsApp Sticker Keys`
returned no references; this search alone would not prove the domain.

## Implementation and limits

**Offline implemented:** ordinary URL-bearing WebP stickers use `media-list`,
`media-open`, sealed `received_media()` references and `download_media()` behind
the existing comparison opt-in. Image-domain encrypted hash, MAC, padding,
plaintext hash and length checks apply; verified ciphertext is encrypted at rest.
The existing downloader's approved HTTPS hosts, one-shot reservation, 8 MiB
limit, expiration checks and account binding remain enforced. The CLI selects
visual playback for stickers. Synthetic tests establish descriptor decoding,
wrong-domain/corruption refusal, cached retrieval without a connection, sealed
reference projection, mixed-message refusal and quote/view-once exclusion.
Their plaintext is a crypto fixture, not evidence of a renderable WebP.

**Unknown/live unverified:** no incoming sticker stanza or handset/viewer result
has been captured. Animated WebP is accepted without separately validating its
frames; actual display/playback is unverified. Lottie-marked payloads are left
unsupported. DirectPath-only retrieval, sticker sending, pack metadata, avatar/AI
semantics and timestamp units are not implemented or established here. A missing
URL refuses with `invalid media URL`; URLs/keys are not exposed by references.

**Recorded tooling limitation:** `xtask apk-proto-fields --class 'LX/BZC;'`
refuses this class with 22 fields against 13 directly visible names, because it
does not expand the two array-fill helpers. The refusal is correct; it is not
proof that the schema is unavailable. Reproduce the trace with `xtask apk-smali`
for `BZC.dynamicMethod`, `6al.A1W`, `Ayq.A11` and `1qB.<clinit>` against the
identified APK. Bulk dumps remain private under ignored `lab/`.
