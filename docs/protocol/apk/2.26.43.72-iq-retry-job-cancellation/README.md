# Disconnect cancellation map owns delayed retriable-IQ jobs

**Confirmed static**, APK 2.26.43.72 code 264307238, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution, account operation or REA Evidence envelope claim. This closes
selected ownership/effect gaps in the
[shutdown cancellation-map trace](../2.26.43.72-iq-shutdown-eligibility/README.md)
and [ping registry lifecycle](../2.26.43.72-client-ping-registry-lifecycle/README.md).
It does not establish a general IQ replay policy.

## Job-map ownership and normal cancellation path

Exact physical field census for 0Sf.A0E:Map returns its constructor assignment
and reads in 0Sf.A0J, T56.A02 and TKl.invokeSuspend. T56.A02 constructs selector-5
TKl(plan,sender,wrapper,null,continuation,5), launches it through 2L8.A1S and stores
the returned job under its IQ ID while holding the map monitor. The exact generic
constructor binds plan to A01, sender to A04, wrapper to A02 and the null optional
replacement stanza to A03. These are delayed retry jobs, not the IQ callback maps
A0I/A0H or a durable replay queue.

0Sf.A0J calls 0Sp.AG3(null) on each stored job and clears the map on normal return.
A callback throw can leave the loop before clear. The selector-5 coroutine's
CancellationException catch routes to T56.A01(null,wrapper), which forwards the
original callback's BsS(id). This is a failure notification, not successful
completion or evidence that no write ever occurred. Cancellation is requested,
not joined; this trace does not prove synchronous completion or retraction of an
executing write. Full coroutine launcher/runtime cancellation internals remain
outside this bounded source trace.

## Retry decisions are explicit and scoped

0Sf.A0N wraps an original callback in T56 and registers it before dispatch. When
the ready flag/channel is absent it instead registers the callback and passes the
message to 0TX.A02. The exact A0N caller census has only TKl's recursive retry and
GetStatusPrivacyJob.A0L. No inference applies this wrapper to ordinary A0A/A0B
pings, arbitrary actions or every transport request.

| T56 input | Selected behavior |
| --- | --- |
| CK4 successful result | Forwards the original callback's CK4 without retry. |
| BsS delivery failure | Computes an eligible plan, then asks original CmQ(id). KNW permits scheduling; Cgb or absent plan forwards original BsS. Other result variants throw in this selected branch. |
| Bvj error | Requires parseable signed integer code, eligible plan, CmQ=KNW, code in inclusive 500..599, and exclusion-set absence before scheduling. Otherwise forwards original Bvj. |

The excluded codes are **501, 531, 532, 533, 536, 542, 547, 548**. Constants,
range constructors and range-end arithmetic are retained; no HTTP policy is
inferred from these IQ error numbers. Missing/unparseable code logs and forwards.
The callback selector CmQ is not bypassed by a server backoff attribute.

A00 reads the supplied node's `backoff` as Java signed decimal Long and multiplies
by 1000 with long arithmetic. It then calls shared 1C8.A01; a null budget result
prevents a plan even when the server supplied backoff. With available budget, a
parsed override replaces the generated delay. Remaining budget is wrapper A08
minus elapsedRealtime since construction, and delay must be strictly smaller than
remaining budget. No positive clamp or overflow check is present in this helper;
this is source arithmetic, not permission to fabricate negative waits.

The budget generator increments its count before returning a delay, and is called
before CmQ and error-code eligibility are checked. Thus a budget count is not a
count of proven network retries. It returns null once count reaches configured
A03; before saturation it scales base by 2^min(incrementedCount,50), takes half
plus abs(random.nextLong % scaledBase), then caps/sticks at A04. Caller values,
Java overflow/remainder and generator state matter; this is not a new universal
Rust delay formula. No independent runtime arithmetic oracle is claimed here.

## Delayed execution preserves identity and the remaining window

SMN stores selected delay in A01 and remaining window in A00. TKl5 delays by A01,
then rechecks elapsedRealtime minus wrapper construction time against A00.
Exhaustion forwards BsS. Otherwise it removes the job-map entry, then calls A0N
with the same ID, stanza, callback, shared generator, family, flag and remaining
window. This recursive call creates a new wrapper with a new construction time.
Cancellation catch also forwards BsS. The exhausted branch precedes the explicit
map removal; no universal per-branch job-entry removal is claimed.

GetStatusPrivacyJob's selected property 3843 enables A0N with a generator whose
constructor inputs are count limit 3, cap 3600000ms and base 1000ms, plus a 32000ms
window and family 121. Disabled property uses A0W with 32000ms instead. The request
is status IQ get with privacy child. These are caller literals; effective property
selection, all enclosing job-queue retry behavior and original callback semantics
are separate. Do not turn these values into defaults for all IQs.

## Rust implications and open boundary

No production Rust change or live traffic. Existing durable Sent/uncertain
reservations and link-local callbacks remain unchanged. This trace identifies a
bounded, callback-authorized native mechanism that can resend with the same ID;
it does not prove idempotency, server acceptance or safe replay of arbitrary
uncertain writes. A parity port needs the original caller/callback eligibility,
budget lifetime, durable ownership and cancellation ordering, rather than a
generic 5xx retry loop. Complete queued A0I disposal remains covered only by the
separate writer drop/drain evidence; cancellation of this job map does not clear it.

Remaining: original status callback CmQ/effective flag, complete job-queue policy,
0TX admission/disposal, cancellation after map removal/before write, already-claimed
callback ordering and durable ambiguous-commit recovery. Dynamic aliases and
reflection are not excluded by a physical field census. No ban guarantee follows.

## Reproduce

Use xtask apk-smali APK --class CLASS [--method METHOD] --signatures --exceptions
for full T56/1C8/SMN, 0Sf.A0J/A0N/<clinit>, TKl.invokeSuspend/<init>, 0Sp,
0Si.<init>, 0Sg.<init>, 0Sk.A00 and GetStatusPrivacyJob.A0L. Selected coroutine
ranges retain full export SHA256 and exception code-unit metadata. Sender method
extracts retain the full-class export hash. Use apk-field-refs for class 0Sf,
field A0E, type Ljava/util/Map; and apk-callers for class 0Sf/method A0N. Checksums
cover every other file; raw APK and bulk sender/coroutine exports remain local.
