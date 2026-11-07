# Ordinary receipt parse-error dispatch and send-gate correction — APK 2.26.43.72

**Confirmed selected static / implementation gap**, build 264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX methods/catch tables and static
reference searches from the existing xtask fallback. No native execution,
account traffic or REA Evidence envelopes. Rust runtime is unchanged here.

## The ordinary parser reaches the common exception handler

[Exact F10.A03 callers](parser-callers.txt) are 15H.A04 and 19t.A04.
[15H constructor](ordinary-registration.txt) registers integer handler type1
through 14h; [15H.A04](ordinary-handler.txt) calls F10.A03 at0048, then F21.A05.
Its catch-all removes the context from its pending set and rethrows: a parser
LIL is not silently swallowed there. [19t.A04](status-handler.txt) also calls
F10.A03 and dispatches its event to Ens.A01; its other participant/status-specific
admission is not generalized to ordinary receipts by this trace.

The retained [14h base handler](../2.26.43.72-retry-ack-dispatch/base-handler.txt)
invokes A04 inside its exception region. SQLiteFullException and
SQLiteDatabaseCorruptException are explicitly rethrown. Other exceptions reach
0176. A false BX2 tests for LIL at01a6 and invokes 1rg.A04 at01d2; non-LIL is
rethrown. A true BX2 calls 1ic.A0E at019e.

[Complete declared BX2 override census](wrapper-flags.txt) across 114178 classes
finds interface 1k2 plus 1k3 returning constant true and 1uU returning constant
false. This resolves the selected wrapper flag, not all runtime dispatch or
external/inherited-default possibilities. [1ic.A0E](queued-error-handler.txt)
checks LIL and calls the SAME 1rg.A04 at0066 when the queued stanza has a node.
It then constructs completion bookkeeping 2Dx with 487 for LIL or 500 for other
throwables. Non-LIL with a node instead invokes 1rg.A00. Complete downstream
queue cleanup, retries and duplicate suppression are not inferred here.

[1rg.A04](../2.26.43.72-retry-ack-dispatch/parse-error-policy.txt) includes receipt
in its allowlist and calls A03(node,487) at0128..0132, under its own catch table.
[Fresh-context A03](../2.26.43.72-retry-original-chat/parse-error-context-reader.txt)
sets class/id/error, source, optional original type and generic participant;
it does NOT copy recipient/edit/category or aggregate children. It sends through
0Sf.A0S. Descriptor construction and eventual write/remote acceptance differ.

## Corrected prior false interpretation: the flag logs, then falls through

The earlier retry-dispatch README incorrectly said A07(true) logs a bypass and
returns without producing Message76. A fresh complete 0Sf.A07 export matched
[the retained bytes](../2.26.43.72-retry-ack-dispatch/sender-common.txt) exactly:
false branches at0000 to0058; true logs at004e..0058 and FALLS THROUGH to0058.
Both call 1i8.A03 at0060 and A0V at006e before the only return at0074.
The old conclusion was an interpretation error, not a changed APK.

Consequently the [0Sf.A0S](../2.26.43.72-retry-ack-dispatch/sender-context.txt)
receipt/non-retry feature flag 0x382c selects that A07 boolean; this is not a
wire-suppression gate at this method. Retry bypasses that flag check. Do not
infer absence of a NACK from the diagnostic name ccq_android_ack_bypassed.
This resolves the local branch, not every upstream policy or queue operation.

## Reuse the already-closed final writer binding

Fresh 1i8.A03, 1i8.A07 and 0Sf.A0V exports matched their retained files exactly.
The [later writer-binding bundle](../2.26.43.72-retry-version-error-ack/README.md)
already closes Message76 conversion/direct writer paths to F3e.A05, including
readiness/pre-ACK conversion. [Writer-builder callers](writer-builder-callers.txt)
and [conversion callers](conversion-callers.txt) are retained as exact static
search results; caller presence does not prove a runtime route. The same
existing [conversion](../2.26.43.72-prekey-upload-replies/prekey-ack-conversion.txt)
selects Message76 at0018 ->0202 and calls F3e.A05 at0214.
No duplicate copies of those exports are retained here.

## Consequence for Rust and next implementation boundary

Required DeviceJid failures in offline aggregates are proven native LIL cases,
not merely a local inability to process a feature. Both selected wrapper error
routes reach the parsing-error descriptor with 487. Current Rust validation
refuses the ordinary aggregate without sending that error ACK; this is now an
explicit implementation gap, not a native requirement to remain silent.

Before porting the response, introduce a distinguishable native parse-error
cause for evidenced constructor/cast failures. Do not map every
receipt.aggregate error to487: local bounds, unsupported features, stored-row
corruption and storage failures have different authority. Construct the fresh
root descriptor (not an ordinary success copy), retain capture-before-write,
apply selected address/writer guards and test error response, no partial facts,
no resend/query/identity changes and redelivery handling. Keep unsupported
metadata/registration paths and actual server acceptance labelled separately.
This research chunk does not change outgoing traffic or enable account use.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-callers "$APK" --class 'LX/F10;' --method A03
target/debug/xtask apk-smali "$APK" --class 'LX/15H;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/15H;' --method A04 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/19t;' --method A04 --signatures --exceptions
target/debug/xtask apk-virtual-overrides "$APK" --class 'LX/1k2;' --method BX2 --signature '()Z' --include-interfaces
target/debug/xtask apk-smali "$APK" --class 'LX/1ic;' --method A0E --signatures --exceptions
target/debug/xtask apk-callers "$APK" --class 'LX/F3e;' --method A05
target/debug/xtask apk-callers "$APK" --class 'LX/1i8;' --method A07
```

Fresh comparison exports use apk-smali with 0Sf.A07/A0V, 1i8.A03/A07 and
1rg.A04. Compare respectively to retry-ack-dispatch/sender-common.txt,
retry-version-error-ack/A0V.txt, retry-ack-dispatch/ack-message-factory.txt,
prekey-upload-replies/prekey-ack-conversion.txt and
retry-ack-dispatch/parse-error-policy.txt. The comparisons all matched.
checksums.json covers every other file. No APK, private state or bulk tree.
