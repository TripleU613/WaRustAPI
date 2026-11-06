# MEX result cardinality, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected constructor overloads via supported xtask DEX fallback; Android provider remains unavailable as previously diagnosed. No APK execution/live traffic or REA Evidence envelope claim.

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/7Ou;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/7Ow;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6oe;' --method A04 --signatures --exceptions
```

From repository root. checksums.json binds every other retained file. Only the overloads with (0Tp,0mK) descriptors are interpreted; other overloads are retained because the method-name filter exports them too. Reuse [typed delegate parsing](../2.26.43.72-prekey-lid-mex-iq-expiry/iq-delegate.txt), which attempts these success/request-error constructors before server-error fallback.

7Ou(0Tp,0mK) asserts the IQ root, validates request-correlated metadata through8B6/88J and finally collects named result children at01cc..0208. At0208..022a it requires size>=1 and size<=1, storing the sole child at022a..0236. Zero and multiple results throwLIL through0238..025a. The retained6oe.A04 helper returns AbstractCollection.size widened to long, closing the lower-bound count mapping. That exact-one check is explicit; no assumptions about8B6 duplicate semantics are needed for this conclusion.

7Ow(0Tp,0mK) calls6oe.A1M, uses88J.A00 and requests IQErrorBadRequest|IQErrorFeatureNotImplemented through8B6.A0K. Full helper/error-model constraints are not reconstructed and no generic HTTP/IQ status interpretation follows. This constructor was explored to locate the cardinality boundary; its complete export preserves that search scope without claiming every overload was interpreted.

Rust parse_json_response previously selected the first result and ignored duplicate siblings. It now rejects multiple result children with a fixed local diagnostic before JSON parsing or update fallback. Result-over-update priority and update-only parsing remain; update cardinality/native streaming behavior is not changed or newly claimed. Root/request admission remains the caller's separate responsibility. This corrects selected non-Pando result cardinality and supplies a local ambiguity guard to this shared JSON parser, not complete native MEX parity. Final query serialization, Pando behavior and mapping adoption remain incomplete.
