# Current read-self ACK preference synchronization

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
This re-anchors the historical37.71 read-setting trace in the pinned current
build. No APK/runtime/account execution and no production Rust behavior change.

## Wire producer is a specialized ACK side effect

Fresh1rH.A03 exactly matches the complete
[retained reader](../2.26.43.72-distribution-ack-admission/native.txt), reused here.
Its receipt class/type selection admits read-self, constructs a nonnull remote
chat/key with required ID, then01fc..021c reads the original node's root
readreceipts attribute. A nonnull value forwards the original node with selector
227 through1lZ.A1C. Other receipt types take their own branches; a bare attribute
on an arbitrary node is not equivalent. Earlier auth-challenge diversion and
key/type validation still apply. No complete JID/subtype admission is newly
reconstructed by this trace.

A1C calls A0R, whose Message.obtain arguments are what=0, arg1=selector,
arg2=0 and obj=original node.227 is an internal dispatch discriminator, not a
wire error code, stanza ID, ACK type or new request field. The selected reader
then forwards through the generic A1D path. Event scheduling before that call
is not proof that preference persistence finishes before callback settlement.
The readreceipts literal census has13 physical references; unrelated schema,
MEX and UI references are not independent producers of this event.

## Dispatch and normal enabled binding

The exact interface-ancestry census finds root1lW.CPm plus the declared1lY
implementation, not two active receivers.1uW.A00 reads Message.arg1.1lY.CPm
checks0ln.A0G(selector): true clones the message and calls A07; false sets what0
and posts through its Handler. The latter legacy route remains untraced here.
A07 sets what1/arg1=selector and posts to0m2. Its selected what1 path ultimately
calls0ln.A00, with optional instrumentation/exception paths retained as excerpts.
All inherited/default/external/runtime implementation resolution remains open.

With0ln.A0Q=true, A0G admits227 at02e4 ->0300. A00's227 branch at04b6 selects
SQ0.A0d and joins the common provider get/0Vw.BLn call. SQ0's constructor binds
A0d to00D.A00(0xbd9), i.e.3033.00D.A00 constructs05h; its constructor saves the
selector and get caches00D.A02. A02/A03's normal factory fallback reaches1Uw.A00.
Fresh1Uw.A00 exactly matches the
[retained router](../../rea/2.26.43.72-privacy-transport/di-router-dex.txt).
For3033 the high selector is0; low3033 satisfies0xaf7 <3033 <=0xc10 and routes
through A01 ->A0S, whose case3033 constructs15E.

A03 also contains DI cache/override handling; the selected factory is not proof
of every effective runtime object. A0Q=false has dynamic SparseIntArray/list
admission and handler lookup instead. Effective mode, legacy registrations,
overrides and account initialization are not inferred from compiled code. A
zero direct BLn caller census is a virtual-interface boundary, not inactivity;
the constructor census identifies1Uw.A0S as its direct producer.

## Event227 preserves a disabled local preference

The selected15E.BLn branch accepts only exact root readreceipts all/none.
Missing/other values log and return local handled=true without preference
mutation or corrective send. That Boolean return is not a wire ACK.
For admitted values it reads09S.A1H and compares the local Boolean:

| Existing local | Received | Selected effect |
| --- | --- | --- |
| true | all | unchanged |
| false | none | unchanged |
| true | none | putBoolean(false), apply, then0oe.A0Q(singleton readreceipts) |
| false | all | retain false and call0oe.A0P(readreceipts,none) |

Equal values jump00e0+0x153*2 to0386 returntrue. The unequal local-false path
jumps012a+0x2e*2 to0186 and queues the corrective setter. It does not enable
read receipts from all. The local-true branch can only adopt false because it
was reached after the inequality test. Preferences use apply, not a proven
synchronous disk commit or transaction with observer/network work.

The corrective setter is the separately retained
[pending-intent/classic-or-MEX producer](../2.26.43.72-privacy-setter-pending-lifecycle/README.md).
This is new setting work, not replaying the original receipt or ACKing an ACK.
Effective protocol flag, durable ownership, request ordering/recovery and
redelivery behavior must be reconstructed before porting that send.

This differs from0oe.A0R's validated category-map adoption, which can enable
read receipts when no contradictory pending intent owns that category. Do not
merge both into a generic server-setting-wins update. Conversely, do not copy
this event's local-off rule into every account_sync category notification.

## Rust boundary and reproduction

Rust captures incoming nodes and has durable receipt ACK correlation; it has
no corresponding event227 preference/corrective-setter handler. Ordinary ACK
settlement and receipt callback ownership do not prove that this global side
effect was applied. No new preference default, receipt policy, correction send,
automatic retry, processed-ID claim or ACK-of-ACK is introduced here. Full Rust
privacy setter/mode ownership remains unported. Historical source remains
attributed to37.71; this bundle establishes selected current-build parity only.

Build xtask, then run from repository root:

```sh
cargo build --locked -p xtask
python3 docs/protocol/apk/2.26.43.72-read-setting-sync/verify.py ../android-lab/apks/WhatsApp.1.apk
```

verify.py contains all exact export commands and excerpt recipes, checks the APK
digest before analysis, reproduces twenty retained exports/excerpts, compares
two reused methods byte-for-byte, verifies selected branch/provider arithmetic
and every bundle checksum. It invokes only offline xtask APK readers. Factory,
provider and synthetic dispatcher excerpts are explicitly partial; unrelated
branches stay local. checksums.json binds every other bundle file by bytes and
SHA256, including this README and the authored verifier. No REA Evidence ID is
invented for DEX exports and no duplicate REA record is added.
