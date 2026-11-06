# Identity notification ACK dispatch, APK 2.26.43.72

Confirmed static DEX path, not observed execution or REA Evidence records. Local APK `../android-lab/apks/WhatsApp.1.apk`, SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`, code264307238; APK digest verified this pass. REA Android provider remains unavailable from the already diagnosed Java configuration; focused xtask static fallback used without repeating setup. No account traffic.

## Reproduce

From repo root, set `APK=../android-lab/apks/WhatsApp.1.apk`, then:

```sh
target/debug/xtask apk-smali "$APK" --class 'LX/15v;' --method A09 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Fc1;' --method run --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Fc1;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/17o;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0YM;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0v6;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0WJ;' --method A0W --signatures --exceptions
```

Each retained export is a complete selected method with signature and exception table. The synthetic Fc1.run contains other discriminator branches; conclusions below concern only case15. Byte PCs are method-relative, branch annotations use code units. checksums.json binds every other file. No media, APK, bulk tree or private account data retained.

## Dispatch versus ACK

15v.A09 selects first child, tests identity00d4..00e4. It optionally reads metadata UserJid from/lid/type/display_name, emits843/191 metadata work, then rejoins DeviceJid handling at018c. The null metadata-peer branch00f8 adds0x4a code units and also reaches018c; it does not prove all identity work is skipped. DeviceJid conversion from base0v6.A01 field occurs018c..019c and is asserted nonnull. At01bc..01d0 it constructs Fc1 with DeviceJid, handler, discriminator15 and calls17o.A00. It returns01d0..01d2 without invoking deferred-ACK method0v6.A07 in this branch. Exact export matches the previously retained prekey-ack-count-dispatch file byte-for-byte, now given focused identity analysis and checksums here.

Fc1 constructor stores discriminator and captures; case15 enters0db6. It opens a per-address context, calls0WJ.A0W(getIdentityKey)0df0..0df6, and if nonnull calls14t.A0A with one requested DeviceJid, reason8,true at0dfc..0e26. It then checks0Fy.A0W andEx8.A06 and may call synchronized14t.A07 with additional flags0e26..0e82. It closes context and returns0e8e..0e96. No direct base-handler A07/descriptor-ACK completion call occurs in this selected worker branch. The meaning/effects of those secondary gates and14t scheduling are not established here.

17o.A00 submits under decrypt_order to0YM.A02. That wraps through0YO.A01 and invokes ThreadPoolExecutor.execute. Executor construction/rejection policy, actual concurrency and wrapper instrumentation remain untraced; this is not proof that execution cannot fall back to the caller thread. More importantly, neither selected method adds the identity notification to base deferred-ACK map.

0v6.A01 invokes virtual A09 at0204..020a. Normal return and caught LIL join0298; it takes the current2DT descriptor, checks deferred mapA0A, and invokes baseA00 at02c2..02c8 only when not deferred. Thus identity key-fetch completion or candidate adoption is not the normal ACK prerequisite. This does **not** guarantee that ACK reaches the socket before worker execution/query dispatch. Rust's ACK-before-draining-read ordering remains an implementation ordering, not full native scheduling parity.

## Failure and lookup boundaries

The base method rethrows SQLiteFull/Corrupt at020c; LIL is logged then rejoins ACK handling. Other Exception enters02d8, and when incoming BX2 is true builds completion2Dx with code500 then calls1ic.A0G; otherwise it returns. It does not follow normal ACK continuation. This is not a license to catch every Rust/local-state error and send a success ACK, or to claim an end-to-end500 wire encoding from this method alone.

0WJ.A0W validates the scoped address throughA04, locks, reads bytes via0YT.A0K, decodes viaF3k.A02, wraps Ee1 and closes. Absent/unresolved returns null. Caught ECt logs invalid saved identity, callsA0G then returns null; A0G cleanup effects remain untraced and are not copied to Rust. Held identity lookup is separate from replacement-key trust.

## Result and limits

Current build supports the earlier37.71 conclusion that identity ACK handling does not wait for fetched-key adoption. No Rust mismatch requiring an ACK widening was established in this selected path. Existing supported PN/LID/participant/id validation and local-state-fault behavior remain. The producer of Eoh.A02 (base A01) and current address routing/ACK renderer are not re-anchored here. Unsupported/malformed routes, alternate transport rendering, downstream metadata/alias authority, full14t job/backoff policy, executor failure recovery and live ACK acceptance remain outside this audit. No code or network behavior changed; no new test-only assertion is treated as APK evidence.
