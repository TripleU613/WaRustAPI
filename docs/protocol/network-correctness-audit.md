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

## Explicit inactive reach-out correction (2026-11-04)

**Confirmed pinned static/refuted prior assumption:**7vL.A00 ignores expiry
for explicit inactive updates and resets restriction preferences. Rust's previous
strict expiry requirement incorrectly rejected a legitimate release lacking
expiry. It now clears explicit inactive state without parsing the field while
malformed active updates still preserve prior lock and skip ACK.
[Native dispatch/state/parser and limitations](rea/2.26.43.72-reachout-state/README.md).
Upstream sender authority and config/enum policy remain open; the retained
leaf does not establish a new server-only JID requirement. Regression covers
both active malformed preservation and inactive release. No live traffic.

## MEX reach-out dispatch correction (2026-11-04)

**Confirmed pinned mismatch fixed:**0v7.A09 dispatches only first update
child and lowercases op_name. Rust no longer admits arbitrary child tags or
later matching updates, and supports ASCII case-insensitive operation names.
[Router and JSON adapter evidence/tests](rea/2.26.43.72-reachout-state/README.md#current-mex-operation-routing-2026-11-04-offline-corrected).
Sender authorization and full typed conversion remain open. No live traffic.

## Reach-out envelope before state write (2026-11-04)

**Confirmed local integrity fix:** recognized lock updates now run existing
ACK-envelope validation before checkpoint mutation. Missing/invalid routing
fields cannot clear a lock and only later fail mex_ack. Receive still ACKs
after commit; malformed input preserves capture/lock and continues.
[Base-handler evidence, test and origin limitation](rea/2.26.43.72-reachout-state/README.md#base-dispatch-and-local-envelope-before-state-guard-2026-11-04).
Valid JID syntax does not establish account-state sender authority, which
remains open. No live traffic or native origin-parity claim.

## MEX wrapper/ordering boundary (2026-11-04)

**Confirmed pinned static:**0v7 registers0x102, reserves handler ordering
before0v6 wrapper dispatch;7MU inherits no-op8AF.A08 reservation. Worker
13 records bookkeeping and dispatches directly or via executor. These leaves
do not establish sender authority; IncomingStanza/message producer remains
the next origin trace. [Evidence](rea/2.26.43.72-reachout-state/README.md#message-wrapper-and-reach-out-ordering-reservation-2026-11-04).
No runtime policy change or live traffic.

## Receive registry and schema separation (2026-11-04)

**Confirmed pinned static discovery:** receive1s2 notification branch uses
Jb9/JoW type/operation maps and gated wrapper dispatch. This alternate
registry is not yet fully bound to reach-out or legacy0x102 routing.7Ou's
mex literal is an IQ-result overload, not a notification sender validator.
[Evidence and bounded negative results](rea/2.26.43.72-reachout-state/README.md#receive-registry-discovery-and-schema-false-lead-2026-11-04).
Origin authentication and fallback paths remain open; no runtime change.

## Receive persistence gate classification (2026-11-04)

**Confirmed pinned static:**0ZP.A04 is offline buffering/persistence policy,
A05 combines consumer readiness before that gate; neither leaf proves sender
authorization. Inverted result feeds wrapper construction in notification
receive. A03 is queue-flush scheduling, not legacy dispatch.
[Evidence and remaining lifetimes/fallback](rea/2.26.43.72-reachout-state/README.md#receive-gate-is-persistence-not-origin-validation-2026-11-04).
No Rust runtime or live-account change.

## Legacy notification MEX code binding (2026-11-04)

**Confirmed pinned static:** fallback tag-map wrapper reaches shared handler;
1rc notification branch uses EL2 literal mex->0x102 mapping, the code
registered by0v7. JID checks in1rR belong to a separate message branch and
are not notification origin evidence.
[Binding and remaining map/consumer authority](rea/2.26.43.72-reachout-state/README.md#legacy-notification-to-mex-code-binding-2026-11-04).
No runtime change or live traffic.

## Conditional offline notification preflight (2026-11-04)

**Confirmed pinned static:** shared notification JID classification is behind
offline>=11; classification codes5..8 select one callback branch. That is
not proof of universal MEX sender authority. Handler map factories declare
tags and duplicate registrations throw;1rc declares notification.
[Methods and remaining classifier/origin gaps](rea/2.26.43.72-reachout-state/README.md#high-offline-preflight-and-handler-map-construction-2026-11-04).
No runtime validation policy or live-account change.

## Offline classifier result table (2026-11-04)

**Confirmed pinned static/derived table:** nullable device, special identity,
BRq/BVC, device0 and concrete subtype predicates yield categories1..8/null.
Notification call uses final false; selected categories affect high-offline
callback routing rather than uniformly rejecting other senders.
[Exact branches and remaining predicate authority](rea/2.26.43.72-reachout-state/README.md#offline-device-classification-table-2026-11-04).
No runtime trust/ACK expansion or live traffic.

## Recognized reach-out body refusal (2026-11-04)

**Confirmed source defect corrected offline:** recognized operation with
missing/non-JSON content, malformed JSON or missing/nonboolean is_active
previously returned Ok(None), then received generic MEX ACK without applying
its state. It now returns redacted InvalidInput before any checkpoint write.
Existing receive handling preserves capture/prior restriction, skips ACK and
continues; storage failures remain separate. Unknown operation handling is
unchanged. No guessed default active state or expiry is introduced.

Real-store regression covers empty body, broken JSON, null, missing object
and string-valued active flag, with an existing lock and an unknown-op control.
Pinned8AF.A09's retained typed conversion distinguishes dispatch failure;
the local no-ACK policy is not claimed to match every native error branch.
[Native adapter and limitations](rea/2.26.43.72-reachout-state/README.md#current-mex-operation-routing-2026-11-04-offline-corrected).
No live traffic or full MEX conversion/authentication parity claimed.

## Reach-out Java decimal expiry (2026-11-04)

**Confirmed static / offline correction:** pinned 6oe.A03 uses
Long.parseLong. Active expiry strings now use the shared Java-style BMP
decimal reader, with accepted plus/non-ASCII digits and invalid-input
prior-state preservation covered in a real store. Nonnegative bounds and
JSON integer acceptance remain local policy. Explicit inactive ignores
expiry. JSON adapter and enforcement enum limits are retained in the
[evidence](rea/2.26.43.72-reachout-state/README.md#json-adapter-and-java-decimal-expiry-2026-11-04); sender authority and full native policy remain open.

## Reach-out enforcement representation (2026-11-04)

**Confirmed static/derived:** enum ordinals, internal integers and stored
TOwmL_type are distinct representations. All named variants and conversion
branches are retained in the [mapping](rea/2.26.43.72-reachout-state/README.md#enforcement-names-and-stored-type-mapping-2026-11-04). Restrict-all-companions
becomes saved 20, used by one banner reader. Another read discards the
result. Remaining consumers, companion enforcement and all outbound guards
remain open; no runtime behavior or request was changed.

## Native outreach predicate coverage (2026-11-04)

**Confirmed static / source comparison:** native 6pn.A06 combines cached
active state, recipient-set membership, record/account predicates and
optional visibility. Its set is populated from trusted-contact database
rows and mapped JIDs. One higher decision caller short-circuits on true;
full send semantics and other callers remain open. Rust's group-create
active/expiry guard does not implement this chain across outbound paths.
[Exact exits, provenance and remaining work](rea/2.26.43.72-reachout-state/README.md#outreach-decision-chain-and-recipient-set-2026-11-04).
No new policy or network operation was introduced.

## Outreach call and sticker caller semantics (2026-11-04)

**Confirmed static:** the higher predicate prevents normal sticker-send
processing in ShareMediaViewModel.A01; 1v7.A02 has a conditional pre-call
restriction branch. Jmt inverts the higher predicate for selected user-JIDs.
[Exact branches and record correction](rea/2.26.43.72-reachout-state/README.md#caller-semantics-call-gate-and-sticker-refusal-2026-11-04). Prior per-message wording
was too strong: the argument is a JID-bound cached record. Account/cache/
expiry lifetimes and full execution boundaries remain open; no runtime
policy or wire fields were guessed from UI consumers.

## Restriction deadline release correction (2026-11-04)

**Confirmed static / fixed offline:** native visible countdown expiry
fetches current restriction state, then applies success through 7vL.A00.
It does not locally write inactive. Rust group creation previously used
the deadline-filtered inspection helper and could bypass its guard with
time alone. It now checks recorded active state until explicit inactive;
no action is queued on refusal. The inspection helper is informational.
[Native chain, cache references and regression](rea/2.26.43.72-reachout-state/README.md#deadline-is-a-refresh-trigger-not-an-inactive-update-2026-11-04). Automatic refresh
and full outreach policy remain unported; stale active state can refuse
beyond the displayed deadline. No network request or identity reset added.

## Restriction recheck at action reservation (2026-11-04)

**Confirmed source defect / offline correction:** groups.queue_create
checked the restriction at queue time, but actions.reserve_limit decoded
and marked every queued create sent without another restriction check.
connection's action-drain branch calls reserve_next then link.send. A
create queued before a received active update could therefore bypass the
existing guard. This is a local ordering defect, not evidence of a new
server ban cause or identical native group-create policy.

Both reserve_limit (bulk and actor reserve_next) and reserve_one_in now
read the recorded active restriction in their reservation transaction
before decoding/marking a queued group-create sent. Bulk/actor selection
skips held creates and continues scanning other action kinds; a redacted
log explains the deferral. Selected reservation returns an explicit error.
The full row, ID, wire, timestamps and status remain unchanged while held.
Store decoding errors still propagate; no missing state or clock is
invented. Explicit inactive makes the existing queued create eligible.
Already sent/settled rows retain their normal no-replay semantics.

Real encrypted-store regression queues a create, receives active with an
already elapsed end, queues an unrelated action, and verifies bulk selects
only the unrelated request. Actor selection returns none; selected reserve
refuses. The blocked row equals its original complete JSON value. After
inactive, actor selection returns the exact original node and marks sent;
later active/inactive changes never reserve it again. No socket is opened.

Native context remains the pinned 2.26.43.72 countdown/fetch/apply chain in
[reach-out evidence](rea/2.26.43.72-reachout-state/README.md#deadline-is-a-refresh-trigger-not-an-inactive-update-2026-11-04).
The local guard is still group-create-only. Full native recipient/config
policy and automatic refresh remain unported. A stale active record can
continue deferring. The transaction is the admission boundary, not a
cancellable network write: this does not retract already reserved/sent
requests or claim atomicity between a future notification and socket I/O.

## Restriction refresh construction and document availability (2026-11-04)

**Confirmed static:** pinned FetchReachoutTimelockQuery uses empty variables
and a non-Pando spec, selecting XMPP MEX after shared transport checks.
The bounded asset server ID differs from its client ID. Native runtime
registry lookup can yield null and fail before send; its population is
still untraced. [Wire construction, asset export and limits](rea/2.26.43.72-reachout-state/README.md#refresh-request-construction-and-runtime-document-lookup-2026-11-04).
Response/correlation/atomic state application and request ownership remain
open; automatic refresh and startup polling are not introduced.

## Refresh registry initialization resolved for normal executor (2026-11-04)

**Confirmed static:** normal 0ig construction registers the MEX asset-backed
provider. 0im.CRh reads doc_id; client-ID gating is a separate method.
This resolves the prior normal-registration unknown. Global alternative
registrants, concurrent/startup order and teardown remain open.
[Exact chain and limits](rea/2.26.43.72-reachout-state/README.md#executor-registration-and-asset-backed-server-id-lookup-2026-11-04). Response conversion, correlated durable
application and automatic refresh remain unfinished; no runtime change.

## Refresh response callback admission (2026-11-04)

**Confirmed static:** XMPP response adaptation tries request-bound success,
request error, then server error. Typed conversion precedes callback
admission. The refresh GraphQL-error callback returns false and suppresses
success even with partial data; empty typed data also declines.
[Exact chain and remaining decoder/correlation work](rea/2.26.43.72-reachout-state/README.md#iq-response-conversion-and-refresh-callback-admission-2026-11-04). IQ result
or generic action settlement alone must not become restriction release.
No runtime behavior or notification ACK policy changed.

## Fetch defaults and body-selector domains (2026-11-04)

**Confirmed static:** JSONObject fetch requires its nested object but its
is_active reader is coercing/defaulted, not strict-required. Notification
8sX accessors are separate and unchanged. 8TV selectors 37/38 produce
Argo/JSON 7NM variants 0/1, mapped to parser formats 1/0 respectively.
[Exact accessor/mapping evidence and open validator semantics](rea/2.26.43.72-reachout-state/README.md#fetch-field-defaults-and-body-format-selectors-2026-11-04).
No fetch applier or raw-JSON fallback was guessed; full correlation and
durable application remain unported.

## Refresh request-bound shape and JSON selection (2026-11-04)

**Confirmed static/derived:** selected native refresh success shape binds
response id/from to original id/to, requires parent type=result and one
result child. JSON alternative ignores its format-validator return, so a
nonnull byte-body wrapper can reach JSON decoding despite missing/other
format. Full decoding and callback/error gates remain required.
[Validator and caller evidence](rea/2.26.43.72-reachout-state/README.md#request-bound-equality-and-json-alternative-error-propagation-2026-11-04). Pending-request/account ownership
and durable application are still open; no runtime behavior changed.

## Pinned refresh JSON decoding and extraction (2026-11-04)

**Confirmed static:** DI resolves 0iz; a runtime gate selects lenient
streaming object parsing with explicit end check or custom JSONTokener.
Data/errors absence/null becomes empty containers, wrong types remain
conversion failures. Good data plus malformed errors synthesizes an
error array, so the refresh callback suppresses application.
[Parser/provider chain and exact failure matrix](rea/2.26.43.72-reachout-state/README.md#pinned-json-decoder-branches-and-extraction-failures-2026-11-04). Platform/Argo
coverage and durable account-bound application remain open; no runtime change.

## Refresh pending callback and timeout ownership (2026-11-04)

**Confirmed static:** callback/timer registration precedes send handoff.
Reply and timeout paths remove ownership under a shared lock, then invoke
workers; unmatched IDs have no callback. Native 32000 delay is ms and
includes queue handoff time. [Exact methods and limits](rea/2.26.43.72-reachout-state/README.md#pending-iq-registration-dispatch-and-timeout-ownership-2026-11-04).
This does not establish durable account-bound settlement, lower writer
completion, or reconnect replay policy. Rust refresh remains unported.

## Disconnect cleanup and durable restriction reopen (2026-11-04)

**Confirmed static:** 0ln cleanup posts handler case3, which clears the
reply map and invokes failure callbacks while not clearing the earlier
queued map in that IQ branch. Initial timer reset is conditional.
**Offline verified:** existing Rust regression now reopens the encrypted
store while held and after sent reservation; complete intent/sent rows
remain intact, and already reserved creates never become replayable.
[Exact sources and fault/account-scope limits](rea/2.26.43.72-reachout-state/README.md#disconnect-callback-cleanup-and-durable-reopen-regression-2026-11-04). No production
behavior or automatic refresh changed; full lifecycle audit remains open.
