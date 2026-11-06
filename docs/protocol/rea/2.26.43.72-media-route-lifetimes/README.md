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

**Unknown at the initial trace:** the origin of callback A00/base (closed by
the follow-up below), every deadline consumer, refresh
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

## Request clock origin follow-up

**Confirmed static:** `0VM.A07` obtains the current route's optional last ID,
checks throttle and callback in-flight state, hands the IQ to `0Sf.A0R` with
32000-ms timeout, then stores SystemClock.elapsedRealtime into `1xY.A00`.
`1xY.CK4` consumes that field as the constructor base before clearing it.
The checked field-write census identifies four direct writes across DEX: the
request handler and callback timeout/error/result resets. The complete checked
callback class retains those reset instructions. Native deadlines therefore use
the request handoff clock, not callback receipt time.

**Confirmed Rust defect/fixed:** the CLI media-route result overwrote its
reservation timestamp with response time. It now retains explicit requested_at
through reservation/result commits. Upload route expiry uses this request time;
old caches without it refuse and require explicit refresh. No historic timestamp
is reconstructed from receipt time. The local wall-clock reservation precedes
socket write, conservatively earlier than native post-handoff time; this is not
complete native monotonic-clock/lifecycle parity. No automatic route refresh.

Request builder conditional last_id, full throttle policy, callback timeout/error
side effects, host choice and all consumers of both deadlines remain unported or
untraced. Capturing this metadata does not authorize a new request or retries.

- [request-clock.json](request-clock.json) — SHA-256 `71dc8e661cc0a7988fdc616bbc40f1efae7be8b7e0f4cfb881e89b1b08c522fe`.
- [request-clock-checked.txt](request-clock-checked.txt) — SHA-256 `13feab4c4b06c35965a573fb368b9a86f70bdd577ffa200969e7834a440bf2bb`.
- [clock-writers.txt](clock-writers.txt) — SHA-256 `cf24d23e0c4d99095e26a9ef482caecb36f2ad7f1db46b94367e16295a98f4ee`.
- [callback-class-checked.txt](callback-class-checked.txt) — SHA-256 `a88c3e3d59559f311c45550abc8528e7ccd83e31ad169a3db45ad99aa151fe7b`.

## Conditional merge/current-route follow-up

**Confirmed static:** `0VM.A0P` returns the current route under its lock without
filtering expiry. `0VM.A0A` installs a whole reply if no prior object exists or
is_new is true. With a prior object and false is_new, it constructs a new object
using prior A0C hosts and the new response's auth/lifetimes/base/IDs/metadata.
This closes the getter/setter branch, not all refresh/deadline consumers.

At this tracing stage Rust refused non-new standalone cached replies. The
[subsequent prospective wrapper](../../media-network-outcomes.md#prospective-refresh-integration-2026-11-04)
now supplies prior-host binding and atomic merge for new contexts; legacy
non-new standalone cached replies remain unsupported. The prospective wrapper
accepts first-route false replies because its frozen prior route is absent;
subsequent false replies retain the frozen prior hosts. No server refusal/retry requirement is inferred.

- [current-route.json](current-route.json) — SHA-256 `139b8e70d071687bf8a23616ec35a398222c02fef29feaee7ff5a49b542bd258`.
- [route-setter.json](route-setter.json) — SHA-256 `6cf815f762fd3aeb71e06152287c950e38c4b5d3ba2e92c174929099ea3e39cc`.
- [route-setter-checked.txt](route-setter-checked.txt) — SHA-256 `6d1319f4e94450986a591b96ce315b293d91d45f080c8e5b73959c5fbe842d4f`.
- [current-route-checked.txt](current-route-checked.txt) — SHA-256 `85ae1507488bc0133b895f1f0f931a4aa134b5ed1b7e5db1c7738b4294e2dff7`.

## Explicit media refusal throttle

**Confirmed static**, `1xY.Bvj`: clear in-flight request time, iterate error
children, skip missing/zero or malformed code/backoff entries, and consume the
first valid nonzero code/backoff pair. Both attributes use default-zero Java
integer readers. Code507 with positive backoff sets the media throttle deadline
to current 08q clock plus min(backoff,10800) seconds. Code503 resets that
throttle and invokes a separate randomized Fibonacci Handler retry path. Other
valid nonzero codes terminate without changing the throttle.

Existing checked callback class offsets `00e2..0158` corroborate 503/507 branches,
positive check, cap constant and deadline store. Its `0xcb` arithmetic opcode is
not symbolized by the current xtask; the addition expression is recovered Java
source rather than claimed fully symbolized DEX. The cap double literal
0x40c5180000000000 independently decodes to 10800.0. Success CK4 resets the
throttle before installing routing metadata, as retained in callback.json.

Rust now persists the confirmed throttle/reset observations and rejects explicit
refresh preparation while now <= deadline. This is durable local continuity; the
native throttle is in-memory. No native Fibonacci scheduler or automatic retry
is claimed. Parser corruption does not fabricate a wait. Existing stored auth
faults cannot be erased by treating a reset as an absent checkpoint.

- [error-throttle.json](error-throttle.json) — SHA-256 `3b67bb39b90c37240151901fe71738eef35dd19f73a34d114c481c2b345ecd52`.
