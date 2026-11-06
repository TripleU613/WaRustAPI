# Missing-LID multi-query route and timing, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238, SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected method exports, supported xtask DEX fallback; Android provider remains unavailable as previously diagnosed. No APK execution, live account operation or REA Evidence ID claim.

## Reproduce

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0p7;' --method 'A0Q' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/3GQ;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0vF;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1X5;' --method 'invoke' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0p7;' --method 'A0O' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0p7;' --method 'A0W' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0p7;' --method 'A03' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0p7;' --method 'A0H' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0p7;' --method 'A0B' --signatures --exceptions
```

From repository root. checksums.json binds every other file. Reuse the preceding [complete executor export](../2.26.43.72-prekey-lid-queue/executor.txt), rather than exporting it twice. Only the selected blocks below are interpreted.

## Route binding

0vF initializer binds A01 to lazy1X5 selector26. Its invoke switch selects02f0, constructs the preset-operation map, and includes0vC.A0M at033e..0354 (operation inputs8,9).0wB.run tests this map at226c..2288, enters MultiProtocolQueryHandler, gets targets from3I5.A00 and skips if empty. For the nonempty path22d0+0x394*2=29f8. The batch loop eventually calls0p7.A0O at2be6..2c0c. The target-filter/batch materialization and map helper's internals are not reconstructed here; this establishes the preset's selected handler, not that every candidate is emitted unchanged.

The executor passes its captured LID local v24 into the eighth boolean argument of3GQ constructor (v51 in range v43..v56). Constructor maps the eighth input toA04. This is separate from unrelated obfuscated A04 fields. A0O reads thatA04 at001e and uses it in admission. No account-dependent category/config value is inferred from static defaults.

## Send and wait are distinct clocks

A0O has configuration-dependent protocol splitting:083c..086c reads Gut.A04, A0X and3GQ.A02;08a8/08c0 read properties0x2604/0x20e5, then A00/A01 derive flag groups. Its A0H helper filters/builds per-user Hck entries through A0B and A0V; it does not itself establish a different transport. The separately retained A03 helper decodes protocol masks into user flags. Effective configuration, masks, privacy enrichment and the complete final user/wire body remain unresolved.

On the selected nonempty constructed-list dispatch at09aa..09d2, A0O calls A0Q to build HcZ, then0pD.A07 with literal0x1f400=128000ms, thenA0W. A0W independently calls Future.get(128000,MILLISECONDS) at0030..0040 and requires non-null0p7.A00 response state before returning true. A0O maps false to1xM.A04 at09d4..09e0 and only enters the selected downstream result path on true. The wait return alone does not prove valid LID mapping or final adoption. Timeout/interrupt/exception handling is retained in full; no retry/reset is inferred from it.

Thus the preflight's earlier untimed3KB/FfK wait does not mean every nested operation has no timeout. Equally,128seconds is not a universal usync, bulk-key or Rust contact deadline: the independent phone-query factories retain their own32-second arguments. Send-callback timing and synchronous future waiting must not be summed into a claimed256-second request lifetime or treated as identical start instants. Timer registration and scheduling semantics remain in the previously retained0pD/0Sf evidence.

A0Q was first reached while exploring the executor's sidelist/full branches. Those call sites do not prove this preflight's route. The map and actualA0O call now supply the route binding; final wire/profile parity is still unknown. The source exports are retained to avoid repeating that false shortcut. No Rust timeout/request/retry behavior changed and no live operations occurred.
