# Network correctness audit

Owner priority, 2026-11-04: strengthen request/response correctness before adding
features. Avoiding account restrictions is the objective; zero bans is not a
measurable guarantee from static analysis. No restriction cause has been proven
by this audit. No live account traffic was sent.

## Audit ledger

This is a work queue, not a completion percentage. Each family needs endpoint or
stanza, caller conditions, serialization, ordering, response/error branches,
acknowledgement, durable state and retry/wait review against a named APK build.
A field or URL existing in an APK does not establish a mandatory request.

| Family | Rust ownership | Current result / next trace |
| --- | --- | --- |
| Registration HTTPS exist/code/register/security/challenge | registration/{factory,request_config,wire}, onboarding | [Six implemented versus 17 cataloged paths](registration-network-audit.md); existing encrypted request/wait/reservation guards; token/build preflight corrected below. Full current-build conditional field and error matrix remains open. |
| Noise/login/startup/reconnect/ping | transport/{handshake,login,connection}, runtime/comparison | [Startup audit](network-startup-audit-2026-11-04.md) and [Noise-pair/auth-key lifecycle](authkey-lifecycle.md); malformed explicit saved versions and late validation fixed. [Server ping response](server-pong.md) fixed from current writer evidence; [Auth-key/CAT outcome handling](authkey-outcomes.md) is now traced through conditional startup and timeout callbacks; full bootstrap order, reconnect flags and idle cadence remain unknown. |
| Notification dispatch and ACK | transport/notifications, groups, identity_notifications | [Current privacy-token ACK audit](notification-ack-audit.md); [Account/server sync and auth-key rotation](notification-sync-obligations.md), with [server lifecycle details](server-sync-lifecycle.md), [versioned fetch primitive](app-state-versioned-fetch.md), [patch atomicity and explicit backoff](app-state-patch-atomicity.md), [supported privacy notifications](account-privacy-notifications.md) and [privacy/MEX configuration](privacy-mex-config.md), require stateful processing before ACK; no ACK-only port claimed. Unknown type is not permission for generic ACK. |
| Direct/group message, server ACK, receipts and retry | transport/{messaging,receipts,retry_answer,group_receipts,outgoing} | Existing durable reservations; retry counts 1/2 corrected in previous batch. Higher counts, base-key checks and complete routing remain open; see retry-enc-count.md. |
| Devices/identity/prekeys/key queries | transport/{devices,peer_keys,identity_query,prekeys,device_key_batches} | Existing signed baseline and bounded query machinery; conditional fields, teardown and all negative responses require continuing current-build review. |
| Media upload/download and connection metadata | transport/{media,image_upload,image_download,voice_note} | [Media outcome persistence and pinned lifetime readers](media-network-outcomes.md) corrected/traced offline; both lifetimes are required Java longs, while full deadline/refresh policy remains open; authenticate metadata, URL/host selection, headers, expiry, hashes, bounded errors and request replay remain in the current-build queue. No endpoint-literal scan can establish required traffic. |
| Groups/status/presence | transport/{groups,group_send,status_receipts,presence} | Existing ports are partial. Queue full notification-to-refresh ordering and response obligations, not feature expansion. |
| App state/MEX/settings/history | transport/{app_state_sync,mex,chat_settings,history_notice} | Queue conditional startup dependencies, sync cursors, mutation acknowledgements and persistence. Mandatory startup status remains unproven. |
| Optional services/telemetry/third parties | versioned APK evidence, ownership not yet complete | Queue trace callers and runtime gates before classifying required/conditional/optional. Never invent events, device provenance or values. |

## Token material and declared version

**Confirmed source defect, fixed offline:** account creation derived a token from
arbitrary supplied material while retaining the fixed template User-Agent build.
Token verification checked token bytes without checking that pairing. APK version
2.26.43.72 versus template 2.26.37.70 demonstrates the mismatch locally. Existing
version/token evidence is in apk-2.26.43.72-comparison.md and AGENTS.md; this is not
new evidence of a server ban trigger.

`TokenMaterial` now reads literal manifest `versionName`, exposes `version_name`
and offers `validate_user_agent`. Account creation calls it before any directory,
key or store write; `token verify` calls it before comparison. Missing version,
malformed numeric build, non-com.whatsapp package or different configured build
refuses locally. The guard does not upgrade the User-Agent, alter token inputs,
authenticate an APK signature, or change existing account identity. Legacy
material lacking a version needs its original versioned manifest. Resource-only
versionName is not resolved by this minimal parser and therefore refuses.
Low-level token derivation remains available without this policy check. Existing
saved configurations and direct library requests do not acquire APK provenance
merely from this guard; callers must use versioned material verification.

Tests cover matching/mismatched/missing/malformed/foreign metadata and redacted
errors. Current APK extraction was additionally checked through `accounts new` with
a fresh synthetic local destination: APK 2.26.43.72 refused against template
2.26.37.70 and neither account nor profiles directory was created.

## Evidence standard and remaining risk

The pure request/baseline binder and recursive selected-data header stage are
now implemented. They preflight frozen request/domain/generation claims and
whole collections without applying state. Native fatal reasons, local parser
restrictions and external-data deferral remain distinct. Full store-origin,
current-baseline/pending-owner checks and authenticated cursor/state commits are
still obligations of the future transaction wrapper. See
[binding specification](app-state-inline-fetch-application-spec.md#implemented-pure-binder)
and [headers](app-state-structural-headers.md).

The [current patch-header audit](app-state-patch-header-audit.md) refutes a
suspected silent external/exit drop: existing decoders already refuse both.
Whole-collection nested-protobuf/header validation remains required before a
future per-patch fetch worker commits. The
[inline application specification](app-state-inline-fetch-application-spec.md)
binds prospective requests/baselines and real local generation fencing; it does
not reinterpret frozen version-zero requests or invent prepared hash authority
for pure-fetch version acknowledgements.

The [fetch-completion trace](app-state-fetch-completion.md) now has a pure ordered
structural classifier and offline response inspector. Categories are not integrity
or application verdicts; external references, protobuf semantics, request binding,
keys/MACs and generation remain separate admission stages. The
[session/source trace](session-source-me-current.md) closes selected reader and
primary-Me origins while retaining logout/switch fencing as unknown.

**Confirmed fetch wait omission, fixed:** lack of patch preparation previously
skipped app-state fetch refusal processing. Supported exact outer errors now use
the existing explicit backoff handler, with receipt-time recovery after a cut.
No duration is inferred from malformed hints or inner errors; successful fetch
contents and native retry/error classification remain separate required work.
See [fetch-specific recovery](app-state-patch-atomicity.md#fetch-specific-recovery-correction).

**Confirmed local actor correction:** generic action pacing now uses a shared
sixteen-write drain budget and a selected monotonic timer, reserving one eligible
action per send opportunity. Inbound processing and stop run between paced
writes; later actions stay queued across a failed write. This does not establish
native app-state flight ownership, send-cancellation responsiveness or a global
rate limit. [Design and implemented scope](app-state-connection-flight-design.md).
Current [pairing authentication traces](primary-pairing-auth-current.md) close
input/callback gates while leaving upstream source/account authority explicit.

**Confirmed local replay defect, corrected for new sequenced settlements:** new
correlated terminal replies acquire durable monotonic order in their settlement
transaction. Contacts/groups/blocklist replay supported kinds oldest-first;
blocklist state/hash/markers commit atomically. Legacy missing order and native
generation/fetch freshness remain unresolved. See
[settlement ordering audit](action-settlement-ordering.md).

The [current scheduler flight trace](server-sync-trigger-flight.md) distinguishes
one live multi-collection IQ job from durable historical `Sent` rows: a blanket
permanent block would not port native timeout/completion ownership. The
[own-companion ingress trace](own-companion-ingress-current.md) distinguishes
primary authenticated pairing from companion metadata reconciliation; a device
list or temporary Me presence alone does not establish read-self authority.

2026-11-04 follow-up: conflict catch-up now resolves already-held record keys
separately from its patch signing key, following current-build
`SyncResponseHandler.A07`; missing keys still roll back the whole transition.
See [transaction and key-resolution regressions](app-state-catchup-atomicity.md).
The subsequent [logical-index correction](app-state-logical-index.md) fixes
cross-key replacement/removal and validates saved map/hash consistency. Legacy
nonempty states require authenticated same-version snapshot reconstruction;
`app-state-inspect` exposes provenance and an explicit bounded local migration.
This does not establish complete sync integrity, native scheduling or projection.
Legacy prepared-only/bound writes additionally reconstruct missing logical-index
metadata from their unchanged authenticated original patch when all frozen
integrity fields match a current authenticated baseline. They do not reconstruct
missing entire preparations or retry uncertain sends; see
[prepared-state compatibility](app-state-patch-atomicity.md).

**Confirmed** means traced source or reproducible local behavior; **inferred**
means a compatibility implication; **observed** requires actual server evidence;
**unknown** stays open. A local correctness fix does not prove server acceptance
or quantify restriction risk. No unsolicited replay, identity reset, fabricated
telemetry or ignored cooldown is authorized by this queue. Existing histories,
keys, uncertain outcomes and challenge requirements remain authoritative.

## Malformed reach-out expiry preservation (2026-11-04)

**Confirmed source defect corrected offline:** record_reachout_timelock used
unwrap_or(0) for missing/malformed time_enforcement_ends. A recognized
update with valid is_active could overwrite an active saved restriction with
zero, making active_reachout_timelock return none. This could remove our
existing local group/outreach gate without a valid expiry update.

The parser now requires nonnegative i64 expiry (JSON integer or decimal
string); missing/null/malformed/negative/overflow/fractional input refuses
before storage. The connection catches this InvalidInput, preserves capture
and prior restriction, skips that MEX ACK and continues receiving. Storage
errors still propagate; valid updates keep commit-before-ACK ordering. This
is a local integrity correction, not a newly proven native timestamp grammar,
ban cause or full restriction-policy implementation. Existing enforcement
coverage outside group preparation remains an audit obligation.

Real-store regression checks active and inactive malformed updates cannot
erase a known active lock. Existing valid update/expiry regression remains.
No live account traffic was sent; no identity/cooldown reset is introduced.
