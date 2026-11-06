# Prekey fetch error backoff: scoped state and delayed drain

**Confirmed static**, WhatsApp 2.26.43.72 code 264307238, pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target execution, live request or REA Evidence envelope claim. New current-build
anchors identify 14o.BLn and 14t rather than reusing older obfuscated callbacks.
Current DF6 is an AnimatorListenerAdapter; no DFC method body was found. Those
names from 37.71 are not current key-directory evidence. The earlier shape-based
refusal attribution in Rust remains explicitly older-build evidence.

## Error ingress and affected request state

14o.BLn's dispatch selector 76 -> 074c obtains an EoY[] request-key array,
Bundle errorCode and isPqSessionUpgrade (default false). PQ-upgrade errors log
and return, with optional upgrade metrics. The non-upgrade branch reaches 07ec,
then the manager obtained from 14q.A09. Network callback-to-Message construction
and error-code grammar are not closed by this listener trace.

Under manager monitor, every affected request key is removed from in-flight map
A0B. For error code >=500 and <600 only, an existing EX8 entry is moved into
pending map A0C. Non-5xx or absent entries do not gain a synthetic pending retry.
For 5xx, a delayed retry is posted when either backoff-mode A01 was false, or A01
was true, no retry is posted (A00 false), and in-flight map A0B is now empty.
A01 true plus A00 true suppresses a second post; other live in-flight work can
also defer posting. Posting sets A01 and A00 true, computes delay, creates
Fea(manager,19), and calls Handler.postDelayed. Return value is not checked here.
Then A01(manager,affectedSet) notifies listener snapshots via FtW.C9b; this is
not itself a key write or success callback.

No server `backoff` attribute or wall-clock deadline is read by this selected
manager branch. It is local scoped retry state, not proof of a mandatory global
server cooldown. Error 406 has a separate downstream Boolean in the listener;
that consumer's full effects remain untraced.

## Delay generator has an initial zero

The constructor creates separate HashMaps A0B/A0C and 0Vf(base=10,threshold=610).
0Vf reset sets count=0, A00=0, A01=base. Its A01 increments count, then if old
A00<=threshold shifts (A00,A01) to (old A01, old A00+old A01), returning old A00.
When old A00>threshold it returns threshold without another shift. The listener
multiplies seconds by 1000 before posting. Derived source sequence:

`0, 10, 10, 20, 30, 50, 80, 130, 210, 340, 550, 610, 610, ...` seconds.

`derived-delays.json` records those state calculations. This is derived static
arithmetic, not an Android timing observation or executed APK oracle. Zero post
delay does not prove immediate wire send; the Handler, drain and job queue remain.
Do not replace this with an invented minimum ten-second server wait.

## Delayed drain and explicit reset differ

Fea selector 19 -> 0f38 calls manager.A04, not A03. A04 under monitor returns immediately when the pending map is empty, without
clearing A00 on that empty branch. On its nonempty path it groups
pending entries, reconstructs in-flight EX8 entries using uptimeMillis, calls
A05 with reason 9 for grouped work, clears pending map and clears posted flag
A00. It does not clear backoff mode A01 or reset 0Vf in this selected method.
A05 is a separate job admission path; a queued job is not a successful network
write or permission to repeat arbitrary uncertain Rust reservations.

Explicit A03 acts only while A01 true: clears A01/A00, resets 0Vf and invokes
A04. Exact caller census includes A00, two FeR.run sites, SendE2EMessageJob.A0J
and SendE2eStatusJob.A0J. Complete predicates and effects of those external resets
remain untraced. A03 does not visibly remove an already posted Fea from Handler;
no cancellation/join is inferred from Boolean reset alone.

A00's complete source demonstrates why a blanket guard is unsupported: it skips
PQ-upgrade batches while A01 true, while normal admission still consults and
updates the in-flight map, invokes A05 and can explicitly call A03 at its tail.
Thus persistent A01 is not proven to stop every newly requested legacy key query.
Complete normalization, deduplication, job dispatch and external caller policy
remain separate from the selected error-state transitions.

## Rust comparison and remaining work

Current peer_keys inspection binds refusals to one selected peer; device key
batches retain refused/invalid/uncertain state and block later batches. These
are local no-replay guards, not this manager's pending-map/delay/reset lifecycle.
No Rust retry/admission/identity/key/persistence behavior changes here. No generic
cooldown or automatic same-ID replay is added from partial source. Native ingress,
5xx attribution to original devices, EX8 lifetime, outer job scheduling, all reset
authority and durable recovery must be bound before implementing parity. Runtime
server acceptance or any causal ban-prevention effect is unverified.

## Reproduce

xtask apk-xref APK --anchor uses the exact three PreKeysRequestManager strings
in anchor-xrefs.txt. Use apk-smali APK --class CLASS [--method METHOD]
--signatures --exceptions for 14o.BLn, 14t.<init>/A00/A01/A03/A04, full 0Vf,
and Fea.run. Excerpts retain full export SHA256 and exact byte-PC ranges; manager
method extracts name the full-class hash. Census: apk-callers class 14t/method A03;
apk-field-refs class 14t/field A01/type Z. Checksums cover every other file;
raw APK and bulk unrelated switch/source exports stay outside Git.
