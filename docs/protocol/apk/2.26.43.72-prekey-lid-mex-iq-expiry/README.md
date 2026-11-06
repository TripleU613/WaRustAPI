# Non-Pando MEX IQ expiry callback, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238, SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues the diagnosed unavailable Android provider. Full selected methods/classes and direct caller inventory retained; no APK execution, live traffic or REA Evidence envelope claim.

## Reproduce

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0mb;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0md;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mZ;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mU;' --method 'A06' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mT;' --method 'A03' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/7MB;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/8bj;' --method 'run' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/8bj;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Hbw;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-callers "$APK" --class 'LX/0lG;' --method BsS
```

From repository root. checksums.json binds every other file. Reuse [non-Pando request builder](../2.26.43.72-prekey-lid-mex-operation/iq-handle.txt), [registry expiry](../2.26.43.72-abprops-callback-timer/expire.txt), [timer registration](../2.26.43.72-abprops-callback-timer/register.txt), [base MEX delivery](../2.26.43.72-prekey-lid-mex-callback/base-callback.txt) and [error handlers/predicates](../2.26.43.72-prekey-lid-mex-callback/README.md). Only selected blocks are interpreted; the rest of the large synthetic runnable is retained without feature claims.

## Pending callback removal precedes failure delivery

The earlier0j3.A06 non-Pando branch constructs0mb and submits it with the copied operation timeout through0Sf.A0W.0mb wraps0md. Its BsS forwards to0md.BsS, which calls0mY.BL6; the captured concrete adapter is0mZ from that same construction.

Existing0ln.A03 expiry removes the pending ID fromA0I, or fromA0H's wrapper if needed, while holdingA0F; it then releases that lock before constructing8bj selector21 and calling098.execute. The retained constructor preserves this selector and callback/ID captures.8bj.run case21 at045c..046e invokes0lG.BsS(id). Thus selected expiry claims/removes pending callback ownership before scheduled failure delivery. This is not proof that the failure callback runs at exactly the nominal deadline or that expiry always wins a concurrent reply.

## Typed delivery failure reaches exceptional future completion

0mZ.BL6 updates its0mT/0mU telemetry then constructs7MB(callName) and calls the captured callback's Bx5.7MB extendsLIM and its constructor labels the error TRANSPORT / mex-delivery-failure with code0x1d7=471. This is a local typed transport failure, not an observed server471 refusal or ordinary request-level GraphQL extension error.0mU.A06 annotates delivery_failure_iq_id and ends its telemetry marker; it does not itself remove the registry entry or cancel a socket.

RetainedBaseMexCallback.Bx5 ->A06 constructsHbw(throwable,null) and invokes virtualA08. The newly retainedHbw constructor inserts aLIM throwable directly into its error list. Therefore Hbw.A06 detects this7MB and selectedIE6 error selector2 takes its generic MEX failure branch01da..01f4, createsGiw and callsFfK.BsR. The retained future get wraps this inExecutionException; the outerA0W handles its failure path. It is distinct from a request extension error's BLg/BsT(null) path. No retry or durable Sent reset follows.

## Result/error callback parsing is not outer-type acceptance

Both0mb.CK4 andBvj forward to0md, whose two methods call the sameA00 parser. A00 attempts7Ou success, then7Ow request error, then7Ol server error, falling through only onLIL; if none parses it throwsLIL with accumulated diagnostics. This is a shared typed parser, not proof that every IQ result succeeds or that all error bodies are interchangeable. Complete response constructors/serialization and their admission remain untraced.0mZ.BKn/BKl/BKo and error adaptation are retained, but full body/model interpretation and side effects are not claimed here.

## Late reply boundary and remaining work

The selected expiry removes ordinary registry ownership before failure delivery. A subsequent ordinary registry lookup therefore cannot retrieve that removed entry unless another authority reintroduces it. The global IQ fallback previously traced for bulk jobs remains separate; no complete late-reply suppression or concurrent callback proof is asserted. Reply-wins ordering, Pando callbacks/cancellation, reconnect/re-registration, queued callback delivery, exact timer cadence, MEX response parsing and authenticated LID adoption remain incomplete. No native race, restriction cause, live acceptance or new Rust behavior is claimed.
