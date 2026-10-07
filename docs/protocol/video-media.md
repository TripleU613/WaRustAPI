# Ordinary incoming video and video notes

All new static evidence below is **confirmed** for APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This work extends incoming media; it does not implement video sending.

## Schema and type evidence

`BZc` Message descriptor identifies `videoMessage=9` and `ptvMessage=66`.
DEX field references establish their shared type `LX/BZE;`:
`classes.dex` field_ids[24825] names `videoMessage_`; `classes7.dex`
field_ids[15694] and [15721] name `ptvMessage_` and `videoMessage_` respectively.
These are type references, not wire-number evidence; wire tags come from the
Message descriptor ([descriptor method](apk-analysis.md)).

`BZE.dynamicMethod` builds its object-name array at `0024..01a4`;
`6al.A1W` provides bookkeeping and `Ayq.A12` (`0000..001a`) supplies
`mimetype_`, `fileSha256_` and `fileLength_` at slots 2–4.
The descriptor at `01a4..01a8`, passed to `newMessageInfo` at `01ac..01b2`,
contains 30 fields, including two repeated messages. Enum verifier entries
and repeated-message classes occupy additional array slots; neither is a field.
Joining the fully traced name array and descriptor establishes:

| Field | Name | Type |
| --- | --- | --- |
| 1 | url | string |
| 2 | mimetype | string |
| 3 | fileSha256 | bytes |
| 4 | fileLength | uint64 |
| 5 | seconds | uint32 |
| 6 | mediaKey | bytes |
| 7 | caption | string |
| 8 | gifPlayback | bool |
| 9 | height | uint32 |
| 10 | width | uint32 |
| 11 | fileEncSha256 | bytes |
| 13 | directPath | string |
| 14 | mediaKeyTimestamp | int64 |
| 16 | jpegThumbnail | bytes |
| 17 | contextInfo | message |
| 18 | streamingSidecar | bytes |
| 19 | gifAttribution | enum |
| 20 | viewOnce | bool |
| 21 | thumbnailDirectPath | string |
| 22 | thumbnailSha256 | bytes |
| 23 | thumbnailEncSha256 | bytes |
| 24 | staticUrl | string |
| 25 | annotations | repeated message (6tZ) |
| 26 | accessibilityLabel | string |
| 27 | processedVideos | repeated message (BY7) |
| 28 | externalShareFullVideoDurationInSeconds | uint32 |
| 29 | motionPhotoPresentationOffsetMs | uint64 |
| 30 | metadataUrl | string |
| 31 | videoSourceType | enum |
| 34 | smartThumbnailTs | int64 |

Fields 12, 15, 32 and 33 are absent from this descriptor. Enum value sets,
metadata semantics and timestamp units are not traced here. Context decoding
uses the existing quote/expiration subset; its full current-build type is not
established by this trace.

**Confirmed crypto domain:** `1qB.<clinit>` loads `WhatsApp Video Keys`
into `v5` at `0054..006a`. The `video` / `VID` constructor at `02e4..02fc`
and `ptv` / `PTV` constructor at `02fc..031a` both receive that same `v5`.
A `gif` type also receives it at `0328..0342`; GIF presentation is not implemented
separately here. The existing image/audio domains must not be reused for video.

## Implemented receive behavior and limits

**Offline implemented:** URL-bearing ordinary MP4 video and video notes use
`media-list`, `media-open`, event projection and sealed library media references
under the existing comparison opt-in. `media-list` distinguishes `Video` and
`VideoNote`; the public `MediaReference.kind` is `Video` for both, with the
original protobuf and sealed selector retained. Captions use VideoMessage field
7. The player enables audio and video; it does not impose the image viewer's
no-audio or voice-note viewer's no-video flags.

Video-domain HKDF and the existing encrypted-hash, MAC, CBC/padding,
plaintext-hash and length checks run before access. Existing approved HTTPS host,
account binding, one-shot reservation, encrypted ciphertext cache, expiration and
8 MiB local limits apply. The 8 MiB ceiling is our implementation limit, not an
APK/server limit. Direct viewOnce=20 and all supported view-once wrappers are
excluded; mixed media envelopes are rejected. Quote projection stays unsupported.
`decrypt_video` is a crypto primitive and does not enforce consumption itself.

**Test evidence:** `tests/fixtures/video-media.json` was independently generated
using Python standard-library HKDF/HMAC/SHA-256 and OpenSSL AES-256-CBC, with a
fixed synthetic key and text plaintext. Reproduction uses 32 bytes of `0x07`
as the key, SHA-256 HKDF extract with an all-zero 32-byte salt, and expand with
`WhatsApp Video Keys` to 80 bytes. Bytes 0–15 are the IV, 16–47 the AES key,
and 48–79 the HMAC key. OpenSSL `enc -aes-256-cbc` supplies PKCS#7 padding;
append the first ten HMAC-SHA-256 bytes over IV plus ciphertext. Hash plaintext
and ciphertext-plus-tag separately. Literal Message and VideoMessage wire
tags test both kinds, wrong image/audio domains, hash corruption, MIME refusal,
view-once exclusion, caption projection, durable once-only journal events across
reopen and cached retrieval without a connection. This is not a renderable MP4
or proof of live interoperability.

**Unknown/live unverified:** no incoming video/PTV stanza, display or playback
has been observed. DirectPath-only retrieval, large/streamed files, processed/HD
variant selection, video sending, circular PTV rendering, looping GIF behavior
and view-once video consumption are unimplemented. Generic MP4 playback does
not reproduce the APK's video-note UI.

**Negative/tooling:** `xtask apk-proto-fields --class 'LX/BZE;'` reports
30 fields against 28 directly visible names and refuses to guess. It does not
expand `Ayq.A12`. Reproduce with `xtask apk-smali` for `BZE.dynamicMethod`,
`Ayq.A12`, `6al.A1W` and `1qB.<clinit>` against the identified APK. Bulk dumps
and field-reference extracts remain private in ignored `lab/`.

## Bounded saved-account check (2026-10-07)

**Observed:** the rebuilt release client completed one 60-second `listen` window
with chat sends disabled. Authentication/activation succeeded and shutdown was
clean. Preflight had zero eligible sends, zero queued actions and nine
server-acknowledged receipts. Afterward the journal still contained the same
20 events (complete summaries, cursor zero, no truncation), receipt states were
unchanged, and one existing uncertain action remained `sent`. No new video/PTV
event, receipt or playback was established. `media-list` returned zero eligible
ordinary attachments; this does not establish why older media is unavailable.

Private evidence is in `lab/accounts/l/video-receive-check-2026-10-07/`: original
encrypted database backup, separate baseline inspection copy, receive log,
before/after event summaries and action/receipt/outbox inspections. Inspectors
can write normal database-open metadata; they did not advance a journal consumer
or delete a marker. No chat send, media GET, identity reset or resend was used.
