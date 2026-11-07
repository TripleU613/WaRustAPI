# Audio ingress and exact supported Opus MIME — APK 2.26.43.72

**Confirmed selected static / implemented offline preflight correction**,
build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports and exact schema decoding via xtask fallback;
no target execution, new REA Evidence IDs, account traffic or live GETs.
Method PCs are byte offsets; branch displacements use16-bit code units.

## Schema and selected converters

[Dhn descriptor](audio-descriptor.txt) and [decoded fields](audio-fields.txt)
confirm url1, mimetype2, fileSha2563, fileLength4, seconds5, ptt6, mediaKey7,
fileEncSha2568, directPath9, mediaKeyTimestamp10, streamingSidecar18,
waveform19, backgroundArgb20, viewOnce21 and accessibilityLabel22. Rust's core
Audio descriptor uses the same consumed tags. [Di0](excluded-document-fields.txt)
was a bounded candidate and is the document-shaped schema, not the audio class.
The earlier DhY submessage is not substituted for Dhn: shared names do not make
all schemas equivalent.

Exact [PTT](ptt-refs.txt) and [viewOnce](view-once-refs.txt) field censuses find
FIVE direct DEX sites each. [FPQ.CR7](ordinary-converter.txt) requires the audio
message envelope and refuses present-and-true viewOnce before constructing9JM.
[FPd.CR7](view-once-converter.txt) selects present-and-true viewOnce and constructs
J9v with integer82. Both invoke the common Etl.A00 parser with a caller mode.
Full wrapper/converter dispatch and other field consumers remain unknown.

[Common parser](audio-projector.txt) copies ptt into message.A05 at000a..0012;
it does NOT require ptt=true. [DDS.A07](audio-classifier.txt) independently
classifies an audio envelope as `ptt` versus `audio` from that flag at0134..0164.
Rust's PTT-only retrieval is therefore a supported-scope limit, not a universal
native audio-admission rule or reason to reject every non-PTT server message.
View-once audio remains outside Rust's ordinary replayable media path.

Etl.A00 requires media key on selected fresh paths, positive fileLength at00d0,
32-byte plaintext hash at0106 and32-byte encrypted hash at012a before their
message setters. Update-mode presence rules are separate. It adopts waveform
only when present with length1..192; background color is optional. Absent or
oversized waveform skips that optional block. Missing directPath is logged in
this leaf rather than universally throwing; native URL construction/recovery
must still be traced before equating this to Rust's descriptor URL requirement.
Local malformed-hash error14 is not an HTTP or proven ban/retry code.

## Exact MIME leaf and confirmed Rust mismatch

On selected fresh/non-update input Etl.A00 calls [0jZ.A08](audio-mime-selector.txt)
with logging enabled and throws Dxp.A03 for an unrecognized MIME. That selector
uses exact String.equals matches:

| Native MIME input | Selected suffix |
| --- | --- |
| audio/aac | aac |
| audio/mp4 | m4a |
| audio/amr | amr |
| audio/mpeg | mp3 |
| audio/ogg; codecs=opus | opus |

Everything else returns null (with optional logging), including bare audio/ogg,
Vorbis, changed case, omitted separator space and appended parameters. This is
the selected native fresh parser's exact leaf, not a general MIME standard or
complete app-wide codec capability census. Runtime update inputs can skip this
validation; their provenance remains outside this supported fresh-descriptor fix.

**Confirmed Rust defect / fixed:** the explicit Opus-only voice-note path used
`mime.split(';').next().trim() == "audio/ogg"` in media_url. Unsupported codec
metadata could become a selectable attachment, reach a GET reservation, or reuse
valid cached Opus ciphertext despite contradicting the pinned native MIME leaf.
The Audio descriptor now excludes anything except exact `audio/ogg; codecs=opus`
in addition to existing PTT=true and viewOnce=false requirements. This runs
before candidate listing, sealed references, cache lookup/handoff and new GET
reservation. Outgoing voice-note serialization already emits that exact string.

Generic AAC/MP4/AMR/MP3 audio is NOT enabled by this change, nor is it asserted
native-invalid. The receiver remains explicitly Opus PTT only. Unsupported
historical metadata is held unselectable without deleting/reclassifying cached
ciphertext or authorizing a retry. No timestamp, device provenance, codec,
sidecar or header is fabricated. No new server response/ACK behavior is claimed.

## Regression and limits

`voice_note_mime_gate_precedes_cache_handoff_and_network_reservation` uses a
real encrypted store and actual audio-domain ciphertext/MAC/hash for a synthetic
bounded Ogg/Opus fixture. Canonical PTT remains listable/referenceable and decrypts
from cache. Bare/contradictory/case/spacing/extra-parameter MIME, unsupported AAC,
non-PTT and view-once inputs cannot become ordinary selectors or open the cache;
checkpoint rows and journal remain unchanged. After cache removal and store
reopen unsupported inputs remain unselectable and create no reservation. Those
uncached fixtures additionally use an unapproved host as a no-network backstop;
the test does not rely on a live CDN failure or claim a captured HTTP exchange.

**Unknown:** full receive dispatch and error/ACK mapping, exact update-mode
inputs, generic audio/sidecar provider and waveform storage, native request URL/
range/status recovery and account/output authority. Rust's8-MiB bound, mono Opus
page checks and uncertainty/no-replay policy remain local limits. No206/416
relaxation, automatic request/retry/resume, live acceptance or zero-ban claim.

Reproduce `apk-proto-fields <apk> --class 'LX/Dhn;'` (Di0 for the bounded
negative); `apk-field-refs <apk> --class 'LX/Dhn;' --field ptt_ --type Z` and
`--field viewOnce_ --type Z`; and `apk-smali <apk> --class '<descriptor>'
--method <name> --signatures --exceptions` for Dhn.dynamicMethod,Etl.A00,
FPQ.CR7,FPd.CR7,DDS.A07,0jZ.A08. [checksums.json](checksums.json) pins all files.
