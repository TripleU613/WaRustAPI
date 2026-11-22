# Login `ClientPayload` construction (Android primary)

Status: **partly implemented**. The Android user-agent block is filled from
configured, real device facts. The remaining top-level fields are traced below
but still absent (see [Gaps](#gaps-still-open)).

APK **2.26.43.72** (version code 264307238, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`), read with
`jadx 1.5.7` (sources only, `--no-res`). Class names are the JADX
de-obfuscation of the release DEX; anchors are given so a re-anchored build can
be re-checked by string/field rather than by name.

## Where the payload is built

**Confirmed static.** `X.C38091lc` (`X.1lc`) is the payload builder:

```java
C38111m3 builder = (C38111m3) C38101lh.DEFAULT_INSTANCE.createBuilder();
for (InterfaceC38121m4 filler : C0AJ.A00(this.A01, C09H.A04(prefs, 7483))) {
    filler.ABS(input, builder);
}
return (C38101lh) builder.build();
```

`C38101lh` is `ClientPayload`. The source set is built once in the constructor
with `C00D.A05(7482)`, so the enabled filler set is server-configured
(AB props **7482**/**7483**) and was not observed at runtime. Each filler is an
anonymous implementation of `InterfaceC38121m4.ABS(C38081lb input, builder)`;
`C38081lb` carries connection facts (attempt number, session counter, passive
flag, network state, DNS source, timings).

Field numbers read from `C38101lh`'s retained field-name array and
`CONNECT_*_FIELD_NUMBER` constants (they agree with the vendored
`proto/whatsmeow/waWa6/WAWebProtobufsWa6.proto` `ClientPayload`):

| # | field | # | field |
| --- | --- | --- | --- |
| 1 | username | 23 | oc |
| 3 | passive | 24 | lc |
| 5 | userAgent | 30 | iosAppExtension |
| 6 | webInfo | 31 | fbAppId |
| 7 | pushName | 32 | fbDeviceId |
| 9 | sessionId | 33 | pull |
| 10 | shortConnect | 34 | paddingBytes |
| 12 | connectType | 36 | yearClass |
| 13 | connectReason | 37 | memClass |
| 14 | shards | 38 | interopData |
| 15 | dnsSource | 40 | trafficAnonymization |
| 16 | connectAttemptCount | 41 | lidDbMigrated |
| 18 | device | 42 | accountType |
| 19 | devicePairingData | 43 | connectionSequenceInfo |
| 20 | product | 44 | paaLink |
| 21 | fbCat | 45 | preacksCount |
| 22 | fbUserAgent | 46 | processingQueueSize |

`pairedPeripherals` (47) and `testIsolationId` (48) are also present. The
release descriptor's `RawMessageInfo` names a `messageSts` field that the
vendored schema does not have; its number was **not** recovered (the ordinal
string alone is not a field number).

## Android user agent (`case 456`, class `X.1m5`)

**Confirmed static.** This filler always writes `platform = ANDROID` and then:

| Payload field | Source in 2.26.43.72 | Notes |
| --- | --- | --- |
| `appVersion` | `AbstractC38171m9.A00()` | splits the literal `"2.26.43.72"` on `.`; requires ≥3 parts; always sets primary..tertiary, sets quaternary only for a 4-part version |
| `mcc`, `mnc` | `TelephonyManager.getNetworkOperator()` → `C38201mj.A00(op, "000", "000")` | regex `(\d{3})(\d{2,3})`, `mnc` re-formatted `%03d`; both fall back to `"000"`; written only when a `TelephonyManager` exists |
| `osVersion` | `C38211mk.A05` = `Build.VERSION.RELEASE` | |
| `manufacturer` | `C38211mk.A03` = `Build.MANUFACTURER` | |
| `device` | `C38211mk.A06` lazy value | `Build.DEVICE`; only appends `_cheets`/`_alexatab`/`_fireos`/`_gbos` on those platforms, otherwise plain |
| `osBuildNumber` | `C38211mk.A02` = `Build.DISPLAY` | |
| `deviceBoard` | `C38211mk.A01` = `Build.BOARD` | omitted when empty |
| `deviceModelType` | `C38211mk.A04` = `Build.MODEL` | the marketing model; `device` above is *not* the model |
| `phoneId` | `C38231mm.AzD().A01` | random UUID generated once and persisted with a timestamp (`phoneid_id`/`phoneid_timestamp`) |
| `deviceExpId` | `StringUtils.A09(X.1JX.A04())` | `X.1JX.A04` persists a `perf_device_id` UUID in `DebugWaSharedPreferences` (a `UUID.nameUUIDFromBytes` of the OS performance id when present, else a random UUID); `StringUtils.A09` converts it to its 16 raw bytes and Android Base64 flags 11 (URL-safe, no wrap, no padding) |
| `deviceType` | device-form setting | PHONE default, else TABLET/VR/DESKTOP |
| `localeLanguageIso6391` | locale language | omitted when empty or `"zz"` |
| `localeCountryIso31661Alpha2` | locale country | omitted when `"ZZ"` |

Also written by this filler, at top level: `oc`, `yearClass`, `memClass`.

- `oc` starts as `AbstractC38331mx.A00(application) == 1`, then, only when that
  is true, is re-evaluated as `!HSb.A00(packageName, signature)` where `HSb`
  XOR-decodes a fixed key and compares against an allowlist of signature bytes.
  An app with an allow-listed signature therefore ends with `oc = false`. The
  semantic meaning of the flag is **not** established from this trace.
- `yearClass` = `C38351mz.A01(prefs, app)`: a **RAM/CPU class**, not the model's
  release year. It is cached in `startup_prefs`
  (`year_class_cached_value_2016_pref`) and computed from
  `ActivityManager.MemoryInfo.totalMem` with a core-count tie-break
  (`AbstractC50846MVc.A02/A01/A00`): ≤768 MiB → 2010/2009, ≤1 GiB → 2012/2011,
  ≤1.5 GiB → 2013, ≤2 GiB → 2013, ≤3 GiB → 2014, ≤5 GiB → 2015, else 2016. The
  observed range is therefore **2009..=2016** and Linux-swap/RAM class, not
  hardware age.
- `memClass` = `C1n0.A01(app)` = `ActivityManager.getMemoryClass()` in MiB
  (the app heap growth limit, e.g. 256 on a 6 GiB phone).

## Other confirmed fillers

- `case 457` (`X.1n1`): `trafficAnonymization = STANDARD` when the
  server-configured experiment is on, else `OFF` (AB prop **4982**).
- `case 458` (`X.1n3`): always writes `connectType` from the live network
  (WiFi → `WIFI_UNKNOWN`; cellular subtype; otherwise `CELLULAR_UNKNOWN`),
  `dnsSource` `{dnsMethod, appCached}` from the resolver that answered,
  `connectAttemptCount` = the login context's `attemptedSuccessfulConnections`,
  and a packed `connectionSequenceInfo` (43). Confirmed bit layout of the
  packed value: bits 0..2 port class (80→0, 443→1, 5222→2, other→3), bits 2..5
  host-type code via `AbstractC37551kP.A00`, bit 5 `!pref.A02`, bits 7..12 the
  sequence step (`& 31`), bit 12 `networkProvider.A0P() == true`. The host-type
  input (`ConnectionMetadata.A06`), sequence step and the two provider booleans
  remain untraced, so Rust writes none of these four fields yet.
  `X.1je.A01` shows the resolver's own hardcoded path for `g.whatsapp.net` is
  `dnsMethod=HARDCODED, appCached=false`; the system path is
  `dnsMethod=SYSTEM, appCached=false`.
- `case 459` (`X.1n9`): `passive`; `sessionId` from the caller's
  `ConnectionMetadata.sessionId`; `shortConnect` from a preference; `lc` from the
  persisted `connection_lc` counter; `connectReason` written as `UNKNOWN` first
  and upgraded to `USER_ACTIVATED` (1) or `PUSH` (2) only when the trigger is
  under 10 s old. `lidDbMigrated`, `messageSts`, `preacksCount` and
  `processingQueueSize` are conditional on companion/queue facts.

  The two counters were traced to their producers in this build:
  `X.C0ZJ.A1k` runs after the connect routine and both increments
  `X.C0ZJ.A00` (the in-memory `attemptedSuccessfulConnections` passed as
  `XmppLoginContext.attemptedSuccessfulConnections`, read in `A0z` and written
  as `connectAttemptCount`) and calls `X.1jy.A05`, which increments the
  persisted `connection_lc` preference (wrapping `Integer.MAX_VALUE` to 0).
  `ConnectionMetadata.sessionId` is not a counter: `ConnectionThread/tryOpenSocket`
  logs `socket connected; sessionId=<n>` for `nextInt()` of a `Random` created
  per socket, and that value is passed to the metadata constructor.
- `case 460` and the remaining fillers were not read.
- `case 461` (`X.1nX`): `username` from the login JID's numeric user part
  (`userJid.user`, or the LID prefix for a LID login, which also sets
  `product = WHATSAPP_LID`), and `pushName` from the saved `push_name`
  preference (`X.C09O.B2L`, default empty) when non-empty. A login with neither
  fact leaves both fields absent.

## Rust implementation

`LoginDevice` (`crates/whatsapp-api/src/registration/request_config.rs`) holds
the user-agent facts plus `year_class`, `mem_class` and
`traffic_anonymization`. `primary_login` fills them **only when configured**, so
an old profile keeps the previous minimal payload and no value is invented.
Validation refuses values the APK cannot produce: printable ASCII 1..=64 for
the text facts, exactly three digits for `mcc`/`mnc`, a canonical UUID for
`phone_id`, `year_class` 2000..=2100, `mem_class` 16..=2048.

Example (private JSON; the values below are what the plugged-in Pixel 4a
reports today, with `<mcc>`/`<mnc>` left as the operator's own network code — a
tracked file must not carry a personal carrier identity):

```json
"client": {
  "profile_id": "android-2.26.37.70-registration-lab",
  "user_agent": "WhatsApp/2.26.43.72 Android/17 Device/Google-Pixel 4a",
  "device": {
    "os_version": "17",
    "manufacturer": "Google",
    "device": "sunfish",
    "model": "Pixel 4a",
    "os_build_number": "CP2A.260605.016",
    "board": "sunfish",
    "mcc": "<mcc>",
    "mnc": "<mnc>",
    "phone_id": "<uuid>",
    "perf_device_id": "<uuid>",
    "year_class": 2016,
    "mem_class": 256,
    "traffic_anonymization": "off"
  }
}
```

`preview` reports which device facts are configured, never their values.

**Install identifiers.** `phone_id` and `perf_device_id` are persisted
per-install UUIDs. `accounts new` writes a fresh canonical UUID v4 for each,
the way a fresh install generates them; `primary_login` sends `phone_id` as-is
(`userAgent.phoneId`) and `perf_device_id` as `userAgent.deviceExpId`, which is
the URL-safe unpadded Base64 of its 16 raw bytes. They are never rotated per
request. Unit vectors compare the encoder with independent `base64.urlsafe`
output for two UUIDs.

**Display name.** `primary_login` takes the name this account already sent,
read from the `push-name` record written when its presence went out
(`presence::saved_push_name`, the analogue of the APK's saved `push_name`
preference). It is trimmed and validated with the same rule as the presence
stanza and dropped if unusable; when nothing was ever sent, `pushName` stays
absent rather than empty. The caller passes it explicitly, so a login builder
cannot silently invent a name.

**Connection facts.** Every login carries `LoginConnection`: the reason, the
finished-attempt count (`connectAttemptCount` 16 and `lc` 24), and a random
`sessionId` (9). The count is persisted under the private checkpoint
`comparison-connection-counter-v1` and advances once after a login stanza was
written, wrapping like `X.1jy.A05`; the payload therefore carries the number of
attempts that already finished. The APK keeps `lc` in preferences and the thread
counter in memory, so this client models both with the persisted value: they
agree within one process, and after a restart the second can be higher than the
APK's in-memory counter. `sessionId` is four fresh random bytes per attempt,
matching the APK's `Random.nextInt()`.

**Connect reason.** An explicit request (`connect`, `listen` start,
`lab-transport --login`) is the APK's `USER_ACTIVATED` trigger. Every automatic
reconnect carries `UNKNOWN`, the value `X.1n9` writes before any ten-second
upgrade, so a retry never claims a user action that did not happen.

**Consistency guard.** The registration User-Agent is produced by the same
`Build.*` reads (`0re.A00`), so one device must not be reported two ways. When
`client.user_agent` has the APK shape
`WhatsApp/<version> Android/<release> Device/<manufacturer>-<model>`, validation
requires its release, manufacturer and model to equal `client.device`'s
`os_version`, `manufacturer` and `model`; a disagreement is refused with the
offending `client.user_agent …` field before any request. A UA outside that
shape is not parsed as a device claim and is left to the operator (historical
custom values stay readable). A hyphenated model such as `SM-G975F` keeps its
hyphens after the first separator.

## Gaps still open

- Which fillers the server actually enables (AB props 7482/7483) — unknown.
- `connectType`/`dnsSource`/`connectAttemptCount`'s connection-sequence packing
  (`connectionSequenceInfo`, 43): the values come from live network state and a
  packed bit field that was not re-read.
- `oc` semantics; this client never sets it.
- `messageSts` field number, and whether `product`, `shards`, `pull`,
  `paddingBytes`, `lidDbMigrated`, `paaLink`, `preacksCount`,
  `processingQueueSize` or `interopData` are written for an ordinary primary
  Android login.

No live login was performed for this change; it is an offline construction
parity change.

## Observed device facts (2026-11-22, USB-connected Pixel 4a)

**Observed**, read with `adb shell getprop …`, `/proc/meminfo` and
`dumpsys telephony.registry`; these are the values `Build.*` hands the filler,
not values invented for a test:

| Fact | Device value | Command |
| --- | --- | --- |
| `ro.product.model` | `Pixel 4a` | `getprop ro.product.model` |
| `ro.product.manufacturer` | `Google` | `getprop ro.product.manufacturer` |
| `ro.product.device` | `sunfish` | `getprop ro.product.device` |
| `ro.product.board` | `sunfish` | `getprop ro.product.board` |
| `ro.build.version.release` | `17` | `getprop ro.build.version.release` |
| `ro.build.display.id` | `CP2A.260605.016` | `getprop ro.build.display.id` |
| `ro.build.version.sdk` | `37` | `getprop ro.build.version.sdk` |
| `dalvik.vm.heapgrowthlimit` | `256m` → `memClass` 256 | `getprop dalvik.vm.heapgrowthlimit` |
| `MemTotal` | 5,717,360 kB ≈ 5.45 GiB → `yearClass` **2016** (>5 GiB branch) | `/proc/meminfo` |

Two caveats worth keeping: the device's `ro.build.fingerprint` still names
`…:13/TQ3A.230805.001.S1:…` while `ro.build.version.release` reports `17`, so
this handset's release/display props do not match its fingerprint — the APK
sends the props it reads, and so does this client. The network operator code
(`TelephonyManager.getNetworkOperator()`, derived from
`gsm.operator.numeric`/registration state, `mcc`/`mnc`) is deliberately not
recorded here.
