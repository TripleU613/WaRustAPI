# Protocol evidence

[Design index](README.md) · [Current status](../../STATUS.md)

Design requirements from the implementation plan revised 2026-09-17.
These describe intended behavior; see current status for implemented capabilities.
Original section numbers are retained for source and evidence references.

## 6. Evidence is part of the implementation

### 6.1 Pin the reference and record server observations

Maintain a machine-readable reference manifest containing:

- Android application package, version name/code, release source, signing-certificate fingerprint, and APK/split hashes.
- Research device OS/architecture and relevant configuration, without real account identifiers.
- Observation date and whether the server accepted the reference client during that observation.
- Relevant Rust/reference implementation revisions.
- Token dictionary hash, schema hash, supported framing/profile identifiers, and associated test fixtures.

A pinned APK is authoritative for that client's behavior. It is **not** proof that the current server still accepts that behavior. Track client findings and dated live-server observations separately.

Use four evidence labels: `Observed`, `Derived`, `ReferenceOnly`, `Unknown`. Every important wire constant must point to evidence. A known-good third-party web implementation starts as `ReferenceOnly` for the Android primary profile.

### 6.2 Findings needed for the first proof

| Area | Questions to resolve |
| --- | --- |
| Registration | Endpoints, payload encoding, prerequisites, key-generation timing, verification steps, account challenges, returned credential material |
| Login | Client payload, primary-device markers, version fields, required startup requests, resume behavior |
| Transport | Endpoints, direct stream versus WebSocket, any TLS layer, preamble, frame lengths, compression, maximum records |
| Noise | Pattern, algorithms, prologue, handshake payloads, trust anchors, certificate encoding and static-key binding |
| Binary nodes | Token pages, packed strings, JID encodings, duplicate attributes, empty/content representations |
| E2EE | Session/message versions, key encodings, signatures, prekey upload/fetch, padding/wrappers, retry semantics |
| Devices | Address types, device-list discovery and invalidation, own/remote device distinctions |
| Messaging | ID format, message envelope, receipt/ACK classes, request correlation and error stanzas |

OTP receipt by itself proves neither registration nor authenticated messaging.

### 6.3 Fixtures

Fixtures must include independent known-good examples, not just bytes produced by our own encoder. Test decoding against expected semantics and encoding against independently established valid forms. Do not require byte-for-byte equality where multiple encodings are valid unless the profile requires canonicalization.

Keep real keys, phone numbers, message contents, OTPs, registration responses, and capture secrets out of committed artifacts. Sanitization must preserve or regenerate authentication consistently; changing bytes inside an authenticated capture generally invalidates it. Use synthetic test identities for redistributable cryptographic vectors. Keep any necessary private lab evidence separate.
