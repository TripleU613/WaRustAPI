# Outgoing retry error mapping — APK 2.26.43.72

**Confirmed static / research only.** Version code264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected checked-DEX exports retain signatures and exception tables;
these local exports are not REA Evidence envelopes. **Observed:** no Android
execution, account traffic or acceptance of a retry error code. **Inferred:**
Rust must preserve a typed eligible cause before choosing a wire retry error.
**Unknown:** complete exception-class identity/primitive parity, every failure
producer, route/capability admission and full native retry policy. No code added
to Rust, no guessed error default, and no success/recovery claim.

## Three distinct number domains

[The prior writer trace](../2.26.43.72-outgoing-retry-timestamp/README.md) establishes
F3e.A08 always adds retry/@error fromEYI.A04. EuT.A01 takes that **wire** value,
passes it directly intoEYI orSendRetryReceiptJob.retryErrorCode, and that named
job field reachesEYI.A04. It is not receipt-ACK error487.

For the selected crypto path,F2q.A01 consumesEtY.A00 **crypto status**. Only after
EtY.A00()'s predicate succeeds does it setF1Z.A00, an **internal failure kind**.
F2q.A02 andE5U.A0B read that kind, invokeDxB.A00 and pass its result toEuT.A01.
NonpersistentF2q.A0A instead forwards its explicit final integer directly; its
other integer can separately updateF1Z.A00. Conflating those parameters would
send a failure-kind enum where a wire error is expected.

## Eligible selected crypto-result conversion

EtY.A00() accepts exactly these status values in its retained body. F2q.A01 then
selects the internal kind, andDxB.A00 produces the last column:

| Crypto status | Internal kind | Wire retry error |
| --- | --- | --- |
| -1008 | 1 | 1 |
| -1002 | 2 | 2 |
| -1003 | 3 | 3 |
| -1005 | 4 | 4 |
| -1206 | 2 | 2 |
| -1205 | 13 | 8 |
| -1204 | 12 | 7 |
| -1203 | 11 | 5 |
| -1202 | 10 | 6 |

DxB independently maps kind8→11 and9→10, and otherwise defaults to0 outside its
explicit1..4/8..13 branches. That default is not proof every decryption failure
should send0: the selected crypto path admits a defined subset first. Status0,
-1000, -1001, -1006 and-1007 are not inEtY.A00's eligible set. F2q has separate
handling for other outcomes; this table is not a universal no-retry statement.

## Concrete native string refinements, with their guard

EtY(String,int) refines a **nonempty message only when incoming status=-1005**:

- exact Invalid signature! => -1203 => wire5;
- startsWith Unknown type: / No keys for: / Bad key type: => -1002 => wire2;
- exact Over 2000 messages into the future! => -1202 => wire6;
- exact Bad Mac! => -1204 => wire7;
- exact No valid sessions. => -1205 => wire8.

3q4.A1Y is explicitly String.startsWith with the retained argument order.
Other messages/statuses preserve the supplied status. These strings are native
classifier evidence, **not** instructions to parse arbitrary Rust/provider error
messages or log peer-bearing exception text. Rust already redacts by typed
SignalProtocolError variants; its generic direct_message/Undecryptable boundary
loses that specificity before the outgoing retry queue. A typed bridge is still
needed rather than assigning a blanket value from the word undecryptable.

EHq.A00 additionally handles successful plaintext/padding and explicit failure
status. Empty/null plaintext becomes-1000; invalid/entire padding becomes-10000.
For nonzero status it constructsEtY with the exception message/status, so the
refinement above is an actual downstream producer path, not just unused strings.

## Producer scope and negative results

0WJ.A0O is explicitly decryptForIndividualPreKey. Its retained catch table maps
ED4→-1001,EDY→-1005,EDR→-1003,EDQ→-1002,2xu→-1010,ED6→-1007 and other eligible
Exception→-1000, with special early ED5→-1006 and EDY→-1005 paths. Storage/explicit
special exceptions can rethrow. Class constructors forEDR/EDY/EDQ/ED4 only expose
generic Exception/String/Throwable structure; they do **not** establish names
like NoSession or InvalidKeyId. Do not assign those obfuscated types to Rust
variants from a guessed naming convention. Other Signal primitives remain open.

The F1Z.A00 writer inventory findsnine writes acrossF0b/F2q/Fc1. F0b's retained
message-processing path can setkind9 for an ED9 processing failure; that is not
proof of a cipher MAC error. F2q.A04 has device/ADV/nonpersistent paths; its
explicit A0A calls include wire12 and11. 1v2.A00 directly callsEuT.A01 with13.
Those bypassDxB, so its0..11 outputs are **not** a universal wire-error maximum.
The five EuT.A01 caller sites are retained; untraced Fc1 writer causes remain
unknown. Complete selected admission methods are retained to preserve their
conditions rather than presenting isolated constants as universal rules.

## Port consequence and reproduction

The current Rust timestamp fix remains unchanged. Do not add retry.error=487,
error0 for every failure, or a restriction rejecting all errors>11. First carry
a typed failure cause through direct/group/own-device processing, map only
semantically proven causes, and preserve native request eligibility separately
from error serialization. Unsupported content, duplicate/counter outcomes,
identity policy refusals and storage faults must not collapse into one generic
retry request. Full error/key/count/routing parity is still unfinished.

Reproduce each exported signature using `target/debug/xtask apk-smali <apk>
--class 'LX/CLASS;' --method METHOD --signatures --exceptions`; complete small
EtY/exception classes use no method filter. apk-callers onEuT.A01,F2q.A0A and
EtY.<init> reproduces inventories; apk-field-refs --class 'LX/F1Z;' --field A00
--type I --writes-only reproduces the failure-kind writes. Linked prior bundles
retain the complete wire serializer/job admission. Checksums cover every other
file here.
