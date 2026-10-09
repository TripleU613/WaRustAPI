# Own-account prerequisite for local verification

**Confirmed local, 2026-10-09:** `transport::peer_verification::set_local`
previously accepted the saved login identity `lid` as a known own account.
`model::jid::Jid::parse` intentionally accepts server-only destinations; that
value parses with an empty user and zero agent/device. The prerequisite checked
server, agent and device but omitted the user. With an otherwise valid remote
pin, a mark could therefore succeed without enough information to exclude self.
This is our local guard defect, not an observed WhatsApp server response.

The prerequisite now also requires a nonempty user, inside the existing write
transaction and before verification metadata is written. Missing own user,
nonzero own device or nonzero own agent returns the existing diagnostic:
`saved local LID is unknown; cannot exclude own account`. No identity, session,
login record or account identifier is repaired or replaced by this command.

**Refuted hypothesis:** textual zero-device forms do not bypass the existing
own-address comparison. The Rust parser uses `user[.agent][:device]@server`;
`user:0@lid` and `user.0:0@lid` already parse as plain primary addresses.
An initial underscore fixture tested a different literal user, not an agent.
No additional normalization or parser change was needed or retained.

## Evidence and reproduction

The isolated regression `unknown_own_user_refuses_decisions_even_with_valid_remote_pin`
failed before the fix: a server-only saved own LID allowed a valid remote mark,
producing verified=true and action sequence 1. It now refuses both mark and
unmark; reopening encrypted storage preserves the unchanged state. Companion and nonzero-agent
saved own identities are negative controls. The test checks unchanged checkpoint,
peer-pin and Signal-session rows, unchanged event head and absent verification
metadata.

`own_primary_wire_encodings_refuse_decisions_without_mutating_state` checks
plain, explicit zero-device and explicit zero-agent/device own identities.
Each refuses both decisions without changing rows or events, including after
reopening. A distinct remote positive control can still mark and unmark with
the expected pin and sequence. All addresses and keys are synthetic fixtures.

Run `cargo test --locked -p whatsapp-api --all-features peer_verification`.
The existing `peer-keys` and `peer-verification` CLI commands remain the inspection
and explicit human decision surfaces; this adds no command or automatic decision.
The OpenAPI description records a CLI-only, non-live capability.

## APK boundary

**Confirmed static, comparison APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
`LX/Cg3.A00` rejects self at 006c..008a and rechecks the rendered identity under
its address lock at 0130..0164. `LX/0jc.A0K` preserves the action sequence while
invalidating a verified bit on key change. The detailed traces and companion
fan-out boundaries are in [peer identity evidence](peer-identity.md).

**Unknown:** native parity for an empty saved own user. This fix enforces our
existing known-own-account contract; the APK trace does not establish the same
malformed-login branch. No real account was marked/unmarked, no verification
message was transmitted, and no fresh live identity transition was exercised.
A receive-only window would not exercise this explicitly local setter.
