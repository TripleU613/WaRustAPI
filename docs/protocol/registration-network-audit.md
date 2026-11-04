# Registration network audit — 2026-11-04

Offline request ownership review. This ledger concerns correctness and conditional
traffic, not adding registration methods or establishing ban causation.

## Confirmed Rust request surface

`registration/wire.rs::RegistrationEndpoint` catalogs 17 paths from comparison
APK 2.26.37.70. Catalog membership does not mean a request is implemented or sent.
A source search for endpoint variants and inspection of
`registration/factory.rs::StoredComparisonRequestFactory` found six constructed
endpoint families:

| Path | Caller/condition in the saved comparison flow |
| --- | --- |
| `/v2/exist` | Explicit same-identity probe, opt-in preflight, or consent inspection with no decision |
| `/v2/code` | Explicit SMS/voice request or supported email second-factor request; saved attempt/history and waits apply |
| `/v2/register` | Explicit entered OTP completion; same saved material/key bundle |
| `/v2/security` | Explicit account PIN verification; saved challenge and guessing wait apply |
| `/v2/consent` | Actual supplied consent/DOB decision; saved applicable context |
| `/v2/challenge` | Supported explicit challenge parameters; same saved key bundle |

No factory request was found for cataloged `reg_onboard_abprop`, `acverify_request`,
`acverify`, `autoconf`, `autoconf_verifier`, `device_logout_send`,
`device_logout_fetch`, `client_log`, `pre_pn_client_log`, `passkey_auth`, or `wfs`.
This is a source-ownership gap, **not** proof each is mandatory for SMS/voice.
Their producer callers and runtime gates must be traced before changing traffic.

The constrained HTTPS transport uses certificate-verified HTTPS to
`https://v.whatsapp.net/`, rejects redirects, disables automatic retries and
bounds time/body. It differentiates pre-write connector/build errors from later
uncertain outcomes. `ComparisonHttpFlow::send` does not infer success from HTTP
2xx alone; unreadable success/5xx/408/429 remain uncertain. Application response
branches preserve waits/challenges. These are existing Rust properties, not new
proof of complete current Android HTTP fingerprint or response equivalence.
No new HTTP behavior was justified by this audit chunk.

## Current request-code branch checks

**Confirmed static:** checked current `RequestCodeRepository.A00` instructions
at 0998–09ae add `mistyped` only for a non-null source; 09c4–09da similarly gate
`hasav`. At 0a9e–0ac0, `push_code` requires a non-null, non-empty saved
`registration_push_notif_code`. Absence does not justify inventing these inputs.
At 0b2a–0b58, `feo2_query_status` comes from saved preference with literal
`did_not_query` fallback. That literal, and the false education/prefer-SMS
preference defaults, already exist in the Rust request template. This reanchors
those specific defaults to the current build; it does not prove all helper-added
fields, metrics provenance, runtime gates or wire headers equivalent. The two
native Kotlin/WAMSYS branches are visible; their selection gate remains untraced.

`Hfo.A01` classifies `autoconf` as method enum1 but is not an eligibility check.
No additional traffic follows from that classifier. The retained request-code
Java has extensive duplicate-control-flow warnings, so the branch statements
above were checked against DEX, rather than trusting rendered Java conditionals.

## Runtime parameter producers still missing from equivalence

**Confirmed static construction, server requirement unknown:** current
`RequestCodeRepository.A00` calls `Hfq.A0P` before transport selection. The
checked helper adds `network_radio_type` from the connection manager, `simnum`
from a SIM-related source-length test, `hasinrc` from an `rc2` file-existence
check, `pid` from Android process state, then calls `A0c` to append the APK release-channel enum as `rc`.
These values are absent from the default Rust template. They are runtime facts,
not constants supplied by an APK version. Exact source fallback meanings and the release enum numeric binding remain open; no device facts were synthesized.

`Hfq.A0S` adds `sim_type`, `airplane_mode_type`, `cellular_strength` and sometimes
`roaming_type` only behind property4435; current effective property value is
unknown. `Hfq.A0M` adds `cred_token` only behind property25565 and a non-null
number-keyed stored token. Neither helper justifies unconditional fields.

**High-priority unresolved producer:** `Hfq.A0U` invokes native JNI dispatcher7,
then dispatcher16 returning a map, and merges that map into request parameters.
The keys, values, provenance and native/runtime conditions are unknown here.
The Rust recipe does not establish equivalence to this native-generated map.
An empty-H envelope is not proof of satisfying any of these native prerequisites.
Trace that native producer and relevant gates before deciding what can honestly
be implemented in Rust. No claim that these omissions caused a ban follows.

Checked DEX was retained for all four helpers. The `A0S` Java rendering itself
shows suspicious boolean/int coercion; no scalar encoding was ported from it.
Additional complete current REA helper bodies are retained below. These are
static producer leads, not full downstream/gate equivalence; no field was added
or synthesized from this table.

| Helper | Producer and condition visible in current source |
| --- | --- |
| A0O | `sim_status` only with a saved cached-SIM-status string |
| A0R | Cached or calculated device RAM string, using DecimalFormat and Android memory source |
| A0T | `first_api_level` only with property39257 and a non-null platform lookup |
| A0X | `gpia` only through property3753 and a non-null runtime result; downstream producer and genuine prerequisite remain open |
| A0Q | Property7999; obfuscated setting and parameter names are calculated, unresolved here |
| A0Y | `fid` from a non-null provider model, combined with its timestamp-like member; exact provenance remains open |
| A0W | Preload manager ID and attribution each require a non-null provider result |
| A0Z | `tos_version=5` only when the eligibility predicate and positive saved state hold |
| A0V | `entrypoint` is conditional `create_paa` or `suma`, not a universal value |
| A0a | `old_phone_number` only with existing old-account metadata; request-code caller passes false and does not force a lookup |
| A0b | `recaptcha` only from a non-empty supplied result, never a generated placeholder |
| A0c | `rc` from RELEASE enum; numeric binding and applicability remain open |

Effective flags, provider data, encoding helpers and downstream native map are
still open. The field list cannot establish that omissions are ban triggers.

## Current APK autoconf lead and decompiler limitation

**Confirmed static, APK 2.26.43.72**:
[retained metadata, method and checked instructions](rea/2.26.43.72-registration-network-audit/README.md).
`AutoconfUseCase.A00` logs an already-sent autoconf response-verification step,
loads a contact point, and only with a non-null point calls
`VerifyCodeRepository.A03` with `autoconf` string arguments. With no contact point
it constructs a local error result. This is not an unconditional call proven
necessary before every SMS code request. The caller eligibility gate, HTTP route
inside the downstream repository and actual auth-response provenance remain
**unknown** in this trace. Do not synthesize that response or send an autoconf
request based on an endpoint name.

REA reports complete Java source, but this coroutine method contains duplicated
blocks, an apparently uninitialized local and impossible nested status tests.
Completeness of text is not semantic reliability. Checked DEX independently
confirms the contact-point branch and downstream call. No response enum or
success/failure branch was ported from the malformed Java rendering.

The exact-anchor DEX search `reg_onboard_abprop`, `pre_pn_client_log`, `client_log`,
`autoconf` returned eight `autoconf` references; it did not establish absence of
the other endpoints. Paths may be constructed elsewhere or encoded differently.
The broad URL-string scan was candidate discovery only: URL literals include
links and optional services, and are not a network request inventory.

Next: trace repository producers/callers and conditional identity fields, then
compare errors and persistence against the existing saved flow. No live account
request or additional automatic traffic was used.
