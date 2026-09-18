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

### `bad_param` investigation: an untraced additional-parameter path

`bad_param` establishes rejection of request parameters, not whether a field
was missing, malformed or incompatible with the selected branch. No raw bodies
from the two retained `bad_param` responses are available, so a possible
offending-field hint cannot be recovered retrospectively.

The next static trace found a concrete gap in our coverage:
`RequestCodeRepository.A00` at 0x0bec calls `I4D.A0H`, applies more map
augmenters through 0x0c3a, adds `feo2_query_status` at 0x0c74, and passes
additional parameters through `generateAuthCodeBlocking$1`. The bridge calls
`Hvq.A06`, whose byte-array map values are encoded by `HNM.A00` and entered
in a pre-encoded set. Our plain-text extension fields alone did not implement
this path's exact binary encoding.

| Producer | Added fields | Evidence / unresolved inputs |
| --- | --- | --- |
| `I4D.A0H`, 0x0014–0x006c | `mistyped`, `reason`, `hasav`, `client_metrics` | `mistyped`/`hasav` conditional; reason and serialized metrics inserted on this helper path. Their caller/runtime values are not inferred. |
| `I4D.A0K`, 0x0012–0x007c | `mcc`, `mnc`, `sim_mcc`, `sim_mnc` | Strings from two carrier-info objects, converted to bytes; repository reads network and SIM operators. No carrier is inferred from a phone number. |
| `I4D.A0H`, 0x0082–0x00e0 | `education_screen_displayed`, `prefer_sms_over_flash` | Stored booleans, default false in these preference reads. Existence is not evidence of server-requiredness. |
| `I4D.A0H`, 0x00fc–0x016a | `push_code`, `new_acc_uuid` | Added only for non-empty stored values. Do not fabricate them. |
| `RequestCodeRepository.A00`, 0x0c50–0x0c7e | `feo2_query_status` | Stored string with fallback `did_not_query`, encoded to bytes. |
| `I4D.A0r`, 0x010c–0x0116 | `recaptcha` | Conditional challenge-related data from another component. Not implemented or fabricated. |

`Hvq.A01` inserts a non-null text value; `A02` omits only null, not empty
strings. `A00` encodes internal integer 0 as `false`, 1 as `true`, and omits
other states. `HBS` defines PHONE=0 and WA_TWO_FA_CONTACT_POINT=1; bridge
`A0S` only adds `type` when its enum argument is non-null. No universal
`type=0` default or fabricated permission claim has been added.

Implemented offline: `RegistrationForm::push_bytes` and private profile
`fields.code_bytes/register_bytes/security_bytes`, using RFC 3986 encoding
exactly once. Text and byte maps cannot collide or replace protected fields.
Space is `%20`, tilde stays `~`, and star becomes `%2A` on the byte path;
ordinary Java-style form text instead uses `+`, `%7E`, and `*` respectively.
Raw binary vectors, duplicate/protected fields, and factory wiring have tests.
The existing sent requests had empty extra maps: their body bytes have not
been retroactively changed, and this is not a demonstrated live fix.

Future response diagnostics recognize a string `param` hint only when its
value matches a known request-field name. Unknown strings and unexpected types
are ignored without losing the main status/reason. This is defensive support
for a possible diagnostic extension, **not** a claim that either captured
response included such a hint. The hint is diagnostic output, not a newly
persisted registration-state field. No new live request was sent in this work.

Reproduce the added trace with `xtask apk-smali <apk> --class 'LX/I4D;'`
and `--method A0H`, `A0K`, or `A0r`; inspect `LX/Hvq;` methods A00/A01/A02/A06
and `LX/HBS;` separately. All names refer only to comparison build 2.26.37.70.

### Additional-parameter producers (comparison build)

`LX/1GF;->A00` parses the complete operator string using the class initializer's
`(\d{3})(\d{2,3})` regex. It preserves the three-digit MCC and formats the MNC
as three decimal digits with `Locale.US`. Null/nonmatching input takes the
`("000", "000")` fallback at 0x00b2–0x00bc. Thus a two-digit MNC is padded;
the fallback is an observed no-operator representation, not a carrier to guess
from a phone number. `RequestCodeRepository.A00` obtains network/SIM operator
inputs from telephony APIs before `I4D.A0K` emits their byte-valued fields.

`LX/HcR;->A01` emits a JSON `attempts` property from its constructor-supplied
integer, plus `app_campaign_download_source` only when the supplied string is
nonnull. The class does not establish a default attempt count. One caller,
`LX/GyW;->A0Z`, increments the persisted `reg_attempts_check_exist` counter
before construction (0x0018–0x0038); this is an existence-check path, **not**
evidence for the `/v2/code` counter. `LX/H2s;->A00` forwards its caller's integer
and a source string obtained through `08l.A0L -> 0gI.A04`. Trace that caller
before choosing any signup metrics value. Neither telemetry value was added
to a live profile.

Nearby calls are not metrics producers: `I4D.A0u` reads or generates a persisted
backup token; `I4D.A0v` reads or generates a persisted identifier through `00L`.
Their proximity to additional-map assembly is not evidence to substitute them
for `reason` or `client_metrics`. No live request was sent for these findings.

Further tracing identifies the code-specific producer: `I4A.A0E` reads
`reg_attempts_generate_code` with default zero (0x0010–0x001c), increments and
persists it (0x001e–0x0028), then constructs `H2r` using the inherited `HcR`
constructor (0x0038–0x0042). A fresh counter therefore supplies 1, not 0.
This is not permission to reset a previously used attempt counter.
`H2r.A01` additionally emits nullable `is_sim_number`, `is_sim_absent`,
`is_permission_granted`, and `isUserChoosingToMigrateFromConsumerAppDirectly`.
These are conditional runtime inputs, not constant claims to add to the profile.

The interactive APK reader now shows move/result register operands (compact
snapshot output is unchanged). This resolves `reason`: the normal code branch
loads `I4A.A00` at repository A00 offset 0x0a4c, passes it as v25 at 0x0be0,
and `I4D.A0H` consumes it as v8 at 0x0022. `I4A.A0R` writes this static value
and persists it via `0DL.A0Z`. `RegisterPhone.A5j` resets it to the empty string
at 0x001c–0x0026; later UI/error paths change it. Thus `reason` is stored
flow state, not the `enter_number` screen label in repository A02. The empty
initial value is distinct from omission, but is not proven correct for the
already-refused lab attempt. No device state or counter was reset.

The local profile preview lists these observed additional fields as absent,
text-path, or byte-path, without exposing values or claiming server requirements.

### Registration transport continuation

Following the actual bridge resolves the shared transport chain:
`KotlinRegistrationBridge.A09 -> RetryingHttpClient.A01/A00 -> HsU.A01 ->
128.A07 -> 128.A0E` (virtual dispatch). `HsU.A01` at 0x0068–0x0088
sets form content type and encoded byte length; our sender already does this.
The earlier zero exact-call result for `129.A0E` was a superclass-dispatch
limitation, not evidence that registration bypasses the shared HTTP helper.

`HsU.A01` calls `G6W.A0v`, which reads `128.A02` and calls `0re.A02`.
That method uses the literal comparison version and delegates to `0re.A00`.
The producer reads Android OS release, manufacturer and model and sanitizes
them before constructing the User-Agent. This establishes an observed header
presence mismatch: our private test configs did not specify a User-Agent.
It does **not** establish the reason for refusal. No device properties were
invented or populated to disguise the Rust client as a real Android device.

The alternate body-builder branch of `RetryingHttpClient.A00` logs separate
`isEncrypted` and `sendAttestation` flags, can add `Authorization` at 0x05a2,
and sets form headers at 0x05be–0x05de before `128.A07` at 0x0642.
Our sender implements only the clear form. Runtime selection, authentic
attestation inputs and server requirements are unresolved. No integrity
claims, proxy/domain-fronting strategy or retry behavior was reproduced.

### Encryption selector resolved for the Kotlin code-request path

Further APK-only inspection narrows the earlier uncertainty: bridge A09 calls
`RetryingHttpClient.A01`, which sets v7 to **true** at 0x0000 and passes it as
the `isEncrypted` argument to A00 at 0x0034. This particular path requests
encryption, not an unknown runtime flag. `HRT.<clinit>` maps `HSP.A06` (code)
to `HgX(true)` at 0x004e–0x0062; A01 reads that value for `sendAttestation`.
This does not prove which of the repository's alternate transports is selected.

The envelope itself is statically reproducible:

- A00 0x0398–0x03e2 generates a fresh Curve25519 pair (`B2G.A01`) and agrees
  with the 32-byte public key in `HQQ.<clinit>`; `B2G.A0A -> 1NH.A02` returns
  the agreement bytes directly, with no intervening KDF.
- 0x03ea–0x0432 uses AES/GCM/NoPadding, a newly zero-filled 12-byte nonce,
  a 128-bit authentication tag and no AAD. The key pair must be fresh for
  every seal; the persisted account keys are not encryption nonce substitutes.
- 0x0438–0x0452 removes the public-key type byte, prefixes the raw 32-byte
  ephemeral public key to ciphertext plus tag, and encodes using Android
  Base64 flags 11 (URL-safe, no wrap, no padding).
- `HpO.A00` XORs characters with 0x12: the strings backing `HRz.A02/A03`
  decode to `ENC` and `H`. Encryption-only wrapping is `ENC=<base64url>`.

Implemented `EncodedRegistrationRequest::encrypt_comparison` as explicit
offline opt-in behind experimental Signal crypto. Existing CLI/factory requests
are **not** silently changed. Tests cover peer decryption, tag tampering,
fresh ephemeral keys, noncontributory peers, outer encoding and redaction.
They are local compatibility tests, not evidence of server acceptance.

Attestation is separate: A00 0x0488 onward signs the encoded payload via
`13h.A07`, adds `H`, and may obtain an Authorization payload via `13h.A03`.
Failure paths can leave an empty H and omit Authorization; their existence is
not proof that the server accepts that fallback. No H or attestation token is
manufactured, and no live request was sent in this implementation.

### Subsequent authorized ENC-only experiment

One explicit request reused the latest saved account identity and unchanged
612-byte inner form, wrapped in an 884-byte `ENC` body. The response remained
HTTP 200 / fail / bad_param (58 bytes, 186 ms), with no recognized `param` hint
or retry/SMS/voice wait. This does not identify the offending parameter or
prove the server decrypted the envelope. No H/Authorization data was supplied.
The result and a dedicated one-shot reservation are persisted; no second send
occurred. The new CLI diagnostic is separate from ordinary begin and from the
earlier unchanged-payload diagnostic, whose reservation remains intact.

### Follow-up experiments and first changed refusal — 2026-09-18 UTC

APK-only findings justified four further explicit tests on the same saved
attempt and keys. No proxy, identity rotation, reset, fake certificate or
fabricated attestation signature was used. All responses were HTTP 200/fail;
none included a recognized parameter hint or retry/SMS/voice wait.

| Change | Outer body bytes | Response | Response bytes / elapsed |
| --- | ---: | --- | --- |
| Append observed missing-signature fallback `H=` to ENC | 887 | `bad_param` | 58 / 222 ms |
| Add four `000` carrier fields, two false preference defaults and `feo2_query_status=did_not_query` | 1065 | `bad_param` | 58 / 100 ms |
| Also add post-failure reason and metrics with preparation count 6 | 1167 | `bad_param` | 58 / 111 ms |
| Also set APK unknown-build-properties User-Agent; preparation count advances to 7 | 1167 | `format_wrong` | 64 / 189 ms |

The reason is `server-send-request-bad-param`: `VerifyPhoneNumber.A5P`
0x0f6a–0x0f74 passes it to A2B, which persists it via I4A.A0R at
0x003c–0x0042. Metrics count the actual local preparations; the CLI checks the
value before reserving the next request. Counts were not reset to fresh-install
values. Field values live in private JSON patches, not sender hardcoding.

`13h.A07` returns null when disabled or signing fails/missing key; A03 can
return null when disabled/retrieval fails. Body builder A00 then uses an empty
H and omits Authorization. This reproduces absence, not successful attestation.

`0re.A00` catches unavailable build-property reads and substitutes `unknown`
for release/manufacturer/model (0x00d0–0x0124). Its final assembly gives
`WhatsApp/2.26.37.70 Android/unknown Device/unknown-unknown`. This is the explicit
header tested, not an invented model. The changed response coincided with this
header and the next metrics count; it does not isolate causality or prove
successful decryption, genuine Android execution, number validity or signup.

The APK's interpretation of `format_wrong` is now traced end-to-end:

- `KotlinRegistrationBridge.A09` 0x04c4–0x04d8 maps it to `02S.A0M`;
  `02S.<clinit>` assigns that constant integer 29 (0x023c–0x0248).
- `VerifyPhoneNumber.A5P` dispatches case 29 to `A19` in both the voice/flash
  switch (0x0152 → 0x0196) and general request switch (0x0d8e → 0x0dce).
- `A19` logs `VerifyPhoneNumber/onFormatWrongError`, persists the next reason
  `format-wrong` through `I4A.A0R`, and requests dialog 53 through `HmE.A01`.
- `onCreateDialog` case 53 (0x0062 → 0x0318) formats the phone and country,
  loading string resource `0x7f123741` at 0x0372. The default string recovered
  directly from this APK's `resources.arsc` is:
  `%1$s\nis not a valid mobile number for the country %2$s.`

This establishes the official client's user-facing interpretation, not the
server's internal validation rule or independent proof that a number is
unassigned. The bridge's separate country-code/national-number fields agree
with the current request assembly; no supported digit or country correction
was found. Current saved state retains the refusal, and bad-param-only
diagnostic commands no longer accept this state. No new live request was sent
during this analysis. Repeating unchanged input or inventing a different
country would not test a demonstrated protocol correction.

Diagnostics also now recognize the APK's `param="number"` alias (A2B compares
it at 0x0042–0x0050), plus ENC/H field names. Earlier logs could suppress an
unrecognized hint; the new flow-state and header tests still reported none.

### Registration-token salt correction

The later JSON-configured test on another supplied number returned `bad_token`.
Rechecking `HQP.<clinit>` against our token derivation found a transcription
error at zero-based salt byte 62: the code used `0xfc`, while Base64-decoding
the APK's constant gives `0xf8`. The other 89 bytes match. This changes the
PBKDF2 key and every resulting HMAC token, regardless of phone number.

The implementation now matches the APK constant. A regression test compares
the salt's Base64 representation to the APK text, and the synthetic derivation
fixture was independently recalculated with Python `hashlib`/`hmac`. The APK's
`AvQ.A0v` still confirms ordinary padded Base64 with Android flag 2 (`NO_WRAP`);
no output-encoding change was justified. This fixes a verified local bug but
does not by itself prove server acceptance.

Live verification on the existing `bad_token` attempt changed only the token
in the inner request, preserving account keys and previous fields. The server
returned HTTP 200 / `sent` (347-byte response, 3219 ms) to the 1123-byte ENC/H
request, with retry/SMS wait 125 seconds and voice wait 65 seconds. The state
now awaits the SMS code. No code has been submitted and handset receipt is
unverified; this is not yet completed signup or messaging.

### Refusal information loss fixed

Previously `ComparisonHttpFlow::blocked_response` dropped the parsed protocol
reason and stored HTTP status only for statuses >=400. Therefore old
`ServerRefused { code: None }` records cannot identify why signup was refused.
`Blocked` is our local terminal state, not proof that Meta banned the number;
the original protocol reason was discarded, so those are distinct claims.
New records preserve actual HTTP status and the recognized reason enum;
unknown server text becomes `Other`. Old records still deserialize, and an
already blocked attempt remains blocked without another network request.
Tests cover HTTP 200 plus `fail/bad_token`, HTTP 403 plus an unknown reason,
serialization compatibility, secret exclusion, and no resend after resuming.
These are synthetic tests, not improved live server responses.

### First retained live refusal — 2026-09-18 UTC

Following operator confirmation of number ownership, one explicit legacy
diagnostic used the same saved profile, attempt and keys. `/v2/code` returned
HTTP 200 with `status=fail` and `reason=bad_param` (58-byte response). The
recognized fields contained no retry/SMS/voice wait. This is a concrete
parameter-level refusal, not proof of a banned number or successful signup.
The available diagnostics do not identify which field failed. The current
User-Agent and body-mode gaps are still hypotheses, not established causes.
The result is now durably recorded. No further request was made.

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
