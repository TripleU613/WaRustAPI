# Async status ACK edit admission — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX methods/classes via xtask fallback, not REA Evidence or
execution. No account operations. The method exports are complete; downstream
storage/callback authority is only partially traced as explicitly bounded below.

The [previous diversion evidence](../2.26.43.72-message-ack-diversions/README.md)
binds1rH.A03's scheduledFcQ(...,0) to selected status/type5 or exactgroup-status
flag with group/type1 or newsletter/type21. FcQ.run packed-switch case0 begins
03ec. It requiresEoh id, thenF2a.A01 extracts a base id andA00 extracts a part:
first '__part__' at index>0 splits base/suffix, otherwise base unchanged/partnull.
An empty suffix is still nonnull. F2a's random-ID generation is retained in the
complete class but not invoked or ported by this change.

FcQ044c..045c takesEoh.A07 edit; absent sets0 at057e, otherwise directly invokes
Integer.parseInt. This is signed Javai32 grammar, NOT ASCIIu32, trim, or lenient
0HF parsing. Malformed/out-of-range text throws BEFORE success/refusal dispatch.
The runnable has no catch protecting this selected path (complete catch table
retained). It is asynchronous after reader scheduling: no fatal reader/reconnect
behavior follows from this parse failure. Global error459 diverted even earlier.

Only after edit parsing does it readt, server_id, addressing_mode, rcat,count
and branch on raw error presence. No-error0500..0512 converts participant and
calls1r6.A03, thenF1x.A04/A05 with its boolean result. Refusal0586 onward parses
error leniently with0HF.A00. For newsletter/error404 and1dX.A0A(edit),05ae..05e6
explicitly logs treating404 as success and invokesA03 instead. The retained
1dX.A0A permits exactly7/8. This is NOT a general404-success rule and does not
authorize a revoke retry or Rust outcome conversion without that operation/key.
Other refusal paths convert participant/from and invoke1r6.A00 or part-specific
handlers. Complete run preserves further branches; no blanket ordinary-ACK rule
is inferred from their existence.

## State authority still open

1r6.A03 is not a trivial success flag: with null part it invokes privateA01 and
returnsfalse (0000..004a); nonnull parts inspect1ti/DyI andF1x part rows, routing
through additional update helpers and operation-specific paths.1r6.A00 inspects
existingDyI state and can skip a failure state update ifACg.A00 says already
terminal; it also maps group refusals and has additional side effects. Complete
methods are retained, but privateA01, part-store transactions, state-enum meanings,
callbacks and full terminal authority are not established here. Scheduled
newsletter's separate17f/0TX branch remains open. No model-wide status success/
refusal/404 reinterpretation or part-ID settlement is added from partial evidence.

Rust now ports ONLY the direct parse-failure boundary for selected status routes,
shared across outbox and expected-chat classification. Invalid edit retains
uncertainty on both success/refusal; receive epoch stays usable. Valid edit
controls retain existing behavior without claiming complete native status parity.
Binary attributes, all async lifecycle semantics and server policy remain open.

## Reproduce

From checkout root, pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/FcQ;' --method run --signatures --exceptions > runnable.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r6;' --method A03 --signatures --exceptions > success-handler.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r6;' --method A00 --signatures --exceptions > refusal-handler.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/F2a;' --signatures --exceptions > wire-key.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1dX;' --method A0A --signatures --exceptions > revoke-edit.txt
```

Displayed PCs are bytes, branch deltas code units. No live or zero-ban claim.
