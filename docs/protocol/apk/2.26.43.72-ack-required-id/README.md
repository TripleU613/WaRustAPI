# Required ACK ID — APK 2.26.43.72

**Confirmed static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. xtask DEX fallback, not REA Evidence envelopes or runtime
observations. No live account operation.

The retained [complete 1rH.A03](../2.26.43.72-distribution-ack-admission/native.txt)
calls 0Tp.A0L("id") at0054..0064, before numeric error at00a2 and class
routing at00c6. A0L calls A0M(name,null), returns any nonnull string (including
empty), and throws LIL when null. A0M returns 0Tl.A03 for a found attribute,
or its supplied default for absence. Thus an absent ACK id cannot reach
ordinary callbacks or auth-challenge459 diversion, irrespective of class.

[Selected LIL propagation and reader exit](../2.26.43.72-stanza-corruption-routing/README.md)
apply to this same mapped handler. Rust stateful paths now capture then stop
on absent id using the shared received-ACK validator. No new generic NACK,
reconnect or replay follows. Empty textual id is not rejected by this check.

**Limits:** the id attribute's native A03 population/coercion for nontext forms,
full JID typed accessors, duplicate attributes, Efa/Eoh construction and concrete
listener cleanup remain open. This finding proves missing-id handling, not
complete envelope equivalence. Local pure outcome/correlation helpers are not
transport admission validators; raw borrowed-channel callers own discard.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0L --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0M --signatures --exceptions
```

checksums.json covers every other file here. Existing complete handler and
reader exports are linked instead of duplicated. No restriction guarantee.
