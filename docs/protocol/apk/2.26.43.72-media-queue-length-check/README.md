# Native queue length check and Pando-only getter — APK2.26.43.72

**Confirmed selected static**, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Verified reconstructed ARM64 libwhatsappmerged.so SHA256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
No library/target/account execution. Host LLVM23.1.3 and checked DEX, not new
REA native decompilation. Follows the [count/total callback trace](../2.26.43.72-media-length-consumer/README.md).

## Separate queue enforces a declared bound

Complete selected native entry5432b8..54339c obtains a queue-related pointer
through its holder (helper2e2bc0 retains x0 and loads its first pointer; entry
loads through that pointer). It gets incoming buffer size fromargument+0x18,
or0 if null, then reads two32-bit values atqueue+0x74/+0x78. Literal bindings
name these expectedBytes and cumulativeQueuedBytes respectively.

5432e8 adds incoming size to cumulative and stores the result at+0x78.
5432ec compares new cumulative with expected as signed32-bit values.
5432f4 CCMN expected,#1,#4,GT and5432f8 B.NE select the error path exactly when
new cumulative > expected AND expected != -1 (for the represented values).
Expected-1 bypasses the declared bound. Equality is allowed. This is the
instruction-level branch, not a claim of safe behavior after integer overflow:
32-bit addition wraps; callers' size validity and range guarantees are untraced.
Do not copy its integer arithmetic into Rust as a new input policy.

On the selected non-error branch it calls633d08 and returns1; the callee's
complete enqueue/write result is not established. On error it optionally logs
MNSBufferQueueAppendBytes: body exceeds content-length with both counters,
subtracts this incoming size back out of+0x78, constructs local error text
Request body size exceeds content-length viaMNSUtilCreateMCFErrorWithFormat,
and delivers the error through5406b0, returning0. This is local queue failure,
not a server refusal, proof no earlier bytes escaped, or replay permission.

Two retained partial caller windows53c288..53c2c4 and53c530..53c554 call this
entry, including a nonempty-buffer check and result branch at53c2a0..53c2b4.
Their complete state machine and dynamic selection remain unknown. Nearby
header-copy window542fa0..5431e8 was inspected but does not establish the
producer ofqueue+0x74. Exact-offset store search in the nearby0x54 region found
no direct STR/STP #0x74] match; adjusted bases, paired offsets, helper writes and
other regions were not excluded. Do not infer an absent initializer.

**Critical separation:** queue+0x74 is not automatically the previous owner+0x320
progress total. No pointer/producer chain between them was established. A native
queue bound now exists in evidence, but final ordinary-upload framing and this
bound's input source remain unknown. Rust's known byte body includes its seal;
no evidence justifies replacing its real length with an unbound native value.

## Disable-header getter belongs to another namespace

Initial lookup undercom.crossapp.tigonhttp.TigonHttpClientConfig found no method
matching ContentLength. Full301-method signature inventory and exact negative
lookup are retained. APK string search finds getDisableContentLengthHeader;
whole-APK checked method search instead resolves its sole selected body to
com.facebook.pando.primaryexecution.tigon.PandoTigonConfig, returning that
object's disableContentLengthHeader Boolean. This is not an ordinary-upload
configuration binding or evidence Rust should suppress Content-Length.

Native guarded lookup window3fa81c..3fa85c references that literal and caches a
resolved target. Full receiver/class/provider binding and effective Boolean are
not established here. The same string in DEX/ELF is not proof this upload uses
Pando. No getter defaults or values are invented.

**Unknown:** queue expected-size initialization, relation to explicit HTTP header,
JNI provider length or native stack synthesis; actual transport binding; full
append/cancellation/error state machine; Pando consumer/defaults. No Rust behavior
changed or live validation. Next trace expectedBytes producer before applying
any native length value to Rust request construction.

## Reproduction

From repository root, with existing reconstructed pinned library:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
LIB=../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
python3 docs/protocol/apk/2.26.43.72-media-queue-length-check/inspect-length-check.py "$LIB" > windows.json
target/debug/xtask apk-smali "$APK" --class 'Lcom/crossapp/tigonhttp/TigonHttpClientConfig;' --signatures --exceptions | rg '# full signature:' > client-config-inventory.txt
target/debug/xtask apk-smali "$APK" --class 'Lcom/crossapp/tigonhttp/TigonHttpClientConfig;' --method ContentLength --signatures --exceptions > wrong-config-lookup.txt 2>&1
target/debug/xtask apk-strings "$APK" --contains DisableContentLength > length-apk-strings.txt
target/debug/xtask apk-smali "$APK" --class '' --method getDisableContentLengthHeader --signatures --exceptions > disable-getter.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x5432b8 --stop-address=0x54339c "$LIB" > overflow.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x542fa0 --stop-address=0x5431e8 "$LIB" > header-copy.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x3fa81c --stop-address=0x3fa85c "$LIB" > disable-native-lookup.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x53c288 --stop-address=0x53c2c4 "$LIB" > queue-caller.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x53c530 --stop-address=0x53c554 "$LIB" > queue-late-caller.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x2e2bc0 --stop-address=0x2e2bcc "$LIB" > queue-arguments.txt
```

The first apk-smali attempt omitted required --class and was corrected to the
explicit empty filter shown above; no tool change was required. windows.json
pins mapped selected bytes/literals; checksums.json covers all other files with
bytes/SHA256. Nearest LLVM symbol headings are not function identity claims.
