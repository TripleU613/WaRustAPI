# Full framed-block decode and native-driver boundary — APK 2.26.43.72

**Observed offline decoding / constrained emulation; static driver mapping.**
All 213 framed blocks now have output buffers. No complete native library has
been reconstructed. Selected type-42 instructions execute under Unicorn with
controlled imports; no Android app, JNI startup, target network or account
operation is involved. These results are not REA Evidence envelopes.

Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 carrier SHA256:
`1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5`.
ARM64 decoder SHA256:
`fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`.

## Complete framed-block experiment

The [framing inventory](../2.26.43.72-superpack-transform-blocks/README.md)
accounts for all archive bytes. For each of its **140 type-1 blocks**, Python's
XZ decoder reached EOF, left no unused bytes, and produced exactly the declared
output length, with a 128 MiB decoder-memory bound and output limit of declared
length+1. This is now frame-bounded decoding, improving on the earlier raw XZ
candidate scan. It does not independently validate the outer archive's hashes.

All **73 type-42 blocks** returned to the emulation sentinel with exactly their
declared output length. Blocks 0, 168 and 201 reused the verified candidates from
the [prior emulation experiment](../2.26.43.72-superpack-kind42-emulation/README.md);
the other 70 were newly emulated with that retained harness. emulation-runs.txt
retains all 70 successful stdout records. Execution bounds, allocator differences,
controlled imports and all other emulator limitations from that bundle apply.
Only the selected small block had the two-helper comparison; this full set has
not been independently decoded by another implementation or a real device.

The 213 output buffers total **81747344 bytes**. decoded-blocks.json retains
ordinal, kind, method, output length, SHA256 and prefix for every buffer. None
starts with ELF magic. The 83 metadata records declare a combined **65766648
bytes** of member output. Therefore the block outputs must not be presented as
already recovered library files or concatenated into a purported ELF image.
Actual block-to-member mapping and the additional format reconstruction remain
unverified. No decoded body is committed; all are in ignored research storage.

## Concrete next stage: the so format driver

Static 0xb43c allocates registry state, passes exact strings `native_driver` and
`so` to setup helper 0x1d654 with callback table VA 0x3cdf0, initializes another
helper and registers the state via 0x1d5e8. Setup copies the name/type into state
and stores the callback table at state +8. The registry's comparison in 0x1d720
uses the requested type against that stored string (with other selection branches
retained, not flattened into an unconditional rule).

For operation selector 1, dispatch loads registry state +8, then table +8, and
calls its pointer. The existing [member iterator path](../2.26.43.72-superpack-member-output/README.md)
reaches 0x1d6bc, which supplies selector 1 to 0x1d720. ELF RELA resolves table
slot **VA 0x3cdf8 to callback 0x1af20**; actual file mappings are retained in
driver-mapping.json, independently of zero on-disk placeholder slots.

The callback gets member bytes/declared length through existing accessors and
calls **0x1adb8** in a loop, advancing its output pointer by the consumed value
reported by that helper. The helper opens parsing via **0x1a30c**, initializes
state via **0x1a420**, gets an output-size value via 0xeadc and invokes 0xe8d4
with callback table VA 0x38390. It also invokes 0xf188 and several cleanup helpers.
These are traced call edges, not a reconstruction of their complete semantics.
They identify where to continue instead of treating raw decoded blocks as files.

## Reproduction and validation

Run `python decode-blocks.py` from the repository root with the isolated Python
containing unicorn==2.1.4. It expects the pinned APK in
`../android-lab/apks/WhatsApp.1.apk` and writes only under ignored android-lab
research output. It checks APK/carrier and encoded-payload hashes; the imported
emulator also checks the decoder hash. Three optional preexisting prior candidates
must match their retained SHA256 before reuse; absent candidates are emulated.
Method labels differ when those caches are absent, while output lengths/digests
should match decoded-blocks.json. This pinned experiment script is not a generic
safe archive extractor or a production Rust client capability.

The observed initial run used the same decode loops with the original three
already verified candidates. The retained script adds explicit candidate-cache
digest checks and uses the invoking interpreter instead of an absolute venv path.
Every saved buffer was rehashed against the retained manifest after the run;
all 73 successful return records and all 140 XZ EOF/length checks are accounted
for. Raw buffers stay local. No claim of a second fresh full decode is made.

Assembly reproduction uses Rizin `pD SIZE @ ADDRESS` with ranges.json, the pinned
decoder and the [retained tool version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt).
[Full relocations](../2.26.43.72-superpack-jni-registration/elf-relocations.txt)
and [section mappings](../2.26.43.72-superpack-decoder-entry/elf-sections.txt)
are retained. checksums.json covers this bundle.

Next trace native-driver parsing, block selection and output reconstruction,
recover usable ELF members, then resume native metrics schema/storage/network
and server-response auditing. No Rust behavior, HTTP route, sender, telemetry,
retry or live account/network operation changed.
