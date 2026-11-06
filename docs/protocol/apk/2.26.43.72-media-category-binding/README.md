# Serialized upload category/purpose reach the native request

**Confirmed selected static**, APK2.26.43.72 SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
ARM64 libwhatsappmerged.so SHA-256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
No target execution, emulator, runtime trace, account or network operation.
This resolves the selected category conversion boundary left open in the
[Priority header producer](../2.26.43.72-media-priority-header/README.md).

## Exact byte-to-member chain

The retained complete [1Fb.A01 serializer](../2.26.43.72-media-java-native-submit/request-serializer.txt)
at DEX byte0x007e..0x00a6 loads requestCategory.value and then purpose.value,
calling1Fa.A00 for each. The complete integer encoder here writes four bytes,
low8 bits first, then shifts8/16/24 with masks. It therefore writes the32-bit
bit pattern little-endian. Category and purpose follow the four64-bit timeouts;
there is no enum-name lookup in this serialization step.

The retained converter651860 prefix constructs its reader through0x65483c.
Complete reader construction here installs vtable0x778d10, initializes cursor
and limit, and retains its input reference. Its slot0x10 resolves to0x6548ec,
whose normal path returns data_pointer+old_cursor and advances cursor by the
requested count. It does not reverse or transform bytes. It stores the updated
cursor before the over-limit assertion path; this is not a claim about arbitrary
negative counts, overflow robustness or malformed-stream recovery.

Native reads after the four8-byte timeout loads request4 bytes at0x651c34,
then `LDR w21,[x0]` at0x651c44. The immediately following read requests another
4 bytes at0x651c50, then `LDR w27,[x0]` at0x651c58. Thus w21 receives category
and w27 purpose as raw32-bit little-endian words.

The native request is based atsp+0x80, established by the
[full prior transfer continuation](../2.26.43.72-media-request-conversion/request-transfer.txt).
At0x651de0, `STR w21,[sp,#0x260]` writes category to request+0x1e0.
At0x651de4, `STR w27,[sp,#0x264]` writes purpose to request+0x1e4.
There is no intermediate enum remapping, range check or fallback assignment on
this selected normal path. The native exported category setter independently
uses+0x1e0 in the prior header bundle.

The verifier decodes all four actual AArch64 instruction words, checks the
32-bit unsigned-immediate LDR/STR opcode family, source/target/base registers
and scaled offsets, and computes member offsets from request base. It pins the
reader vtable/RTTI and every retained window. The example byte pair
0800000000000000 is an offline encoding illustration for category8/purpose0,
not a captured or executed request.

## Register lifetime, including outlined helpers

register-lifetime.txt retains all caller instructions from category read through
both stores. Intermediate full-size calls use the AArch64 preserved-register
contract for x21/x27. Outlined helpers cannot simply be assumed to preserve
those registers, so the selected ones are retained and checked too:

- string-decode0x6536b8 explicitly saves x21 and restores it through0x2dbea4
  before tail-calling std::string initialization; it does not use x27;
- URL construction0x668708 saves x27 and other callee state;0x724a34 restores
  both x21 and x27 on the normal return path;
- bool normalization, map/request initialization, string-view extraction and
  argument-pair forwarding helpers affect other registers, not x21/x27.

The string-view helper's normal return and assertion trap are retained. This
is selected static normal-path value flow, not execution of the entire large
converter or a proof of all error/cold branches. Standard C++ library calls are
not newly reverse-engineered. No assertions about untrusted malformed streams
or arbitrary enum values are inferred from this mapping.

## Consequence for the standard upload

[Standard MOu metadata evidence](../2.26.43.72-media-priority-header/README.md)
sets K1R.A06=true/A05=false and yields MEDIA_UPLOAD(value8), NONE(value0).
The JAT setter -> serializer -> raw reader -> native stores now connects those
values to the8-byte pair copied by WAHttpPriorityContextInterceptor from
request+0x1e0 into its typed context. For category8, the WA calculator yields
(3,1,0) regardless of purpose or configuration flag38178. The downstream service
therefore constructs prefix `u=3, i` for that selected path.

**Still unknown:** the original writer/engagement/value of typed root property3,
which can append a positive r suffix, and live runtime/server acceptance. Static
normal graph activation and named JNI binding were already closed in the
activation bundle; those are not runtime observations. Existing non-WA contexts,
other upload factories and other transports remain separate. This turn changes
research/docs only; no Rust header, retry, timing or persistence policy changes.

## Exact reproduction

From repository root, existing xtask and LLVM23.1.3:

```sh
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1Fa;' --method A00 --signatures --exceptions
python docs/protocol/apk/2.26.43.72-media-category-binding/inspect-binding.py ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
```

Outputs reproduce integer-encoder.txt and binding.json. The verifier reuses the
digest-pinned ELF section/relocation parser from the sibling media-body-provider
bundle. For every [start,end) in native-windows.json reproduce the named.txt:

```sh
llvm-objdump -d --no-show-raw-insn --start-address=START --stop-address=END ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
```

Existing complete REA category Evidence is linked from the header bundle; no
new native REA record is fabricated where Ghidra/Hopper remain unavailable.
Numeric addresses/relocations identify routines, not nearest exported headings.
Checksums cover every file except themselves; no raw APK/native library.
