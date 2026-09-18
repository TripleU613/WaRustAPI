# Outbound HTTP research — 2.26.37.70

Static analysis started 2026-09-18 against local `WhatsApp.apk`, SHA-256
`b2090b0207e9b97449a5b48e98eb4d1649ae8e5e738e9b7431d6242717fee732`.
This is the comparison build, not pinned 2.26.36.75. No network requests were
sent for this investigation. Static client behavior is not proof that a server
requires a field, accepts a fingerprint, or receives every possible request.

## Initial inventory

The [reproducible cross-reference inventory](http-post-xrefs.txt) contains
39 literal `POST` references, 24 `User-Agent` references and two literal
`v.whatsapp.net` references. Counts are instruction references, not distinct
URLs. Dynamic methods, native code, assembled strings and socket traffic are
not fully covered. This is a starting inventory, not a completed all-traffic audit.

| Inspected path | Static evidence | Meaning for signup |
| --- | --- | --- |
| Shared HTTP helper `LX/129;->A0E` | URL supplied by caller; `User-Agent` set at 0x00a2; conditional POST at 0x0120; JSON default content type at 0x0136; optional gzip request/response headers | Generic transport, not a registration body schema. Do not copy its JSON default into the form-encoded registration sender. |
| `LX/IN5;->ACJ` | `gdrive-api/begin-transaction`, `clients/wa/backups/`, `:beginTransaction`; JSON options `mode`, `estimatedDeltaBytes`, `axolotlMode`, `retryCount` | Backup API path, not evidence of a Meta signup prerequisite. Host resolution still needs tracing. |
| `KotlinRegistrationBridge->A04` / `A05` | Client funnel / pre-phone-number funnel builders; `current_screen`, `previous_screen`, `action_taken`; A05 also has `event_name`, language/locale and session identifiers | Client telemetry exists. Mandatory ordering and whether signup depends on it remain unknown. Do not invent or replay activity. |
| `KotlinRegistrationBridge->A08` | Register builder includes `code`, conditional `auth_response`, `context`, `method`, `advertising_id`; uses `HSP.A0A` | Registration completion differs from requesting a code. Response-parser keys later in this method are not outgoing fields. |
| `KotlinRegistrationBridge->A09` | `token` at 0x00b6, `method` at 0x00c4, optional `context` at 0x00d2; branch-dependent permission/education fields at 0x00f0–0x010e; optional `client_start_message` and `advertising_id`; `HSP.A06` passed to `RetryingHttpClient.A01` | Code-request builder. Profile extension maps can represent these values, but their runtime producers and mandatory subset remain unproven. No automatic retries copied. |
| `KotlinRegistrationBridge->A0T/A0U/A0V` | Shared phone, language/locale, fdid/expid, access-session ID and recovery/backup token appenders | Already represented by the editable profile and persisted prerequisites. |
| Port-80 chat fallback | Existing inspected `POST /chat`, chunked body, `c.whatsapp.net`; see comparison fixture | Socket transport fallback, not the HTTPS signup form. |

Exact call search for `LX/129;->A0E` returned zero direct callers. This does
not imply dead code: dispatch through superclass/interface must be traced.
The remaining POST references are explicitly unclassified in the inventory,
including generic Tigon/TEE paths. Class names alone do not establish payloads,
destinations, purpose, mandatory ordering or server requirements.

## Profile update rule

For each proposed field, record endpoint, APK class/method/offset, producer,
encoding, conditionality, and whether an authorized runtime observation exists.
Keep unknown values absent rather than manufacturing a device model, carrier,
advertising identity, integrity result or a supposed successful fingerprint.
Real OTP/PIN input remains ephemeral. Preserve server cooldowns and blocked states.

Next: trace registration's actual shared transport and User-Agent producer;
classify each remaining POST call site; distinguish request assembly from
response parsing; then add offline byte-level fixtures for supported fields.
The current signup sender still has no demonstrated server acceptance.

## Reproduce selected inspections

```sh
target/debug/xtask apk-xref ~/Downloads/WhatsApp.apk \
  --anchor POST --anchor User-Agent --anchor v.whatsapp.net
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/129;' --method A0E
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/IN5;' --method ACJ
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'KotlinRegistrationBridge;' --method A04
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'KotlinRegistrationBridge;' --method A05
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'KotlinRegistrationBridge;' --method A08
```
