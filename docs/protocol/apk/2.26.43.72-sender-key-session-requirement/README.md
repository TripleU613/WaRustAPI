# Sender-key session requirement boundary — APK 2.26.43.72

**Confirmed static**, DEX text exports, not executed APK behavior or REA Evidence
envelopes. Pinned APK SHA-256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Identity reverified in the preceding message-ACK forwarding chunk. Commands:
`target/debug/xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`; omit method for the two complete requirement/policy classes.
`apk-field-refs` uses exact class, field and descriptor. Offsets are relative
bytes; signed jumps use two-byte DEX code units.

## Why the sender-key requirement is not a distribution ACK producer

AxolotlMultiDeviceSenderKeyRequirement extends AxolotlMultiDeviceSessionRequirement.
Its two declared bodies are constructor and dependency injection Cf0. Constructor
passes fetchPqPrekey=false to the base and saves groupParticipantHash,
useLidForEncryption and useParticipantUserHash. Inherited methods contain the
specialized behavior; the subclass name alone does not identify a separate send.
Exact groupParticipantHash field references found only this constructor's write
and the base A01 read. This is a physical DEX census, not reflection coverage.

### Recipient selection before session checks

Base A01 returns a collection. The ordinary sender-key branch starts at00f2;
status subclasses have separate branches and must not be conflated with it.
At013a..0162 an explicit nonempty EfU.A04 takes EfU.A00 directly to the common
return/caching tail. Otherwise EfU.A01 supplies candidate set v6 (0162..016e).
A valid group is required through 0nV.A00 and a nonnull assertion.

At01de..0206, useParticipantUserHash selects 0i9.A0I versus A0H (the alternate
call is at0312). The selected current hash is compared with groupParticipantHash.
**Important ordering:** on mismatch, 0206's +0xd6 reaches03b2, retaining the
original candidate set v6. It bypasses the later sent-key recipient filtering.
This is evidence against assuming old sent-key markers alone narrow every
session prerequisite after membership/hash change. It does not prove a new
outbound SKDM is sent or acknowledged for each candidate.

On the equal-hash path, A0A loads participant state. Selected type/gate branches
can inspect maps A0B/A0C, ElE.A03 and ElE.A01 (including EHo.A00-derived device
string), own-device predicates and EeT predicates. The alternative uses
1fm.A0P then retainAll(original candidates), and a selected 0Dn.A0Q branch
adds candidate devices satisfying BRq. These are conditional selection paths;
full numeric type/flag meanings and helper semantics remain untraced.
The tail at03b2 maintains cached list/count and resets batch position on its
selected changed-input conditions. No durable Rust state contract follows from
these native in-memory cache fields.

### Readiness and PQ selection are separate from sender-key delivery

A08 reads ELJ.A00. A09 has two batching paths: A00/ImmutableList and A02/list
of lists. Both call A0A with candidate count. In the ordinary sender-key case,
A0A calls Ex8.A07(group,null,0i9.A0G(group),count) once unless already latched;
it writes the resulting boolean into both subclass A08 and A07. Status
subclasses select false in this method. Other requirement types return their
fetchPqPrekey flag. This is a session-policy decision, not an ACK result.

When the boolean is false, A09 checks 0WJ.A18 for each selected batch; when
true, it instead checks A03/A04's missing-session set. A03/A04 build E7N-scoped
session descriptors via F3k.A08 and call 0WJ.A0i, with an extra selected-scope
pass on the true path. F3k.A08 derives scope from sessionScopeValue and delegates
construction to A07. 0WJ.A18 explicitly implements containsAllSessions, with
conditional locking and invalid-session cleanup. Its deeper store helpers and
all E7N enum meanings are not fully traced here; no network absence claim is
made about unexpanded transitive callees.

Ex8.A07 gates selected group policy on positive group metadata, configurable
metadata/count bounds, a feature flag and A02 readiness. Its nullable Boolean
argument is only a fallback after eligibility checks; null is not a blanket
permission to enable PQ. Effective values and lifetime/reset policy remain
unknown. Do not port guessed feature defaults or change PQ advertisement.

## Producer search boundary and remaining work

The includeSenderKeysInMessage field census has five physical references: one
constructor write, A00 diagnostic read, A02 validation read and two A06 reads.
No second writer was found in this exact census. Fresh SendE2EMessageJob.A0L
and FhB.invoke exports retain the two A06 invocation paths. FhB passes a literal
true as A06's last boolean; this is not proof of a separate distribution job.
The large A0L/A06 branch graph is not fully resolved by this inspection.

The skdm string census had five entries; the obvious status-distribution anchor
resolved to Ed5.A00. That method builds a new 18U from the supplied 1dW and
clock then calls 0wZ.A0O. It is status-specific and does not identify the normal
group/reaction direct-distribution producer. Its full scheduling path is unknown.
The exact includeSenderKeysInMessage anchor had no xrefs; the longer validation
string identifies SendE2EMessageJob.A02 instead. Source-file strings need not be
instruction-referenced anchors.

**Unknown:** whether the local reaction direct-SKDM workaround corresponds to
any native ordinary producer, its expected ACK tuple if so, all selected group
policy/helper semantics and durable per-distribution confirmation. Retain the
local experiment/inference labels; do not treat an unlocated native producer as
proof one exists or proof none exists. No Rust source, replay, reset, live
request or feature-flag behavior changed.

`checksums.json` records bytes and SHA-256 for every other file.
