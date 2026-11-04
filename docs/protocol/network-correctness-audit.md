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
| Media upload/download and connection metadata | transport/{media,image_upload,image_download,voice_note} | Queue: authenticate metadata, URL/host selection, headers, expiry, hashes, bounded errors and request replay. No endpoint-literal scan can establish required traffic. |
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
