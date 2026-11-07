# MobileConfig network fetch route and MEX callback

**Confirmed static**, APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Repository DEX exports and selected persisted-ID extraction, not REA Evidence
envelopes, APK execution, observed requests or effective account values.

## Distinct missing network pipeline

The declared interface-aware override scan covers114178 APK classes and finds
Hmm implementing MobileConfigFetcher.fetch plus the bodyless interface. The
interface accepts strings,map,handler and optional stack integer; these logical
endpoint strings are not demonstrated HTTP URLs. Hmm dispatches mobileconfig,
mobileconfig_emergency_push, logging, sessionless and QE-name branches separately.
Only selected configuration-fetch behavior is described here; logging/WWW/QE
callbacks and all producer parameters are not claimed fully traced.

Gf7's enum initializes Erlang=0,WWW=1,IGServer=2. For the main configuration
endpoints, stack1 plus property0x7237 selects WWW MetaConfigFetchMutation in
whatsapp-android-www. Otherwise selected A13/GuG.A00 gating can complete locally
with false and{} rather than fetching. The MEX path uses MobileConfigFetchQuery
inwhatsapp-android-mex and modelG7o. The complete source retains branch/order
and emergency endpoint selection. These gates are not equated to a live account's
actual values, and no fallback stack or flag override is fabricated.

Rust abprops.rs explicitly treats saved AB replies as evidence only; search of
the transport implementation found no MobileConfigFetchQuery/MetaConfigFetchMutation
adapter or table-hydration pipeline. This MEX/SDK route is distinct from AB-property
inspection and is a confirmed implementation gap. Absence alone does not establish
that every connection must issue it, or that omission causes a restriction.

## MEX input construction

The selected MEX GraphQlCallInput setters receive:

| Field | Source/default at this branch |
| --- | --- |
| api_version | parsed map integer,0 if missing/invalid |
| fetch_type | map string or empty |
| unit_type | unit1 SESSIONLESS,unit2 SESSIONBASED,otherwise UNKNOWN |
| client_config_hash | global_value_hash map string or empty |
| ep_refresh_id | parsed map integer or0 |
| mc_debug_id | GwY provider string; producer/value not resolved here |
| queries | null setter input |
| query_hash | null setter input |
| query_string | null setter input |

Null setter input is not a claim that final JSON serializes explicitnull; the
serializer and final transport envelope need separate confirmation. SYNC_FULL
and emergency booleans are retained for the callback. No query hash, unit ID,
debug ID or configuration hash is invented for Rust. The WWW branch constructs
a different field set and authentication flags; it is not interchangeable.

Selected persisted IDs reproduced from the packaged MEX asset:

| Operation | Server doc_id |
| --- | --- |
| MobileConfigFetchQuery | 29248978061376279 |
| MobileConfigConsistencyLoggingQuery | 34033663249614454 |

persisted-ids.json retains namespace, asset hash, client and server identities.
A doc ID establishes packaged query selection, not variables, scheduling or
permission to send. Logging operation ID retention is not a logging-protocol port.

## MEX result ordering

IDq selector0 installs success selector1 and errorIE9 selector30. Constructors
are retained to prove captured-field permutation. Success selector1 schedules
ICt selector0 throughHmm.A01: a GuG.A01 bit chooses098 executor versus direct
invoke. This is worker selection, not retry/backoff authority.

ICt looks upxwa2_mobile_config_fetch.fetch_result_json. A missing/null getter
result reports onComplete(false,"No result from server"). A nonnull string,
including empty, follows the success path; this Java branch does not prove the
string is valid JSON or usable table data. It may first save optionalep_refresh_id
using captured request unit_type/unit_id viaHmm.A00; absent required metadata
skips that preference update. It then calls onComplete(true,raw_string), with
selected telemetry around the callback, followed by additional SYNC_FULL ab_key
preference/notification work and optional WWW follow-up on gated branches.
Exact full methods and exception scopes are retained; side effects are not
assumed atomic across these callbacks/preferences.

MobileConfigFetcherHandler.onComplete delegates toonCompletion, which has no
DEX body in this export and loads mobileconfig-jni. Forwarding success here is
not proof of native parsing, file commit, cache invalidation, token-policy
activation or server acceptance. The IE9 error callback and native completion
are next boundaries; no generic success/repair behavior is substituted.

## Reproduction and remaining work

Existing apk-virtual-overrides with --include-interfaces targets the exact
MobileConfigFetcher.fetch prototype retained in the file. apk-smali with
--signatures --exceptions exports the interface/handler,Hmm.A00/A01,IDq/ICt
constructors andinvoke,Gf7.<clinit>,G7o. Selected IDs reproduce with:

```sh
target/debug/xtask apk-mex-queries ../android-lab/apks/WhatsApp.1.apk --operation MobileConfigFetchQuery --operation MobileConfigConsistencyLoggingQuery --check docs/protocol/apk/2.26.43.72-mobileconfig-fetch-route/persisted-ids.json
```

No Rust request or persistent setting changed. Producers, request envelope/null
serialization, full WWW/error paths, native persistence, cache readiness and
effective timing remain unfinished. No private account files, live traffic,
resets or account operations. Full protocol parity and zero bans are unproven.
