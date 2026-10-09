# Linked metadata observers and platform selection

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Five-topic recipe](apk/recipes/2.26.37.71-linked-metadata-observers.json),
[selected full methods](apk/2.26.37.71-linked-metadata-observers/targets.json),
[full callback-name inventory](linked-metadata-observer-inventory.txt).
This follows [runnable metadata writes](linked-runnable-writes.md).
Offsets are method-relative bytes; retained exception ranges use code units.
No private account state was inspected or changed.

## Last-active platform flag

**Confirmed:** HxZ.A04 checks B6a.ordinal() and returns true exactly for 1..6,
false otherwise. B6a.<clinit> gives those ordinal names CHROME, FIREFOX, IE,
OPERA, SAFARI and EDGE. DM7 case 2 additionally admits ordinals 7 DESKTOP,
15 IOS_CATALYST and 21 UWP for its later companion-timestamp preference update.
These meanings come from constructor ordinal arguments and literal enum names,
not a guessed UI label or a supplied device identity. Full platform producer
validation remains **unknown**. This flag does not gate the selected last_active
SQL write and does not validate keys.

## View lookup is not raw-map lookup

**Confirmed:** 0jV.A03 resolves 1IW, invokes 1IY.A07 and gets the supplied
DeviceJid from that returned map (0028..0042). A07 is the saved own-PN view
builder traced in [linked-set storage](read-linked-set.md). This differs from
1IY.A02's already populated raw-map/alias fallback: a view lookup can load/build
its cache. No additional alias fallback or cryptographic verification is visible
in A03. Missing record returns null; complete producer authority remains unknown.

## Case 26 does not pass the stored record

**Confirmed:** the already retained
[D3f.COL](apk/2.26.37.71-companion-cleanup/identity-deleted-callback.txt)
case 26 enters 0516. It casts the observer to 0rY and invokes **Bjk()V**
at 051e..0524, then returns. It does not read D3f.A00 or pass HxZ to that
callback, despite metadata callers storing a record in the callback object.
The initial attempt to inspect D3f.run found no matching bodies; the correct
entry is COL, not a runnable worker. Complete 078 dispatch scheduling and
observer registration remain **unknown**.

**Confirmed scoped inventory:** the all-DEX method-name substring scan returned
17 methods in 17 classes. One is an unrelated crash-reporter lambda containing
Bjk in its name; it is not an observer implementation. Of 16 exact Bjk
methods, 0rY is abstract, thirteen concrete bodies immediately return, and two
have effects. This is physical source coverage, not proof every named class is
registered on this specific dispatcher, nor reflection/native closure.

- IS9.Bjk checks its discriminator equals 3, casts its held object to GJm and
  calls GJm.A0b (1RT).A0C(null). Other discriminator values return. Full state
  consumers remain unknown; this is not a key-verification result.
- 12J.Bjk calls 12H.A00. That method first checks 12I.A01. When enabled it
  iterates 0jV.A0O's list, obtains each HxZ DeviceJid, attempts a display name
  via 0jV.A0M, and catches Exception to fall back to A0C or B6a.name(). It
  constructs Hla(DeviceJid,name,HxZ.A01, membership in 0jV.A0R), publishes a
  replacement 12I through 0J9.A00, then calls 12H.A01. Disabled mode skips
  list reconstruction and still calls A01 with current state. Its diagnostic
  names EavesdropMonitor. Full A0O/A0M/A0R inputs, A01 downstream effects and
  publication guarantees remain unknown.

**Negative / bounded:** no scanned exact Bjk body directly decrypts, replaces a
Signal pin/session or retries a captured message. Transitive helpers remain
untraced, so this is not a whole-program absence claim. A general metadata
notification does not supply authenticated identity-change permission or prove
that a preceding SQL write committed. Full receive trust, linked-device lifecycle
and read-self fanout remain unfinished. No Rust runtime or live capability was
added, and no account reset or network request occurred.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-linked-metadata-observers.json \
  --output docs/protocol/apk/2.26.37.71-linked-metadata-observers --check
cargo run -p xtask -- apk-smali ~/Downloads/WhatsApp.apk \
  --class '' --method Bjk --signatures --exceptions
```
