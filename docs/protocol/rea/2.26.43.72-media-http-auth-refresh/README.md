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
