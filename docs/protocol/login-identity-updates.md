# Partial authenticated login identity updates

2026-10-09. **Confirmed local bug/fix:** `connection::connect` replaced the
whole `login-identity` checkpoint whenever either success attribute `pn` or
`lid` parsed. A PN-only reply therefore erased an already known LID; a LID-only
reply erased the stored PN. Present malformed values were also treated as absent.
This is our persistence behavior, not an observed server requirement or refusal.

**Implemented/offline tested:** login-result persistence now updates only
supplied identity fields in the existing write transaction. Absent fields and
unrelated checkpoint members remain unchanged. With neither field supplied,
the checkpoint is not rewritten. Newly supplied users must have the expected
PN/LID namespace, a nonempty user, agent zero and plain/AD encoding. Device-bearing
forms normalize to their user, consistent with the existing own-LID lookup.
A newly supplied PN can still initialize a missing AccountRecord.own_jid;
known account PN is not replaced by this patch. No identity is invented.

Malformed present fields return fixed redacted diagnostics naming `success pn`
or `success lid`. An existing nonobject record or malformed nonnull saved member
refuses a requested identity update rather than being silently repaired. This
validation does not run when neither identity field is supplied. A valid newly
supplied value retains the prior replacement behavior; this patch is not a new
policy for legitimate server-provided own-LID changes.

The login checkpoint, authenticated time and confirmed clock sample share the
same transaction. A refused update rolls all of them back, preserving the prior
`outcome_unknown` reservation. The real connect path already preserves the raw
authenticated reply before that transaction. It returns the fixed local error;
that error is not recoverable by automatic reconnection. No reservation is reset,
no login payload is replayed, and no account/Signal/Noise key rotates.

The checkpoint `at` records the latest identity-field update, not a new observation
time for every retained field. No per-field provenance or historical migration
is claimed. Missing historical fields remain missing; malformed history requires
inspection rather than automatic repair.

## Evidence boundary and reproduction

**Observed previously:** successful authenticated sessions carried `pn` and `lid`;
see [transport evidence](messaging-transport.md). This change does not claim a
new live partial success or malformed server response.

**Confirmed static context:** APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
[stored own-identity evidence](own-identity.md): 08i.A01 keeps a typed PN from Me,
while A02 stores a typed LID user/device separately and its shown other-input
branches do not replace those fields. This context supports distinguishing own
namespaces; it does **not** establish identical native login partial-update,
exception or malformed-history handling. Those are **unknown**. No new APK source
was discovered or native requirement inferred in this implementation.

The failing-before partial-success regression showed `(Some(PN), None)` after
PN-only success despite the earlier saved LID. It now covers PN-only, LID-only
with device encoding, absent-both input, own-LID inspection and encrypted reopen.
Negative tests cover empty/server-only/wrong-namespace/agent-bearing/nontext
input, malformed saved objects/members, unchanged account/checkpoint/pin/session/
outbox rows and event head, and preserved uncertain login status after reopen.
All data is synthetic; no real account or network operation ran.

```sh
cargo test -p whatsapp-api --all-features login_identity
```

Existing `listen` performs this login persistence; `peer-keys`, `peer-identity`
and `clock-inspect` remain redacted offline inspection paths. No new command,
flag or HTTP route is supplied. Fresh live partial-login behavior and complete
native own-identity lifecycle remain **unverified/unported**.

Validation: 840 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, formatting and whitespace checks passed. Repository hygiene
checked 2,182 local links and 108 existing APK bundles. No fresh live test.

## Bounded live regression (2026-10-09)

**Observed live:** a newly built experimental CLI from commit `051861b` used
the original registered account and key for one receive-only session. The private
request copy differed only in reconnect max_attempts=0 and mark_read=false;
`--send-outbox` was absent. Original settings and key stayed in private storage.
No queued actions or unacknowledged receipt records were present before starting.
The session authenticated, completed its requested **45-second receive window**,
and stopped cleanly with exit 0. Total process time including connection was
53.52 seconds. No additional receive run or automatic reconnect was launched.

[Redacted aggregate evidence](live-login-receive-validation.json) compares fresh
pre/post inspections: zero eligible outbox intents, 20 actions (17 ok/one error/
two sent) and nine acknowledged receipt records are equal after
JSON decoding. Event journal head stayed **20**. Inbound checkpoint count grew
804→813; total checkpoints grew 1,045→1,055, including one new confirmed-clock
record. Clock inspection now reports login_success/saved_offset and the existing
separate inferred sample. This is a recognised live clock source, not proof of
every native clock producer or offset's accuracy.

Twelve Unsupported log lines were six ib, four notification/psa, one
notification/encrypt and one notification/mex. Repeated captures may reuse rows;
these line/checkpoint counts do not represent user replies. No new supported
message/update/receipt or qualifying peer identity transition was observed.
A raw control classification is not by itself proof that control handling failed.

**Limits:** normal authenticated login exercises the changed production
persistence path, but no fresh partial or malformed success was observed; those
cases retain their offline validation. Pre/post queue and event equality does
not establish that every account, pin or session value remained identical;
authentication/login/clock state can update normally. Private detailed logs and
inspection snapshots remain ignored with 0600 permissions. No handset delivery,
read receipt, scan verification or complete native identity parity is claimed.

Live-check validation: the experimental CLI build, CLI-reference regeneration,
API/library-symbol checks, formatting, whitespace and repository hygiene passed
(2,186 links/108 APK bundles). One bounded live receive run completed; no fresh
workspace runtime suite was rerun for this evidence-only change. The preceding
840-test implementation validation remains separate.

## CLI shares the authenticated identity update (2026-11-06)

**Confirmed local gap / fixed offline:** CLI lab-login separately parsed lid with
an Option chain. A present malformed value became None, so its success row and
authenticated account timestamp could commit as if the field were omitted. It
also recorded only the legacy group LID checkpoint, bypassing canonical PN/LID
partial-update handling. The legacy writer already rejected invalid parsed LID
domains; the defect was not a universal absence of LID validation.

connection.update_login_identity is now a public transaction helper requiring a
success root. Its existing supplied-field/history rules remain. CLI invokes it
inside the owned result transaction, initializes a missing account own_jid from
a validated PN, and updates its legacy group LID from the validated canonical
checkpoint only when LID was supplied. PN-only success preserves the known LID;
malformed supplied fields roll back result/account/identity writes. Earlier reply
capture remains durable. Failure output does not claim a supplied LID was recorded.
The helper does not authenticate its input or authorize login; callers retain
capture and reservation checks. This is local persistence consistency, not a new
native malformed-success policy claim.

Regression cli_login_identity_preserves_partial_fields_and_refuses_malformed_success_atomically
covers LID-with-device normalization, PN-only continuation, canonical/legacy LID
agreement, malformed/wrong-domain LID, empty PN and non-success root. Negative
cases preserve account/checkpoint rows and journal with fixed diagnostics; the
valid LID survives reopening. Existing API identity corruption/partial-update
controls still exercise the same helper. No live operation or key rotation.

## Pinned initial reader audit (2026-11-07)

**Confirmed static, bounded:** 2.26.43.72 `0ZJ.A13` reads success lid using
`0Tp.A0A(Jid.class, "lid")`, normalizes DeviceJid for comparison and conditionally
passes the original JID to `09F.ChR`. Its getter logs protocol noncompliance and
continues to conversion; that check alone is not a rejection gate. No pn literal
read occurs in this selected method or its two retained direct success helpers.
The string parser, typed attribute producers, effective setter flag/implementation
and full own-identity lifecycle remain unresolved. Existing Rust namespace and
partial-update rules remain a local contract; do not infer stricter numeric
login grammar from ACK-specific rules. The native initial reader separately
throws for unexpected nodes rather than treating each as a failure stanza.
See [complete methods, negative selectors and reproduction](apk/2.26.43.72-login-success-reader/README.md).
No runtime change or live operation.

## LID constructor grammar at login

2026-11-07. **Confirmed local defect/fixed offline:** supplied login LIDs with
alphabetic, leading-zero or oversized users could become own-account identity.
The pinned 0ZJ.A13 ->0Tp.A0A ->0Dd.A02/A00 ->0De.A02 ->0Dx construction chain
requires 1..15 ASCII digits without leading zero. This is constructor validation,
not the separate compliance check that only logs. The existing shared
login_user helper now reuses that grammar for Server::Lid in both supplied and
saved requested-update validation. PN grammar, agent/device normalization and
valid replacement policy remain unchanged. Existing malformed-history and omitted
field policies remain local contracts.

The owned real-store regression failed before the fix with an alphabetic LID
settling successfully. Text/typed invalid users now preserve prior durable reply
capture, account/checkpoint/pin/session/outbox rows, event head and uncertainty,
including after reopen; another reservation remains blocked. Valid one- and
15-digit controls succeed; existing AD and shared CLI controls remain.
No live account operation or complete native malformed-success policy is claimed.
See [constructor/setter evidence and limits](apk/2.26.43.72-login-lid-constructor/README.md).

The earlier setter implementation unknown was scoped to that reader trace:
[existing own-Me evidence](own-companion-ingress-current.md#me-is-persisted-identity-data-not-standalone-authentication-proof)
already shows 09O.ChR fills an absent LID and preserves an existing one. The new
scan corroborates it. Its effective login flag/provider and complete replacement
lifecycle remain unknown; this patch does not apply that conditional setter's
policy universally to the canonical authenticated-identity checkpoint.
