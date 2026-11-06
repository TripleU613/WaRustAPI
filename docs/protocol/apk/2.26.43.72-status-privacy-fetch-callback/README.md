# Status-privacy fetch callback and error nesting

**Confirmed static**, WhatsApp 2.26.43.72 code 264307238, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target execution, account request or REA Evidence envelope claim. This closes
original callback eligibility/error handling from the
[bounded retriable-IQ trace](../2.26.43.72-iq-retry-job-cancellation/README.md).
Rust has no corresponding status-audience fetch/adoption path; no feature or
runtime retry is added from this evidence.

## Request and callback binding

GetStatusPrivacyJob.A0L builds IQ get, xmlns=status, id, typed to=0mL.A00 and a
privacy child. Helpers explicitly fill xmlns/type array positions 1/2; this is
not the classic privacy namespace or proof of the status setter. Property 3843
selects A0N's bounded wrapper versus ordinary A0W, with the same 32000ms caller
window/family 121 retained previously. Effective property value remains unknown.

The producer constructs 8RJ(7pl(job,AtomicInteger),future,17). Its exact generic
constructor stores future in A00, consumer in A01 and selector 17. The switches
bind selector 17 to BsS 01d2, Bvj 0470 and CK4 0036. CmQ unconditionally returns
KNW, so this original callback permits the wrapper's selected retry gate; the
budget, node location and other eligibility checks still apply.

## Root IQ attributes and nested error code are different inputs

The retained router 0ln.A01 passes its original IQ node unchanged to Bvj:
[dispatcher](../2.26.43.72-client-pong-schema/iq-callback-dispatch.txt).
T56.Bvj reads `code` directly on that supplied node, and its A00 reads `backoff`
directly there. The original 8RJ17 callback instead calls 6oj.A05 -> 1ud.A00/A01:
it walks direct error children, skips children without code, and returns the
first child with a code attribute parsed through 0HF.A00(text,0). Empty, invalid
or overflowed integer text yields 0; absent matching child also yields 0. No
exception-driven retry follows solely from those fallback zeros.

| Selected received shape | Static consequence, subject to ordinary callback admission |
| --- | --- |
| IQ has no root code; child error code=500 | T56 does not satisfy its root-code retry check and forwards. Original records 500/completes future; the enclosing job throws. |
| IQ has no root code; child error code=404 | Original records 404 and logs no settings found; future completes normally. No success parser/adoption runs. |
| IQ has no root code; child has invalid code | Original code becomes 0; this selected enclosing-job path does not throw for 500. |
| IQ has an eligible root code | T56 may schedule before original callback runs, only with remaining budget, KNW and exclusion-set checks. Child code cannot be substituted for that root lookup. |

These are derived static branch consequences, not captured responses or live
server behavior. Root/nested discrepancies are not merged into a synthetic code.

BsS17 sets the future exceptionally with RuntimeException. Bvj17 sets the atomic
code and completes the future with null, including 404. A0L waits on the future
for its caller window; after normal completion it throws only when atomic code
is 500. Job.A0N(Exception) logs and returns true. This closes that hook's result,
not scheduler retry limits, backoff, cancellation or durable attempt ownership.

## Successful settings are parsed before conditional adoption

CK4 requires a privacy child using 0Tp.A0G; absence throws LIL. A present privacy
child is iterated for direct list children. Each list's user JIDs are parsed as
0DY; null parses are skipped. Type must be present or LIL is thrown. Recognized
types are contacts, whitelist, blacklist and customlist; unknown nonnull types
are ignored by this selected parser. Only exact default="true" selects a mode.
An insertion-ordered set records mode values 0/1/2/4 respectively, with contacts
mode 0 used if that set remains empty. This fallback requires the successful
privacy-child path; it does not apply to a missing child or IQ error.

Custom lists retain id (empty if absent), listname, emoji, deleted=="true",
serialized member JIDs and default flag. Full 8DN constructor/field semantics,
member normalization, list source groups and persistence transactions are not
reconstructed here. The success branch checks 0bd.A0Q before writing settings;
its meaning is untraced. It conditionally writes distribution modes, legacy
allow/deny lists and custom-list updates through other gates, then completes the
future. This is not proof of atomic adoption or unconditional clearing. Required
child/type failures occur before that selected adoption block. No Rust audience
is changed to contacts from an error or absent response.

## Boundaries and reproduction

Rust account_privacy handles selected account_sync/readreceipts notifications,
not this status fetch; status recipients remain the explicit local sending
surface. Setter policy, effective flags, 0bd state/gates, full store transaction
behavior, malformed-response callback exception handling for this family, job
scheduler limits and durable uncertain-write recovery remain open.

Use xtask apk-smali APK --class CLASS [--method METHOD] --signatures --exceptions
for full GetStatusPrivacyJob/7pl, 8RJ.<init>/BsS/Bvj/CK4/CmQ, 6ok.A0f,
0Tp.A0G, 6oj.A05/A1Q, 6oe.A1T, 6oh.A1R/A1L, 1ud.A00/A01, 0HF.A00 and
1Xl.A1Y. Selected callback ranges retain the full-class export SHA256 and original
full-method exception metadata. Checksums cover every other file. Raw APK and
bulk 8RJ export stay outside Git. No claim of server acceptance or zero bans.
