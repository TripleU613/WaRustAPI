# ADR 0004: Noise backend candidate `snow` (Rust resolver, `std` off)

Status: accepted as experimental · 2026-09-17

## Findings

- `snow` 0.10.0 feature `std` expands to `ring/std`, which activates the
  optional `ring` dependency. With `default-features = false` and
  `default-resolver-crypto` only, the graph contains no `ring` (verified by
  `cargo tree -i ring`: no match) and `Noise_XX_25519_AESGCM_SHA256` builds
  and completes a handshake against a snow responder (unit test).
- `snow` rejects records above 65,535 bytes. The web references use a 3-byte
  frame length (max 16 MiB). The adapter returns `RecordTooLarge` instead of
  fragmenting; whether WhatsApp ever emits Noise records above 64 KiB is
  `Unknown` and must be observed before this backend is accepted.
- Pattern, prologue (`WA\x06<dict>`), trust root and certificate structure
  are `ReferenceOnly` (web multi-device). The references also use an IK
  resume pattern with a cached server static key; not implemented.
- Certificate signatures in the references are XEdDSA (Signal-style), so
  chain verification depends on the Signal backend decision (ADR 0005). The
  transport module takes a `SignatureVerifier` and ships no accept-all path.

## Decision

Compile the adapter only behind `experimental-noise`. Every connection epoch
performs a fresh handshake; transport state is never persisted; any failure
closes the transport permanently.
