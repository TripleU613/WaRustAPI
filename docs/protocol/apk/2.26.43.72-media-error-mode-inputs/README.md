# Media error response mode inputs — APK 2.26.43.72

**Confirmed selected static / research only.** Build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX fallback exports; no execution, new REA Evidence ID,
account traffic or Rust runtime change. Method PCs are bytes; branch offsets
are16-bit units. This narrows the prior
[conditional error491 boundary](../2.26.43.72-media-parser-error-boundary/README.md).

## The receipt boolean has a native name and distinct callers

[Constructor](processor-constructor.txt) stores its first boolean in F0b.A06
at001c and initializes the mutable F0b.A00 action flag TRUE at0020. The exact
[A06 write census](receipt-mode-writers.txt) has one direct DEX writer, this
constructor. This does not cover reflection/native mutation. The prior retained
[EdS.A00](../2.26.43.72-media-parser-error-boundary/receive-error-consumer.txt)
passes its first boolean v47 into constructor argument v18 at003a; it reads A06
at0050 and logs it as `sendReceipt=` at0074. Thus the name is native evidence,
not a guess from ACK behavior. It is not itself proof a receipt was transmitted.

[EdS direct caller census](plaintext-callers.txt) finds two direct invokes:

| Caller | First boolean / sendReceipt | Remaining booleans |
| --- | --- | --- |
| FKc.BLW | FKc.A08 inherited from EmK.A04 | false, false |
| FSC.BLN | true | true, true |

[FKc.BLW](decryption-callback.txt), logged DecryptionCallbackV2/handlePlaintext,
loads A08 into v15 at0014 and preserves it into EdS call range v7..v17 at0192.
Its [constructor](callback-constructor.txt) copies EmK.A04 into A08 at0072..007a.
BLW has feature/routing branches, including an explicit parse-failure legacy
fallback; this table applies when execution reaches that selected EdS invoke,
not every decrypted payload. EmK.A04 producer policy remains unknown.

[FSC.BLN](reprocess-callback.txt) loads true into v12 at0008, copies it into
v13/v14 at009a/009e and invokes EdS with range v4..v14 at00a0. Its supplied
boolean parameter v24 is not used to select these three arguments. This is a
bounded reprocessing candidate (takes1k8 bytes and retrieves/builds E0A); the full
interface/caller lifecycle and ordinary incoming profile binding remain unknown.
Do not assume an arbitrary public boolean controls this sendReceipt argument.

## Retry flag is set before queue success

[Exact F1Z.A03 writer census](retry-flag-writers.txt) has five direct sites:
EuT.A01, F2q.A01/A03/A09 and Fc1.run. Only EuT is newly followed here; the other
four and reset/default lifecycle remain unknown. The prior F0b.A01 logs this
field as isSendRetry and suppresses the selected error action when it is true.

[EuT.A01](retry-producer.txt) logs RetryRequest/send retry receipt, then writes
TRUE to F1Z.A03 at004e..0054, before subsequent session/key/future branches.
On the selected later path it constructs SendRetryReceiptJob at0650, invokes its
constructor at065c and enqueues through0fW.A01 at0662. Consequently this flag
cannot be treated as proof of queue success, socket transmission or server
acceptance. This trace does not classify every branch or establish retry replay
permission after uncertain outcomes.

## Action flag also has stop paths

[Exact F0b.A00 census](action-flag-refs.txt) has ten direct reads/writes across
EdS.A00, the constructor, F0b.A00/A01/A02. Initial TRUE is not unconditional
permission to act throughout processing. [F0b.A02](processor-stop-actions.txt)
checks18w.A03's typed result: FSU logs stop/recent, conditionally calls0x0.A0Y
when sendReceipt is true, then clears A00 at0052 and returns false. FSS logs
stop/ack and calls F0b.A01 with its stored original reason before returning
false; FST is the expected Continue case. The semantics of18w.A03,0x0.A0Y and
other A00 writers are not fully classified here. In particular stop/recent must
not be relabelled as a parser error491 simply because processing stops.

## Integration boundary

**Confirmed:** selected error action requires mutable processing permission,
sendReceipt and no isSendRetry. The first has additional stop behavior; the
second differs between selected callbacks; the third can precede queue success.
**Inferred:** replacing these with a single Rust unsupported-message boolean
would collapse distinct lifecycle states and could issue a response in the wrong
context. **Unknown:** EmK.A04 origin, four other retry writers, complete flag
clearing, BX2 branch, native invalid media versus local unsupported features,
durable response ownership and direct writer dispatch. No error491 emission or
retry change is justified by these mode inputs alone. Next follow EmK producers,
remaining retry writers and typed stop-result producers before runtime wiring.

Reproduce `apk-smali <apk> --class '<descriptor>' --method <name> --signatures
--exceptions` for F0b.<init>/A02,FKc.<init>/BLW,FSC.BLN,EuT.A01. The copied direct
reference censuses retain their full selected output; prior EdS/failure bodies
stay pinned by their original bundle. [checksums.json](checksums.json) covers
all files here. This is protocol research, not evidence of ban causality or any
zero-ban guarantee.
