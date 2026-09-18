# Scope and milestones

[Design index](README.md) · [Current status](../../STATUS.md)

Design requirements from the implementation plan revised 2026-09-17.
These describe intended behavior; see current status for implemented capabilities.
Original section numbers are retained for source and evidence references.

## 0. The actual objective

Build `crates/whatsapp-api`: a standalone async Rust library that registers and operates as a WhatsApp **primary device**, using a phone number the operator controls. Normal operation must require neither a paired phone nor another WhatsApp client process. Preserve the `WhatsAppClient` / `SyncEngine` / `SyncEvent` separation so the existing bridge can adopt the library through an adapter.

The long-term ambition is broad Android-client protocol coverage. The first release must make a smaller, explicit set of capabilities work reliably. “Everything the Android app does” is an eventual product ambition, not an implementable v0.1 acceptance criterion.

**First decisive result:** complete a primary registration, authenticate, exchange an encrypted text message with the official Android app, terminate the Rust process, and reconnect using only the persisted Rust-owned state. Complete this before building the full groups/media/bridge surface.

This document replaces the supplied design. It specifies engineering decisions and the evidence needed to resolve protocol-dependent decisions. It does not turn unverified protocol guesses into requirements.

### What was and was not inspected

This revision is based on the supplied excerpt and the public upstream material linked below. No repository checkout, pinned Android APK, packet captures, or account credentials were supplied. The exact existing `tzibbur-api` interfaces and encryption implementation were therefore not inspected. Compatibility with them must be established during repository reconnaissance. No WhatsApp interoperability experiment was performed while writing this plan.

### Main changes

| Original assumption | Revised decision |
| --- | --- |
| Implement the whole architecture, then discover whether registration works | Prove primary registration and a minimal encrypted round trip before expanding features |
| Use the newest official Signal library | Evaluate exact wire compatibility, state format, build dependencies, and licensing before selecting a backend |
| Reimplement every layer | Evaluate reusable Rust protocol components first; verify them against the Android profile |
| `rusqlite` satisfies pure Rust | Use `redb` for the new default store; SQLite's implementation is C |
| Selecting rustls automatically establishes pure Rust | Audit its selected cryptography provider and the complete enabled dependency graph |
| Noise static identity and Signal identity can be treated as the same key | Model separate key types and lifecycles; only implement a relationship if evidence establishes it |
| A serialized Signal record is already safe to store | Treat session/prekey/sender-key serialization as secret plaintext and encrypt it at rest |
| Session file plus independent message database is sufficient | Put ratchet mutations, minimal inbox/outbox state, and durable events in one transactional protocol database |
| `send_message` returning an ID means the message was sent | Returning a send ticket means the message is durably queued; expose subsequent delivery states |
| An in-memory callback is a reliable sync interface | Use a durable event journal, bounded delivery, replay, and explicit consumer acknowledgement |
| Primary device means only one destination device matters | Resolve recipients' devices and implement the observed per-device encryption/fanout rules |
| Media download can return an unbounded `Vec<u8>` | Stream into a bounded, integrity-verified sink |
| A server can reconstruct all old chat history | Expose observed catch-up capabilities and history gaps; do not promise unavailable history |

## 1. Scope and release boundaries

The primary-device requirement is fixed throughout all phases. QR pairing, phone-number *linking codes*, a WhatsApp Web session, and a hosted WhatsApp gateway do not satisfy primary phone-number registration.

| Capability | First proof | v0.1 release | Later coverage |
| --- | --- | --- | --- |
| Phone-number registration and supported verification steps | Required | Required | Additional server-supported flows |
| Persistent login and reconnect without fresh registration | Required | Required | Additional device/profile migrations |
| Direct text send/receive | Required | Required | Richer message types |
| Recipient device discovery and direct-message fanout | Minimum needed for proof | Required | Advanced own-companion management |
| Receipts, duplicate handling, retry/recovery | Basic observability | Required | Additional receipt classes |
| Groups: list, create, metadata, participants, text | No | Required | Communities, advanced moderation |
| Images, documents, audio, video as existing encoded files | No | Required | Optional encoding/transcoding |
| Replies, reactions, edits, revocations | No | Required for verified message forms | New forms as discovered |
| Local persistence, event replay, crash recovery | Minimum durable state | Required | Additional storage implementations |
| Profiles, contacts, presence, privacy settings | Only required bootstrap pieces | Explicit capability flags | Incremental expansion |
| Voice/video calls, payments, channels, status, backup import | No | Not advertised | Separately scoped projects |
| Telegram bridge | No | Adapter contract and integration proof | Full product integration separately |

Expose a capability matrix containing `Supported`, `Experimental`, and `Unsupported`, plus the tested protocol profile. An unsupported operation returns a typed error; it never silently reports success or drops its unsupported fields.

Do not promise calendar completion dates before the primary-device experiment. Registration acceptance and protocol drift are external dependencies, not ordinary implementation tasks.

## 4. Order of execution and acceptance gates

| Gate | Work | Evidence required to pass |
| --- | --- | --- |
| G0 — Repository and dependencies | Inspect local instructions and existing crate APIs; choose initial target; evaluate Rust components, storage, TLS, Signal candidates | Repository map, dependency decision records, current build baseline, explicit unresolved blockers |
| G1 — Minimal protocol evidence | Pin Android reference and build only enough registration, framing, nodes, and crypto to attempt one conversation | Hashed reference manifest, fixture provenance, minimal working replay harness; no invented wire constants |
| G2 — Primary-device proof | Register one controlled test number, authenticate, exchange encrypted direct text, stop and restart | Official peer receives and sends valid messages; Rust reconnects from its own state without QR, companion app, or fresh OTP |
| G3 — Durable messaging | Implement transactional crypto state, outbox/inbox, event replay, receipt handling, disconnect/reconnect and resource limits | Crash matrix and replay tests pass; ambiguous delivery is represented honestly |
| G4 — Devices and groups | Complete recipient device lifecycle, sender-key distribution, group operations and membership changes | Verified behavior with multiple recipient devices and group membership changes |
| G5 — Media and message operations | Streaming media upload/download; replies, reactions, edits, revocations, expiry handling | Official-client round trips and adversarial integrity/resource-limit tests |
| G6 — Integration and release | Bridge adapter, documentation, packaging, feature/target checks, operational runbook | Standalone crate packages cleanly; advertised capabilities have evidence; bridge consumes/replays events correctly |

G2 is a **vertical slice**, not a requirement to finish every lower layer first. Implement the minimum real pieces needed to make it decisive. An authenticated socket alone does not pass G2. A message exchanged using imported linked-device credentials does not pass it either.

G0's production TLS gate may remain unresolved during offline work or a labeled experimental lab spike. It must be resolved before production readiness is claimed.

If primary registration is blocked, produce a concrete blocker report containing the request stage, sanitized response class, reference profile, attempted supported flow, and evidence still needed. Keep offline codec/store work useful. Do not rename a linked-device implementation to make the milestone appear complete.

## 21. Exact starting instructions for the implementation agent

1. Read applicable repository instructions and this document. Inspect `crates/tzibbur-api`, especially session storage, sync/events, errors and bridge adapters; record actual compatibility requirements.
2. Run the existing relevant build/tests and record the baseline without changing unrelated code.
3. Resolve the reference APK/profile and create the evidence/unknowns ledger. Do not invent version strings or wire constants.
4. Evaluate reusable Rust codec/Signal components, the strict TLS dependency gate and `redb` integration. Record precise decisions and blockers.
5. Build the smallest Rust CLI/replay harness needed for registration, authenticated transport and one encrypted direct conversation.
6. Attempt G2 on a controlled account only when its dependencies and required operator inputs are available. Record genuine success or the specific blocker.
7. Make the messaging path transactional and pass the crash matrix before broadening the API.
8. Complete device lifecycle and groups, then media and message mutations, following the gate order.
9. Add the existing bridge adapter only after the standalone path works. Preserve source identity and durable replay semantics.
10. Package, document and report capabilities with evidence. Never report stubbed, mocked-only or untested live operations as working.

## 22. Definition of done

The standalone v0.1 goal is complete only when all of the following are true:

- The Rust implementation completes a verified primary-device registration without a linked phone/client dependency.
- It authenticates, sends and receives verified direct and group messages with official clients.
- It maintains recipient-device and group sender-key state correctly for the tested scenarios.
- It resumes after clean and abrupt restarts without fresh registration, silent message loss from local transaction gaps, or stale-state reuse.
- Queued messages and delivery uncertainty are inspectable; consumer events replay with documented at-least-once semantics.
- Advertised media, reply, reaction, edit and revocation operations interoperate and enforce resource/integrity limits.
- Secret-bearing state is encrypted at rest, the master key is separate, and migrations/backup limitations are tested and documented.
- The release graph meets the pure-Rust contract, including its selected TLS, Signal, code-generation and storage paths.
- The crate packages and runs independently of the existing bridge workspace.
- Every advertised capability maps to dated interoperability evidence and relevant regression tests.

If a required item remains blocked, the result is an experimental implementation with an explicit blocker list. A polished API and passing mock tests do not substitute for the primary-device proof.
