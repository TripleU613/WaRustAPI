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
