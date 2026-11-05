# Durable contact-protocol refusal inspection

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No live account operation or APK execution. Static DEX exports are not REA
Evidence envelopes. The [full protocol-result parser, model and callback](../2.26.43.72-contact-protocol-backoff/README.md)
are retained in the earlier bundle. This bundle closes the numeric-reader gap
and adds durable local inspection; it does not finish the native policy port.

## Distinct parser and scope

0pD.CK4 removes the owned in-memory request before examining the response. With
its contact flag set it selects `usync/result/contact` and invokesA01. An error
child there is distinct from an outer IQ error and from per-user contact type.
The selected A01 branch first reads backoff through0Tp.A08 with absent default
7200seconds, wraps signed Java multiplication by1000, then caps above at3600000.
It next reads code through0Tp.A05 with absent default-1. A08/A05 default only
when the attribute reader returns null. Present strings delegate toA09/A06,
which useLong.parseLong/Integer.parseInt and wrapNumberFormatException inLIL.
A01 has no catch table. A malformed backoff prevents code parsing; a malformed
code leaves the previously computed numeric wait in local diagnostic metadata.
No diagnostic failure is interpreted as a valid wait or retry permission.

This differs from [outer-IQ refusal parsing](../2.26.43.72-usync-outer-refusal/README.md):
outer parsing starts with code(default0), then backoff(absent sentinel-1).
The shared UsyncRefusal value shape is reused, but its containing field names
identify the source and therefore its absent-value semantics and parse order.
Explicit backoff-1seconds is-1000milliseconds; it is not absence. Negative
and signed-overflow results remain distinct from positive waits.

## Local implementation

`actions::record_reply` now captures `contact_protocol_refusal` in the same
transaction as its accepted reply and settlement sequence. It selects only an
original iq/get xmlns=usync with requestedquery/contact and an outeriq/result
containingusync/result/contact/error. First direct children are used, matching
the selected native lookups. Unrequested protocols, unrelated nested errors,
outererrors and a missingerror do not manufacture this metadata. Nontext
numeric attributes receive a fixed parse diagnosis without echoing values.

The field is exposed through existing actions plain/JSON inspection and survives
contact projection and store reopen. It cannot be changed by a duplicate settled
reply. Historical rows default to absent/unknown and are not backfilled by
replaying old replies. Null/absent metadata does not assert protocol success,
no wait, integritypass or account absence. Empty-user rate-limit responses can
now be inspected as contact code429 even though the outer action isOk and
projection has no matching user. The action status retains its existing outer
outcome semantics; generic error_code/error_text are not repurposed.

The metadata does not suppress all per-user results. HxV's retained lightweight
consumer tests contact429 specially for an empty user array and follows a
separate nonempty-user path. The full native integrity, snapshot_recovery,
refresh and partial-result/adoption policy remains unported. Neither the
current locally composed Rust request nor its policy is claimed to be an exact
HxV factory. Observer-scoped deadline persistence, clock authority, native gate
reachability and notification/reset handling remain unfinished. No automatic
retry, wait enforcement, request change or live account operation is added.

## Independent verification and reproduction

ContactRefusalOracle.java runs desktop Java parsing/arithmetic on26 synthetic
cases, including absent values, Unicode digits, signs, malformed strings,
i32/i64 limits, multiplication wrap, one-hour cap and both fields malformed.
It never executes the APK or accesses an account. Regenerate with:

```sh
java docs/protocol/apk/2.26.43.72-contact-protocol-refusal/ContactRefusalOracle.java
```

Rust compares every parsed value, attribute-presence bit and fixed failure
against that independent fixture. Scope/first-child/redaction tests and an
actual queued contact-check test cover outerOk plus contact429, no matching
user, durable reopen, projection, duplicate settlement and no action replay.
Reproduce extra source exports withxtask apk-smali APK --class 'LX/0Tp;'
--method A05/A06/A09 --signatures --exceptions (one method per invocation).
The previous bundle retains A08 and0pD.A01/CK4/HLG. Checksums cover all other
files in this directory; raw APK/private data stay outside Git.
