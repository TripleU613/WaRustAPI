# Missing-LID MEX callback outcomes, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238, SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected methods/classes via supported xtask DEX fallback. Android provider remains unavailable as previously diagnosed. No APK execution, live account traffic or REA Evidence envelope claim.

## Reproduce

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/IDw;' --method 'invoke' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/IDw;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/IE6;' --method 'invoke' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/IE6;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0id;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Fwv;' --method 'A05' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Hbw;' --method 'A06' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Hbw;' --method 'A04' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0p9;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/HSo;' --method 'A00' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/graphql/pando/BaseMexCallback;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0pA;' --method 'BLh' --signatures --exceptions
```

From repository root. checksums.json binds every other retained file. The large IE6 success handler and base callback class are retained completely but only the selected entry/completion blocks below are interpreted. Existing [MEX producer](../2.26.43.72-prekey-lid-mex-selection/README.md), [future wait](../2.26.43.72-prekey-lid-query-route/await.txt), [future implementation](../2.26.43.72-bulk-key-job-completion/future.txt) and [observer binding/backoff](../2.26.43.72-contact-protocol-backoff/README.md) remain part of this chain.

## Constructor-bound handlers

HSo.A01 creates IDw selector1 at0488..04a6. Its constructor stores the supplied selector and captured arguments; it does not preserve argument order as field order. IDw.invoke case1 at009a..00fc assigns0id.A00=IE6 selector1 and0id.A01=IE6 selector2. IE6 constructor likewise has nonsequential capture assignment. The selected handler casts confirm success selector1 holds HSo/H9Z/maps/observer/FfK and error selector2 holds FfK/observer/HcZ/maps.0id.A05 invokes A00;0id.A08 invokes A01 and returns its Boolean. These bindings prevent confusing another synthetic-lambda selector with this producer.

## Request, scoped and exceptional errors

IE6 selector2 uses its switch default0012. Hbw.A06/A04 are predicates for contained LIM/7ME instances; those hit the generic exception branch01da. Their exact semantic category and constructor provenance remain untraced, rather than naming all failures network errors.

Otherwise it iterates Hbw.A01. An Hxy whose path JSONArray has length1 is handled as a request error before checking the scoped codes. Hxy.A00 is parsed as optional Long; when parse succeeds Fwv.A05 performs Java long multiplication by1000, then signed values greater than3600000 are capped. Missing/unparseable uses-1. This is not a saturating multiply, lower clamp or proof of safe deadline authority for overflow/negative inputs. It calls observer.BLg(context,request-id,error-code,backoff-ms) at00f6..0108, then FfK.BsT(null) at0112..011a. It returns Boolean false; multiple errors also produce a diagnostic. Future completion here is normal despite request refusal.

Other Hxy errors with code0x23f (575) and path length>=3 call HSo.A00 into one captured map as protocol errors; code0x257 (599) with length>=3 populates the other as user errors. HSo.A00 reads path[2], optional backoff and protocol-info fields; full key normalization/collision and downstream metadata use are not reconstructed. Other entries log ignoring and iteration continues. When iteration completes, the callback returns true, allowing the data path to proceed. Do not collapse these scoped errors into request refusal or unconditional request success.

The generic MEX-error branch constructs Giw(Hbw) and invokes FfK.BsR at01da..01f4, returning false. Earlier retained FfK.get wraps this as ExecutionException; A0W has its own exception handling. This is separate from BsT(null), not permission to replay uncertain Rust requests.

## Data continuation and response-state completion

BaseMexCallback.A04 calls A08 when RsF.A03 indicates errors. The returned false reaches00aa (0054+0x2b*2) without calling A05. True may continue into A05 according to RsF terminal/error flags and A07; full Pando delivery/retry/concurrency is untraced. Base A06 also forwards throwable-derived Hbw toA08. The observed subclass0id does not override A07; the retained base default is false.

Selected success IE6 case1 begins0256, reads xwa2_fetch_wa_users and eventually constructs HMk/Gzg. At1024..102a it calls observer.BLh(Gzg,id) **before** FfK.BsT(null) at102e..1034. Concrete0pA.BLh first stores Gzg in0p7.A00 at0000..0012. The earlier0p7 constructor binding shows its A0b observer is0pA and is passed into HSo byA0F. A0W requires non-null0p7.A00 after its timed future wait. Thus releasing the future alone is not equivalent to installing response state or adopting authenticated mappings. If response state is absent A0W returns false; clearing/staleness/concurrent ownership of this shared response field remains a separate unresolved audit, so no unconditional outcome is claimed.

## Remaining work

Full success/user/protocol model parsing, collision/repeated-path policy, mapping authentication/adoption, durable backoff clocks/gates, shared response-state lifecycle, MEX envelope/query identifier, native cancellation/send timeout and server acceptance remain incomplete. Retaining the full success method does not mean it has been fully interpreted. No Rust request, retry, backoff reset or new callback was implemented; no account operations.
