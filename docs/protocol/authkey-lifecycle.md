# Auth-key lifecycle continuation

2026-11-04. **Confirmed static**, APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Complete retained evidence](rea/2.26.43.72-authkey-lifecycle/README.md).
This continues the [notification obligation](notification-sync-obligations.md#auth-key-rotation),
not a live rotation or a restriction workaround.

## What key is rotated

`1nq.A00` uses the selected `1oA("best")` curve provider to generate a private
key then derive its public half. `1nq.A02` delegates private/public serialization
to `1nr.A05`; this chunk does not independently specify that helper's layout.
`144.A0B` serializes the supplied pair and invokes password-backed persistence;
it returns false on runtime failure. `144.A06` attempts the underlying write
at most twice, then throws. These are local storage writes, not automatic IQ
retries. The exact password-encryption/Android-keystore storage formats remain
outside this portable Rust store.

`144.A0D` returns the synchronized active wrapper loaded by `144.A00`.
`1jv.A0C` uses that wrapper to construct the connection's `1o0` key input;
if the stored pair is absent it throws and postpones login. Thus the auth-key
rotation callback's active-pair replacement affects subsequent connections,
not merely a disconnected registration token. The independent checked DEX
confirms this load/absence branch.

`144.A0E` similarly returns the active pair or throws. Its four reported
using-nodes include companion registration, a command handler, a verification
helper and a JNI dispatcher. Inspecting the companion call confirmed that it
supplies the current public half as `companion_server_auth_key_pub`; this is
not the primary connection loader. The distinct `A0D` caller trace found
`1jv.A0C`. Reference totals count static using-nodes, not all dynamic/native uses.

## Replacement side effects

The previously retained `2IS.CK4` callback performs replacement only after the
IQ result. On successful local replacement it swaps the active wrapper, resets
`connection_lc`, then calls `145.A04`. The new `145.A04` trace shows conditional
Blacknoise Android-keystore certificate invalidation and resets its refresh and
alias metadata. It is not a request to change the E2EE Signal identity, reset
registration history or regenerate arbitrary account keys.

`144.A00` contains platform-keystore migration/verification/fallback branches
and provider-dependent controls. Complete Java carries duplicate-block warnings;
no entire fallback/recovery algorithm is inferred from its apparent fallthrough.
Only the getter/call-site relationship and explicitly tested storage boundaries
above are used here. No portable key-rotation code was justified by these bodies
alone.

## Still needed before a faithful rotation port

- Independently establish raw32-byte public encoding and pair serialization,
  curve-provider choice and active-key binding used by the actual handshake.
- Trace request manager331 timeout/delivery outcomes and every caller of the
  rotation manager, including the lifetime of its reused in-memory candidate.
- Determine current behavior after lost server results or process restart.
  Candidate-memory and success-callback bodies alone do not prove whether the
  server retains the old key or how acceptance can be reconciled. Persisting an
  unconfirmed candidate as active, or replaying an uncertain IQ, is unjustified.
- Integrate accepted replacement atomically with Rust account generation and
  existing registration material. They currently contain separate Noise copies;
  silently updating only one would cause a later material/account inconsistency.
- Preserve native attempt/cooldown policy and test stale callbacks, duplicate
  notification, failed store commit and crash boundaries. Keep existing Signal
  identity, prekeys and history intact.

These are specific remaining network/state obligations. They are not evidence
that a particular historical restriction was caused by missing rotation.

## Separate confirmed Rust preflight correction

Source inspection found `connection::login_material` reconstructed persisted
Noise halves without checking their relationship. `classic::ClassicFinish::finish`
sends the saved public half and computes DH with the saved private half; a
mismatch can reach the socket before cryptographic failure. This is a local
consistency defect, independent of any ban claim.

Preflight now derives the Curve25519 public half using the configured Snow
backend and compares it with the saved half before endpoint/DNS/TCP/Noise. It
refuses a mismatch with a fixed redacted error and never repairs/replaces keys.
The independent registration producer (wacore_libsignal) supplies a valid pair
in the regression, followed by mutated public, swapped private and zero-public
cases. All mismatches preserve account bytes, journal and absent login
reservation, including after reopening. The low-level unchecked `from_parts`
constructor remains unchanged; this guard is scoped to comparison login.

## Feature-boundary verification

The reduced Noise/transport check exposed a separate existing compile defect:
`presence::held_tctoken` referenced `inbox::lid_for` even with Signal crypto
disabled. That helper is now gated with its inbox dependency; all production
callers already require the same feature. It does not silently fall back to a
PN-only lookup. Full-client wire behavior is unchanged. The compiler feature
check and reduced-feature tests verify this boundary.
