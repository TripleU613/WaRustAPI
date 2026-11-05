# Trusted-contact token issue: callback ownership and send-time gap

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX exports, not runtime observation or REA Evidence envelopes. No
account operation, token generation on a live service or real user traffic.

## Distinguish the token protocols

The current class is com.whatsapp.infra.privacytoken.jobqueue.job.
GeneratePrivacyTokenJob. The shorter old package selector finds no body in
this build; the exact generate-tc-token and trusted_contact anchors locate it.
A0L builds privacy/set/to=s.whatsapp.net with one tokens/token, type=
trusted_contact, user JID and stored timestamp t, without token bytes.
It passes logical request299 (0x12b) and32000ms (0x7d00) to0Sf.A0W
at0144..015c. This is not the separate client-auth token/nonce request314
used in earlier writer-lifecycle research. Similar duration is not identity.
[Shared registry/admission evidence](../2.26.43.72-abprops-callback-timer/README.md)
retainsA0W/A0A and0ln.A08: positive timer registration precedes dispatch,
and promotion does not restart it. Timer and reply compete for ownership.

## Native callback capture and job outcome

A0L constructs8R8 selector3 with captured timestamp,user,job andFfK future.
The complete constructor's field writes establish A01=timestamp,A02=user,
A03=job,A00=future. CK4 selector3/default reads those captures, calls0ht.A0V
with the captured user/time, then completesFfK.BsT(null). It never reads the
reply-node payload. Bvj selector3 rejects the future withGiq(node,id), while
BsS rejects with7gg(id). A0L unmarks87x pending membership after normal future
completion, or when input is missing/old; its caught Exception is rethrown
without that normal unmark. A0K cancellation unmarks separately.

A0N's selected retry predicate declines only when the cause isGiq with an IQ
error code400..499; other cases return true. This is job retry eligibility,
not HTTP status, a retry interval or permission to replay uncertain Rust IQs.
Full job-queue retry/backoff/terminal dispatch remains unported.

0ht.A0V normalizes the user, uses conditional sent/real_issue timestamp updates
and an insert fallback before later cache/counterpart effects. Complete selected
body is retained; this chunk does not port its persistence/counterpart behavior
or equate a queued sent timestamp with successful issuance.

## Send-time gap identified in the preceding chunk

**Historical gap at17469da8:** the following reservation/ownership and mandatory
age-floor correction is now [implemented in the follow-up](../2.26.43.72-token-reservation-refresh/README.md).
Stricter configured admission remains open. NativeA0L reloads0ht.A0O immediately
before construction. It requires a current stored row and captures its current
7p3.A00 timestamp. It compares that timestamp against0hu.A01, skipping strictly
older values and admitting equality. The preceding Rust schedule_after_ack_in froze user/t
into the durable action at message-ACK time; generic action reservation then
did not reload current token state or check this execution-time threshold.
A long-held or superseded queued action can therefore differ from the native
job's eventual input. Do not rewrite a Sent row or fabricate a fresh timestamp
to hide this gap. A proper follow-up needs queued-action/pending-job ownership,
current state, supersession/coalescing and a reproducible expiry decision.

0hu.A01 is min(A00 using properties996/997, A02). Effective bucket properties,
A02/current-clock policy and full pending-job scheduling remain unresolved in
this current-build trace; Rust's182-day scheduling fallback is not substituted
for this native execution gate. A0L also skips raw JID types26/27 via0Dn.A0O.
Their complete type/runtime mapping is not established here; Rust currently
schedules only PN/LID domains. No broadened target admission is added.

## Rust correction and verification

**Implemented offline:** metadata-only trusted-contact issue shapes get their
own32s callback timer from immediately before awaited local write. Dedicated
expiry releases live authority only, retaining durable Sent uncertainty and
the original scheduled SendState marker. It neither retries nor clears the
window/changes keys or timestamp. Account-privacy setters sharing xmlns=privacy
are excluded. Typed privacy-token result follows selector3's payload-ignoring
success; outer error/code and other nested-refusal policies remain intact.
No native success-store adoption or job-queue retry policy is claimed.

Paused-clock tests cover PN/LID,31/32s boundary,5s local-write delay, result/
error/duplicate, synthetic result containing error403, and exact checkpoint
preservation on expiry. Another message ACK inside the scheduling window queues
nothing after any outcome. A same-namespace privacy setter gets no invented
timer. Normal-driver expiry test now includes the real token scheduler with
33s local-write delay, late reply, subsequent receipt ACK, no resend and clean
stop. The synthetic extension is not an observed server response. Rust dispatch
is serialized after awaited local writes, not full Android concurrency parity.

## Reproduction

`target/debug/xtask apk-xref APK --anchor 'generate-tc-token' --anchor
 'trusted_contact'`; `apk-smali APK --class CLASS [--method METHOD]
--signatures --exceptions`. Retained: GeneratePrivacyTokenJob full class(job),
8R8.<init>/CK4/Bvj/BsS(constructor/result/error/failure),0Dn.A0O(user-exclusion),
0hu.A01(expiry-threshold),0ht.A0V(success-store). Only selector3 and the
identified job/store paths are interpreted; unrelated branches remain in the
complete method exports. checksums.json covers every other file. APK, real
credentials, generated token bytes and bulk trees are not committed.
