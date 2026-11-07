# Network correctness audit

**Ordinary media source binding.** Reservation and successful ciphertext adoption
now atomically compare the selected inbox bytes; plaintext handoff rechecks the
eligible source. Refusal observations remain attempt-owned. Offline regression
covers replacement/removal/metadata changes and reopen. Native lifecycle remains
unverified. [Details](media-network-outcomes.md#ordinary-download-source-binding-2026-11-06).

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
| Groups/status/presence | transport/{groups,group_send,status_receipts,presence} | [Current presence targets and token merge](presence.md#current-target-types-and-outbound-guard-2026-11-06) verified; unsupported target bypass and incoming PN/LID metadata projection fixed. Other incoming JID families and upstream typing activity/cancel, effective producer gates and full notification-to-refresh ordering remain open. Existing group/status ports are partial. |
| Account/profile/contact actions | transport/{actions,about,profile_picture,blocklist,two_factor,report,contacts} | [Producer-specific action callbacks](chat-actions.md); photo outer/error selection and report32s callback ownership corrected offline. Full payload/native-state adoption, report optional metadata and complete per-producer response matrices remain open. Generic Ok is not feature acceptance. |
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

### ACK API result variants and distribution-shape qualification (2026-11-04)

**Confirmed static, APK43.72:**0Sf.A0B/A0C register through0Th.A01/2Gc0,
which discards the returned node; A0D and ordinary SendE2EMessageJob use
A02/2Gc1 and retain it. Null get() results therefore do not universally mean
a missing ACK. Failure forwarding does not retire the held pre-ACK marker.
SendLiveLocationKeyJob builds notification/type=location with participants,
not ordinary direct messages. Neither that job nor includeSenderKeysInMessage
proves the native ordinary reaction distribution shape or callback address tuple.
[Methods, caller inventories and remaining gaps](apk/2.26.43.72-ack-api-variants/README.md).
No production behavior, speculative address restriction or live operation changed.

### Auth-challenge ACKs preserve uncertainty (2026-11-04)

**Confirmed static / fixed offline:** APK43.72 1rH.A03 diverts numeric459
to onAuthChallengeReceived before ordinary ACK class processing. The traced
manager transfers an existing request between maps; it is not an ordinary
terminal message failure or proof of a ban. Rust direct/group ACK settlement
now excludes459, preserving original reservations and uncertainty. Java decimal
spellings are covered; no challenge bypass, retry or account-wide policy added.
[Methods, before/after regression and unported boundaries](apk/2.26.43.72-ack-auth-challenge/README.md).
No live operations. Full challenge-controller parity remains incomplete.

**Validation follow-up:** the full run exposed an unrelated outbox eligibility
fixture assuming distinct wall-clock milliseconds. Production sorts timestamp
ties by local ID. The fixture now assigns distinct accepted_at values; send
ordering policy is unchanged. The failed run is retained locally, and the full
suite is rerun after this test-only correction.

### Comparison sends share message ACK classification (2026-11-04)

**Confirmed local defects / fixed offline:** CLI comparison direct and final
group sends stored every nonaccepted ACK as server_refused, including numeric459.
The out-of-band group distribution loop accepted any ACK family without an
error and did not preserve the matched ACK itself with its other captures.

All three now use messaging::message_ack_outcome: ordinary message-class ACKs
classify as accepted/refused; numeric459 is AuthChallenge before class dispatch;
other ACK families are Unrelated. Challenge/unrelated map to outcome_unknown
and return an explicit no-retry error after saving the response. Distribution
ACKs are preserved through the encrypted inbound capture path, and unresolved
ones stop the loop before any later distribution/group write. They do not
confirm sender-key possession. Original intents remain nonreplayable.

**Confirmed static support, APK43.72:**1rH.A03's class dispatch and special459
diversion, Integer.parseInt grammar and callback-map distinction are retained
in [auth-challenge evidence](apk/2.26.43.72-ack-auth-challenge/README.md) and
[ACK admission evidence](apk/2.26.43.72-distribution-ack-admission/README.md).
The shared helper classifies a node only; ID/address binding remains the caller's
duty. Existing wait_for matches tag/ID. Full native expected-address tuples,
participant/type/edit rules and challenge recovery remain incomplete.

The bounded CLI exits with uncertainty rather than waiting for a later matching
family; this is local no-replay policy, not a port of the native callback
scheduler. Existing ordinary refusal statuses remain unchanged. Historical
saved server_refused rows are not rewritten. This chunk adds no retry, reset,
new request shape or live operation.

Validation: protocol table covers missing/wrong classes, numeric459 decimal
variants including full-width digits, ordinary500, accepted ACKs and non-ACK
nodes. Existing durable actor reservation regressions still exercise the same
shared challenge parser. Full socket-level CLI sequence coverage is not claimed.

### Comparison IQ waiter reply admission (2026-11-04)

**Confirmed local defect / fixed offline:** messaging::wait_for matched only
node tag and ID. A same-ID IQ get/set, absent type, empty type or differently
cased Result could return as a request response, ending the wait and reaching
a caller's success/refusal or parsing path. The shared comparison waiter now
requires exact result/error for IQs. Other stanza families keep tag/ID matching;
message ACK classification remains separate.

**Confirmed static, APK43.72**, SHA-256887a6f94…0c2e:
0ln.A01 reads type and invokes CK4 only for result and Bvj only for error;
unexpected types reach unexpected_recv_iq_type diagnostics/assertion.
[Retained REA method](rea/2.26.43.72-authkey-outcomes/incoming-iq-callback.json),
Evidence ev_445bcabed23b1091f4b30e63e2bd073f980299ad5b82df8787e84b425d8bf9c2.
This supports dispatch grammar, not identical native pending-map lifetime or
assertion/error policy. Rust retains unmatched IQs in its existing bounded
pending buffer and continues awaiting a valid reply. No new resend.

The existing30-second whole-operation timeout and32-node count bound remain
local policy. Regression comparison_iq_wait_only_admits_native_result_error_replies
fails with the old extracted predicate and passes after the guard; it covers
request/absent/empty/case-mismatched types, both response types, wrong IDs/tags
and unchanged ACK tag/ID admission. This is a predicate regression, not a new
encrypted-socket integration proof or complete native callback-map parity.

**Remaining capture gap:** wait_for returns buffered unmatched nodes only on
success. Timeout, terminal stanza, read error and count exhaustion currently
discard that local buffer; the helper has no store parameter. That loss is not
fixed by this admission guard and must be addressed separately without turning
a failed wait into permission to resend reserved requests. Generic ACK sender
normalization and challenge-controller integration also remain incomplete.
No live traffic or account operation.

### Comparison wait captures survive failed exits (2026-11-04)

**Confirmed local defect / fixed for repository CLI callers:** wait_for kept
unmatched decoded nodes only in its pending Vec, returned only on success.
Timeout, transport error, terminal stanza and count exhaustion discarded that
buffer before callers could persist it. This impaired diagnosis of reserved
requests; it was not evidence those requests had never reached the server.

New messaging::wait_for_captured uses the same bounded read/match loop with an
observation callback that commits each decoded node through inbox::preserve
before matching, checking a terminal tag or advancing the count. It captures
the matched reply too. Capture failure propagates and stops the operation; it
does not authorize settlement or another write. Every current repository CLI
comparison IQ/ACK wait now supplies its existing encrypted store, including
media routes, direct/group key queries, distributions and group retry refetch.
Older successful-path captures remain for compatibility and content-hash writes
remain idempotent; these are diagnostics, not ACKs or processing confirmations.

The existing public wait_for signature is retained for callers without a store;
it still has no durable capture guarantee. Both wrappers share the same IQ
result/error admission,30-second whole-operation deadline and32-node bound.
An explicit post-observation monotonic check prevents capture latency from
allowing reply settlement after the deadline. Blocking storage can delay
observing expiry; no hard real-time responsiveness guarantee is claimed.

**Evidence scope:** this is local diagnostic/persistence hardening, not a claim
that the APK uses our inbox table or content-hash keys. The pinned43.72
[IQ callback grammar](rea/2.26.43.72-authkey-outcomes/incoming-iq-callback.json)
remains the admission reference; native callback-map/challenge parity and
per-request durable recovery are separate work.

Regression decoded_wait_captures_survive_each_exit_and_reopen_without_settlement
uses the actual shared wait loop and encrypted ProtocolStore. Synthetic readers
exercise valid response, timeout, read error, terminal stanza, count exhaustion
and injected capture failure. After reopen, every committed decoded-node
capture exists and a saved uncertain intent remains unchanged.
Capture failure prevents reading the queued matching response. The test is
not an encrypted-socket/Noise interoperability proof.

**Limits:** undecodable or partially read frames do not produce a Node and are
not captured by this helper. Nodes not yet read at timeout cannot be preserved.
No existing historical loss is reconstructed, uncertain send is replayed,
identity is changed, or server wait is overridden. Raw-node capture alone does
not perform protocol ACK/business processing for unrelated notifications.
No live account or network operations were used.

### Undecodable comparison wait frames retained (2026-11-04)

**Confirmed local gap / fixed offline:** wait_for_captured persisted decoded
nodes but still used read_node, which discarded a decrypted frame on stanza
inflation or binary decode failure. It now reads FrameRead and preserves an
Undecodable frame through the existing encrypted inbox capture mechanism
before returning a fixed uncertainty/no-retry error. Already captured decoded
nodes remain. The loop never classifies the malformed frame as a response or
ACKs it, and never reads another node after that failure.

The stored record retains the exact decrypted, pre-inflation frame, fixed codec
error class, first observation time and epoch=None (comparison/lab window),
using the existing content-hash identity. The existing default `undecodable`
command lists these captures redacted and rechecks them with the current
decoder; it does not send traffic or automatically settle them. Capture-store
errors propagate and cannot become successful request settlement.

The shared wait-loop regression now includes malformed binary and invalid
compressed payload cases after two decoded unmatched nodes. It verifies the
error, all prior captures, exact saved frame bytes/class/size, absent actor
epoch and unchanged uncertain send intent after reopening the encrypted store.
Reader injection tests the shared control flow and capture storage; it is not
a Noise-handshake or live malformed-server test.

**Evidence/authority:** this is local diagnostic hardening, not a reconstructed
APK decoder-failure policy. Existing native result/error dispatch evidence
remains the admission reference; whether the APK continues or closes after each
decode failure remains untraced. The comparison waiter exits conservatively.
The ordinary actor's existing continue/capture policy is unchanged.

**Limits:** ciphertext that fails authentication, incomplete frames and data not
yet read do not yield a decrypted frame for this path. The legacy no-store
wait_for remains uncaptured. The earlier decoded-only comparison limitation is
closed for successfully decrypted records; no old missing evidence is restored.
This adds no replay, account reset, fabricated response, new route or live operation.

Validation follow-up: the initial full run reached a stale CLI-reference check
after the command description changed during verification. The regenerated
reference and final source are checked together in the subsequent full rerun.

### Activation reply grammar and deadline ownership (2026-11-04)

**Confirmed local defects / fixed offline:** comparison receive classified any
same-ID IQ as activation result/refusal, including request-shaped IQs. Actor
activation already checked result/error but could admit a late result when the
incoming select branch won against its elapsed timer. Both also started their
30-second response window only after the activation write completed.

The comparison path now uses shared messaging::iq_reply_matches, admitting only
exact IQ/id/result-or-error, and checks its deadline after capture. The actor
checks the same grammar and monotonic deadline before activation-triggered
availability, readiness and startup work. Both create the existing30-second
deadline before sending the request. An awaited write can still delay timer
observation; completion does not restart the response window or authorize a
late result. No activation resend or identity reset was added.

**Confirmed static architectural support, APK43.72:**0ln.A01 dispatches exact
result/error, and0Sf.A0A registers through0ln.A08 before request dispatch.
A08 schedules an ID-specific delayed Handler message for a positive duration.
[Retained callback grammar](rea/2.26.43.72-authkey-outcomes/incoming-iq-callback.json),
[registration](rea/2.26.43.72-authkey-outcomes/pending-registration.json) and
[send order](rea/2.26.43.72-authkey-outcomes/send-iq-internal.json).
This supports grammar and timer ownership, not proof of a native30-second
activation duration or complete callback-map/activation scheduler parity.

Regressions: activation_admission_rejects_the_deadline_boundary_and_later_replies
checks result/error before, at and after expiry using the production admission
helper. slow_activation_write_does_not_restart_the_response_deadline holds the
fake activation write for31seconds, then verifies prompt terminal uncertainty,
no readiness and no availability/startup write. Existing non-reply IQ actor
coverage and native server ping/activation cut tests remain. The comparison
loop's encrypted-socket behavior is not independently newly exercised here.

Captures precede reply admission. A comparison request-shaped activation IQ
stays captured rather than becoming a refusal; if no timely valid
response arrives, activation remains unconfirmed. Existing account/key/send
reservations are not cleared by expiry. Other reply families, awaited-write
responsiveness and full native request-address policy remain separate audits.
No live operations or complete ban-prevention guarantee.

### Comparison request deadlines start before write (2026-11-04)

**Confirmed local timing gap / fixed offline:** comparison IQ/message callers
performed write_node and only then started wait_for_captured's30-second window.
A slow request write could therefore extend request ownership by a fresh30s.
Activation was already fixed separately; the remaining9 repository comparison
write/wait pairs now use messaging::write_and_wait_captured.

The helper creates the existing30-second monotonic deadline before exactly one
write and passes that same deadline into the capture-aware wait. An expired
deadline immediately returns uncertainty without reading a queued late response.
The existing post-capture expiry guard also remains. Request envelopes, IDs,
query batching, encryption and saved send reservations are unchanged. No send
is reserved or retried by the helper itself; the caller retains that duty.
Membership/read-only and retry-refetch request durability is not newly claimed.

Covered pairs: group membership, device batches, directory batches, direct
distributions, final group send, group retry key refetch, direct message send,
media-route query and direct-message key query. Other unpaired writes and
receive loops retain their existing policies. Read-only wait_for and
wait_for_captured APIs still start a window when called; request senders should
use the new combined helper to avoid splitting deadline ownership.

**Confirmed static architectural support, APK43.72:**0Sf.A0A calls0ln.A08
registration before A05 dispatch; A08 schedules its positive-duration callback
timer by ID. [Retained send order](rea/2.26.43.72-authkey-outcomes/send-iq-internal.json)
and [registration](rea/2.26.43.72-authkey-outcomes/pending-registration.json).
The separate [ACK APIs](apk/2.26.43.72-ack-api-variants/README.md) also register
with0Th before dispatch; their full timer policy remains distinct.
This supports registration-time ownership, not a claim that30s is the native
duration for every covered request or that all native scheduling is ported.

Regression expired_request_deadline_is_not_restarted_when_reply_wait_begins
advances the paused clock past a preexisting deadline before entering the actual
shared wait loop, then verifies Deadline and zero reader calls despite a queued
matching result. This models slow-write latency; it is not a new encrypted
socket integration test. Existing decoded/malformed capture and uncertainty
reopen regressions exercise the same deadline-aware core.

**Limits:** a pending write is still governed by its existing transport I/O
policy and can delay observation of the response deadline; it is not silently
cancelled by this helper. A failed write returns its existing error. Unread
late records are not captured, and expiry never proves the server did not
receive the write. Original uncertain intents remain nonreplayable. No native
request-address/challenge-controller parity, new route or live operation.

### Challenged-request selection and re-dispatch ownership (2026-11-04)

**Confirmed static, APK43.72:**0TV.A03 selects eligible held requests without
removing those source entries; over-budget entries are removed.0Sf.A08/A0L
re-dispatch through A04 with the original callback retained when absent from
the new invocation. A token-result callback and the selected handleConnected
block reach selection. The token callback calls key rotation before dispatch,
but does not prove rotation completion. Client-side selection counters are not
a server retry allowance or authority to replay Rust reservations.
[Exact methods, map inventories, count conditions and remaining gaps](apk/2.26.43.72-challenged-request-ownership/README.md).
The post-token helper is now connected; token construction/converters, request
family/callback ownership and durable restart recovery remain incomplete.
No production change, new retry authority or live operation.

### Client-auth-token writer grammar and validation (2026-11-04)

**Confirmed static, APK43.72:** exact Dzb(String,String,int) selector28 builds
a server IQ GET in urn:xmpp:whatsapp:push with a pn child containing UTF-8
manager input. That label is not proof of phone-number input; the branch does
not create an auth_token element. The ID helper checks String.length, not a
53-bit numeric value. Nonnull length violations log via00L.A0C and return true
in the selected validator;300 is not a proven server or hard constructor limit.
[Selected overload, helper chain, checksums and unknowns](apk/2.26.43.72-client-auth-token-writer/README.md).
Writer construction is now connected; input eligibility/authenticity, reply
converters and durable challenge recovery remain unported. No production
change, request sender, new retry authority or live traffic.

### Client-auth-token reply binding and byte grammar (2026-11-04)

**Confirmed static, APK43.72:** FUT's selected result pipeline first requires
iq, matching original request ID, parsed from equal to original destination
and type=result. It then reads opaque cat bytes with strict1..1024 bounds.
The value converter alone does not perform request correlation. Native child
lookup picks the first cat, without duplicate counting in this selected path.
Invalid header throws schema error; null token conversion after a valid header
returns without token persistence or held-request dispatch.
[Converters, helper chain and remaining eligibility/recovery limits](apk/2.26.43.72-client-auth-token-reply/README.md).
This closes the selected reply converter gap, not native JID parser, input
authenticity, failure/backoff or durable recovery parity. No production change
or live operation; Rust uncertain holds remain.

### Client-auth-token trigger boundary (2026-11-04)

**Confirmed static, APK43.72:** GCM map key pn maps to the push-consumer
argument labelled pushNonce. The manager call is guarded by its nonnull check;
full v31 control-flow preservation and trusted delivery provenance remain open.
0AI.A0O is exactly state==2; traced transitions write3 (with server-disconnected
diagnostic) and4. The state2 producer and complete eligibility remain unknown.
[Checked exports, caller inventory and limits](apk/2.26.43.72-client-auth-token-trigger/README.md).
No sender, retry authority, production behavior change or live traffic.

### Client-auth-token readiness producer (2026-11-04)

**Confirmed static, APK43.72:** 0AI.A0M writes state2, opens the condition
variable and invokes the selector2 observer notifier. Both exact callers are
in the connected handler, on distinct branches; state2 does not prove every
connected-path side effect completed. The notifier invokes BzL; the token
manager implementation drains one replaceable in-memory pending slot and
rechecks readiness. Registration, scheduling, instance identity and concurrent
lifecycle ordering remain untraced.
[Full selected bodies and limits](apk/2.26.43.72-client-auth-token-readiness/README.md).
No runtime sender, durable recovery claim, new replay authority or live traffic.

### Client-auth-token observer topology (2026-11-04)

**Confirmed static, APK43.72:** readiness dispatcher separates inline ordinary
observers from marker-dependent inline/scheduled paths. The state constructor
uses provider-backed combined observer sets; explicit registration search alone
cannot establish membership. A separate connection-thread pending-slot drain
precedes a conditional rotation invocation in its selected block. Membership,
dispatch category, provider identity and cross-thread ordering remain open.
[Selected bodies, excerpt, searches and limits](apk/2.26.43.72-client-auth-token-observer/README.md).
No Rust behavior change, sender, new retry authority or live traffic.

### Client-auth-token observer membership and delivery (2026-11-04)

**Confirmed static, APK43.72:** readiness set binding0x1d2c includes0x83,
which resolves to the token-manager factory through the global cached resolver.
The combined set preserves per-member resolvers into observer iteration. DEX
interface declarations place1jA in07v, so the null-hint readiness notification
enters scheduled dispatch. The task re-enumerates observers before invoking
the notifier; token-manager delivery still rechecks readiness.
[Bindings, declarations, dispatch and limits](apk/2.26.43.72-client-auth-token-membership/README.md).
Membership/category gaps from the prior trace are closed for this selected
path. Executor completion/order, cache resets, generation binding, trusted
push origin and failure/backoff remain open. No runtime or live change.

### Client-auth-token nonce register preservation (2026-11-04)

**Confirmed static, APK43.72:** a digest-bound conservative CFG checker
finds only the original v69-to-v31 copy reaches the manager argument move.
The later v31 reuses cannot reach it without another definition; the sole
call chain is guarded by the original nonce's nonnull check. The previously
retained method was regenerated from the pinned APK and matched byte-for-byte.
[Reproducible checker, results and limits](apk/2.26.43.72-client-auth-token-nonce-flow/README.md).
This closes the register-preservation gap, not trusted push origin, freshness,
complete eligibility, failure/backoff or durable recovery. No runtime change,
sender, new retry authority or live traffic.

### Client-auth-token delivery failure versus IQ error (2026-11-04)

**Confirmed static, APK43.72:** FUT's token selector restores the original
nonce on BsS only via compareAndSet(null,nonce), preserving newer pending
input. Bvj only logs in that selector; client-config selector0 behavior does
not apply. Neither branch immediately sends or dispatches held requests.
Submit-time A0U requires ready flag A06 plus state2, beyond the manager's first
state2 check; failure calls BsS directly, while success registers before dispatch.
[Callbacks, helper, submit chain and outcome limits](apk/2.26.43.72-client-auth-token-failure/README.md).
Timer expiry shares BsS restoration, not proof of rejection or safe replay.
Global backoff, shutdown/callback races and durable recovery remain open.
No Rust sender, runtime change, new retry authority or live traffic.

### IQ pre-write promotion and shutdown boundaries (2026-11-04)

**Confirmed static, APK43.72:** writer C0e promotes callbacks fromA0I toA0H
before its node-write entry point. The concrete0uw binding is traced through
writer initialization. A0H membership is not proof of wire delivery. Callback
presence check and promotion are separate, leaving a possible timeout
interleaving; removal alone cannot establish definite non-delivery. Local
drops post a failure path; shutdown case3 snapshots/clearsA0H and schedules
BsS, without clearingA0I in its selected IQ block.
[Complete selected bodies, ordering and limits](apk/2.26.43.72-iq-promotion-shutdown/README.md).
Full socket/reconnect behavior, shutdown eligibility, concurrent and durable
ownership remain open. No Rust behavior change, new replay authority or live traffic.

### Partial encrypted comparison write regression (2026-11-04)

**Observed offline local fixture:** the verified classic-Noise handshake test
now exercises `messaging::write_and_wait_captured` with a real encoded IQ,
encrypted record and framed AsyncWrite over a local Tokio duplex stream. The
peer reads the three-byte record header and64 ciphertext bytes, then drops
the stream. The advertised record exceeds the8192-byte duplex capacity, so
the whole record cannot be buffered before disconnect. The helper returns
BrokenPipe; a second attempt on that consumed channel returns Closed. A
synthetic outcome_unknown checkpoint is unchanged after encrypted-store reopen.

Test: `transport::handshake::tests::partial_encrypted_comparison_write_keeps_durable_uncertainty_and_consumes_channel`.
The fixture synchronizes after handshake-finish consumption to avoid hiding
record bytes inside the peer's frame-reader buffer. It uses a valid local
certificate chain, not disabled certificate verification.

This adds a partial-wire cut to the existing callback-level failure and raw
framing/cancellation checks. It supports the [native pre-write promotion
boundary](apk/2.26.43.72-iq-promotion-shutdown/README.md): registry admission
and an I/O error cannot establish definite non-delivery. The checkpoint is
a test fixture, not a production outbox reservation or complete reconnect
policy; this is local duplex I/O, not a TCP/live-server observation. No runtime
behavior, sender, retry authority or live account operation changes.

### IQ shutdown eligibility and cancellation-map ownership (2026-11-04)

**Confirmed static, APK43.72:** ordinary disconnect passes the original reason
and channel.BNJ (false when absent) into the registry flush. The selected
login-back failure branch instead gates the call on BNJ and passes(true,-1).
Concrete1A3.BNJ is queued-handler-message4 OR delegate.BNJ, not a connection
state alias. Adjacent0Sf.A0J invokes cancellation callbacks and clears a
separate map; it does not directly clear IQ maps or the pending token slot.
[Complete selected bodies, channel path and limits](apk/2.26.43.72-iq-shutdown-eligibility/README.md).
Delegate semantics, cancellation callback effects, full reconnect ordering and
durable ownership remain open. No Rust behavior change or live traffic.

### Concrete channel queued-work predicate and deferred drain (2026-11-04)

**Confirmed static, APK43.72:** selected writer readiness binds0vI into
0ZJ.A04. The channel predicate combines connection-handler message4, writer
message2 and a nonempty deferred deque. Those are queued-work checks, not
wire-delivery or executing-write completion proofs. Connection case0 clears
the writer flag and drains head Messages throughA00's existing gates; case1
sets the flag. Dequeue/drain does not prove retransmission or safe replay.
[Concrete binding, submit chain, drain and limits](apk/2.26.43.72-channel-queue-predicate/README.md).
Full reconnect ordering, races, replacement/generation ownership and durable
recovery remain open. No Rust behavior change or live account operation.

### IQ response ownership precedes token parsing (2026-11-04)

**Confirmed static, APK43.72:** response admission cancels the tracked timer
and looks upA0H; the ordinary token callback is later claimed by removal in
A02 before executor parsing. Failure/shutdown can win between those stages;
lookup or timer removal is not successful token installation. The selected
LV5 catch handles LIL's recoverable flag without reinsertion, BsS restoration
or held-request dispatch. Recoverable does not authorize network replay.
[Dispatch chain, declarations, exception routing and limits](apk/2.26.43.72-iq-response-ownership/README.md).
Full upstream admission, hooks, executor ordering and durable/generation
ownership remain open. No Rust behavior change or live account operation.

### IQ reader admission and unmatched fallback (2026-11-04)

**Confirmed static, APK43.72:** the selected1rD IQ processor reaches the
ordinary response bridge only for result/error with nonnull ID. get/set have
separate branches. A lower-level1ru map can process a result first; an
ordinary IQ-registry miss posts a fallback which can remove a separate
message callback and invokeBsT(null). A registry miss is not proof of global
nonprocessing. Declared iq tag mapping and processor-map registration are
traced; full injected/alternate bindings and callback effects remain open.
[Selected bodies, caller inventories and limits](apk/2.26.43.72-iq-reader-admission/README.md).
No Rust behavior change, new replay authority or live account operation.

### Shared lower-level IQ ownership and push-config producer (2026-11-04)

**Confirmed static, APK43.72:** the selected setup passes one new map to
reader1rD.A00 and writer1rj.A03. Push-config GET registers1rv before writing;
its result builds a configuration object and forwards it separately from the
ordinary IQ registry. The generic token node path has no explicit low-level
record insertion in the selected writer bodies. Alias escapes, ID collisions
and full Cyu behavior remain open; this is not a global absence claim.
[Shared-map setup, producer, result bridge and limits](apk/2.26.43.72-lower-level-iq-ownership/README.md).
Full configuration grammar/application, other families and durable/generation
ownership remain untraced. No Rust behavior change or live account operation.

### Push-config projection and selected consumer (2026-11-04)

**Confirmed static, APK43.72:** parsed config projects eight internal Bundle
keys: push tokens, mute state, account count, pkey and VOIP payload type.
Long-to-String and per-field holder overwrite behavior are traced; the keys
are not proven wire attribute names. Selected type6 consumer forwards seven
Strings to1uj.A04 and handles FBNS separately via conditional1uo.A00.
[Projection, dispatch/consumer bodies and limits](apk/2.26.43.72-push-config-projection/README.md).
Full wire grammar, injected handler binding, downstream effects and durable
reconciliation remain open. No Rust behavior change or live account operation.

### Push-config verification work and numeric defaults (2026-11-04)

**Confirmed static, APK43.72:**1uj.A04 builds VERIFY work and omits null/empty
String extras. It puts account count/VOIP ints with parse fallbacks1/0; the
helper uses Java Integer.parseInt without explicit trim/clamp. Enqueue failure
can be logged with normal return, so parsing/consumer return does not establish
work completion or corrective send. ServiceA05 separately compares state and
applies additional numeric/helper gates before invokingEbv.A00.
[Projection, callers, selected gates and limits](apk/2.26.43.72-push-config-verification-work/README.md).
Complete token/provider lifecycle, comparisons, sender outcomes and durable
ownership remain open. No Rust behavior change or live account operation.

### Corrective push-config construction and local admission (2026-11-05)

**Confirmed static, APK43.72:** Ebv emits server iq/set with one config child;
builderA04 merges same-tag fragments instead of appending another wrapper.
Provider-token config ID and generated IQ ID are distinct. A0W/A0A requires
ready flag/channel presence, logs connection state without gating on it,
registers before dispatch and can permit deferred offline writer queueing.
Its boolean is local admission. Config FUT selector0 differs from token
persistence/recovery and its immediate-failure path.
[Sender, helper composition, validators and limits](apk/2.26.43.72-push-config-corrective-submit/README.md).
Full attribute/trigger policy, helper effects, races and durable/generation
ownership remain open. No Rust behavior change or live account operation.

### Push-config reply collection and variant choice (2026-11-05)

**Confirmed static, APK43.72:**7Ow bounds successfully converted config wrappers
at0..2, rather than counting every raw config child.8B6.A0K returns the first
nonnull variant or null after all candidates fail; the constructor skips null
conversions. Converter exceptions are not caught by the selected helpers.
[Complete selected methods and limits](apk/2.26.43.72-push-config-reply-collection/README.md).
Individual variant grammar, thrown failures, duplicate semantics and durable
reconciliation remain open. No Rust behavior change or live account operation.

### Android push-config wire fields and null gates (2026-11-05)

**Confirmed static, APK43.72:**8TD requires platform adm/fbns/gcm and token
id String length1..1024 to construct its Android variant. Optional mute/count/
pkey/VOIP results may be null; appid/settings parse results are discarded.
Item/hash parsing bounds converted entries at0..1 and nullable item failure
does not itself reject the Android variant.
[Field mapping, exact helpers and limits](apk/2.26.43.72-push-config-android-grammar/README.md).
Other variants, header/Long lexical details, provenance and durable reconciliation
remain open. No Rust behavior change or live account operation.

### Push-config alternate variants and Long syntax (2026-11-05)

**Confirmed static, APK43.72:** the five non-Android converter types have
selected admission gates and still count toward the reply collection bound,
even though their visitors do not update Android projection holders. Long
conversion handles optional sign, leading zeros and overflow without trimming;
Character.digit delegates digit acceptance to the Android runtime.
[Selected converter gates, complete exports and limits](apk/2.26.43.72-push-config-variant-admission/README.md).
Full optional/Apple item semantics, Unicode tables, header validation, provenance
and durable reconciliation remain open. No Rust behavior or live traffic changed.

### Push-config GET startup ordering and404 work (2026-11-05)

**Confirmed static, APK43.72:** the sole direct GET producer caller is in the
selected startup method after token drain/conditional rotation. Lower-level
error ownership is removed before code parsing and can emit more than one
callback for multiple error children. Its type27 bridge reaches selected404
handling that queues VERIFY with seven null String inputs, not a direct GET
retry. [Caller, excerpt, error bodies and limits](apk/2.26.43.72-push-config-trigger-error/README.md).
Full startup eligibility, provider work, timeouts and durable recovery remain
open. No Rust behavior or live account operation changed.

### FBNS local/server comparison and provider entry (2026-11-05)

**Confirmed static, APK43.72:**1uo suppresses refresh only for a nonempty matching
server/local token and matching saved app version, behind its eligibility gate.
It updates last_server_fbns_token before refresh; null removes that server view,
not the local token in this body. Refresh selects an accepted Android provider
and requests registration through explicit service IPC, not a direct IQ resend.
[Complete methods and limits](apk/2.26.43.72-push-config-fbns-reconciliation/README.md).
Provider callback/authenticity, token persistence, later SET and durable recovery
remain open. No Rust behavior, provider IPC or live account operation changed.

### FBNS token receipt and optimistic server marker (2026-11-05)

**Confirmed static, APK43.72:** registered-token worker validates the provider
PendingIntent creator, eligibility and nonempty token before its state path.
When the token differs from last_server_fbns_token it invokes config SET,
discards the admission boolean and writes that marker on normal return before
local-token comparison/save. The marker is not exclusively server-observed state.
[Receiver, explicit excerpts, helpers and limits](apk/2.26.43.72-push-config-fbns-token-received/README.md).
Full provider/generation authority, ACKs and durable recovery remain open.
No Rust behavior, provider IPC or live account operation changed.

### FBNS provider message ACK continuation (2026-11-05)

**Confirmed static, APK43.72:** the selected FBNS worker reaches provider ACK
after normal message handling, empty data or its JSONException catch. Nonempty
sender/notification ID and sender allowlist gate an authenticated Android
broadcast with processor_completed=true. This does not prove payload success,
durable processing or WhatsApp stanza acknowledgement.
[Branch offsets, error continuation and limits](apk/2.26.43.72-push-config-fbns-token-received/README.md#provider-message-ack-after-normal-handling-or-json-failure).
Full downstream processing, provider delivery and durable ACK ownership remain
open. No Rust behavior, provider IPC or live account operation changed.

### Unified-session startup node and cached ID (2026-11-05)

**Confirmed static, APK43.72:** the node immediately after config GET in the
selected startup block is ib/unified_session, not another IQ. Its String ID is
cached/generated from an offset-aware clock modulo a week, with separate refresh
and identity-observer producers. Refresh submission uses family447 and its local
boolean does not establish delivery.
[Exact methods, callers and limits](apk/2.26.43.72-unified-session-startup/README.md).
Full eligibility, clock authority, cadence, writer/ACK and durable recovery remain
open. No Rust behavior or live account operation changed.

### Unified-session eligibility is Me-record presence (2026-11-05)

**Confirmed static, APK43.72:**0Sb's global provider binds09G, whose BUx delegates
to09O: lazy-load then test nonnull Me record. That predicate is not socket
readiness or current authenticated-session proof. The cached unified ID fast path
can return before rechecking it; explicit refresh and identity observer are separate.
[Provider route, metadata, exact predicate and limits](apk/2.26.43.72-unified-session-eligibility/README.md).
Full provider/load authority, generation fencing, writer/ACK policy and durable
recovery remain open. No Rust behavior or live account operation changed.

### Unified-session envelope and writer queue (2026-11-05)

**Confirmed static, APK43.72:** logical kind447 travels in arg2 of generic
arg1=6; normalization returns no replacement for this envelope. The selected
writer skips IQ promotion, can defer offline and calls the generic node writer.
IOException requeues locally; neither queueing nor local return proves delivery
or durable replay authority.
[Helpers, exact branch offsets and limits](apk/2.26.43.72-unified-session-writer/README.md).
Full Cyu/reconnect, server ACK policy and generation recovery remain open.
No Rust behavior or live account operation changed.

### Conditional startup availability and bounded inbound search (2026-11-05)

**Confirmed static, APK43.72:** after config/unified-session, selected startup
conditionally calls the bare presence/type=available writer. Rust shape agrees;
full availability/mode/control-flow policy remains unproven. Exact unified_session
literal search finds only its two outbound builders, not proof of absent ACKs.
[Complete writer, selected gates, query and limits](apk/2.26.43.72-startup-availability/README.md).
Full caller/companion/passive semantics, response policy and generation recovery
remain open. No Rust behavior or live account operation changed.

### Post-availability session hooks and metrics handoff (2026-11-05)

**Confirmed static, APK43.72:** selected startup records session ID, starts
conditional offline-resume metrics and records uptime. Persisted metric bytes
are removed before parsing/handoff; prior incomplete metric entries can be
removed before sink invocation. This is metrics state, not proof of application
completion, mandatory telemetry or request replay authority.
[Complete methods, explicit worker excerpts and limits](apk/2.26.43.72-startup-session-hooks/README.md).
Full sink transport/sampling, offline-complete semantics and durable/generation
recovery remain open. No Rust behavior or live account operation changed.

### Startup metrics sink selection and sampling (2026-11-05)

**Confirmed static, APK43.72:** metrics provider959 resolves to a factory selecting
0NY,KJu orKJv by configuration. PrimaryCSJ can drop via nullable sampling result;
accepted events reach another conditional local queue. Routing may target primary,
alternate or both, not a universal per-connect network write.
[Factory route, complete entries, metadata and limits](apk/2.26.43.72-startup-metrics-sink/README.md).
Full predicates, event authority, queue/transport, responses and durable generation
recovery remain open. No Rust behavior or live account operation changed.

### Primary metrics worker reaches a native boundary (2026-11-05)

**Confirmed static, APK43.72:** selected worker14 can enqueue validation diagnostics
and still continue the original event. It awaits a latch (interruption returns
normally after reinterrupt), maps channels and calls JniBridge with event fields.
The return is discarded; native/network delivery remains unproven.
[Constructor, selected worker, wait and limits](apk/2.26.43.72-startup-metrics-native-handoff/README.md).
Native serialization/transport, alternate sink, durability and generation ownership
remain open. No Rust behavior or live account operation changed.

### Metrics latch release and activation scheduling (2026-11-05)

**Confirmed static, APK43.72:** static CountDownLatch(1) status is opened by a
separately queuedI9Q selector3, after initialization submissions inALK. The
activation flag is per-instance; initialization native return is discarded.
Count zero is not by itself proof every instance's native setup succeeded.
[Selected activation, release, status and native boundary](apk/2.26.43.72-metrics-readiness-latch/README.md).
Executor failure/order, activation/source authority, native transport and durable
generation recovery remain open. No Rust behavior or live account operation changed.

### Metrics FIFO advances after exceptional completion (2026-11-05)

**Confirmed static, APK43.72:** reverified098/1VR FIFO dispatches the next task
after either task return or catch-all cleanup. Initialization47 precedes release3
on the same queue, but exceptional completion can still advance toward latch
release. An open latch therefore remains distinct from native success.
[Full executor links and scoped failure analysis](apk/2.26.43.72-metrics-readiness-latch/README.md#fifo-completion-advances-even-after-an-initialization-exception).
Underlying dispatch failure, middle task, activation authority and native transport
remain open. No Rust behavior or live account operation changed.

### Metrics native carrier and provider boundary (2026-11-05)

**Confirmed static, APK43.72:** exact event/setup Java methods are native. The
selected ARM64 libs.so exposes superpack archive coordinates, not named JNI
dispatchers in its six-entry dynamic-symbol table. Carrier digest and differing
file/virtual offsets are retained; inner code is not yet decoded. Current REA
native providers are unavailable under their configured paths.
[Declarations, carrier metadata, readelf and scope](apk/2.26.43.72-metrics-native-carrier/README.md).
Unpacking, JNI registration, native transport and durable recovery remain open.
No Rust behavior or live account operation changed.

### Superpack loader and bounded XZ reconstruction probe (2026-11-05)

**Confirmed static, APK43.72:** loader/API bodies connect to the packaged
superpack decoder. A digest-bound probe finds140 XZ magic candidates and
reproduces successful bounded decoding of the first12. None starts with ELF;
these outputs are partial payloads, not recovered native libraries.
[Complete loader/API, reproducible probe and limits](apk/2.26.43.72-superpack-unpack-boundary/README.md).
Archive transforms, member identity, JNI registration and native transport/recovery
remain open. No Rust behavior or live account operation changed.

### Superpack decoder usability is a narrow JNI probe (2026-11-05)

**Confirmed static, APK43.72:** decoder member/export bytes pinned. The56-byte
usability export constructs47 77 53 and passes them through one JNI-table call;
it does not exercise archive decompression. Host JNI layout supports setter
identity as an inference, not observed Android execution. Openbox Java wrapper
exposes file-range/reader-pointer boundaries, not a restored container schema.
[Exact assembly, symbols, API and limits](apk/2.26.43.72-superpack-decoder-entry/README.md).
Native registration/transforms, complete inner libraries and metrics transport
remain open. No Rust behavior or live account operation changed.

### Superpack relocated JNI entries mapped (2026-11-05)

**Confirmed static / derived, APK43.72:** selected registration call sites and
RELATIVE-relocated tables connect8 AssetDecompressor and5 Openbox methods to
native image addresses. All13 full signatures match retained DEX declarations;
host JNI layout checks support the registration interpretation. This is not
executed registration or recovered inner native code.
[Reproducible table resolver, relocations and bounded assembly](apk/2.26.43.72-superpack-jni-registration/README.md).
Container transforms, complete libraries and metrics transport/recovery remain
open. No Rust behavior or live account operation changed.

### Superpack carrier loading and decode-core handoff (2026-11-05)

**Confirmed static / derived, APK43.72:** mapped from-so wrapper supplies default
ranges, resolves exact carrier archive symbols and passes end-start to0xa964
under the spo storage branch. Missing symbols/error and cached-buffer paths
remain distinct; the core format and complete recovery are not yet reconstructed.
[Bounded assembly, byte digests, exact Strings and limits](apk/2.26.43.72-superpack-from-so/README.md).
Full transforms, inner libraries and metrics network/durable behavior remain
open. No Rust behavior or live account operation changed.

### Superpack member selection and disk-output boundary (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** core 0xa964 iterates members,
selects ordinal start/exclusive-end (end -1 means exhaustion), writes selected
members and returns their names. The selected writer checks fwrite length but
ignores fclose result; this body has no fsync/rename or prior-file rollback.
[Complete selected core/writer windows, ABI probe and limits](apk/2.26.43.72-superpack-member-output/README.md).
Parser transforms, usable complete libraries and native metrics transport remain
unverified. No Rust behavior or live account operation changed.

### Superpack structured native-member inventory (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** traced header and metadata parser
yields 83 records, including libwhatsapp.so (ordinal 63) and whatsappmerged
(64). Raw/effective type, declared size, opaque second word and offsets retained;
metadata ends at archive offset 0xc73. This is not decoded ELF recovery.
[Digest-bound inspector, full inventory and parser assembly](apk/2.26.43.72-superpack-metadata/README.md).
The spo transform, complete members and native metrics network behavior remain
unverified. No runtime behavior or live account operation changed.

### Superpack spo transform dispatch and complete block framing (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** relocated spo descriptor and
42-byte block headers yield 213 framed records ending exactly at archive length.
140 kind-1 blocks start with XZ magic; 73 kind-42 blocks call a separate decoder
0x218bc. The old XZ-only probe did not reconstruct complete native libraries.
[Digest-bound descriptor/block inspector and full reader/worker windows](apk/2.26.43.72-superpack-transform-blocks/README.md).
Decoded stitching, complete ELF recovery and metrics network behavior remain
unverified. No runtime behavior or live account operation changed.

### Type-42 native decoder bounded emulation (2026-11-05)

**Observed in Unicorn emulator / static entry mapping, APK 2.26.43.72:** selected
type-42 blocks 0, 168 and 201 return declared output lengths; ordinal 0 reproduced
byte-for-byte and 201 agreed across two memory-helper implementations. This
executes selected target instructions in memory, not an Android app or account.
[Digest-bound harness, results and execution limits](apk/2.26.43.72-superpack-kind42-emulation/README.md).
Candidates are not ELF images; member stitching/checksums, other type-42 blocks
and native metrics network behavior remain open. No Rust behavior/live traffic changed.

### Full Superpack block decode and so-driver boundary (2026-11-05)

**Observed offline / emulated plus static mapping, APK 2.26.43.72:** all 213
framed blocks produce declared lengths (140 XZ EOF-complete, 73 bounded ARM64
emulations), totaling 81747344 bytes. None starts with ELF magic; complete native
libraries remain unreconstructed. The registered so-driver resolves to 0x1af20,
then helper 0x1adb8 for the additional member-output stage.
[Full output manifest, 70 new emulation logs, driver assembly and limits](apk/2.26.43.72-superpack-all-blocks/README.md).
Native metrics network/persistence behavior remains open. No Rust/live change.

### All ARM64 packed native members reconstructed and metadata-checked (2026-11-05)

**Observed bounded emulation / independent offline checks, APK 2.26.43.72:** all
83 members reconstructed, 65766648 bytes. Every size and XXH64(seed=0) matches
pinned metadata; readelf layout checks pass. First 37 also match the alternative
sort-helper run. Recovered whatsapp/whatsappmerged are available for static JNI
and metrics analysis, not proven Android execution or wire correctness.
[Harness, full digest manifest, verification and limits](apk/2.26.43.72-superpack-reconstructed-members/README.md).
Native metrics behavior and broader network audit remain open. No Rust/live change.

### Recovered native metrics JNI event routing (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** exact relocated JNI tuples map
event/setup dispatch into recovered libwhatsapp.so. Event entry narrows longs,
keeps channel-dependent psId handling, visits typed fields, conditionally hands
to a state-based local manager and returns literal zero. Local append/scheduling
is not delivery or native persistence proof.
[Tuple resolver, complete selected native bodies, outlined flow and limits](apk/2.26.43.72-metrics-native-event-routing/README.md).
Setup initialization, flush/storage/transport and replies remain open. No Rust/live change.

### Native metrics pending drain, time enrichment and selected persistence (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** queued thunk reaches native pending
worker, which removes array index zero before processing. Channels 2/3 bucket a
clock-derived field 47 by 3600. Selected snapshot/file helpers reach fwrite and
fflush; flush return is ignored by the selected snapshot caller. Guard/caller
reachability, full durable format and actual transport remain unverified.
[Complete selected bodies, clock/thunk resolver and scoped limits](apk/2.26.43.72-metrics-native-drain-persistence/README.md).
No queue removal/stdio flush is promoted to delivery or crash durability. No Rust/live change.

### Native metrics metadata path, modes and recovery envelope (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** selected metadata path appends
/wam-metadata, uses rb recovery/wb writing, and rewinds before snapshot writes.
Reader checks 1692-byte MTD/version-4 envelope, matching writer header bytes;
a normal worker save passes the snapshot's zero guard. Termination and complete
validation/durability/transport semantics remain open.
[Mode/branch resolver, paired read/write evidence and limits](apk/2.26.43.72-metrics-native-metadata-file/README.md).
Local metrics metadata is not account/message replay authority. No Rust/live change.

### Native metrics snapshot range validation and replacement (2026-11-05)

**Confirmed static / observed synthetic emulation, APK 2.26.43.72:** local
validator checks four groups, with A/B nonnegative and C/D in 1..65535. Later
checks win within a group; the first failing group stops validation. All 43
bounded native fixtures pass. Recovery replaces rejected local snapshot state;
this is not server/account/message replay authority.
[Native bodies, exact ranges, synthetic harness/results and limits](apk/2.26.43.72-metrics-native-snapshot-validation/README.md).
Error objects, full manager lifecycle and event transport remain open. No Rust/live change.

### Native metrics integer payload encoding and selected field framing (2026-11-05)

**Confirmed static / observed synthetic emulation, APK 2.26.43.72:** native
integer helpers choose signed 1/2/4/8-byte little-endian payloads; scalar 0/1 use
special tags without bytes. All 42 boundary fixtures match independent expected
bytes/tags. Worker-to-field-encoder calls and selected header packing retained;
full field/event framing and send/reply behavior remain incomplete.
[Native bodies, digest-bound fixture harness and scope](apk/2.26.43.72-metrics-native-integer-encoding/README.md).
No new Rust telemetry or inferred mandatory request. No runtime/live change.

### Native metrics HTTP response ordering and task outcomes (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** native buffer sender constructs
HTTP-body request objects. Its response worker reads URL status and accepts the
ordinary 200/no-error branch; error/endpoint fallback precedes distinct 401 and
other failures. Regular task labels explicitly distinguish success, retryable,
non-retryable and failure; callback ownership/deletion remains incomplete.
[Complete selected transport bodies, labels/thunk resolver and scoped rules](apk/2.26.43.72-metrics-native-transport-responses/README.md).
These rules are not universal account/login retry authority. No Rust/live change.

### Native metrics request framing (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** imported POST method, five
header sources, four multipart sections and exact boundary/header bytes retained.
Host comes from selector 0x1a; access_token from selector 0x17. Endpoint index
zero is removed before construction. Credential/provider decoding, metadata
semantics and complete transport ownership remain open.
[Digest-bound literal/relocation resolver, helper bodies and limits](apk/2.26.43.72-metrics-native-request-builder/README.md).
No Rust runtime or live traffic changed; no mandatory telemetry or ban guarantee.

### Native metrics credential lifecycle (2026-11-05)

**Confirmed static / observed synthetic emulation, APK 2.26.43.72:** credential
admission checks use count/clock, consumes a use before construction, and can
promote a spare. Constructor raw lifetime increment172800/default count64,
nonnegative setter and refresh guards have17 bounded native fixture cases.
Mode-zero count180 override, transient cleanup and raw result switch retained
statically. These are metrics-local counters/results, not server/account limits.
[Native bodies, pinned resolver, fixtures and unresolved provider/crypto scope](apk/2.26.43.72-metrics-native-credential-lifecycle/README.md).
No Rust runtime or live operations changed; selector7 and completion binding
remain open.

### Metrics credential IQ producer and native callback mapping (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** Java selector7 constructs
privatestats/sign_credential version1 with diagnostic32-byte blinded_credential bounds;
callback registration and32000ms delay precede dispatch. Selected not-ready
wrapper returns false without Java BsS. Hyg parses required integral t and
signed/public-key data plus optional config_id. Both selected failure methods
map through JNI selector4 to native raw1, not the Java diagnostic value3.
[Producer, parsing/queue evidence, native bridges and binding limits](apk/2.26.43.72-metrics-credential-iq/README.md).
Native method-cache initialization, crypto, full error/timeout dispatch and
operational Rust support remain open. No Rust runtime or live request added.

### Credential IQ timeout and malformed-response ownership (2026-11-05)

**Confirmed static / derived, APK43.72:** Hyg implements only0lG, so incoming
responses take ordinary handler2/claim-before-parse routing. Timeout handler4
removes queued/sent callback ownership and schedules BsS via8bj21, reaching
null-data/native result1. Required-field parse errors can consume callback
ownership without invoking that native completion; broader recovery is unknown.
[Callback-specific interface proof, shared dispatcher trace and scoped risk](apk/2.26.43.72-metrics-credential-iq/README.md#credential-callback-timeout-ownership-and-schema-failure-2026-11-05).
No Rust/live change or automatic retry authority; full generation/recovery
behavior remains unproven.

### Outbound metrics JNI cache binding closed statically (2026-11-05)

**Confirmed static / observed bounded synthetic emulation, APK43.72:**
JNI_OnLoad resolves JniBridge.jnidispatchIO(ILjava/lang/Object;)J at index0
into cache0xc467f0, consumed by the outbound static-long bridge. Five native-loop
fixtures verify success and early lookup failures. Earlier slots may remain
nonnull while the all-lookup flag is false; runtime initialization is unobserved.
[ELF symbol/tables, indexed-store proof, native fixtures and limits](apk/2.26.43.72-metrics-jni-method-cache/README.md).
This supersedes the prior static cache-binding unknown. Whole-loader recovery,
crypto and runtime availability remain open. No Rust/live change.

### Native metrics credential persistence (2026-11-05)

**Confirmed static / observed synthetic admission, APK43.72:** paired mode-
suffixed private credential file uses version2/25-byte header and three data
segments.23 bounded native header cases match independent parsing. Loader
overrides constructor defaults with saved expiry/count; header admission permits
zero lengths/trailing bytes and is not cryptographic validation. Save checks
write/close/count but uses wb; atomic durability remains unproven.
[Paired native bodies, raw schema, fixtures and failure-branch limits](apk/2.26.43.72-metrics-native-credential-file/README.md).
Observer callback is persistence, not endpoint production. Endpoints, full load
scheduling, crypto and runtime behavior remain open. No Rust/live change.

### Native metrics endpoint production and finite-list fallback (2026-11-05)

**Confirmed static / observed bounded synthetic append/store, APK43.72:**
scheduler replaces one task's URL list with accepted provider entries in order
then native selector22 fallback. JNI index26 binds Java selector0 provider; its
selected prefix/suffix decode to https:// and /deidentified_telemetry. Seven
native cases verify append/store ordering with controlled URL results. Sender
consumes index0 before construction; selected response fallback reuses the list.
[Producer conditions, native bodies, fixtures and remaining scope](apk/2.26.43.72-metrics-native-endpoints/README.md).
Actual dynamic/default endpoint values, generic URL policy, full retry timing
and generation fencing remain open. No Rust runtime or live operation added.

### Native request analytics tags and metadata sources (2026-11-05)

**Confirmed static / observed synthetic branch selection, APK43.72:** selected
sender purpose1 maps to prefetch and category4 to analytics; empty surface/
application fields omit.24 bounded cases verify native enum/pair selection
with append/zero-format adapters, not full JSON serialization. Separate
meta_data rt depends on channel3; optional config_id derives from credential
data, not Host. Raw clock/mode semantics and actual product remain unknown.
[Descriptor, tables, formatter scope, metadata provenance and fixtures](apk/2.26.43.72-metrics-native-request-tags/README.md).
No Rust runtime, invented telemetry or live requests added. General formatting,
selector plaintext and crypto remain open.

### Complete selected native analytics serialization (2026-11-05)

**Observed bounded synthetic emulation, APK43.72:**19 additional cases run the
complete native tag builder/pair/retry code with explicit allocation/strlen and
host snprintf adapters reading native va_list. Exact bytes match independent
expected JSON or an intentional invalid-quote fixture. Retry text truncates to
four characters; raw object insertion and no quote escaping are corroborated.
[Full-builder harness, exact outputs, ABI windows and limits](apk/2.26.43.72-metrics-native-request-tags/README.md#complete-selected-serialization-fixtures-2026-11-05).
This extends the earlier24 branch-only cases. No Android libc, real product,
allocator-failure behavior, live traffic or Rust telemetry implementation proved.

### Native metrics connection-status repeating timer (2026-11-05)

**Confirmed static / observed bounded synthetic execution, APK43.72:** body
72ab10 is connection-status handling, correcting a provisional load-completion
label. Ready0 schedules tasks and starts interval30 timer; unavailable1 cancels
then signals error. Fifteen native arm/event/cancel/callback fixtures verify
repeating/cancellation and readiness gates with explicit queue/mutex adapters.
Not-ready callback does not itself stop generic timer repetition.
[Native bodies, corrected scope, fixtures and concurrency limits](apk/2.26.43.72-metrics-native-recovery-timer/README.md).
Rust IQ review did not establish a universal from rule; no guessed guard/retry
was added. Status producers/generation/concurrency and live behavior remain open.

### Native metrics task cancellation and destructor ordering (2026-11-05)

**Confirmed static / observed bounded synthetic execution, APK43.72:**
cancel helper issues handle cancels and optional table+0x28 callbacks before
clearing active tasks/provider/network. Endpoint map is separate; absent-network
path does not clear busy byte. Destructor clears callback table first, so helper
does not enqueue those callbacks during teardown.12 native cases verify exact
synthetic cancel arguments, queued task keys and cleanup/invariant branches.
[Native bodies, direct callers, fixtures and delivery/ownership limits](apk/2.26.43.72-metrics-native-task-cancellation/README.md).
Real cancellation, queued callback completion and durable replay remain open.
No Rust runtime or live operations changed.

### Native regular drain status versus task outcome (2026-11-05)

**Confirmed static / observed bounded synthetic execution, APK43.72:** helper
7273a8 ends active drain once, optionally creating local error3; it is not
backoff or persistent buffer mutation. Fourteen native helper cases verify
flag/argument/optional-table/repeated-call behavior. Task outcomes separately
retain retryable/failure entries or remove success/non-retryable entries.
[Start/end bodies, helper fixtures and ownership distinctions](apk/2.26.43.72-metrics-native-drain-outcomes/README.md).
Upper callback/network job binding, backoff, generation and durable ownership
remain open. No Rust runtime or live operation changed.

### Regular metrics w:stats IQ and diagnostic-bound correction (2026-11-05)

**Confirmed static, APK43.72:** regular native job calls Java selector6, builds
iq/set/xmlns=w:stats/add binary payload, registers32000ms callback before send
and waits through FfK/T54. Exact result returns true; inhibition/error/wait
failures return false, mapped to native success0/failure3. Callback-backed wait
is distinct from hard end-to-end deadline and durable delivery.
**Correction:**00L.A0C logs AssertionError and returns. A04 bounds are diagnostic,
not hard writer rejection; prior credential exact32-byte guard wording corrected.
[Native/Java contract, callback interfaces and diagnostic source](apk/2.26.43.72-metrics-regular-iq/README.md).
No Rust runtime or live request added. Full buffer/retry/durable ownership
remains open.

## Metrics subcode handling (2026-11-05)

2026-11-05 — **Metrics error subcodes and credential effects traced.**

Pinned native paths recognize3067002/3067003/3067004, release the current
metrics credential and promote a spare or request gated renewal. Failure-reason
suffix parsing is distinct from response-body parsing; diagnostic numbers are
not account restriction codes. [Evidence and limits](apk/2.26.43.72-metrics-error-subcodes/README.md).
Validation:24 synthetic body-parser cases,7 worker integer-slice cases, pinned
resolver and bundle checks. Full retry/durable ownership and server meanings
remain open. No Rust runtime or live traffic changed.

## Regular metrics recovery (2026-11-05)

2026-11-05 — **Regular metrics recovery timer and provider gates traced.**

Matching provider/status0 with idle low bit starts a drain and interval30 timer;
status1 cancels the existing timer and signals drain error. Timer callbacks
require ready and idle low bits. [Evidence and limits](apk/2.26.43.72-metrics-regular-recovery/README.md).
Validation:72 native status-decision and16 timer-decision fixtures plus pinned
window resolver. Weak resolution, actual timers and durable ownership remain
separate; no Rust runtime or live traffic changed.

## Metrics buffer deletion (2026-11-05)

2026-11-05 — **Metrics completion-to-file-removal chain traced.**

Selected regular table now binds owner callbacks: null-error success queues
buffer removal; terminal local error skips that deletion path despite set
removal. Worker remove failures report errno separately. [Evidence and limits](apk/2.26.43.72-metrics-buffer-deletion/README.md).
Validation:4 controlled full-worker cases and pinned construction/function
windows. Downstream bookkeeping/recovery remains open; no real files removed,
Rust runtime change or live traffic.

## Metrics file recovery (2026-11-05)

2026-11-05 — **Metrics buffer loader admission and bulk-delete distinction traced.**

Loader accepts selected prefix/type and WMT versions1/2/3 with positive bounded
payload; restores checksum/state without payload comparison in this body.
Separate directory walk is bulk deletion. [Evidence and limits](apk/2.26.43.72-metrics-buffer-recovery/README.md).
Validation:18 native header cases and pinned loader/helper windows. Remaining
files can pass local gates; automatic restart replay and cleanup policy remain
unproven. No real I/O, Rust runtime change or live traffic.

## Recovered metrics consumer (2026-11-05)

2026-11-05 — **Recovered metrics consumer checksum and task admission traced.**

Version3 payload Adler32 is checked after file loading; versions1/2 skip this
consumer comparison. Mismatch increments a counter and ends the remaining
snapshot walk. Passing outgoing entries reach conditional transport handoff.
[Evidence and limits](apk/2.26.43.72-metrics-recovery-consumer/README.md).
Validation:12 native checksum cases and mismatch-counter slices plus pinned
binding/windows. Startup flag/lifetime and actual replay remain unobserved.
No real I/O, Rust runtime change or live traffic.

## Saved-session retry reservation guard (2026-11-05)

**Confirmed local correction / implemented offline:** `answer` prepared crypto
outside the write transaction, then checked only the saved session and duplicate
answer marker. A concurrently committed delivery/failure/attempt update could
therefore be ignored, or a changed identity pin could be paired with the prepared
old-session result. The fresh-bundle path already guarded the original row and
peer state; both paths now share the same original/age and identity/session guards.

Immediately before reservation, the saved-session path requires the outbox row
to equal the snapshot used for eligibility/preparation, the current identity and
session to equal the captured inputs, the first attempt still to be no older
than60 days, and the answer marker to be absent. Any failed guard returns a
local error and rolls back the write transaction: no retry attempt, reserved
answer marker or prepared ratchet is committed. It does not overwrite newer
state or silently retry. The exact60-day boundary remains eligible.

The retained APK DNP.run delivery exclusion and150.A04 age checks motivate
rechecking eligibility before resend; transactional equality/CAS is our local
concurrency mechanism, not a claimed Android database implementation or new
server policy. Native count/root/addressing rules remain separate.

Regression tests simulate a captured snapshot followed by a committed delivery,
failure or extra attempt, then try the same reservation guard/transaction. They
verify unchanged tables on refusal and success at exactly60 days; another case
crosses the age boundary without a row change. Existing peer-state tests cover
identity/session changes and retained pending-job state, and the complete
saved/fresh retry workflow remains covered. These are deterministic store/guard
simulations, not an end-to-end scheduled thread race or live retry test.

The guard assumes every outbox/pin/session change uses the same ProtocolStore
transaction boundary. Other retry families, higher counts and complete current
APK root normalization remain open. No live account operation occurred.

[Retained current-APK root gate instructions](apk/2.26.43.72-retry-root-gates/README.md)
remain incomplete for a normalization port.

## Refetch query reservation inputs (2026-11-05)

**Confirmed local correction / implemented offline:** the initial retry key-query
reservation previously checked only for an existing answer marker. The eligibility
and peer snapshots came from earlier read transactions, so competing store work
could make them stale before the fetching marker/job were committed. Even though
this branch does not await crypto, other store users can commit between those
transactions. The later refetch-completion guard did not prevent that unnecessary
query reservation.

`reserve_refetch` now reads the current original via its wire-id index within
the same write transaction as the two pending records. It requires the original
outbox row to equal the eligibility snapshot, repeats the strict >60-day age
check and compares identity/session bytes with the captured inputs (including
an absent session). A stale/missing original or changed peer state returns a local
error with no fetching marker or job committed. An existing answer marker still
returns false without another job. Only a successful reservation allows `answer`
to return the key-request node to its caller; no socket write occurs here.

This uses the same local guards as saved-session/fresh-bundle retry reservation.
It preserves the retained APK original-recipient/delivery/age exclusions without
claiming Android uses Rust's transaction mechanism. It does not delete/replace
a session, relax counts, rotate identity or add automatic retry after refusal.

A deterministic regression calls the actual reservation helper after a competing
transaction commits delivery, failure, an extra attempt, identity/session change,
an absent→present session, or deletion of the original. An aged captured original
also refuses at this boundary. Each refusal preserves all checked outbox,
checkpoint, identity and session rows. The unchanged case commits one pending
query; a second query ID with the same answer key returns false and adds no job.
The existing full saved/fresh retry workflow remains covered. These are store
interleaving simulations, not scheduled multithread races or live key requests.

**Observed live:** none. Full current-APK root normalization and higher-count
base-key policy remain unresolved. No new CLI command, route or protocol feature.

## Receipt-carried keys capability boundary (2026-11-05)

**Confirmed / corrected offline:** the normal direct retry driver previously
ignored a receipt's `keys` child despite receipt-key handling being unported.
It could answer under the saved session or reserve a directory query for a
branch whose real key/session/verification policy was not implemented.

The driver now explicitly returns no send/query and records local
`receipt_keys_unsupported` after existing original/recipient/count/acceptance/age
eligibility checks. Any top-level keys child triggers that local limit, including
malformed/empty keys; no key material is adopted or validated here. Repeats keep
one unanswered checkpoint and leave normal answer/query markers absent. A later
supported receipt without keys can still follow the ordinary policy.

[Current-build checked parser and session policy](apk/2.26.43.72-retry-receipt-keys/README.md)
retain F10.A04,163.A02/A05 and the distinct163.A01 legacy/PQ cleanup helper.
The explicit refusal is our capability boundary, **not** a claim that Android
always rejects key-carrying retries. Its conditional AEA/session/registration and
legacy/PQ fallback behavior still requires a full port. Receipt ACK policy and
other consumers of the parser are unchanged.

The full retry workflow regression tests matching/changed registration with
empty, byte-valued and partial structured keys. It asserts no node, unchanged
outbox/session/pin rows, only one unanswered checkpoint, no refetch job and
successful later no-keys resend. These are synthetic offline fixtures, not valid
native key bundles or live compatibility proof.

## Retry parser field admission (2026-11-05)

2026-11-05 — **Retry parser registration/version/defaults corrected.**

Shared parser requires four registration bytes, defaults missing v/count to1
and refuses unsupported version/malformed count. Invalid registration no longer
falls through to a key query. [Evidence and tests](retry-parser-admission.md).
No live request or expanded higher-count/key handling.

Parser and full retry-workflow regressions verify malformed fields preserve all
retry state and default-count receipts can produce the supported count1 answer.
Receipt ACK and complete parser/routing parity remain separate.

## Comparison group retry admission (2026-11-05)

2026-11-05 — **Comparison group retry admission strengthened.**

Before reserving a device or querying keys, the window checks group-chat/message
correlation, supported count, carried-key exclusion and original/duplicate device.
[Evidence, tests and remaining issues](group-retry-admission.md). No live request or group policy
completion claimed.

Group retry write-checkpoint ordering, session commit guard and encryption-child
count remain explicit follow-ups; status routing equality is not inferred.

## Peer snapshot and write settlement correction (2026-11-05)

**Confirmed local correction / implemented offline:** group/status comparison
retry preparation now captures identity and session together in one read snapshot.
Its write transaction requires both to remain unchanged and refuses an existing
retry checkpoint before saving the prepared session/pin. A concurrent ratchet
advance, session appearance/removal or pin change therefore preserves newer
state and creates no replacement reservation. This is local concurrency control,
not a newly inferred native reset/fetch policy.

The prepared peer state and checkpoint commit atomically **before** socket I/O.
The checkpoint is now **reserved**, with the key-query ID as reservation token,
received count and reservation timestamp. A successful `write_node` is followed
by owned settlement to **written**; an error settles to **uncertain**. Settlement
requires the same token/count and still-reserved status. Duplicate/stale settlement
cannot overwrite another state. Existing checkpoints, including legacy written
records, are not re-reserved or migrated into a resend.

A failed/ambiguous write does not roll back the advanced session and is not
retried. A crash after reservation leaves reserved state, also not replayable.
If socket I/O succeeds but settlement fails, the reservation remains held and an
explicit diagnostic reports the socket outcome without claiming a settled send.
The caller no longer claims the recipient can read the message: written proves
only the local write operation completed, not server ACK, delivery or decryption.

Two deterministic store regressions exercise the actual reservation/settlement
helpers: unchanged input; changed identity/session; absent→present and removed
session; existing legacy checkpoint; interruption/reopen after reservation;
stale token/count settlement; duplicate reservation/settlement; successful and
uncertain outcomes across reopen. Assertions preserve all checked tables on
refusal and preserve advanced peer state after either outcome. Socket outcomes
are supplied to the settlement helper; no real socket, process crash, concurrent
thread, crypto or injected storage fault is executed by these tests.

The group retry encryption-child count/root policy, status addressing and full
native base-key/key handling remain open. This fixes the two source issues above
without claiming full group/status retry parity or live acceptance.

[Implementation and prior source findings](group-retry-admission.md).

## Group comparison retry enc count (2026-11-05)

2026-11-05 — **Comparison group retry enc counts corrected from pinned DEX.**

Received count1/2 now reaches root skmsg and pairwise distribution enc children
unchanged; original first sends still omit it. Ciphertext/ID/addressing remain
preserved. [Binding, wire regression and limits](group-retry-count.md).
No live request or full group retry parity claimed.

Checked callback construction/interface/composition and both builder branches
support the count correction; complete root/routing/key policy remains open.

## Nested message key and timestamp correction (2026-11-05)

**Confirmed static / corrected offline:** the shared parser used root receipt id
as RetryRequest.message_id and ignored nested retry.id/t. Current F10.A04 instead
requires nested t/id and builds the original message key from nested id. Complete
[required readers and message-key construction](apk/2.26.43.72-retry-message-correlation/README.md)
close that selected correlation gap.

The parser now returns **nested retry.id** for original-send lookup. It requires
nested t to parse as a signed64-bit integer and applies existing local identifier
bounds to nested id. Root receipt id remains separately validated; no equality
between root/nested IDs is invented. Missing/malformed nested fields produce a
redacted field diagnostic and no actionable retry, before any outbox lookup,
query reservation, crypto or retry checkpoint mutation.

The timestamp is validated, not turned into a new retry/expiry clock. Native
F10 multiplies its parsed seconds by1000; complete downstream timing consumers
remain open. Rust still checks age against the persisted original first attempt.
The model/API adds no timestamp field, and full Java Unicode numeric syntax is
not claimed. Receipt ACK construction is unchanged and retains its own root
correlation semantics; all native parser/ACK exception effects remain unproven.

A parser regression covers independent root/nested IDs, missing/empty/oversized
nested id, missing/nonintegral/overflow t and signed64-bit boundary values.
The full saved/fresh retry workflow verifies root W1 + nested OTHER sends nothing
and leaves all retry tables unchanged. Missing/invalid nested fields do likewise.
Conversely, root ROOT-RECEIPT + nested W1 reaches the intended W1 resend with the
same original wire ID and remains once-only. Existing parser/version/count/key
fixtures now include valid id/t so they continue testing their intended fields.
The group comparison admission fixtures likewise use nested IDs. No live receipt,
key query, resend or ACK was sent.

## Retry original-chat correlation (2026-11-05)

2026-11-05 — **Retry original-chat selection/correlation strengthened.**

Shared parser separates explicit recipient/normalized chat from asking device.
Normal direct retry checks the selected chat against original peer/known LID
before session work or a query. [Evidence and regressions](retry-original-chat.md).
No receipt-derived alias adoption, live request or full native routing claim.

Native effective alias/context binding and typed-reader fallback remain open;
malformed explicit recipients use a strict local supported-path refusal.

2026-11-05 — **Retry ACK dispatch traced; plain-ACK suspicion refuted.**

Current Rust excludes retry ACKs. Selected APK handler separates handled,
unhandled/error487 and parse-exception paths; actual receipt context factory
located. Final Message76 writer binding/async order remain open.
[Evidence and limits](retry-ack-dispatch.md). No runtime policy change or live request.

2026-11-05 — **Unsupported retry versions now receive error487.**

Shared confirmation mirrors supported receipt context for explicit non-1 text
versions after four-byte registration admission. Writer binding and deferred
callback ordering traced. [Evidence, regressions and limits](retry-ack-dispatch.md#unsupported-version-error-response-correction--2026-11-05).
Other retry ACK/error/recovery policy remains incomplete; no live traffic.

2026-11-05 — **Retry schema-error ACK context corrected.**

Confirmed required-field/ASCII numeric errors now receive487 with fresh native
context, omitting recipient/edit. Unsupported-version487 keeps its copied context;
first-error order and local capability refusals remain distinct.
[Evidence, tests and limits](retry-ack-dispatch.md#required-field-parsing-error-responses--2026-11-05). No live traffic.

2026-11-05 — **Retry refetch settlement ownership strengthened.**

Refusal and resend reservation now recheck the pending job snapshot and exact
fetching/query/count checkpoint before consumption. Changed jobs/terminal or
legacy markers remain held without overwriting peer or retry state.
[Race regressions and limits](retry-answers.md#refetch-completion-ownership-guard-2026-11-05). No new send or success ACK.

2026-11-05 — **Reserved direct retries now ACK before resend write.**

Normal receive attaches success ACK only after durable message reservation and
uses root receipt correlation. ACK failure prevents resend submission; any
uncertain work stays held. Queries/refusals/duplicates/group paths remain outside
this ACK authority. [Evidence, tests and limits](retry-ack-dispatch.md#reserved-direct-resend-success-ack--2026-11-05). No live traffic.

2026-11-05 — **Durable retry key queries now ACK first.**

Selected post-ACK job/session-requirement fetch entry traced. Normal wrapper
attaches root receipt ACK only after pending job and fetching marker commit;
ACK/query failure stops without replay. [Evidence, tests and limits](retry-ack-dispatch.md#durable-retry-key-query-ack-ordering--2026-11-05).
No key-request field/backoff change or live traffic; recovery remains incomplete.

2026-11-05 — **Retry key-query manager-to-wire construction audited.**

Classic non-identity/context-disabled IQ baseline matches Rust. Source12 is not
a wire field; identity/PQ/context attrs have separate predicates. Manager entry
expiry is not a proven IQ deadline. [Evidence and remaining gaps](retry-key-query-wire.md).
No production field/backoff/replay change or live traffic.

2026-11-05 — **Retry refetch callbacks now belong to their live connection.**

Only a locally written query can complete through normal receive; result/error
consumes its callback once. Disconnect/restart never resurrects callbacks from
held jobs. [Native cancellation evidence, tests and limits](retry-answers.md#live-refetch-callback-lifetime-2026-11-05).
No guessed timeout, new request, reset or automatic replay; no live traffic.

2026-11-05 — **Outgoing retries now preserve original message timestamp.**

Automatic retry.t uses failed message root t instead of local queue time.
Missing/unusable timestamps refuse without queue writes; held retries do not
reset on changed metadata. [Native chain, regressions and limits](outgoing-retry-receipts.md).
Error-code/key/count progression remains incomplete; no live traffic.

2026-11-05 — **Outgoing retry error domains/eligibility mapped.**

Crypto status, internal failure kind and wire error are distinct; selected
crypto eligibility and guarded refinements recovered. Other producers bypass
the converter. [Evidence and remaining typed-cause work](outgoing-retry-receipts.md#error-domains-and-eligibility-recovered--2026-11-05).
No blanket error default, runtime behavior change or live traffic.

## AB-properties callback timer ordering — 2026-11-05

**Confirmed static / inferred race boundary:** current APK HBE.A00 passes
32000ms to0Sf.A0W/A0A; ready admission registers0ln.A08 before dispatch.
The positive-duration timer starts with queued ownership and is not restarted
byA0C promotion. Expiry handler4/A03 removes callback ownership; HyT selector2
BsS returns without scheduling a retry. Response lookup cancels the tracked
timer before its later response claim, without an independent age comparison.
An overdue timer is not proof that its callback has already expired.
[Complete producer/admission/expiry exports and limits](apk/2.26.43.72-abprops-callback-timer/README.md).
Rust's connection-scoped action owner still lacks per-action expiry. Porting
requires producer-specific timing and competing claims; a universal32s cutoff
or fresh post-write budget is unsupported. No runtime behavior or live traffic
changes in this research chunk.

## Initial AB-properties callback expiry ported — 2026-11-05

**Implemented offline / confirmed static timer:** the normal connection now
expires the initial abt/get/props protocol1 empty-hash callback32s from the
pre-write boundary, including local-write delay. A dedicated select branch
releases callback authority without changing the durable Sent row, replaying,
applying configuration or stopping receive. Response claims cancel the timer;
an overdue timer not yet dispatched remains a competing event, not a separate
reply-age predicate. Other action producers receive no invented default timeout.
[Producer, not-ready placeholder removal, implementation and regressions](apk/2.26.43.72-abprops-callback-timer/README.md).
Paused-clock unit and normal-driver tests cover expiry, reply cancellation,
late replies, write delay, unrelated IQs and continued receive. Rust serializes
expiry after awaited local writes; full Android concurrent scheduling, other
producer durations and configuration scheduling remain open. No live traffic.

## Block-list read/change callback timing corrected — 2026-11-05

**Confirmed static / implemented offline:** current APK1dH.A05 block/unblock
uses20000ms through0Sf.A0W/A0A;1dH.A0M fetch passes0 throughA0Q/A06,
which supplies no positive-duration replacement. The normal Rust loop now
expires block/unblock live callbacks20s from the pre-write boundary, retaining
Sent uncertainty without list/hash mutation, rollback or retry. Fetch receives
no invented20/32s timeout. Reply cancellation and expiry remain competing claims.
[Complete producer, failure callback, tests and limits](apk/2.26.43.72-blocklist-callback-timing/README.md).
Paused-clock tests cover19/20s, delayed write, result/error cancellation and
fetch reply at60s. Normal-driver delayed-write/late-reply tests now cover both
changes and initial AB-properties, continued receive and no resend/projection.
The native change failure callback reaches2KT.CE5(false); its concrete downstream
side effects remain open. Full native scheduler parity and other producers'
deadlines remain unfinished. No live account operation.

## Account-default disappearing timer callback expiry — 2026-11-05

**Confirmed static / implemented offline:** current APK ChangeDMSettingActivity.A0c
set and0oy.A00 get both pass20000ms to0Sf.A0W. Normal live action ownership
now assigns20s to these account-default shapes from the pre-write boundary.
Expiry preserves durable Sent uncertainty without replay, applying the requested
value or local rollback. Result/error cancels that timer. Per-chat E2E timer
messages and usync disappearing_mode children do not inherit it.
[Producer, native success/failure distinction, tests and limits](apk/2.26.43.72-default-timer-callback/README.md).
Paused-clock tests cover19/20s, delayed write, result/error and a separate
AB-properties callback surviving until32s. The normal-driver expiry test also
covers default-timer write delay, late reply, receive ACK and no resend.
Native timeout publishes the saved duration through observers; successful
callback has a distinct preference-write path. Complete Rust default-setting
adoption, alternate-mode gates, native error/observer behavior and exact
concurrent scheduling remain open. No live account operation.

## About reply semantics and UI/network lifetime distinguished — 2026-11-05

**Confirmed static / implemented offline:** current APK FxR.A04 sends writer29,
which storesJAj in a legacy map shared by1rj/1rh/1rD. SetAboutInfo.A5f's32s
UI wait does not itself remove the callback in the selected handler. No32s
network timer is invented for About. Selected reader teardown invokes failure
callbacks then clears this map; full lifecycle/concurrency policy remains open.
Native outer result removes the callback andJAj.A02 ignores the payload, so a
synthetic nested error extension must not turn typed About success into refusal.
Rust now preserves this producer-specific meaning; outer error/code and other
producers' nested refusals remain intact. [Full alias/dispatch evidence and regression limits](apk/2.26.43.72-about-callback-lifetime/README.md).
Paused-clock regression covers empty/nonempty About at60s, independent32s
AB-properties expiry, outer result/error and duplicate claims. Normal-driver
regression verifiesOk/no refusal code, continued receipt ACK and no resend.
Saved About adoption, UI observers and full native lifecycle remain unfinished;
no live account operation or observed server extension response is claimed.

## Profile-photo callback construction and upload result — 2026-11-05

**Confirmed static / implemented offline:** current Hyd.A00's namespace helper
also returns the to=s.whatsapp.net attribute, which Hyd appends; the existing
Rust address is confirmed rather than inferred. 6p1 lookup builds that address
explicitly. Both selected producers pass duration0 through distinctA0W/A0R
wrappers, so no generic20/32s callback timer is assigned. Upload success requires
the first child picture and ignores later error extensions; Rust now preserves
this typed success shape instead of reporting a synthetic nested refusal.
Outer errors and other action refusal parsers remain intact.
[Producer/helpers, separate lookup parser, tests and limits](apk/2.26.43.72-profile-photo-callbacks/README.md).
Tests cover exact4/5 attribute counts, server address, same-connection claims at60s,
outer result/error, duplicate/mislabeled requests and normal receive continuation.
Lookup result adoption, full malformed-result admission, cache/removal/account-center,
target normalization and image transforms remain unfinished. No live traffic or
observed server extension response is claimed.

Validation follow-up: the existing registration handle test searched an entire
serialized attempt for a four-digit fixture PIN, which can occur coincidentally
in its random hex ID or clock metadata. Its diagnostic copy now normalizes only
identity/timestamps before that content assertion. Raw-store secret checks and
the original Debug phone-redaction assertion remain unchanged; no registration
runtime, persisted identity or server request changes.

## Two-step verification callback and diagnostic privacy — 2026-11-05

**Confirmed static / implemented offline:** current1HL.A07 passes32000ms to
0Sf.A0W; FUS.CK4 schedules original supplied values without reading response
payload. Normal two-factor-set callbacks now expire32s from pre-write start,
retaining Sent uncertainty without retry or credential adoption. Typed outer
result ignores nested error extensions; outer error/code remains separate.
[Producer, timeout/success paths, tests and limits](apk/2.26.43.72-two-factor-callback/README.md).
TwoFactorChange's derived Debug exposed PIN/email; custom Debug now preserves
field presence and redacts values without changing inputs or request bytes.
Paused-clock tests cover31/32s, delayed write, PIN/disable/email-only changes,
result/error/duplicate and unchanged durable rows after expiry. Normal-driver
late-result/receive/no-resend test includes this producer. Native credential
adoption, encryption, email-status job and error/observer behavior remain open;
no live account operation or server extension response is claimed.

## Trusted-contact token callback and execution-time gap — 2026-11-05

**Confirmed static / implemented offline:** current infra/privacytoken
GeneratePrivacyTokenJob.A0L request299 uses32000ms through0Sf.A0W. This is
separate from client-auth request314. Normal trusted-contact callbacks now
expire32s from pre-write start, retaining Sent uncertainty and scheduled state
without retry/window reset. Typed result follows8R8 selector3's captured-data
success rather than generic nested-refusal parsing; outer errors stay separate.
[Full job/callback evidence, tests and important remaining gap](apk/2.26.43.72-privacy-token-callback/README.md).
PN/LID tests cover31/32s, delayed write, result/error/duplicate, checkpoint
preservation, unchanged same-window scheduling and same-namespace exclusions.
Normal-driver late-reply/receive/no-resend test includes the actual scheduler.
**Remaining confirmed gap:** the APK reloads scheduled state and checks the
execution-time threshold before constructing the request; Rust freezes t at
queue time. Queued supersession/coalescing, effective expiry/clock, success-store
and native job retry policy need a separate port. Sent rows must not be rewritten
or replayed as a workaround. No live account operation.

## Trusted-contact queued timestamp reload and age floor — 2026-11-05

**Confirmed static / implemented offline:**87x.A04 advances scheduled state before
pending-set admission; the job reloads the stored timestamp at execution. Queued
Rust token actions now coalesce by user/schedule owner and refresh their stored
wire from current state in the same transaction that marks Sent. Reserved bytes,
IDs and send facts are never rewritten or replayed. Legacy duplicate ownership
is recovered without resurrecting a current Sent/Ok/Error action.
[Native age-floor proof, ownership, clock and regressions](apk/2.26.43.72-token-reservation-refresh/README.md).
0hu's max-per-branch then min construction proves a mandatory182-day lower bound.
Reservation defers missing state, below-floor time and superseded rows, recording
optional ActionOutcome.deferred_reason for actions plain/JSON; unrelated drain
and receive continue. Scheduling/reservation use confirmed A04 clock semantics,
without inferred stanza time. Boundary, reopen, corruption rollback, legacy and
normal-driver tests cover these paths. Stricter configured bucket admission,
full native job/pending/retry/counterpart and success-store policy remain open.
No live account operation; the age floor is necessary, not full eligibility.

## Token bucket configuration follow-up — 2026-11-05

**Confirmed static, 2.26.43.72:** selected 08Y.A0Q packaged integer entries
assign 604800 to 865/996 and 4 to 909/997. With these effective values,
0hu expiry would be 21 to under 28 days old, not the Rust 182-day fallback.
This conditional calculation is not a claim about live account configuration.
00E.A0a/A02/A06/A01/A0h show cache, preference/default and guarded MobileConfig
paths; a returned AB property is insufficient to prove the effective getter.
Whole-map final defaults, adapters and complete override precedence remain open.
[Selected entries, complete resolver methods and reproduction](apk/2.26.43.72-token-bucket-configuration/README.md).
No send policy changed; no live traffic. Next work is effective-policy resolution,
then configured scheduling/admission rather than silently hardcoding these entries.

## Token bucket returned-value inspection — 2026-11-05

Existing abprops-query --show reports token_bucket_values: selected 865/909/996/997
signed integers only. Missing keys are absent/inapplicable, null means invalid
last value; duplicates replace earlier values, including valid-to-invalid.
Full hashless replies expose none; delta/hash rules match the existing parser.
Legacy views deserialize with an empty map; saved replies reparse after reopen.
This does not resolve cache/preferences/MobileConfig or apply values to sending.
[Evidence, reused current-APK adapters, tests and unresolved mapping](apk/2.26.43.72-token-bucket-configuration/README.md).

## Token MobileConfig binding and one ready transition — 2026-11-05

**Confirmed static, 2.26.43.72:** concrete 0A8.B8Q indexes 0El's table.
865/909/996/997 all have nonzero SDK specifiers; missing raw AB rows do not
imply an unmapped/zero SDK property. ID66 binds08Y, ID252 binds0A8. A retained
0A8.BPZ branch sets08Y's A08 ready true and resets its SDK cache viaA0t.
One producer is closed, not the complete initialization or override lifecycle.
[Exact specifiers, full methods, derivation and binary-asset boundary](apk/2.26.43.72-token-mobileconfig-binding/README.md).
Effective account integers, parameter-map defaults and configured token policy
remain unported. No send policy changed and no live account traffic occurred.

## Trusted-contact successful issue state — 2026-11-05

Confirmed current 0ht.A0V/8R8.CK4 selected timestamp persistence now accompanies
Rust token action settlement atomically. Captured request time confirms a matching
schedule/nulls real metadata, or advances nonnull real time while protecting a
newer schedule; missing state inserts captured completion, dominating state stays
unchanged. A0W carries nonnegative decoded issue time: known SQL NULL means prior
sent time, raw zero means unissued. Missing/negative projection initializes zero.
Legacy unknown completion history stays distinct.
Existing actions plain/JSON reports optional token_issue_completion without users
or times. No historical Ok replay/backfill; legacy missing metadata is unknown.
[Native source, independent SQL vectors, reorder/reopen/rollback tests and limits](apk/2.26.43.72-token-success-state/README.md).
Configured bucket policy, canonical counterparts and full job/cache lifecycle remain
unfinished. No live account operation or automatic replay is introduced.

## Token namespace selection and existing bot guard —2026-11-05

Confirmed current87x.A00 uses14303 to choose pending/job user, while0ht.A0L
independently normalizes PN storage through0Ws.A0E. Counterpart deletion also
requires15491=false and original==normalized before0Ws.A0K; a mapped PN does
not automatically authorize deletion of its original row. Lookup readiness,
own-user/ranked-row authority and effective flags remain unported.
Current getType26/27 are bot user/device identities (@bot), explaining87x.A04's
existing exclusion. Rust's PN/LID allowlist already excludes them; a regression
proves no action/state mutation and retains PN/LID positive controls.
[Current methods, conditional namespace table and mapping limits](apk/2.26.43.72-token-namespace-selection/README.md).
No automatic alias merge/deletion or live account operation was introduced.

## Contact-check preparation and reservation atomicity —2026-11-05

Confirmed local fault: separate action/correlation commits could leave a sendable
orphan on disk. queue_check now prepares both atomically. Real full-disk and
uncertain-fsync tests verify neither/both consistency after reopen. Existing
Queued requests lacking correlation defer with contact_check_correlation_missing;
invalid/mismatched correlation refuses locally, and unrelated eligible traffic
drain continues. No automatic correlation repair, Sent reset or replay.
[Fault evidence, native caller-dependent usync timers and tests](apk/2.26.43.72-contact-queue-atomicity/README.md).
Current0pD.A07 receives a caller timeout: selected HxV lightweight call is32s,
while another0p7 call is128s. Rust contact-check's exact factory/deadline remains
unported; no universal usync timer is inferred. No live traffic was sent.

## Contact protocol metadata/backoff obligation —2026-11-05

Confirmed current0pD.CK4/A01 retains contact protocol success/integrity/error,
refresh and backoff separately from per-user in/out/invalid. Error backoff uses
seconds*1000 capped at3600000ms, with a missing-attribute default before the cap.
Selected observers persist positive deadlines using08q.A00;503 also writes
contact-executor global_backoff_time, while429 has an additional guarded scope.
Regular and lightweight query observers write different per-scope preferences.
[Bindings, full selected methods, strict deadline gate and remaining port requirements](apk/2.26.43.72-contact-protocol-backoff/README.md).
Rust still lacks full metadata/wait adoption. Outer IQ success alone is not contact
protocol success. No new wait, retry or account-state mutation is implemented here.

## Single-contact projection bound to original request —2026-11-05

A reproduced local defect let a successful contact-check reply overwrite status
for any returned phone. Projection now reloads/validates saved request provenance
and applies only a uniquely matching requested number. Unrelated/duplicate/
missing-origin/malformed cases preserve contact data and record a content-free
ActionOutcome.contact_projection through existing actions inspection. Status,
projection diagnostic and once-only marker commit atomically; settled wire/reply/
sequence and historical consumed markers stay unchanged. No query is replayed.
[Red regression, disk-fault/reopen tests, inspection and native-policy limits](contact-projection-correlation.md).
Current native protocol integrity/backoff and exact factory/deadline remain open.

## Contact factory flag comparison —2026-11-05

Current HxV lightweight A0B leavesHck.A0F false; H6w maps it toA0J and0pD
uses that flag to emit disappearing_mode. The regular A0C/common factory sets
it true alongside additional picture/business/username flags. Rust's fixed
contact/lid/disappearing_mode request is therefore not exact lightweight parity.
Both selected native phone-query dispatch paths are32s, but other usync callers
use128s; equal visible mode/context does not bind a complete factory policy.
[Full constructors, field transfer, writer and caller comparison](apk/2.26.43.72-contact-factory-selection/README.md).
Factory/configuration/observer/backoff integration remains open. No request or
callback policy changed and no live account operation occurred.

## Usync outer-refusal numeric metadata —2026-11-05

Confirmed current0pD.Bvj uses absent code0/backoff-1, unlike protocol-result
A01's absent-backoff default before its cap. Present values use signed JVM
parse and wrapping seconds*1000 capped at3600000ms. Durable usync get-action
outer errors now save optional ActionOutcome.usync_refusal with presence flags
and fixed numeric parse failures, exposed through existing actions plain/JSON.
Metadata survives projection/reopen; duplicate replies do not rewrite it.
[Full callback, independent JVM vectors, tests and scope limits](apk/2.26.43.72-usync-outer-refusal/README.md).
This does not enforce a wait, select an observer/deadline or replay historical
callbacks. Factory/read/reset/clock adoption remains unfinished; no live traffic.


## Contact backoff gates and reset callers — 2026-11-05

**Confirmed static**, APK2.26.43.72: the selected contact executor filters
protocols individually; its FULL contact/sidelist wait-key selection depends on
property37559. Exact0pQ.A02 direct callers are registration contact-sync start
and IBi selector16 scheduled by a contact `sync` notification. This event-bound
reset does not clear every wait key and is not generic retry permission.
[Evidence, delay arithmetic, branch scope and remaining port requirements](apk/2.26.43.72-contact-backoff-gates/README.md).
Rust eligibility/reset behavior is unchanged; query reachability, clock authority,
authenticated notification/reset adoption and effective configuration remain open.
No live account operations or server acceptance claims.


## Contact protocol refusal inspection — 2026-11-05

**Confirmed static and locally tested**, APK2.26.43.72: an outer usync IQ result
can carry a separate contact-protocol refusal. Actions now durably expose
`contact_protocol_refusal` with native backoff-first parsing, absentcode-1 and
absentbackoff3600000ms. Outer-IQ refusal defaults remain separate. Numeric
parse failures are fixed/redacted diagnostics, not guessed waits. Existing
plain/JSON actions inspection retains the field across projection and reopen;
duplicates cannot settle again. Legacy absent fields remain unknown.
[Source readers, JVM oracle, tests and unported adoption/wait policy](apk/2.26.43.72-contact-protocol-refusal/README.md).
Outer action status/per-user projection remain separate; no automatic retry,
wait enforcement, live operations or complete native integrity-policy claim.


## Pinned contact backoff clock and NTP scheduling — 2026-11-05

**Confirmed static**, APK2.26.43.72: contact backoffs use NTP-preferred08q.A00,
whereas token scheduling uses08q.A04. Constructor NTP cache admission is less
than one day; selected in-process getters do not expire it. Newly traced
property170 selects WorkManager12h/6h-flex versus12h elapsed alarm scheduling;
both setup paths also invoke acquisition immediately. A separate six-hour
process-static recent-success guard may skip it. Property2225 conditionally
controls the dual one-day adoption rejection; neither effective value is known.
[Source exports, field/caller census, search limits and port obligations](apk/2.26.43.72-contact-backoff-clock/README.md).
No Rust clock/eligibility, NTP acquisition, retry/reset or live account changes.


## Client keepalive error response correction — 2026-11-05

**Confirmed static / locally corrected**, APK2.26.43.72:1xa.Bvj routes a ping
IQ error through the sameFea(24) manager response as validated success. Rust
previously ended receive as nonrecoverable keepalive refused. A matched live
ping error now clears only that ping and receive continues with a fixed
redacted diagnostic. Duplicate old errors do not consume successor pings;
non-replies/late replies and unowned IQs retain existing admission. No clock,
account, request retry or restriction-reset authority is added.
[Callback binding, red/green regression and open cadence/schema gaps](apk/2.26.43.72-client-ping-errors/README.md).
Full native keepalive scheduling/configuration and successful-result schema
remain unported; no live account operation or causal ban-prevention claim.


## Client pong success schema and consumed callback — 2026-11-05

**Confirmed static / corrected locally**, APK2.26.43.72:1xa.CK4 requires parsed
from, original-request ID equality, bounded signed-long t and exactresult.
Malformed responses throw recoverableLIL before the manager response, after
one-shot IQ callback removal. Rust now separates callback claim from ping timer:
malformedresult consumes the claim, leaves the original timeout active and
continues receive; a later same-ID validresult/error cannot revive it. Fixed
field diagnostics contain no supplied values. Error-response behavior remains
separate. Existing Rust JID text bounds are a local supported-path limit.
[Reader/exception/ownership proof, regressions and remaining gaps](apk/2.26.43.72-client-pong-schema/README.md).
No clock adoption, replay, new request, cadence/timeout change or live traffic.


## Client ping interval restarts on response — 2026-11-05

**Confirmed static / locally corrected**, APK2.26.43.72:1A7.A05 schedules the
next periodic ping from its accepted response. Rust's fixed shared tick could
send again1second after a response delayed19seconds. Separate maintenance and
next-ping deadlines now restart only the ping interval; queued work still drains
at its original local maintenance deadline. Invalid/duplicate/unowned replies
do not restart it. Red/green actor regression covers bothresult anderror.
[Evidence, packaged15s/240s/32s entries and resolver limits](apk/2.26.43.72-client-ping-scheduling/README.md).
Effective native durations/configuration/mode remain unported; existing local
values, wire requests, uncertainty guards and no-live-operation boundary remain.


## Client ping first-access configuration precedence — 2026-11-05

**Confirmed static**, APK2.26.43.72: ping getters use00G.A02 with
firstAccessValue=true, selectingA07 rather than the token getters' ordinary
cache. Selected readiness resetA0t replacesA0d, notA07. The SDK comparison
branch clones exposure options while preserving first-access semantics; a
successful legacy read can populate the winner before SDKputIfAbsent. Direct
SDK selection remains distinct. No effective value is inferred from raw AB
rows or missing specifiers, and cache owner/replacement scope remains open.
[Complete options/getters, field census and conditional precedence proof](apk/2.26.43.72-client-ping-first-access/README.md).
Research only; native effective timing/configuration remains unported. No live,
request, timer, retry/reset, account or clock behavior changed.

### 2026-11-05: final compiled ping defaults

**Confirmed static / derived static, APK 2.26.43.72:** whole integer-builder
argument-flow analysis of 08Y.<init>, A0B, A0I and A0Q finds 4158 integer puts,
3585 distinct keys, and exactly one write each for 15103=32000, 15183=240000
and 15270=15000. The final snapshot binds to this.A02; no later target write
replaces those entries. This closes the selected-assignment-only caveat for
these three keys. **Unknown/unported:** effective account values and configuration
owner lifetime. No runtime timer, retry, replay or live-account behavior changes.
[Reproducible bounded proof and limits](apk/2.26.43.72-client-ping-final-defaults/README.md).

### 2026-11-05: ping channel lifecycle and scheduler-mode authority

**Confirmed static, APK 2.26.43.72:** 0YB.A01 -> 2JJ selector 2 binds startup.
Same connected callback identity returns without resetting; new accepted channel
resets the ping counter/outstanding timestamp/timeout flag and schedules the
initial interval. Counter reset does not reset the configuration first-access
cache. Scheduler mode derives from push-registration preference presence and
logins_with_messages < 3, then effective property 6493 for Runnable selection.
**Unknown/unported:** actual preference/config values, complete owner lifetime,
on-demand/channel-replacement callback interactions and receiver failure paths.
[Source, exact field census and transition table](apk/2.26.43.72-client-ping-channel-lifecycle/README.md).
No runtime policy or live-account behavior changed.

### 2026-11-05: normal pings and ACK-confirmation callbacks differ

**Confirmed static, APK 2.26.43.72:** A08 uses normal A04 pending guards; direct
A0B callers also include offline-resume and coroutine pre-ACK confirmation.
Bound Fef14 and Feq9 continuations request A0J(marker) only after 1xa's validated
success, not IQ error or delivery failure. Enqueued manager-liveness work and
synchronous success completion have no join establishing a fixed execution order.
A0A additionally serves arbitrary calling/signaling callback families; shared
request shape does not establish shared response policy. **Still open:** registry
cleanup across channel replacement, coroutine cancellation and complete durable
replay/retirement parity. No Rust timing, retry, replay or account behavior changed.
[Producer census, constructor bindings and limitations](apk/2.26.43.72-client-ping-producer-families/README.md).

### 2026-11-05: ping registry reset and already-claimed callbacks

**Confirmed static, APK 2.26.43.72:** 1xZ.BVv selects gated alternate dispatch;
nonnull success continuations use 1xa's serial path. Reset A0F posts handler 3,
which snapshots/clears awaiting-response map A0H and schedules delivery failure.
Queued A0I and already-claimed worker ownership are distinct; no synchronous
all-callback cancellation follows. **Verified locally:** stale result/error from
a prior closed link cannot settle or extend the restarted link's ping after store
reopen. **Open:** complete queued disposal, native executor/registration ordering
and claimed-worker cancellation. No production timing/retry policy changed.
[Evidence, regression and limits](apk/2.26.43.72-client-ping-registry-lifecycle/README.md).

### 2026-11-05: disconnect cancels a scoped retriable-IQ job map

**Confirmed static, APK 2.26.43.72:** 0Sf.A0E holds delayed T56/TKl retry jobs,
not queued IQ callback ownership. Disconnect requests job cancellation; selected
coroutine cancellation forwards original delivery failure. Scheduling requires
remaining budget and callback CmQ permission; errors additionally require allowed
500..599 code excluding the retained eight-code set. Direct application caller
is a feature-gated status-privacy job, not every IQ. **Unported/unknown:** caller
callback eligibility, full job-queue/cancellation ordering and durable recovery.
[Complete callback/budget bodies, caller census and boundaries](apk/2.26.43.72-iq-retry-job-cancellation/README.md).
No runtime retry, replay, timer, account or persistence policy changed.

### 2026-11-05: status-privacy callback distinguishes root and nested errors

**Confirmed static, APK 2.26.43.72:** original 8RJ17 permits bounded wrapper retry,
but T56 reads root IQ code/backoff while the original reads nested error code.
Nested 500 alone reaches original code/future handling and the job's exception;
404 logs no settings without success adoption. Successful parser requires privacy
child/type before conditional audience persistence. **Open:** effective flags,
store transactions, scheduler limits and durable recovery. Rust has no matching
status-audience fetch/adoption path; no runtime retry or settings change added.
[Exact callback binding, branch table and limits](apk/2.26.43.72-status-privacy-fetch-callback/README.md).

### 2026-11-05: preserve About refusal callback codes

**Confirmed static / implemented offline, APK 2.26.43.72:** the ordinary About
error path processes signed Java Integer codes for each direct error child in
order, stopping on malformed present input. Existing generic u16 metadata lost
signed/large/subsequent values. Optional ActionOutcome.about_refusal now retains
the codes and malformed boundary atomically with settlement, exposed by actions
plain/JSON. Legacy absence remains unknown. Duplicate/reopen/no-replay and JVM
oracle tests cover the correction. No retry, cooldown or saved About adoption.
[Exact source, oracle and limits](apk/2.26.43.72-about-error-codes/README.md).

### 2026-11-05: current prekey error backoff has scoped state

**Confirmed static, APK 2.26.43.72:** non-PQ error selector 76 moves affected
existing in-flight entries to pending only for 500..599. A01 mode and A00 posted
flag control scheduling; the 10/610 generator starts at zero delay. Delayed drain
clears posted flag but retains mode; explicit reset differs. Complete normal
admission does not establish a blanket stop of all key queries. **Unknown:**
network-to-listener attribution, external reset authority and durable job/write
recovery. No Rust retry, admission or reservation policy changed.
[Exact anchors, source transitions and delay derivation](apk/2.26.43.72-prekey-error-backoff/README.md).

### 2026-11-05: current key-fetch error attribution and signed codes

**Confirmed static / implemented offline, APK 2.26.43.72:** writer binds Dxh to
captured devices; outer errors map to that set and per-user errors are grouped
only while iterating original requested devices. A1S/A0e constructs event 76
matching the backoff listener. Signed Java integer code grammar is retained.
DirectoryInspection.server_error_code widens from Option<u16> to Option<i32>,
preserving valid negative/large/Unicode codes after existing unique-context guards.
JVM-case regressions cover root/user attribution, redaction, reopen and no refresh
or resend. Other inspectors/generic action fields remain unchanged. Full native
registration, duplicate/missing-device and durable retry ownership remain open.
[Source, API consequence and limits](apk/2.26.43.72-prekey-wire-error-attribution/README.md).

### 2026-11-05: non-reply IQ cannot consume a pending key page

**Implemented offline, confirmed APK43.72 outer dispatch:** key-batch settle now
returns false without store mutation for IQs lacking result/error type, even with
matching ID and error child. Previously reusable settlement could terminally
classify such input and lose the later actual reply. Normal group-driver already
filtered these types. Red/green regression covers state equality, blocked reserve,
reopen, later valid settlement, duplicate refusal and complete local assembly.
No new callback authority, key adoption, timeout or automatic retry is added.
[Evidence, scope and unresolved native lifetime](device-key-batches.md#key-batch-reply-admission-correction-2026-11-05).

### Ordinary message ACK forwarding (2026-11-05)

**Confirmed static, APK43.72:** selected ordinary message success and refusal
event paths both jump back to generic A1D with the original node/Eoh key.
This closes those forwarding edges into the previously traced concrete callback
bridge. Error459 and malformed/early paths remain separately qualified. The
destination rewrite helper inspected here is FBID-migration-specific; Efa's
null type default does not prove the out-of-band producer leaves type unset.
[Exact offsets, retained exports and remaining producer gap](apk/2.26.43.72-message-ack-forwarding/README.md).
No runtime guard, replay policy or live account operation changed.

### Sender-key requirement/session boundary (2026-11-05)

**Confirmed static, APK43.72:** the sender-key requirement inherits conditional
recipient selection and pairwise session readiness. A selected hash mismatch
retains the original candidate set instead of applying later sent-key filtering.
The latched A07/A08 booleans come from session/PQ policy, not distribution ACKs.
The exact native ordinary direct-SKDM producer is still unlocated; the status
requester and include flag do not establish it.
[Methods, offsets, search scope and unknowns](apk/2.26.43.72-sender-key-session-requirement/README.md).
No Rust runtime, replay or live account behavior changed.

### Membership guard across distribution ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** after reserving a reaction's
pairwise distributions and group ciphertext, a saved membership change while
waiting for a distribution ACK did not stop the next distribution or final
group stanza. Query/key preparation had a frozen-membership digest guard, but
the ACK-driven continuation omitted it.

Every continuation now compares the saved group snapshot digest before returning
another stanza, including the initial distribution return and final group return.
A mismatch persists the existing `membership_changed` preparation block and
returns no dependent ciphertext. Already exposed keys, pairwise sessions,
original encrypted attempt, uncertainty and reservations stay held; no replay,
rollback, fresh IDs, automatic membership query or rotation is authorized.

The regression `membership_change_during_distribution_cannot_release_more_ciphertext`
prepares two real pairwise distributions, applies the real remove-notification
snapshot handler while either the first or second ACK is pending, then supplies
the matching successful ACK. Old code failed by releasing stale ciphertext.
The corrected path stops before the remaining distribution or final group send,
preserves all crypto/outbox state and all non-preparation checkpoints, and stays
nonselectable after reopen. Existing unchanged-membership crypto/decrypt tests
continue to cover successful release.

**APK corroboration, not scheduler parity:** APK43.72's session requirement
A01 01de..0206 distinguishes a changed participant hash from equal-hash
sent-key filtering, as retained in
[the requirement bundle](apk/2.26.43.72-sender-key-session-requirement/README.md).
This fix extends the existing local frozen-snapshot policy; native per-ACK
revalidation or identical blocking semantics have not been proven. The check
is a saved-state observation before returning a node, not an atomic transaction
with a remote socket write; the entire snapshot digest also conservatively
includes non-membership group fields under the existing policy. Native direct
SKDM producer/address ownership remains unresolved. No live account operation.

### Authority guard across distribution ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** distribution continuations now
revalidate post-reservation routing and identity authority, preserving accepted
pin replacements from the original transaction. Changed saved device lists,
peer pins or own authority block with `routing_changed`; ratchets are excluded.
[Red/green regression, exact scope and native/local limits](normal-group-outbox.md#authority-guard-across-distribution-ack-waits-2026-11-05).
Original ciphertext, keys and uncertainty remain reserved; reopen cannot replay.
No live operation.

### Sender-key validity across distribution ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** a distribution ACK can no longer
release dependent old-chain ciphertext after saved exposure invalidation or
chain replacement. The reservation captures chain authority, and continuation
checks it alongside routing. Same-chain delivery bookkeeping remains allowed.
[Red/green invalidation, replacement and positive-control cases](normal-group-outbox.md#sender-key-validity-across-distribution-ack-waits-2026-11-05).
Original reservations/uncertainty remain held; no reset, replay or live operation.

### Device-refresh revision guard (2026-11-05)

**Confirmed local defect / corrected offline:** saved recipient device-sync
revisions now participate in fanout OCC and post-reservation continuation.
New affected-recipient refresh work blocks stale ciphertext even before a new
list arrives; unrelated-contact work still permits continuation.
[Red/green ACK-wait regression, fresh/cached race and limits](normal-group-outbox.md#device-refresh-revisions-across-preparation-and-ack-waits-2026-11-05).
No pending-work deletion, counter reset, replay or live account operation.

### Cancelled group preparation replies (2026-11-05)

**Confirmed local defect / corrected offline:** a cancelled pre-reservation group
intent now settles its owned current query and stops with intent_cancelled,
without another query, crypto adoption or actor disconnect. Cancellation after
InFlight remains rejected by the existing API.
[Actor red/green test, reply history, reopen and boundaries](normal-group-outbox.md#cancellation-while-a-preparation-query-is-pending-2026-11-05).
No reset, replay or live account operation.

### Direct cancellation reply boundary (2026-11-05)

**Confirmed local / actor regression:** matching result/error replies and their
redelivery after direct intent cancellation emit no ciphertext, adopt no keys,
and preserve receive service. Original cancelled intent/query reservation stay
held across reopen. This was already correct; no production code changed.
[Actor proof, diagnostic distinction and remaining malformed-input scope](messaging-transport.md#direct-send-cancellation-reply-audit-2026-11-05).
No live operation or replay authority.

### Direct directory wire refusal isolation (2026-11-05)

**Confirmed local defect / corrected offline:** selected-user/classic-bundle
parser failures suspend the original direct intent with invalid_directory_reply
instead of ending receive. New-session classic signature refusal uses existing
invalid_signed_bundle. Neither permits resume; no keys/ciphertext are adopted.
[Actor red/green matrix, reopen and stored-corruption control](messaging-transport.md#malformed-direct-directory-replies-stay-intent-local-2026-11-05).
No live operation, server-refusal fabrication or replay authority.

### Invalid-directory explicit resume audit (2026-11-05)

**Confirmed local:** after reopening, the actual direct resume API refuses all
seven bad-directory/signature cases under both identity policies without any
table or journal mutation. CLI suspension wording now includes directory
validation failures; fixed reason fields are unchanged.
[API-entry regression and boundaries](messaging-transport.md#explicit-resume-refusal-for-invalid-directory-suspensions-2026-11-05).
No runtime resume policy, live operation or replay authority changed.

### Unique group-info recovery projection (2026-11-05)

**Confirmed local defect / corrected offline:** ambiguous queued group-info
replies can no longer bypass normal-send validation through restart recovery.
Shared projection rejects multiple group elements; missing queued group data
stays unapplied. Prior snapshot, pending marker and reply remain preserved.
[Red/green recovery, direct-recorder control and native/local scope](normal-group-outbox.md#unique-group-info-projection-across-recovery-2026-11-05).
Valid single groups still apply once. No query replay or live account operation.

### Unusable group-info identity stays pending (2026-11-05)

**Confirmed local defect / corrected offline:** missing/nontext ID in a selected
group-info element no longer consumes its pending projection as a successful
no-op. Recovery preserves reply, prior snapshot, checkpoints and journal.
[Red/green cases and bookkeeping boundary](normal-group-outbox.md#unusable-group-info-identity-preserves-pending-projection-2026-11-05).
No network retry, cache fallback or live account operation.

### Combined media route/wait preservation (2026-11-05)

**Confirmed local / regression:** six malformed refresh results preserve the
installed route and saved 507 wait together, with raw reply retained. Exact
duplicates and late conflicts cannot mutate them; reopening stays throttled.
A valid-result control still applies routing and the existing wait reset.
[Combined matrix, native evidence and history-scan limits](media-network-outcomes.md#malformed-refresh-replies-preserve-route-and-wait-together-2026-11-05).
No runtime policy change, new query or live operation.

### Current group-info MEX timer boundary (2026-11-05)

**Confirmed static, APK43.72:** GetGroupInfoProtocolHelper.A00 builds
QueryGroupInfo GraphQL input; the selected non-Pando dispatch serializes a
w:mex IQ and forwards the new operation's32000ms default to0Sf.A0W. The
helper's30-second initializer calculation is discarded, not timer evidence.
[Complete selected writer/dispatcher methods, branch gates and unknowns](apk/2.26.43.72-group-info-mex-timeout/README.md).
Rust's classic group IQ remains an inferred separate shape; neither its
30-second local preparation limit nor all group-info deadlines gain native
parity from this MEX trace. No runtime policy or live operation changed.

### Missing participant-identification context lifecycle (2026-11-05)

**Confirmed static, APK43.72:** the MEX override set is stored in preferences.
0zu.A02 adds a group when the selected marker child is present; absence does
not clear it.0co.A03 ->82U.A0Y ->0wE.A0e/A0b carries the group-info boolean
to an add/remove operation before later group processing.
[Persisted-set methods, exact register mapping, ingress gates and unknowns](apk/2.26.43.72-group-missing-identification/README.md).
Rust already exposes the explicit MEX context vocabulary; automatic lifecycle
and fully admitted ingress parity remain unported. No runtime/live operation.


### Missing-identification numeric handler boundary (2026-11-05)

**Confirmed static, APK2.26.43.72:** marker consumers0w9/19k construct
BaseNotificationHandler with numeric registrations209/189. The base checks
that integer before processing IncomingStanza; factory construction alone does
not establish wire notification type or authorization. Runnable selector paths
and the upstream integer producer remain unknown. No runtime port or live
operation. [Retained constructors, base handler, factory excerpts and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Current notification type mapping and picture gap (2026-11-05)

**Confirmed static, APK2.26.43.72:** EL2 maps w:gp2 to209 and picture to189;
1rN.A05 reads IncomingStanza type and passes that code as Message.arg1 via
2Ai.A00. Marker admission remains feature/group/child dependent; picture uses
set/delete. Rust has w:gp2 ACK handling but no picture consumer/ACK or automatic
missing-identification lifecycle. Native deferral and state ordering still need
reconciliation before porting. [Retained map, consumer and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Picture notification base ACK (2026-11-05)

**Confirmed static / locally implemented:** current19k picture handler has only
constructor/A09 and no base A07 deferral calls;8aj selector1 returns processing
to0v6.A01. Rust now emits the shared base ACK for picture after durable capture.
Malformed envelopes log/no-ACK and leave receive running; redelivery is ACKed
again. Photo updates and missing-ID lifecycle remain unported, with no automatic
queries or live validation. Actor regression verifies capture-before-write, exact
participant routing, redelivery, ping after refusal and reopen persistence.
[Source and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Picture ACK wire-field provenance (2026-11-05)

**Confirmed static, APK2.26.43.72:** integer76 dispatches0v0 ->0v2.A02 ->
F3e.A05, which builds the final ACK. F0R's fresh notification key populates
from/id/participant/type/class; recipient/edit/error/offline are not copied.
The existing picture actor regression now supplies those attributes and requires
the same exact base ACK. Native photo exception/CCQ/full address parity remains
unverified. [Retained final builder, key fields and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Notification address diagnostic boundary (2026-11-05)

**Confirmed static, APK2.26.43.72:** DBe.A1A/0Tl preserve Jid objects and raw
strings for ACK attributes.0Tp.A0A logs failed isProtocolCompliant then continues
toward class conversion/return; it does not reject solely on that boolean. A
proposed blanket rejection based on this diagnostic was ruled out. Current
binary-reader zero-AD normalization and subtype writer parity remain unclosed;
no shared Rust address policy changed. [Retained source and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Current-build ACK zero-device normalization (2026-11-05)

**Confirmed static, APK2.26.43.72 / offline regression:**123.A01/A02/A04
reconstruct binary zero-device PN/LID without a device suffix;1r2.A00 chooses
pair for zero compliant getters and AD for positive device/agent. The existing
picture actor test now covers zero/nonzero PN/LID sender and participant forms.
Native domain flags are distinct from Rust's agent field. No runtime policy
change; all subtype/hosted/interop behavior and live acceptance are not claimed.
[Current source and census](apk/2.26.43.72-group-missing-identification/README.md).


### Current notification constructor census (2026-11-05)

**Confirmed static:**34 physical0v6 constructor callers are retained with their
numeric code/base-executor boolean. The type map has38 entries; these counts
are different populations and do not measure completeness. server_sync210 maps
to14l (typed collection/bootstrap work), server235 to15I (log/abprops/push-config
follow-ups). Rust has no dedicated handler/ACK for either. Full state/helper/job
and ACK ordering remain unported; no generic ACK, token fabrication or runtime
change was added. [Census and complete selected handlers](apk/2.26.43.72-notification-handler-census/README.md).


### Server-sync callback ownership reconciliation (2026-11-05)

**Confirmed static:**14l.C6N ->2JI selector6 emits count-minus-one aggregate
metrics and clears its counter map; it is not an ACK completion gate. The
existing lifecycle audit already proves dirty-version assignment and normal
ACK-before-async-result ordering. The constructor census now links that evidence
instead of presenting it as newly unknown. Rust role/recovery/bootstrap and
durable notification scheduling remain unported. No runtime change.
[Callback evidence and limits](apk/2.26.43.72-notification-handler-census/README.md).


### Rust server-sync gate mapping audit (2026-11-05)

**Confirmed source:** current primary-login checks, held-fetch admission, pending
patch exclusion and receipt-time server backoff were mapped individually against
the native lifecycle. None supplies the missing bootstrap/recovery/ready-mutation
gates by default. Generic reservation/ActionStatus::Ok is not sync-flight ownership
or authenticated collection advancement. [Mapping, source boundaries and integration consequences](server-sync-rust-state-mapping.md). No runtime or live operation changed.


### Registered device-index writer provenance (2026-11-05)

**Confirmed static:** exact registration_device_id string scan finds8 references.
Selected107.BLn device-JID getter feeds the saved value/cache;09O.A01 and
G0j.A02 have zero resets;0Qq.A0a reads selected-account state. Existing role/
Me-ingress audits qualify the newer mapping's unknowns. Full Rust role migration
and producer/account binding remain unported. [Writer/reader census and limits](apk/2.26.43.72-notification-handler-census/README.md). No runtime reset, query or live operation.


### Group notification storage failures stop before ACK (2026-11-05)

**Confirmed local defect/fix:** the actor no longer logs a failed group snapshot
update then emits w:gp2 ACK. Store/transaction errors now surface first; wire
skip behavior is unchanged. Red/green actor regression proves no ACK after a
corrupt-snapshot read, unchanged saved bytes and durable capture across reopen.
No automatic replay/repair or live operation. [Full correction and APK boundary](notification-ack-audit.md#group-notification-storage-failures-stop-before-ack-2026-11-05).


### Group envelope admission precedes membership updates (2026-11-05)

**Confirmed local defect/fix:** invalid group ACK envelopes previously changed
saved membership before refusal. The actor now validates the existing envelope,
commits the supported change, then writes ACK. Missing/empty ID and unusable
participant regressions preserve exact snapshot bytes and receive continuity;
a valid control commits before ACK and survives reopen. Store faults remain
visible; groups_dirty retains its own path. [Evidence, red/green test and limits](notification-ack-audit.md#validate-group-ack-envelope-before-membership-mutation-2026-11-05).


### Public group-change envelope guard (2026-11-05)

**Confirmed local defect/fix:** public groups::apply_change_notification now
validates the same ACK envelope before store access; direct callers cannot
bypass the receive actor's prerequisite. Three invalid-envelope regressions
preserve exact snapshot bytes and journal head. A routing-race fixture gained
a synthetic notification ID without weakening its assertions. The helper sends
nothing. [Details and red/green validation](groups.md#public-group-change-admission-matches-receive-2026-11-05).


### First group event owns the supported change (2026-11-05)

**Confirmed static / locally fixed:** APK43.72 0Tp.A0D/0w9.A09 selects first
child as the event. Rust previously applied every outer sibling; unsupported+
remove could remove membership and add+remove applied two changes. The helper
now selects only the first event, retaining inner participant iteration and
existing ACK/envelope/store guards. Red/green regression preserves unselected
membership and standalone valid updates. Extra metadata siblings are permitted;
full native event/timestamp/alias parity is not claimed.
[Checked selection evidence](apk/2.26.43.72-group-event-selection/README.md).


### Dirty group event with metadata siblings (2026-11-05)

**Confirmed static / locally fixed:** current0w9 selects first-child groups_dirty
without requiring exactly one outer child. Rust previously skipped refresh for
metadata siblings and could generic-ACK without scheduling. Dedicated dirty
handling now selects the first event; generic ACK stays suppressed for it.
Existing durable queue/dedup/envelope/inner-list guards remain. The expanded
queue-before-ACK regression covers metadata sibling, sole ACK ownership and
reopen without another reservation. Missing-identification state remains unported.
[Evidence and limits](apk/2.26.43.72-group-event-selection/README.md).


### Dirty list filters named group entries (2026-11-05)

**Confirmed static / locally fixed:**0w9.A09 calls0Tp.A0N(group), which selects
matching children and skips others. Rust's whole-list rejection on unrelated
children suppressed valid refresh/ACK. It now filters group entries; selected
identity/duplicate/device, empty-work, envelope and256-total-child guards remain.
Red/green queue-before-ACK/reopen regression includes metadata with an unusable
jid beside a valid group. Unknown-only remains unsupported.
[Exact helper and limits](apk/2.26.43.72-group-event-selection/README.md).


### Privacy-token legacy agent qualifier (2026-11-05)

**Confirmed local defect/fix:** supported primary from/sender_lid admission
checked device0 but not agent0; a qualified address could write token state.
Those guards now require both zero, retaining PN/LID namespace handling and
existing invalid-alias policy. Native10G.A09 uses UserJid getters; Rust's LID
domain bit is not its legacy agent field. Red/green typed/text PN/LID and alias
regression preserves checkpoint/pin/session snapshots across reopen. Reach-out
restriction envelope admission was separately rechecked and already guarded.
[Complete current source and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### First named privacy-token container (2026-11-05)

**Confirmed static / locally fixed:**10G.A09 ->0Tp.A0G/A0F selects the first
named tokens container. Rust's uniqueness guard suppressed valid first data/ACK
when another container followed. Selection now matches the APK; later containers
cannot replace state or rescue a malformed first. Existing selected-list/time/
size/address/atomic guards remain. Red/green regression includes leading metadata,
later newer/malformed data and invalid-first/valid-second.
[Complete helper evidence and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### Consume token time only for supported data (2026-11-05)

**Confirmed static / locally fixed:**10G.A09 reads parent t only inside the
trusted_contact branch. Rust's eager timestamp parse suppressed ACK for empty
or ignored-only lists. Parent time is now parsed only for supported entries;
supported-token Java/range/atomic checks remain. Red/green empty/unknown-only
regression covers absent, malformed, negative and overflowing unused t without
changing prior state. [Retained branch proof and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### Ignored privacy tokens do not consume sender alias (2026-11-05)

**Confirmed static / locally fixed:**10G.A09 reads sender_lid only after selecting
trusted_contact. Rust's eager alias resolution suppressed ACK for empty/ignored
lists with invalid unused metadata. After the existing container/list checks it
now returns no updates before alias/time resolution when no supported entries
exist. Supported alias/time/primary/atomic guards remain. Red/green regression
combines ignored lists with malformed or agent-qualified unused alias and time,
preserving prior token state. [Evidence and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### Account privacy first-event admission (2026-11-05)

**Confirmed static / locally fixed:**19N.A09 selects the first event. Rust's
exact-one outer child guard suppressed supported privacy adoption/ACK with later
siblings. It now requires the first event to be supported privacy and ignores
later event siblings; unsupported/malformed first events cannot be rescued or
overridden. New red/green regression preserves prior state for those refusals.
Category/envelope/pending-intent/transaction guards remain.
[Evidence and boundary](account-privacy-notifications.md#first-event-selection-with-later-siblings-2026-11-05).


### Account privacy named-category selection (2026-11-05)

**Confirmed static / locally fixed:**19N.A09 uses0Tp.A0N(category). Rust's
whole-child parsing rejected unrelated metadata beside supported readreceipts.
It now filters named categories before validation, retaining64-total-child,
selected category/value/body, envelope/pending/transaction guards. Metadata-only
remains refused. Red/green regression ensures metadata cannot adopt preference
state and does not suppress the selected valid category.
[Source and limits](account-privacy-notifications.md#named-category-filtering-2026-11-05).


### Count ACK optional source and malformed-source refusal (2026-11-05)

**Confirmed static / locally fixed:** deferred89Y preserves original Eoh; the
common final builder omits absent to. Rust's server fallback fabricated explicit
to for absent source and redirected explicit parse failure. Count ACK now omits
to for absence, retains normal parsed routing and locally refuses explicit
invalid source before ACK/upload. Red/green actor coverage checks source-less
PQ no-upload and malformed-source capture/receive continuation; valid legacy
upload controls remain. Native ACK-after-work timing remains unported.
[Complete deferred chain, wire proof and limits](apk/2.26.43.72-prekey-count-ack-source/README.md).

### Transactional stanza-to-ACK dispatch (2026-11-05)

**Confirmed static, APK2.26.43.72:** Flr selector7 has a conditional outer
transaction that persists selected stanzas and computed pre-ACKs before
Euj.A02 send admission. A thrown outer close propagates before that send call.
The alternate branch persists stanzas without this explicit outer pre-ACK
insertion. Both operate on a filtered return list, not all captured input;
A0I is set after send admission returns, not after server confirmation.
[Five exact methods, offsets and limitations](apk/2.26.43.72-preack-transactional-dispatch/README.md).
This closes the Flr insertion-caller gap, not effective configuration,
count-specific bulk membership, crash replay or upload/socket ordering.
No Rust runtime behavior or live account operation changed.

### Malformed count content still receives its control ACK (2026-11-05)

**Confirmed static and fixed offline:** current15v.A09 throws LIL for a
malformed present PQ count before deferring metadata or queueing upload work.
0v6.A01 catches LIL and dispatches the still-present original ACK context;
SQLite full/corruption failures instead rethrow. Rust now ACKs supported count
envelopes on content refusal without upload. Invalid IDs/sources still refuse
ACK; storage and uncertain write failures still propagate. This supersedes the
earlier local no-ACK policy for malformed PQ content.
[Exact parser/catch paths, incoming-kind census, regression and remaining queue limits](apk/2.26.43.72-count-content-error-ack/README.md).
Valid-count upload/ACK ordering and full queue replay remain open. No live operations.

### Deferred ACK six-second watchdog is not a send deadline (2026-11-05)

**Confirmed static,43.72:** the0v6.A07 scheduled8bZ selector41 reports pending
ACK failures and clears tracking maps; it does not ACK, cancel upload work or
consume held89Y tokens. Deferral resets the shared6000ms schedule. A later
89Y completion ignores whether removal found an entry and can still dispatch
its one-use token. [Exact schedule, branch and token distinction](apk/2.26.43.72-deferred-ack-watchdog/README.md).
This closes the prior timer-action unknown, not count-specific routing,
socket ordering or recovery parity. No Rust timer/replay or live operation added.

### Registered signed-prekey absence versus stored-state faults (2026-11-05)

**Confirmed local defect, fixed offline:** actor maybe_upload caught every
error from registered_signed_prekey and reported a normal missing-key skip.
That hid corrupt saved registration JSON, account/material mismatches and
other lookup/validation failures. The shared factory now has an internal
optional lookup: only an absent registration-lab-material row returns None.
Existing JSON/account/signature/private-key/serialization errors propagate.
The public required-key lookup and digest caller still require a row and
return their existing unavailable error for absence. No identity, signed key
or one-time key is generated to repair a failed lookup.

The actor skips only None and propagates errors before generating/reserving
or writing an upload. An injected-link regression reproduces the old swallowed
corrupt-row error (actor never ended), then checks clean absence skip, Storage
for corrupt JSON and InvalidInput for a mismatched account. Saved material and
prekey rows are unchanged. Existing valid-material upload controls remain.
This is local diagnosability/integrity behavior, not a claim about exact native
signed-key fallback. The native base-handler SQLite exception distinction is
retained in [the count error audit](apk/2.26.43.72-count-content-error-ack/README.md).
Count ACK-before-upload ordering remains unported and unchanged, so this does
not claim all count work errors occur before its control ACK. No live operations.

### Signed-prekey public/private consistency before upload (2026-11-05)

**Confirmed local defect, fixed offline:** registered signed-prekey lookup
verified the identity signature over the saved public key but constructed a
SignedPreKeyRecord from an independently deserialized private half without
checking that the halves matched. A valid public signature does not validate
that private half. The shared optional/required lookup now derives its public
key and refuses a mismatch with a secret-free InvalidInput reason before
serialization. Upload admission propagates that error before generating,
reserving or writing any upload. The digest required-key lookup also uses the
same validation; no private bytes are exposed or repaired.

Expanded actor corruption regression keeps the registered signed public key
and its valid signature unchanged but substitutes the independently generated
identity private key. Old code reached an upload write instead of returning an
error (the actor-end expectation timed out). Corrected code returns the exact
pair-mismatch error and leaves saved material and prekey rows unchanged.
Existing clean-material upload and absence/corrupt-JSON/account-mismatch
controls remain. This is a local cryptographic consistency check, not a new
APK-derived wire requirement or proof of native corrupt-store handling.
Registration request-factory loading outside this signed-key lookup is not
claimed audited by this change. No identity rotation, key replacement, new
retry or live account operation; valid-count ACK ordering remains unchanged.

### Registration material key-pair integrity before request construction (2026-11-05)

**Confirmed local defect, fixed offline:** StoredComparisonRequestFactory.load
checked frozen attempt/configuration, account public metadata and the signed
public-key signature, but did not prove the saved private halves matched the
advertised Noise, Signal identity or signed-prekey public keys. Its account
comparison also ignored private-half corruption in the separate AccountRecord.
A request could therefore be constructed from material unusable for the later
private-key operation, even with a valid signed public-key signature.

Material.check_account now derives/checks all three material key pairs plus
the account Noise and Signal identity pairs after its existing account/public
metadata check. It reports only a fixed pair-specific InvalidInput reason.
Both normal request loading and repeated/corrected preparation use this guard.
The registered-signed-prekey lookup also shares check_account. Derivation
compares effective public keys, not raw private-byte identity; no new key,
request field, registration retry or account generation is manufactured.

Regression: for each of five stored private-half substitutions (material Noise,
identity, signed; account Noise, identity), retain public keys/signature/config,
then require code/exist/complete/security request construction and repeated
prepare to fail. Exact Account table bytes and journal head remain unchanged;
reopen still refuses and retains bytes. Clean construction is a control in
each case. Old code failed the Noise case because it built the request.
This is local cryptographic integrity, not an APK wire-field requirement or
proof of native corrupt-store behavior. No HTTP/account operation ran.

### Count ingress has a gated tracked fallback (2026-11-05)

**Confirmed static,43.72:** notification reader routing is conditional on
provider selection and configuration. With no named provider,3992 true creates
a tracked1k4 wrapper for recognized encrypt; false reaches direct handling.
A bounded census of ten physical routing-provider constructors declares no
encrypt entry, but does not establish the full effective injected set.
[Registry identity, enum, wrapper, persistence-policy methods and exact limits](apk/2.26.43.72-count-ingress-routing/README.md).
The reader inverts buffering-policy output for a wrapper Boolean; its durable
meaning and count-specific bulk ownership remain unproven. No Rust routing,
ACK order, default flag or replay changed; no live operations.

### Tracked count queue admission is separate from disk commit (2026-11-05)

**Confirmed static,43.72:** reader buffering-policy inversion becomes wrapper
A0D;1k4 selects1ix. A0D=false requires a storage backend, but1id.A0P first
adds the stanza to memory. Its successful return is not commit proof.1ix
A0L=false disables the generic inline-state promotion. A conditional500ms
disk schedule cancels/replaces a prior job and launches the already-traced
Flr selector7 transactional writer.
[Seven complete methods, Boolean mapping and processing-gate limits](apk/2.26.43.72-count-queue-admission/README.md).
The arrival hook reaches1il/2Ek processing helpers, still untraced here. No
Rust timer, ACK ordering, replay or live account behavior changed.

### In-memory processing selection is not disk-commit proof (2026-11-05)

**Confirmed static,43.72:** consumer scheduling reaches1id.A08 selection. Its
first memory predicate requiresA0H=A05 and optional ordering, not a row ID;
new wrappers start inA05. Selection changes memory state toA06, then routes
selected1k4 entries by class into1rA.A03. SQLite full/corruption failures
rethrow; other selected handler failures go to1ic.A0E.
[Fourteen exact methods and remaining injection/hold/refill/error limits](apk/2.26.43.72-count-queue-processing/README.md).
This refutes a blanket inference that all processing must await disk commit.
ACK durability remains a separate conditional writer boundary. No Rust queue,
ACK ordering, replay or live account behavior changed.

### Escaped handler errors have separate NACK and completion paths (2026-11-05)

**Confirmed static,43.72:** queued router storage full/corruption errors rethrow.
Other selected escaped errors reach1ic.A0E, requesting487 for LIL or500 for
other failures and constructing completion metadata with ACK-list flagfalse.
Selected completion marks FAILED_TO_PROCESS; that false flag prevents adding
another ElB ACK from its list-generation branch. Queue enum names confirm
QUEUED/READY_TO_PROCESS/PROCESSING/PROCESSED/FAILED_TO_PROCESS are distinct
from persistence and server confirmation.
[Eleven exact methods, response/completion separation and callback limits](apk/2.26.43.72-count-handler-error-completion/README.md).
Count PQ parse LIL caught inside0v6 still follows its original normal ACK,
not this escaped-error487 path. No Rust NACK/retry/replay or live operation added.

### Failed completion disk retirement and cache removal differ (2026-11-05)

**Confirmed static,43.72:** failure callback Feh(46) calls1id.A0J for cache
retirement. In the selected transaction route,1ic.A0A separately collects
nonnull row IDs for backend1ih.A04 deletion; the callback is normally released
after successful outer completion. Its no-work path can invoke the callback
without a transaction. Cache removal alone is not universal commit proof.
1iq.A0F checks key membership only, not row/account/content identity.
[Exact methods,975-ID batches and backend/rollback/helper limits](apk/2.26.43.72-count-completion-retirement/README.md).
This is stanza-row retirement, distinct from confirmed pre-ACK watermark
deletion. No Rust deletion, replay, retry or live operation added.

### Queue transaction and retirement share a memoized database provider (2026-11-05)

**Confirmed static,43.72:** manager transaction supplier and stanza retirement
supplier both resolve provider0xc3e, then0Xd.A00. Global provider-ID caching
and owner lazy caching support the same memoized0Xp within a stable cache
lifetime. Its646a backend choice occurs once per owner, not per operation.
[Seventeen exact methods and stable-lifetime/configuration limits](apk/2.26.43.72-count-database-provider/README.md).
0Xp.A07 can construct0aq or0ar; effective branch, connection ownership,
provider reset and crash/disk ambiguity still need tracing. Shared helper
identity alone is not full atomicity proof. No Rust backend switch, ACK,
replay or live account operation changed.

### Base and alternate database wrappers share transaction primitives (2026-11-05)

**Confirmed static,43.72:**0ar delegates acquisition to0aq; both use the helper
writable-handle cache and shared0HC listener. Both virtual transaction factories
construct1hk, passing the same handle/listener and differing health state.
Within a stable provider/open-handle interval, synchronous nested retirement
reaches the same SQLiteDatabase, narrowing the normal transaction identity gap.
[Eight complete methods, assumptions and exceptional-close limits](apk/2.26.43.72-count-database-wrappers/README.md).
Health callback failure, base close, replacement/reset, ambiguous disk commit
and crash recovery remain open. No Rust backend, ACK, replay or live operation changed.

### Owned prekey inventory keys cannot be ignored or alias an ID (2026-11-05)

**Confirmed local defect, fixed offline:** prekeys.scan_prefix discarded invalid
UTF-8 names even under its owned prefix, while meta_map and record_ids silently
filtered unparsable ID suffixes. Parsed alternate spellings such as01 collapsed
onto the same numeric ID; metadata collection could thereby merge distinct
stored names. This can make corrupt publication state look like absent state
or an otherwise usable inventory. It is not a demonstrated server restriction
trigger or proof that every alias causes a resend.

Owned namespaces now reject invalid UTF-8, and record/metadata ID suffixes must
be exactly canonical decimal u32 text. Existing writers already use that form.
No new numeric wire-width/zero-ID policy is introduced; existing serialized
record/ID and upload validation remains. Unrelated namespace keys remain outside
the selected scan. The caller receives a fixed secret-free InvalidInput reason
instead of a partial inventory; no metadata is repaired, merged or deleted.

Regression replaces canonical record or outcome-unknown metadata names with
leading-zero, invalid, overflowing or invalid-UTF8 names. Old code accepted the
first alias. Inventory lookup, generation and upload admission now refuse; exact
prekey/checkpoint bytes remain unchanged and reopening still refuses. This is
local storage-integrity behavior, not a claim that native SQL IDs use this Rust
key format. Normal native inventory/wire evidence remains in this document.
No live operation, new retry, reservation reset or key regeneration was added.

### Legacy one-time prekey private/public consistency (2026-11-05)

**Confirmed local defect, fixed offline:** legacy_record_in checked the
serialized ID against its store key, and upload construction decoded the
public key, but neither derived public from the saved private half. The
inspected locked wacore-libsignal0.7.0 PreKeyRecord deserializer parses the
record/normalizes stored public encoding; its getters do not establish pair
consistency. A well-formed record could therefore publish a public key whose
saved private key cannot perform the corresponding private operation.

The shared legacy record lookup now derives and compares the public key,
returning fixed InvalidInput on mismatch or unusable private data. Upload
selection and local digest inspection use that lookup. This is local crypto
integrity, not an APK wire-field change or a claim about native corrupt-row
recovery. No key repair, replacement, rotation or retry is authorized.

Regression corrupts record50 in a generated50-key batch while retaining its
valid ID/public key. Old code returned a reserved upload. Corrected admission
returns the exact pair-mismatch error; all prekey and checkpoint bytes remain
unchanged, including rollback of earlier keys' tentative reservations. Reopen
still refuses with identical rows. Existing normal upload and independent
digest controls remain. No live account or network operation ran.

### Direct upload API validates the supplied signed private half (2026-11-05)

**Confirmed local defect, fixed offline:** begin_upload and its shared selected
upload implementation accept a caller-supplied serialized registered signed
prekey. They checked wire ID, public key and identity signature but did not
validate the private half. The earlier factory lookup guard did not protect
callers using this API directly. A valid public signature therefore allowed a
record with a mismatched private key to reserve an upload.

The shared upload entry now decodes signed.private_key, derives public and
compares it with signed_public before allocating the IQ ID or entering the
reservation transaction. Unusable private data and pair mismatch produce fixed
InvalidInput reasons. Signature and account/store ownership checks remain.
No key is repaired or replaced, and no alternative identity or request is
created. This is local input integrity, not a new APK wire requirement or a
claim about native corrupt-store fallback.

Regression preserves the signed ID/public/signature/timestamp while substituting
an independently generated private key. It explicitly confirms the public
signature still verifies. Old code returned a reserved upload. Corrected code
returns pair-mismatch, preserves exact prekey/checkpoint bytes and still refuses
after reopen; the original valid signed record then reserves successfully.
This direct-API boundary is separate from factory loading and does not claim
every other raw serialized-key API was audited. No live account operation.

### KEM batch scans refuse corrupt owned inventory (2026-11-05)

**Confirmed local defect, fixed offline:** kem_prekeys.all_rows skipped JSON
parse failures and accepted parsed records without binding their IDs to their
owned storage names. Migration could inspect sent/last_resort from rows whose
material or key binding had not been validated; replenishment could count a
partial inventory and generate around malformed records. This was distinct
from the existing strict point lookup.

The write-side batch scan now parses every owned row or refuses, requires the
exact canonical record_key(row.id), and applies existing row.validate before
returning any inventory. This covers migration detection, SET selection and
ADD replenishment. Corrupt sent or otherwise unselected rows also refuse the
whole write-side inventory. Diagnostic counts retain their documented ability
to count malformed rows; they are not upload or migration authority.

**Refuted suspected gap:** point lookup already checks row ID, lengths, signature
length and the public key embedded in expanded secret. Those checks were not
missing and were not reimplemented. This change applies them to batch admission;
it does not prove additional algebraic secret/public consistency or native
corrupt-store recovery. Existing signature/public-key upload validation remains.

Regression injects malformed JSON, mismatched row ID, leading-zero alias,
invalid-UTF8 suffix and a sent last-resort row with mismatched embedded public
material. Migration, SET selection and ADD replenishment all refuse without
changing exact prekey/checkpoint bytes; reopen still refuses. Old code failed
the malformed-JSON migration case by returning a partial result. No identity
rotation, replacement keys, new retry, live account or network operation.

### Restored KEM rows cannot override consumption markers (2026-11-05)

**Confirmed local defect, fixed offline:** consume_in atomically deletes an
ordinary KEM row and records authenticated consumption. Generation already
refuses IDs with a marker, and a missing consumed row may settle a late upload
ACK. But a stale row restored beside its consumption marker could still load,
enter batch inventory, be consumed again or be marked sent. Row presence was
incorrectly sufficient to bypass the durable marker's one-time meaning.

A live-row/marker conflict now refuses point lookup, batch inventory, repeated
consumption and present-row upload settlement. Any marker bytes beside a live
row hold that ID; malformed marker contents do not grant authority. Missing-row
late-ACK validation remains unchanged and still checks the saved marker. No
marker is erased, no key is repaired and last-resort retention policy remains.
This protects against conflicting local restore/corruption, not wholesale
rollback of both row and marker or proof of native recovery parity.

Regression installs and consumes a real KEM row, verifies missing-row late ACK
succeeds without restoring it, then restores the original row under its correct
key. Lookup, repeated consumption, migration, SET/ADD selection and present-row
settlement all refuse. Exact row/checkpoint bytes survive the refusals and
reopen. Old code failed by returning the restored row. Existing consume-before-
ACK and last-resort controls remain. No live operation or new retry authority.

### KEM upload settlement is bound to reserved public material (2026-11-05)

**Confirmed local defect, fixed offline:** PendingUpload.settle bound durable
KEM ID lists to the callback, but mark_sent_in loaded whichever valid rows
occupied those IDs. An accepted reply could mark a substituted public key as
uploaded. A missing-row consumption marker was checked by ID alone even though
it already retained a public fingerprint. IDs are not exact publication evidence.

New Batch.kem_public_sha256 stores SHA256 of every selected KEM public key,
in kem_ids wire order, in the same reservation transaction as frozen request
bytes. PendingUpload retains the identical list. All settlement first requires
list equality and matching cardinality; an accepted result then checks each
current row's validated public fingerprint or the consumed marker's ID/hash
before writing batch/key status. Existing live-row/marker conflict guards
remain. Any mismatch aborts the full transaction and preserves uncertainty.
Refusals still do not adopt current KEM public material or retry uploads.

The additive field defaults empty when reading historical Batch records.
Historical batches remain inspection/hold evidence; no callback reconstruction,
missing-hash inference, automatic migration or replay is added. Binding is
public-key identity, not a new signature/private-key equality claim or APK
wire-field requirement. Whole-store rollback remains outside this local guard.

Expanded existing uncertainty regression replaces a live KEM row with a different
well-formed pair under the same ID: old code accepted the reply. New code refuses
with exact row/checkpoint preservation. It also rejects durable fingerprint-list
tampering for result/error and a wrong consumed-public hash. Existing missing-row
refusal, consumed-before-ACK success, reopen and uncertain-no-replay controls
remain. Synthetic fixture restoration only continues independent controls;
production does not repair reservations to make replies pass. No live operation.

### Legacy upload public identity gap (2026-11-05)

**Initial finding, fixed below:** a synthetic same-ID valid legacy key replacement
was accepted by upload settlement at 9b26f558. Direct and group receive delete
legacy rows atomically but retain no consumed-public fingerprint. Full
reproducer, source path analysis and required fix matrix are retained in
[the audit](legacy-prekey-publication-binding.md). KEM binding does not cover
legacy keys. No live operation or new runtime guarantee.

### Legacy upload public binding and atomic consumption (2026-11-05)

**Confirmed local fix:** reservations now retain legacy public fingerprints;
accepted settlement checks live rows or authenticated consumed ID/hash evidence.
Direct/group receive commit consumption evidence atomically with row deletion.
Restored conflicts, unexplained loss and altered fingerprints refuse without
settlement writes; legitimate missing-row session continuation remains supported.
Historical state is not inferred or replayed. [Implementation and regression
scope](legacy-prekey-publication-binding.md#implemented-public-binding-and-atomic-consumption-2026-11-05).

### Legacy inventory validates uploaded and unselected rows (2026-11-05)

**Confirmed local defect, fixed offline:** record_ids decoded JSON byte arrays
and checked canonical store names, but did not validate the serialized prekey.
unsent_in then filtered Uploaded/OutcomeUnknown rows before strict point lookup.
A malformed, wrong-ID, private/public-inconsistent or restored-consumed row
marked Uploaded could therefore be ignored while inventory counted available
keys, generation replenished, or another upload reserved different rows.

record_ids now applies legacy_record_in to every owned row before returning IDs.
This reuses three-byte ID, stored ID, serialized-record, private/public pair and
consumption-marker conflict validation. No row is skipped or repaired. Inventory,
replenishment and upload decisions that consult it refuse the complete snapshot;
summary also refuses invalid legacy material rather than reporting normal counts.
An earlier saved-refusal hold may return without consulting inventory, as before;
it grants no reservation or retry authority. KEM diagnostic counts retain their
separate existing behavior. Valid uploaded/uncertain rows remain excluded from
unsent capacity only after validation. There is no new APK wire-field claim.

Regression legacy_inventory_refuses_corrupt_unselected_uploaded_rows marks row50
Uploaded, leaving49 valid unsent rows. Four independent fixtures inject malformed
serialized bytes, a different stored ID, a validly encoded mismatched private
half, or a consumption marker beside the live row. Inventory, wants_upload,
generation and upload all refuse without changing exact key/checkpoint rows,
including after reopen. Old code failed the first inventory refusal assertion.
The existing target-count generation test now uses valid serialized records in
its synthetic bulk inventory instead of one-byte placeholders, preserving its
count/target/wrap assertions. No production regeneration around corrupt rows or
live account operation was added. Whole-store rollback remains outside the guard.

### PQ digest uses validated KEM inventory (2026-11-05)

**Confirmed local fix:** gated LR digest selection now shares all_rows validation
with publication. Corrupt unselected ordinary/unsent records can no longer hide
behind LR/status filtering and produce Matches. Four injected corruption cases
preserve exact rows on refusal; existing published-signature controls remain.
[Scope, reproducer and limits](prekey-count-notifications.md#pq-digest-rejects-corrupt-unselected-kem-rows-2026-11-05).
No live request, resynchronization or replay authority.

### Upload settlement checks reserved account authority (2026-11-05)

**Confirmed local fix:** PendingUpload now retains a publication-authority hash
and persists it in Batch. Settlement rereads the account and checks transaction
generation, registered authority and durable binding before status writes.
Nine changed/missing authority cases refuse result/error across reopen; unrelated
authentication-time updates still settle. [Scope and remaining write/rotation
limits](prekey-count-notifications.md#upload-settlement-retains-publication-account-authority-2026-11-05).
No live traffic, replay or inferred historical binding.

### Count completion key removal and conditional rescheduling (2026-11-05)

**Confirmed static, APK 2.26.43.72:** FgR selector8 removes every cached stanza
whose 2DG key is in the completion set; only removed non-QUEUED entries increment
its delta counter. 07W.A0S/A0T/A0R prove remove-if semantics on RandomAccess and
iterator paths. 1iq.A03 sends the negated nonzero count to its optional callback.
These are memory/counter effects, not SQL retirement or server confirmation.

1id.A0J later calls 1im.CHt with remaining noncompleted cache count. Physical CHt
census finds the abstract interface and 1il/1in/1iw implementations. Base1il
ignores the integer and conditionally calls its existing scheduler through A06:
1iz uses config0x6f06; other selected kinds use1kB.A00. Effective DI/configuration,
counter storage, key lifetime and complete rejection/recovery policy remain
open. Completion is a scheduling stimulus, not eventual work or ACK ordering.
[Complete exports, commands, checksum manifest and boundaries](apk/2.26.43.72-count-completion-rescheduling/README.md).
No Rust runtime or live account operation changed.

### Count queue consumer construction is resolved (2026-11-05)

**Confirmed static, APK2.26.43.72:** queue1ix provider0xd04 follows1Uw.A00
page0 ->A01 range(0xc10,0xd29] ->A0T case3332 ->0Zy.A0J ->new1iz.
This closes the selected stable-provider construction unknown:1iz inherits
base1il.CHt and uses its0x6f06 configuration branch. It does not use the
separate1in/1iw ordered overrides on that construction path.

The1iz hold-owner binding0xd09 resolves through case3337 ->0Zy.A0I ->new1k7;
its A0A AtomicBoolean initializes false. Later mutation/value/release is open.
00E.A15 passes null Boolean to A0D, not false; neither the declared default
nor effective0x6f06 value is established. Shared provider-cache lifetime,
pause/hold/capacity/CAS, rejection and crash boundaries remain distinct from
job execution, ACK ordering or disk/server confirmation.
[Complete branch exports, reproduction and limits](apk/2.26.43.72-count-consumer-binding/README.md).
No Rust runtime change, replay authority or live operation.

### Unordered hold mutation and conditional release (2026-11-05)

**Confirmed static, APK2.26.43.72:** selected2JI runner holds unordered pulls
behind loaded ordered metadata when0av.A0U is true. Arm numberA00, change
counterA0B, hold atomicA0A and one-batch bypassA01 have distinct roles.
Ordered completion1k7.A03 clears the hold and asks the scheduler to run only
when no loaded ordered metadata is pending and the old hold was true.

The FeQ selector21 timer checks its captured arm. With ordered work pending,
either unchanged nonnull saved head (OR, not both) or two matching null head
pairs can permit a one-batch bypass; changed heads otherwise rearm the timer.
The runner separately detects concurrent release throughA0B. Pending checks
skip unloaded caches and inspect key-map size, not SQL or in-flight states.
No fixed hold delay/default is inferred from0av.A06, and full executor/timer
failure semantics and ACK ordering remain open.
[Full exports, exact predicates, direct field census and limits](apk/2.26.43.72-count-hold-release/README.md).
No Rust queue change, retry/replay authority or live account operation.

### Hold gate/delay descriptor and registry inputs (2026-11-05)

**Confirmed static, APK2.26.43.72:**0av constructor samples hold gateA0U from
0ax.A0C(key0x9952), whose descriptor Boolean inputs are false/false. When false,
it stores delayA06=0; when true, it reads0ax.A0G(key0x9953), widens the signed
int and stores it, without a local clamp. That descriptor's inputs are60000/
60000; the timer log labels the value milliseconds. Scheduler gate0x6f06 is
separate. Direct write censuses find one constructor write forA0U andA06 each.

Selected08Y default-map entries showfalse for0x9952/0x6f06 and60000 for0x9953.
The selected fallback resolver checks cached values, registry defaults before
descriptor fallback, then saved preferences. These declared inputs/entries do
not establish effective runtime configuration or enable a fixed Rust timer.
Full outer MobileConfig precedence, refresh/provider lifetime, complete map
collision/value-flow and scheduler handling remain open. Large source methods
are retained as explicit bounded excerpts with full-source hashes/ranges.
[Exports, selected data flow and coverage limits](apk/2.26.43.72-count-hold-config/README.md).
No Rust network change, replay permission or live operation.

### Queued prekey uploads recheck ownership at writer dequeue (2026-11-05)

**Confirmed local defect, fixed offline:** begin_upload_selected checks account
authority when reserving and settle checks it on reply, but NodeLink previously
handed a queued Node directly to the writer. An account change committed after
reservation/handoff could therefore reach encoding and transmission before a
later settlement refusal. Reservation-time checks are not writer admission.

Outbound is now an internal envelope with an optional prekey guard. All three
maybe_upload triggers attach the reserved PendingUpload and store handle;
ordinary sends retain their existing path. After dequeue, before encode or
writer.write, shared Outbound.prepare invokes check_write_authority in one
read snapshot. It requires registered current authority, matching transaction
generation and the callback's account hash, plus an outcome_unknown batch with
unchanged account binding, legacy/KEM IDs and public-hash vectors/cardinality.
A refusal sends the actual local error to the waiting actor and stops the real
writer epoch before encoding/writing that node. It does not settle, clear,
repair or replay the reserved batch. Rounds/pending ownership stay consumed.

queued_prekey_upload_rechecks_authority_before_writer_encoding holds an actual
injected outbound envelope after reservation/handoff, then revokes the account
or tampers its durable batch binding. Shared preparation refuses before the
test's encoding counter advances; Account/Prekeys/Checkpoints remain byte-for-
byte unchanged by preparation and the caller receives failure with its pending
claim/round retained. An authentication-time-only update still encodes once.
The initial regression used an accepting preflight to preserve the previous
unguarded writer behavior and failed on revocation. Real writer code invokes
the same preparation immediately before encoding; this is not a live socket
observation. Existing fake-writer helpers now execute preparation too.

This detects changes already committed when the writer's read snapshot opens.
It does not lock account/store state across encoding or an awaited socket write,
prevent changes after preflight, validate same-identity signed-key rotation, or
make network and store effects atomic. Direct callers that transmit the returned
Node through their own I/O do not gain this actor guard. Local key mutation after
reservation and before write remains a separate material-lifetime concern.
No new APK callback policy, wire fields, reconnect/retry authority or live
account operation is claimed.

### Queued publication requires live original key material (2026-11-06)

**Confirmed local defect, fixed offline:** writer dequeue checked account and
durable batch bindings but did not reread selected key rows. Replacing a legacy
row with a different valid pair under the reserved ID still let the old frozen
public bytes reach encoding even though their private key was no longer held.
Missing/consumed KEM or legacy rows had the same admission gap. Later settlement
checks could refuse an ACK, but could not undo that publication.

check_write_authority now checks every selected legacy key's OutcomeUnknown
metadata and IQ owner, loads it through strict legacy_record_in, requires a live
row and compares its public fingerprint with the reservation. Every selected
KEM key must likewise pass strict load (including live-row/consumption-marker
conflict validation), exist and match the original public fingerprint. All checks
share the existing read snapshot at dequeue, before encoding/socket write.

A valid consumed-public marker is deliberately insufficient before first write:
publication needs a live private row, whereas settlement after transmission can
legitimately accept authenticated consume-before-ACK evidence. The fix leaves
that late-ACK behavior intact. There is no marker deletion, private-key recovery,
new key generation or reservation release to make admission pass.

Expanded queued_prekey_upload_rechecks_authority_before_writer_encoding uses
actual held injected outbound envelopes. New cases cover a well-formed legacy
pair replacement, missing legacy row, real legacy consumption, changed legacy
metadata status, well-formed KEM pair replacement, missing KEM row, real KEM
consumption, and restoration beside its real marker. Every case refuses before
encoding, preserves Account/Prekeys/Checkpoints exactly after the injection,
and leaves the pending claim/session round consumed. Existing revocation,
batch-binding and authentication-time positive controls remain. The old code
failed the first added replacement case. These are synthetic local-store/channel
checks, not server or real socket observations.

Snapshot-to-write races remain after admission. This binds selected public
identity and reuses existing private/material consistency validation; it does
not prove arbitrary KEM private algebraic consistency or bind every signature,
role/status metadata byte to the original row. Same-identity signed-key rotation,
custom direct I/O and whole-store rollback remain outside this guard. No APK
wire change, replay authority or live account operation.

### Upload deadline fences writer and direct settlement admission (2026-11-06)

**Confirmed local defects, fixed offline:** a reserved upload could wait in the
outbound queue until its30-second reply deadline, then still reach encoding and
transmission. Separately, PendingUpload.settle itself did not check that deadline;
only the actor's reply dispatch did, so direct library callers could adopt an
expired result/error. These are local claim-lifetime gaps, not newly inferred
APK timeout values or observed server restriction causes.

A shared check_deadline rejects Instant::now() >= the original pending deadline.
Writer preflight checks before store/material inspection and again at its end,
so validation cannot deliberately renew the claim or admit an already expired
request. Direct settle checks after exact IQ matching and again in the write
transaction after ownership/material validation, before status writes. Deadline
failure leaves the uncertain batch/key ownership untouched. The existing writer
envelope reports its local error and stops the epoch before encoding that node.
No fresh deadline, metadata release, replacement upload or automatic replay.
Expiry uses the dedicated local InvalidInput field prekey_upload_deadline.
The actor maps only that context to a captured late reply and continues receive
if the deadline crosses during settlement; storage/ownership errors still
propagate. The queued-expiry regression also exercises this dispatch helper.

queued_prekey_upload_cannot_start_at_or_after_reply_deadline holds a real
injected outbound envelope and uses paused monotonic time:29,999ms admits one
encoding;30,000ms and30,001ms refuse without encoding. It preserves exact
Account/Prekeys/Checkpoints, pending/round consumption and50 uncertain keys,
including after reopen; there is no replacement write. The old writer failed
at exactly30,000ms.

Direct regression direct_upload_settlement_requires_an_unexpired_callback_claim
covers result and error at the same three boundaries. Before the deadline both
retain their normal meaning; at/after it neither changes keys/checkpoints,
including on a repeated held callback after reopening. Old direct settlement
failed on the exactly-expired result. Existing actor late-reply capture and
consume-before-ACK controls remain separate tests.

This uses the existing session's monotonic deadline, not reserved_at wall time,
server timestamps or inferred historical age. It does not restore callbacks on
reconnect/restart or guarantee the deadline stays unexpired across later encoding,
socket I/O or transaction commit. Those operations are not atomic with the clock.
No APK wire field, configured timeout length, native hold delay, retry authority
or live account behavior changed. Tests use synthetic stores/injected channels.

### Device-query deadlines fence ingestion and queued writes (2026-11-06)

**Confirmed local fixes:** public direct/cursor ingestion now rejects expired
claims with transaction rollback; cursor expiry defers users despite fresh
revisions. Standalone receive continues on classified expiry; grouped sends
block without adopting lists. Standalone/group device-query envelopes retain
the original deadline and refuse expired writes before encoding. Existing
prekey guards remain in the typed envelope. [Boundary regressions, coverage and
remaining limits](device-query-deadlines.md). No wire/timeout-length change or
live account operation.

### Device-query durable user/revision ownership (2026-11-06)

**Confirmed local fix:** writer dequeue and ingestion now require the saved
uncertain query's users/revisions to exactly match the original callback.
Changed/missing bindings refuse before encoding or application; historical
missing revisions are not inferred. Newer pending notifications remain distinct
from the original query binding. [Regression matrix, snapshot scope and limits](device-query-deadlines.md#durable-userrevision-ownership-2026-11-06).
No repair, new retry policy or live operation.

### Key-batch continuation validates saved answered evidence (2026-11-06)

**Confirmed local fix:** reserve_next now re-decodes and validates every saved
Answered reply before reserving another page or returning completion. It shares
exact reply/query/target/bundle checks with assembled. Corrupt earlier wire can
no longer authorize an extra query from its status flag alone. Stored summary
counts remain diagnostic. [Regression matrix and scope](device-key-batches.md#saved-answered-pages-cannot-authorize-continuation-by-status-alone-2026-11-06).
No retry, pin replacement, wire change or live operation.

## Directory result dispatch (2026-11-06)

Confirmed static APK43.72 outer-type dispatch now governs shared bundle and frozen-page adoption. A root error inside a result no longer overrides its validated device list. Per-user refusals and cryptographic checks remain. [Evidence, regression and remaining diagnostic attribution gap](device-key-batches.md#directory-result-and-error-branch-selection-2026-11-06). No live requests.

Peer directory result/error attribution now follows saved outer type and selected user independently of generic action status. Typed peer-key root-error fallback corrected; historical outcomes retained without refresh/replay authority. [APK offsets, tests and remaining identity-only/foreign-action limits](peer-identity.md#peer-directory-inspection-uses-reply-context-2026-11-06). Offline only.

Identity-only reply context and signed error codes now match the pinned43.72 FeM/FUP/0ln path. [Trace, regression, API type widening and remaining timer/adoption limits](peer-identity.md#identity-only-reply-context-and-signed-codes-2026-11-06). Offline only.

Full-directory action refusal now binds to the original selected peer instead of first foreign nested error. Shared context guard, ordering/reopen tests and negative duplicate-list refresh check: [audit](peer-identity.md#full-directory-action-errors-belong-to-the-original-peer-2026-11-06). Historical Error and other-family semantics unchanged; offline only.

Local directory refresh and inspection now validate original single-peer request ownership, action kind and reply correlation before relying on signed success. [Eight corruption/reopen cases, unchanged state and limits](peer-identity.md#explicit-directory-refresh-retains-request-ownership-2026-11-06). Offline integrity guard, not an added server requirement.

Identity-refresh staging/application now validates producer request ownership and freezes exact request bytes plus action kind across derivation/commit. [Confirmed race regression, local request-shape checks and remaining prior-notification gate](identity-refresh-application.md#original-request-authority-across-staging-and-commit-2026-11-06). No live operation or new server requirement.

Prior identity-notification success gate now validates action kind, original request/device and correlated reply before another read. Removed the last reply-only success helper. [Seven reopen/unchanged-state cases and native-policy limits](identity-notification-key-read.md#prior-notification-success-retains-request-ownership-2026-11-06). Offline only.

Identity-refresh typed action/inspection now follow pinned43.72 outer-type and exact-device error context; signed codes retained as i32. [Verified writer/callback hashes, end-to-end guarded adoption controls and limits](identity-refresh-application.md#identity-refresh-resulterror-context-2026-11-06). Offline only.

Outer typed directory refusals now take generic action diagnostics only from one enclosing root error, never nested device errors. [Pinned callback sources, missing/duplicate-root matrix, reopen and no-replay controls](peer-identity.md#outer-directory-errors-cannot-inherit-nested-refusal-diagnostics-2026-11-06). Scope is diagnostic; no request/retry changes.

Identity-only callback now has its pinned32-second session timer, anchored before send; expiry preserves uncertain Sent and never recreates retry authority. [Producer/registration evidence, PN/LID paused-clock cases, competing claims and writer limits](peer-identity.md#identity-only-callback-lifetime-2026-11-06). Offline only.

Current43.72 identity ACK dispatch re-anchored through15v/Fc1/17o/0YM/0v6 plus held-key lookup: normal ACK does not require bundle adoption, and general exceptions differ from LIL. [Complete methods, negative comparison and scheduling/routing limits](identity-notification-ack.md#current-apk-identity-dispatch-and-ack-independence-2026-11-06). Research only; no ACK widening or live operation.

Current identity-triggered manager flags and suppression scoped: conditional PQ fetch is not a session upgrade; REGULAR/PQ differ in in-flight keys; strict60-second uptime cleanup is not durable retry authority. [Complete current methods, gates and remaining job/wire limits](bulk-prekey-refresh.md#current-notification-fetch-flags-and-manager-suppression-2026-11-06). Research only; no additional request/replay.

Concrete bulk future/runner trace closes a prior1ue-effects gap: IQ result/error fallback can complete the job normally; disconnect/future exception is separate from server refusal and key acceptance. [Typed future, runtime-exception rethrow, requirement/defer and cancellation limits](bulk-prekey-refresh.md#bulk-job-completion-is-not-fetched-key-acceptance-2026-11-06). Research only; no retries added.

## Media callback-window follow-through (2026-11-06)

[Media refresh uses its pinned nominal32s callback window](media-network-outcomes.md#media-refresh-reply-window-2026-11-06)
with a pre-write absolute deadline; configured shorter socket bounds remain
honored. Failing-before encrypted31s response,32s expiry,30s I/O cap and reopen/
no-resend controls retained. Selected callback/registry methods do not establish
a sender whitelist; complete upstream sender admission remains open.

## Captured caller timing and direct query ownership (2026-11-06)

Audited the remaining generic captured-wait call sites (direct/full key query,
group membership, usync, sender-key distribution/message ACK and retry refetch).
The identity-only32s timer does not establish a full-prekey timeout; no guessed
family deadline was assigned. **Confirmed local defect/fixed:** the experimental
direct CLI bypassed the normal outbox reservation before querying keys. It now
uses the existing frozen-target durable key-batch path. [Reservation-before-write,
legacy-cache boundary, response correlation and reopen controls](replay-safety-audit.md#experimental-direct-cli-key-query-reservation-2026-11-06).
Complete timing contracts for these other families remain open.

## Full prekey failure ownership (2026-11-06)

[Selected full-key callback cleanup and provider bridge](retry-answers.md#full-key-cleanup-and-future-failure-are-separate-2026-11-06)
now distinguish no-op Dxh reader cleanup from the job's separate FfK completion/
failure map. Exact0Th.A03 failure callers and conditional shared provider0x90
are retained. This narrows lifecycle ownership without inventing a32s full-key
callback timer, clearing durable reservations or copying native retry loops.
Global provider-reset/account-switch and complete watchdog policy remain open.

## Android analysis runtime recovered for fresh CLI (2026-11-06)

[Scoped Java-path correction and successful REA evidence](apk-analysis.md#rea-android-java-path-repair-and-cli-proof-2026-11-06)
restore a working fresh CLI using already installed Java/JADX. Current MCP remains
on its old environment until reconnected; no client-wide restart was attempted.
The provider-cache accessor audit narrows the prior reset uncertainty to selected
initialization paths without inventing account-switch or replay authority.
No protocol or live-account behavior changed.

## Blocklist recovery contains a separate MEX request (2026-11-06)

[Current pinned migration obligation](chat-actions.md#blocklist-migration-request-and-state-authority-2026-11-06)
connects selected dirty-list merge/lookup, persisted addressing mode, distinct
MigrateBlocklistLid input/response and conditional cache adoption. This closes
selected call/field links while retaining the absence of a Rust sender and
complete mode/identifier/callback authority. Native isRollback completion can
log without either mode or cache writes. Do not replay the original set or
promote a current cached mapping into migration permission. No runtime change,
new timer, automatic fetch or live operation.

## MigrateBlocklistLid values and error/timeout boundaries (2026-11-06)

[Selected value/coroutine audit](chat-actions.md#migration-values-and-coroutine-completion-2026-11-06)
separates null-retaining intermediate GraphQL maps from null-skipping primitive
wrappers and untraced final serialization. Literal-false error policy forwards
failure; selected non-Pando send inherits32000ms, while Pando/outer wait and
writer/account authority remain open. No universal retry timer, cancellation,
new Rust request or live result is inferred. No runtime behavior changed.

### Blocklist location-disable producer boundary (2026-11-06)

**Confirmed selected static:** [full selected methods](apk/2.26.43.72-blocklist-location-disable/README.md)
connect changed-contact observers to a conditional live-location cancellation
and queued outbound notification. Admission requires an existing share record;
the job rechecks sharing before using its saved sequence as disable.id and a
fresh independent outer request ID. A0C/81 future completion follows the retained
reply-discarding contract. Full retry, persisted work recovery and native state
authority remain open. No speculative Rust packet or live operation added.

### Blocklist result dispatch and direct error attribution (2026-11-06)

**Confirmed APK mismatch / fixed offline:** [callbacks and regressions](apk/2.26.43.72-blocklist-reply-dispatch/README.md)
keep owned fetch/block/unblock outer results on their list-processing path,
ignoring unrelated error extensions for settlement. Invalid list admission is
still refused separately. Outer errors select the first direct coded child,
skipping uncoded children and nested user failures. Native set-only400 plus
failed migration admission can clear preferences and queue a full fetch;
missing Rust authority means no such automatic recovery is invented here.
All three action kinds have encrypted-store reopen regressions. No live test.

### Signed blocklist numeric diagnostic (2026-11-06)

**Confirmed static / implemented offline:** [native parseInt helper and tests](apk/2.26.43.72-blocklist-reply-dispatch/README.md#signed-blocklist-error-diagnostic-follow-up)
bind signed32/default0 semantics to the first direct coded blocklist error.
ActionOutcome.blocklist_error_code persists that value separately from generic
u16 compatibility diagnostics; CLI exposes it. Old rows are not backfilled;
other producers/results remain outside this field. No automatic400 recovery or
request policy changed. Native migration authority remains unported.

### Privacy duplicate raw values precede semantic validation (2026-11-06)

**Confirmed APK mismatch / fixed offline:** [producer/consumer evidence and
reopen/session tests](account-privacy-notifications.md#duplicate-category-values-are-validated-after-selection-2026-11-06)
show19N builds raw HashMap entries before0oe validates final values. Rust no
longer rejects an overwritten unsupported textual readreceipts value before its
supported replacement. Required metadata/body/envelope/category-name guards,
final enum admission, pending setter checks and atomic state-before-ACK remain.
No refresh, broader category or generic account-sync ACK is introduced.

### Privacy setter error is not pending ownership completion (2026-11-06)

**Confirmed selected static / bounded Rust fix:** [setter and callback binding](apk/2.26.43.72-privacy-setter-pending-lifecycle/README.md)
show0oe.A0P inserts pending desired category before dispatch and binds a false
continuation flag;8R5 failure returns through6pB without retiring it. Rust no
longer releases its unowned generic setter guard just because a saved outer
IQ refusal exists. Fresh notifications remain captured/held, receive continues,
and the refusal/status/reply survive. Typed category-specific lifecycle and
full reconciliation remain open; no broader runtime setter or retry added.

### Privacy optimistic display does not confirm a request (2026-11-06)

**Confirmed selected static:** [observer/value and receipt-preference boundary](apk/2.26.43.72-privacy-optimistic-state/README.md)
separates category-list notification, pending integer overlay and stored
read-receipt Boolean. A setter can notify before confirmation; neither the
selected observer nor getter establishes successful remote adoption.23 literal
references are not23 writers; analytics/navigation/comparison hits are bounded
negatives. Remaining listeners/writers/recovery stay open. No runtime change.

### Current read-self ACK privacy side effect and corrective request (2026-11-06)

**Confirmed selected static:** [twenty exports/excerpts and two reused methods](apk/2.26.43.72-read-setting-sync/README.md)
connect root readreceipts on selected read-self ACKs to Android arg1 event227,
normal enabled provider3033/15E and a local-off-preserving truth table. Mismatched
localfalse/serverall calls the separate classic-or-MEX setter with none;
localtrue/servernone writes false. This is not ACK-of-ACK, ordinary receipt
callback completion or generic category-map adoption. Runtime mode/DI/legacy
registrations and durable corrective ownership remain open. No Rust send added.

### Privacy MEX mutation has its own input and success contract (2026-11-06)

**Confirmed selected static:** [setter input/response and exact persisted IDs](apk/2.26.43.72-privacy-mex-set/README.md)
identify SetPrivacySettingMutation with setting:{feature,setting} Enum.name
input, distinct from the GET query_input/own-LID model. A response needs the
successful mutation model and its separate null/translation rules before a map
reaches pending reconciliation. Failures do not turn into native classic error
codes. No Rust request, receipt preference or automatic correction is added;
full SDK/mode/recovery remains open. Typed receipt ACK settlement does not own
the unimplemented global read-self setting side effect.

## MEX mutation intent guard (2026-11-06)

**Confirmed static / offline tested correction:** the [SDK wire chain](apk/2.26.43.72-privacy-mex-set-wire/README.md)
serializes SetPrivacySettingMutation as w:mex IQ/get with direct query
query_id `26887749497493182`. The previous classic privacy/set-only guard missed
this form. The same transactional unowned-intent hold now recognizes the exact
operation ID on a direct query child. Queued, sent, Ok and Error outcomes stay
held across encrypted-store reopen; outer IQ settlement cannot substitute for
native pending-category reconciliation. Existing processed redeliveries re-ACK
without applying old state. Getter IDs, nested query IDs and body text matching
the mutation ID do not claim setter ownership.

The regression was red before correction, green after; the actor test covers
both refused wire forms, continued ping handling and no notification ACK,
adoption or resend. Full workspace:1534 tests, nine suites including doc tests.
No new sender, replay, timer, automatic refresh or live-account operation.
Native category-keyed owner, effective protocol selection and success recovery
remain unported; this conservative recognition is broader than native ownership.

## Photo error callback outcomes (2026-11-06)

**Confirmed static / implemented offline:** [pinned APK43.72 callbacks](apk/2.26.43.72-photo-error-callbacks/README.md)
Hyd.Bvj and6p1.Bvj use1ud.A00 (lookup through6oj.A05), selecting the first
direct error with a present code, skipping uncoded children. A present invalid
code yields0 rather than searching later codes. Rust's generic nested-error
search could instead select nested401 ahead of actual direct404; the regression
failed before correction and now passes for the actual upload/lookup producers.
Only matching saved set/image/bytes or get/picture/image/query=url wire forms
receive this rule; a mislabeled ping does not.

Optional ActionOutcome.photo_error contains native signed32 code and
empty_lookup. The latter is true only for supported lookup404/410, which calls
8pX.CAe with empty H5d metadata in the APK; other lookup codes callCAd, and
upload codes callITY.onError without this special branch. The outer action
status stays Error and generic error_code remains the u16 compatibility field.
This diagnostic is visible through actions text/JSON and persists atomically
with the original reply. Legacy absence remains unknown and is not backfilled.

Tests cover competing nested/direct/uncoded errors, first-invalid-code precedence,
missing/empty/overflow/signed/BMP-digit conversion, upload versus lookup404/410,
duplicate settlement, legacy serde and encrypted-store reopen with no resend.
No native photo/cache adoption, image transformation, request construction,
source/callback lifetime, retry, timer or HTTP-route behavior was added. Downstream
ITY/8pX state effects and the meaning of the saved callback long remain unknown.
No live account operation or claim that these codes cause account restrictions.

## Photo lookup result extensions (2026-11-06)

**Confirmed static / implemented offline:** [lookup CK4 and named-child proof](apk/2.26.43.72-photo-lookup-result/README.md)
select the first direct picture by name, independently of unrelated siblings,
and never route root/nested error extensions to Bvj. Missing picture constructs
empty metadata with the saved request ID input; URL/id parsing still has its
own LIL failures. The new constructor binds that ID field and shows the saved
callback long originates in SystemClock.elapsedRealtime(), not a timeout value.
Its downstream uses remain untraced.

The old generic action parser incorrectly classified actual photo lookup results
as refusals on unrelated error extensions. Matching saved lookup wire shapes now
keep their outer-result outcome. Result-with-picture, preceding metadata,
missing picture and later duplicate pictures retain the raw reply and settle
once across reopen; no resend. Both kind-only upload/lookup negatives retain
the generic parser. Outer photo error classification and other producer policies
remain unchanged.

ActionStatus.Ok is an envelope/recognized-refusal outcome, not a claim native
URL/id validation, cache adoption or photo publication succeeded. That admission
and downstream state port remain unfinished. No timer, ACK, refresh, retry,
query or live operation was added.

## Individual report callback timer (2026-11-06)

**Confirmed static / implemented offline:** [current RPC-to-timer chain](apk/2.26.43.72-report-callback-timer/README.md)
binds IndividualSpamReportRpc -> SpamRPCManager.A01 -> MessageClientSmaxWrapper.A00,
which supplies32000 to0Sf.A0F. Cgd selector2 maps delivery failure/expiry through
BWM to null. Shared0ln registration requires positive duration to post message4.
Rust previously left this report callback claim untimed. Supported individual
spam/set/spam_list requests now use the existing32s monotonic callback deadline.
At expiry only the live claim ends: durable Sent/uncertainty stays, late replies
cannot settle through that expired owner, and restart does not requeue the report.
No inferred native offline queue/retry/coroutine-cancellation policy was added.

The regression was red with no deadline, green with result/error at31,999ms and
expiry at32,000ms, unchanged saved state on late reply, reopen and no resend.
A report label on a ping cannot borrow the timer. No live operation.

The current generated response path locates report/id via Shk.A00 ->7NL,
optionally null; manager tries typed success before its error model. Full native
schema/correlation/source/error adoption and report-ID projection remain unported.
Generic Ok does not prove native report acceptance or moderation outcome.
The current flow fragment has100 schema entries; Rust's seven flows are a local
subset, not the complete native list. List lower bounds and effective optional
metadata were not newly reconstructed, and no message/trace field is invented.

## Report response and list bounds (2026-11-06)

**Confirmed static / implemented offline:** [current list and response readers](apk/2.26.43.72-report-response-bounds/README.md)
resolve the previous lower-bound unknown: B23.A03 returns0, so Dzb's declared
list bounds are0..210,0..5,0..5. Empty lists are permitted by this constructor;
no server acceptance follows. Out-of-range nonnull lists log an AssertionError
through00L.A0C and still return true, so schema diagnostics must not be described
as hard server enforcement. Rust still supplies no reported-message children.

Rv2 reads request id then checks response id equality and type=result through
8B6.A0J. Shk's report/id uses1..20 UTF16 code units, but a missing/invalid ID
produces a nullable model that the selected RPC can still wrap as success with
null report ID. It must not be turned into a guessed whole-IQ refusal. The
from reader uses generic Jid/UserJid assignability; this constructor does not
establish a server-only source requirement. Full native source/schema/duplicate
policy and report-ID projection remain unported.

The old generic action parser incorrectly selected unrelated root/nested error
extensions in a valid report result. Matching saved report requests now preserve
their outer-result outcome. Red/green regression covers live ownership,
root403/nested401, original reply, duplicate refusal, reopen and no resend;
kind-only ping negatives keep the generic parser. Generic Ok is not native
schema admission or moderation acceptance. No request, ACK, retry, source guard
or live operation was added. Existing32s callback expiry remains authoritative.

## Current target types and outbound guard (2026-11-06)

**Confirmed static / implemented offline:** [APK43.72 writers and target hierarchy](apk/2.26.43.72-presence-targets/README.md)
re-anchor composing/audio/paused and the subscription writer. Current1bc is
concretely GroupJid/g.us; DeviceJid and server-only0mL are separate Jid types.
The native schema checker logs type mismatch, not a proven server ban rule.
Rust's supported named primary-chat API previously admitted empty-user server
addresses and AD forms when numeric device was0. chat_target/chatstate/subscribe
now require nonempty user and !is_ad alongside their existing server set;
subscription context gets the same named non-device restriction. Explicit
AD0/0 wire kind remains distinct from a plain primary identity. No stripping or
global JID-parser change. CLI validation occurs before transport::run.

Red/green regression covers empty PN/LID/group, agent-only/device-only/AD0/0,
invalid group contexts and valid primary/token cases. Existing codec round trips
retain supported wire construction. Native group subscription remains outside
the existing people-only Rust subset; no feature was added.

DEd's2500ms message2 pause scheduling and strictly>10000ms event suppression
comparison are retained with their0AI.A06/queued-message/bot-state gates. They
are not a mandatory heartbeat. Effective gate meaning, upstream activity/cancel,
full scheduler/restart behavior and the bot/interop branches remain unported.
No extra network request, ACK, automatic timer traffic or live operation.

Current0Tj.A03/A04 additionally confirm smax:any token wrappers merge into a
plain tctoken byte child. The existing Rust subscription wire form needs no
change; other merge branches and complete token-issuance policy remain separate.

## Incoming chatstate routing (2026-11-06)

**Confirmed static / implemented offline:** [current1rB reader and dispatch](apk/2.26.43.72-chatstate-routing/README.md)
select child0, unwrap device addresses through0Dn.A00, cast participant to
UserJid and enqueue Android messages20/21. Reused current device/type evidence
binds ordinary PN/LID devices17/19; class/factory mapping is not inferred from
arbitrary suffixes. Rust previously retained these device-qualified addresses
and known non-user participants in ephemeral metadata. Supported PN/LID now
unwrap to plain user metadata; group/server-only PN/LID authors become None,
and a supported server-only from is omitted because the metadata needs a chat.
Original Node and stored identities are unchanged. Incoming unwrapping does not
loosen the outbound device-target refusal.

Red/green regression covers PN/LID device/agent/AD0/0, group/server authors,
valid group chat, untouched input and first-child refusal despite later
composing. Other JID domains retain their prior metadata behavior pending full
class mapping; this is not a complete native admission/trust validator. Native
null-chat downstream handling and event consumer policy remain untraced.
No journal/state adoption, ACK, refresh, request, replay or live operation.

## Hosted incoming typing identities (2026-11-06)

**Confirmed static / implemented offline:** [hosted factory/constructor/reader chain](apk/2.26.43.72-hosted-chatstate/README.md)
separates the original hosted device domain from its underlying non-hosted
UserJid.0De.A02 maps hosted to PhoneUserJid and hosted.lid to0Dx; device99
constructs Dv4/Dv6 and incoming0Dn.A00 returns their parent userJid. Rust now
projects these supported incoming aliases to ordinary PN/LID metadata. Explicit
hosted device forms require99; no unsupported explicit device observation or
author is fabricated. Plain model aliases and valid device99 text are covered.

The original Node and text/typed attributes remain unchanged. This projection
must not be applied to device hashes, stored identity/alias authentication,
outbound targets or token lookup. No such paths changed. The earlier hosted
hash/domain distinction remains valid. Red/green tests cover both domains,
plain/99/group-author/text inputs, original attributes and explicit device0/2.

Full text grammar (including pre-entry zero-qualifier normalization), all JID
factories and other special families remain unported. This is ephemeral metadata,
not a trust/admission validator. No ACK, request, replay, state adoption or live
account operation.

## Hosted textual zero qualifiers (2026-11-06)

**Confirmed static / implemented offline:** [qualifier-presence trace and regression](hosted-chatstate-text.md)
show native0Dd.A00 chooses a device path on dot/colon presence before examining
numeric values. The shared Rust parser had flattened zero components to Plain,
allowing invalid hosted device0 metadata to look like an unqualified alias.
The incoming-only raw-text reader now retains AD wire kind for qualified hosted
text; the existing device99 check omits invalid observations/authors. Plain
aliases, valid99 text, typed AD handling and PN device0 projection remain.
Original frames/typed inputs, shared JID parsing, hashes and saved identities
are unchanged. Pre-collapsed typed values and full native text/agent grammar
remain an open boundary. No ACK, request, state adoption or live operation.

## Current PSA deferral gap (2026-11-06)

**Confirmed static / unresolved runtime obligation:** [current PSA branch matrix](psa-ack-lifecycle.md)
refutes universal plain-base-ACK wording. Handler104/19n transfers ownership
before feature-gated surfaces/wakeup and other zero-sender work; current1844,
revoke/campaign and message ingestion branches differ from the Rust helper.
487 is conditional, and the message worker can throw before normal completion.
Effective feature settings, worker failures, persistence/recovery and full ACK
correlation remain required. Runtime ACK policy is unchanged; no guessed flag,
extra487, blanket hold or fabricated completion was introduced. No live request.

## PSA watchdog and wakeup completion (2026-11-06)

**Confirmed static:** [current wakeup/scheduler follow-up](psa-ack-lifecycle.md#wakeup-completion-and-scheduler-follow-up-2026-11-06)
reuses the already-retained diagnostic watchdog: tracking-map cleanup neither
sends ACK nor consumes a held completion token. Declared scheduler methods
bind6000 to Handler.postDelayed milliseconds and cancel returned wrappers.
Waking worker27 normally completes after promotion selection/rendering method
return; its selected Exception path uses487. No remote fetch or queued-render
success is established by that call. Effective flags, wrapper/provider/runtime
lifecycle, native application and recovery remain open. Runtime policy unchanged.


### Receipt recipient text-class admission (2026-11-06)

**Confirmed local fix / current static class evidence:** [receipt recipient audit](receipt-user-recipient.md#current-text-class-admission-including-dot-qualifiers-2026-11-06)
reuses current F0R.A03 UserJid getter, parser/factory and class ancestry. Raw .0
or ignored dot suffixes previously became plain-user recipients; invalid ordinary
PN/LID text could also pass. ACKs echoed those recipients and fact routing skipped
matching ordinary receipts. The shared getter now uses the existing native
PN/LID/hosted classifier. Optional invalid recipient takes the null path while
original target/actor/attempt/uncertainty and callback settlement boundaries stay.
Both ACK and fact regressions failed before correction; encode/decode/reopen
controls verify the correction offline. Binary-zero and valid users remain.
Full current fact dispatch, special families and live acceptance remain open.
No global identity normalization, new request, replay or account operation.


### Binary user-recipient precedence and hosted output (2026-11-06)

**Confirmed local fixes:** [receipt user output and message ACK correlation](receipt-user-recipient.md#hosted-user-output-and-binary-recipient-precedence-2026-11-06)
now reuse native UserJid projection for known zero-component binary recipients.
Plain hosted receipt recipients render their underlying PN/LID address. Binary
AD0 message recipients retain UserJid precedence rather than falling back to an
unrelated from; hosted binary pairs can match the original PN/LID chat. Both a
false settlement and missed settlement were demonstrated by failing outbox
regressions. Current pair/AD reader, factory/getServer, receipt descriptor/writer
and1rH precedence evidence are linked. New tests retain original entry/journal,
matched success/refusal, deduplication, reopen and no-resend behavior. No global
normalization, alias trust, new request or account operation. Other typed/from
families and complete native/live parity remain unverified.


### Carrier-independent native long-PN ACK admission (2026-11-06)

**Confirmed local fix:** [long phone-user recipient correlation](message-ack.md#long-phone-user-recipient-admission-is-independent-of-carrier-2026-11-06)
previously accepted native-valid long text but refused the equivalent binary
primary/hosted pair or AD0 recipient. The projected user now passes the same
native constructor predicate irrespective of carrier. The extended outbox
matrix failed before correction; it covers valid20/1008-prefix and invalid21,
leading-zero and prohibited10-prefix cases, opposite from, success/refusal,
state/journal preservation, duplicates, reopen and no resend. LID/special/decode
policy limits remain explicit. No new target builder, replay or account operation.


### Binary ACK from constructor/order correction (2026-11-06)

**Confirmed local fixes:** [binary from construction before chat projection](message-ack.md#binary-from-construction-precedes-chat-projection-2026-11-06)
now follows the native selected pair/AD factory before projecting a PN/LID user.
Hosted binary from reaches its underlying chat; invalid device100/128/255 cannot
be erased into primary authority or bypassed through a valid recipient. Retained
reader/joiner/constructor/class/dispatch evidence supports the order. The new
real-wire outbox matrix covers four domains, seven carrier/device forms,
matching/mismatching from, recipient precedence, success/refusal, unchanged
entry/journal, one terminal fact, duplicates, reopen and no resend. Three failing
runs exposed the missed match, false settlement and qualified bypass. Other
attributes/typed families and global decoder/error policy remain open. No live
operation, identity merge, extra request or retry was introduced.


### Binary optional ACK field admission and class separation (2026-11-06)

**Confirmed local fixes:** [common binary ACK address classes](message-ack.md#binary-optional-address-construction-and-participant-class-2026-11-06)
now gate constructor-invalid from/participant/recipient before message route
selection. Hosted binary participants cannot bypass the ordinary-success guard;
invalid participant/refusal and invalid recipient/fallback cannot settle through
an otherwise matching route. Valid Device/NonUser/server recipients retain the
native failed-UserJid-cast fallback, and ServiceUser/NonUser participant conversion
remains separate. Two failing actual-wire runs exposed false Accepted and Refused
classifications. New expected-chat/outbox tests cover constructor/cast controls,
nullable text contrast, unchanged entry/journal, one outcome, duplicates, reopen
and no resend. Other typed families and receipt/global decoder policy remain
explicit work; no identity merge, extra request, retry or live account operation.


### Bare self-pair decode, ACK conversion and receipt authority (2026-11-06)

**Confirmed local fixes:** [bare self-user integration](message-ack.md#bare-self-user-pairs-and-carrier-independent-guards-2026-11-06)
recognizes two native empty-user JID_PAIR names without widening general server
or qualified-alias grammar. Conversion and bare call sender guards now share
text/binary classification. Native-invalid binary call senders retain capture,
consumed epoch and no replay. Self objects remain unwritable notification routes;
known receipt self qualifiers/actors cannot become ordinary read/delivery facts
or ACKs. A textual self recipient previously recorded1 read fact. Literal codec,
actual wire, actor/state preservation and reopen tests retain these boundaries.
Full native aggregate scheduling, other classes and decoder recovery remain
open. No live account operation or automatic request family was introduced.


### Selected automatic ACK writer compliance (2026-11-06)

**Confirmed local fix:** [mapped noncompliant classes and complete declaration census](apk/2.26.43.72-jid-write-compliance/README.md)
now prevent automatic notification/receipt ACKs containing known writer-rejected
objects. Type2/8/9/11 from/participant and selected UserJid recipients are guarded;
omitted nonuser recipients and fresh retry parse-error metadata do not cause a
blanket refusal. Device-backed service users remain allowed. Both actual-codec
regressions failed on previously constructed service-user ACKs. This preserves
native context/cast distinctions while leaving general grammar, codec policy,
writer recovery and live acceptance explicit. No new request, receipt fact,
identity normalization or account operation.

## Literal zero-device ACK class (2026-11-06)

**Fixed offline:** [source-aware ACK projection](receipt-primary-ack.md#literal-zero-device-ack-wire-class-2026-11-06) preserves literal LID Device0 AD-JID output separately from binary-zero UserJid pair output. Shared notification and receipt paths are covered; metadata/fact canonicalization, wrapped original device integers and live acceptance remain open.

## Original device getter range at ACK writing (2026-11-06)

**Confirmed static / fixed offline:** [complete byte writer and outer path](apk/2.26.43.72-jid-device-byte-range/README.md)
resolve the wrapped-device writer gap. 1r2.A03 requires 0..255; 1r2.A00 passes
original getter integers, not narrowed constructor bytes. Positive 256/355 PN/LID
and negative LID values that pass construction can therefore be unwritable.
The selected-output guard now refuses these ACK fields. Omitted device-valued
recipients remain harmless; a valid negative PN device instead takes JID_PAIR
because its original getter is nonpositive and agent is zero. A bounded raw-text
projection reproduces that pair without changing captures or existing selectors.

The field regression failed on an ACK built with 12345:256@lid before correction.
The receive actor exercises refusal followed by a ping barrier, and capture-before-
write for valid negative PN and exact LID zero. Global encoder/model, native
exception recovery, constructor-null hosted strings, other classes and live
acceptance remain separate; no account operation or new request family.

## Nullable receipt context and participant output (2026-11-06)

**Confirmed static / fixed offline:** [complete descriptor construction trace](apk/2.26.43.72-ack-nullable-context/README.md)
shows F0R.A00/Efa.A00/Eoh do not assert a sender or participant; F3e.A05 omits
null fields. This does not establish every handler's null-sender admission.
The supported Rust ACK path retains its sender requirement. Its optional
participant now honors known constructor-null PN/LID/hosted text rather than
copying the broader generic parser's result. Nonnull Device256 writer failures
remain distinct and still refuse ACK construction. Regression/actor checks cover
actual encoding, durable capture and receive continuation. Fact attribution,
notification handlers, typed constructors and live acceptance remain open;
no account operation, stored selector or new request-family change.

## Native-null participant and ordinary direct facts (2026-11-06)

**Confirmed selected static / fixed offline:** [current parser, event and consumer trace](apk/2.26.43.72-receipt-null-fact-actor/README.md)
follows F10.A03 -> FSI -> F21.A05. The ordinary outgoing direct branch uses
its remote sender device; constructor-null participant text cannot supply a real
participant target. Direct Rust receipt parsing now honors known native null
before generic parsing, reusing its existing authenticated sender fallback.
An expanded encrypted-store regression failed before correction with zero facts
instead of one; PN/LID delivery/read/played controls now retain correct actor,
timestamp, send intent/attempts/progress, reopened persistence and idempotency.
Valid participant target checks, global/aggregate/group/retry parsing and request
behavior remain unchanged. Complete status/feature/storage branches, required
sender grammar, binary constructor fidelity and live acceptance remain open.

## Constructor-null aggregate actors skip before timestamp parsing (2026-11-06)

**Confirmed selected static, APK 2.26.43.72 / fixed offline:** the retained
[complete F10.A03 parser](apk/2.26.43.72-receipt-null-fact-actor/receipt-parser.txt)
reads each key-aggregate item using 0Tp.A0A(Jid,jid) at byte PC035e. A null result
branches at0366 to03a4, advancing the item index. Only a nonnull generic Jid
reaches DeviceJid conversion at036a and timestamp parsing at0372. The relative
branch is in code units: 0366 + 2*0x1f =03a4. The prior
[context/constructor evidence](apk/2.26.43.72-ack-nullable-context/README.md)
and [native constructor trace](apk/2.26.43.72-lid-device-ack-text/README.md)
bind known PN/LID/hosted text failures to null.

This differs from an ordinary direct receipt's nullable participant: an invalid
aggregate item is omitted entirely, not expanded with an absent participant and
then attributed to the sender. A nonnull generic object whose DeviceJid
conversion returns null still reaches timestamp parsing; no general
"conversion-null skips time" rule is inferred or implemented.

**Confirmed local mismatch:** key_aggregate previously accepted constructor-null
text via the generic model, then parsed its malformed timestamp and refused the
whole aggregate, including valid neighbors. It now uses the existing native text
null classifier before generic parsing/expansion. It leaves raw captures, root
metadata, existing selectors and valid actor handling intact.

The expanded `key_aggregates_use_inner_key_root_kind_and_optional_target_times_once`
regression failed at validation before correction. Its actual codec-roundtripped
aggregate now includes PN device100, LID128 and hosted-LID355 with malformed
item times, alongside valid device2 and primary rows. It records exactly the
two valid inner-key read facts with their proper device/time, leaves the envelope
send and original intent/attempts/progress unchanged, survives encrypted-store
reopen and adds no duplicate fact or journal entry. Existing valid-address
malformed-time controls retain full-table atomic refusal and error redaction.
These atomic controls verify Rust policy, not every native worker/storage path.

This is the key-aggregate branch only. Offline message_id aggregates use a
required DeviceJid accessor and keep separate failure rules. Typed binary
constructor fidelity, nonnull generic-to-device conversion details, all group
and feature-dependent native storage paths, and live acceptance remain open.
No new request, identity/trust authority or account operation.

Reproduce with `cargo test --locked -p whatsapp-api --all-features key_aggregates`.
The retained complete parser's reproduction and checksums are in its
[existing evidence bundle](apk/2.26.43.72-receipt-null-fact-actor/README.md);
no new export or duplicated evidence bundle was necessary.

## Required DeviceJid in offline aggregate items (2026-11-06)

**Confirmed selected static / fixed offline:** [complete required accessor and conversion predicate](apk/2.26.43.72-offline-required-receipt-actor/README.md)
bind F10.A03's message_id branch to A0B(DeviceJid,jid). Constructor-null or
failed class conversion throws LIL instead of skipping an item. The supported
Rust parser now refuses known text constructor/cast failures, generic non-device
families and bounded ordinary binary failures before expanding any batch facts.
Key-aggregate nullable skip behavior remains distinct. A codec-roundtripped
mixed valid/invalid regression failed before correction; now validate/record
refuse with unchanged checkpoint/outbox rows and journal and redacted errors.
Native ordinary parse-error response dispatch, full class/cache/binary fidelity,
valid-device projection and live acceptance remain open; no new request or account
operation. This guard does not assert universal native NACK or rollback behavior.

## Ordinary parse-error convergence and corrected send gate (2026-11-06)

**Confirmed selected static / runtime implementation gap:** [ordinary handler, wrapper census and queued error path](apk/2.26.43.72-ordinary-receipt-parse-error-dispatch/README.md)
bind F10.A03 to ordinary15H and status19t handlers. Both selected BX2 variants
reach 1rg.A04 for LIL, which builds a fresh error487 context. The queued path
also creates distinct completion bookkeeping. Required offline aggregate actor
failures are therefore candidates for an evidenced error response; current Rust
silence is not claimed to match a native silence policy.

**Corrected earlier evidence interpretation:** 0Sf.A07(true) logs a bypass then
falls through into the same Message76 construction and enqueue path as false.
It does not return without sending. Fresh bytes match the old retained export;
its README and checksum now carry the correction explicitly. The receipt feature
flag in A0S selects this boolean, not a wire-suppression gate in A07. The later
writer bundle already closes the selected Message76-to-F3e.A05 binding.

No runtime changed in this research chunk. The next implementation must carry
confirmed native parse-error causes separately from local limits/storage faults,
use the fresh descriptor without recipient/edit, preserve capture/write ordering
and forbid resend/query/identity work. Queue recovery, unsupported metadata and
remote acceptance remain unverified; this is not permission to ACK all refusals.

## Ordinary required-device error responses implemented (2026-11-06)

**Confirmed local correction / offline**, grounded in the pinned APK2.26.43.72
[ordinary handler/wrapper/error-dispatch trace](apk/2.26.43.72-ordinary-receipt-parse-error-dispatch/README.md)
and [required DeviceJid accessor](apk/2.26.43.72-offline-required-receipt-actor/README.md).
F10.A03 required-actor failures reach 1rg.A04/A03 and its fresh error487
context. This implements the previously documented ordinary response gap for
proven required-device causes, not every local receipt refusal.

### Cause classification precedes output

The pure parser assigns `receipt.aggregate.native_required_device` only to a
missing required actor, known native-null PN/LID/hosted text, known wrong-class
text or generic classes, and the bounded binary service/nonuser/server-only
cast failures. Ordinary binary `Invalid` remains a generic refusal because its
upstream constructor/recovery error route is not established by a class-cast
proof. Local batch bounds, opaque/unknown representations, other unported
schema/time/type failures, self-user policy, and stored-row/storage failures
are NOT automatically mapped to487. The field is an internal diagnostic cause,
not an HTTP route, user-supplied switch or identity authority.

A supported recipient previously caused validate/record to return before
inspecting offline aggregate grammar. Both now preflight only the explicit
message_id aggregate in that no-fact-projection branch. A valid recipient still
does not grant ordinary fact projection, while a confirmed malformed required
actor exposes the same native cause. Other recipient/aggregate paths retain
existing behavior.

### Fresh root descriptor, all selected receive paths

`messaging::received_parse_error_confirmation` is a public, pure, validation-gated
builder: it returns a Node only for that native cause and supported root output.
It performs no capture, store mutation or I/O. Normal connection.rs, the CLI
comparison receive window and group-send post-send window now emit the response
after preserving the inbound frame. A genuine redelivery may receive another
NACK; a stored capture does not authorize local write replay.

The descriptor uses ROOT id/from/type/participant and error487, without
recipient/edit/category or aggregate children. It never substitutes the inner
message_id for the stanza id. Delivery type is omitted; other textual root types
are copied. Known-null root participants are omitted; known noncompliant or
out-of-range selected writer fields suppress construction. An omitted recipient
cannot make the fresh output unwritable. Existing supported root-id and sender
admission remains: this does not yet port arbitrary nullable sender output,
unknown metadata or every native subtype.

The normal driver sets receipt_recorded=false after the validation failure, so
it does not record facts, invoke retry answers or build an ordinary success ACK.
Its error ACK diagnostic is distinct from receipt success/facts. The comparison
window reuses existing outcome_unknown-before-write/written-after-write markers;
write failure ends the window. Group-send error ACK write failure ends its
post-send window without replay or claiming the original send failed.

### Verification and remaining scope

`offline_required_actor_parse_error_gets_fresh_487_after_capture` first timed out
waiting for the missing NACK. It now checks a real UserJid recipient, exact fresh
root descriptor, capture-before-write, repeated server delivery, unchanged
outbox/peer-identity/session snapshots, exclusion of a 4097-item local bound,
a following ping barrier, a synthetic write cut, preserved capture after reopen
and no automatic NACK replay on the next activation. Existing aggregate tests
retain atomic refusal with a valid sibling and unchanged facts/journal.

`ordinary_native_parse_error_cause_excludes_local_and_binary_refusals` binds the
cause distinction. `ordinary_parse_error_descriptor_copies_only_selected_root_fields`
checks literal LID0 projection, selected writer refusals, null participant omission,
codec output and delivery-type omission. Full CLI typechecking covers the two
additional receive call sites; they are not represented as independently observed
network traffic.

Unknown native constructor/queue/recovery behavior, other parse-error causes,
full nullable sender grammar, all request families and live acceptance remain
open. No new query, resend, identity change or live account operation occurred.
Run `cargo test --locked --workspace --all-features -- --test-threads=4` plus
strict workspace Clippy and the existing CLI/OpenAPI/coverage/hygiene gates.

## Aggregate timestamp native error responses (2026-11-06)

**Confirmed static / fixed offline:** [required/defaulted Long parser chain, branch countercheck and regression table](apk/2.26.43.72-receipt-timestamp-errors/README.md)
bind 0Tp.A07/A0L/A09 and A08/A09 to LIL and the existing ordinary fresh error487
path. Missing required offline item timestamps and malformed supported signed-long
item text now have a distinct native_timestamp validation cause; key item absence
still uses its existing clock default. Opaque representations and local/storage
failures are not automatically NACKed. Normal and selected CLI receive paths use
the same cause predicate/builder. Recipient-bearing participants aggregates also
preflight this grammar without granting new fact projection.

The hypothesis that root delivery gates key-aggregate parsing was refuted by
following 028c ->03aa ->02d0: only optional biz/privacy extraction is skipped.
Existing non-delivery key routing and read/played facts remain unchanged. The
new numeric codec regression and extended actor check fresh response fields,
capture-before-write, no fact/identity/session mutation and default/valid-number
controls. Existing atomic-fact and write-cut/no-replay tests remain. Ordinary root
timestamps, complete binary/native clock/storage behavior and live acceptance
remain open. No account operation or new request family.

## Current-build ordinary root time remains lenient (2026-11-06)

**Confirmed selected static / verified offline:** [complete43.72 optional-string, numeric-wrapper and lenient-long chain](apk/2.26.43.72-receipt-root-time-fallback/README.md)
binds ordinary root t to 6oe.A18 ->6ol.A06 ->0HF.A01(default0), not the strict
aggregate item chain. Missing/empty/malformed/overflow text defaults rather than
throwing LIL; blanket487 classification would be wrong. Production behavior is
unchanged. A new codec/encrypted-store regression verifies ordinary read ACKs
without error, no parse-error NACK, correctly attributed facts with unknown time
rather than fabricated supplied0, valid0/BMP controls, preserved send state,
reopen and duplicate idempotency. Complete native row/carrier/other-handler and
live behavior remain open; no account operation.

## Required time precedes offline item status conversion (2026-11-06)

**Confirmed static / fixed offline:** [complete status converter and non-parse error handler](apk/2.26.43.72-receipt-item-status-order/README.md)
bind F10.A03 order: required actor, optional type string/default, required time,
then F10.A00 conversion. Rust now parses required item time before refusing an
unknown kind, so a missing/malformed time is not masked and retains its487
response. With valid time, unknown/empty kind conversion is non-LIL, outside the
487 classifier. Native queued500 versus legacy rethrow remains a separate policy
gap; no new500 response is added. Codec cause/ACK precedence and actor-priority
controls pass alongside aggregate atomic-refusal tests. No new fact kind, query,
resend, identity change or live account operation.

## Missing aggregate key versus local ID bounds (2026-11-06)

**Confirmed selected static / fixed offline:** [identifier selector/read-order, constructor and chat-resolution evidence](apk/2.26.43.72-receipt-aggregate-id-boundaries/README.md)
bind key absence to required A0L before chat resolution, after root-kind selection.
The shared native error cause set now includes native_required_key for the
supported selected key branch and emits a fresh root487 after capture. A valid
message_id branch needs no key. Empty/overlong IDs remain local refusals: A0L
and the selected message-key constructor assert nonnull, not those bounds.
Codec/store controls prevent a local bound from authorizing a native NACK or
falling back to ROOT-ID facts. The actor checks capture-before-write and unchanged
outbox/identity/session state. Chat resolution precedes the required message_id
read but follows key read; the earlier blanket ordering hypothesis was refuted.
No synthesized ID, new fact/identity authority, query/resend or account operation.
Opaque string/root-type policy, full native key/storage/chat/queue behavior and
live acceptance remain open.

## Receipt routing precedes root-type response classification (2026-11-06)

**Confirmed selected static / runtime gap:** [full receipt adapter, sets, message forwarding and negative searches](apk/2.26.43.72-receipt-router-boundary/README.md)
bind inherited1rA.A03 to receipt-specific1rU.A05. Epa.A00 is a diagnostic hash,
not the type classifier, and zero direct14h.BLn callers do not mean no dispatch.
The adapter handles first participants/message_id before ordinary type membership,
then separate call/biz/view/simple/retry/rekey/error cases and unhandled fallback.
Unknown root types cannot be classified from F10.A03 alone.

The admitted unhandled path invokes2Cx with the original context and wire487,
preserving recipient/edit; it is distinct from fresh1rg parse-error487. This is
a confirmed routing/construction gap in Rust, not a runtime change in this chunk.
Future response work must preserve the router's earlier branch selection and
special types before deciding copied versus fresh fields. Feature/provider/queue
and binary completeness, actual traffic and server acceptance remain unverified;
no account operation or new request is performed.


## Upstream routing excludes two ordinary retry paths (2026-11-06)

**Confirmed selected static / implemented:** pinned1rU.A05 forwards a FIRST
participants/message_id before root-type retry selection; empty message_id is
nonnull. category=peer instead selects code263, not ordinary retry code2.
The shared selector now prevents ordinary retry parsing, error487 classification,
handled-success ACK and resend/refetch reservation on both routes. Key-only or
later participants do not select the early route. Nontext selectors remain local
unsupported inputs. Saved-session regression preserves outbox/checkpoint/ratchet/
identity state and emits no resend/query/confirmation for alternate routes.
[Evidence, status277 registration, lenient-offline negative result and limits](apk/2.26.43.72-retry-upstream-routing/README.md).
Full special/unknown-type routing and live acceptance remain open. The router's
error branch tests root type=error before requiring its child; an arbitrary error
child alone does not select it. No live account operation.


## Key-only aggregate selection follows upstream root routing (2026-11-06)

**Confirmed selected static / implemented:** without a leading nonnull
participants/message_id, pinned1rU.A05 forwards only its seven ordinary status
root kinds to F10.A03. A key-only participants child cannot turn retry or another
special/unknown root kind into an ordinary aggregate. Expansion now preserves
those roots for existing consumer policy instead of raising a key-aggregate
unknown-type refusal that prevents valid retry processing. Ordinary missing-key
causes and early message_id precedence remain. Unsupported special consumers
remain unsupported; no new success ACK, fact, unknown-type response or live
operation is introduced. The saved-session workflow exercises validation,
recording and resend/refetch policy with first-count key-only children.
[Exact route evidence and test/coverage limits](apk/2.26.43.72-retry-upstream-routing/README.md).


## Peer route shares parser errors, not ordinary resend handling (2026-11-06)

**Confirmed selected static / implemented correction:** code263 registers17t;
17t.A04 calls the same F10.A04 as ordinary160, then processes peer-message records
through a separate eligibility/age/count/crypto/job pipeline. Its uncaught LIL and
null parser result therefore retain fresh487 and copied487 respectively. Earlier
exclusion of peer from ordinary retry processing was too broad for error ACKs.
Shared parser-route selection now permits proven peer schema/version responses;
ordinary RetryRequest, handled-success ACK and resend/refetch still exclude peer.
Leading participants/message_id retains precedence. Valid peer schema remains
without ordinary success ACK or resend; full peer responder policy is unported.
[Complete handler, registration census, error-context tests and open boundaries](apk/2.26.43.72-peer-retry-handler/README.md).
No live account operation or server acceptance claim.


## Supported unhandled root routes receive copied487 (2026-11-06)

**Confirmed selected static / implemented:** after preceding1rU.A05 routes,
2Cx copies the original descriptor and adds wire487 (local diagnostic490 differs).
Supported ordinary user/group unknown roots now receive that response through the
shared ACK path, preserving recipient/edit and outer id/type, without received
children/category. Ordinary/special types and leading participants/message_id
remain excluded; key-only/later aggregate or unrelated error child do not divert
it. view on supported user/group routes fails the native type21 condition and
also reaches fallback. Local address/string/writer/outer-ID guards remain.
A receive-loop regression proves capture-before-write, explicit-redelivery ACK,
unchanged outbox/ratchet/identity and no uncertain ACK replay after cut/reopen.
[Complete evidence, context/type predicates and remaining boundaries](apk/2.26.43.72-unhandled-receipt-root/README.md).
Full sender/provider/binary/special-handler parity and live acceptance remain open.


## Non-user constructor-null receipt addresses (2026-11-06)

**Confirmed selected static / implemented:** generic receipt Jid reading also
returns null for failed group/broadcast/newsletter/temp/call text construction.
The shared receipt null classifier now reuses the existing APK-backed predicates,
so invalid optional participants are omitted from success/copied487/fresh487
contexts and treated as absent by direct facts. A null source grants no response;
valid-but-unwritable objects and typed binary carriers retain their separate
rules. No generic JID rewrite or identity learning. Codec/fact/receive-loop tests
cover malformed non-user text and explicit redelivery/cut/reopen boundaries.
[Exact constructor links, interpretation and remaining coverage](receipt-optional-addresses.md#non-user-text-constructor-failures-in-receipts-2026-11-06).
No live account operation or server acceptance claim.


## Native-null root/actor guards precede group projection (2026-11-06)

**Confirmed selected static / implemented:** incoming group source/participant
now use the existing text-constructor-null classifier before generic target
parsing. Exact historical chat/target matches cannot grant fact authority to a
known-null incoming object. Known-null root source is checked before aggregate
expansion can erase its text origin; current source-required fact paths leave it
unprojected. Tests cover normal/legacy matching targets through codec/public
record, all three facts, every-table/journal preservation and reopen/redelivery;
malformed group sources cover direct/key/message_id shapes. Existing typed
objects and stored validation/history remain distinct.
[Evidence, native versus local boundaries and remaining scope](group-receipts.md#constructor-null-incoming-addresses-cannot-match-saved-rows-2026-11-06).
No live operation, identity adoption or response-policy expansion.


## Requesting device follows original chat class (2026-11-06)

**Confirmed selected static / implemented:** pinned160.A04 uses context.A02 for
direct original-key chats; broadcast3/5/6 and group1/2/28 instead branch to
context.A01 participant before the shared DeviceJid conversion. The parser no
longer lets an optional participant redirect an ordinary direct retry. It also
preserves known text-constructor null for root/recipient/selected actor rather
than resurrecting generic objects or using a missing group actor's root.
Nonrepresentable native null-device policy remains unsupported, not error487.
The saved-session success/duplicate workflow includes a misleading direct
participant; null-source variants with matching explicit qualifiers preserve all
outbox/checkpoint/ratchet/identity rows and produce no request/confirmation.
[Complete predicates, exact byte/code-unit branches and corrected expectations](apk/2.26.43.72-retry-device-source/README.md).
Original chat/identity/reservation guards remain; no live account operation.


## Preflight retry ACK encoding before eligible direct work (2026-11-06)

**Implemented/offline:** direct retry work now requires a pure supported ACK
candidate AND successful binary encoding before any state read/reservation or
crypto preparation. Previously work could commit before ACK construction failed,
then be returned with no confirmation. Both answer entrypoints share this guard;
no success ACK is exposed until actual work commits. Known constructor-null
optional participants are admitted as omitted fields, while unsupported or
unwritable contexts leave outbox/checkpoint/ratchet/identity rows unchanged.
Saved-session tests cover resend/refetch candidates, actual candidate-versus-codec
failure, and valid count2 refetch with null optional participant. Existing ACK-
before-write, uncertainty/no-replay and group capability policy remain distinct.
[Evidence chain, ordering limits and complete tests](retry-ack-dispatch.md#ack-capability-preflight-before-direct-retry-work-2026-11-06).
No live account operation or server acceptance claim.


## Definite-null unknown optional text uses shared factory rules (2026-11-06)

**Confirmed selected static / implemented:** receipts now share the existing
call-ACK text factory/cache null predicate, including empty text/user and unknown
bare/domain outside recognized factory/cache names. Retry context construction
omits those optional participants instead of suppressing supported487 or durable
completion ACKs. Typed objects and recognized native bare objects remain distinct;
false remains unclassified. Root/recipient capability, ACK encoding preflight,
reservation and no-success-before-work rules remain. Count2 saved-session
query/ACK/refetch now exercises null bare/domain participant text.
[Complete retained evidence and corrected test expectations](receipt-optional-addresses.md#shared-definite-null-text-factorycache-boundary-2026-11-06).
No live operation, identity adoption or generic JID rewrite.


## Valid Java-reference numbers do not mask later retry errors (2026-11-06)

**Confirmed getter calls / implemented with explicit host-reference limits:**
retry count/t now share the existing signed BMP-decimal parser instead of Rust
ASCII admission. Valid numbers advance error classification to the next required
field; invalid non-ASCII still does not gain automatic487 authority. Count retains
signed32/nonnegative-model bounds and default1. Independent host Java178-case
int/long fixture drives admission/error-order tests; saved-session Arabic count1
and fullwidth time1 normalize to the same durable reservation as ASCII1 and the
existing outgoing enc count remains ASCII. Android runtime digit-table parity is
not claimed. ACK encoding/owner/identity/refetch/no-replay controls remain.
[Exact APK/API evidence, reference provenance and reproduction](retry-parser-admission.md#shared-java-reference-numeric-admission-and-later-error-ordering-2026-11-06).
No live account operation.


## Media lifetime wrap cannot become indefinite upload auth (2026-11-06)

**Confirmed pinned static / implemented:**1t5 separately wraps ttl/auth_ttl
seconds-to-ms products. Upload route selection previously saturated the smaller
raw-seconds value, allowing natively expired wrapped durations to remain usable.
It now converts both first, then applies the existing local positive-duration/
request-age guard. Tests use independent Java arithmetic vectors, each lifetime
separately, usable positive-wrap controls, and every-table/journal preservation
before/after reopen. No raw reply/provenance rewrite or automatic refresh/HTTP
operation. Full base-add/elapsed/reboot/native scheduling parity remains open.
[Exact DEX operations, reference results and remaining limits](media-network-outcomes.md#wrapped-lifetime-conversion-before-local-route-eligibility-2026-11-06).


## Media deadline consumers separate refresh from auth validity (2026-11-06)

**Confirmed selected static / research only:** exact1t5 field census finds A04
route deadline read only by0VM.A0T's120s refresh test; auth A02 has distinct
cache/timer/wait/upload/supplier consumers. A06 schedules auth expiry minus60s;
A0C requires changed auth versus an optional old token plus strict future expiry
with a caller margin. A0B waits in <=1s chunks only with request flight, using an
uptime budget while auth uses elapsedRealtime. Selected upload paths pass120s
or0 margin with20s wait input, and may still accept future auth after requesting
refresh. Feature/online/offline and post-offline-skip transfer checks remain
separate. This refutes a blanket120s hard auth cutoff and does not establish
route TTL as every upload's hard expiry. Rust's current minimum is local policy.
[Complete field census, nine methods, exact gates and required next work](apk/2.26.43.72-media-deadline-consumers/README.md).
No timer, automatic refresh, runtime behavior or live account operation changed.


## Upload preparation cannot bypass current route/account reservation checks (2026-11-06)

**Confirmed Rust race / implemented local fence:** host/auth selected before
media encryption is now revalidated against current route age/coordinator and
registered phone/generation inside the upload reservation transaction. Seed and
outcome_unknown row commit together; failed builder/expired or replaced route/
changed account cannot leave a seed or upload row. No silent URL/token switch,
new refresh or POST occurs on a failed guard. Owned active-pointer selection
shares identical read/write rules and retains corruption classification.
Tests cover all-table/journal preservation with/without seeds, reopen, successful
atomic commit, duplicate refusal and uncommitted owner/corruption visibility.
[Native versus local scope, preparation window and remaining response/transfer work](media-network-outcomes.md#atomic-upload-routeaccount-fence-at-reservation-2026-11-06).
No live account operation; this is not full native post-commit lifetime parity.


## Late upload observation is not account authority (2026-11-06)

**Confirmed Rust gap / implemented local owner binding:** new upload reservations
store version1 phone/generation metadata. Final payload promotion rechecks current
registered owner and exact reservation in one transaction, preserving owner fields.
Raw headers/body remain observations even after revocation; they cannot alone
produce uploaded status, payload reuse or resend authority. Prospective completed
cache rows check the owner before both preparation paths return bytes. Corrupt or
partially missing version/owner metadata remains a redacted storage error.
Tests cover owner loss/replacement, benign auth-time updates, stale rows, metadata
faults, all-table/journal/reopen preservation and real cache/config cross-phone
refusal. Historical unbound rows remain explicitly legacy, without backfill.
[Format, native-versus-local scope and remaining lifecycle/cancellation work](media-network-outcomes.md#upload-response-adoption-and-prospective-cache-owner-2026-11-06).
No live operation or automatic retry/reset.
