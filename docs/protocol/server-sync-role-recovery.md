# Server-sync role and recovery gates

Static current-APK audit, 2026-11-04. **Confirmed** means shipped code in
WhatsApp Android 2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Complete evidence and checked DEX](rea/2.26.43.72-server-sync-role-recovery/README.md)
extend [the lifecycle trace](server-sync-lifecycle.md). No live request, account
operation, deletion or APK execution occurred. No Rust behavior changed here.

## Role predicate is a registered device index

**Confirmed:** direct DEX interface implementors of `09F` are `09O` and
`09G`. `09O.BU7()` calls `BU8(false)`. That method initializes account state
through `A07()`, reads `registration_device_id` with default 0, and returns
whether the integer is greater than zero. `09O.Auf()` reads the same integer.
`09G.BU7()` delegates to `09O.BU8(false)` in the checked DEX. Thus these two
current implementations agree on the same predicate.

`0Yz.A05` names the true branch "companion mode" and refuses to set the
primary recovery counter there. This closes the earlier ambiguity: the native
`BU7` role test is **companion device index > 0**, not simply "registered",
"has a key", "has connected", or "has any companion devices".
`09O.A01` resets the saved registration device ID and its cache to zero.
`09T.A00` writes `companion_registration_state`, a different preference; it
must not be mistaken for the device-index setter.

**Bounded provider discrepancy:** REA reports `09G.BU7` as body unavailable.
The DEX declares a real forwarding body, retained verbatim. The existing
`apk-virtual-overrides` command follows superclass ancestry only and cannot
discover interface implementors; a direct class-def/interface-list scan found
the two implementors before the separate superclass checks. This does not claim
runtime/reflective dispatch completeness outside this APK.

## Recovery counter is distinct from collection dirty versions

**Confirmed:** native `syncd_dirty` is a primary recovery state/retry counter.
Default `-1` means no recovery in progress (`0Yz.A09 == false`). It is not a
collection version or notification delivery counter.

| Transition | Current code |
| --- | --- |
| Mark recovery | `0Yz.A03(reason)` returns if already dirty; otherwise calls `A05(0)`, removes last-companion-deregistration time and stores the reason. |
| Mark fatal recovery | `0lJ.A01(1)` additionally stores `syncd_last_fatal_error_time`; companion mode is forbidden. |
| Attempt cleanup | `0lJ.A03(force)` cancels its scheduled callback, requires connectivity and dirty state, and permits an attempt when forced or the counter is below 4. It increments the counter before work. |
| Existing companions | Cleanup logs out companions using a reason selected from dirty reason (8 hosted pairing, 6 change number, 11 register, otherwise sync failure). |
| No remaining companions | When the additional `BUx` predicate permits, it sends a `BVw` delete-all-data IQ with callback `0lJ`, discriminator 250, timeout 32,000ms. |
| Successful cleanup result | `0lJ.CK4` notifies its observer, removes the dirty reason and calls `A05(-1)`. |
| Device removed during recovery | `0lJ.Bsp` resets the counter to 0 and invokes cleanup again; the last companion removal also notifies an observer. |

`0lJ.A04()` is the narrower retry eligibility `counter != -1 && counter < 4`.
Its full error/disconnection retry schedule and the `BUx` delete eligibility
are **not** established by this audit. Do not reproduce cleanup by blindly
deleting local checkpoints, requesting server deletion, or treating timeout as
success. Those actions require their complete state transition and callbacks.

**Confirmed fatal gate:** `SyncResponseHandler.A08` marks primary recovery
or `companion_syncd_dirty` under feature 25799, releases in-flight sync without
immediate rescan and schedules recovery. The companion path persists its dirty
flag, reports the failure to its primary, and enters its own logout path. It
is not an instruction to continue syncing or ignore a failed integrity check.

## Actual initialization and reset boundaries

The default `-1` comes from the native reader, not from a Rust migration rule.
The following real writers/triggers were traced:

- `13N.A05(int)`, logged **RegistrationManager/reregister**, schedules
  `Fel(this,7)`. Its discriminator-7 body calls `0eE.A01`, then
  `0aS.A0R(11)`. `A0R` creates `Dp8`, whose constructor asserts primary mode
  and marks recovery using that reason before scheduling deletion work.
- `13N.A0K()` is **completeChangeNumber** and schedules `Fel(this,8)`, a
  different branch. A nearby worker class or the word "register" in a reason
  table does not prove every fresh registration marks recovery.
- Current sync DB reset `28m.A0G(SQLiteDatabase)` recreates sync tables and
  calls its observers; `2Bf` invokes `0Yz.A03(5)`. REA's parsed method has no
  body, so the exact checked DEX is the evidence for this path.
- The database downgrade path `1sj.onDowngrade` also reaches `0Yz.A03`.
  Its structural body is retained; no general store downgrade port is claimed.
- `FDp.CKt` clears manager caches, calls a store-reset hook and removes the
  keys enumerated by `0Yz.A03`'s static set, including `syncd_dirty`, its reason
  and bootstrap fields. This is a bounded callback/reset observation, not
  permission to clear those fields after any error.

**Refuted inference:** this trace does not establish an unconditional
delete-all-data request on every successful fresh registration. Reason 11's
confirmed entry is re-registration. **Unknown:** all fresh-registration and
generation migration initialization obligations, and the full destructive
recovery operation/response policy. Preserve the distinction in future work.

## Mapping to existing Rust state

**Confirmed source audit** (shared ingestion code is being updated separately):

| Native concept | Existing Rust representation / boundary |
| --- | --- |
| Registered identity and device role | `session::AccountRecord` has generation, registration status and optional own JID; `Jid.device` holds an address device component. There is no native `registration_device_id` preference. Mapping a validated primary identity to role index 0 is a porting decision; missing own JID or just `Registered` is not native role evidence. |
| Held collection version/LT hash | `SavedCollectionState` at `Checkpoints/app-state-collection:<name>`; restore/validate it, never substitute an advertised version. |
| Active mutation integrity map | `SavedCollectionState.values` carries index/value MACs, not the complete native syncd action row schema. |
| App-state decryption keys | Separate `app-state-key:<id>` checkpoints; owning one does not prove any native syncd mutation row exists. |
| Ready outgoing work | Durable `actions` rows plus app-state pending baseline/prepared checkpoints. `Queued`, `Sent`, `Ok`, `Error` have reservation semantics; native `pending_mutations.is_ready_to_sync` is not proven equivalent to any one of those statuses. |
| Primary/companion recovery and bootstrap gates | No equivalent durable generation-bound lifecycle row currently exists. Login success and an absent checkpoint cannot establish recovery completion. |

The native notification eligibility is **not blocked by recovery AND
(any syncd_mutations row OR any ready pending mutation)**, with a separate
critical-companion-bootstrap-state-1 alternative at the notification handler.
`0g1.A0S` tests **any row**, without filtering an active flag. Counting Rust
MAC-map entries or keys as the same predicate remains **inferred/unproven**:
native unsupported/inactive action rows and queued outgoing mutation policy
must be reconciled first. A collection at a positive held version can be empty.

## Narrow next implementation obligations

1. Trace native `0g1` row creation/removal/activation and ready-pending
   transitions against the actual Rust held/pending/action records. Establish
   the eligibility predicate including empty collections and unsupported actions.
2. Define validated role and generation-bound recovery/bootstrap checkpoints
   at their actual creation, re-registration, reset and fatal transitions. New
   primary defaults and migration of existing stores need explicit evidence;
   do not silently mark historical unknown recovery as completed.
3. Reuse the now-implemented `fetch_request_at_versions` primitive only after
   those gates are mapped, then bind dirty metadata, one-flight reservation,
   verified apply, pagination and completion rescan to atomic persisted state.
   Full destructive reset behavior remains a separately traced obligation.

This closes the concrete role predicate and exposes real recovery writers;
it does not complete a notification scheduler. Session listen remains the
affected operation. Capturing an unknown notification without a generic ACK
is unchanged until its processing obligation can be carried out faithfully.


### Registered device-index writer provenance (2026-11-05)

**Confirmed static:** exact registration_device_id string scan finds8 references.
Selected107.BLn device-JID getter feeds the saved value/cache;09O.A01 and
G0j.A02 have zero resets;0Qq.A0a reads selected-account state. Existing role/
Me-ingress audits qualify the newer mapping's unknowns. Full Rust role migration
and producer/account binding remain unported. [Writer/reader census and limits](apk/2.26.43.72-notification-handler-census/README.md). No runtime reset, query or live operation.
