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
| `deviceExpId` | `StringUtils.A09(...A0J().A04())` | provider body not traced; see gaps |
| `deviceType` | device-form setting | PHONE default, else TABLET/VR/DESKTOP |
| `localeLanguageIso6391` | locale language | omitted when empty or `"zz"` |
| `localeCountryIso31661Alpha2` | locale country | omitted when `"ZZ"` |

Also written by this filler, at top level: `oc`, `yearClass`, `memClass`.

- `oc` starts as `AbstractC38331mx.A00(application) == 1`, then, only when that
  is true, is re-evaluated as `!HSb.A00(packageName, signature)` where `HSb`
  XOR-decodes a fixed key and compares against an allowlist of signature bytes.
  An app with an allow-listed signature therefore ends with `oc = false`. The
  semantic meaning of the flag is **not** established from this trace.
- `yearClass` = `C38351mz.A01(prefs, app)`: a RAM/CPU-derived performance year
  cached in `startup_prefs` (`year_class_cached_value_2016_pref`); e.g. ≤768 MiB
  → 2010/2009, ≤1 GiB → 2012/2011, ≤1.5 GiB → 2013, ≤3 GiB → 2014, ≤5 GiB →
  2015, else 2016.
- `memClass` = `C1n0.A01(app)` = `ActivityManager.getMemoryClass()` in MiB.

## Other confirmed fillers

- `case 457` (`X.1n1`): `trafficAnonymization = STANDARD` when the
  server-configured experiment is on, else `OFF` (AB prop **4982**).
- `case 458` (`X.1n3`): `connectType` from the live network (WiFi →
  `WIFI_UNKNOWN`; cellular subtype mapping; otherwise `CELLULAR_UNKNOWN`),
  `dnsSource` from the resolution method, `connectAttemptCount`, and a packed
  `connectionSequenceInfo`. Full bit layout not re-read.
- `case 459` (`X.1n9`): `passive`; `sessionId` from the caller's per-connection
  counter; `shortConnect` from a preference; `lc` from the persisted
  `connection_lc` counter; `connectReason` written as `UNKNOWN` first and
  upgraded to `USER_ACTIVATED` (1) or `PUSH` (2) only when the trigger is under
  10 s old. `lidDbMigrated`, `messageSts`, `preacksCount` and
  `processingQueueSize` are conditional on companion/queue facts.
- `case 460` and the remaining fillers were not read.

## Rust implementation

`LoginDevice` (`crates/whatsapp-api/src/registration/request_config.rs`) holds
the user-agent facts plus `year_class`, `mem_class` and
`traffic_anonymization`. `primary_login` fills them **only when configured**, so
an old profile keeps the previous minimal payload and no value is invented.
Validation refuses values the APK cannot produce: printable ASCII 1..=64 for
the text facts, exactly three digits for `mcc`/`mnc`, a canonical UUID for
`phone_id`, `year_class` 2000..=2100, `mem_class` 16..=2048.

Example (private JSON; values are the operator's real device facts):

```json
"client": {
  "profile_id": "android-2.26.37.70-registration-lab",
  "user_agent": "WhatsApp/2.26.43.72 Android/13 Device/Google-Pixel 4a",
  "device": {
    "os_version": "13",
    "manufacturer": "Google",
    "device": "sunfish",
    "model": "Pixel 4a",
    "os_build_number": "TQ3A.230805.001.S1",
    "board": "sunfish",
    "mcc": "000",
    "mnc": "000",
    "phone_id": "<uuid>",
    "year_class": 2020,
    "mem_class": 512,
    "traffic_anonymization": "off"
  }
}
```

`preview` reports which device facts are configured, never their values.

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
- `sessionId`/`lc`: a per-connection counter and a persisted connection
  counter. Rust has no persisted counter yet.
- `connectType`/`dnsSource`/`connectAttemptCount`/`connectionSequenceInfo`.
- `phone_id` auto-generation and persistence (currently operator-supplied only).
- `deviceExpId` producer body (`StringUtils.A09` input source).
- `oc` semantics; this client never sets it.
- `messageSts` field number, and whether `product`, `shards`, `pull`,
  `paddingBytes`, `lidDbMigrated`, `paaLink`, `preacksCount`,
  `processingQueueSize` or `interopData` are written for an ordinary primary
  Android login.

No live login was performed for this change; it is an offline construction
parity change.
