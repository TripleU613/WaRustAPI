# Identity-only reply context, APK 2.26.43.72

Static xtask DEX fallback, not runtime observation or REA Evidence records. REA Android provider remained unavailable from the previously diagnosed Java configuration; no repeated setup or target execution. Pinned local APK `../android-lab/apks/WhatsApp.1.apk`, SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e` (verified this pass), code264307238. No live account traffic.

## Reproduce

Run from repo root with `APK=../android-lab/apks/WhatsApp.1.apk`:

```sh
target/debug/xtask apk-xref "$APK" --anchor 'type node should contain exactly 1 byte'
target/debug/xtask apk-callers "$APK" --class 'LX/FUP;' --method '<init>'
target/debug/xtask apk-callers "$APK" --class 'LX/0lG;' --method CK4
target/debug/xtask apk-smali "$APK" --class 'LX/FUP;' --method CK4 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/FUX;' --method CK4 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/FUP;' --method B --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/FUP;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ln;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/FeM;' --method run --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/137;' --method A03 --signatures --exceptions
```

All callback/dispatch exports are complete selected methods, including other discriminator cases. `producer-excerpt.txt` alone is bounded: its sidecar gives inclusive source line ranges, full source bytes/SHA-256 and extraction scope. Re-export the full source and select those ranges after checking its digest. Offsets are method-relative bytes; branch comments use DEX code units. `checksums.json` binds every other retained file.

## Confirmed static path

- `FeM.run` switch case16 enters1740; captured owner10L and requested list are retained. Register v3 is zero at1760; after user rows and encrypt/get IQ construction, identity node1854..1860, FUP constructor1870..187a receives requested list, owner, discriminator0. Timeout0x7d00 and selector0x99 passed to0Sf.A0W1882..1888. Constructor stores discriminator and captures without reinterpretation. This does not establish Rust deadline parity.
- `0ln.A01`0014..0036 dispatches result to0lG.CK4 and skips error; error0036..004e calls Bvj. No root error child veto on the result branch.
- `FUP.CK4`case0 selects list0076..0082 then user children0082..008e, DeviceJid00be..00d2, and that user's error00d2..00fc. Code is read through0Tp.A04 returning signed int. Otherwise identity/type/optional device-identity00fe..0124; type body must contain one byte0124..0134. It schedules FdC case1 at0168..017c. This successor differs from the older37.71 DNk worker; its downstream adoption is not claimed here.
- `FUP.Bvj`case0 reads outer error via1ud.A00 returning int004e..0056, then applies it to captured requested devices0056..008a. `BsS`case0 returns at0038 without a retry in that callback.

## Negative scope and limits

Exact anchor census has four physical references: Dxh.A02 (full directory), F10.A04, FUP.CK4, FUX.CK4. Retained FUX callback is a missing-device prekey helper with user/device nesting and registration/key parsing, not this identity-only producer. Constructor census has137.A03,1HS.A00,DFS.handleMessage,FeM.run; the traced FeM discriminator binding is what selects identity-only case0. No claim of exhaustive runtime callers or reflective references. Retained `other-producer.txt` (137.A03) binds discriminator1, not0; no identity-only claim is made from that caller.

Rust fixes follow outer-type and per-user context, and expose recognized signed i32 codes instead of u16. A malformed/overflowing code remains null without inventing a code or wait. Existing local unique-list/user/key guards remain stricter than APK iteration, and no identity adoption, companion verification, request replay or account operation is added. Generic action Ok is not proof of usable identity. Native callback error throwing/default semantics are not reproduced as Rust account policy.
