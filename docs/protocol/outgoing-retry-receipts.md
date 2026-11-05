# Outgoing retry receipts — 2026-11-05

**Confirmed static / timestamp correction:** APK2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Retained parser/envelope/job/writer chain, exact fields and limits](apk/2.26.43.72-outgoing-retry-timestamp/README.md).
No live traffic or APK execution occurred.

Rust's automatic undecryptable-message retry previously set nested retry.t to
local queue-creation time. Native selected FVv/Ewu message paths preserve the
original root t through Dyc/Dyb seconds, Edc/F1Z milliseconds,
SendRetryReceiptJob.timestamp andEYI.A06; F3e.A08 divides by1000 for retry/@t.
The normal queue now uses the original message's seconds. Missing/unusable
values are refused with a redacted field/reason and no queue write, rather
than being replaced with the local clock. Values that cannot safely convert
to milliseconds are a local unsupported boundary, not a claimed native error.

The public retry builder's explicit seconds argument is now named
message_seconds; its wire field meaning is documented. Registration/counter,
addressing and reservations are unchanged. Valid redelivery with a different
timestamp cannot reset an already-held logical retry request.

A new queue/reopen regression verifies exact original seconds, invalid-input
no-write behavior and no replay. The real normal receive-loop direct/group
write-cut test asserts the emitted original timestamp and retains the existing
uncertain reservation across a failed write/restart. Older synthetic retry
fixtures now include a usable timestamp; malformed-input cases remain explicit.

Native retry/@error is also confirmed unconditional inF3e.A08, but its numeric
producer classification is unported. It must not be guessed from generic
Undecryptable or confused with receipt ACK error487. Conditional extra keys,
retry-count progression, full native time admission/ranges and special routing
remain open. This is not a complete outgoing retry receipt port.

Validation:1327 workspace/all-feature tests passed, strict workspace/all-target/
all-feature Clippy and formatting passed, CLI reference regenerated, and APK
coverage plus research checksum/link hygiene passed. Offline verification only.
