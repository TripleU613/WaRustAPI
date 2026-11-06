# MEX error dispatch versus telemetry, APK43.72

**Confirmed static:** WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues the diagnosed unavailable Android provider. Complete selected exports and physical field-reference inventories retained. No runtime observation or REA Evidence envelope claim. checksums.json binds every other file.

## Actual failure delivery

[Previous error-model trace](../2.26.43.72-mex-iq-error-models/README.md) left0mT.A02 virtual dispatch unresolved. The complete0mU method-body export has noA02 override; retained0mT.A02 is a concrete telemetry annotator. It records has_data, errors, error_codes, has_multiple_errors and first_critical_error/code through0BX. It does not call a product callback or schedule a retry.0mU's ownA04/A05/A06 methods add decoding/format/delivery-failure telemetry. Its full method-body export is bounded evidence, not a general proof of all possible subclasses.

The product error delivery is the separate private0mZ.A00(LV5), already retained in [response-adapter.txt](../2.26.43.72-prekey-lid-mex-iq-expiry/response-adapter.txt), not0mT.A02. The newly retained constructor stores its0ia argument in0mZ.A02. Existing [0j3 non-Pando send](../2.26.43.72-prekey-lid-mex-operation/iq-handle.txt) passes0j2.A03 as that argument. AtA00+0000..0016, LIM exceptions receive requestInfo=0mF. The selected ordinary path at0096..00a0 calls that callback's0iZ.Bx5(Throwable). The method also contains7MK/corrupt-stream andLIC paths plus exception classification/reporting through0ma; do not claim all callbacks always return normally or that those paths implement a send retry.

[BaseMexCallback complete export](../2.26.43.72-prekey-lid-mex-callback/base-callback.txt) resolvesBx5 ->A06(Throwable), which constructsHbw(Throwable,null), invokes its own callback telemetryA03, thenA08(Hbw). The selected [0id binding](../2.26.43.72-prekey-lid-mex-callback/binding.txt) forwardsA08 to its captured error function. Thus theHbw made in0mZ.BKo/BKl for0mT telemetry is not itself the object passed into the product callback: the callback gets the exception and wraps it afresh. Both wrappers preserve the same LIM category. This distinction prevents mistaking telemetry completion for product completion or response adoption.

The earlier missing-LID callback trace establishes Hbw.A06's LIM check and its generic exceptional future path; this chunk closes the dispatch gap into that callback, rather than adding a new native retry claim. Full0ma failure reporting, corrupt-stream policy, all callbacks, timer/backoff persistence and Pando remain separate work.

## Bounded negative field inventories

apk-field-refs found exactly one physical reference to7Ol.A01:0Tp: the constructor write. It likewise found exactly one reference to7MD.error:7Ol: the constructor write. There are no direct DEX reads of those exact fields in this inventory. This makes a direct getter consuming the retained7Ol.A01 backoff unlikely; it does not prove the raw IQ is inaccessible. In particular7Ol also stores the IQ in inheritedEEy.A00; superclass access, generic original-node consumers, reflection/native code and broader request handling are outside these exact-field searches. No universal statement that the server backoff is ignored, absent or safe to bypass follows.

No Rust runtime code changed. Rust's MEX JSON payload parser remains separate from outer admission, product completion and retry authority. No live account operation.

Reproduce from repository root:
```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0mU;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mT;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mZ;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mZ;' --method A00 --signatures --exceptions
target/debug/xtask apk-field-refs "$APK" --class 'LX/7Ol;' --field A01 --type 'LX/0Tp;'
target/debug/xtask apk-field-refs "$APK" --class 'LX/7MD;' --field error --type 'LX/7Ol;'
```
TheA00 export was rechecked against the existing retained complete response-adapter export; reuse that source instead of retaining another copy. Field inventories are static instruction references, not runtime reachability or account observations.
