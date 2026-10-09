# Receipt clock, device cache and concrete columns

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [18-topic recipe](apk/recipes/2.26.37.71-receipt-clock.json) preserves
[full method signatures and branches](apk/2.26.37.71-receipt-clock/targets.json).
This extends [receipt factory/store dispatch](receipt-factory.md), without
changing Rust's independent delivery/read/played observations.

## Three timestamp input policies

**Confirmed:** direct Cmk.A03 at 04e0..04fe reads optional root `t` through
8kT.A18, calls 0Gv.A01(string,0), then multiplies the returned signed long by
1,000. The resulting milliseconds reach DDH's constructor at 05a2..05bc.
8kT.A18 calls 0hs.A0M with null default. 0Gv.A01 returns the supplied default
for null/empty input or NumberFormatException, otherwise Java Long.parseLong.
Consequently absent, empty and invalid direct timestamps become zero; a valid
negative value is not silently replaced with the current time. Multiplication
is DEX mul-long, not checked arithmetic; overflow is not a parse failure.
Do not infer the server's semantics for malformed values from this client code.

**Confirmed:** the offline aggregated `participants` branch at 00aa..00be
reads each participant's `t` using 0hs.A07 and multiplies by 1,000.
A07 obtains required A0L then invokes A09. A09 wraps NumberFormatException
as B7l; it does not return zero or the current time. Required attribute absence
semantics inside A0L remain **untraced** here. Native error text includes the
input; a Rust port must retain redacted diagnostics rather than copy that log.

**Confirmed:** the successful-message participants branch at 0340..0388
gets 1xj.A02(clock provider), divides milliseconds by 1,000, then supplies that
seconds value to 0hs.A08 for each participant's optional `t`, multiplying the
result back by 1,000. A08 returns its default only for an absent attribute;
a present malformed or empty value goes to strict A09 and throws. Thus this
branch has a distinct fallback, rounded down to whole seconds for positive
clock values; it cannot share the direct zero-default parser indiscriminately.

1xj.A02 resolves 08B and calls its static A00. That clock first samples Android
elapsedRealtime. If A03 is nonzero it returns A03+elapsed; else if A04 is
nonzero it returns A04+elapsed; otherwise it returns currentTimeMillis-A02.
**Unknown:** producers/units of the adjustment fields, calibration events and
which branch is active for the retained account. This confirms a potentially
adjusted millisecond clock, not permission to invent server time or telemetry.

## Earliest positive timestamp and columns

**Confirmed:** 7hf.A01(DeviceJid,long) rejects nonpositive input. With no
existing 7QL it creates one, writes A00 timestamp and inserts it. With an
existing row it returns false when existing A00 is at least 1 and no greater
than the new time; otherwise it updates A00 and returns true. It does not
rewrite the existing 7QL.A01 nullable primary version. Alongside 0Z5.A0A's
batch minimum and existing-row update, this establishes earliest-positive
retention across already cached device rows, not latest-arrival replacement.
A duplicate timestamp is unchanged.

The ordinary 0Z6 overrides resolve 0Z5's dynamic schema calls:

| Method | Confirmed return |
| --- | --- |
| A0F | `message_row_id` |
| A0G | `receipt_device` |
| A0H | `MessageReceiptDeviceStore/` (log prefix) |
| A0I | `receipt_device_jid_row_id` |
| A0J | `receipt_device_timestamp` |
| A0K | `primary_device_version` |

These are concrete ordinary-message columns. They do not identify a read or
played column. The [version writer trace](receipt-version.md) covers current
nullable version insertion and existing-row timestamp-only updates. Read and
played are still independently meaningful message facts; a device receipt time
is not proof of playback or a reason to promote every fact to delivery.

## Conditional namespace mapping

**Confirmed:** 0Z5.A01 examines the message-key chat with 0DE.A0l. If false
it returns the original DeviceJid. If true it passes device.userJid through
0Yo.A07, requires a UserJid result and builds a DeviceJid preserving getDevice().
0DE.A0l returns true for nonnull JIDs of numeric types 0, 7, 11, 18, 22, 26.
Those numeric types are retained without assigning untraced human names.

0Yo.A07 can return its original input, or resolve a user through 0Xw.A0I.
Its branches depend on 0DE/0hd predicates, 0Yo.A0H and, on one branch,
0Fs.A0F's 0Ze.A0l enum. Missing mapping returns the original input. The
retained `LidUserJid` cast label anchors one branch, but complete type/default
and mapping authority remain **unknown**. No proof here authorizes arbitrary
PN/LID merging; existing Rust authenticated-association guards remain required.

**Unknown/unported:** complete 15A status admission, native cache/SQL loading,
identity-change row deletion/invalidation, adjustment-field producers, exact
namespace mode authority and remaining batch parsing. The Rust normal outbox
preserves immutable original device IDs and nullable versions; this research
does not introduce a mutable receipt ledger, synthesize historical timestamps,
change original sends or relax uncertain-outcome/retry guards.

Reproduce using apk-research-bundle with the recipe, its --output directory
and --check. New releases require a separate anchored bundle before comparison.
No CLI/API capability changed and no account or network operation ran.

Receipt-clock validation: 18 source topics / 23 files / 82,330 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified 18 unchanged
topics and four unchanged metadata artifacts; this does not establish parity
between different releases. Repository hygiene checked 1,350 local links and
29 APK bundles / 8,141,757 bytes. Diff hygiene passed. Changes are research/docs
only; no new runtime tests, live receipt or account changes are claimed.
