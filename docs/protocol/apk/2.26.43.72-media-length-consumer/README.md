# Stored upload length consumer — APK2.26.43.72

**Confirmed selected static**, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Verified reconstructed ARM64 libwhatsappmerged.so SHA256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
Host ELF/LLVM inspection only; no target/library/account execution. Follow-through
of the [conditional native length handlers](../2.26.43.72-media-native-length-handlers/README.md).

## Selected read and callback dispatch

Complete selected routine51aa80..51abdc obtains an owner through517ecc; null
owner skips the work. Helper724748 retains incoming x1 inx19, the increment.
When an optional object atowner+0x280 exists, its counter+0x68 is decreased by
the increment under its mutex+0x10, with conditional notify_all. This separate
queue/accounting counter is not the declared length field.

51aae0..51aae8 adds that increment toowner+0x318. It then locks the same
owner+0x290 mutex used by the preceding MNS length setter, loadsowner+0x320
intox23 at51aaf4, and unlocks. The accumulated count and loaded declared value
are captured together at51ab24 into[sp+0x10,sp+0x18]. Capture also retains the
owner's callback-related pointer values+0x250/+0x258 and dispatch object+0x260.

The routine builds a closure whose invoke pointer is51abdc and passes it through
an indirect dispatch. Closure51abdc..51abfc loads x1 fromcapture+0, x2 from+8,
and object pointer from+0x10. Helper72f330 loads that object's vtable and moves
it tox0; the closure tail-branches through slot0x30. Thus the selected path
passes (accumulated count, declared length) to a downstream callback unchanged.
It contains no comparison of those two values or HTTP header construction.
No callback execution, final sink or full pointer ownership is observed.

**Inferred:** this pair is upload-progress reporting. The counter update,
paired declared total and callback shape support that interpretation; matching
onUploadProgress strings exist in this library. Literal presence alone does
not bind that exact vtable target, so this is not labelled a confirmed method
name. The unrelated JNI length-report stream slot0x30 and this callback's
slot0x30 belong to different interfaces; equal offsets do NOT identify a method.

The partial registration window5206e0..52079c installs function pointer51aa80
atsp+0xf0 among other callbacks before call53b9c4. Its full parameter meanings
and complete transport setup are untraced. The owner resolver517ecc unwraps
parameters and calls5573c8, also used by the prior setter; pointer deobfuscation
and runtime alias identity are not completely resolved. Matching field/mutex
layout supports the selected producer/consumer association, not a whole-program
alias proof.

## Search coverage and limits

Full pinned .text disassembly yielded207 textual instructions containing exact
operand suffix #0x320]. All are retained in offset-census.txt; stack fields,
unrelated object types, stores and reads are mixed. This is NOT207 accesses to
this upload field. The candidate was selected for matching mutex/layout and
callback context. The census excludes aliased base adjustments, other immediate
formats, indexed accesses, optimized copies and indirect consumers. It cannot
prove there are no other readers or framing/enforcement paths.

The selected declaration-to-callback trace provides no basis for truncating
Rust's upload body or changing Content-Length. Its known byte body includes the
finished-voice seal. Native final framing, effective0x666 and concrete transport
binding remain unknown. No Rust behavior, reservation, replay or account changed.
Next scope: callback target if needed, or actual stream/provider framing; do not
keep treating this progress-shaped field consumer as evidence of enforcement.

## Reproduction

From repository root, using the already reconstructed pinned library:

```sh
LIB=../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
python3 docs/protocol/apk/2.26.43.72-media-length-consumer/inspect-consumer.py "$LIB" > windows.json
llvm-objdump -d --no-show-raw-insn "$LIB" | rg '#0x320\]' > offset-census.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x51aa80 --stop-address=0x51ac1c "$LIB" > consumer.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x72f330 --stop-address=0x72f33c "$LIB" > callback-helper.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x5206e0 --stop-address=0x52079c "$LIB" > consumer-registration.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x724748 --stop-address=0x724768 "$LIB" > consumer-arguments.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x517ecc --stop-address=0x517f50 "$LIB" > owner-resolver.txt
strings -t x "$LIB" | rg 'onUploadProgress|uploadProgress|onBodySent|bytesUploaded' > progress-literals.txt
```

LLVM23.1.3, Python stdlib and existing digest-pinned ELF resolver. Numeric
addresses are authoritative; nearest exported-symbol headings are not function
identities. windows.json pins selected bytes. checksums.json covers all other
files with byte count and SHA256. No bulk binary/disassembly is committed.
