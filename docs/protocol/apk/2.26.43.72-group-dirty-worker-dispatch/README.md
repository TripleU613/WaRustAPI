# Current groups_dirty worker-to-request admission

**Confirmed static**, pinned WhatsApp2.26.43.72/code264307238, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `c5df433b`. Offline xtask DEX fallback; no REA-provider or runtime
observation. [Known provider failure](../2.26.43.72-retry-key-envelope-bytes/rea-provider-attempt.json).
No new Rust runtime policy, queue processing, account operation or live request.

## Raw collection becomes a typed worker input later

The [bounded notification excerpt](notification-scheduling-excerpt.txt) retains
0w9.A09's raw group-string collection and its5780..57aa scheduling block.
It supplies7e9.A05, null max-rounds, the collected list, a fresh HashSet and
params3 to FetchTruncatedGroupsJob, then queues via0fW.A01. The round argument
is retained as v5; this excerpt alone does not independently derive that value.
[7e9 initialization](context-enum.txt) binds A05 to ordinal1,
PER_GROUP_DIRTY_RECOVERY_TRUNCATABLE. The complete notification method remains
outside Git; reproduction validates its length/hash before deterministic
selection. Other instructions/exception tables are explicitly omitted.

The [complete job](job.txt) constructor stores the supplied list/context/round/
params. Its A0L returns on null context, computes a participant budget using
free memory and configuration6267 (0x187b), groups supplied pairs under that
budget, and processes the selected first list. Effective memory/configuration
values and complete round continuation are not established by this summary.

At0174..01aa A0L maps **every selected pair's first string** through
[1bd.A01](group-parser.txt) before either network dispatch branch. That parser
calls0Dd.A00, requires the concrete1bc type and otherwise throws09P. Reuse the
[complete g.us constructor/factory evidence](../2.26.43.72-group-dirty-constructor-boundary/README.md).
Thus the preceding raw-string scheduling boundary is followed by a typed group
construction gate before the selected worker's request. A parsing failure in
this loop does not reach either later dispatch call in that A0L invocation;
A0L has no local exception table. This does not imply a universal ACK, queue
cleanup or outer-runner retry outcome.

## MEX and stanza dispatch differ

For this ordinal1 context, MEX selection at01b4..01e4 checks configuration
0x344e then0x2a35. Effective values are unknown. The selected branch calls
[7uj.A00](mex-dispatch.txt) with typed group list, round/params wrapper7xh and
FfK. For each group it writes group_jid using
[2LD.A0e](jid-wire-text.txt), which returns getRawString, and
query_context=INTERACTIVE. Top-level groups/request_context are supplied to
QueryBatchGetGroups; ordinal1 maps PER_GROUP_DIRTY_RECOVERY_TRUNCATABLE.
The round/params wrapper is passed into the callback capture, not written as
additional JSON fields in this selected method. Full operation/document ID,
MEX envelope/deadline and callback validation remain separate.

The other branch chooses per_group_dirty_recovery_truncatable for ordinal1,
constructs [8UL](stanza-operation.txt) with ID/context/typed group list,
obtains its lazy BVu serializer and sends via0Sf.A0W with32000ms. Its FfK.get
is untimed at the Java callsite; that does not erase the separate request timer.
8UL's constructor retains the three inputs and installs8cm selector10 for its
lazy value. The concrete lazy serializer body, callback lifetime and full wire
field inventory are not newly traced here. The shown plain branch and MEX
branch must not be treated as one identical envelope/timing policy.

A0N returns true only for exception7gm or an immediate cause7gm; other supplied
exceptions return false. This is this job's exception predicate, not the full
shared job runner, a generic rate-limit bypass or authority for Rust replay.
Result handling can enqueue later jobs and invoke group-state work; complete
persistence/restart/continuation is not claimed by the admission trace.

## Consequence for Rust and remaining work

The previous raw-string consumer unknown is now closed for the selected worker's
pre-request gate. Rust's generic/local group admission still supports broader
legacy fixture forms and is not relabeled native-complete. A prospective
request-boundary correction needs to account explicitly for existing saved work,
legacy adapters, exact selected request builder and supported fixture migration;
this evidence alone does not authorize rewriting or replaying historical rows.
No blind global parser change, new MEX request or automatic retry is introduced.
The broader network audit remains active.

Reproduce six complete exports plus one bounded excerpt:

```sh
python3 docs/protocol/apk/2.26.43.72-group-dirty-worker-dispatch/reproduce.py ../android-lab/apks/WhatsApp.1.apk
```

APK and complete source hashes are checked before comparison; checksums.json
pins every retained file. No raw APK or complete bulk notification tree is added.

Validation: six complete exports and the bounded scheduling excerpt reproduced
byte-for-byte, with complete source hash/length verification. APK coverage,
formatting, CLI-reference regeneration, OpenAPI library-symbol test, whitespace
and staged hygiene passed. Full runtime tests were not rerun for this
research-only chunk; no Rust runtime source changed.

Followup: the [concrete stanza serializer](../2.26.43.72-group-dirty-stanza-serializer/README.md)
now closes8cm selector10/Dyu11/BVu4 and selected IQ/query/group fields. Native
assertion, callback, effective flag and persistence limits remain separate.
