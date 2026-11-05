# Retry key query: manager to IQ — APK 2.26.43.72

**Confirmed static / bounded construction audit.** Version code264307238,
APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected checked-DEX exports retain signatures/exception tables; these
are local developer-tool exports, not REA Evidence envelopes. **Observed:** no
runtime or account traffic. **Inferred:** the classic, non-identity, optional-
context-disabled branch is the comparison baseline for Rust's current query.
**Unknown:** effective flag0x6530, complete accepted context-JID grammar/type
mapping, TEE/hosted/PN-LID routing, PQ request selection, manager batching/backoff
parity, response/callback deadline and full queue recovery. No runtime change.

## Manager admission, not a universal timeout

[Prior job-added entry](../2.26.43.72-retry-query-ack/README.md) reaches14t.A07,
which delegates toA06 under scopeE78.A02. A06 locks and callsA00 with final
isPqSessionUpgrade=false. A00 has a distinct PQ-upgrade/backoff skip, expires
in-flight EX8 entries only when stored uptime+60000 is **less than** current
uptime, filters candidate devices and suppresses already-in-flight addresses.
That60000 is a manager-entry expiry, **not a proven IQ callback timeout** or
permission to replay an uncertain query. EX8 stores request context/metrics;
source integer12 from the retry job must not be invented as a wire attribute.

A00 callsA05 with candidate and identity-changed lists. A04 separately drains
its pending map, partitions candidates by scope/PQ and invokesA05 before clearing
the pending map. This chunk does not port those queues, reset backoff or relax
Rust's held query reservations.

A05 filters TEE-bot candidates, returns if the effective device list is empty,
and constructsBulkGetPreKeyJob. It conditionally partitions only for large
lists (size >=128 and a positive effective0x399 partition value smaller than
size). These native batching conditions are not a server maximum. Rust's32-device
bound is explicitly local and is not changed from this trace.

## Exact job-to-writer binding

BulkGetPreKeyJob.A0L reconstructs requested and identityChangedJids, creates an
Android Message with arg1=87 at01ec, and places id/jids/identityJids/fetchPqPrekey/
contextJid/sessionScopeOrdinal/isPqSessionUpgrade in its Bundle. It inserts its future into0Th.A05, dispatches via0Sf.A05, calls0ln.A0E
(which removes an optional router entry), then waits onFfK.get. The
[callback lifetime follow-up](../2.26.43.72-retry-refetch-callback-lifetime/README.md)
corrects the earlier mistaken description ofA0E as registration. Its numeric request
source contributes toDqO diagnostics; no source12/retry reason is placed in
this writer message or the final selected IQ attributes.

The complete previously retained
[0v0.A00 writer](../2.26.43.72-retry-version-error-ack/writer-dispatch.txt)
selects case87 at1696. It reads those Bundle fields, rejects PQ upgrade without
fetchPqPrekey, and registersDxh callback context. It creates one user per device:

- membership in identityJids => user/@reason=identity (1796..17d6);
- otherwise isPqSessionUpgrade => user/@reason=pq (17d8..17f4);
- otherwise only user/@jid (17f4..1808).

The selected retry job-added call passes false for fromIdentityChange, so a
registration mismatch is not automatically an identity refresh request. Rust's
plain retry user/@jid remains consistent with this non-identity branch. Neither
request-source12 nor remote registration alone proves user/@reason=identity.

The key node gets pqsupport=true only when fetchPqPrekey is true (1818..1836).
It gets context_jid only when effective flag0x6530 is true, context is nonnull,
and at least one0Dn.A0l/A0Q/A0h predicate passes (1854..1898). The retained
predicates are: A0Q type3; A0h type5; A0l types1/2 after excludingA0V. The A0V
meaning and complete type-to-JID class mapping are not resolved in this chunk.
Do not unconditionally add a context attribute, guess that gate is enabled,
or identify a JID class solely from these numbers.

Writer18b4..1948 constructs exactly root id, xmlns=encrypt, type=get and server
to, with one key child holding the optional attrs and user children, then calls
1r0.Cyt. Rust key_request/key_request_many reproduce that baseline branch.
Its existing identity-change helper marks only the explicitly affected device
set. PQ and effective context selection remain outside that baseline claim.

## Why no production field change

No missing unconditional attribute was found for the audited baseline. Adding
reason=retry/source=12 would be unsupported; adding reason=identity for every
registration change would conflate different native producer paths. Adding
pqsupport or context_jid for every retry would discard their actual predicates.
The remaining positive gaps are effective context/PQ policy, manager concurrency/
batching/backoff, timeout/callback lifecycle and recovery—not justification to
invent provenance or send extra fields. No universal request parity is claimed.

Reproduce each selected signature using `target/debug/xtask apk-smali <apk>
--class 'LX/14t;' --method A00|A04|A05|A06 --signatures --exceptions` (one method
per invocation), the BulkGetPreKeyJob class/methodA0L, and0Dn.A0l/A0Q/A0h.
The complete87 writer body was retained in the linked prior bundle; no duplicate
full writer export is needed. Checksums cover every other file in this bundle.
