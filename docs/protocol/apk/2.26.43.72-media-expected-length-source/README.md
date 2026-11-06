# Queue expected length comes from request Content-Length — APK2.26.43.72

**Confirmed selected static**, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Verified reconstructed ARM64 libwhatsappmerged.so SHA256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
No target/library/account execution. This resolves the immediate initializer
question from the [native queue length check](../2.26.43.72-media-queue-length-check/README.md).
Host ELF/LLVM evidence, not new REA native decompilation.

## Actual initializer, including paired store

Selected entry53b9c4 retains its second argument inx23. On a selected predicate
branch it calls540018 with that request/header object and key788e58. The key's
pointer slot788e80 has R_AARCH64_RELATIVE addend e0240, whose NUL-terminated
string is content-length. inspect-source.py verifies this relocation/literal;
this is stronger than a nearby matching log string or guessed field name.

If lookup returns nonnull it converts through642378 and retains the int result
inw26 at53ba38. Missing lookup/other selected predicate branch setsw26=0 at53ba40.
The other predicate54ec58 has not been fully interpreted; do not claim every
request necessarily takes the lookup path.

53ba64..53ba74 admits construction only forw26>0 ORw26==-1. Other values branch
past this selected queue allocation. The136-byte allocation at53ba98..53ba9c
initializes +0x74 through a PAIRED store at53baa8:

```text
STP WZR,W26,[X0,#0x70]  -> +0x70=0, +0x74=converted Content-Length
STR WZR,[X0,#0x78]     -> cumulative queued bytes=0
```

That paired write explains why searching only explicit stores at#0x74 missed
the initializer. The second same constraint guards an abort edge at53bad0.
The normal selected construction transfers this pointer onward via53af6c and
53a53c. Complete owner lifetime, all callbacks, other construction branches and
dynamic ordinary-upload selection remain outside this bounded window.

The preceding queue entry5432b8 reads exactly+0x74/+0x78 and rejects bytes above
the former unless-1. Its expected-size input is therefore the selected request's
header value, not automatically JNI getContentLength orowner+0x320 progress total.
No chain proving which component injected an absent explicit header is established.

## Header lookup and conversion boundaries

540018 and54edb0 resolve a key in the object's header collection/dictionary and
return a nullable string throughMCFDictionaryGetValue and type checking. Complete
methods are retained; the MCF container/key infrastructure is not universally
reconstructed. The separate542d90..542df8 copy window uses the same key and
conversion, corroborating its role; the whole copy routine remains untraced.

Complete selected conversion642378..642418 copies a string to a C buffer,
then calls strtol with endptr=null and base0. It saturates the result to
[-2147483647,2147483647], deallocates the buffer and returns the32-bit result;
missing copied buffer returns0. There is no end-pointer/errno validation in
this wrapper. This is NOT an instruction to accept malformed or base-prefixed
HTTP lengths in Rust: the current client derives a canonical length from its
known bytes, rather than taking arbitrary header text from a caller.

The partial53a980..53ac80 request-validation window uses the same key and
conversion and contains negative-length handling. Its earlier entry conditions,
full control flow and all exceptions are not closed, so this chunk does not
invent a complete header-validation policy. The inspected633c80..633e00 queue
core window is also partial and supplies no further header-source claim.

## Search exclusions and reproducibility limits

exact-store-census.txt contains43 exact STR/STP/STUR operand matches for#0x74]
across the disassembled.text, including unrelated stack/object/vector stores.
The apparent51c594 candidate is a validated configuration store: literal
http2MaxInFlightRequestsPerConnection binds it to settings, not request length.
Its complete selected surrounding window is retained as queue-initializer.txt
(an initial candidate filename, NOT an accepted queue initializer).

The nearby53xxxx census additionally includes starts at0x6c/0x70/0x78 and finds
actual paired53baa8. Neither is an alias-complete field census: adjusted bases,
other widths, offsets and compiler transformations remain possible. An initial
rg -n address-range filter also matched printed line numbers; it was replaced
with parsing the instruction address before filtering. No conclusions rely on
those noisy hits. The545bd4/545bd8 zero/float initialization inspected in the
full local disassembly was a different allocation; no queue binding was found.
No broad unknown-object write is promoted into a request field.

## Rust consequence and remaining scope

Rust image_upload reserves the exact wire_body, includes the finished-voice seal
where applicable, and passes that known byte body to reqwest. A length-aware
request therefore uses the actual sent byte length. The native queue finding
supports preserving that relationship; it does not justify using the encrypted
hash-input length (which excludes the seal), truncating bytes, adding guessed
resume ranges or suppressing Content-Length based on the separate Pando flag.
No Rust behavior changed; existing uncertain uploads stay frozen and unreplayed.

Remaining: full predicate54ec58, concrete upload/request object selection,
native synthesis when JAT adds no explicit length, and entire wire stack. The
actual initializer is now closed; full upload framing parity is still unproven.

## Reproduction

From repository root, with existing recovered pinned library:

```sh
LIB=../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
python3 docs/protocol/apk/2.26.43.72-media-expected-length-source/inspect-source.py "$LIB" > windows.json
llvm-objdump -d --no-show-raw-insn --start-address=0x53b9c4 --stop-address=0x53baec "$LIB" > actual-queue-init.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x540018 --stop-address=0x54006c "$LIB" > header-lookup.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x54edb0 --stop-address=0x54ee7c "$LIB" > key-resolution.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x642378 --stop-address=0x642418 "$LIB" > number-conversion.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x542d90 --stop-address=0x542df8 "$LIB" > header-copy-binding.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x51c510 --stop-address=0x51c640 "$LIB" > queue-initializer.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x53a980 --stop-address=0x53ac80 "$LIB" > header-reader.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x633c80 --stop-address=0x633e00 "$LIB" > queue-core.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x545b68 --stop-address=0x545bdc "$LIB" > excluded-allocation.txt
python3 docs/protocol/apk/2.26.43.72-media-expected-length-source/search-stores.py "$LIB" /tmp/wa-expected-search-repro
cmp /tmp/wa-expected-search-repro/exact-store-census.txt docs/protocol/apk/2.26.43.72-media-expected-length-source/exact-store-census.txt
cmp /tmp/wa-expected-search-repro/near-store-census.txt docs/protocol/apk/2.26.43.72-media-expected-length-source/near-store-census.txt
```

Censuses: disassemble.text with the same LLVM23.1.3 command, parse each line's
hex instruction address, select STR/STP/STUR with exact#0x74] for the full
census; for near census select addresses[0x530000,0x540000) and exact offsets
{0x6c,0x70,0x74,0x78}. These are physical operand searches only. Python stdlib
and existing digest-pinned ELF resolver; no new dependency. windows.json pins
selected bytes/key. checksums.json covers every other file with bytes/SHA256.
LLVM nearest-symbol labels are not accepted function names.
