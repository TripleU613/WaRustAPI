# MEX server JID equality, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues the diagnosed unavailable Android provider. Complete selected methods/classes retained; no APK execution/live traffic or REA Evidence envelope claim.

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method 'A0A' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/Jid;' --method 'equals' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dd;' --method 'A02' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Tl;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dd;' --method 'A00' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mF;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mK;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0mL;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0De;' --method 'A00' --signatures --exceptions
```

From repository root. checksums.json binds every other file. Only the server/equality/selected reader paths below are interpreted, not every user/group/device branch or attribute serializer. One byte-field store in0Tl is rendered as an unknown opcode; no semantics are inferred from that instruction.

## Original MEX destination is a concrete server JID

0mF constructor derives queryId/variables JSON and creates0mK, whose node becomes0mF.A01.0mK constructor001a..0034 adds to using0mL.A00 as a typed Jid, alongside w:mex/get and its independent request ID/query content. This is the original request-to used by the [prior response correlation trace](../2.26.43.72-mex-envelope-correlation/README.md).

The complete0mL class extendsJid, constructs user="", returns server="s.whatsapp.net" and type13. It has no equals override; inheritedJid.equals compares user, getServer() and getType(), with reference identity as an early success. This closes equality for this server singleton; it does not equate arbitrary users or device/alias encodings.

## Typed attribute versus exact server text

0Tp.A0A first reads an existing0Tl.A01 typed Jid; otherwise it reads text and calls0Dd.A02. That nullable parser returns null for empty/null or caught parse failure and delegates other strings toA00. For strings without @,A00's0036+0x144*2 reaches02be and calls0De.A00.0De maps the exact case-sensitive s.whatsapp.net string to0mL.A00. It also has separate g.us/call singleton cases, which are not the same server or type. No lowercasing, prefix match or arbitrary empty-user alias is shown on this exact server branch. Full user/device/hosted/legacy aliases remain outside the interpreted scope.

Therefore the selected request's typed server and exact text from=s.whatsapp.net can resolve to the same inherited equality identity. This is not proof that every wire encoding produces that singleton: the binary decoder and every subclass override are not reconstructed here. In the prior8B6 helper, a missing/mismatching parsed sender still fails the selected caller checks rather than receiving a destination default.

## Compliance diagnostic is not an admission gate

0Tp.A0A calls isProtocolCompliant at002e..0034. True jumps from0036 to008a; false logs and also reaches008a. It does not return null solely because of that diagnostic. The method then has a DeviceJid-specific conversion only when requestedClass==DeviceJid, followed by Class.cast; ClassCastException is caught and returns null. The selected MEX validator requests baseJid, so that special DeviceJid conversion is not its path. Null/error parsing and downstream expected-value equality are separate guards. Do not copy a guessed global compliance-rejection rule into Rust.

## Boundaries

This closes the original server destination and exact server-text equality route for the selected non-Pando MEX envelope. Binary JID decoding, arbitrary subtype equality overrides, parser caches/aliases, sender provenance, MEX response models and authenticated mapping adoption remain incomplete. The request builder export also preserves current queryId/variables/w:mex construction without claiming the runtime query identifier for UsyncQuery is resolved. Rust payload inspection remains separate from request-owned live admission. No Rust sender policy, runtime request, retry or account operation added.
