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
