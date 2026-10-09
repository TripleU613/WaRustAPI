# Incoming keep ordering metadata

2026-10-09. **Confirmed static**, APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [four-topic recipe](apk/recipes/2.26.37.71-keep-timestamps.json)
reproduces [constructor and parser evidence](apk/2.26.37.71-keep-timestamps/targets.json).
The [write/caller inventory](keep-timestamps-inventory.txt) records negative
candidates and other timestamp writers, rather than assuming immutability.

## Native selection and clock sources

**Confirmed:** [0Y5.A00](apk/2.26.37.71-group-keep-outbox/keep-store.txt)
loads the current keep add-on via 0Z0.A06 with type 0x44. At
`06ee..0748`, a candidate with smaller 178.A0F is stale; a larger value proceeds.
Equal A0F compares BnF.A02 at `0752..0770`, again preferring the larger time.
Equal times compare the add-ons' own 1gc.A01 IDs at `077a..07a4`:
`current.compareTo(candidate) > 0` proceeds to `0828`, so the lexicographically
**smaller** candidate ID wins this tie. A nonnull candidate can replace a null
current ID; a null candidate cannot replace a nonnull current ID. Equal IDs
and two null IDs take duplicate reason 7. Java String comparison is UTF-16.
Passing this selection gate does not prove permission or expiry acceptance:
subsequent branches still inspect the parent and other eligibility conditions.
No Rust keep application policy is inferred from this gate alone.

**Confirmed:** [D9L.A01](apk/2.26.37.71-group-keep/keep-receive.txt)
constructs BnF from CoT.A05 and the envelope key at `05f0..060c`, and records
BV3.timestampMs in BnF.A02 at `0656..0682`. The descriptor's timestamp field
is 3, distinct from the parent key's ID and the add-on's own envelope ID.
D9L reads the primitive timestamp directly without a presence test here, so
native missing-field handling supplies the protobuf default zero; the Rust
record retains the original absence instead of collapsing it into explicit zero.

**Confirmed normal receive provenance:** in the existing
[envelope evidence](apk/2.26.37.71-group-envelope-own/targets.json),
CnN.A02 parses stanza `t` into CaF.A09. CgC.A01 (`00c2..00e0`) parses that
string with 0Gv.A01, using 08B.A00 divided by 1000 as fallback, then multiplies
seconds by 1000 and writes boxed milliseconds to Cih.A0J (`01c4..01d0`).
Cih.A01 reads A0J.longValue into v46 (`0018..0034`), the first long passed to
Bp2's constructor; Bp2 passes it to CjN.<init>, which stores A03.
Cna.A01 (`0020..002a`) passes CjN.A03 through C9O.A00 to CRD.<init>. CRD
stores A0Q, then CRD.A00 passes it to CoT.<init> as A05. D9L's BnF/1iZ
construction passes that value to 178.<init>, which stores A0F.
Cih.A04 is a different long passed to Bp2; it is not the source of CjN.A03.
0Gv.A01 returns its fallback for empty strings or NumberFormatException;
its exception table proves the invalid-input branch. Native parsing does not
impose the Rust event's nonnegative/checked-multiplication policy.

**Unknown/unported:** A0F mutations beyond this constructor path (21 total field-write sites), adjusted-clock
offset producers, full permission/expiry application, edited keep handling,
missing-parent scheduling and cross-device keep state synchronization. The
constructor trace establishes this normal incoming path, not every caller.
The earlier Cna.A00 candidate was a stanza-type helper and established no clock
provenance; its negative result is retained in the inventory.

## Rust recording and compatibility

**Confirmed local:** IncomingKeep and MessageChange::KeepInChat now preserve
optional `sender_timestamp` in milliseconds. Absent wire fields remain None;
explicit zero, negative and signed-64-bit extrema retain their actual values.
The event also records optional `update_id`, the supported envelope ID, while
MessageUpdate.key.id remains the original target's ID. Missing or unsupported
action IDs remain unknown. Stanza-derived `server_timestamp` and EventEnvelope
local `committed_at` remain separate. Missing/invalid stanza times are not
replaced with an invented server timestamp or an assumed adjusted clock.

`events` exposes these fields in its existing full JSON journal output;
`events --summary` continues to report redacted event kinds. They are received
facts, not a completed keep/expiry scheduler. Historical journal records load
with both new fields None and serialize in their original shape; existing
publication markers are not reset or reprojected just to fill these fields.
Adding fields changes Rust enum construction/exhaustive patterns; serialized
historical compatibility is covered independently by a literal old envelope.

**Confirmed offline:** literal field-51 fixtures cover omitted/zero/42/negative
and maximum signed timestamps, plus malformed varints. Direct projection
separates target ID, action ID, payload time and stanza time. The 24-case group
fixture checks the new metadata alongside author/device attribution and verifies
identical journal records after reopening without repeat publication.
Fresh live keep/unkeep reception and handset application remain **unknown**.

**Confirmed verification:** 786 workspace/all-feature tests passed (658 API,
4 property, 9 crash, 51 CLI, 1 API-symbol and 63 xtask), strict all-target/all-feature
Clippy, Rustdoc with warnings denied, default CLI tests, API/library-symbol
checks, CLI-reference regeneration, formatting and the experimental CLI build.
The new bundle reproduces four topics / nine files / 41,117 bytes. Hygiene
checks 1,264 local links and 21 APK bundles totaling 6,489,409 bytes.
The first strict Clippy pass caught an unnecessary qualification in the new
projection; it was corrected and the complete check script then passed.

**Observed offline saved-state check:** all 497 capture rows and normalized
shapes remained unchanged. Peer keys, actions, receipts, pending outbox and
event summaries remained identical. Two retained group reaction targets were
valid; no saved group keep payload was available. This validates compatibility
with this saved account, not fresh keep semantics or handset rendering.

**Confirmed comparison check:** the new checksummed bundle compares to itself
with all artifacts/topics unchanged. Its recipe maps clock/parser changes to
the three affected Rust files. This checks the comparison plumbing; it is not
evidence of compatibility with a different APK release.

**Observed live receive check:** one 45-second receive-only window authenticated
and stopped cleanly, with chat sends and automatic reconnect disabled. Captures
grew from 497 to 502; every earlier normalized capture shape remained retained.
Peer keys, actions, receipts, pending outbox and durable event summaries stayed
identical. The same two saved group reaction targets remained valid. No group
keep payload was received or retained, so live preservation of keep timestamps
and IDs, fresh group sends and handset application remain unverified. Account
inspections finished before the live process acquired storage ownership.
