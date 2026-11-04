# Current media route lifetime readers

**Confirmed static**, WhatsApp Android 2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution or network request. REA/JADX 0.7.1 complete method envelopes
retain tool identity, limitations and exact DEX signatures.

- `LX/1xY;.CK4` (`X.C1xY`) obtains the required media_conn child, auth string
  and both `ttl` and `auth_ttl` through `0Tp.A07`. Optional fields explicitly
  use default-bearing helpers, unlike those lifetimes. Checked DEX offsets
  `0052..0068` corroborate both required-long calls.
- `0Tp.A07` calls required attribute `A0L`, then `A09`. A0L throws when absent;
  A09 calls `Long.parseLong` and converts NumberFormatException into LIL.
  No missing-auth-TTL fallback was found in this direct callback chain.
- `1t5.<init>` stores both lifetimes and computes route deadline A04 as
  `base + ttl*1000` and auth deadline A02 as `base + auth_ttl*1000`. Checked
  DEX confirms the arithmetic. This does not prove overflow-safe native math.
- `0VM.A08` conditionally restores JSON under property149, reads required
  conn_ttl/auth_ttl, converts saved absolute send time to elapsedRealtime, and
  installs only if no current route and the restored auth deadline is future.

**Unknown:** the origin of callback A00/base, every deadline consumer, refresh
ordering, overflow/recovery policy, effective property149, full host choice,
server-side behavior and ban implications. Rust's conservative minimum-lifetime
guard is not proven to match every native route/auth decision. Native retry
metadata defaults do not authorize implementing automatic retries.

The initial `X.1xY` name did not resolve; class-name discovery returned
`X.C1xY`, matching raw DEX `LX/1xY;`. That lookup failure is not evidence of an
absent native callback. String references are bounded to exact literal matches.

## Retained files

- [cache-reader.json](cache-reader.json) — SHA-256 `de8b95e6be9422c26e9a9968bb7980df596fc0953c556a0cfad5d6331e771749`.
- [callback-checked.txt](callback-checked.txt) — SHA-256 `ac6714ea1f05aaeb5ae2ef6a4d47938c97eb7df0b1f3a9ace5ff7013a6d52acf`.
- [callback.json](callback.json) — SHA-256 `b9d4204acdc32b4221cacdcd533716e4244c40131a98efa59a436905122780f0`.
- [model-checked.txt](model-checked.txt) — SHA-256 `871a384f4596e23a069f52000021b7d96b64a31eb9f3c9920b046c1dd4712c1c`.
- [parse-long.json](parse-long.json) — SHA-256 `4467f6f0525f666eae70df2de1526186117a1002405960d692b71a53c124d763`.
- [required-attribute.json](required-attribute.json) — SHA-256 `a1268839c7ef4d7405e31463d0d0a6984a10c647c1f75b1bae418808da6793c9`.
- [required-long.json](required-long.json) — SHA-256 `3c7181bb6178d8ffbb2595289c4f11e53a4cb1bf2d57d9c3965cc688569bd3b7`.
- [route-model.json](route-model.json) — SHA-256 `50cf70ace08c5ece63dd689a408d004cd00db824cd0c675f5b36d3d4cd9628d4`.
- [string-xref.txt](string-xref.txt) — SHA-256 `0b8376e57ced07ee90524be2965a4b9190f3eb3e0676ef0ae7a8190b4b0c2674`.

Reproduce checked output with `xtask apk-xref APK --anchor auth_ttl --anchor
media_conn`, `xtask apk-smali APK --class 'LX/1xY;' --method CK4 --signatures
--exceptions`, and the same smali command with `LX/1t5;` / `<init>`.
