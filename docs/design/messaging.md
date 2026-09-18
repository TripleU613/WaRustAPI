# Domain model, encryption and public API

[Design index](README.md) · [Current status](../../STATUS.md)

Design requirements from the implementation plan revised 2026-09-17.
These describe intended behavior; see current status for implemented capabilities.
Original section numbers are retained for source and evidence references.

## 9. Domain model, addresses and device identity

Use newtypes for `AccountId`, `AccountGeneration`, `ChatId`, `ContactId`, `DeviceAddress`, `MessageId`, `ClientRequestId`, `EventId` and `EventCursor`. Avoid APIs where all of these are interchangeable strings.

A contact identifier is not automatically a phone number, and a cryptographic session belongs to an addressed device, not just a human-readable contact. Preserve observed phone-number JIDs, logical IDs, device suffixes and group IDs as distinct representations. Resolve aliases only from authenticated protocol evidence. Never merge identities based on display name or guessed number equality.

Meta's published multi-device architecture describes per-device identity keys, device discovery, direct-message fanout and group sender keys. This is useful architectural evidence, not a current Android wire specification. Our design consequence is that “we are primary” does not eliminate recipient device handling. [Meta multi-device architecture](https://engineering.fb.com/2021/07/14/security/whatsapp-multi-device/)

Required device behavior:

- Fetch and cache authenticated device lists with an explicit version/expiry policy.
- Invalidate them on observed device-change notifications and applicable server errors.
- Key sessions and peer identity decisions by full device address and account generation.
- Handle device additions/removals and races between preparation and transmission.
- Implement recipient fanout and any required own-device copies from evidence.
- Keep own-companion enrollment a separate capability; do not pair a phone to satisfy initial registration.

Use composite logical message keys including local account generation, chat and sender/direction components as required by the protocol. Do not assume message IDs are globally unique.

Represent message content as a tagged domain enum, including an explicit `Unsupported` form. Keep raw bounded payloads where needed for later decoding; do not assume a protobuf decode/re-encode preserves unknown fields byte for byte.

Edits, reactions and revocations refer to an original message identity. They must not create unrelated messages because the original is absent locally. Persist pending mutations/tombstones and reconcile when the referenced message arrives. Preserve server timestamps separately from local observation time.

## 10. Signal and sender-key integration

### 10.1 Backend selection gate

Place all backend-specific APIs and serialization behind `src/signal/`. The public library must expose neither backend protobuf types nor backend storage traits.

Compare the reusable Rust WhatsApp implementation and a compatible pinned official Signal revision. Do not assume newer Signal client features are required or accepted by WhatsApp. Do not downgrade a library merely until it stops rejecting inputs; establish the actual expected version and its security properties.

The backend must pass independently established vectors for:

1. Identity and signed-prekey encodings/signatures, registration ID ranges and one-time prekey handling.
2. Initiator/responder session setup and the actual prekey/message envelope variants.
3. Bidirectional ratchet advancement, out-of-order messages, skipped-message limits and duplicate rejection.
4. Persist/reload of all relevant session forms.
5. Sender-key distribution, group encryption/decryption, sender devices and key rotation.
6. Authentication failures, invalid signatures, identity replacement, exhausted/missing prekeys and malformed records.
7. Interoperability with the pinned Android app, beyond backend-to-itself round trips.

Do not implement cryptographic primitives from scratch. If no candidate passes, document the incompatibility and scope a specific compatibility adapter or maintained fork. Do not hide wire incompatibility behind a generic trait and declare the integration complete.

### 10.2 Secret state and trust

Signal session records include material needed to continue encryption/decryption. Their serialization is **not storage encryption**. The same applies to prekey and sender-key records. Encrypt every secret-bearing record before it reaches the database.

Store peer identity decisions as well as our own identity. First-contact trust and subsequent identity changes are distinct events. Define an explicit default policy: accept an initial identity according to the verified protocol trust model; on an unexpected replacement, emit `IdentityChanged` and suspend affected sends until the configured policy resolves it. Do not silently overwrite trusted keys on every fetch.

Persist prekeys before upload, record ambiguous upload outcomes, and reconcile without losing private keys. Use `(kind, id)` keys so signed and one-time identifiers cannot collide. Retain old signed prekeys for an observed grace period; do not delete them immediately upon rotation. One-time prekey consumption must commit atomically with the inbound session mutation it enables.

### 10.3 Sender keys

Index sender-key records by group, full sender-device identity and the observed distribution/key identifier. A group ID plus human contact ID is insufficient.

Track which participant devices need the current sender-key distribution. Distribute through the required pairwise encrypted channels before or with group traffic according to the observed protocol. Handle membership changes, removed devices, stale metadata, retries and out-of-order distribution messages. Do not assume the server distributes sender keys for us.

The account actor serializes crypto operations in v0.1. Optimize concurrency only after proving ordering and transactional correctness; two concurrent sends must never both consume the same pre-mutation session snapshot.

## 14. Public API contract

Keep a cloneable `WhatsAppClient` handle and a separate `SyncEngine`. Exact Rust signatures must be adapted to the inspected workspace conventions; the following behavioral contract is mandatory.

| Operation | Required behavior |
| --- | --- |
| `WhatsAppClient::builder()` | Accept validated profile, account store, key source, HTTP backend, limits and policies; no implicit registration |
| `registration().begin(...)` | Start/resume a durable primary-registration attempt and return its current step |
| `registration().submit(...)` | Submit a typed answer for a specific step; return next step or completed registration |
| `registration().reconcile()` | Resolve an ambiguous request through the profile's observed read-safe flow without blindly repeating the mutation |
| `connect()` | Idempotently start one runtime and wait for authenticated readiness, with timeout/cancellation |
| `connection_state()` | Watch current status, reason and connection epoch; no durable-message guarantee implied |
| `disconnect()` | Stop reconnecting, shut down owned tasks and preserve resumable state; no account logout |
| `send_message(SendRequest)` | Durably enqueue and return a stable `SendTicket` |
| `message_status(ticket)` | Retrieve current aggregate and per-recipient delivery facts |
| `send_receipt(Receipt)` | Queue an explicit valid receipt under configured privacy policy |
| `groups(PageRequest)` | Return a bounded page and continuation, not an unbounded list |
| `create_group(CreateGroupRequest)` | Track request identity and ambiguous outcomes; do not blindly recreate on timeout |
| `upload_media(source, options)` | Bounded staged upload of caller-provided encoded content |
| `download_media(reference, sink)` | Enforce limits and release only verified content to the final sink |
| `events(consumer, cursor, limits)` | Replay durable domain events with explicit acknowledgement |
| `capabilities()` | Return support levels and tested profile information |

`request_otp` and `verify_otp` may remain convenience methods for a profile's supported SMS flow. They must delegate to the registration state machine and surface additional steps rather than pretending every flow ends after OTP submission.

A compatible event model must cover:

```rust
// API sketch: associated domain types are defined in src/model/.
pub struct EventEnvelope {
    pub account_id: AccountId,
    pub account_generation: AccountGeneration,
    pub event_id: EventId,
    pub cursor: EventCursor,
    pub event: SyncEvent,
}

#[non_exhaustive]
pub enum SyncEvent {
    NewMessage(Message),
    MessageUpdated(MessageUpdate),
    MessageDeleted(MessageKey),
    ReceiptUpdated(Receipt),
    SendStatusChanged(SendStatusUpdate),
    GroupCreated(Group),
    GroupUpdated(GroupUpdate),
    IdentityChanged(IdentityChange),
    HistoryGap(HistoryGap),
    UnsupportedMessage(UnsupportedMessage),
    SessionRevoked(RevocationReason),
}
```

Use a separate live connection-state watch for `Connecting`, `Ready`, `Backoff`, `Disconnected`, `NeedsRegistration`, `Revoked` and `Stopped`. If the existing bridge expects connection changes inside `SyncEvent`, map them in its compatibility adapter with clear live-event semantics.

An old callback interface may be an adapter over durable events. Prefer an async fallible handler or explicit batch acknowledgement; a synchronous `Fn(SyncEvent)` cannot establish that a consumer durably processed an event.

Typed errors must distinguish invalid input, unsupported capability/profile, registration action required, rate limit with deadline, authentication failure, revoked session, identity change, invalid certificate, malformed protocol, storage failure, wrong storage key, exhausted resources and uncertain delivery. Preserve useful sanitized server codes without leaking payloads.

Reconnect only for classified recoverable failures, with jittered capped backoff. A revoked account, invalid certificate, corrupted store, required registration action or incompatible profile must not enter an endless reconnect loop.

## 15. Groups and message operations

Create groups and change membership through durable operation records. Group creation is a non-idempotent server action unless evidence proves otherwise. After a lost response, reconcile through an observed request/group identity or report an uncertain outcome; do not create duplicate groups automatically.

Before sensitive group actions, validate the latest known membership/admin state. Treat server results as authoritative and update local state transactionally. Handle pagination and large memberships without unbounded allocation.

For group notifications, verify the available authenticated sender/context and revision semantics. Deduplicate them and tolerate out-of-order arrival. Reconcile stale membership before distributing new sender keys.

Replies preserve the original chat/message/sender reference. Reactions track add/change/remove semantics. Edits and revocations validate the acting identity and reference; they must not mutate another sender's message merely because a supplied ID matches. Separate local deletion from a protocol request to delete for others.

Preserve disappearing-message timers and view-once indicators in the domain model. Apply verified expiry rules to local bodies and media, while retaining only the minimal bounded tombstones required for correctness. Do not invent ordinary downloadable content when a message type is unsupported. Bridge support for these features requires a separate capability decision.

## 16. Media without hidden native dependencies

Media references carry more than a URL: retain the observed encryption key material, hashes/MAC information, length, type, expiry/reupload information and any required binding to the message. Encrypt these references at rest and redact signed URLs and keys in logs.

Implement exactly the observed media construction. It may differ from both Noise encryption and at-rest encryption. Add RustCrypto AES/CBC/HMAC/HKDF components only if verified wire behavior needs them. Do not replace a legacy media cipher construction with a preferred modern cipher and expect interoperability.

Use streaming I/O with configurable byte/time/concurrency limits. Never trust `Content-Length` as the only limit. Hash and authenticate what was actually read.

When full-message authentication is required, stage encrypted content to a bounded temporary file, verify it, and only then expose plaintext through a final destination. If partial plaintext staging is unavoidable, use protected staging and do not publish it before final verification. Failed/cancelled downloads clean up staging state.

For uploads, generate and persist per-media secrets and resumable job metadata before the dependent message send. Verify the completed upload response before constructing a sendable reference. Handle expired URLs/reupload only through observed supported flows.

Apply a dedicated media HTTP policy: validated HTTPS, constrained redirects, no forwarding of registration/auth headers to arbitrary hosts, and rejection of loopback/private/link-local destinations unless explicitly required by a local test configuration. Treat remote filenames as labels, never filesystem paths.

Accept already-encoded image/audio/video/document bytes in v0.1. No FFmpeg, native codecs, ImageMagick, or shell subprocesses. Generate optional thumbnails only using reviewed Rust libraries; clearly report unavailable optional previews. File transport does not require implementing a video encoder.

## 17. Bridge compatibility without coupling the library to Telegram

Inspect the existing bridge before selecting adapter names. Preserve its high-level sync/event pattern, but add the durability and capability information it actually needs. Use a small adapter over the two real implementations; do not introduce a speculative universal messaging framework.

The adapter must preserve source account/chat/message identity, source revision, event ID, reply relationships and outgoing client request IDs. Persist destination mappings and pending forwarding work transactionally in the bridge's own store, then acknowledge the library event. Prevent loops by origin identity, not by comparing message text.

Use source events to create destination work; do not make Telegram network calls from the WhatsApp socket reader. Handle rate limits, partial forwarding and uncertain destination responses explicitly. The bridge must not promise cross-platform exactly-once delivery.

The bridge is an endpoint that can read messages in order to relay them. WhatsApp E2EE does not extend automatically through that endpoint into the destination platform. Reflect this boundary and destination retention in product documentation. Capability differences for edits, disappearing messages, group roles and view-once content must be visible rather than silently flattened.

A standalone registration/messaging CLI is the proof harness. Telegram availability must not determine whether the WhatsApp library itself works.
