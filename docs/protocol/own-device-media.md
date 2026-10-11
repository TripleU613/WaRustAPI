# Own-device ordinary media synchronization

2026-10-11. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Implemented/offline:** a held, pinned own-device session can synchronize
ordinary image, push-to-talk audio, sticker, document, video and video-note
messages into the normal event journal and attachment retrieval path.
This extends [own-device text](own-device-text.md), retaining its account,
source, destination, transaction and deduplication checks.
Follow-up: [normal outgoing echo correlation](own-device-echo.md) suppresses
a second message event for a matching existing intent without changing its
send outcome; complete cross-source/placeholder reconciliation remains open.

## Evidence and admission

The [release recipe](apk/recipes/2.26.37.71-own-device-media.json) maps the
[wrapper/descriptor DEX bundle](apk/2.26.37.71-own-device-media/targets.json)
to the four runtime files. `Cm3.A02` validates the wrapped destination;
`CjY.A01` selects the nested Message and replaces its context with the outer
context when present. These shared operations precede content dispatch.
The [wrapper trace](device-sent-wrapper.md) retains the successful DEX branch
lost by its Java rendering and the own-device gate's unported exceptions.

`BZc` fields 3/7/8/9/26/66 select image/document/audio/video/sticker/video-note.
Current image and audio classes are **BZD** and **BZ0**; BWp/BWZ names in earlier
comparison source comments describe another release. Complete REA envelopes retain
[BZD](rea/2.26.37.71-own-device-media/image-descriptor.json) and
[BZ0](rea/2.26.37.71-own-device-media/audio-descriptor.json).
Other schema details remain in [documents](document-media.md),
[video](video-media.md), and [stickers](stickers.md).
Static descriptors do not establish execution or live server acceptance.

The helper-generated names are retained independently:
[Ayp.A1K](rea/2.26.37.71-own-device-media/image-name-helper.json) supplies
image caption/hash slots 3/4; [Ayq.A12](rea/2.26.37.71-own-device-media/audio-header-helper.json)
supplies audio MIME/hash/length slots 2/3/4;
[Ayq.A13](rea/2.26.37.71-own-device-media/audio-key-helper.json) supplies
audio key/encrypted-hash/path/time slots 7/8/9/10. Joining these names with
the encoded descriptor, rather than using array positions as tags, confirms:

| Content | Plain hash | Length | Key | Encrypted hash | View-once | Other |
| --- | --- | --- | --- | --- | --- | --- |
| BZD image | 4 | 5 | 8 | 9 | 25 | caption 3, context 17 |
| BZ0 audio | 3 | 4 | 7 | 8 | 21 | push-to-talk 6, context 17 |

The current `apk-proto-fields` command refuses both descriptors because it
does not resolve these helpers (29 fields/28 names and 16 fields/11 names).
No successful automatic descriptor decode is claimed for this follow-up.

Rust admits one supported inner content field plus Message context, retaining
media submessage bytes through outer-context replacement. The existing
ordinary-media validator checks descriptor bounds, URL/MIME policy, key/hash
sizes, view-once exclusion, quote refusal and supported audio/sticker forms.
Unsupported content cannot commit the advanced receive ratchet.
Media projection requires registration HTTPS support, as does the existing
ordinary-media module; the full comparison client enables it.

## Storage, API and CLI

The authenticated transaction retains the original DeviceSentMessage wrapper
under `received-own:<recipient>:<id>`, its actual source and original stanza,
and commits the advanced session and outgoing NewMessage event together.
The event has the recipient chat, no sender, and an account-bound sealed media
reference. Attachment keys and URLs do not enter the journal. No send intent,
request ID, identity replacement or download is created.

Attachment enumeration unwraps these saved records using their exact saved
chat. The selector hashes the original `received-own` key, matching the event
reference. Existing media listing, reference retrieval, explicit export and
cached ciphertext behavior apply without a new command or option. Expiry still
uses the saved stanza timestamp and nested media context. Local clearing removes
the original plaintext and makes the selector unavailable; historical events
remain historical. The committed route also uses
[sender confirmation](own-device-confirmation.md), including redelivery.

## Validation and limits

A real Signal fixture now receives wrapped image ciphertext, deduplicates it,
checks the sender receipt and attachment selector, reopens without publishing
again, clears text and media together, and continues the held ratchet.
An independent encrypted-store fixture checks outgoing chat/direction, sealed
reference selection from the original wrapper, no outbox row, and refusal of a
reference after clearing. Existing ordinary-media tests exercise descriptor,
view-once, expiry, URL, reference and retrieval boundaries shared by this path.

New own-device bootstrap, alternate-recipient mapping, control synchronization,
quoted media, view-once lifecycle, primary echo reconciliation and native
read-self/retry recovery remain open. There was no fresh live own-device media
sample or download. This port does not prove full APK parity or ban immunity.

Local validation passed all 944 workspace/all-feature tests, strict workspace
Clippy, warnings-denied Rustdoc, 26 default CLI tests and the experimental CLI
build. The real Signal regression also passes with the minimal TCP/Noise/Signal
features, without HTTPS/media enabled, retaining its text behavior.
The 13-topic APK bundle reproduced all 18 files (113,084 bytes) with `--check`.
Cross-release semantic equivalence remains untested.
