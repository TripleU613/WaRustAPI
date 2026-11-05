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

## Timed-IQ lower writer admission and uncertainty (2026-11-04)

**Confirmed static:** timed requests carry a callback guard checked against
the queued map. Concrete observer binding transfers it to the reply map
before raw-node write, serialization/counter advance and output flush.
Native local I/O requeue is qualified by later admission; it is not proof
of universal replay permission or delivery. [Exact path and limits](rea/2.26.43.72-reachout-state/README.md#writer-callback-admission-before-output-stream-write-2026-11-04).
Durable account/request settlement and refresh scheduling remain unfinished;
no production behavior or network operation changed.

## Prekey upload reply type admission (2026-11-04)

**Confirmed source defect / fixed offline:** PendingUpload.matches in
transport/prekeys.rs only checked iq tag and ID. settle treated anything
except exact type=result as a refusal, wrote batch=refused and reset
legacy key metadata to Unsent. connection's matching branch took the
pending upload out before invoking settle. Thus a same-ID get/set,
missing type or unknown type could turn an uncertain upload into keys
available for another upload without any server refusal. This was not
observed live and is not proven to cause restrictions.

**Pinned static boundary:** APK 2.26.43.72, SHA-256
887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e,
0ln.A01 invokes IQ callback CK4 only for exact result, Bvj only for
exact error, and reports other types without passing them as a refusal
callback. The shared dispatcher evidence is retained in
[full method](rea/2.26.43.72-reachout-state/reachout-iq-reply-dispatch.txt).
This class is a shared IQ dispatcher despite the research filename prefix.
No current-build prekey payload/retry policy is inferred from the MEX leaf.

PendingUpload.matches now requires exact result/error as well as iq/id.
Direct settle also checks it before reading/writing batch state, with an
explicit redacted error. The connection retains its pending upload for
non-reply IQs; its existing receive/capture paths and timeout continue.
Supported result/error settlement, count_low behavior and existing
refusal policy remain unchanged. Sender/from, full upload response grammar,
current-build conditional fields and error waits are still separate
audit obligations. No generic sender allowlist was guessed.

A real encrypted-store regression submits same-ID IQs with missing, get,
set, RESULT and unknown types, each carrying an error child, and checks
no match/settlement, complete unchanged batch JSON, 50 uncertain/0 unsent/
0 uploaded keys and no second upload reservation. Reopen retains those
counts; the original matching result settles once and a duplicate refuses.
Existing result/error fixture remains the control for actual refusal.
No network write, key replacement, new registration or automatic retry.

Bounded source review of identity_query and peer_keys also found existing
ID/envelope/type and selected-user validation; this pass makes no new
claim of their full native parity or changes their behavior. The broader
devices/identity/prekeys audit remains unfinished.

## Device-query reply type admission (2026-11-04)

**Confirmed source defect / fixed offline:** DeviceQuery.matches checked iq/id
only, while ingest classified every non-result as refused. The receive path
could therefore consume a same-ID get/set, missing or unknown type as a reply.
Pinned APK **2.26.43.72**, **0ln.A01**, admits exact result to CK4 and exact
error to Bvj; other types do not enter either callback. Retained source:
[shared IQ dispatcher](rea/2.26.43.72-reachout-state/reachout-iq-reply-dispatch.txt).
DeviceQuery now requires exact result/error before matching or ingestion.
The encrypted-store regression checks missing/get/set/uppercase/unknown types
with a misleading error child, complete query-row preservation, unchanged pending
users and absent list, reopen, then one valid result and duplicate refusal.
Existing tests retain real error settlement coverage.

**Scope / unknown:** this establishes type admission, not sender authorization or
full usync body parity. Unlike prekey upload, begin_query does not suppress a new
reservation simply because another query is uncertain: scheduling belongs to
the receive cursor. No new reservation/retry guarantee is claimed by this test.
Direct and group send preparation matchers remain under audit. No live traffic.

## Send preparation reply type admission (2026-11-04)

**Confirmed source defects / fixed offline:** direct outgoing Pending.matches
and outgoing_group Pending.matches (membership and key variants) admitted any
same-ID iq. The actor takes those pending objects before settlement. Direct
prepare classified every non-result as refusal and marked the queued intent
Failed; group membership/key settlement could block preparation or propagate
an error without an actual result/error reply. They now require exact
result/error before matching or settlement; distribution ACK admission is unchanged.
Device waits delegate to the previously corrected DeviceQuery matcher.

**Confirmed static basis:** APK **2.26.43.72**, **0ln.A01**, offsets 0014–004c,
compares exact result/error and calls CK4/Bvj respectively. Other types are
reported, never dispatched to either callback. See retained
[dispatcher export](rea/2.26.43.72-reachout-state/reachout-iq-reply-dispatch.txt).
This establishes callback type admission, not complete native pending-map
lifetime parity: retaining uncertain durable Rust reservations is local policy,
not proof the native callback remains registered after an unexpected type.

**Regression / scope:** direct tests cover missing/get/set/uppercase/unknown
with a misleading error child, unchanged full outbox/checkpoint/session rows,
reopen and refusal to reserve that intent again. Group tests exercise membership
and key wait admission before variant-specific settlement, preserve all five
protocol tables across rejection/reopen, and refuse reselecting the intent.
The key-wait test selects that enum variant explicitly; it is not evidence of
full membership-to-key workflow parity. Existing group workflow tests cover
ordinary result/error paths. Full reply-body, sender, timeout and account
ownership policies remain open; no live account operation occurred.

**Next confirmed source gap from bounded follow-up:** retry_answer.complete_refetch
checks iq/id, loads and deletes its Refetch checkpoint, then treats every
non-result as refetch_refused. It is called before the corrected outgoing
matchers in connection.rs. The same callback-type admission issue remains there;
fixing it requires a separate refetch checkpoint/retry-history regression. This
chunk does not claim that retry refetch or activation admission is corrected.

## Retry-refetch reply type admission (2026-11-04)

**Confirmed source defect / fixed offline:** retry_answer.complete_refetch
loaded and deleted its Refetch checkpoint after iq/id admission, then treated
any non-result as refetch_refused. connection invokes it before direct/group
outgoing matching. It now returns unhandled for every type except exact
result/error before reading/deleting a job or touching retry history.

**Confirmed static basis:** pinned APK **2.26.43.72**, **0ln.A01**, offsets
0014–004c dispatch only exact result/error to CK4/Bvj. Retained
[shared dispatcher](rea/2.26.43.72-reachout-state/reachout-iq-reply-dispatch.txt).
This closes the refetch source gap recorded in the preceding send-preparation
section. Preserving durable uncertain reservations is local policy; native
unexpected-type callback registration lifetime is not established by this method.

**Verified offline:** a reserved-fetch fixture rejects missing/get/set/uppercase/
unknown type with misleading error content, compares full checkpoint/outbox/
session/identity tables, reopens the encrypted store, then settles one genuine
error and ignores its duplicate without further mutation. The existing genuine
retry workflow now injects the same non-replies before its valid signed-bundle
result: table snapshots stay identical and the later reply still produces one
same-wire-ID re-encryption with the received retry count. No retry/requeue,
identity reset or live operation was added. Sender/body policies, higher counts,
base-key decisions and activation admission remain separate unfinished work.

## Activation reply type admission (2026-11-04)

**Confirmed source defect / fixed offline:** connection's activation branch
matched iq/id and pending readiness only; any non-result was terminal
activation refused, including missing/get/set/unknown type. It now admits
only exact result/error before entering that branch. Real error remains
terminal; non-reply stanzas remain captured and do not resolve readiness,
announce availability or schedule post-activation work.

**Confirmed static basis:** APK **2.26.43.72**, **0ln.A01** offsets 0014–004c,
only result/error dispatch to CK4/Bvj. Retained
[shared dispatcher](rea/2.26.43.72-reachout-state/reachout-iq-reply-dispatch.txt).
This is type admission only, not full native activation callback/body/sender
parity or pending-map lifetime equivalence. The Rust activation deadline is
unchanged; unrelated input does not restart it.

**Verified offline actor regression:** inject same-ID missing/get/set/uppercase/
unknown types with an error child before activation. A subsequent server ping
for each proves the actor continued processing in order; its pong is the next
write, readiness is still pending, inbound evidence exists, and account rows/
journal remain unchanged. A later valid result then emits availability and
resolves readiness. Existing cut-point regression retains terminal actual-error
coverage. No live account traffic. The bounded queued-action review also
confirmed actions.record_reply already requires exact result/error before
lookup/mutation; no change or full action-policy parity claim there.

## Keepalive reply type admission (2026-11-04)

**Confirmed source defect / fixed offline:** connection's pending_ping branch
matched iq/id alone, then classified every non-result as keepalive refused.
Missing/get/set/uppercase/unknown type could terminate receive without a reply.
It now admits exact result/error before clearing or refusing the pending ping.
**Confirmed static basis:** APK **2.26.43.72**, **0ln.A01**, offsets 0014–004c,
result/error alone invoke CK4/Bvj. [Retained shared dispatch](rea/2.26.43.72-reachout-state/reachout-iq-reply-dispatch.txt).
Preserving the Rust outstanding ping is local lifetime policy, not proof of
native unexpected-type callback retention or full heartbeat timing parity.

**Verified offline actor regression:** three paused-clock scenarios each inject
five non-reply types with an error child, followed by server pings proving
ordered continued processing and saved captures. A later valid result permits
the next heartbeat; actual error remains terminal; no actual reply still reaches
the existing recoverable timeout rather than clearing pending_ping. Thus
non-replies do not falsely satisfy keepalive. The interval and timeout policy
were not changed. No live traffic.

**Bounded audit notes:** identity_query already checks result/error before its
reply-body application and peer-key parsing has its own typed reply admission;
this skim is not a complete body/sender-policy audit. A guessed file path
media_route_coordinator.rs was absent; media coordinator locations must be
resolved from actual source inventory on the next pass, not inferred from that
filename. Remaining payload/origin, timing and persistence questions stay open.

## Media-refresh settlement admission review (2026-11-04)

**Confirmed source review / existing behavior:** actual implementation is
transport/media_routing/refresh.rs (not media_route.rs or
media_route_coordinator.rs). settle_impl_in validates the sealed record,
action ID, account generation, iq tag, matching reply ID and exact result/error
before mutation. Already-settled byte-identical replies return the stored
outcome; differing replies conflict. New results also require coordinator owner
and generation-consistent current route; changed prior is retained stale.
No production correction was needed for this admission boundary.

**New offline regression:** establish an old applied route and reserve a new
refresh. Missing/get/set/uppercase/unknown types with error content, wrong ID
with otherwise usable media metadata, and wrong tag all fail without changing
any checkpoint (including old current route, owner and pending reservation).
Reopen preserves the complete snapshot and current summary, re-preparing the
same ID refuses, a later valid result applies, and its identical duplicate
causes no further mutation. This verifies durable admission/settlement behavior,
not a live server response or authority of a supplied route.

**Evidence / limits:** exact result/error callback split is pinned APK
**2.26.43.72 0ln.A01**, [retained dispatcher](rea/2.26.43.72-reachout-state/reachout-iq-reply-dispatch.txt).
Native media projection evidence is **0VM.A0A / 1xY.CK4** in
[versioned route evidence](rea/2.26.43.72-media-route-lifetimes/README.md).
Durable account binding, CAS/current pointer, byte-identical idempotence and
strong refusal of repeat preparation are Rust policies; this test does not
establish their native equivalent. Full origin validation, native callback
body constraints and automatic refresh scheduling remain open. No live traffic.

**Inspection limit confirmed:** current_summary returns host count/usability,
so two different usable one-host routes can have identical summaries. The
regression therefore verifies the final Current.action_id and decoded route ID
directly; summary equality is not evidence that route authority is unchanged.

## Retry-refetch atomic settlement (2026-11-04)

**Confirmed local persistence defect / fixed offline:** complete_refetch deleted
its Refetch checkpoint in an independent store.write before parsing/preparation
or outcome storage. Any subsequent storage/preparation error could strand the
answer in fetching with no pending job. Refusal also wrote answer and unanswered
diagnostic in separate commits. It now retires the job inside the same write as
both refusal records, or inside the existing transaction that reserves the
same-wire-ID resend, session and peer identity. Refusal verifies the saved answer
still references this query before changing it. Unexpected errors retain the job;
there is no new replay, timeout retry, ID replacement or recovery-reset API.

**Regression:** the encrypted reopen test now inserts an intentionally malformed
unanswered diagnostic. Refusal fails while reading that row after job deletion
and answer write inside the transaction; comparison of all checkpoint/outbox/
session/identity rows proves rollback preserves the pending fetch and answer.
The synthetic corrupt row is removed only in the test so the same job can then
settle one actual error and ignore its duplicate. Existing real signed-bundle
workflow still verifies exactly one replacement send/session reservation.

**Evidence distinction:** this is a Rust transaction-boundary correction, not
newly reverse-engineered APK durable storage policy. Native retry fresh-bundle
basis remains **2.26.37.71 151.A00** in [retry answers](retry-answers.md);
shared reply admission is **2.26.43.72 0ln.A01**. Native crash/account lifetime
parity and concurrent session-authority comparison remain unproven. No live traffic.

## Retry-refetch peer-state reservation guard (2026-11-04)

**Confirmed local concurrency gap / fixed offline:** complete_refetch read the
pinned identity, awaited fresh-session preparation, then overwrote peer identity
and session in its reservation transaction after checking only answer/query.
It now snapshots the prior session alongside identity and checks both again
inside that write, before retiring the job or reserving a resend. A newer pin
or ratchet produces a redacted InvalidInput and leaves pending work and newer
peer state unchanged. This is optimistic concurrency protection, not authority
to replace an identity, retry a refused query or reset any history.

**Offline regression:** the exact guard used at reservation accepts matching
identity/session, then independently refuses simulated identity and session
changes. Full checkpoint/identity/session snapshots are unchanged, including
the pending job. This exercises the transactional guard directly rather than
claiming a timed concurrent actor race. Existing signed-bundle workflow verifies
the unchanged-state case still reserves one same-ID replacement. Crash/reopen
coverage for settlement remains in the prior regression.

**Evidence distinction:** local source/read-await-write boundary supports this
fix. APK fresh-bundle behavior remains **2.26.37.71 151.A00**, retained in
[retry answers](retry-answers.md); no native compare-and-swap or complete native
account lifecycle equivalence is claimed. Concurrent changes to original-send
facts/source and full actor scheduling still require review. No live traffic.

## Retry-refetch delivery recheck (2026-11-04)

**Confirmed source gap / fixed offline:** answer checked server acceptance and
recipient delivery before querying keys; complete_refetch did not repeat either
check when the query eventually answered. Completion now refuses already
Delivered/Read/Played for the original peer or its known LID alias (or an
unscoped receipt), and refuses missing acceptance/recorded failure before crypto.
A result for another device does not by itself satisfy recipient delivery.
The reservation transaction also compares the complete original OutboxEntry
against the pre-preparation snapshot, preventing newly arrived facts or changed
attempts from being overwritten/ignored during the await. A concurrent change
returns a local error and retains the pending fetch; it is not auto-retried.

**Confirmed static basis:** APK **2.26.37.71 DNP.run**, 01c6–01f4, skips a
message already marked delivered for that device. [Existing native trace and
scope](retry-answers.md). Applying this eligibility again after waiting for a
bundle and guarding the local row transaction are Rust implementation choices,
not a claim that native storage uses this exact concurrency mechanism.

**Offline regression:** real signed-bundle workflow runs with and without a
Delivered fact arriving after query reservation. Undelivered still produces one
same-ID re-encryption; delivered settles already_delivered without changing full
outbox/session/identity tables, and duplicate reply is ignored. Recipient alias
lookup is current local mapping, not newly authenticated authority. Original age,
source changes and the full account lifecycle remain under review. No live traffic.

## Retry-refetch age eligibility recheck (2026-11-04)

**Confirmed source gap / fixed offline:** answer checked the 60-day age before
reserving a key query; complete_refetch never rechecked it. Completion now refuses
an original older than the limit before crypto and checks age again within resend
reservation, so an await crossing the boundary cannot commit a new send. A
reservation-time expiry returns a local error with the job retained; it does not
auto-retry. Missing original attempts now retain their distinct diagnostic.
The shared comparison uses widened subtraction to avoid debug panic/release wrap
on extreme i64 saved timestamps; exactly 60 days remains allowed, greater refuses.

**Confirmed static basis / approximation:** APK **2.26.37.71 150.A04**,
0416–043c compares original **178.A0F** with now minus **0x134fd9000 ms**.
[Existing retry trace](retry-answers.md). Rust continues using the first saved
attempt's prepared_at, not a newly proven equivalent of native 178.A0F. Rechecking
after the fetch is local eligibility preservation, not newly traced native
callback timing. Future timestamps retain prior non-old behavior.

**Offline regression:** real signed-bundle workflow adds an aged-original fixture
after query reservation; completion settles original_older_than_60_days without
changing outbox/session/identity tables. Normal and delivered scenarios still run.
Boundary tests cover exactly limit, limit+1, future time and both i64 extremes.
No wall-clock wait, production timestamp migration or live traffic. Full native
original timestamp provenance and pending-account lifecycle remain open.

## Current-build prekey reply-body callbacks (2026-11-04)

**Confirmed static, APK 2.26.43.72:** 140.CK4 and 141.CK4 read optional first
list/pq_list children and compare count_low with true case-insensitively.
0Tp.A0F proves first exact direct-child selection. Missing list is not itself a
refusal in these callback bodies. Existing Rust legacy boolean parsing is
supported; no production change was needed. 141 snapshots/clears pending fields
under a monitor, sets state 12, then schedules Fd9 selector 1 with both flags.
Error callback 141.Bvj uses 1ud.A01 and forwards a failure callback; exact retry
policy and durable key transitions need the downstream consumers.
[Exports, offsets, hashes, diagnostic mismatch and open boundaries](apk/2.26.43.72-prekey-upload-replies/README.md).
This is static method evidence, not a runtime observation or new REA envelope.
No live traffic. Producer ownership, PQ flag policy and native ACK/upload order
remain unproven; this trace does not establish complete prekey correctness.

## Current-build prekey storage and completion policy (2026-11-04)

**Confirmed static:** selected 141/Fd9 successful reply worker reaches
13z.A0X/A0W -> 0WJ.A0x/A0w -> 0Yb.A03/0WJ.A0v: mark sent_to_server,
seconds upload timestamps, 200-ID SQL update groups, upload history rows.
Those calls return before EcZ.A00 completion; cross-family atomicity is not
proven. EcZ adds configuration-dependent 406 and 5xx branches rather than a
universal no-retry rule. Runtime helpers/runnables/config value remain untraced.
[Exact branches, exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#store-effects-and-completion-policy-follow-up-2026-11-04).
This advances the earlier downstream boundary; it does not implement retries,
key regeneration or resets, nor claim the Rust failure policy is full parity.
No live traffic or production change.

## Current-build prekey retry limits and delay inputs (2026-11-04)

**Confirmed static, 43.72:** 13z.A02 gates 406/5xx, distinguishes A05 limited
allowance from the 0Vf counter threshold 11, and selects fixed configured delay
for limited-mode 406/503 versus Fibonacci-style delay elsewhere. EMZ.A02's
descriptor key is 0x8b60 with declared false default; actual runtime evaluation
is not known. Manager seed/cap are 10/610. Configuration, allowance producers
and runnable effects remain untraced; Rust scheduling remains unchanged.
[Exact strict boundaries, raw exports and inferred sequence limits](apk/2.26.43.72-prekey-upload-replies/README.md#retry-limits-and-delay-inputs-2026-11-04).
No live traffic or production change; this is not permission to retry/reforge
uncertain upload reservations or regenerate account keys.

## Current-build limited prekey retry mode producer (2026-11-04)

**Confirmed static, 43.72:** FDh.CbF reads allowance config0x3741 and writes
A05/A00 only under Optional-present scope, whose visible continuations throw a
deleteVNameCert-labelled exception. 14o.BLn resets mode on selector77; selector78
also consumes allowance for a legacy503 branch. The initializer is not generic
startup and the delay helper is not its only consumer.
[Conditional reachability, offsets and retained methods](apk/2.26.43.72-prekey-upload-replies/README.md#limited-mode-producer-boundary-2026-11-04).
Optional provider/config values and listener lifecycle remain unknown. No runtime
change, reset or live account operation. This narrows producer scope without
claiming complete native retry parity.

## Current-build prekey success continuation uses ADD (2026-11-04)

**Confirmed static / unported construction gap:** 43.72 EcZ -> Fcd.run ->
13z.A0B passes useSet=false, conditionally ensures requested key families,
selects ADD with op mode=add and a configured legacy batch limit (EMZ.A03,
declared default200, clamped>=1). The current Rust count_low follow-up reuses
older full SET/812 layout, so current-build equivalence is disproven.
[Exact branch, callback binding, retained methods and porting boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#successful-continuation-selects-add-2026-11-04).
No runtime patch yet: ADD selection SQL, full layout, generation and persisted
batch compatibility must be traced. Existing uncertain reservations remain held.
No live traffic. This gap stays open until implemented and validated.

## Current-build ADD prekey selection and wire layout (2026-11-04)

**Confirmed static, 43.72:** 0Yj.A02/0Yb.A01 selects unsent/non-direct keys
with parameterized LIMIT and no SQL ordering; strips public-key type byte and
uses 1nr.A04 three-byte IDs. 13z.A0B ADD contains op(mode=add), optional legacy
list{id,value}, optional pq_list{id,value,signature}; outer id/xmlns/type/to.
It does not repeat SET identity/registration/type-byte/skey/last-resort fields.
[Exact offsets, exception policy and remaining generation/config/persistence inputs](apk/2.26.43.72-prekey-upload-replies/README.md#add-selection-and-exact-child-layout-2026-11-04).
The known Rust SET-continuation gap remains unpatched; this closes layout and
legacy selection research without changing initial-upload or replay policy.
No live traffic or production change.

## Current-build legacy generation loop/cursor gap (2026-11-04)

**Confirmed static / unported:** 43.72 0WJ.A0k -> 0Yl.A07 fills the 812-key
deficit in <=50-key chunks; its backward branch returns to0046 rather than
exiting after one chunk. 0Yb.A00 count excludes sent/direct-distribution rows.
Native saved cursor ID/next expressions differ from Rust's zero-based checkpoint.
0WJ.A0u saves records and forwards the next value unchanged.
[Exact arithmetic, branch calculation, exports and migration constraints](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-generation-loop-and-cursor-convention-2026-11-04).
Current Rust one-chunk generation is not current-build parity. Cursor initialization,
PQ policy and explicit existing-store compatibility remain to be resolved before
porting. No history reinterpretation, runtime change or live traffic occurred.

## Current-build prekey cursor ownership and creation seed (2026-11-04)

**Confirmed static, 43.72:** 0YT.A01/A06 read/write raw cursor on self identity
tuple(-1,0,0); absent reader row throws instead of defaulting0. Creation0Yg.A02
uses independent 0Yg.A00()+1 seeds for legacy/PQ, each in1..0xfffffe. This
closes the creation range/reader boundary, not schema/JNI migration writers.
[Exact exports, xref bounds and existing-store compatibility](apk/2.26.43.72-prekey-upload-replies/README.md#native-cursor-ownership-and-creation-seed-2026-11-04).
A Rust port must keep local zero-based history distinct and preserve uncertain
batches, not reseed existing accounts. No implementation/reset/live traffic.

## Current-build ADD ordinary-PQ selection/generation (2026-11-04)

**Confirmed static, 43.72:** 0WJ.A1A/0Yd.A02 excludes last-resort PQ keys;
0WJ.A0j/0Yl.A06 fills a supplied target in<=50 chunks with native-loader gating
and separate positive cursor. Target provider and signing/conversion internals
remain open. ADD ordinary-PQ selection does not require SET last-resort material.
[Methods, SQL, branches and porting limits](apk/2.26.43.72-prekey-upload-replies/README.md#add-ordinary-pq-selection-and-generation-dispatch-2026-11-04).
No runtime change/live traffic; this is not full current-build PQ upload parity.

## Current-build PQ signing and wire provenance (2026-11-04)

**Confirmed static, 43.72:** generation enum A02 is ML_KEM_1024; EqE.A00
signs its public array and stores it/signature, and 0Yj.A00 supplies that array
to ADD without legacy-style stripping. 1oA.A03 delegates randomized signing
with a32-byte identity private key. Rust signs the same local row bytes sent
as value, matching that Java dataflow shape. Native generateNative has no DEX
body; current public length/prefix and primitive equivalence are not established
by these wrappers alone.
[Exports, Rust comparison and exact limitations](apk/2.26.43.72-prekey-upload-replies/README.md#pq-signing-and-wire-byte-provenance-2026-11-04).
No production change/live traffic. ADD runtime policy remains unfinished.

## Current-build PQ target provider/default distinction (2026-11-04)

**Confirmed static, 43.72:** 0WJ.<init> supplies property21200 to 0Yj.A00;
matching 08Y.A0Q default registry declares generation target200 and separate
upload batch property21201 default10. Rust currently fills the selected batch,
so setting its batch default200 would confuse inventory and upload controls.
[Provider chain, reused REA identity and lifecycle limits](apk/2.26.43.72-prekey-upload-replies/README.md#pq-target-provider-and-declared-defaults-2026-11-04).
No production/config changes or live traffic. Effective account values and
initial/continuation admission remain unverified.

## Prekey construction and reservation atomicity (2026-11-04)

**Confirmed local source defect / fixed offline:** begin_upload committed legacy
OutcomeUnknown metadata, its Batch and optional newly generated KEM rows/cursor,
then called kem_upload_children outside the write. That helper validates identity
public/private consistency even with an empty PQ row list. A local identity mismatch
could therefore strand an upload reservation before returning any node for writing.
The helper now builds/signs inside the same transaction before Batch commit;
all preceding key metadata/generation rolls back on construction failure.

**Offline regression:** for both PQ-off and a synthetic saved A/B reply enabling
PQ, mutate only the fixture identity public key to force construction failure.
Full Prekeys/Checkpoints snapshots remain unchanged, with50 unsent/0 uncertain.
Reopen preserves that exact state. Restoring the synthetic fixture pair permits
one valid reservation and settlement; a second reservation before its reply is
refused. In the PQ-on case the snapshot also proves generated ordinary/last-resort
rows and cursor writes did not leak from the aborted transaction.

**Evidence distinction / remaining scope:** this is Rust's not-sent construction
boundary, not a newly traced native transaction claim. Native ADD ownership/storage
sources remain in the [current-build evidence](apk/2.26.43.72-prekey-upload-replies/README.md).
No uncertain existing request is reset by this fix. The ADD/generation/current-build
strategy port remains open; this removes a real atomicity obstacle in the shared
reservation path first. No live traffic or account-state migration.

## Current-build SET/ADD entry and migration admission (2026-11-04)

**Confirmed static, 43.72:** 13z.A0V low-count entry requires migrated state
then schedules ADD; A0R is separately gate+guard-admitted PQ-only SET. Ex8.A03
combines property21198 with 0WJ.A0z's EXISTS(sent last-resort PQ row). Shared
A0E guards readiness/inflight/backoff. Fd5.run ignores its stored $t and forwards
flags unchanged; constructor/dispatched selector assumptions are not admission.
[Exact entry arguments, SQL and upstream/lifetime gaps](apk/2.26.43.72-prekey-upload-replies/README.md#setadd-entry-admission-and-migrated-state-2026-11-04).
No runtime strategy port yet; do not fabricate migrated state or reset pending
batches. No production change/live traffic.

## Frozen prekey request provenance (2026-11-04)

**Confirmed local persistence improvement / implemented offline:** new upload
reservations now build the entire node, encode it with the existing comparison
2.26.37.71 token dictionary, and save an encrypted internal checkpoint
prekey-upload-request:v1:<id> containing mode=set, dictionary label and wire
bytes in the same transaction as Batch/key metadata. Encoding/construction
failure rolls back the reservation. The bytes are encoded node payload, not
Noise ciphertext or evidence of a completed socket write.

The separate prefix is intentionally outside the Batch-summary scan. CLI prekeys
remains counts-only; it does not print/export these request bytes or expose a
replay command. No account keys or wire values are copied into documentation.
The checkpoint preserves the existing request semantics, not a claim of native
SET/ADD admission parity. Native callback binding evidence remains
[retained here](apk/2.26.43.72-prekey-upload-replies/README.md).

**Offline regression:** byte-for-byte comparison with the returned node encoding,
mode/dictionary assertions, full checkpoint preservation across reopen, no second
reservation, and unchanged frozen request after settlement. A synthetic historical
batch without the new checkpoint remains held and receives no backfill on reopen
or settlement. The earlier PQ-off/on construction-failure snapshot also now covers
this new write boundary. This is provenance for future versioned-mode work,
never permission to replay uncertain requests. ADD/generation strategy remains
unfinished. No live traffic.

## Uncertain KEM batch replay admission (2026-11-04)

**Confirmed local replay defect / fixed offline:** legacy keys acquire explicit
OutcomeUnknown metadata, but KEM rows retain sent=false until acceptance.
kem_prekeys.upload_rows_in selected those unsent rows again if a new upload had
fresh legacy keys. A reconnect could generate fresh legacy rows, enter a new IQ
and resend the previous uncertain ordinary/last-resort KEM rows under another ID.
This contradicted the runtime's stated uncertain-batch hold policy.

Before KEM generation/selection, begin_upload now scans Batch reservations inside
the same write transaction. With PQ enabled, any outcome_unknown batch carrying
kem_ids holds the complete new upload and emits a fixed warning. It neither
marks those rows sent/refused nor drops PQ from the new request. Existing known
outcome settlement remains authoritative; no replay/reset API was introduced.

**Offline regression:** enable PQ through a synthetic saved A/B reply; reserve
one legacy+ordinary-PQ+last-resort upload; generate a fresh legacy batch. A second
begin returns None without changing any Prekeys/Checkpoints rows. Reopen preserves
the same snapshot and blocks again, with50 unsent/50 uncertain legacy keys.
Only the original matching result settles the first batch; the next valid upload
then omits the already-sent KEM rows under the existing SET policy.

**Evidence distinction / limits:** this is local durable reservation policy,
not an assertion that APK in-memory callbacks implement the same hold forever.
Native selected-array callback ownership/mark-sent paths are retained in
[current-build evidence](apk/2.26.43.72-prekey-upload-replies/README.md). Historical
Batch rows without kem_ids default empty as before; this fix cannot reconstruct
unknown historical wire contents. Current-build ADD/inventory/migration port is
still open. No account-state reset or live traffic.

## KEM acknowledgement inventory integrity (2026-11-04)

**Confirmed local settlement defect / fixed offline:** kem_prekeys.mark_sent_in
silently skipped a missing reserved KEM row. PendingUpload.settle had already
written server_acknowledged within that transaction and would still commit,
misrepresenting local usable key inventory after loss of a reserved secret.
mark_sent_in now requires every reserved row and validates its ID/key material
before marking it sent. Failure rolls back the complete Batch/legacy/KEM settlement
transaction rather than silently skipping or regenerating a key.

**Offline regression:** extend the real PQ reservation/reconnect test, remove the
second reserved KEM row, and attempt the original result. This places the fault
after an earlier KEM row has been marked within the transaction. Complete Prekeys/
Checkpoints snapshots remain unchanged, including batch status and earlier-row
sent flag. Reopen keeps the same damaged state and still refuses settlement.
Restoring only the deleted synthetic fixture allows the original reply to settle;
no real-store repair/reset or replacement key generation is implemented.

**Evidence distinction:** this is local storage-integrity policy, not a claim
that native SQL treats missing update rows identically. APK43.72 0Yb.A03 and
0WJ.A0v mark selected IDs sent before completion, as retained in
[current-build evidence](apk/2.26.43.72-prekey-upload-replies/README.md). Their
zero-updated-row handling/crypto-material consistency is not proven equivalent.
A saved authenticated result remains inbound evidence even when local promotion
fails; this change does not assert server refusal, resend, or clear uncertainty.
Current-build ADD/inventory port and broader audit remain unfinished. No live traffic.

## Consumed KEM key before upload ACK (2026-11-04)

**Confirmed interaction / correction to prior integrity guard:** inbox.rs commits
kem_prekeys.consume_in after authenticated hybrid processing, legitimately deleting
an ordinary KEM row. A message can therefore consume the row before its upload
ACK is processed. The preceding missing-row guard alone was incomplete: absence
must distinguish known consumption from unexplained local loss.

consume_in now atomically writes an internal versioned marker containing key ID
and SHA256(public), no secret, with deletion in the existing inbox transaction.
mark_sent_in accepts a missing row only with that typed, matching-ID marker; it
does not recreate material or waive the matching upload reply. Historical missing
rows without a marker still fail closed; no backfill is attempted. New KEM generation
refuses an occupied or consumed ID even if a stale local allocator points there.

**Offline regressions:** the PQ upload/reconnect/integrity scenario first proves
unexplained deletion still rolls back settlement, then invokes the exact inbox
consume helper on the restored synthetic ordinary row. Across reopen the original
result settles while the consumed row stays absent. This is helper-level ordering,
not a newly decrypted live envelope. Existing inventory test additionally cuts the
inbox transaction after consume: both row deletion and marker roll back. A stale
allocator pointed to consumed ID5 refuses reuse. Last-resort consumption guard
remains unchanged; last-resort rows are retained, not marked consumed by this path.

**Evidence distinction:** APK37.71 0kh.BWa ordinary-only deletion evidence remains
in [hybrid incoming](hybrid-incoming.md). The receipt/consume ordering and marker
are local durable policy, not a native tombstone-equivalence claim. Markers are
internal encrypted-store provenance, not sender authority or HTTP/CLI wire output.
Future cursor migration must preserve occupied/consumed IDs. Full authenticated
receive-to-upload interleaving and historical absent-row diagnosis remain open.
No live traffic or key resurrection. This qualifies the prior integrity section.

## Current-build prekey notification/daily routing (2026-11-04)

**Confirmed static, 43.72:** 15v.A02 distinguishes migrated ADD (two presence
flags), migrated legacy-only ADD and non-migrated legacy SET via A0S. PQ-only
non-migrated skips; ACK call follows the worker decision, not proven upload wire
order. FKS daily missing-migrated-state branch ensures PQ and reaches A0R(reason10).
[Configuration descriptor, caller inventory, exact scope and unported gaps](apk/2.26.43.72-prekey-upload-replies/README.md#notification-and-daily-pq-entry-routing-2026-11-04).
Unified Rust maybe_upload is not full native strategy. A0S/Fea startup paths and
raw dispatch/effective config still need tracing. No production change/live traffic.

## Current-build legacy SET message85 handoff (2026-11-04)

**Confirmed static,43.72:** A0S admits throughA0E then snapshots account/key
material inA03 and queuesFdv. Fdv stores selected arrays under a monitor and
hands EXb through message85 to0Sf.A04; this is distinct from modernA0B callback
SET/ADD. Reason==1 enables a separate builder ensure/override argument whose
upstream scope remains open.
[Exact argument flow, PQ conditions and final-writer boundary](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-set-snapshot-and-message85-handoff-2026-11-04).
No runtime switch/live traffic. Native durable/account-lifetime equivalence and
final writer/reply-event routing are not established by this snapshot trace.

## Current-build legacy writer and success-event path (2026-11-04)

**Confirmed static,43.72:** retained0v0.A00 case arg1=85 constructs fullSET,
conditional op gating, hex ID and separate1rj reply-map entry290. Inherited
1ru.A02 forwards count_low flags to1lZ.A1Q/A0c ->what0,arg1=77 success event.
The preceding handoff prose incorrectly named what85; corrected to what0,arg1=85,
consistent with the writer's actual arg1 read. Raw evidence did not change.
[Reused writer identity, fields and callback/error lifecycle gaps](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-writer-shape-and-success-event-path-2026-11-04).
No production change/live traffic; legacy routing is not proven by shared0ln alone.

## Current-build legacy reply removal/error event (2026-11-04)

**Confirmed static,43.72:** 1rA.A03 exact result/error branches remove the
legacy ID map entry before handling.290 error uses1ud.A01/default0/null and
1lZ.A0r/A0X ->what0,arg1=78; result reaches1ru.A02 and generic forwarding.
Generic1ru.A01 is not290 error handling. Writer/reader map alias and CPm listener
binding still need proof.
[Offsets, raw/negative evidence and remaining policy gap](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-reply-removal-and-error-event-path-2026-11-04).
Legacy type admission independently supports the existing Rust correction.
Numeric error/scheduling parity remains unported; no runtime change/live traffic.

## Current-build legacy map reference binding (2026-11-04)

**Confirmed static,43.72:** normal0ZJ.A0z creates one ConcurrentHashMap and
passes it to both1rj writer and0ZJ.A0S reader context.1rC.AL1 supplies that
context map to1rD. WriterA03 and readerA00 alias on this factory path; prior
map-reference boundary is closed conditionally on factory use.
[Constructor offsets, branch scope and remaining registration/lifetime proof](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-writerreader-map-alias-2026-11-04).
Factory-set membership, listener dispatch and teardown remain open. No durable
account ownership, runtime change or live traffic is inferred from map aliasing.

## 2026-11-04 — Legacy IQ registration boundary

**Confirmed static (43.72):** inherited 1rA.A02 maps 1rD to `iq`;
0ZJ.A19 rejects duplicate tag registration. Exact 1rC constructor search
finds no direct invoke callers, which does not establish factory absence.
Configured factory membership and actual dispatch remain unknown.
[Exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-iq-handler-registration-boundary-2026-11-04). No production change or live traffic.

## 2026-11-04 — Concrete legacy listener handoff

**Confirmed static (43.72):** normal setup binds1lZ.A01 to1lY;
CPm uses arg1 and conditionally copies events to0ln/0m2 or queues on1lY.
A0Q-enabled77/78 use0ln; subsequent14o manager binding remains unknown.
[Exports, branch boundaries and interface-search limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-concrete-listener-handoff-2026-11-04).
No production change or live traffic.

## 2026-11-04 — Legacy enabled-route manager binding

**Confirmed static, conditional (43.72):**0ln.A00 routes77/78 through
SQ0.A0s provider6735, whose application-scope factory constructs14o.
This closes the A0Q-enabled route to retained14o.BLn upload branches.
Alternate configuration/factory membership remain unproven.
[Exports and conditions](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-enabled-route-manager-binding-2026-11-04). No production change or live traffic.

## 2026-11-04 — Legacy fallback event route

**Confirmed static, conditional (43.72):** selected cold-start fallback
0ZJ.A0q ->0YU.CPn/what5 returns without handling77/78. This does not
prove these events take fallback in normal configuration. Enabled route
remains bound to14o; lifecycle guards remain unverified.
[Exports and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-fallback-event-route-2026-11-04). No production change or live traffic.

## 2026-11-04 — Configured legacy IQ factory membership

**Confirmed static (43.72):** A1H set7466 includes binding126, which
allocates1rC through08o.A0Q with Object constructor inlining. Membership
is now proven; earlier zero direct constructor callers did not cover it.
A0Q comes from property0x4c7a at construction; runtime value unknown.
[Exports and remaining lifecycle scope](apk/2.26.43.72-prekey-upload-replies/README.md#configured-legacy-iq-factory-membership-2026-11-04). No production change or live traffic.

## 2026-11-04 — Disabled-flag registry route

**Confirmed static after initialization (43.72):** set7469 includes14o
(binding6735), Amf declares77/78, and0ZJ.onLooperPrepared calls0ln.A05
to register them. Disabled flag does not itself imply callback loss.
Fallback disposition remains true if reached; early-event/lifecycle unknowns
remain. [Exports and conditions](apk/2.26.43.72-prekey-upload-replies/README.md#disabled-flag-registry-route-2026-11-04). No production change or live traffic.

## 2026-11-04 — Upload reply fields separated

**Implemented offline,43.72 callback evidence:** successful prekey replies
ignore error metadata; refusals ignore count_low flags. Synthetic mixed-body
reply regression verifies outcomes and persisted batches after reopen.
[Evidence, test and remaining gaps](apk/2.26.43.72-prekey-upload-replies/README.md#rust-reply-field-separation-2026-11-04). No live traffic.

## 2026-11-04 — First coded upload error and numeric conversion

**Confirmed static / implemented offline (43.72):**1ud.A01/0HF.A00
select first present code, parse signed Java integer, default0. Rust now
persists canonical decimal without eight-character truncation; historical
batches retained. Reopen regression covers malformed-first and limits.
[Evidence and remaining scope](apk/2.26.43.72-prekey-upload-replies/README.md#rust-coded-error-selection-and-numeric-conversion-2026-11-04). No automatic retry or live traffic.

## 2026-11-04 — Upload settlement reservation ownership

**Implemented offline, local invariant:** correlated upload settlement now
rechecks ordered durable batch IDs and each key metadata status/owner inside
the transaction. Mismatches refuse without partial writes; private-row
absence does not imply lost ownership. Fault/reopen regression retained.
[Details and limits](prekey-count-notifications.md#upload-settlement-reservation-ownership-2026-11-04). No new retry or live traffic.

## 2026-11-04 — KEM upload reservation binding

**Implemented offline, local invariant:**PendingUpload retains the KEM IDs
from its reservation transaction and compares the durable list at settlement.
Empty/reordered/substituted lists refuse before any writes, on result/error.
Existing consumed-key late-ACK regression expanded.
[Details and limits](prekey-count-notifications.md#kem-upload-reservation-binding-2026-11-04). No new retry or live traffic.

## 2026-11-04 — Independent PQ count-low retention

**Implemented offline,43.72 callback evidence:**persist independent PQ
flag, retain historicalunknowns and expose redacted acknowledged-batch count
in existing CLI/HTTP prekey status. No PQ-to-legacy trigger conflation or
automatic PQ replenishment added.
[Evidence and regression](prekey-count-notifications.md#independent-pq-count-low-retention-2026-11-04). No live traffic.

## 2026-11-04 — Durable refusal upload hold

**Implemented offline, local incomplete-policy hold:** source audit found
refused unsent keys could be reserved again on notification/reconnect.
Any saved refusal now holds new reservations transactionally and is visible
in prekey status. Native retry strategy is still required, not declared
complete. [Evidence, regression and correction](prekey-count-notifications.md#durable-refusal-upload-hold-2026-11-04). No live traffic.

## 2026-11-04 — Legacy refusal workers/reset boundary

**Confirmed static (43.72):**legacy406 Fea9 callsA05/A0m(reason9)
thenA0S(reason7), distinct from modernFea13/A06/A0R. A0m records identity
reset reason and enters cleanup whose full effects remain unknown.
Legacy5xx Fea10/11 ->Fea8 rechecks backoff/admission beforeA03.
[Exports and incomplete port scope](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-refusal-retry-workers-and-reset-boundary-2026-11-04). No production change or live traffic.

## 2026-11-04 — SQLite reset base/helper selection

**Confirmed static (43.72):**base helper attempts database deletion and
checks sidecar absence;Eoe clears three collections. Helper flag0x646a
selects0Xv(axolotl.db) or28o path. Full virtual/native cleanup and rebuild
semantics remain unproven. [Exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#sqlite-reset-base-implementation-and-helper-selection-2026-11-04).
No production change, account reset or live traffic.

## 2026-11-04 — Last-resort producer and timestamp

**Confirmed static (43.72):**A0e reuses newest unsentLR or generates at
rawKEM cursor and advancespositive cursor. Storage writes upload_timestamp
while sent=false, so timestamp is not migration/ACK proof. Current Rust
SET helper differs; entry-driven ADD/rotation port remains incomplete.
[Exports and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#last-resort-producer-and-pre-upload-timestamp-2026-11-04). No production change or live traffic.

## 2026-11-04 — Refusal hold actor verification

**Verified synthetic actor:**503 settlement, repeated count notification
ACKs, clean receive and exact-store reopen produce no replacement upload
or changed private prekey rows. Only the original refused batch remains.
[Regression scope and limits](prekey-count-notifications.md#refusal-hold-actor-verification-2026-11-04). No live traffic or new retry capability.

## 2026-11-04 — Legacy success reaches migrated ADD admission

**Confirmed static (43.72):**14o77 ->Fcj3 marks both families, clears
backoff, then either demandflag ->Fc8(selector7) ->A0V. Thus existing
migration/admission/ADD evidence applies to legacy SET success too.
Rust unconditional legacy SET followup remains a confirmed gap.
[Exports and port requirements](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-success-to-migrated-add-admission-2026-11-04). No production change or live traffic.

## 2026-11-04 — Migrated success followup ADD

**Implemented offline,43.72 evidence:**independent success demand enters
gated uploaded-LR ADD reservation; family-specific inventory/batch defaults
and exact ADD children frozen transactionally. Existing cursors/SET wire
and uncertainty/refusal holds retained. Actor/PQ-only and rollback/reopen
regressions added. [Scope and remaining routing/config gaps](prekey-count-notifications.md#migrated-success-followup-add-2026-11-04).
No live traffic or full replenishment parity claimed.

## 2026-11-04 — Count notification SET/ADD routing

**Implemented offline,43.72 declared-default branch:**migrated legacy
notifications use ADD;non-migrated use SET;PQ-only remainsACK-only under
EMv0x70d7 defaultfalse. Explicit actor trigger preserves entry semantics.
Reopen/uncertain-KEM and synthetic actor tests expanded.
[Evidence and config/ACK/startup limits](prekey-count-notifications.md#count-notification-setadd-routing-2026-11-04). No live traffic.

## 2026-11-04 — Startup new-store/digest distinction

**Confirmed static (43.72):**new-store flag drivesFea6, which sends
existingunsent keys or onlygenerates;non-new-store digestflag drives
A0M/readiness/elapsed>60000ms thenarg1=88. A0Q mark-unsent entry is not
proven genericstartup. Rust startup parity remains incomplete.
[Exports and port requirements](apk/2.26.43.72-prekey-upload-replies/README.md#new-store-startup-and-digest-entry-distinction-2026-11-04). No production change or live traffic.

## 2026-11-04 — Digest request/reply wire grammar

**Confirmed static (43.72):**arg1=88 GETencrypt/digest with291selector1;
result fixed-sizefields/id lists and hash20/32 dependingonLR presence.
Success/error events82/83/84 distinguished; numeric parser differs from
upload290. [Exports and verification/port limits](apk/2.26.43.72-prekey-upload-replies/README.md#digest-request-and-wire-callback-grammar-2026-11-04).
No production change or live traffic.

## 2026-11-04 — Digest local verifier/resynchronization

**Confirmed static (43.72):**orderedlocalkey-material digest with
SHA1/SHA256 chosenbyPQgate+LRpresence;IDs arenot hashed. Registration,
key availability/ID andhashmismatches have separateA0Q reasons.
Attempttime andpersisted digestflag clears distinguished.
[Exports, preimage and recovery limits](apk/2.26.43.72-prekey-upload-replies/README.md#digest-local-material-verification-and-resynchronization-2026-11-04). No production change or live traffic.

## 2026-11-04 — Digest error/resync metadata scope

**Confirmed static (43.72):**404 worker callsA0Q4 thenclearsdigestflag;
503 onlyclearsattempttime. Resync updates formerlysent legacy/KEM metadata
and clears key-type uploadhistory, preserving recordfields inselectedSQL.
Uncertain Rustreservations remain distinct; correlation/port stillopen.
[Exports and statechange boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#digest-not-foundserver-error-and-metadata-resync-scope-2026-11-04). No production change or live traffic.

## 2026-11-04 — Offline digest inspection

**Implemented/verified offline:**existingcapturereply grammar andlocal
materialcomparison exposedthroughprekeys --digest-capture. Redactedresult
only; no statechange. FrozenACKedLRsignature used without randomized
resigning; unavailable/ambiguous evidence stays explicit. IndependentSHA1
andread-only/PQregressions retained. [Limits and usage](prekey-count-notifications.md#offline-digest-inspection-2026-11-04). No live traffic.

### Exact KEM publication signatures (2026-11-04)

Implemented offline: KEM upload reservation now atomically persists the exact
identity signature sent for each selected row. Later construction reuses and
verifies saved signatures; digest inspection prefers the saved last-resort
signature, with acknowledged frozen SET recovery for historical unsigned rows.
Missing historical signatures remain unavailable; inspection never re-signs or
writes. Invalid saved signatures fail locally. Reservation rollback and consumed
row handling remain unchanged.

Confirmed static source remains pinned APK 2.26.43.72 EqE/Dfd signature storage
and Fdf digest preimage (see the prekey-upload-replies evidence bundle). This is
partial storage parity: Rust signs at reservation, whereas native generation
stores the signature earlier. No live verification, recovery, retry-policy or
ban-prevention guarantee is implied.

### PQ gate and key transaction consistency (2026-11-04)

**Implemented, offline:** prekey SET/ADD reservation reads the saved property
21198 through the same WriteTx that selects/generates key rows and freezes the
publication request. Digest inspection reads the saved gate, account, registered
signed key and local prekeys through one ReadTx. A prior separate store read
could combine a previously read gate with later key state. This is a local
transaction consistency fix, not new APK timing evidence or a MobileConfig port.

Both paths reuse the existing correlated AB-property parser and its absent/
invalid false default. The reservation path also validates decoding of the
saved original query through the existing action write-view helper. A missing
referenced action or malformed stored action remains a local error, never an
inferred enabled gate. Static gate provenance remains APK 2.26.37.71 D1G.Bat /
07t.A0L and 2.26.43.72 13z migration predicates in the retained prekey bundles.

Regression holds an immutable snapshot across a committed config/key change,
checks uncommitted gate deletion in a write transaction and rollback, and
reopens the resulting synthetic store. Request/reply origin, current actor
account binding, effective MobileConfig, startup recovery and safe resync remain
separate unfinished work. No live operation or automatic recovery was added.

### Prekey publication account authority (2026-11-04)

**Confirmed local defect, fixed offline:** begin_upload_selected accepted an
AccountRecord supplied before reservation without rereading the durable account.
It also serialized the supplied signed-prekey signature without verification on
the PQ-disabled path. A stale caller could therefore freeze a SET under obsolete
registration/identity material, or submit a signed key from another identity.

SET and ADD admission now require registered caller and stored accounts, equal
transaction/stored/caller generation, registration ID, own JID, phone and Signal
public/private identity. These checks run in the existing reservation WriteTx
before key selection, generation, signature persistence or request freezing.
Missing account and changed authority have fixed local errors. Signed-prekey
signature is verified over the type-prefixed serialized public key under the
publication identity even when PQ is off. Authentication timestamps/credentials
are not publication identity and do not invalidate an otherwise matching caller.

This is a local consistency invariant. APK 2.26.43.72 13z.A0B/A0S request layout
and stored key sourcing remain documented in the prekey-upload-replies bundle;
these Rust error messages and checks are not claimed as native callback policy.
Regression fixtures now persist their synthetic account before reserving uploads.
Nine stale/missing/revoked account cases leave Prekeys and Checkpoints unchanged
across reopen; a foreign-identity signed key fails with PQ off, while an unrelated
authentication timestamp update permits the valid request.

**Remaining:** same-identity signed-key rotation ownership, account changes after
reservation/before write, correlated digest recovery and native retry/reset
policy are not closed by this guard. No live operations or automatic resets.

### Prekey wire ID integrity (2026-11-04)

**Confirmed local construction defects, fixed offline:** signed-prekey IDs wider
than three bytes were truncated by be3. Legacy publication trusted the numeric
store-key suffix without checking the deserialized PreKeyRecord ID, so a row
could publish its public key under a different ID. Offline digest comparison
likewise accepted those inconsistent rows and could match an aliased signed ID.

Signed IDs are now checked before publication or digest comparison. Legacy rows
share a checked loader that requires a lossless three-byte requested ID and an
embedded record ID equal to its store key. Publication failures roll back the
reservation transaction; read-only digest failures leave records untouched.
KEM publication already checked the three-byte width and row ID binding.

**Confirmed static boundary:** pinned APK 2.26.43.72 1nr.A04 writes low24 bits
(shift16/8/0), retained under ADD selection and exact child layout in the
prekey-upload-replies bundle. Its truncation is native encoder behavior; rejecting
wide local values is our consistency guard, not evidence that native rejects
those values. Native legacy row parsing can skip caught IOExceptions; Rust's
stricter invalid-row refusal remains explicitly different. No cursor, sorting,
generation, retry, reset or acknowledgement policy changes are inferred.

Regressions cover mismatched legacy store/embedded IDs, wide legacy IDs, signed
IDs 0x1000000/u32::MAX, and exact 0/0xffffff field boundaries. An independent
SHA1 digest fixture is reused with unchanged public bytes but a mismatched row
ID, and with an aliased wide signed ID: both now fail instead of falsely matching.
These field-boundary tests do not establish native generation of ID zero.

### Count ACK metadata and queue boundaries (2026-11-04)

**Confirmed static, APK 2.26.43.72:** count dispatch requests deferred metadata;
89Y.A01 consumes a one-use7p2 token viaA00. The downstream0v6.A00 branches to
bulk1ic.A0G or0Sf.A0O readiness/pre-ACK queue admission. The latter creates
message76 and can convert to queued pre-ACK message43; it is not a synchronous
socket-completion callback. [Exact methods, offsets, limitations and 18 checked
exports](apk/2.26.43.72-prekey-upload-replies/README.md#count-notification-ack-token-and-queue-stages-2026-11-04).

This closes the meaning of the worker's ACK call, not native wire-order parity.
The same metadata object dispatches at most once, but durable/redelivery-wide
ACK deduplication is not established. A six-second handler timer and pre-ACK
threshold200 were observed statically; neither is a new Rust timer/default.
Concrete live branch selection, queue durability,43 pre-ACK conversion/flush and
failure/reconnect drain behavior remain unknown. Rust ACK-before-upload-decision
ordering remains unchanged pending those boundaries; no speculative socket
reorder, pre-ACK outcome_unknown reservation, retry or duplicate suppression.
No production source change or live account operation in this evidence chunk.

### Durable pre-ACK storage and confirmation watermark (2026-11-04)

**Confirmed static, APK2.26.43.72:** the ACK conversion helper accepts message76
and uses the same generic ACK builder as its direct writer. Message43 writes the
already-built node. Native1iV.A0F serializes queued nodes into SQLite preacks.ptn;
1iY loads by ascending row ID and deletes cumulatively through a confirmed row
watermark. This closes the prior unknown about existence of durable pre-ACK
storage, but not whether every live count notification selects that branch.

The ping manager snapshots a queue marker and its successful IQ callback requests
retirement; socket write alone is not proven retirement authority. Replay-deferred
and passive-state holds, a reconnect-epoch check, capacity-driven pending drops,
periodic flush scheduling and persisted/pending selection are retained. [Exact
methods, boundaries and 24 new checked exports](apk/2.26.43.72-prekey-upload-replies/README.md#pre-ack-conversion-sqlite-persistence-and-retirement-2026-11-04).

**Still open:** effective configuration, complete initial load and replay-send
ownership, bulkCCQ selection, ping correlation/error timing, partial/ambiguous
commit handling, and all retirement entry points. Rust has durable raw inbound
captures, not this native pre-ACK queue. No ACK replay/deduplication, new ping,
automatic deletion or socket-order change was implemented from partial evidence.
No live account operation; this is a documentation/evidence chunk.

### Independent keepalive response deadline (2026-11-04)

**Fixed, offline:** heartbeat maintenance writes could delay an outbound ping,
then the next heartbeat expired it after only the remaining fraction of the
interval. A paused-clock regression reproduces a19-second maintenance delay
followed by a ping that previously timed out1second later. Each ping now owns
an admission-time deadline and a separate timeout selection; silence still
produces the existing recoverable Timeout after the configured full window.
Heartbeats cannot send a second ping while one is pending. Existing response
correlation, duration policy, request shape and reconnect classification remain.

**Confirmed static, APK43.72:** the pre-ACK confirmation ping is an empty
IQ/get/w:p to s.whatsapp.net. 0Sf.A06 registers its ID callback in0ln.A08 before
send admission, with an independent Handler delay (32000ms for this native
purpose). [Request, timer methods and regression limits](apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
The native pre-ACK delay does not replace Rust's local keepalive setting. This
does not port durable ACK replay/retirement, or guarantee timely observation
while the actor is blocked in a write. No live operation was performed.

### Pre-ACK reload and replay-generation ownership (2026-11-04)

**Confirmed static,43.72:** initial load has unbounded and oldest-window modes;
window mode tracks total persisted count separately and can skip up to eight
empty decoded pages. Reload preserves SQLite IDs with loggableID=-1. The store
compares dictionary version3 and deletes incompatible nonzero-version pre-ACK
rows; this is not identity/key reset authority. Declared descriptor values are
retained, while effective configuration remains unknown.

Replay generation initialization clears the sent watermark and increments an
in-memory connection counter. Mark-sent, clear-deferred and delayed retirement
check that generation; a watermark is not an account-generation or stanza ID.
Capacity SQL prunes newest excess rows in batches500, preserving the oldest
prefix. An independent synthetic SQLite oracle verifies strict-after paging,
cumulative retirement and that pruning direction. [Exact boundaries,11 new
exports and oracle](apk/2.26.43.72-prekey-upload-replies/README.md#pre-ack-reload-capacity-and-connection-generation-ownership-2026-11-04).

A supplied-transaction insertion path also exists, but its callback commit/
rollback semantics and full replay-batch ownership remain unfinished. No Rust
send/replay/deletion, socket order, timer or account behavior changed here.

### ACK transaction success and rollback bridge (2026-11-04)

**Confirmed static,43.72:** ACK insertion's0aq.A04 registers MZH selector1.
Its Runnable executes in BjU after successful outer transaction completion,
not directly in SQLite.onCommit. Separate selector2/BjQ handles rollback.
1hk.close ends SQLite first and only selects success release when no captured
end error, its success flag is set, and no rollback marker is present. Inner
transaction closure does not release the outer callback map.

Bulk1ic.A0G ->8b6(selector1)->1ic.A0A can insert pre-ACKs with the same0aq
transaction used by its queue helpers, then register success/rollback Runnables.
After successful completion, TB5(11) promotes persisted entries directly or
schedules further projection according to load/window state. This closes the
normal callback boundary, not crash/disk-error semantics or complete provenance
of every count notification's ACK list. [Methods,19 new exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#ack-transaction-completion-and-rollback-bridge-2026-11-04).

**Remaining:** complete count-specific bulk/config selection, Flr insertion
coroutine, TB8 projection, exact window comparator, ambiguous persistence and
full replay ownership. No Rust network/recovery behavior or live account state
changed. Future port must distinguish durable insertion from post-commit memory
promotion and confirmation-based retirement.

### ACK projection serial queue and row-order helpers (2026-11-04)

**Confirmed static,43.72:** TB8 selector8 directly calls A07; it does not add
another latch wait. The queue owns a098 serial executor: its load/projection/
periodic jobs use a FIFO wrapper that attempts next-task dispatch on normal or
exceptional completion. This closes same-instance queued-job ordering, not all
cross-thread epoch mutation, dispatch failures or database snapshot isolation.

Fey14 orders persisted row IDs naturally, substituting Long.MAX_VALUE for null;
no loggable-ID tie-break is used. Out-of-order window merge concatenates, sorts
and takes a prefix without deduplication. Refill keeps a null/backend failure
separate from a completed empty read, and adjusts only memory/count state on
successful short reads. [Methods,17 new exports and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#ack-projection-serialization-merge-order-and-refill-2026-11-04).

These are additional recovery-port inputs, not implemented Rust ACK replay or
proof every callback schedules after loading. Complete bulk ACK-list provenance,
Flr insertion, replay ownership and ambiguous persistence failures remain open.
No production Rust source or live account/network behavior changed.

### Actor IQ waits expire independently (2026-11-04)

**Confirmed local defects, fixed offline:** prekey upload and device-query waits
had30-second monotonic deadlines, but only a20-second heartbeat checked them.
A reply at31seconds could settle either request before the40-second tick. In
paused-clock regressions, the old code promoted50 prekeys to uploaded and
converted a reserved device query to server-refused from such late replies.

The actor now selects each wait's own deadline. Expired uploads drop only their
session-local waiter and keep durable outcome_unknown/key reservations. Expired
device queries are deferred for this session while pending users and the durable
query row remain unchanged. This timeout does not authorize a resend, key reuse,
repeating those deferred users, cooldown bypass or clearing uncertain history. Existing admission
triggers and the selected30-second local durations are retained.

Prekey, device-query and keepalive reply admission also checks that its waiter
has not expired, so a simultaneously ready incoming branch cannot settle an
expired waiter merely by winning select. Original inbound capture still commits
before reply handling. Session monotonic deadlines are not reconstructed from
wall-clock/server timestamps after restart; offline library settlement/inspection
and other request families remain separate authority boundaries.

**Confirmed static timing architecture, APK43.72:** 0Sf.A06 registers a pending
ID callback through0ln.A08 before send dispatch; positive delays use an
ID-specific Handler message. [Retained methods](apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
That supports independent request timers, not proof that30seconds is this APK's
native timeout for both families or that Rust now ports its entire callback map.
The earlier native pre-ACK confirmation ping's32-second delay remains distinct.

Regressions: late_prekey_reply_cannot_settle_between_deadline_and_heartbeat and
late_device_reply_cannot_settle_between_deadline_and_heartbeat. They preserve
late captures, check uncertain/query/pending-user state, service an inbound ping
and reopen without reinstalling account material. The former verifies50 keys
remain uncertain and none uploaded; the latter verifies the query remains
identical instead of storing a late503 refusal.

**Limits:** awaited writes may still delay observing expiry. Device admission
and unrelated maintenance remain on their existing cadence. Outgoing/group
waits and other request families are not converted by this chunk; no full
network/timing or ban-prevention guarantee is claimed. No live operations.

### Outgoing preparation owns its deadline (2026-11-04)

**Confirmed local defect, fixed offline:** direct key-query and group preparation
expiry was checked only on the one-second outbox tick. A group device reply
at30.75s, after its30.5s deadline but before the31s tick, could start an encrypt
query. The paused-clock regression fails on the old actor and passes with
independent timers and deadline checks before direct/group reply advancement.
The same admission check covers the group's distribution-ACK waiter.

Existing30-second local durations and failure policy remain: direct key-query
expiry ends the actor with its reserved/no-retry error; group expiry persists
blocked/outcome_unknown and defers an outstanding device query. No automatic
resend, identity change, reservation release or retry was added. Nodes received
by the actor still commit their raw capture before reply handling; actor exit
does not promise reception of later traffic. Generic durable reply/ACK
processing remains unchanged.
Awaited writes can still delay timer observation; no family-specific native
timeout duration or complete callback-map parity is claimed.

**Confirmed static architectural support, APK2.26.43.72:**0Sf.A06 registers
callbacks through0ln.A08 before dispatch, with an ID-specific delayed Handler
message; see the retained [timer methods](apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
This supports independent ownership, not copying the separate native32s
pre-ACK confirmation timeout into outgoing preparation.

Regression: group_preparation_deadline_cannot_advance_on_a_late_device_result
services the30s tick before advancing past the30.5s deadline. It verifies no
new key query, a retained late capture, continued ping service and the same
durable blocked summary after reopening without replacing account material.

**Unknown / next trace:** outgoing_group::Pending::matches accepts a distribution
ACK by description and ID; settle_with_policy inspects its error attribute.
The separate outgoing::acknowledge route has stronger class/from/ledger checks.
Native distribution ACK sender/class/callback ownership and whether every
out-of-band distribution is represented in that generic ledger remain untraced.
This is a local audit lead, not a confirmed APK mismatch; do not guess sender
address requirements or claim generic ACK persistence covers every distribution.
No live account or network operations were used.

### Distribution ACK class admission (2026-11-04)

**Confirmed static / fixed offline:** APK43.72 1rH.A03 dispatches ordinary
message ACKs by exact class; Efa/Eoh field propagation and Eoh.equals confirm
class is part of callback identity. Rust's distribution waiter previously
accepted any ACK family with the same ID. It now requires class=message for
both success and error settlement. Missing/receipt/notification/call/case-mismatch
regressions fail before the fix and pass afterward, with valid reaction
distributions still completing the existing crypto and reopen/no-replay test.
[Retained methods, error459 qualification and unknowns](apk/2.26.43.72-distribution-ack-admission/README.md).
Native distribution callback registration, address normalization and complete
durable confirmation remain open; no raw sender-equality rule or retry was added.
No live operations or complete ban-prevention claim.

### Sender-key ACK marking, one-time reset and callback ownership (2026-11-04)

**Confirmed static, APK43.72:** SendE2EMessageJob.A06 registers FfK before
dispatch and waits for its matched callback; normal successful ACK processing
checks key existence and marks selected devices. The one-time-key branch instead
reaches0i9.A0X ->0j9.A01, which resets sent_sender_key=0, not1. Blanket
interpretations of every post-ACK helper as marking reusable key possession
are refuted. This is not proof that reactions use the one-time branch.
0Th.A02 also connects2Gc selector1's pre-ACK marker origin: a matched callback
invokes the wrapped result before scheduling marker retirement, so those effects
are not one atomic transaction.
[Methods, offsets, local persistence gap and next port boundary](apk/2.26.43.72-sender-key-ack-confirmation/README.md).
Native out-of-band distribution address/registration ownership remains open.
No production change, new replay authority or live operation.

### ACK callback normalization and inbound bridge (2026-11-04)

**Confirmed static, APK43.72:** Ewq/Eng preserve the job's Eoh callback key.
0Sf.A02 canonicalizes supported device-zero from/participant fields to user
JIDs and retains nonzero companions. This helper does not establish PN/LID
alias equivalence. The generic1lZ.A1D ->1lY.Big ->0ln.A09 bridge is now
connected; the selected inbound bridge does not call that outbound normalizer.
A0T is exact normalized-key failure removal, not success handling.
[Field propagation, type predicates, callers and port limits](apk/2.26.43.72-ack-key-normalization/README.md).
Distribution producer inputs and parser/type mapping still need tracing before
changing expected-address admission. No production or live behavior changed.
