# Retry receipt ACK dispatch — APK 2.26.43.72

**Confirmed static:** APK version code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
These are complete local checked-DEX exports, not REA Evidence envelopes.
**Observed:** no runtime, account traffic or native execution. **Inferred:**
the selected non-BX2 dispatch is relevant to Rust's ordinary receipt handling;
complete upstream handler registration remains unproved. **Unknown:** BX2's full
semantics, downstream Android Message76 dispatch to the final writer, and when
all asynchronous retry work completes relative to ACK submission.

## Confirmed dispatch distinctions

`160` extends14h and registers integer type2. Its A04 invokesF10.A04; a null
parser result returnsfalse at0fee. This includes the parser's unsupported-version
case. `14h.A00` invokes the handler at00a0; false branches at00b4 to2Cx.A00 at016e.
That helper emits local diagnostic490 but adds **error487** to an Efa copy and
freezes it. Do not send490 on the wire. With BX2false it calls0Sf.A0O; BX2true
instead queues2Dx through1ic.A0G, retaining487 as its error argument.

A true handler result takes the ordinary0Sf.A0O branch when BX2false, after an
optional A02 child. This does not establish successful delivery or completion of
work scheduled by the handler. BX2true uses separate1ic.A0G completion bookkeeping.
The try table explicitly rethrows SQLiteFullException/SQLiteDatabaseCorruptException.
Other exceptions reach0176; BX2true calls1ic.A0E. On BX2false, an LIL parse error
reaches1rg.A04; other exceptions are rethrown. 1rg's class initializer explicitly
includes receipt in its allowed set (also notification/message/call/status).
A04 callsA03(node,487), whose parsing-error context construction is retained in
[the prior bundle](../2.26.43.72-retry-original-chat/parse-error-context-reader.txt).
This is distinct from simply returningfalse.

0Sf.A0O passes BX2 toA07. 0Sf.A0S explicitly bypasses its receipt feature-flag test
when context.type is retry; after its other tag checks it reachesA07(false).
A07(false) calls1i8.A03 thenA0V; A03 creates Android Message with arg1=76 and the
Eoh context as obj. A07(true) emits ccq_android_ack_bypassed and returns without
that message. The final Message76 consumer binding is still open. The previously
retained [F3e.A05 writer](../2.26.43.72-prekey-upload-replies/prekey-ack-wire-builder.txt)
constructs ack with context id/to/class/type/participant/recipient/edit and copies
extra attributes, but this chunk does not claim to close the intervening binding.

## Actual receipt context factory located

14h.BLn callsF0R.A03(node,null,loggableStanzaId) at007e before scheduling its
selector10 runnable. F0R.A03 callsA00 with null from override, class receipt,
default type delivery, then sets recipient/category/edit and freezes. A00 reads
root from, required id, participant and type (default only when absent), storing
from inEfa.A02. This closes the factory gap left by earlier candidate-only
ACK/NACK/message builders for this base-handler entrypoint; complete factory/
registration dispatch and alias policy remain separate unknowns.

## Rust countercheck and implementation boundary

`messaging::received_confirmation` delegates receipts to receipt_ack, whose
receipt_kind admits only delivery/read/played. Retry is excluded regardless of
whether parse_retry succeeds. Thus the earlier suspicion that malformed retries
currently receive a plain ACK was **refuted**. Existing tests explicitly assert
retry gets no confirmation. No ACK code is changed here: the remaining real gap
is missing retry ACK/error policy, not accidental success ACKs. A local unsupported
keys/count/recipient refusal must not automatically become a native parse error.

## Reproduction

From the repository, use `target/debug/xtask apk-smali <pinned-apk> --class
'LX/CLASS;' --method METHOD --signatures --exceptions`. Exports cover14h(all),
14h.A00,160(all),2Cx.A00,1rg.A04/<clinit>,0Sf.A0O/A0S/A07,1i8.A03,F0R.A03/A00.
Reproduction uses the existing offline developer CLI, no new command or route.
`checksums.json` records bytes/SHA256 for every other file in this bundle.
