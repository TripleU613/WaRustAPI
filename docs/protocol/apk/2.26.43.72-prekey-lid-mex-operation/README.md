# MEX lookup operation ownership and transport timeout, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238, SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected classes/methods from supported xtask DEX fallback, continuing the diagnosed unavailable Android provider. No APK execution/live traffic or REA Evidence envelope claim.

## Reproduce

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0iq;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0j2;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0j3;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/RsH;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/2LA;' --method 'A0Y' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ig;' --method 'A01' --signatures --exceptions
target/debug/xtask apk-field-refs "$APK" --class 'LX/0iq;' --field A00 --type J
```

From repository root. checksums.json binds every other file. Reuse [HSo lookup builder](../2.26.43.72-prekey-lid-mex-selection/profile-dispatch.txt), [A0W future wait](../2.26.43.72-prekey-lid-query-route/await.txt) and [FfK implementation](../2.26.43.72-bulk-key-job-completion/future.txt). The complete alternateRsH class is retained for bounded comparison, not a full interpretation of its transport.

## Handle returned, then discarded

HSo uses2LA.A0Y, which casts its factory to0ig and callsA01.0ig.A01 constructs a fresh0iq and returns it without a further mutation.0iq constructor assigns A00=32000ms at0080..0088. It does not explicitly assign A03; the fresh Java object's boolean default is false. The HSo path invokesAQY directly without BZr or a timeout-field assignment. Ten physicalA00 references include other producers' overrides; those are not evidence that this newly constructed lookup operation was overridden.

AQY constructs0id, applies the supplied callback initializer, callsAQV, and returns void. AQV creates0j2 through0iq.A00, starts it viaA04 and returns the handle; AQY discards that result. Thus this selected producer does not retain the returned handle for cancellation in its outer future wait. A0W's timeout/interrupt branches return false and do not invoke cancellation; retainedFfK.cancel returns false. These are bounded selected-path facts, not proof that all underlying transports lack cancellation or that a late callback actually occurs.

## Selected operation class and two transport branches

0iq.A00 checksA03: false constructs0j3 at00dc..017a; true constructsRsH. The fresh HSo lookup selects0j3 from the verified factory/default/call chain. BZr merely setsA03=true for callers that invoke it; its name must not be mistaken for a cancel method. RsH has no ownA04 body in its full export (a focusedA04 search returned no match); start behavior is inherited from0j2. This negative result is inheritance, not provider failure.

0j2.A04 sets up callback telemetry when applicable, checks its request identity and invokes virtualA06. Missing request identity producesLI6 throughBx5 instead of dispatch.0j3 constructor copies the supplied timeout into its A00, obtains an IQ ID from0Sf.A0G and creates0mF. That construction does not prove an IQ was sent, sinceA06 still selects transport.

0j3.A06 asks0io for0mO and a potential Pando request. Its isPando branch dispatchesMja.ARN at00ce..00d4 and discards the returnedNXp. Its non-Pando branch creates0mZ/0mb callbacks, reads0j3.A00 and calls0Sf.A0W at013a..0148 with selector0x155 and that timeout. Therefore the selected non-Pando MEX send inherits32000ms from this lookup's fresh0iq default, while the lookup's enclosingA0W waits128000ms. Neither value is a universal effective Pando timeout, and their start instants/expiry scheduling are separate. The previous classic usync branch supplies its own128000ms to0pD; it is not this nested MEX builder default.

## Boundaries

Effective isPando selection, Pando cancellation and deadlines,0mb reply/timeout cleanup, operation aliasing, request identity serialization/query hash, late callback suppression and mapping adoption remain untraced. The absence of a retained cancellation handle in this path does not establish a native race or ban cause. No Rust timeout/request/retry behavior was changed; request-owned callbacks and uncertain-state/no-replay rules remain. No live account operation.
