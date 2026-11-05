# Superpack member selection and disk output — APK 2.26.43.72

**Confirmed static / derived control flow**, bounded ARM64 disassembly. No APK
native code was loaded or executed. These are local Rizin exports, not REA
Evidence envelopes. APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Decoder `lib/arm64-v8a/libsuperpack.so`, 220920 bytes, SHA256:
`fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f`.
The extracted decoder was checked byte-for-byte against that APK member.

## Core entry and ordinal ranges

[The from-so wrapper](../2.26.43.72-superpack-from-so/README.md) calls 0xa964.
Its complete selected body is 0xa964..0xabd0 (exclusive), retained in
core-assembly.txt. Incoming arguments are JNI environment, archive pointer,
length, configuration, start, end, and output-directory bytes. It calls 0x1c7ac
with archive pointer, literal `spo`, length, configuration, end, and null.
The returned value is accepted if zero or greater than 0x17; values 1..0x17
invoke AssetDecompressionException and return null. This pointer/error convention
is observed in comparisons, not proof that a null parser result is safe.

An ordinal counter starts at zero. At the top of the loop, end == -1 bypasses
the limit; otherwise counter >= end exits before obtaining another member.
The iterator call is 0x1cca4. Zero ends iteration. For counter < start, the member
is released via 0x3322c and the counter advances. Selected nonzero values <=0x17
raise an extraction exception; other selected members go to 0xa774. Thus the
wrapper's start/end select iterator ordinals with an exclusive end, not byte
ranges. The no-range wrapper's start 0/end -1 selects until iterator exhaustion.
This is derived from the loop; the parser's own end-dependent limiting and
member ordering still require separate proof.

A zero return from 0xa774 leads to a 16-byte list entry containing a bounded
name duplicate. Helper 0x33570 reads member offset 8; 0x331a8 wraps strndup
with limit 0x1000 and aborts on null. The selected member is then released.
A nonzero writer result invokes 0x95d4 for the exception message, releases the
current member, destroys iterator state and frees accumulated names/list nodes.
There is no unlink or rollback of previously written files in this body.
Called helpers are not assumed to have no additional effects.

## Returned names are distinct from durable reconstruction

After iteration, 0x1c714 destroys iterator state. The body resolves
`java/lang/String`, creates an object array sized to selected output count, then
turns each stored name into a Java string and sets the corresponding array
index. It releases local string references and frees names/list nodes. JNI
allocation/exception behavior is not reconstructed beyond these calls.

JNI table offsets 0x30, 0x560, 0x538, 0x570 and 0xb8 match FindClass,
NewObjectArray, NewStringUTF, SetObjectArrayElement and DeleteLocalRef in the
retained host JNI layout probe. That host C program alone was compiled and run;
it supports ABI interpretation, not an observed Android invocation.

## Selected disk writer

The complete selected writer is 0xa774..0xa934. It calls 0x33628 with output
directory and the name accessor result; full path construction/validation in
0x33628 remains untraced. It stats the resulting path, temporarily sets the raw
mode bit 0x80, tolerates chmod errno 2, and opens with exact mode `wb`.
Accessor 0x33578 reads member offset 0x18 as the requested output length.
The writer issues raw syscall number 0x2f with fd, zero, zero, length; platform
syscall naming is deliberately not inferred here. A failure with errno 0x1c
returns status 3 after logging; other failures log and continue to the write.

It obtains bytes via 0x2dbc8 and invokes fwrite(bytes, 1, length, stream).
A short write yields status 4. On successful-length write it clears mode bit
0x80 (including a fresh stat when the original stat failed); failure produces
status 7. Other selected errors include path-helper error 1, fopen 2 and chmod
6. It closes an opened stream but does not inspect fclose's return value.
No fsync, rename or atomic commit appears in this selected writer body. These
facts limit what its zero status proves: it is not independently established
as crash-durable or an all-or-nothing multi-file extraction. No Rust persistence
rule is inferred from this extraction helper.

## Reproduction and remaining work

With the verified decoder, run `rizin -2 -q -e scr.color=0 -c
'pD 620 @ 0xa964; q' FILE`; use the other sizes/addresses from ranges.json for
the writer/accessors. All selected .text and .rodata locations have VA equal to
file offset, as shown in the [retained section map](../2.26.43.72-superpack-decoder-entry/elf-sections.txt).
strings.json retains exact relevant literals. ranges.json hashes raw byte
windows; checksums.json hashes every retained file. Rizin version is retained
in the [earlier bundle](../2.26.43.72-superpack-decoder-entry/rizin-version.txt).
Compile jni-layout.c with the installed Temurin 21 JNI include and include/linux
paths using cc; jni-layout.txt is the resulting host-only output.

Next trace 0x1c7ac's archive header/metadata and transform initialization, then
0x1cca4/0x1d6bc's decoded member production. Raw XZ candidates are still not
validated complete libraries. Native metrics schema, storage, networking and
server-response behavior remain open. No Rust behavior, HTTP route, telemetry,
retry, sender or live account/network operation changed.
