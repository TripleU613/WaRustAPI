# MEX IQ error models and transport classification, APK43.72

**Confirmed static:** WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues the previously diagnosed unavailable Android provider. Complete selected method exports retained; no runtime observation or REA Evidence envelope claim. checksums.json binds every other file. Only selectors32..36 of8TV.ABJ are interpreted here; its complete method is retained to preserve switch targets and shared blocks.

## Discriminator order and retained errors

Reuse [0md delegate](../2.26.43.72-prekey-lid-mex-iq-expiry/iq-delegate.txt), [success/request-error constructors](../2.26.43.72-mex-result-cardinality/README.md), and [shared original-request equality](../2.26.43.72-mex-envelope-correlation/README.md). The delegate attempts7Ou success, then7Ow request error, then7Ol server error, falling through only onLIL. Both ordinary success/error callback entry points use that same delegate. A callback's method name or a present error child alone is not the complete classifier. Mixed success/error sibling precedence is constrained by this constructor order; arbitrary extra-child rejection is not established.

7Ow(0Tp,0mK) uses8TV selectors32/33 for bad-request and feature-not-implemented.7Ol asserts iq via6oe.A1M, invokes88J.A00 correlation, then selects the error child using8B6.A0K with selectors34/35/36, respectively rate-overlimit, internal-server-error and service-unavailable.8B6.A0K tries each supplied mixin and returns the first non-null model;8B6.A0H descends the requested child path then callsABJ. This is not selection by an arbitrary substring of a diagnostic.

8TV switch maps32->0bc0,33->0b54,34->0ab6,35->0a4e,36->09bc. Each selected branch requires an error element via8B6.A0C, reads text via6of.A1X and code via6of.A1V, using expected text/code pairs: bad-request/400, feature-not-implemented/501, rate-overlimit/429, internal-server-error/500, service-unavailable/503.6oj.A0r andA0s confirm400 and500. These are expected equality values passed toA0J, not defaults that fabricate absent codes. See the earlier retained [A0J expected-value semantics](../2.26.43.72-mex-envelope-correlation/README.md). Helpers/string bounds and full alternate error schemas remain separate.

After its server model succeeds,7Ol at00ce..0138 enumerates error children and requires exactly one (size>=1 AND size<=1), retaining that raw child inA01. Its earlier0026..0056 reads error/backoff as primitive Long with a lower-bound argument from6oj.A0p (1); the returned value is discarded. No backoff field is assigned in7Ol. It would be incorrect to infer a duration, timer or mandatory successful backoff validation from this ignored call; its effect on accumulated helper diagnostics is not completely reconstructed. The raw error child remains available for consumers not traced here. Do not turn this read into permission to retry after an invented interval.

## Callback category differs from GraphQL extension backoff

0mZ.BKo constructs7MD and wraps it inHbw(Throwable,null), then passes false to0mT.A03 and calls its private reportingA00.7MD extendsLIM with TRANSPORT/mex-iq-server-failure and an int-narrowed native code, retaining the7Ol error.0mZ.BKl analogously uses7MC, TRANSPORT/mex-iq-request-failure, retaining7Ow.7NN stores its supplied long code and text.0mT.A03 delegates toA02 and closes its telemetry marker; it contains no delay or retry scheduling.

Reuse [Hbw predicate and selected missing-LID callback](../2.26.43.72-prekey-lid-mex-callback/README.md): LIM presence makesHbw.A06 true. Thus these wrappers belong to that handler's generic transport-failure category, distinct fromHxy path-scoped GraphQL errors whose extension backoff is parsed/scaled. A complete0mT.A02 virtual dispatch and every downstream consumer/timer are not reconstructed here. No universal all-client retry policy follows.

The exploratory full7ME export is retained as a negative distinction: it is client cancellation (TRANSPORT/mex-request-cancelled,475), not the request-error wrapper7MC. An initial8TV method query forA00 returned no bodies; the actual selector method isABJ and the full four-method export resolves it. No setup retry or runtime provider result is implied.

## Rust boundary and remaining obligations

The existing Rust parse_json_response is explicitly a result/update JSON payload decoder, not a correlated IQ error classifier, scheduler or live dispatcher. It has no error-child/backoff parsing here; no runtime call path was added. Rejecting any mixed result/error packet in that helper without the complete outer admission model would invent policy. This research supplies the concrete native error families and prevents conflating IQ transport failures with GraphQL per-user backoff; it does not justify a new automatic retry. Complete alternate error handling, callback dispatch, backoff consumers/units/persistence, outer admission and Pando remain unfinished. No Rust behavior or live account operation changed.

From repository root:
```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/6oe;' --method 'A1M' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/7Ol;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/8TV;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/8B6;' --method 'A0K' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/8B6;' --method 'A0H' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/8B6;' --method 'A0C' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6of;' --method 'A1V' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6of;' --method 'A1X' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6oj;' --method 'A0p' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6oj;' --method 'A0s' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/6oj;' --method 'A0r' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/7NN;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/7MD;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/7MC;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/7ME;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mZ;' --method 'BKo' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mZ;' --method 'BKl' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mT;' --method 'A03' --signatures --exceptions
```
