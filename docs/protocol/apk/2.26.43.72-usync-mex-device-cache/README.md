# UsyncQuery per-user device cache fields, APK43.72

Confirmed static APK2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues diagnosed unavailable Android provider. Complete selected methods retained; no execution, live traffic or REA Evidence envelope claim. checksums.json binds every other file.

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0jD;' --method 'A07' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0jD;' --method 'A00' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0jD;' --method 'A04' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Nha;' --method 'A00' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Fwt;' --method 'A1H' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/F2N;' --method 'A03' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Nha;' --method 'A01' --signatures --exceptions
```

From repo root. Reuse [HSo builder](../2.26.43.72-prekey-lid-mex-selection/profile-dispatch.txt), [manager-to-user flags](../2.26.43.72-prekey-lid-query-route/user-flags.txt), [H6w field transfers](../2.26.43.72-contact-factory-selection/user-record.txt), and [variable map conversion](../2.26.43.72-usync-mex-variables/serialization.txt).

## Actual device gate and cache provenance

HSo.A01 at0172..018e requiresH6w.A0I=true and a non-null/nonemptyH6w.A09 hash before creating the per-user devices object. The independent include_devices aggregate elsewhere comes fromA0I, so requesting devices does not imply a per-user cached hash object exists. LID-onlyA0M is a separate field; do not manufacture cache data for a LID-only request.

The preceding0p7.A0B copies3GQ.A02 toHck.A0E, then when enabled reads0jD.A07 hash intoHck.A06,0jD.A00 timestamp intoHck.A02 and0jD.A04(...).A02 expected timestamp intoHck.A01 (zero if absent). H6w transfers these toA0I,A09,A02,A01 respectively.0jD.A07 snapshots a copied set fromA0C; empty/null-target paths return empty string, otherwiseF2N.A03 selects hash formatterA05 versusA04 by property0x415b. The collection source, both formatters and effective property value are untraced here; no single hash format is inferred.

0jD.A00 returns0 ifA04 finds no record, otherwise2Eu.A05. A04 has separate self/other and configured alias/storage paths, including0mi.A08 and2Eq.A01. Complete store/authentication/alias authority, atomicity across the separate reads and timestamp units are not established. These are cached fields, not permission to substitute local wall-clock time or invented metadata.

## Gated timestamps are decimal strings

HSo initializes expected_timestamp/hash/timestamp entries to null in a freshNha, then appends the actual nonempty hash. WhenH6w.A02>0 it adds timestamp throughFwt.A1H; only within that positive-timestamp branch does it add expected_timestamp whenH6w.A01>H6w.A02. Equal/older expected values do not take that write branch. Fwt.A1H explicitly callsString.valueOf(long), thenNha.A00, so non-null timestamps reach variable-map conversion as decimal Strings, not JSON Number objects.

Nha.A00 appends key/value entries and increments its count; its privateA01 validates mutability/key, it does not remove prior keys. RetainedGraphQlCallInput.A01 visits stored entries in order andMap.put overwrites an earlier key with its later value. Thus actual hash/timestamp replace initial nulls at the intermediate map stage. The map conversion retains null values; exact AndroidJSONObject/Pando null serialization and all final wire representations remain untraced here, so the initialized-null entries are not described as omitted or as a universal wire-null requirement.

## Remaining obligations

Effective device/hash configuration, complete cached record provenance/units, alias normalization, per-request coherent snapshots, null serialization, full per-user variables and authenticated response adoption remain incomplete. These findings constrain a future port: decimal-string timestamps, ordered comparison gates and real cache authority must be reproduced from appropriate evidence. No Rust UsyncQuery/device preflight, cache fabrication, retry or live operation added.
