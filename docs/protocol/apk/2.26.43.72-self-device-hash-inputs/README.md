# Self PN/LID device-hash input normalization, APK43.72

Confirmed static WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues diagnosed unavailable Android provider. Complete selected methods retained; no execution, live traffic or REA Evidence envelope claim. checksums.json binds every other file.

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0jD;' --method 'A0A' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0jD;' --method 'A0B' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0jD;' --method 'A02' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method 'A0d' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method 'A0Y' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method 'A04' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0EB;' --method 'A02' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1nG;' --method 'A08' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/PhoneUserJid;' --method 'getType' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/F0I;' --method 'A04' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/2L9;' --method 'A1N' --signatures --exceptions
```

From repo root. Reuse [0jD.A0C selected own/non-self dispatch](../2.26.43.72-device-hash-formats/input.txt) and the preceding hash-format bodies. These methods interpret only account-scoped input normalization, not proof of authenticated membership acceptance.

## Source set and record filter

0jD.A02 returns empty ImmutableSet when09F.BUx is false; otherwise it takes1nG.A08 keys. The effective meaning/value ofBUx is untraced here.1nG.A08 rebuilds1nH.A06 entries while excluding values whoseF0I.A04 is true. F0I.A04 compares itsA02 long with zero and2L9.A1N returns comparison>0, so positiveA02 records are excluded. The field's units/authority and source-store authentication remain unknown: it is not labelled an expiry or tombstone without further evidence.

## Self PN output set

0jD.A0B reads own PhoneUserJid via09F.Aum and iteratesA02.0Dn.A0d testsgetType()==0 (PhoneUserJid.getType confirms0); such entries are retained directly. Other entries must satisfyA0Y (types18/19/25), own PN must exist and09F.BVC(userJid) must be true. Then0EB.A02 builds an own-PN DeviceJid with the original device number; null conversions are skipped. Its wrapper catches failures fromA01 and returns null. Full subclass labels for every acceptedA0Y type and BVC implementation are untraced.

## Self LID output set

0jD.A0A requires own LID09F.Auj. It maps type0 entries to that own LID while preservinggetDevice using0Dn.A04. Device99 has a distinctDv6 constructor; other values use0EM. Mapping09P is logged and that entry skipped, rather than failure of the entire collection. Existing entries satisfyingA0Y are retained directly; other categories are skipped. This mapping is safe to interpret only inside the selected own-account source path, not as a generic incoming PN/LID alias relation.

The enclosingA0C chooses these paths only when the requested user equals ownPN or ownLID, then adds09F.Aul/Aui respectively (asserted non-null) to the returned set. It does not substitute non-selfgetPrimaryDevice behavior for own current-device identity. The ordinary other-user path remains separate and explicitly adds its primary.

## Port boundary

This closes the two own-input helper bodies left unknown in the prior formatter comparison. Complete source-store verification, BUx/BVC semantics, subtype/device99 normalization, own-device identity, snapshots and account lifecycle remain incomplete. Existing Rust ReconciledDevices::device_hash is a validated non-self candidate/hash utility, not a new native own-account normalizer. No generic alias rewrite, identity migration, hash-default change, query or live account operation was added. A matching membership hash remains separate from identity authentication.
