# Account default disappearing timer: callback ownership and expiry

**Confirmed static**, WhatsApp2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX method exports, not runtime observations or REA Evidence envelopes.
Only the selected ordinary account-default IQ paths and callback branches are
interpreted. Per-chat E2E settings and usync contact data are separate.

## Native requests and timer

Exact-string xref `disappearing_mode` locates ChangeDMSettingActivity.A0c.
Its selected send branch builds to=s.whatsapp.net, type=set, xmlns=
disappearing_mode, with one disappearing_mode/duration child. It constructs
Cgi callback and passes20000ms (0x4e20), logical type277 (0x115), to0Sf.A0W
at018a..0198. Initial branches handle unchanged/sentinel selections and
manager gates; these are not authorization to fabricate their runtime values.
CJN.A00/A01 builds1cy conditionally using0wU; the selected write serializes
positive1cy.A00, otherwise1cy.A01, widened from signed int to long. Full gate
meaning, UI value domain and alternate-mode policy remain open. This chunk
does not add an invented duration whitelist or claim the Rust u32 input fully
reconstructs that native selection policy.

0oy.A00 builds the empty-child account-default get IQ, createsCgd selector1,
and passes20000ms, logical type296 (0x128), to0Sf.A0W at00b4..00c2.
Both paths therefore use20s, not a universal32s action timeout.
[Shared send admission and timer evidence](../2.26.43.72-abprops-callback-timer/README.md)
retains0Sf.A0W/A0A and0ln.A08: callback registration/timer starts before
writer dispatch, and promotion does not restart it. Expiry is a competing
callback claim; neither callback existence nor timeout establishes delivery.

## Timeout differs from successful setting adoption

Cgi.BsS postsCvG selector37 through0Js.CbR. InCvG.run 0bb2..0bf8 that
branch callsCgi.A00 with delivery_failure/false, invokes a UI helper, reads
the saved disappearing_mode_duration_int via0p0.A02, and publishes it through
6of.A1C ->07e.A0D. Cgi.A00 buildsBSL fields and submits to0NX.CSW;
the downstream telemetry sender and observer consequences remain untraced.
The selected timeout branch does not call0p0.A09 or resubmit an IQ.
It is not a server refusal or permission to undo uncertain remote changes.

In contrast, Cgi.CK4 postsCvG selector36. That branch callsCgi.A00 with
null/true, compares requested1cy with saved0p0.A06, and, if different, calls
0p0.A09 with the current08q clock sample. A09 writes duration/timestamp/
after-read preferences and notifies07f. Its branch consults0wU; complete
selection/gate and observer policy remains separate. Native error Bvj posts
CvL selector39, whose body is not traced here. The completeCgi export retains
this unresolved endpoint, not an assertion that all callback branches are done.
Cgd.BsS selector1 returns immediately for the read path; no retry is scheduled
in that selected method. Complete read-response application remains open.

## Rust correction and regressions

**Implemented offline:** normal LiveReplies assigns20s to an empty-child
account-default get or a set with one disappearing_mode child and text duration.
Existing queue_default_timer is covered; no new read CLI or scheduler is added.
Timer starts immediately before awaited local write and is installed with that
original start after success. Dedicated select expiry removes only live callback
ownership; durable Sent uncertainty remains, with no replay, setting adoption
or local rollback. Result/error consumes ownership and cancels its timer.
No independent absolute-age predicate is added to reply settlement.

Paused-clock test covers get/set,19/20s boundary,2s write delay, result/error
cancellation, late replies after expiry, and an independent AB-properties
callback that survives the20s expiry until its own32s deadline. The normal-driver
expiry test now also covers queue_default_timer with33s local-write delay,
late reply, subsequent receipt ACK, no resend and clean stop. Existing block/
unblock/AB-properties cases remain. Rust waits for local write completion
before dispatching expiry/receive; full Android concurrent scheduling parity,
native default-value adoption/observers, server policy and other producers'
deadlines remain unfinished. No live account operation or APK execution.

## Reproduction

`target/debug/xtask apk-xref APK --anchor 'disappearing_mode'`;
`apk-smali APK --class CLASS [--method METHOD] --signatures --exceptions`:
ChangeDMSettingActivity.A0c (set),0oy.A00 (get),Cgi full class (set-callback),
Cgd.BsS (get-failure),CJN.A00/A01 (duration-selection/mapping),CvG.run
(set-runnables),6of.A1C (timeout-notification),0p0.A02/A06/A09
(saved-duration/saved-default/save-default). Full methods retain unrelated
branches; only selectors36/37 ofCvG.run are interpreted. checksums.json covers
every other file in this bundle. No raw APK or bulk tree is committed.
