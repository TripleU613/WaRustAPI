# Retry original-chat correlation — 2026-11-05

**Confirmed static / bounded local correction:** pinned APK2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Checked original-key selection and context evidence](apk/2.26.43.72-retry-original-chat/README.md).
No live request or receipt was used.

F10.A04 constructs the original message key using a remote chat as well as the
nested retry.id. It selects a typed user recipient if present; otherwise it
normalizes context.A02 through0Dn.A00, which strips a DeviceJid to its userJid.
Its A01 alias helper then applies additional native policy. Full inbound context
registration and effective alias policy remain open; the retained candidate
ACK/NACK/message-context builders are not claimed as that missing factory.

Rust previously ignored recipient and compared only the requested device against
a global original-message-ID lookup. A different chat with a matching participant
or an explicit recipient naming another user could therefore enter session work
for that ID. The shared parser now makes **chat** the supported plain user-like
recipient when present, otherwise normalized from without its device suffix.
The **device** still comes from participant/from, independently of recipient.

A malformed, non-user or device-form explicit recipient refuses locally with a
redacted field diagnostic. Rust's supported-path refusal deliberately does not
reproduce every native typed-reader null/fallback case. Plain text recipient
syntax with a device/agent suffix is refused even when it spells zero, since
Rust can normalize that text into a plain JID. Full native UserJid grammar and
hosted/legacy mapping compatibility are not claimed.

The normal direct driver now checks selected chat against the original saved
peer or the existing peer→LID relation before accessing the Signal session,
preparing encryption or reserving a key query. A mismatch records local
**not_the_original_chat** and returns no send/query. It never creates a PN/LID
relation or adopts identity from the receipt. Original device, delivery, age,
count, carried-key and reservation guards remain in force. The existing mapping
direction is preserved; no universal reverse alias resolution is added.

Parser tests separate original chat from asking device, exercise normalized
from, valid explicit LID, invalid/group/device recipients and typed device0.
A driver regression uses the actual answer entrypoint with an otherwise eligible
original and verifies wrong chat/matching participant or wrong explicit recipient
produces only one unanswered checkpoint, preserving outbox/session/pin/source
records and creating no query job. The full saved/fresh retry workflow now uses
a pre-saved LID recipient for the PN original and still succeeds under the same
message ID/count, without deriving any mapping from a wire receipt.

Tests use synthetic store rows and decoded-node inputs; no native methods,
server-authenticated mapping, real socket or concurrent thread is executed.
Other shared-parser consumers (including the comparison group window) see the
selected original chat, and retain their own admission gates. This does not
complete status/group/addressing, alias-property, ACK or receipt-key policy.

Validation:1320 workspace/all-feature tests passed, along with strict workspace/
all-target/all-feature Clippy, formatting and CLI-reference regeneration. Selected
DEX mapping/JID exports reproduced; APK coverage and hygiene checked before push.
