# Notification ACK audit evidence

**Confirmed static:** APK **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
REA 6.5.0 with headless JADX 0.7.1. No APK execution or live operation.
[Audit, correction and limits](../../notification-ack-audit.md).

- [type-map.json](type-map.json): complete current `X.EL2.<clinit>`, overload 0,
  Evidence `ev_fe5ee3bc40e5d8c8af478aaac7fc1712498f17fad749af62abfa63c757bf772c`.
  The native map binds `privacy_token` to integer 234 (`0xea`). Names in this
  map alone do not establish generic ACK authority for all listed types.
- [privacy-token-handler.txt](privacy-token-handler.txt): complete checked DEX
  instructions for `LX/10G;`, constructor and `A09(LX/0Tp;I)V`. It extends
  `LX/0v6;`, registers 234, skips unknown token kinds, and never calls the base
  defer helper. The SHA-256 of the text is
  `dd4801ed3f9eea8b2c811c61aa8321f9fb3cf5f4fe519210965696c5de89e040`.
- [base-dispatch.json](base-dispatch.json): complete current `X.C0v6.A01`,
  overload 0, Evidence `ev_45fabd4024bc78f07158d51230407c7081599240c6f057d86e02f4a280457d7b`.
  Dispatch precedes ACK; a deferred handler is excluded until completion.
- [base-ack.json](base-ack.json): complete current `X.C0v6.A00`, overload 0,
  Evidence `ev_a3cc8831b099eadd66044acaf5e2823a987754d2739368c003107bdbc11f805b`.
  It dispatches through queued/local or socket ACK delivery, not token-kind tests.

Current handler discovery intersected exact `trusted_contact` and `sender_lid`
string-reference results. REA's class-name search for `10G` found no named row,
and lookup of raw `X.10G` was rejected, so source was read with the repository's
checked DEX extractor rather than guessing a JADX alias. REA resolves the base
as `X.C0v6`; all quoted artifacts carry the current APK identity.

```sh
target/debug/xtask apk-xref /path/to/WhatsApp.1.apk \
  --anchor trusted_contact --anchor sender_lid
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/10G;' --signatures --exceptions
```

The ownership sidecar marks supporting source for the existing Rust notification
handler. It does not establish complete native handler, router or protocol parity.
