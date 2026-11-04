# Conditional media HTTP auth refresh

**Confirmed static**, APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX, not execution. Existing REA Java-provider limitation remains;
these are checked text exports, not new REA Evidence envelopes.

## Three direct A0S callers

Exact invocation scan finds Mtv.BkV, Mtv.Bvd and MVv.A04. This bounds direct
DEX calls only, not reflection or all transfer loops. The already-traced
0VM.A0S forwards only401/403 to the existing gated route query A07.

- **Mtv.BkV** accepts only401/403, requires A0N=true, nonnull stored token
  A03 and nonnull route A01, then feature predicate08Y.A17(0tF.A0L)=true.
  If LtU.A00>=2 it returns false. If the stored token differs from the
  supplied token argument, it increments that counter and returns true
  without calling A0S or waiting. Otherwise it invokes A0S first.
- It samples uptimeMillis, initializes negative LtU.A01 from manager A0K,
  and only with positive budget waits via A0B if A0C is not already true.
  It rereads current route and rechecks A0C before retaining the candidate.
  Mtv.A04 subtracts positive elapsed uptime from the budget, floored at0;
  LtU constructor initializes budget=-1 (counter has Java zero default).
- **0VM.A0C** requires a nonnull route, auth token different from the supplied
  old token, and auth deadline strictly greater than elapsedRealtime+margin.
  BkV supplies margin0. No connection-deadline comparison occurs in A0C.
- If no qualifying changed route remains, BkV marks A07=true and returns
  false. Otherwise it increments the retry counter, replaces stored auth A03,
  rebuilds route A01 via Mtv.A00 and returns true. True is eligibility from
  this callback, not proof of an actual resend or successful transfer.
- **Mtv.Bvd** snapshots then clears A07, invokes A0S only when status!=-1
  and the snapshot was false, increments supplier A00 and invokes A05.
  This suppresses duplicate refresh reporting after the failed auth-change path.
- **MVv.A04** catches LIV HTTP-response failures: at0e78 reads responseCode,
  retains it in MOv.A0X, invokes A0S, constructs MXt with downloadStatus,
  and preserves responseCode in MXt.A02. Its exception table retains the
  original code-unit handler locations. This is download error handling,
  not evidence of automatic same-request replay.

## Still unknown / implementation boundary

Manager constructor initializes A0K=20000ms; the exact direct-write scan finds
only that constructor write. [Previously retained constructor](../2.26.43.72-media-retry-scheduling/manager-constructor.txt).
This bounds direct DEX writes, not reflective changes.

Mtv field producers, actual feature-flag values,
callback caller loops and their request replay/idempotency rules are untraced.
A0B wake/cancellation semantics are not established by these leaf exports.
Rust still persists refusals and does not automatically resend transfers.
Do not replace this conditional lifecycle with generic401/403 retries.

## Reproduce and verify

Use apk-callers for LX/0VM; A0S and apk-smali with --signatures --exceptions
for LX/Mtv; BkV/Bvd/A04, LX/LtU; (whole class), LX/0VM; A0C and
LX/MVv; A04. Each file below is the complete selected output.

- [refresh-callers.txt](refresh-callers.txt) — SHA-256 `5f99c5cc36671dc799b94f11d80475568e68d764b868af3038b27222880e9860`.
- [auth-failure.txt](auth-failure.txt) — SHA-256 `49222de67d993367389912641c00bd145c2d9c27c9cb3467d497b83e6e41db10`.
- [response-report.txt](response-report.txt) — SHA-256 `e77449ebeb45ca9b118f48bea83d8317538e7e971fe5de2c6828768ae045f4eb`.
- [wait-budget.txt](wait-budget.txt) — SHA-256 `d972dfc8327f0a417607a54d1a1af449611ac2a99f5b54461b3e22f5dc894d3a`.
- [retry-context.txt](retry-context.txt) — SHA-256 `a3ef4d1f27bdf3f980a5f6a6a7b685c753aa0ad876b23b57b788764027c831b3`.
- [changed-auth-predicate.txt](changed-auth-predicate.txt) — SHA-256 `c142f152e9b67d97c9c9341e3af73f16525c66da8672270889112c9d44dd6f8d`.
- [download-error.txt](download-error.txt) — SHA-256 `5669991138eb71c0b016c6861f129f0944b8b874c3216d89b2200674fc4525e2`.

- [budget-writers.txt](budget-writers.txt) — SHA-256 `6a105ddbc1020ca79f3c5cb350184ac2433b6b90861a757b01feb7c1c68d4b4b`.

## Transfer-loop continuation (2026-11-04)

**Confirmed static**, same pinned APK. Concrete-call scan finds LIY.BkV
forwarding to nonnull Mtv and returning true only for its true result. Exact
Nd8.BkV interface-call scan finds MS4.A03. A zero concrete LIY.BkV call
count does not establish unreachability: the caller uses the interface.

In MS4.A03, initial/current routes come from Nd9.Aex. Function0 is checked
before the transfer callback, and optional Nd8.CW7 may change route readiness.
At0086 the supplied NYs.Cb6(route) produces Ly8. With A05 false, result
code1 plus replacement supplier updates A00 and goes to the delay path.
Otherwise an Nd8 supplier receives BkV(shared LtU, attempted route.A01,
result.A00). True sets a retry marker and goes to that same delay path.
False continues through Bvd(result.A04,result.A00); result.A03 then controls
return versus continuation. Do not infer the meaning of every Ly8 field
from the short obfuscated name.

The delay path at0104 calls current supplier.AXk, Thread.sleep, reloads
supplier.Aex and loops to route-presence/cancellation/preparation checks before
calling NYs.Cb6 again. The next callback after a true retry increments the
MS4 AtomicInteger. InterruptedException during the sleep goes to the method's
existing terminal handling (DBd.A1N and last payload return), not an immediate
callback replay. AXk delay policy, CW7 behavior, DBd.A1N details and concrete
NYs operation/idempotency bindings remain untraced. This confirms an actual
loop edge, not that every upload/download may safely use that edge.

Mtv constructor takes auth-enable A0N from its final boolean argument; exact
A0N write scan finds only this constructor. That closes the field assignment,
not the higher-level caller's decision to enable it. Auth token A03 is the
constructor's ninth reference argument (v18); its later update was traced in
BkV. Constructor caller argument provenance remains open.

Reproduce: apk-callers exact Mtv.BkV and Nd8.BkV; apk-smali LIY.BkV,
MS4.A03 and Mtv.<init>; apk-field-refs exact Mtv.A0N:Z --writes-only.

- [auth-wrapper.txt](auth-wrapper.txt) — SHA-256 `811c64cfa599bc290d55e121ab36a574a4c65c5ea22a845fe4222e083bf9ba28`.
- [auth-concrete-callers.txt](auth-concrete-callers.txt) — SHA-256 `eb3027ec086ff570bc4041be67592abff4e33ddba99aef8b1043af953369b2ea`.
- [auth-interface-callers.txt](auth-interface-callers.txt) — SHA-256 `4eb996a11b3f22a87dcfbcdd375096d70340bad4ed65db23fb892ca360661d86`.
- [supplier-constructor.txt](supplier-constructor.txt) — SHA-256 `d20b0a26331fb4915d3b1125290720e80e7a5afbc2f52255cb86a64938ad7eb2`.
- [auth-enable-writers.txt](auth-enable-writers.txt) — SHA-256 `4ce2bc6a5b997bc129f3b56391c86818eb13aaf8243b1bb9f34127c0bbae4ef2`.
- [transfer-loop.txt](transfer-loop.txt) — SHA-256 `3dd7f041229181b23c79068e193abd8c22d7b9ea8576196b28d6f45d382beea2`.

## Readiness and distinct delay suppliers (2026-11-04)

**Confirmed static**, same pinned APK, no runtime execution.

- LIY.CW7 forwards to Mtv only when its backing supplier exists; otherwise
  false. Mtv.CW7 requires auth-enable, stored token, current supplier route
  and the same feature predicate as BkV. It initializes the shared negative
  wait budget from manager A0K, samples uptime and checks manager auth.
- If current manager auth is valid for strictly more than120000ms, it may
  adopt it immediately when different from the stored token; a matching
  token yields no replacement. If absent/almost expired it invokes gated
  A07, then waits via A0B only with positive budget and missing/already
  expired manager auth. Still-valid almost-expired auth triggers refresh
  without waiting here. Afterward replacement requires different auth and
  deadline strictly greater than elapsedRealtime, not the two-minute margin.
  A04 accounts elapsed uptime; replacement updates auth and rebuilds route.
  This readiness path does not increment LtU's auth-failure retry count.
- Mtv.AXk returns configured A0D unless an exponential object exists and
  supplier count A00>=A08; then it obtains 1C8.A01, with null falling back
  to A0D. Constructor creation requires positive configured A0C and the
  constructor's enabling boolean; it supplies max draws=A0C, cap3000ms,
  base1000ms. 1C8 increments draws, computes base*2^min(draw,50), halves
  it and adds abs(nextLong()%scaledBase), then clamps at cap and latches
  capped mode. Exhausted draws return null. A02 resets draw count only,
  not the cap latch. No uniform-distribution claim is made.
- **LIY does not override AXk.** Its superclass Mtx.AXk returns A02, which
  Mtx constructor receives as a long argument. Both LIY constructors pass
  that value unchanged. Thus LIY auth forwarding does not imply that MS4's
  sleep calls Mtv's exponential-delay method.
- MvM.A00, on its A04-true branch with a constructed nonnull Mtv route,
  wraps either the backing supplier (feature true) or route only (false)
  in LIY. Both receive manager0VM.A02 as fixed delay, then MS4 with fresh
  LtU. With no supplier route it uses Mtv directly; the other A04 branch
  calls manager.A0M. A04 predicate, manager A02 producer and all alternative
  construction paths remain untraced.

Mtv.A05 full output is also retained because it was inspected: route selection
uses counts, configuration and input boolean; its complete branch semantics
are not claimed reconstructed here. Operation-specific NYs callbacks,
request idempotency and server acceptance remain open. No Rust retry enabled.

Reproduce with apk-smali class/method pairs named above, plus whole LX/1C8;.

- [readiness-wrapper.txt](readiness-wrapper.txt) — SHA-256 `4a113b3ddf335e9b85bbc139c06733f545dceeb29167c6b823ec69ca3593db99`.
- [readiness.txt](readiness.txt) — SHA-256 `c11f9b5fc9fbd449cc3c9473aafcf6d2e20b9ce4d85b2e27a282b51ee9296357`.
- [supplier-delay.txt](supplier-delay.txt) — SHA-256 `7c07c5d887bea6b20c6a0555e934923bdc8581d691164ec662507dbd85243c79`.
- [wrapper-delay.txt](wrapper-delay.txt) — SHA-256 `8d41b1fdfc6cef195af819dc2469d1d2eb8e0e9ab2482e0358cde0cd7659b764`.
- [wrapper-constructor.txt](wrapper-constructor.txt) — SHA-256 `db4be57cbc8a30d23b89f1082762367de4fa3293d638d0235e754c9b32d42511`.
- [auth-wrapper-constructors.txt](auth-wrapper-constructors.txt) — SHA-256 `b6630fa02a56200b724fe5b795babe360dcafe118cca18f64e3467dbfb52b53f`.
- [wrapper-selection.txt](wrapper-selection.txt) — SHA-256 `bca25c511dc89cf1169c728502fa261381eb915402a6bea5bc2872c229fe6e36`.
- [supplier-route-selection.txt](supplier-route-selection.txt) — SHA-256 `be7f4716f77fb5e7c93436c836aadc5265e210f9fd696261696f2c18960c82e1`.
- [exponential-delay.txt](exponential-delay.txt) — SHA-256 `0594ff0b79fd5c3ee4ec8ac2c1d40b68e6ed750b96f38e1c10e385c869b2bfe0`.
