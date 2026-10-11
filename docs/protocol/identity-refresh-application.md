# Changed primary classic identity-refresh application

2026-10-10 follow-up: [identity-auth admission correction](identity-auth.md#admission-correction--2026-10-10)
supersedes the identity_auth presence refusals described below. Structurally valid
metadata follows existing classic admission; malformed required fields defer.
Certificate/signature verification remains explicitly unported. Attestation/PQ
and all other application guards remain unchanged. Earlier validation below is
historical and does not verify this follow-up.

Follow-up: [changed companion directory application](companion-changed-key-refresh.md)
now verifies full ADV for a guarded existing companion against its independent
current primary baseline. First-seen, primary/raw-ID changes and full native
admission remain unfinished.


Follow-up: [known companion same-key refresh](companion-same-key-refresh.md)
now applies the native listed-device/equal-key fast return. Changed companion
identities and full ADV/admission remain separate and unfinished.


2026-10-09. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Implemented/offline:** the authenticated actor now applies a verified changed
primary classic key reply under `transport.identity_changes=apk_authenticated`.
It preserves the old identity/session and atomically commits a fresh session,
new pin, existing transition invalidations and an application marker. This is
possession of the new signed key, not human safety-number verification.

This continues [notification reads](identity-notification-key-read.md),
[original guards](identity-refresh-guards.md), [cryptographic preview](session-refresh-preview.md)
and [stored attestation/property tracing](attestation-state.md). Same-key,
companion, attestation/PQ/identity_auth admission and identity-notification ACK
remain unported. These are remaining client implementation gaps.

## Admission that does not require guessing properties

**Confirmed static:** [CPB.A00](apk/2.26.37.71-identity-refresh-application/primary-admission.txt)
returns Bjo when main property 0x6f6f is disabled (`0030 -> 00cc`). With it
enabled, absent CWS initially classifies E2EE (`02d4 -> 0054`). Comparison with
an existing different primary identity sets v12=1 at `00d2`. Absent CWS then
branches from `00f0` to `01e4`, selects E2EE and reaches `0222` (`01e8 + 0x1d*2`).
At `0226`, nonzero v12 branches to `0306` (`0226 + 0x70*2`), constructing Bjn.
This skips the later old-C0F/override/reject-downgrade gate. Main=false returns
continue-without-state; main=true returns continue-with-E2EE-state. Both admit
this candidate; they do not produce the same stored-state effect.

[14J.A04](apk/2.26.37.71-identity-refresh-application/admitted-bundle-worker.txt)
continues processing Bjo/Bjn, separates Bjp refusal, and invokes 0jc.A0L. Its
companion ADV policy remains separate. [Bot.A03](apk/2.26.37.71-identity-refresh-application/candidate-parser.txt)
produces CWS from selected-user non_e2ee_attestation and separately parses PQ
and identity_auth data. Rust defers **any presence** of those metadata children,
including malformed data. A valid signed key is not authority to skip them.

**Implemented:** the supported branch requires a correlated successful saved
action, complete singleton selected-device classic bundle, valid signed-key
signature, different held pin, primary device 0 and matching original notification
guard. Historical untracked reads cannot be adopted. The existing Pinned policy
remains refusal; no configuration is silently changed. The actor uses the receive
identity_changes policy because this read originates in the authenticated
receive path. Outgoing identity policy is still separate.

**Unknown / unported:** native attestation state and effective MobileConfig
assignments for this Rust account. No NOT_SET/E2EE value, migration completion,
override or feature assignment is invented. This implementation commits the
branch-invariant pin/session result; it does not claim optional native C0F
persistence or full CPB equivalence. Same-key candidates still need their actual
state/property admission, even if their signed key verifies.

## Changed-key session reset, not old-key ratchet reuse

**Confirmed static:** [0jc.A0L](apk/2.26.37.71-identity-refresh-application/session-processing.txt)
compares held session identity at `0044..008a`; a changed key reaches A0E
(`0096..009c`) or A0s (`00a2..00a8`) depending on the mode. Both call A0D:
[A0E](apk/2.26.37.71-identity-refresh-application/session-reset-transaction.txt)
wraps it in a transaction, while [A0s](apk/2.26.37.71-identity-refresh-application/session-reset-mode.txt)
adds mode-dependent address lock/monitor handling.
[A0D](apk/2.26.37.71-identity-refresh-application/session-reset-row.txt)
translates the address through 0kT.A02 and deletes the selected sessions row
(`0000..0072`), then schedules DNt discriminator 32 (`00aa..00d4`). Complete
address-translation and that callback's cache effects remain **unknown** here.
[CQc.A00](apk/2.26.37.71-identity-refresh-application/signed-bundle-derivation.txt)
verifies the signed key before fresh derivation and final session/identity saves.

**Implemented/offline:** after the separate admission checks, Rust invokes
prepare_session_refresh with no held record and the staged approved new identity.
The helper validates the signed key again and builds a fresh classic record.
No chat plaintext/ciphertext, outbox intent or new wire request is constructed.
The old bytes are preserved in the encrypted identity-history record, rather
than copied into the new identity's current/previous ratchets. Exact PN/LID
namespace is preserved; this does not port native counterpart address migration.

## Atomic application and recovery

Derivation happens outside the writer transaction. Commit rechecks:

- The exact correlated saved successful reply, without resetting its action.
- Original account generation, pin fingerprint, exact-device transition marker
  and nullable user primary version, through the shared writer/read guard evaluator.
- Exact current pin and serialized session bytes against the staging snapshot,
  so a concurrent receiving/sending ratchet update is not overwritten.
- The registered saved account's serialized fingerprint, including its local
  Signal key/registration and own address, without printing secret material.
- Current own-account PN/LID authority, including authenticated login aliases
  and legacy own-LID authority; malformed/conflicting authority refuses.

The existing archive_bundle_replacement hook invalidates peer verification,
primary device authority/version, sender-key distribution and applicable receipt
membership. Its journal event, encrypted old-state archive and exact transition
marker share the transaction with new pin/session and identity-refresh-applied
marker. Any failure rolls back all effects. These existing invalidations retain
their documented scope; this is not complete native primary receipt cleanup.

**Implemented:** a repeat of an applied action returns locally even after later
ratchet progress or identity changes. applied is a historical fact, not present
pin agreement. The actor calls application after IQ settlement; startup recovers
eligible settled replies after a cut between those commits. It never retries a
Sent/uncertain/refused IQ, backfills a missing original guard, or reuses a chat ID.
Original action/reply and notification guard remain immutable.

No identity-notification ACK is sent yet. A deferred application logs a fixed
reason (pinned policy, reply unverified, original guard not matching, unsupported
companion/same-key/metadata, own identity or changed reply/account/peer state).
Storage/crypto corruption remains a fixed redacted error rather than silent
success. Receive-only sessions can process identity reads and this application
without enabling outbox sends.

## Inspect and reproduce

```sh
whatsapp-cli --profile PROFILE actions --show ACTION_ID
whatsapp-cli --profile PROFILE actions --show ACTION_ID --preview-session
```

The existing offline --show now includes **applied**, in addition to signature,
attestation, current pin and original guard diagnostics. After a changed-key
application it normally reports applied=true, pin_matches=true and guard=pin_changed:
the original guard correctly describes the old pin. No peer/key/archive or raw
response is printed. Preview remains nonpersisting and does not invoke adoption.
For actual actor processing, use the existing configured listen command; only
explicit apk_authenticated receive policy enables this branch. No HTTP route
for application is claimed. OpenAPI's actions/listen contracts and inspection
schema describe the capability honestly.

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-identity-refresh-application.json \
  --output docs/protocol/apk/2.26.37.71-identity-refresh-application --check
target/debug/xtask apk-research-diff \
  docs/protocol/apk/2.26.37.71-identity-refresh-application \
  docs/protocol/apk/2.26.37.71-identity-refresh-application
```

## Offline evidence

**Implemented/offline:** generated signed-key fixtures cover primary PN/LID
adoption, exact encrypted old-pin/session archival, fresh record identity and
zero old-key archived ratchets, current pin/guard inspection, once-only repeats
after actual ratchet progress and reopening. Recovery applies one already-settled
reply without replaying its IQ; another recovery applies zero.

Refusal/defer fixtures compare all 14 encrypted tables plus journal head for
Pinned policy, same-key, forged, queued, uncertain, server-refused, companion,
present attestation/PQ/identity_auth, historical untracked and own PN/LID cases.
Staging/commit tests introduce actual ratchet progression, changed account,
changed reply status, A->B->A transitions and new own-LID alias. They cannot
commit the old staged candidate. A malformed sender-key distribution forces
failure after primary version invalidation; every table/head remains unchanged.
A fake authenticated actor applies the result with outbox disabled and no
identity ACK/chat write. These are generated offline fixtures, not a reproduced
live peer reset or full official-client cryptographic execution.

## Validation and retained-account receive

All **867** workspace/all-feature tests passed (726 API, 4 property, 9 crash,
54 CLI, 1 API-symbol and 73 xtask), plus strict Clippy, warning-denying Rustdoc,
25 default CLI tests plus its API-symbol check, CLI-reference regeneration,
API-symbol validation, formatting and whitespace. Repository hygiene checked
2,389 local links and 118 APK bundles / 25,217,294 bytes. The 8-topic / 13-file /
119,638-byte source bundle reproduces exactly and self-diffs unchanged.
This self-comparison does not establish equivalence with another APK release.

**Observed live:** one 45-second receive-only window (55.80 seconds including
authentication/shutdown) authenticated and stopped cleanly, with outbox sending,
automatic reads and reconnects disabled. All 21 action outcomes, 9 receipt rows
and zero eligible sends remained equal; generation, registration and journal
head stayed equal. Checkpoints increased by eight, all in the inbound category;
other table counts stayed equal. Eleven unsupported controls were processed,
with no decrypted user message. Counts are not proof of decrypted-byte equality.

The historical primary-LID signed bundle still verifies and matches the current
pin, has absent attestation, and remains untracked with applied=false. No new
qualifying identity-refresh reply or live changed-key application occurred. This
run checks the new actor/startup path against saved state, without claiming a
positive live peer-reset test or resending a message. Original private JSON,
keys, IDs and account history remain local. [Redacted aggregate evidence](identity-refresh-application-validation.json).

## Follow-up: notification ACK timing

The earlier ACK-unported boundary is superseded by the
[37.71 dispatch trace and actor implementation](identity-notification-ack.md).
The ACK precedes asynchronous key application; it is not adoption evidence.

## Server-authorized same-key continuation

[Same-key primary refresh](same-key-refresh.md) now applies the classic subset
with an authenticated selected-user force_override_e2ee_state child. This
supersedes the earlier blanket same-key application gap; no-child, companion
and unsupported metadata admission still remain deferred.

Follow-up: [identity_auth inspection](identity-auth.md) now exposes only metadata
shape, separate from the classic prekey signature result. Certificate policy and
identity-auth verification remain unported; present metadata still defers.
