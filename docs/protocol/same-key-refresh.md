# Server-authorized same-key primary refresh

2026-10-09. **Confirmed static / implemented offline** in APK **2.26.37.71**,
SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-same-key-refresh.json) and
[source bundle](apk/2.26.37.71-same-key-refresh/targets.json) preserve the
parser, Boolean producer, constructor, admission and session derivation.

## A server field, not a local bypass switch

`Bot.A03` with absent CWS and main property enabled reaches 0354. At
0370–0384 it looks up selected-user child `force_override_e2ee_state`,
passes it through `1xi.A1U(Object)`, and puts the Boolean in v38.
`1xi.A1U` tests only nonnull (0000–000a); it does not parse text/body.
The constructor call 0330–0336 maps v38 to its penultimate Boolean;
`Cai.<init>` stores that argument in A08 (0034–0038). Root metadata and
attributes with the same name do not produce this value. Main property
disabled takes the normal Bjo path without needing this field.

Rust reads the child only from a correlated authenticated **saved reply**.
It never manufactures or transmits the field, exposes a configuration override,
changes a challenge, or treats it as permission to skip signature checks.
Present attestation/PQ/identity_auth remains deferred, irrespective of this child.

## Admission invariant

For a held matching primary key and absent CWS, `CPB.A00` classifies the
candidate E2EE (02d4 → 0054), compares identities and sets v12=0
(00c6). With old E2EE, equal-state comparison 00c8 returns Bjo at 00cc.
Other stored states continue through absent-CWS 00f0 → 01e4, loading A08
at 0222. The complete native decoded state domain is NOT_SET/E2EE/HYBRID/NON;
unrecognized stored integers decode to NOT_SET via C9D.A00, as retained in
[the original preview trace](session-refresh-preview.md).

| Main property | Stored state | Selected-user child present | Result |
| --- | --- | --- | --- |
| false | any | either | Bjo: continue without assigning state |
| true | E2EE | either | Bjo: equal-key/equal-state fast path |
| true | NOT_SET | either | Bjn(E2EE), 0246 → 0306 |
| true | HYBRID/NON | yes | Bjn(E2EE), 0252 → 0306 |
| true | HYBRID/NON | no | 027a gate: Bjo if 0x7497=false, Bjp if true |

Thus the explicit server child makes admission independent of unknown current
feature flags and stored state for this **same-key, absent-CWS** subset.
No child remains deferred because the unknown sticky-state branch can refuse.
The runtime still does **not** assign a native attestation state or claim full
CPB parity. Bjo/Bjn state effects differ; their common admission is all that
this invariant establishes. Companion ADV and metadata admission remain separate.

## Preserve the old receiving ratchet

`0jc.A0L` matching identity skips changed-key A0E reset (0086 → 009c),
then still reaches CQc.A00 (0310–0324). CQc verifies the signed device key,
loads the old record, archives its current state (0124–014c), and derives
another state. Rust supplies the latest held session to
`direct::prepare_session_refresh` for matching identity; changed-key application
continues to derive fresh with no old ratchet in the candidate.

The existing explicit `transport.identity_changes=apk_authenticated` policy,
registered-account/own-PN-LID exclusions, original notification guard, exact
reply/account/session/pin commit checks and historical application marker all
remain required. Default pinned behavior is unchanged. The atomic same-key
commit preserves pin, verification, transition/version markers, sender keys,
outbox, receipts and event journal. Old receiving state lives in the refreshed
session; no false IdentityChanged event or old-key transition archive is created.
Startup can recover a settled supported reply once, never replay its key query.

## Inspect and verify

`actions --show ID` adds nullable **force_override_e2ee_state**: null until a
complete parsed bundle, otherwise selected-user child presence. Presence may
be reported with signature_valid=false; it is not signature or admission proof.
`applied` remains a historical fact. Same-key application normally leaves
pin_matches=true and refresh_guard=matches. `--preview-session` still derives
and discards independently, with persisted/admission_evaluated=false.
No new HTTP route, CLI override or outgoing request field is added.

Synthetic PN/LID tests prove new current base key plus archived original base
key/counter, unchanged unrelated tables/pin/event head, repeat/reopen once-only
application, refusal to overwrite real intervening ratchet progress, and refusal for root/attribute-only flag, forged signature, pinned
policy, companions and present unsupported metadata. Existing notification and
application guards also preserve historical untracked/refused/uncertain reads.
No historical read is retroactively assigned a notification-time guard.
Fresh live same-key application remains **unknown** until a qualifying supported
reply arrives; a short receive-only window cannot manufacture that event.

Validation: all 872 workspace/all-feature tests passed (API 731, properties 4,
crash matrix 9, CLI 54, API symbols 1, tooling 73), plus strict Clippy and
warning-denying Rustdoc. Default CLI (25 plus API-symbol), experimental
CLI-reference regeneration, formatting and whitespace passed. Exact export
verified 6 topics / 11 files / 97,899 bytes; cached self-diff was unchanged.
Hygiene passed (2,409 links / 121 bundles). No new release comparison is claimed.

## Bounded saved-account validation

**Observed:** the rebuilt actor authenticated/activated and stopped cleanly in
one 45-second receive-only window (56.31 seconds total). Outbox, automatic reads
and reconnects were disabled; preflight had no queued action/receipt or eligible
chat intent. Actions (21), receipts (9), pending outbox (0), generation,
registration and journal head remained equal. Inbound checkpoints rose 838→848;
other table counts stayed equal. These are summary/count checks, not all-record
byte comparisons. [Redacted validation](same-key-refresh-live.json).

The historical primary-LID read still reports verified_bundle, pin_matches=true,
refresh_guard=untracked, **force_override_e2ee_state=false**, applied=false.
It is not retroactively assigned provenance or reapplied. No new identity
notification/query/reply qualified for this branch. Live same-key application
remains **unknown**; the synthetic tests prove the bounded implementation.
No repeated window or chat was sent to manufacture qualifying evidence.

Live-record checks: regenerated CLI reference, API/library symbols, whitespace
and hygiene passed (2,413 links / 121 bundles). This follow-up changes docs only;
the implementation's 872 tests/strict Clippy/Rustdoc results above remain its
verification. Private configuration, key, logs and account data are ignored.
