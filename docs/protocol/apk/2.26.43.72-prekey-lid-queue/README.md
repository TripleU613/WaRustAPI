# Missing-LID preset and queue semantics, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238, pinned SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. xtask DEX fallback continues the diagnosed unavailable Android provider. No execution, live traffic or REA Evidence envelope claim.

## Reproduce

From repo root:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0p4;' --method 'A04' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0vW;' --method 'A00' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1Wa;' --method 'run' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0vt;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0vW;' --method 'A01' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0vY;' --method '<clinit>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0vY;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0p4;' --method 'A01' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0vJ;' --method '<clinit>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0wB;' --method 'run' --signatures --exceptions
```

All exports contain complete selected methods. Only the blocks explained below are interpreted; the large executor export is not a complete behavioral reconstruction. The prior [context enum export](../2.26.43.72-contact-backoff-gates/context-enum.txt) and [preflight caller](../2.26.43.72-prekey-lid-preflight/README.md) supply already retained inputs. checksums.json binds every other file here.

## Initial flags are not the final wire

0vJ initializer04b6..04d0 identifies A0h as LID_REQUEST (enum value47). The retained context initializer021e..0244 identifies0vC.A0M as MESSAGE_MULTI_PROTOCOL_QUERY, using mode0vE.A05 (QUERY). This is distinct from interactive phone lookup.

For0vY.A0N, initializer0180..01ae calls the14-boolean constructor with v18..v31. At that call only v26 is1; all others are0. In particular v21 was set0 at0040, v22 at0152, v23 at0048, v24/v25 at0076/007a; assignments0184..01a4 clear the remaining inputs. Constructor v2..v15 are those14 inputs, so its v10 (ninth input) is true. A04 is the OR of that input and the first two; all other stored protocol booleans are false. This explicit register mapping matters: obfuscated field letters alone are not evidence of protocol names.

0vt constructor copies0vY.A04 to0vt.A0N at00ec..00f4.0wB.run captures that into v24 at0176..017e; its lid_sync_backoff block0d1e..0d72 conditionally clears that same local. Thus the initial selected preset enables LID; contact A0K and sidelist A0T begin false. This does not prove that the diagnostic name requestSyncDevicesAndLidIfMissingLids means both protocols are requested, or that another protocol cannot be added by queue combination. The backoff block compares deadline>now (equal passes), but its complete reachability and native clock/account authority for this producer remain unported.

0vW.A01 adds non-null users to a HashSet. A00 validates some protocol is enabled then constructs0vt, whose constructor copies target set, flags, reason/context and an08q timestamp.0p4.A04 schedules1Wa selector14 through098.execute;1Wa.run case14 at01f8..020e calls0p4.A01. That method has actual combineRequests handling: it combines target collections at02cc..0308 and ORs LID flags at03c8..03e0 among other protocol merges. Its constructor/compatibility checks, preset-operation persistence, scheduling, offline gates and all concurrent configurations are not fully reconstructed. This is evidence against one-input-object==one-independent-IQ; it is not proof this particular job is always combined.

The retained executor eventually has multiple helpers and inline request constructors. No single final wire body, timer, response acceptance or mapping-adoption rule is asserted here. Full wire tracing must bind the actual post-combination preset, targets and effective gates, rather than reuse the separate lightweight/regular phone factory. No Rust request, retry, wait reset, namespace rewrite or live operation added.
