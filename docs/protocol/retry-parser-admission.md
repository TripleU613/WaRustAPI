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
That does not complete group retry policy. Complete timestamp consumers, recipient/address
mapping, full current root normalization, receipt-carried key processing,
higher-count base-key state and all ACK/error consequences remain open.

Validation:1313 workspace/all-feature tests passed, as did strict workspace/
all-target/all-feature Clippy, formatting and CLI-reference regeneration.
Pinned count-reader exports reproduced; APK coverage and hygiene checked before
push. No live compatibility or account-restriction result was measured.

## Nested message key and timestamp correction (2026-11-05)

**Confirmed static / corrected offline:** the shared parser used root receipt id
as RetryRequest.message_id and ignored nested retry.id/t. Current F10.A04 instead
requires nested t/id and builds the original message key from nested id. Complete
[required readers and message-key construction](apk/2.26.43.72-retry-message-correlation/README.md)
close that selected correlation gap.

The parser now returns **nested retry.id** for original-send lookup. It requires
nested t to parse as a signed64-bit integer and applies existing local identifier
bounds to nested id. Root receipt id remains separately validated; no equality
between root/nested IDs is invented. Missing/malformed nested fields produce a
redacted field diagnostic and no actionable retry, before any outbox lookup,
query reservation, crypto or retry checkpoint mutation.

The timestamp is validated, not turned into a new retry/expiry clock. Native
F10 multiplies its parsed seconds by1000; complete downstream timing consumers
remain open. Rust still checks age against the persisted original first attempt.
The model/API adds no timestamp field, and full Java Unicode numeric syntax is
not claimed. Receipt ACK construction is unchanged and retains its own root
correlation semantics; all native parser/ACK exception effects remain unproven.

A parser regression covers independent root/nested IDs, missing/empty/oversized
nested id, missing/nonintegral/overflow t and signed64-bit boundary values.
The full saved/fresh retry workflow verifies root W1 + nested OTHER sends nothing
and leaves all retry tables unchanged. Missing/invalid nested fields do likewise.
Conversely, root ROOT-RECEIPT + nested W1 reaches the intended W1 resend with the
same original wire ID and remains once-only. Existing parser/version/count/key
fixtures now include valid id/t so they continue testing their intended fields.
The group comparison admission fixtures likewise use nested IDs. No live receipt,
key query, resend or ACK was sent.

Nested-field follow-up validation:1318 workspace/all-feature tests, strict
workspace/all-target/all-feature Clippy, formatting and CLI-reference regeneration
passed. Required-reader/key-construction exports reproduced; APK coverage and
repository hygiene checked before push.

## Original-chat selection follow-up (2026-11-05)

[Supported recipient selection and original-chat correlation](retry-original-chat.md)
now separate the failed message chat from the requesting device, using existing
saved peer/LID state. Full native context/alias/fallback/ACK parity remains open.
