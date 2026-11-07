# Sticker ingress, oneof boundary and whole MIME — APK 2.26.43.72

**Confirmed selected static / implemented offline preflight correction**,
build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports via xtask fallback; no target execution, new REA
Evidence IDs, account traffic or live GET. Byte PCs are method relative; branch
displacements use16-bit code units.

## Recover the core fields without guessing through a oneof

[Di4.dynamicMethod](sticker-descriptor.txt) has a new audio oneof. The existing
apk-proto-fields decoder refuses this class with unsupported oneof pairing;
no literal-order fallback was used. [Di2](excluded-pack-fields.txt) is a sticker
PACK schema, not the individual sticker. [isLottie string census](lottie-schema-xrefs.txt)
finds four schema candidates; shared names do not identify the correct type alone.

Di4's object array is explicitly built with audio_ in slot0, then
[DBe.A1T](oneof-array-helper.txt) writes audioCase_ in slot1 and bitField0_ in2.
[DBg.A1J](hash-array-helper.txt) writes its supplied url_ to3 and fileSha256_ to4;
[DBg.A1H](key-array-helper.txt) writes supplied fileEncSha256_ to5 and mediaKey_ to6.
The main method directly writes mimetype_ to7, fileLength_ to11 and isLottie_ to21.
These are actual array writes, not names paired by their literal appearance order.

The encoded descriptor has one oneof pair and one presence word, followed by22
normal fields and a final oneof entry. [Bounded offline checker](normal_fields_check.py)
validates the exact header, single-word integer form, normal-field prefix and
presence indexes; names/slots are explicit observations of those array writes.
[Output](selected-normal-fields.json) confirms core normal fields:

| Object slot | Name | Wire tag |
| --- | --- | --- |
| 3 | url | 1 |
| 4 | fileSha256 | 2 |
| 5 | fileEncSha256 | 3 |
| 6 | mediaKey | 4 |
| 7 | mimetype | 5 |
| 11 | fileLength | 9 |
| 21 | isLottie | 21 |

These match Rust's consumed Sticker tags. The checker deliberately leaves the
trailing `[26,60,0]` entry unpaired; it is NOT general oneof schema support or
complete new sticker/audio behavior. The descriptor's slot25 is a Dhn class;
actual consumer FPx tests audioCase==26 and casts audio_ to Dhn. This independently
establishes the selected embedded audio branch, not its full wrapper/ownership
semantics. Rust does not implement that embedded audio operation.

## Actual parser and selected MIME rule

[Lottie field census](lottie-refs.txt) finds FIVE DEX read/write sites. The first
candidate [Exd.A01](candidate-envelope-rewrite.txt) is an envelope rewrite/context
helper, not the ordinary sticker projector. [FPx.A00](sticker-projector.txt)
actually reads Di4 fields into message metadata. At00cc..0108 it enforces32-byte
hashes on selected fresh/present paths before known byte-to-message setters.
At01de..01f4 its size leaf rejects negative values, permitting zero at this stage;
Rust's nonempty/8-MiB limits remain local, without claimed native empty-transfer
parity. Caller update-mode/presence conditions are separate.

At0118..016e FPx branches on isLottie. Non-Lottie uses whole-string
String.equalsIgnoreCase against `image/webp`. Lottie instead compares
`application/was`; no Lottie decode/retrieval is enabled in Rust. This is not a
parameter-stripping MIME parser. [FQe.A00](sticker-converter.txt) is a separate
conditional classifier involving audioCase, feature/config predicates and Lottie;
it is not silently treated as complete ingress converter dispatch.

## Confirmed Rust preflight defect and fix

Rust media_url previously split MIME at semicolon and trimmed the base, so a
parameterized/padded `image/webp` could authorize a GET. Descriptor selection also
allowed these metadata values to reach completed-cache plaintext handoff without
media_url. Conversely uppercase WebP was refused at the URL check despite the
selected native whole-string case-insensitive rule.

The ordinary Sticker descriptor now requires whole-string ASCII-insensitive
`image/webp` and rejects Lottie before candidate/reference/cache/network authority.
The URL preflight uses the same whole-string ASCII-insensitive check. Thus ASCII
case variants work without normalization of saved metadata; parameters/whitespace
and unrelated MIME cannot authorize retrieval or cache handoff. This follows the
native rule for the supported ASCII MIME domain; full Java Unicode case folding
is NOT claimed. No generic helper or guessed Unicode matching is introduced.

`sticker_projection_reference_and_cached_retrieval` now exercises canonical and
mixed/upper ASCII case MIME, parameters, padding, unrelated MIME and Lottie with
actual image-domain ciphertext/hash/MAC in an encrypted store. Accepted cases
preserve plaintext; unsupported cases cannot produce selectors/references or open
the cache. Checkpoint rows/journal are unchanged by refusal. Uncached bad cases
add an unapproved-host no-network backstop and remain unselectable without a
reservation. Synthetic crypto plaintext is NOT a WebP renderer-validity test.
No history/cache deletion, uncertain-attempt reset, automatic retry or GET occurs.

## Embedded audio and remaining scope

FPx.A00 also contains sound-sticker key/hash/length/MIME checks after its
case26 branch. These use the separate audio object, not the sticker's image
crypto domain. Complete sound-sticker validation/storage, audio viewOnce/ownership,
feature gating, conversion dispatch and playback remain untraced; no implicit
support is claimed merely because Rust protobuf skips unknown field26.

**Unknown:** complete envelope/router and update modes, native Lottie/embedded
audio handling, zero-byte full transfer, complete Java Unicode case behavior,
request URL/range/status recovery, account/output authority and live interoperability.
No206/416 relaxation, fabricated field/header, automatic request/resume or account
operation is introduced. No server acceptance, ban trigger or zero-ban claim.

Reproduce `apk-smali <apk> --class '<descriptor>' --method <name> --signatures
--exceptions` for Di4.dynamicMethod,DBe.A1T,DBg.A1H/A1J,Exd.A01,FPx.A00,FQe.A00;
`apk-field-refs <apk> --class 'LX/Di4;' --field isLottie_ --type Z`;
`apk-xref <apk> --anchor isLottie_`; `apk-proto-fields <apk> --class 'LX/Di2;'`
for the excluded pack candidate. Run `python3 docs/protocol/apk/2.26.43.72-media-sticker-ingress/normal_fields_check.py`
for the selected normal prefix. [checksums.json](checksums.json) pins all files.
