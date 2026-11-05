# Superpack from-so archive handoff — APK 2.26.43.72

**Confirmed static / derived argument flow**, bounded Rizin disassembly and
verified decoder bytes; not execution, complete decoder reconstruction or REA
Evidence envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Selected ARM64 decoder SHA256:
`fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`.
No target library was loaded or invoked. File/virtual mapping derives from the
[retained decoder sections](../2.26.43.72-superpack-decoder-entry/elf-sections.txt).

## Exact mapped entry and default ranges

The [verified relocated JNI table](../2.26.43.72-superpack-jni-registration/README.md)
maps decompress_from_so to 0x9ec8 and decompress_range_from_so to 0x9a50.
The 12-byte wrapper sets w5=w6=-1 and branches to 0x9a50. Selected argument flow
extracts the three String arguments through JNI table calls, retains the latter
two separately, and later compares the second String to literal spo. Failure
messages identify the roles as so name, storage type and output directory;
these are derived argument roles, not arbitrary filenames accepted in a Rust API.

The start value is normalized to 0 when negative. A negative end becomes -1 in
the primary handoff. A nonnegative end additionally checks original start<=end;
otherwise the selected exception helper is reached. The no-range wrapper thus
passes start 0/end -1 to the primary core call. Full core interpretation of those
numbers remains untraced; they are not assumed to be inclusive file indices or
byte offsets merely from this wrapper.

## Carrier symbols define the primary decode range

The path invokes dlopen with raw flags 6, then uses android_dlopen_ext with raw
flags 2 and null extension argument if the first call fails. No flag-name/platform
semantics are inferred here. With a handle it calls dlsym for exact literals
_superpack_archive_start and _superpack_archive_end. Either null reaches the
corrupt-archive/error continuation; it does not proceed with an invented size.
With both, it computes end-start as the input byte length.

This connects the native wrapper to the [previously measured carrier range](../2.26.43.72-metrics-native-carrier/README.md),
not to the whole ELF file. Referenced literals and their correctly mapped file
locations are retained in JSON. No inner-member identity follows from this range.

The selected ordinary path requires storage type spo, obtains helper 0x3316c's
configuration object, stores a value derived from sysconf and calls 0xa964 with
JNI environment, archive start, length, configuration, normalized range and
output-directory bytes. A null result invokes the selected extraction-error
helper; a nonnull value is returned after shared String cleanup. No archive
format/parser semantics are assigned to 0xa964 from its address alone.

## Cache fallback and failure scope

The body also stores/checks native state near 0x3d778: a 32-bit helper 0xa934 result,
archive length and start pointer. If both carrier load attempts fail and a cached
length exists, another branch allocates/copies cached bytes, compares the same
helper result and can call 0xa964 on that copy. Its selected end argument uses the
original register on this alternate branch. Full cache lifetime, synchronization,
hash algorithm, error helper and all allocation-failure effects remain open.
Do not collapse the ordinary and cached branches into one proven recovery rule.

The retained range covers the selected body and shared cleanup/error continuations
up to 0x9ebc; called helpers remain separate unknowns. Static fallback code is not
an observed successful decompression, disk persistence or restart recovery.

## Reproduction and next proof

With verified decoder FILE, run Rizin color disabled with pD 12 @ 0x9ec8,
pD 512 @ 0x9a50 and pD 620 @ 0x9c50 (spaces between command operands in actual use).
The two body windows are contiguous; range-digests.json records their byte hashes.
The previously retained [Rizin version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt)
applies. No whole-function decompiler or runtime coverage claim is made.

Next trace 0xa964 and its archive/transform calls, establish metadata and native
member reconstruction, then resume metrics JNI schema/storage/transport and
server-response auditing. Native cache/replay authority and generation fencing
remain incomplete. No Rust behavior, telemetry, sender, retry, HTTP route or
live account/network operation changed.
