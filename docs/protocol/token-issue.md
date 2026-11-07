# Outbound trusted-contact token issue producers

2026-10-09. **Confirmed static**, APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [14-topic recipe](apk/recipes/2.26.37.71-token-issue.json) reproduces
[register/catch/full-descriptor evidence](apk/2.26.37.71-token-issue/targets.json).
The [inventory](token-issue-inventory.txt) preserves all physical string/caller
searches, the corrected selector query and the other success writers. This
extends the [refresh lifecycle](identity-token-refresh.md), without enabling
Rust token sending or changing private account state.

## Creating scheduled state

**Confirmed:** exact real_issue_timestamp references locate 0q2.A0W alongside
the previously traced A0V, row decoder and schema builder AUP.AR4. A0W is called
only from HxT.A01 and A04 in this APK's ordinary symbolic invocation inventory;
reflection/native reachability is not claimed.

A0W normalizes through A0L, opens a SQL transaction and reads the outbound row.
When sent_tc_token_timestamp is already at least the supplied timestamp it
commits without replacing it. Otherwise a missing row is initialized with the
new sent_tc_token_timestamp and explicit real_issue_timestamp=0. For an existing
row, the new sent timestamp is written while a nonnegative HYS.A01 is carried
forward as real_issue_timestamp. Null/negative HYS.A01 becomes zero. At
`009c`, the nonnegative branch jumps to `00fc` to move the old issue value into
v12, then jumps back to the common write at `00a0`; it is not a no-write branch.
The native UPDATE_SCHEDULED_TIMESTAMP operation writes JID, supplied sent time
and that carried issue value, then invalidates the canonical cache when its
outcome is not the unchanged sentinel. Its DEX catches are preserved.
AUP.AR4 declares the three columns JID/sent timestamp/real issue timestamp.
A scheduled sent_tc_token_timestamp is therefore not proof of server acceptance.

**Confirmed:** HxT.A04 normalizes its user and loads current HYS through A0O.
Missing state proceeds to scheduling. With existing state, positive property
996 (0x3e4) compares current and stored time buckets; an equal/newer stored bucket
returns. Nonpositive bucket length logs bad configuration and instead compares
against now_seconds minus 15,724,800 (182 days). Expired/older state calls A0W
with a fresh 08B.A04 timestamp divided by 1000, before pending-set admission
through A05 and GeneratePrivacyTokenJob queuing. This creates outbound scheduled
state through a real native producer; it does not require received token bytes.

The additional old HYS.A01 / positive property 4063 (0xfdf) branch applies a
batch-issue threshold before A05. It reads privacy_token_last_batch_time_sec,
current bucket and a converted IVf.A06 interval, computes a ceil-based offset,
then tests the old issue value against a bucket threshold. Active property values,
interval producer/conversion units and complete batch-worker lifecycle remain
**unknown/unported**. The complete branch is retained as source; no guessed
numeric threshold or batch cadence is enabled in Rust.

**Confirmed clock source:** 08B.A04 prefers nonzero A04 plus elapsedRealtime,
otherwise currentTimeMillis minus A02. It does not use the A03 preference of
08B.A00. 1xl.A06 is integer division by 1000; 8kT.A03 widens 00D.A0b's integer
property value. Clock-offset producers/parity remain unknown. Neither outgoing
accepted_at nor a receipt's server timestamp is automatically this native clock.

## Message, call and namespace triggers

**Confirmed:** 17J.BFC handles Android message code 19, logs
MessageReceivedByServerHandler/onMessageReceivedByServer, and ignores the
participant-present branch before scheduling DNf. At `0306..031a`, it creates
DNf with selector 19 and arguments retained message, message key and handler.
The full descriptor identifies the Object/Object/Object/int constructor. Its
field writes place handler in A00, key in A01 and message in A02. The other
constructor has Cp4/Function0/09m/int parameters, with the same VLLLI shorty;
its assignments must not be substituted for this call.

DNf.run's checked switch sends selector 19 to `0746`. It obtains HxT from 17J,
returns for a retained message of type 9NJ or Bn8, and returns for positive
178.A07. A missing retained message bypasses those message-specific checks.
Only a key remote JID accepted by UserJid conversion proceeds to A04 at
`07a0..07a6`. These are confirmed raw exclusions; the semantic type labels and
complete A07 producer are **unknown**, so they are not renamed to imagined
message categories or permissions. This is native server-acceptance follow-up,
not evidence that our Rust ACK handler currently schedules tokens.

The other A04 calls are in DMw.run's call-invite handling (`022e`, `041e`) and
DNh.run's outgoing-call handling (`1ce2`), with visible calling-infrastructure
objects/logs. These separate producers do not authorize adding calls to this
client. Their branches remain in the evidence bundle; complete call semantics
are outside this port.

**Confirmed:** HxT.A01 is an existing-state namespace transfer. It admits the
normalized second argument to the pending set, looks up the normalized first
argument's HYS and applies the A01 expiry threshold. Only an eligible stored
value is passed to A0W for the second user before job queuing. Missing/expired
source unmarks pending membership. The callers are 17a.A09's gated contact
notification branch and 9OK.A07's new-LID change task, not a message ACK method.
Their own/feature gates and source-target mapping remain attributable in source;
canonical reconciliation is not inferred from raw aliases.

The A0V success-writer scan additionally locates CnI.A03 and CoO.A02 as well as
the already traced IX5 callback. Their result/notification semantics have not
been traced in this chunk. They must not be confused with A0W scheduling or
claimed as fully covered token ingestion.

## Full signatures and fast release comparison

**Confirmed local tool improvement:** `xtask apk-smali --signatures` now adds
full declared descriptor comments and full ordinary invoke descriptors. A recipe
can opt in with include_method_signatures=true, preserving the flag through
remap and marking its enabled targets. Defaults remain false and retain old
shorty snapshot bytes. Shorty filters still collapse references and can select
multiple overloads; this flag improves evidence rendering, not selector semantics
or whole-program virtual/reflection/native reachability.

The existing checked method_symbol parser reads DEX proto_id_item/type_list
parameter and return descriptors. Invalid prototype/type-table references surface when this
mode is requested; default output retains its previous behavior. Synthetic
cross-DEX object-overload fixtures cover both ordinary and range invokes,
identical old shorties, distinct full descriptors and malformed parameter type
indices. Legacy rendering stays unchanged when the option is off.

Reproduce without account access:

```sh
cargo run --locked -p xtask -- apk-smali ~/Downloads/WhatsApp.apk \
  --class 'LX/DNf;' --method '<init>' --signatures --exceptions
cargo run --locked -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-token-issue.json \
  --output docs/protocol/apk/2.26.37.71-token-issue --check
```

For another release, re-anchor and export a separate full-signature recipe, then
use apk-research-diff. Full descriptors make overload changes visible in topic
files and their mapped Rust paths. This does not automatically port semantic
changes or establish cross-release client compatibility.

**Unknown/unported:** actual Rust outbound scheduling/storage/refresh, active
canonicalization and bucket properties, base job terminal recovery, batch issue
semantics and fresh live identity-change token behavior. The normal server-ACK
producer is now attributable; implementing its complete follow-up still requires
those protocol/state inputs. Private identities, sends, keys, IDs and waits
were not reset, copied or changed by this offline research/tool change.

**Confirmed verification:** 788 workspace/all-feature tests passed (658 API,
4 property, 9 crash, 51 CLI, 1 API-symbol and 65 xtask), plus strict workspace
all-target/all-feature Clippy, warning-free Rustdoc, default CLI tests, API
symbol checks, CLI-reference regeneration, formatting and build checks.
The new export/check reproduces 14 topics / 19 files / 395,127 bytes. Full
signature CLI inspection shows both DNf overloads. The previous 26-topic
identity-token-refresh bundle reproduces byte-for-byte under default settings.
Cached self-comparison reports all topics/artifacts unchanged in 0.022072 seconds;
this is not a different-release comparison or whole-APK coverage. Hygiene
checks 1,280 local links and 23 APK bundles totaling 6,988,623 bytes.
This chunk changes offline analysis tooling and evidence; no account operation
or live identity/token request was performed.

## Clock producer continuation (2026-10-09)

**Confirmed static:** [clock producers](token-clock-producers.md) closes the
A02/A04 physical writers and both A07 callers. Login uses parsed success@t plus
sampled whole-second local wall time; message selector 4 supplies Bundle time.
Selector-4 wire provenance and complete native/Rust clock parity remain unknown.
The earlier producer unknown is narrowed, not resolved for every clock input.

## Implemented: issuing our token after a direct send (2026-11-01)

`transport::token_issue` now follows the traced path: when `outgoing::acknowledge`
records a successful server ACK for a message to a direct PN/LID chat, it calls
`schedule_after_ack_in` in the same transaction. Unless that user's outbound
state was scheduled within the confirmed 182-day fallback window (`0q3.A00`), it
writes the scheduled `sent` time first (`0q2.A0W`) and queues the
`GeneratePrivacyTokenJob.A0G` IQ on the action queue:
`<iq to="s.whatsapp.net" xmlns="privacy" type="set"><tokens><token jid type="trusted_contact" t/></tokens></iq>`
(no token bytes). The next session transmits it; `actions --show` reports the
reply. A failed issue is not retried until the window expires.

**Still local limits:** bucket properties 996/997 and 865/909 are not applied
(their live values are unknown), so refresh may be rarer than the official
client's; `DNf`'s raw exclusions (`9NJ`/`Bn8`, positive `178.A07`) and the
call-invite producers are not reproduced; groups and broadcasts never schedule
(the APK's `17J.BFC` ignores the participant-present branch). The comparison
`lab-transport` path does not schedule.

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

## SDK and final legacy token defaults —2026-11-07

**Confirmed static:** generated09n.A01 SDK switch values and complete08Y integer
map construction independently resolve865/996=604800 and909/997=4. Declared Arl
implementations lead through defaults, override/silent-push/source dispatch and
bodyless value-reader boundaries. This closes selected-entry overwrite and
SDK-default questions without decoding binary parameter assets. Effective instance,
overrides and initialization still remain open; no Rust policy changes or live
operation. [Methods, derivations and limits](apk/2.26.43.72-token-sdk-defaults/README.md).

## Concrete SDK table reading —2026-11-07

The previous bodyless0A2 boundary is narrowed: Nv3 and0A3 provide DEX overrides.
09u context construction chooses default-only/direct/translated contexts;
Nv3 reads little-endian long and metadata vectors and reports distinct source
classifications. Retained methods and35 analyst-derived metadata cases separate
source gating, fallback masks and values. Manager/handle producers, translation,
readiness/cache and effective connection selection remain open. No Rust policy
change or live operation. [Evidence and reproduction](apk/2.26.43.72-token-sdk-value-reader/README.md).

## SDK handle and cache lifecycle follow-up —2026-11-07

Confirmed static handle producers now reach Java filename caches and native-holder
wrappers. File framing validates optional3c f1 headers/footer and restores raw
cursor0 on nonmagic input; it does not validate effective token values. Latest
context callbacks differ from full session/per-config/override refresh, which is
called fromA0C andBPZ. Twelve analyst Java vectors cover header/cursor behavior.
Manager selection, hydration/translation and00E effective invalidation remain open.
[Retained methods and limits](apk/2.26.43.72-token-config-refresh/README.md). No runtime or live change.

## Parameter-map header and schema gate —2026-11-07

The four packaged binary v4 headers are now decoded throughPfF.A00/A01 and
rechecked against prior member identities. Lightweight0EV installation requires
matching table/map schema prefixes and parsed translation state. Header metadata
is distinct from effective bucket values; body rows, complete manager selection
and loaded-table hydration remain open. No Rust policy or live operation changed.
[Retained methods, extraction and limits](apk/2.26.43.72-token-param-header/README.md).

## Parameter pairs and selected token translation —2026-11-07

ConfirmedPcn loads two little-endian ints per parameter. All four packaged bodies
validate size, config count, unique encoded keys and sorted search partitions.
A pinned DEX payload derivation binds compiled config keys to the token rows;
865/909/996/997 translate to slots101/110/125/126, matching their current original
slots for this packaged map. Dynamic tables, schema versions, silent push and
manager/cache selection remain separate. No runtime timing or live change.
[Methods, selected pairs and reproduction](apk/2.26.43.72-token-param-body/README.md).

## MobileConfig network producer follow-up —2026-11-07

Hmm fetch dispatch now connects selected SDK configuration work to MEX/WWW
branches, packaged query IDs and captured MEX result callback. fetch_result_json
is forwarded to a native handler after selected refresh-ID preference work;
that callback is not proof of native table commit or effective token policy.
Rust has no corresponding MobileConfig fetch/hydration adapter. Producers,
serializer, error/native persistence and effective selection remain open.
[Full selected route evidence](apk/2.26.43.72-mobileconfig-fetch-route/README.md).
No runtime request or live operation enabled.

## MobileConfig selected error handoff —2026-11-07

IE9 selector30 dispatchesID2 selector20, which forwards false and a selected
Hbw message to the SDK handler. Ranking prioritizes extensions.summary, critical
severity, exception family, then other errors, preserving earlier ties. Null and
empty messages remain distinct. The selected Java branch has no explicit retry
or refresh-ID write; native and generic transport policy remain untraced.
[Selected bodies and reproduction](apk/2.26.43.72-mobileconfig-fetch-error/README.md).
No Rust request, state or live operation changed.

## MobileConfig list/null serialization follow-up —2026-11-07

The nullqueries list setter attaches an emptyNhb before returning, while direct
query_hash/query_string null setters retain null scalars. ReusedMap/JSONObject
and Android16 platform source predict[]/null/null on the selected non-Pando JSON
route. Pando/platform runtime and full wire remain unknown; no request enabled.
[Methods and qualified synthetic shape](apk/2.26.43.72-mobileconfig-null-variables/README.md).
