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

## Current implementation boundary (reconciled 2026-11-06)

The following selected static traces are already available. Older dated notes
that leave these exact blocks open are superseded by these sources; this is
not a claim of runtime acceptance or complete Android parity.

| Requirement | Evidence available | Remaining implementation/evidence gap |
| --- | --- | --- |
| Candidate/public versus stored pair | [Candidate and layout](apk/2.26.43.72-authkey-candidate-storage/README.md) | Concrete Curve25519 backend/randomness and independent wire vectors |
| Active auth-key connection binding | Getter/connection trace above | Atomic Rust account/material generation update; current Noise copies are separate |
| Rotation request, gates and outcomes | [Rotation lifecycle](apk/2.26.43.72-authkey-rotation/README.md), [queue/timer/response outcomes](authkey-outcomes.md#queue-acceptance-is-not-server-acceptance) | Wire-to-sent/shutdown generation ownership and uncertainty reconciliation |
| Password-backed local replacement | [Checked commit/reload](apk/2.26.43.72-authkey-commit-reload/README.md), [concrete provider](apk/2.26.43.72-authkey-provider-binding/README.md) | Other keystore branches and platform crash durability; Android ciphertext format need not be ported |
| Direct rotation triggers and reused candidate | [Caller/field inventory](apk/2.26.43.72-authkey-rotation-triggers/README.md) | Durable candidate recovery; exact-field inventory is not global absence proof |
| Conditional token request and result | [Writer](apk/2.26.43.72-client-auth-token-writer/README.md), [correlated reply](apk/2.26.43.72-client-auth-token-reply/README.md) | Complete input authenticity, native JID/runtime behavior and durable token ownership |
| Token readiness/pending nonce | [Membership](apk/2.26.43.72-client-auth-token-membership/README.md), [nonce flow](apk/2.26.43.72-client-auth-token-nonce-flow/README.md), [failure distinctions](apk/2.26.43.72-client-auth-token-failure/README.md) | Cross-thread/generation ordering and durable recovery |
| Post-token held traffic | [Challenged-request ownership](apk/2.26.43.72-challenged-request-ownership/README.md) | Durable request-family ownership; selection is not permission to replay Rust reservations |

There is still no Rust rotation or client-auth-token maintenance flow in the
searched transport source. New state must retain the usable old key plus a
separate durably owned candidate before publication, distinguish not-sent,
uncertain, definite refusal and accepted-result/local-install outcomes, and
coordinate future login material. These are local design requirements, not
claims that the APK implements a durable journal with these names.

A faithful implementation must resolve what the server accepts after a lost
result/restart; current static callback and storage evidence does not answer
that. Do not promote an unconfirmed candidate or resend an uncertain key IQ.
Native attempt/cooldown distinctions must remain, with stale-callback,
duplicate-notification, failed-commit and crash regressions. Existing Signal
identity, prekeys and history are separate and must remain intact. This is an
unfinished protocol-maintenance lifecycle, not evidence about a past ban.

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

[Outcome/startup continuation](authkey-outcomes.md) now distinguishes local
queue acceptance, delivery timeout, definite error and result, and traces
conditional CAT/pending-token triggers. Lost-result reconciliation remains open.

## Registration-copy consistency before login (2026-11-06)

**Confirmed local defect / fixed offline:** a mathematically valid replacement
account Noise pair passed login_material even when the existing registration
record still named the original public key. The regression's valid baseline
passed; replacing both account halves with another generated pair then failed
its expected-refusal assertion before the fix. This differs from mismatched
private/public halves, which the earlier preflight already rejected.

With experimental-signal-crypto enabled, login_material now loads the account
and checks any present registration material in the same read snapshot through
the existing Material.check_account invariant. Present malformed or mismatched
material propagates an error before socket setup/reservation. Accounts without
that optional lab record keep their prior path; reduced builds without Signal
retain existing Noise pair validation. The guard neither repairs records nor
rotates keys. This is a local consistency rule, not an Android storage-layout
requirement or a completed rotation port.

The regression registration_noise_copy_mismatch_refuses_login_before_reservation
uses an independent registration key producer, a valid control, and a distinct
valid replacement pair. It checks no login checkpoint, unchanged account rows
and journal, and repeated refusal after reopening. No live operation. This does
not lock the snapshot through a future handshake or solve concurrent credential
replacement: generation ownership and accepted rotation remain separate work.

## Login reservation binds its preflight snapshot (2026-11-06)

**Confirmed local gap / fixed offline:** connect previously validated saved
credentials, awaited socket/server-handshake work, then reserved login by checking
only the prior login status. Account or registration-material replacement during
those awaits did not invalidate the stale key/payload. Reservation now compares
the current serialized account and, with Signal enabled, optional registration
record bytes against the same read snapshot used for preflight. The comparison
and outcome_unknown write share one transaction. A mismatch returns a fixed
redacted error with no checkpoint or state repair.

Snapshots are in-memory Zeroizing buffers, not persisted copies or diagnostics.
Any account-record change invalidates the snapshot conservatively, including
non-key metadata; the caller must inspect/reprepare rather than bypass the check.
Optional missing registration material and reduced Signal-free builds retain
their existing boundaries. The original Noise-half and present-material
consistency checks still apply before all network setup.

Regression login_reservation_refuses_account_or_material_replacement_after_preflight
prepares a valid snapshot, changes account state or removes material, and checks
refusal without a login checkpoint/journal mutation, including after reopening.
A fresh snapshot reserves once and a second reservation refuses. This models the
await interleaving directly without opening a socket or live account.

**Limits:** this reservation occurs after the initial server handshake but before
VerifiedHandshake.finish publishes the client login/key payload. It does not
prevent initial TCP/ephemeral traffic, mutations after reservation, concurrent
late replies or stale-result installation. Those require broader generation
ownership; this is not a completed rotation or reconnect lifecycle.

## Login reply owns its reservation and credentials (2026-11-06)

**Confirmed local gap / fixed offline:** the comparison connection's result
transaction previously replaced the login checkpoint and applied clock/identity
state without binding the reply to the attempt that sent the login. Reservation
now writes a fresh local ID. Production connect retains that ID and its prepared
credential snapshot through reply capture, and the result transaction requires
both the same account/material bytes and an outcome_unknown checkpoint with
that exact ID before any settlement/clock/account/identity write.

Changed credentials, a missing/replaced/settled checkpoint or duplicate reply
therefore cannot overwrite current state. The inbound reply capture still occurs
first. Refusal returns a fixed local error and leaves current uncertainty intact;
it does not repair keys, clear a refusal or authorize reconnect/replay. A matching
failure still commits its authentication code and returns Authentication, while a
matching success retains existing atomic account/clock/identity semantics.
The old unowned settlement wrapper is compiled only for isolated legacy unit
tests; the production connection calls the owned path.

Regression stale_login_reply_preserves_current_reservation_account_and_clock
covers success/failure across account change, material removal, replaced owner,
already-settled and missing checkpoint cases, plus valid controls. Refused writes
preserve account/checkpoint rows and journal, including after reopen. Valid
settlement rejects a duplicate. The ID is local ownership metadata, not a wire
IQ ID or server authentication claim.

**Scope:** this closes the actor connection's reservation-to-result write race,
not every credential mutation during a live session or an atomic network/store
transaction. The separate CLI comparison lab-transport login implementation is
not routed through this helper and remains a distinct audit obligation. Legacy
uncertain rows without an ownership ID remain blocked rather than upgraded.
No live operation or rotation implementation.
