# Stanza corruption routing — APK2.26.43.72

**Confirmed static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
reverified2026-11-06. Complete selected DEX methods retain signatures and catch
tables. xtask fallback follows the already diagnosed unavailable Android
provider; these are not REA Evidence envelopes. No APK execution, account or
network traffic.

## Root tag decides recoverability

1rg.<clinit> constructs A05 from exactly five strings: receipt, notification,
message, call, status. These are **root stanza tags**, not the class attribute
of an ack. In the length-five branch08w.A0h allocates LinkedHashSet; A0i adds
each array element.07Z.A1W uses Collection.contains for that Set. The exact
physical field census has three references: initializer write and A03/A04 reads.
No other direct DEX access was found; reflection/dynamic mutation is not claimed
exhaustively excluded.

1rg.A04 reads node.A00 into v6 and tests that set at0046..0056. Nonmembership
branches to0142, which rethrows the original LIL outside its inner catch range.
Thus malformed root ack (regardless of its class) does not reach A04's
recoverable logging/NACK branch. Root receipt is a different case. This closes
the earlier generic assumption that every handled LIL simply returns to read.

For an admitted tag, A04 logs metadata, calls A01(node,4) only for root message,
then calls A03(node,487). Its inner Exception catch logs and returns if that
branch fails. A03 first calls A02 (not interpreted here), then for an admitted
tag creates Efa with the original root tag as class, required id, supplied
error string, timestamp sentinel-2 and optional typed from/type/participant,
and submits Eoh through0Sf.A0S. This is selected construction/submission, not
proof a NACK was written or accepted. A01's complete message diagnostic method
is retained but its full Dsn field meanings and CSW delivery are not ported.

## Malformed ACK leaves the selected reader

The linked [complete1s2.run](../2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
invokes the mapped1rA.A03 handler at106c..1072. The already described LIL catch
at10e6 calls1rg.A04 at10f0..10f6. For root ack the latter rethrows. The exported
outer range085c..08ba (code units) maps LIL to08d9, byte11b2. That catch logs
corrupt-stream metadata and at1222 branches to12a4. The latter selects error2,
calls1lX.onError at12be..12c4 and returns. It does not continue this reader.

The retained1lX declarations have no implementation body. Concrete listener
authority, connection-manager cleanup/reconnection, account state, telemetry
provider effects and all alternate failures remain open. The separate generic
throwable map-cleanup block1128..1174 is not automatically this LIL path; no
universal callback-map clearing is inferred. No error response is selected for
the malformed ACK in this path.

## Rust consequence

[Normal actor and comparison-wait correction](../../message-ack.md#malformed-ack-consumes-the-receive-epoch-after-capture-2026-11-06)
now captures then stops the receiving epoch before state processing or later
I/O. Stateful paths return a non-reconnectable local error and retain pending
reservations. This is not native onError2 policy emulation, automatic recovery,
account revocation or a generic487 sender for malformed inputs.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/1rg;' --method '<clinit>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rg;' --method A04 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rg;' --method A03 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rg;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/07Z;' --method A1W --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/08w;' --method A0h --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/08w;' --method A0i --signatures --exceptions
target/debug/xtask apk-field-refs "$APK" --class 'LX/1rg;' --field A05 --type 'Ljava/util/Set;'
target/debug/xtask apk-smali "$APK" --class 'LX/1lX;' --signatures --exceptions
```

The existing linked bundle retains complete1s2.run; it is not duplicated here.
checksums.json covers every other file in this bundle. No live acceptance or
restriction-prevention guarantee follows from static routing.
