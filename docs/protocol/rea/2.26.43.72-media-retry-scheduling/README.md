# Media retry owner, arithmetic and timer

**Confirmed static**, APK 2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
(rehashed during this pass). Checked DEX output only; no APK or account execution.

- `0Vc.<init>` constructs 0Vf with seed1 and cap987 (0x3db) and owns a Java
  Random. `0Vf.A02` resets attempt count/previous term to zero and next term
  to seed. `A01` increments attempts and advances the pair while the previous
  term <= cap; otherwise it returns cap without advancing the pair.
- **Derived after fresh/reset state:** base seconds are 0,1,1,2,3,5,...,987,987,... .
  The attempt counter continues increasing at the cap; no retry-count limit is
  established by this leaf. `A03` rewinds/resets or advances to a nonnegative
  requested attempt count; its callers/lifecycle are not traced here.
- `1xY.Bvj` qualified503 obtains A01, bypasses randomness for base0, otherwise
  sets B=base*1000 and delay=(3*B)/4 + abs(Random.nextLong() % (B/2)).
  For positive base s, the derived inclusive range is 750*s through 1250*s-1
  milliseconds. This is not a claim of a uniform distribution. The 987-second
  cap is the base cap, not the final jittered delay cap.
- `0VM.<init>` constructs 1VI on the main Looper with selector2 and this manager.
  `1VI.<init>` stores that exact selector/object; handleMessage's default branch
  for selector2 routes what0 to 0VM.A07. The existing request-path evidence
  proves A07 still checks throttle and in-flight state at dispatch.
- `0VM.A0A` (retained separately in media-route-lifetimes) resets 0Vf via A02
  before route installation/merge. It then invokes A06. With a current route, `A06` removes existing
  message0 entries and posts one with delay = current auth deadline A02 minus
  elapsedRealtime minus60000. This uses auth expiry, not min(routeTTL,authTTL).
  Negative-delay platform dispatch semantics and all other message producers
  are not established by this trace.

The error callback does not itself remove every prior timer. A future scheduler
must preserve shared Handler triggers and dispatch gates rather than treating503
jitter as a universal server-provided minimum wait. Session ownership, cancellation
on disconnect/account switch, shared DI lifetime and attempt limits remain open.
Rust automatic retries/proactive refresh remain disabled; this does not authorize
changing the existing explicit one-query CLI operation into a retry loop.

## Disassembler correction

The old checked callback left cb and be unnamed. `xtask apk-smali` now names
add-double/2addr and div-long/2addr with the existing 12x widths/register layout,
matching the [AOSP bytecode table](https://source.android.com/docs/core/runtime/dalvik-bytecode).
Two literal-byte regression tests check registers, widths, following instruction
boundaries and truncated prefixes. The regenerated error callback has no unnamed
opcodes and directly exposes the507 deadline addition and503 jitter divisions.
Old retained output is unchanged; its earlier limitation is closed by this file.

## Reproduction and retained bytes

Use `xtask apk-smali APK --class CLASS --signatures --exceptions`, selecting
LX/0Vc;, LX/0Vf;, LX/0VM; with method <init>/A06, LX/1VI; with method
<init>/handleMessage, and LX/1xY; with method Bvj as named by the files.
The model table is an analyst simulation of the retained recurrence, not a live
trace or independent native execution oracle. Provider limitations are separate.

- [auth-expiry-timer.txt](auth-expiry-timer.txt) — SHA-256 `1c968ca81518f79d33f479357b9eec5d31b564eb65e5e2d6c1952ae7dafe0d46`.
- [error-callback-arithmetic.txt](error-callback-arithmetic.txt) — SHA-256 `e30e07fc141c8373d0c4fa40c0bfc268ef0bd76ea3b92c45ff55577ddfab680e`.
- [fibonacci-state.txt](fibonacci-state.txt) — SHA-256 `1fe16d57e0cbacb6707834a4af954e76f64f78658cf9ef133e04ff3801352aeb`.
- [handler-constructor.txt](handler-constructor.txt) — SHA-256 `8b8630d5b4a4b0b36f9434a6d442f033c829505518d4261726b2922ae8eb6ffa`.
- [handler-dispatch.txt](handler-dispatch.txt) — SHA-256 `fb5761f7998e1858d8609731bc7c2e414ce0ad00cde087ba81a5db5cd146643e`.
- [manager-constructor.txt](manager-constructor.txt) — SHA-256 `78cff969873ffdcebe49e8a1815c013f2e3ea104e7bb04df86f932e143eed4e7`.
- [provider-limitations.txt](provider-limitations.txt) — SHA-256 `0ceb2f1cbdd54b8e871d57a70a058a81b58ce543d39ee6c9ffd39e566ae74c9e`.
- [retry-model-reference.txt](retry-model-reference.txt) — SHA-256 `a0a1647693d34b1e6e7c64c0497a00df094c9e21116ca5ef18de753f0bdb6e39`.
- [retry-owner.txt](retry-owner.txt) — SHA-256 `fb0f9bdac9e44857e1b4209ad3013a0cd13742928d09c999ddc4e06d8a9e6cf5`.
