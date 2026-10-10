# Restore returning companion membership from a guarded directory reply

2026-10-09. **Confirmed static / implemented offline**, WhatsApp APK
**2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-companion-directory-addition.json) and
[source bundle](apk/2.26.37.71-companion-directory-addition/targets.json)
retain 28 topics / 33 files / 257,345 bytes. These claims do not borrow
obfuscated class names from another build.

## Absent membership is not an absent identity pin

**Confirmed local gap:** a companion's saved pin/session may remain after its
address is removed from a reconciled device list. Its identity notification can
still queue an original guarded directory read. Rust formerly refused with
missing_device_membership even when that reply had full signed newer ADV.
That was our local restriction, not a WhatsApp server requirement.

**Confirmed native:** 11T.A0D's equal-held-key shortcut requires actual
DeviceJid membership (0098–00a8); absence jumps past equality to the full ADV
path. A matching saved key alone cannot skip signatures in this case.
Both signatures bind the actual directory candidate and independently held
primary, as traced in [changed companion application](companion-changed-key-refresh.md).
The current primary/raw-ID/account-type baseline branch (0494–04a0) then checks
actual membership index and signed Java-long time (04c8–050e). An absent index
with an old/equal ADV is deleted; only a newer candidate can enter addition.
The caller's discriminator 4/Boolean false is retained in 14J.A04, not supplied
by a new local configuration or invented teardown state.

## Remote addition and what it does not replace

**Confirmed:** 0690–06aa builds a singleton actual DeviceJid→signed index.
Cached membership comes from 1IW.A09. 06f2–071e removes every already existing
address from the proposed additions; a newer ADV never overwrites its old index.
0bN.A0G filters ownership, and the primary is included at index zero only if
missing. New companion indices must be positive Java longs after int-to-long
conversion (08f8–0994). B7j.A03 writes user_device membership with conflict
argument 4; it does not advance LbQ.A05 verified list time.

The native nonempty-addition branch chooses users through 1IW.A02. Gated
Chv canonical selection and fallback PN/LID user lookup can select a different
namespace or no user. AzR.A00 rebinds device numbers under selected users.
**Unknown/unported:** effective gates, complete counterpart expansion and
canonical namespace admission for this Rust account. This implementation uses
only the already saved exact namespace and independently bound baseline;
it creates no PN/LID alias, canonical mapping or new account provenance.
[Normal receive addition evidence and limits](peer-identity.md#remote-newer-adv-device-application-in-rust-2026-10-08)
remain relevant, and are not newly claimed complete.

The retained 1IW.A05/A04 and JhE.A0L methods delimit native addition observers.
Removed sets are empty for this call, so removal branches do not apply.
**Unknown/unported:** complete participant/group/cache and registered-callback
side effects. Adding one signed device is not receiving a new full signed list,
changing the primary, rotating identity, or reconciling every linked device.

## Rust application and inspection

**Implemented:** same-key or changed-key returning companions with a valid
held pin and absent membership now require full ADV. The current E2EE/raw-ID
list baseline must bind to an independent primary. Both ADV signatures,
positive index and strictly newer signed timestamp are mandatory. Own/unknown
own namespaces, explicit pending teardown, index collision and the local
1024-device map bound refuse. Missing historical removal registry remains
unknown; it is not overwritten or invented as an empty registry. The directory
caller's native false Boolean does not disable our explicit teardown guard.
First-seen pins still cannot acquire authority from this action.

The existing explicit apk_authenticated policy, valid signed prekey bundle,
original notification guard, unsupported-metadata deferrals and exact
reply/account/pin/session/list/primary/login rechecks remain required. Derive
the new session before committing: an equal key archives the old receiving
ratchet; a changed key starts fresh and preserves old key/session in encrypted
transition history. The same atomic transaction adds the device/index,
records ADV-added provenance, clears cached hash agreement, raises the expected
time/queues sync, writes session/history and saves its once-only application
marker. Verified floor, raw ID, primary pin and notification metadata stay
unchanged. Provenance is a sorted unique collection; a stale provenance entry
for the admitted address is retained once. Inspection alone does not normalize
or rewrite historical state.

`actions --show ID` reports **ready_addition_pending** through the applicable
companion_same_key_authority or companion_changed_key_authority diagnostic.
This means signed addition authority was staged, not that it was committed;
policy, metadata, original guard and exact transaction state can still defer.
`devices` exposes adv_added_devices and expected_timestamp through its existing
inspection surface. No HTTP route, CLI command, request ID, outgoing field or
native state override is introduced. Old equal/malformed ADV with absent
membership now reports the actual full-verifier refusal rather than the
obsolete local membership-only restriction.

## Offline verification and remaining work

Generated PN/LID fixtures cover equal and changed saved keys with independently
signed full ADV. Tests prove exact absent-device/index insertion, explicit
provenance, hash invalidation, preserved primary/floor/raw-ID/notification data,
pending sync, correct old-ratchet versus old-key history behavior, and unchanged
unrelated tables. Repeats and reopen/startup recovery commit once and never
replay the key read. Empty/unknown removal registry is retained; stale provenance
for the admitted address remains unique after its authenticated commit.

Both key cases refuse missing/empty/invalid/foreign-primary ADV, old/equal or
negative timestamp, raw-ID mismatch, index zero/high/collision/overflow,
explicit teardown, unsupported metadata, pinned policy and historical untracked
guard. Every refusal preserves all saved rows/head. Intervening membership,
primary and real ratchet changes prevent commit. A malformed pending-sync row
rolls back even an already staged membership/history/event/session change.
Listed equal-key and existing-index add-only tests remain unchanged.

Fresh live membership restoration is **unknown**. First-seen identity admission,
canonical counterpart selection, complete native addition observer effects,
primary/raw-ID/type reconciliation and receive completion remain unfinished.
No historical guard is fabricated and no chat/query/identity reset is used to
manufacture qualifying evidence.

Validation: all **885** workspace/all-feature tests passed (API 744, properties
4, crash matrix 9, CLI 54, API symbols 1, tooling 73). Strict Clippy and
warning-denying Rustdoc passed, plus default CLI (25 and API-symbol),
experimental CLI-reference regeneration, formatting and whitespace. The
28-topic / 33-file / 257,345-byte APK bundle reproduces exactly and cached
self-diff is unchanged. Self-diff is not
comparison with a newer release or proof of complete native parity.
Repository hygiene passed (2,455 local links / 125 checksum bundles,
26,423,958 APK research bytes). No qualifying live restoration is asserted by
these offline checks.

## Bounded saved-account validation

**Observed:** one 20-second receive-only window (34.01 seconds total)
authenticated/activated and stopped cleanly. Preflight had no queued action,
receipt or eligible send. Outbox, automatic reads and reconnects were disabled.
Actions (21), receipts (9), pending outbox (0), generation, registration and
journal head remained equal. Checkpoints rose 1,118→1,124; all other table
counts were equal. These are summaries/counts, not all-record byte comparisons.
[Redacted validation](companion-directory-addition-live.json).

The historical primary read remains verified_bundle/pin_matches=true,
refresh_guard=untracked, applied=false and both companion authority diagnostics
null. No historical guard is backfilled or read replayed. Nine unsupported
control logs and no decrypted/duplicate/undecryptable or identity ACK log were
recorded. No returning-companion directory reply qualified; fresh live
membership restoration remains **unknown**. No repeated window, chat or
synthetic server input was used to manufacture qualifying evidence. Private
account state, key, profile, config and logs stay outside Git.

Next native boundary: complete addition observer/participant/cache effects in
1IW.A05/A04 and JhE.A0L, including their 0bI helper calls and registered
callback, are retained but unported. Directory membership restoration does
not prove cached group fanout or all cross-device sender-key bookkeeping is
current. Continue from those retained calls without replaying a saved send
or replacing its immutable target/request reservation.

Live-record verification: CLI-reference regeneration, whitespace and repository
hygiene passed (2,456 local links / 125 checksum bundles). This follow-up changes
research/status records only; the implementation's 885 tests and strict
Clippy/Rustdoc results remain its validation. The overall goal stays incomplete.
