# LID-only multi-query transport selection, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238, SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Full selected methods from supported xtask DEX fallback. Android provider remains unavailable as previously diagnosed; no APK execution, live traffic or REA Evidence envelope claim.

## Reproduce

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0p7;' --method A0F --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/HSo;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/3q3;' --method A1V --signatures --exceptions
```

From repository root. checksums.json binds every other file. Reuse complete preceding [query methods](../2.26.43.72-prekey-lid-query-route/README.md) and [record/writer constructors](../2.26.43.72-contact-factory-selection/README.md). Only the named blocks below are interpreted.

## LID flag propagation

For the prior initial LID-only preset, executor constructs3GQ.A04=true. A0O001e..0042 branches from0026 to003e (0xc DEX code units*2), setting v26=true. In the **Gut.A0A==0** route,00ec+0x3a3*2 reaches0832, avoiding the earlier alternate masked-query path. If the later Gut.A04 gate is false,0858+0x14e*2 reaches0af4. This is a conditional static route, not an effective-account configuration claim.

That branch calls A0H with v26 as final Boolean at0b1e..0b24. A0H's admitted users call A0B with that Boolean at00aa..00b4. A0B003c..0040 stores it as Hck.A0I; H6w constructor00be..00c6 copies it toH6w.A0M. Separately A0O01e4..01f0 encodes the Boolean as mask0x100; A03 reads that bit00cc..00d8 and passes its derived Boolean to the same A0B at0180..019a. This closes that one mask bit, not all masks/filter/privacy paths.

The classic0pD.A06 writer readsH6w.A0M at0602 and, when HcZ.A03 is false and H6w.A0L is false, sets aggregate v41. It emits query/lid at0dfa..0e10. These are explicit writer conditions, not unconditional wire parity. HcZ.A03 must not be called a contact flag merely because another contact producer uses it. Other per-user address and protocol branches remain separate.

## The final dispatcher can choose MEX

The same A0O0af4 branch eventually calls A0F at0ba6. For an unmerged initial LID-only3GQ withA01=false andA0B=false,0b72..0b8e sets the final transport-selector input v38=true. A0F first constructs HcZ throughA0Q, then its final Boolean selects HSo.A01 at003a..0058. False instead calls0pD.A07 with128000ms at006c..007c. Therefore this LID-only combination does **not** establish that the classic stanza builder is the final transport. The preceding separate A0O09aa..09d2 path still supplies128000ms directly to0pD.A07; its timing fact was branch-specific, not universal.

HSo.A01 allocates FfK and builds per-user GraphQlCallInput values into query_input. It ORsH6w.A0M into v20 at024a..0252, then includes include_lid=true at0316..032c when any admitted user requests it.3q3.A1V verifies its object and returns1, establishing Boolean true, not a guessed helper meaning. Devices is separate: H6w.A0I ORs into v7 at0272..0278 and controls include_devices at040c..0422. Thus LID alone does not prove the devices protocol was requested.

At045a..047a the typed request is named UsyncQuery with namespace whatsapp-android-mex. It attaches IDw to0iq.AQY and returns its FfK at0480..04ae. This proves builder/dispatch selection, not its eventual serialized GraphQL envelope, HTTP/IQ transport, query ID/hash, native timer or callback acceptance. A0F uses A0W for either returned future; the prior128000ms Future.get remains a wait bound, not proof of this MEX operation's send timeout.

## Still unknown

Effective configuration, merged flags, target admission, per-user address encoding, MEX serializer/query identifier, callback/session lifetime, refusal/backoff/mapping adoption and persistence. HSo's callback is not reconstructed here. No MEX capability, automatic lookup/retry, deadline or Rust contact profile was added. Existing Rust phone/contact-check factory parity remains separate and unproved. Do not force all lookup traffic into classic usync from the shared LID vocabulary.
