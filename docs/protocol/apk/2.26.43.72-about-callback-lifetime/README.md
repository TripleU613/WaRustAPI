# About callback ownership: legacy map, UI wait and reply dispatch

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX exports, not runtime observations or REA Evidence envelopes. Full
selected methods/classes retain unrelated branches; only the paths below are
interpreted. No APK or bulk unpacked tree is committed.

## Producer and writer

Current SetAboutInfo.A5f callsFxR.A04; the older A5H name has no matching body
in this class/build. FxR replaces null/empty text with one space, buildsGkn,
and sends Message arg1=29 via0Sf.A0V/A09. The selected ready path dispatches;
not-ready returns false and calls the separate0Sf.A00 branch. That private
method's full queue policy is not reconstructed here.

[Existing full writer export](../2.26.43.72-iq-promotion-shutdown/writer.txt)
retains0v0.A00 case29 at0966..0a3e. It increments1rj.A00, creates a hexadecimal
ID, insertsJAj into1rj.A03, then builds status/set/to=s.whatsapp.net IQ with
one status text child and calls1r0.Cyt. This does not call0ln.A08 or register
its message4 timer. Rust uses its existing durable action ID; the different
native ID generator does not establish a server requirement to change it.

## Shared legacy map, separate UI timeout

0ZJ.A0z creates ConcurrentHashMap v13 at0916, passes it into A0S, then into
1rh's last Map parameter and1rj's last Map parameter. Constructors retain it
as1rh.A00 and1rj.A03. A0S constructs1r7 with that map; the selected1rC.AL1
factory reads1r7.A03 and builds1rD, which stores it asA00. Inclusion of1rC in
the injectedA1H set is not independently resolved here. 1rA.A02 selects stanza
kind iq for1rD. Reader dispatch uses its stanza handler map and calls1rA.A03.
This traces the shared map identity and selected factory path, rather than
assuming it is the distinct queued/sent0ln registries or observing execution.

SetAboutInfo.A5f posts its own message0 after32000ms. Its constructor binds
K57 selector1 to the UI handler. K57's non-message1 branch invokes an error UI
helper and dismisses the dialog; it does not remove this legacy map entry in
the selected body. Message1 instead callsFxR.A03 with the accepted text before
dismissing the dialog. Thus UI wait expiry is not proof of network callback
expiry. Complete downstream UI/helper behavior is not inferred.

The selected reader throwable cleanup block1s2.run 1128..1174 iterates1rh.A00,
calls1ru.A00 for its entries, clears the map, then rethrows. JAj.A00 posts UI
failure message0. This supports connection-owned lifetime on that cleanup
path, not a claim that every shutdown or concurrent interleaving is equivalent.
Other aliases may participate in additional lifecycle policy; no infinite
native callback lifetime or observed late-result race is claimed.

## Outer result dispatch ignores the About response payload

In1rA.A03's1rD branch, outer type=result with ID removes the legacy callback
and invokes1ru.A02(node) when found. JAj.A02 never reads the response node:
it marks the UI success flag, cancels UI message0 and posts message1 with the
original requested text. It then returns. The general1lZ.A1E call after the
callback is a separate path whose full effects remain untraced.

Outer error also removes the callback, then, for ordinaryJAj, iterates error
children and invokes1ru.A01 for integer codes; JAj forwards those codes to8ow.
Nested error elements under outer result do not select this branch. Rust's
generic nested-refusal parser previously changed this typed About result into
ActionStatus::Error, contrary to that producer's dispatch semantics. Synthetic
extension fixtures here do not claim the server has sent this response shape.

## Rust correction and verification

**Implemented offline:** typed about-set with saved status/set IQ and one
status child settles outer result asOk without interpreting nested error
extensions as refusal. Outer error still recordsError/code; ordinary app-state,
directory and other nested-refusal handling remains intact. Matching saved-wire
identity, connection ownership and once-only claims remain required. No status
reset, automatic retry, saved About adoption or live request is added.

A paused-clock test queues actual empty/nonempty About requests alongside an
AB-properties query. At32s only AB-properties expires; About can still claim
result/error at60s on the same connection. Result with synthetic error403
extension remainsOk with no refusal code; outer error retains403. Duplicate
claims fail. A normal-driver test sends that synthetic result and a receipt
barrier, verifiesOk/no code, continued receive and no resend. Existing nested
error tests cover the other producers. These are offline Rust observations,
not Android execution or proof of all native callback lifecycles.

## Reproduction and bounded searches

Use `target/debug/xtask apk-smali APK --class CLASS [--method METHOD]
--signatures --exceptions`. Exports retain FxR.A04, JAj class,1rj class,
SetAboutInfo class,K57.handleMessage,0Sf.A0V/A09,0ZJ.A0z/A0S,1rh.<init>,
1s2.run,1r7 class,1rC.AL1,1rD class and1rA class. Only1rA.A02/A03's1rD
branches and the identified reader cleanup are interpreted.
`apk-callers` selects1rj.<init> and1s2.<init>. `apk-field-refs` with exact
Map type selects1rj.A03,1rh.A00 and1r7.A03 to trace aliases. The JoW map
seen elsewhere in1s2.run belongs to a different MEX dispatch path and was
not treated as this legacy callback map. Empty exact string searches for
SetAboutInfo/setMyAbout/setMyStatus were not class-absence proof; the
my_current_status and writer status-update anchors plus UI class export
established the path. checksums.json covers every other file here.
