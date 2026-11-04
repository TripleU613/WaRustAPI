# Media transfer result flags and entry points

**Confirmed static**, APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX exports, not APK execution or new REA Evidence envelopes.

## Result constructor and factory matrix

Ly8 constructor parameters are supplier, payload, code, A05, A04, A03.
MS4.A03 checks A05 immediately after callback: true loops to its route/
cancellation/readiness check without BkV/Bvd. Otherwise code1+nonnull
replacement supplier switches supplier and enters delay. Otherwise eligible
BkV=true enters delay before Bvd or the A03 test. If neither shortcut applies,
it invokes Bvd(A04,code); A03=true returns payload, false enters delay.
Thus A03 is not a universal terminal decision ahead of auth recovery.
[Full loop retained previously](../2.26.43.72-media-http-auth-refresh/transfer-loop.txt).

The following is **derived from literal factory argument assignments**, not
a report of server acceptance. Boolean input b/c is caller supplied.

| MXe factory | supplier | payload | code | A05 | A04 | A03 |
| --- | --- | --- | --- | --- | --- | --- |
| A00(supplier,b) | input | null | 1 | false | b | true |
| A01(payload) | null | input | -1 | false | false | true |
| A02(payload) | null | input | -1 | false | false | false |
| A03(payload) | null | input | -1 | true | true | false |
| A04(payload,b,c) | null | input | -1 | false | b | c |

A05(string)/A06(string) log and construct MXt(status8), then call A01/A02
respectively. Those identical payload statuses therefore do not establish
identical loop handling. Direct constructor scan finds14 call sites including
these factories and KMy, MtX, MtY, MtZ, N41; not every producer is traced.

## Cancellation entry points

2LD.A1V simply invokes Function0, casts Boolean and unboxes. MS4.A02 creates
N5N(selector5), whose invoke switch reaches0320 and 2L8.A1A, which returns
Boolean(false). Therefore this convenience entry has no caller cancellation
through that predicate; it still has route/result/interrupt exits.
Exact MS4.A03 direct-call scan also finds LHd.A0c and MvO.ARB. The latter
passes MtZ as NYs and MCO.A0A as Function0 at0490; its concrete predicate
producer and LHd conditions remain untraced. Do not apply selector5's false
behavior to those callers.

## Upload producer scope and HTTP predicate

MtZ.Cb6 is retained complete. It calls MXe.A07 with Lyh.A00 on one tail
branch and passes the returned boolean as Ly8.A03; other internal result
statuses choose distinct factories/replacement-supplier branches. Full
request construction, resume byte accounting and every exception path are
not reconstructed by this pass.

**Derived exact A07 truth table:** false for400..499 except408 when
nonnull config A14(0x35b4) is true; false for501/503; true otherwise.
408 with null config or disabled flag is false. This is the leaf's boolean,
not a generic HTTP success/retry recommendation; preceding producer branches
and auth recovery can change its effect. Preserve the potentially surprising
values until full caller context is traced rather than guessing from code ranges.

## Limits and reproduction

The original upload entry retained unnamed opcodes c8 and63. The decoded
follow-up below closes those mnemonic gaps; no complete upload algorithm
parity is claimed. No Rust
retry loop, resend or live operation is enabled. Higher-level cancellation,
all result producers and operation-specific idempotency remain open.

Reproduce via apk-smali with --signatures --exceptions for the named classes/
methods; apk-callers Ly8.<init> and MS4.A03. Whole N5N.invoke export retained
so selector5 can be checked against its actual switch and helper.

- [result-layout.txt](result-layout.txt) — SHA-256 `e75b2b788c7323ad43ab727858840c6d02a99fed0156e4569e387b26857d4d70`.
- [result-factories.txt](result-factories.txt) — SHA-256 `35c70d4cf74311541897e97d09be0f7e069f23cd070a87ca1741db4ab66bbbed`.
- [result-producers.txt](result-producers.txt) — SHA-256 `9229d37610839138aeb03b26f6f8b11eab5f73d31be909fa036ad378bceac99a`.
- [loop-callers.txt](loop-callers.txt) — SHA-256 `92d0b18919f4aba0ba9f44a6b057a1416896ed398676bfce7ccc62245490c3ef`.
- [default-loop-entry.txt](default-loop-entry.txt) — SHA-256 `1e02d26f53d6bc80746ffc9232ee90b2e6f8006443ade0279fa84cbb0c5a01d1`.
- [boolean-invoke.txt](boolean-invoke.txt) — SHA-256 `1a50315b6a4e1da82035c7bbe40ed4de365c33b2ce96c9fb110fcf06423b258b`.
- [default-predicate-constructor.txt](default-predicate-constructor.txt) — SHA-256 `a3a712c67193e1972c68c679190d9b171b333bf436684bd57cc2e13519309962`.
- [false-helper.txt](false-helper.txt) — SHA-256 `091ccaf77193f8e21cd701ee7bc16bb93273108386bf98dfe09985475423520f`.
- [default-predicate.txt](default-predicate.txt) — SHA-256 `0edd585061e944c2e78905dfaeda7ea22bdcada8a16b508abe4be3458f3e713a`.
- [upload-loop-entry.txt](upload-loop-entry.txt) — SHA-256 `ac1fb6a5758886d6ed260223fa6a356a1f6bf2c72a2f35992293ecef89e81beb`.
- [upload-result-producer.txt](upload-result-producer.txt) — SHA-256 `4104a8ea7c39e7992c93cbadc1c43bb0927223b2d697f26d516b30d86dce4e77`.

## Checked upload mnemonic correction (2026-11-04)

The disassembler now names63 as sget-boolean (21c), and c8 as
mul-float/2addr (12x), matching the
[AOSP bytecode table](https://source.android.com/docs/core/runtime/dalvik-bytecode).
MvO.ARB+00a8 reads ACk.A01:Z; +000c multiplies float v0 by v1 before
float-to-int. The literal multiplication/conversion bytes c810870a were
checked in classes11.dex at0x6ceb3c. The original export remains unchanged.
Two regressions cover register operands, instruction boundaries and truncated
prefixes; the boolean fixture deliberately uses an unresolved synthetic
field index and high register254 to exercise the full21c register byte.
The regenerated method contains no unnamed instructions. No actual upload
or request replay was executed.

- [upload-loop-entry-decoded.txt](upload-loop-entry-decoded.txt) — SHA-256 `e69c75ab378b9a8c1867f15831fa19a5184f0b4435e48de388194d8256f428a1`.
