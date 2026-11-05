# Block-list callback timing: read versus change

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX method exports, not REA Evidence envelopes or runtime observation.

## Current-build anchors and producers

`block_list_v2_dhash` references re-anchor older1ep to1dH. CompleteA0M/A05
exports retain their other branches for verification; only the ordinary
xmlns=blocklist paths are interpreted here. PSA w:comms:chat and w:interop
are distinct protocols and do not inherit this timer policy.

**Read:**1dH.A0M constructs6p9 callback and ordinary blocklist/get IQ, then
passes duration0, logical type198 (0xc6), to0Sf.A0Q at012c..014a.
A0Q forwards unchanged duration toA06 with false. A06 callsA0U; on its true
branch it registers0ln.A08 with the supplied duration before dispatch.
On false it removes a queued placeholder viaA0E and invokes the supplied
callback's BsS. There is no positive timeout substitution in these methods.
[Shared registration/timer evidence](../2.26.43.72-abprops-callback-timer/README.md)
shows A08 posts message4 only for strictly positive durations. Therefore this
selected fetch path installs no message4 callback-expiry timer; this does not
prove it has an infinite lifetime across disconnect or other lifecycle events.
6p9.BsS logs delivery failure and, when migration callback7sa exists, reports
an exception viaFfK.BsR. Its downstream migration/future policy is untraced.

**Change:**1dH.A05 constructs8R7 bound through7vW to7vV, selects block/unblock,
constructs ordinary blocklist/set IQ, and passes20000ms (0x4e20) to0Sf.A0W
at0638..0650. Both block and unblock join this send site. A0W/A0A registers
before writer dispatch; the existing shared evidence shows promotion does not
restart its timer. This is20s, not the32s AB-properties duration.

## Failure callback scope

8R7.BsS logs general_request_timeout and submits7vV.A0E through0Js.CbR.
The owner constructor assigns8bc selector2 to that field. CbR runs immediately
if0LG.A04 returns true, otherwise callsCbP; its full scheduler is not inferred.
8bc.run selector2 (0a06..0a36) conditionally invokes a UI helper and calls
2KT.CE5(false). Concrete CE5 implementations and downstream jobs/state remain
unknown. Thus callback expiry does not prove non-delivery or server refusal,
and this trace does not authorize rolling back block state or automatic retry.

## Rust correction and verification

**Implemented offline:** LiveReplies now assigns20s only to a normal
blocklist/set whose first item action is block or unblock. It keeps the
pre-write start and dedicated expiry-event semantics already used for initial
AB-properties. A blocklist/get has no fabricated20/32s default. Expiry releases
only live callback/timer authority, retains durable Sent uncertainty, and
continues receive. No list/hash change, rollback, retry or new request follows.
Result/error cancels the timer before durable settlement; an overdue timer not
yet dispatched still competes for ownership, with no added absolute-age test.

Paused-clock tests cover block/unblock,19/20s boundary,3s write delay, result/
error cancellation, late replies after expiry, and fetch still claiming a
reply at60s. The production-loop test covers initial AB-properties and both
block changes with a33s local-write delay, expiry before late result, unchanged
local block projection, subsequent receipt ACK, no resend and clean stop.
Other producer deadlines and full native failure side effects remain open.
Rust processes expiry after awaited local writes; this does not reproduce all
native concurrent scheduling. No live account traffic or APK execution.

## Reproduction and retained boundaries

`target/debug/xtask apk-xref APK --anchor 'block_list_v2_dhash'`;
`apk-smali APK --class CLASS --method METHOD --signatures --exceptions`:
1dH.A0M (query),1dH.A05 (set),0Sf.A0Q (query-wrapper),0Sf.A06
(query-admission),6p9.BsS (query-failure),8R7.BsS (set-failure),7vV.<init>
(set-owner-constructor),8bc.run (set-failure-runnable),0Js.CbR (failure-post).
Shared0Sf.A0W/A0A and0ln.A08/A0E are retained in the linked timer bundle.
The selected full methods include unrelated branches; no claim of full
block-list semantics or complete callback implementation coverage is made.
Every other file here is covered by checksums.json. APK/bulk exports stay local.
