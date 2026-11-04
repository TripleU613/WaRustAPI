# Media HTTP outcome persistence

2026-11-04. **Confirmed Rust defect/fixed offline:** ordinary media GET returned
a definite non-200 response but retained only the pre-send `outcome_unknown`
reservation. `transport/media.rs::open` now commits `server_refused` plus numeric
`http_status` before returning its error. A later invocation reports that stored
status and does not issue another request. Transport interruption, length or
authentication failures retain the existing uncertain reservation. No response
body, URL or arbitrary header is printed or added to refusal records.

The private transition requires an existing `outcome_unknown` reservation; it
cannot create an unsolicited outcome or overwrite a downloaded/settled record.
Storage/authentication failures propagate. Tests use a real encrypted store and
cover reopen, missing reservation, redirect/refusal/rate-limit/server-error status
retention and protection of other outcomes. No live media request was made.

**Confirmed local behavior:** the shared client disables redirects, automatic
retries, proxies, referer and decompression; requires HTTPS and authenticated
TLS; bounded streaming precedes ciphertext/MAC/plaintext verification. These are
source observations, not claims of native header/timing parity. View-once GET
already persisted numeric refusals in its separate state machine; this change
only affects ordinary media.

**Unknown/current-build audit queue:** conditional media headers, host selection,
expiry, native retry scheduling, account-specific transport choices and complete
2.26.43.72 download call flow remain untraced. Existing 2.26.37.71 media schema and
crypto evidence does not establish current-build HTTP behavior. No new APK
evidence, server acceptance or restriction-cause claim is introduced.

## View-once preflight (2026-11-04)

**Confirmed Rust defects/fixed offline:** `download_view_once` previously lacked
the ordinary media path's config validation and registered-account/phone match
check. It now performs them before inbox selection, cache return or reservation.
A descriptor's encrypted hash was checked before GET, but an absent/malformed
plaintext hash could still consume a one-shot download and later fail decryption.
Both hashes must now be 32 bytes before any network reservation.

Real-store async tests prove missing or mismatched account/config refuses without
checkpoint changes; invalid timeouts refuse before selection. Descriptor tests
cover absent, short and long plaintext hashes. These are local account and crypto
prerequisite checks, not recovered native account predicates or evidence of a ban
trigger. Native HTTP behavior remains in the current-build audit queue above.
No live account operation, cache deletion or automatic replay is introduced.

## Upload response-header persistence follow-up

**Confirmed source defect/fixed offline:** `image_upload::upload_prepared`
previously obtained HTTP status before bounded response-body streaming but wrote
`http_status` only after the body finished. Interruption or a body exceeding the
local 64-KiB bound lost an already-observed status behind outcome_unknown.
It now persists `response_headers_received` with that numeric status before
body streaming. The transition requires the original outcome_unknown reservation
and preserves encrypted upload material and binding fields. Existing settled
outcomes cannot be overwritten. A body interruption or local size refusal leaves
this header observation durable, without implying upload success or permitting
replay. Complete bodies still move through response_received before validation;
only a fully validated result can become uploaded.

Real-store tests cover 200/302/403/429/503, missing reservation, reopen with no
completed body, binding preservation and replay refusal after header receipt.
They also protect completed/refused rows against overwrite. No interrupted upload
or other live network request was run.

## Explicit auth lifetime validation (2026-11-04)

**Confirmed Rust defect/fixed offline:** upload route selection parsed `auth_ttl`
with `and_then(...).unwrap_or(ttl)`, treating present invalid or overflowing
auth lifetimes as absent. It now falls back to route TTL only when the attribute
is absent. Present malformed/non-text auth TTL refuses. Zero, negative and expired
lifetimes continue to refuse through the existing age/minimum-lifetime guard.

A real-store encoded-route test covers absent and valid lifetimes, empty/malformed/
overflow values, zero/negative values and auth expiry shorter than route expiry.
Route inspection leaves every checkpoint unchanged and performs no refresh or
upload reservation. This was the then-existing local fallback, not a native-parity claim. The
subsequent pinned callback trace below **refutes** that absence fallback; the
current implementation now requires both lifetimes. Native expiry parsing/refresh policy
remains in the current-build audit queue. No live traffic occurred.

## Pinned native lifetime correction (2026-11-04)

**Confirmed static and corrected:** current `1xY.CK4 -> 0Tp.A07/A0L/A09`
requires both ttl and auth_ttl and parses each with Java Long.parseLong.
The previous missing-only fallback was a local assumption and is now removed.
Rust reuses the existing Java-compatible signed-decimal helper for both fields.
The encoded-route regression now refuses missing auth_ttl and admits supported
leading-plus/BMP digit spellings while preserving expiry/invalid-value guards.

`1t5.<init>` converts lifetimes to milliseconds relative to a supplied base;
`0VM.A08` conditionally restores a cached route only while its auth deadline
is future. [Six REA envelopes, checked DEX, hashes and exact limits](rea/2.26.43.72-media-route-lifetimes/README.md).
Base origin, full refresh/deadline consumers, effective restore gate and host
selection remain unknown. No automatic query/retry or live traffic is introduced.
