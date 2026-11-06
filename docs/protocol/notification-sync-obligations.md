# Incoming sync and auth-key notification obligations

Static audit, 2026-11-04. **Confirmed** refers to WhatsApp Android
2.26.43.72, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution, server requests, account changes or key rotation occurred.
Evidence is retained in [the audit directory](rea/2.26.43.72-notification-sync-obligations/README.md).

| Outer notification type | Current dispatcher binding | Processing before handler returns |
| --- | --- | --- |
| `account_sync` | `EL2` integer 203; `19N.<init>` registers 203 with `0v6` | `19N.A09` dispatches the first child to account settings/device/blocklist/status handlers or refresh scheduling. |
| `server_sync` | integer 210; `14l.<init>` registers 210 | `14l.A09` parses collection announcements, updates eligible app-state sync metadata, and requests sync. |
| `authkey_rotation` | integer 248; `19h.<init>` registers 248 | `19h.A09` records five allowed attempts, then calls the gated auth-key rotation manager `19i.A00`. |

**Confirmed ACK boundary:** none of these three `A09` bodies invokes the
base `0v6.A07` deferral method. They return into the ordinary `0v6.A01`
notification ACK path retained in [the preceding ACK audit](notification-ack-audit.md).
That does not mean their processing can be omitted. Async work can continue
after ACK. `14l` and `19h` pass `true` to the base constructor while `19N`
passes `false`; do not treat those constructor flags as ACK deferral.
The base error/descriptor checks still apply; a method lacking an explicit
deferral call does not prove every malformed input is ACKed.

## Account sync

**Confirmed:** `19N.A09(0Tp,int)` reads `notification.A0D()` (first child),
then branches on its tag. Examples important to network/account state:

- `privacy action=modify` schedules `1sv.A01` with the privacy flag;
  otherwise category name/value entries go to `0oe.A0R`.
- `devices` considers the outer `offline` attribute and child `action`.
  Modify/offline schedules a device refresh. Recognized device payloads also
  schedule `Fee`/`FeS` work; three timeout reasons update
  `syncd_bootstrap_fail_time`.
- `blocklist` handles previous-hash/action/admission reconciliation and can schedule a
  fresh blocklist fetch. `status`, `text_status`, `picture`, `notice`,
  `tos`, and gated `biz_opt_out_list` have separate processing paths.

**Unknown:** a complete Rust mapping for every child, callback, feature gate,
and account/device side effect. The complete Java export contains switch
recovery/duplicate-block warnings; the checked DEX export is retained alongside
it. Its apparent Java fallthrough and uninitialized-local tail are not a
specification. No fake marker/analytics calls are needed to establish the
processing obligation.

**Continuation:** [privacy refresh routing](account-privacy-refresh.md) now
traces the `2D6.A05` field through `1sv.A01`, conditional SMAX/MEX selection
and reply adoption. Effective mode selection, complete MEX wire construction
and durable privacy state remain prerequisites to refresh scheduling plus ACK
ordering for that child. Device/blocklist paths require their own reconciliation
and callback traces; an outer `account_sync` ACK-only shortcut loses them.

## Server sync

**Confirmed:** `14l.A09` handles 210. Its parser requires the server sender
and `type=server_sync`, parses optional `delete_all_data`, and accepts a bounded
collection list (0..20). Nonempty collection names and announced versions form
a map. A non-null parsed delete-all-data descriptor increments per-collection
in-memory counters; this handler does not directly delete the Rust store.

When `0aS.A0c()` is true **or**
`companion_syncd_critical_bootstrap_state == 1`, it passes every name/version
to `0aS.A0J` (`0l8.A07(String,long)`), then calls `0aS.A0P()`.
The gates must not be silently dropped or invented for a different client role.

**Continuation:** [the server-sync lifecycle trace](server-sync-lifecycle.md)
now distinguishes dirty-version metadata, eligibility, verified commit and
completion. Incremental wire construction, effective role/recovery mapping and
full scheduling/error policy remain open. Rust already has `transport/app_state_sync.rs` fetch requests,
collection verification/application and durable action queue helpers; these do
not establish the missing notification-driven version/gate/scheduling policy.
Use those primitives after tracing the policy, with deduplication, crash recovery,
and notification ACK sequencing tests. Do not ACK and discard this notification,
replace authenticated collection state with its advertised version, or interpret
`delete_all_data` as permission for untraced local deletion.

## Auth-key rotation

**Confirmed:** `19h.A09` calls `144.A0G(5)`, which clamps the persisted
`remaining_auth_key_rotation_attempts` to 0..5, then `19i.A00()`.
The manager attempts work only when remaining attempts are positive, the last
successful rotation was at least 86,400,000ms ago, and the last failure is absent
or more than 1,800,000ms ago. It keeps a generated `1nq` candidate in memory and
submits a 32-byte public key using:

```text
<iq to="s.whatsapp.net" xmlns="w:auth:key" type="set" id=...>
  <key>{32 public-key bytes}</key>
</iq>
```

The IQ callback is `2IS`, request discriminator 331, timeout 32,000ms.
`2IS.Bvj` records failure time and decrements attempts. Delivery-failure
`BsS` only logs in this body. `CK4` attempts synchronized local key replacement
after the IQ result, resets `connection_lc`, notifies `145.A04`, and only on
successful replacement clears attempts and records success time. Local replacement
failure follows the failure path. Android keystore/password fallback internals
are not fully traced; decompiler warnings make the retained callback DEX essential.

**Continuation:** [auth-key lifecycle tracing](authkey-lifecycle.md) now
establishes candidate generation, password-backed replacement, conditional
certificate invalidation and the connection getter. Pair encoding, request
manager outcome/lifetime and lost-result reconciliation remain open.
Implement a durable pending candidate and server-accepted commit boundary that
can reconcile a lost result/restart without discarding the usable old key or
replaying an uncertain rotation. Persist native cooldown/attempt semantics;
prove the exact 32-byte encoding independently and test refusal, timeout,
duplicate notification and crash boundaries. Existing Noise login key storage
is not an implemented server-requested rotation lifecycle. This is normal
protocol maintenance, never an identity-rotation mechanism for avoiding a refusal.

## Rust outcome

**Initial source audit:** these three outer types were captured without
generic ACK. The follow-on [privacy notification port](account-privacy-notifications.md)
now processes the fully supported non-modify readreceipts-only account_sync
branch. Other account_sync forms, server_sync and authkey_rotation still require
their stateful lifecycle ports; no generic ACK was added. Relevant API is session listen (`POST /v1/session/listen`,
operation `listen`, CLI `whatsapp-cli listen`). No API or code behavior changed
in this audit. The missing network obligations above remain actionable gaps;
retained source/constructor inventories are research coverage, not completed
ports. No live acceptance or ban-related outcome is established.

## Blocklist continuation (2026-11-06)

**Confirmed selected static:** [delta/refresh routing and worker handoff](apk/2.26.43.72-blocklist-account-sync-obligation/README.md)
close the blocklist child branch of the already-retained19N handler. Exactmodify,
previous-hash mismatch or failed6pA admission selects a kind4 account refresh.
Otherwise effective parsed additions/removals are applied; an empty payload or
no effective membership change returns without advancing the hash.8al selector1
persists the current full set, hash and metadata separately from handler ACK.

Refresh flags map2D6.A01 to1sx.A01;1sv records kind4's time and requests1dH.A0M.
The false last Boolean bypasses optional60-second suppression. Enabled retry
bookkeeping with the default false request retry-mode saves3; this is not proof
of three immediate requests. Selected normal handler return reaches base ACK
without0v6.A07 deferral, so async completion may follow it. Admission/mode,
A0A/A0C/backend effects, retries and durable Rust ACK/work mapping remain open.
Rust still captures this unsupported child without ACK; no production behavior
or automatic request is introduced by the trace. Do not replace it with ACK-only
handling or treat advertised delta as a full snapshot.
