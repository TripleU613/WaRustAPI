# Native request header transfer and timeout normalization — APK2.26.43.72

**Confirmed selected static**, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Verified reconstructed ARM64 libwhatsappmerged.so SHA256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
Host ELF/LLVM inspection, not REA native decompilation. No library/target/account
execution. Follows the [registered provider chain](../2.26.43.72-media-jni-registration/README.md)
and retained651860 decoder prefix.

## Header container reaches the native request

The [complete Java1Fb.A01 serializer](../2.26.43.72-media-java-native-submit/request-serializer.txt)'s order is method,
URL, header map, priority/retry fields, then connection/idle/request/response
millisecond longs. The [earlier decoder prefix](../2.26.43.72-media-native-submit-provider/converter.txt) reads the two strings, then a
count and header pairs. The new continuation651b64..651fa0 reads later fields
and constructs a native request object with base atsp+0x80.

651d34..651d5c moves the decoded map's pointer/state words tosp+0x1b0, exactly
native request+0x130, clearing the old temporary ownership words. Selected
nonempty-container bookkeeping updates an internal pointer afterwards. Exported
TigonRequest::addHeader at663844 uses helper7312b4, which adds0x130 tothis before
its map insertion call. That independently supports the member identification.
This is movement of the decoded container, not a new computed Content-Length
entry in this block. Full container internals, duplicate-key policy and later
middleware are not claimed complete.

Selected optional property paths and pointer wrapping are retained through
652080. They include indirect calls and property-container operations whose
complete effect is unresolved. Do not infer that preserving the header map at
one stage means it cannot change later. The retained request/property windows
are prefixes/continuations, not the full large651860 routine or wire output.

## Four timeout inputs pass through validateTimeout

The earlier prefix reads four consecutive64-bit fields in Java order. The
continuation routes each through66377c and stores its result:

| Serialized Java field | Callsite | Native request offset |
|---|---|---|
| connectionTimeoutMS | 0x651e0c | +0x1b8 |
| idleTimeoutMS | 0x651e2c | +0x1c0 |
| requestTimeoutMS | 0x651e20 | +0x1c8 |
| responseTimeoutMS | 0x651e38 | +0x1d0 |

The idle call at0x651e2c uses the stack temporary atsp+0x28 and stores
atsp+0x240.
The request call at651e20 uses the retained request-timeout register and stores
atsp+0x248. Connection stores atsp+0x238; response atsp+0x250. Do not conflate
native field order with register/stack temporary order. Exported request/idle
setters66370c/663744 independently call the same helper and store+0x1c8/+0x1c0.

Complete anonymous helper66377c..663844 is identified by its validateTimeout and
TigonRequest.cpp diagnostic literals, not the misleading nearest setter heading.
On normal return:

```text
input == 0       -> unchanged 0
input > 999      -> unchanged input (signed comparison)
otherwise        -> report soft error, return 1000
```

Thus the below1000 branch includes negative signed values; it is not merely a
positive-subsecond test. It formats Timeout is too small and calls exported
reportSoftError before returning1000. Formatting/exception exits remain in the
complete method; no claim all diagnostic paths return normally. Prologue and
stack-canary epilogue helpers are retained to verify they preserve the input/
return register along the normal branch. This is native request-object timeout
normalization, not proof of actual socket scheduling or a server requirement.

## Rust scope

Current Rust image_upload builds reqwest with configured TOTAL timeout_ms and
connect_timeout_ms. Defaults in request_config are45000/30000ms; validation
requires positive values and connect<=total, allowing shorter user-specified
local bounds. These are existing deliberate local limits, separate from native
idle/request/response fields and per-operation callback deadlines. No native
1s floor is silently applied to those Rust bounds: extending an explicitly
short local deadline is not justified by a different layer's request validator.
No mandatory network mismatch or ban-prevention effect is established here.

Remaining: downstream property/middleware effects, native automatic headers,
actual transport/frame construction, effective timeout configuration and all
runtime scheduling. No Rust behavior, deadline, durable upload or account state
changed; no replay/live validation. The actual native header-member transfer and
four-field normalization are now retained with exact offsets.

## Reproduction

From repository root with existing recovered pinned library:

```sh
LIB=../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
python3 docs/protocol/apk/2.26.43.72-media-request-conversion/inspect-conversion.py "$LIB" > windows.json
llvm-objdump -d --no-show-raw-insn --start-address=0x651b64 --stop-address=0x651fa0 "$LIB" > request-transfer.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x651fa0 --stop-address=0x652080 "$LIB" > request-property.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x66377c --stop-address=0x663844 "$LIB" > timeout-normalizer.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x66370c --stop-address=0x66377c "$LIB" > timeout-setters.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x725af8 --stop-address=0x725b08 "$LIB" > timeout-prologue.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x725ab8 --stop-address=0x725ac8 "$LIB" > timeout-epilogue.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x663844 --stop-address=0x66386c "$LIB" > header-member.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x7312b4 --stop-address=0x7312c0 "$LIB" > header-member-helper.txt
```

LLVM23.1.3, Python stdlib and prior hash-pinned ELF resolver. Numeric entries
identify routines; nearest exported-symbol headings are not attribution.
windows.json pins selected bytes/literals; checksums.json covers every other
file with bytes/SHA256. Full binaries remain local.
