# Retry receipt field admission — 2026-11-05

**Confirmed static / corrected offline:** pinned APK2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Checked DEX evidence](apk/2.26.43.72-retry-parser-admission/README.md).
No live receipt, resend, account or key query was used.

The shared Rust `receipts::parse_retry` previously treated a missing or malformed
registration as absent, ignored the retry version and required an explicit count.
The normal driver could interpret invalid registration as a session mismatch and
reserve a key query. It also ignored valid default-count receipts.

| Field | Selected APK behavior | Corrected Rust behavior |
| --- | --- | --- |
| registration | required binary content, exactly4 bytes; otherwise LIL | returns no actionable retry for missing/wrong type/length |
| retry v | default"1" when missing; another value returns null | missing or exact"1" accepted; other values refused |
| retry count | absent defaults1; present parses Java signed32-bit integer | absent1; present nonnegative signed32-bit value required |

The public RetryRequest.registration remains optional for source compatibility
with direct model construction, but a successful shared parse always provides
four bytes. Downstream supported-count gates remain separate; a parsed count0
or count>4 is not permission to answer it. Negative counts remain unrepresentable
in this Rust model and are refused. Full Java Unicode integer syntax parity is
not established. No new acceptance of higher retry counts was added.

Invalid registration/version/count produces a redacted debug diagnostic naming
the field, with no raw key or attribute value. `answer` returns before lookup or
mutation for these non-actionable inputs, so malformed registration cannot create
a fetching marker, pending refetch job or retry attempt. Unknown receipts can
still be captured by the surrounding runtime; receipt ACK policy is unchanged
and is not claimed equivalent to the native parser's exception handling here.

Parser tests cover missing/empty/wrong-size/wrong-content registration, default
and unsupported version, default/malformed/negative/overflow count and signed
upper boundary. The complete saved/fresh retry workflow also supplies invalid
registration/version/count and asserts unchanged outbox/checkpoint/session/pin
rows. Its first valid saved-session resend now omits version and count, reaches
count1 encryption and remains once-only when a later explicit count1 repeats.
These are synthetic offline tests, not handset/server compatibility proof.

The shared parser is used by the normal direct driver and the explicit CLI group
comparison window, so both now refuse these malformed fields and see defaults.
That does not complete group retry policy. Nested retry id/t, recipient/address
mapping, full current root normalization, receipt-carried key processing,
higher-count base-key state and all ACK/error consequences remain open.

Validation:1313 workspace/all-feature tests passed, as did strict workspace/
all-target/all-feature Clippy, formatting and CLI-reference regeneration.
Pinned count-reader exports reproduced; APK coverage and hygiene checked before
push. No live compatibility or account-restriction result was measured.
