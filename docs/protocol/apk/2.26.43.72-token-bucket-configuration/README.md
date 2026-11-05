# Trusted-contact integer configuration: packaged entries and resolver boundary

**Confirmed static**, APK 2.26.43.72 code 264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
DEX instructions only; no account operations, target execution or REA Evidence ID.
This follows the [age-floor proof](../2.26.43.72-token-reservation-refresh/README.md).

## Packaged entries are not effective account settings

08Y.A0Q inserts integer keys 865/996 with 604800 (seven days) and 909/997
with 4 into its supplied builder. The boxed week is defined at 02ae..02bc
and not replaced before the selected token entries; the boxed four at
17d8..17e2 is not replaced before those entries. The destination-write
inventory in default-entries.txt includes move-result writes, not just consts.
The 08Y constructor passes integer builder v13 into A0Q at 43be, then
buildKeepingLast stores that builder in A02 at cc16..cc28; A1B returns A02.
These selected entries are confirmed. A whole-map last-writer proof across
all helpers and constructor additions has not been performed in this chunk;
do not turn these selected assignments into an effective-value claim.

**Derived, conditional:** if all four effective settings equal those entries,
0hu.A01 and A02 both use max((now/604800-3)*604800, now-15724800).
For positive contemporary epoch times, this threshold is between 21 days and
just under 28 days old. It accepts equality, as retained in the previous bundle.
Scheduling 87x.A04 separately compares current/stored buckets using 996.
Consequently the Rust 182-day fallback is not a faithful implementation of
these positive configured branches. This is a concrete compatibility gap;
it does not establish what the server would do with an older token request.

## Getter path and bounded precedence evidence

00E.A0a selects 00G.A03 and A02 with null optional Integer fallback. A02 supplies
1XK selector 1 and integer metadata to A06; that selector calls A01. The
selector link is checked in the local full 1XK.invoke disassembly; reproduce
it with the command below. A01 first consults A0h, rechecks under a monitor,
then obtains A1B's integer default and calls SharedPreferences.getInt with
the decimal property key and that default. Missing registered/default value
with null optional fallback throws Unknown IntField; it is not numeric zero.
A0h selects A07 versus A0P using 00G.A01. A01 also populates the selected cache.

A06 has additional initialization/mode/cache branches and MobileConfigUnsafeContext
plus 0A7 guards. Its 0AD result HJp.A02, when nonnull, can supply the returned
value at 02c4..02fa; otherwise the fallback value is returned. Thus raw AB
reply rows alone cannot prove the value A0a returns. The complete A06 method
is retained without interpreting unrelated mode/telemetry branches as gates.
The exact 1XN/1XO adapters, 0A7 mapping, 00G.A03 flags, cache refresh/invalidation,
preference application and per-account MobileConfig overrides remain open.
No new value is applied to Rust sends, and no profile/default override is invented.

## Reproduction and search boundary

Use existing CLI `target/debug/xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions`: 00E.A0a/A02/A06/A01/A0h and 1XK.invoke.
08Y.A0Q and its constructor/getter can be exported with class LX/08Y; and
method A0Q/<init>/A1B. default-entries.txt deliberately retains selected
ranges and a destination-write inventory, not an unlabelled full constructor.
Its SHA256 identifies the local full 08Y export used to select the excerpts.
The full current 0hu and 87x scheduling branches are in the previous bundle.
Checksums cover all other files here. Effective policy port and full native
pending/retry/counterpart lifecycle remain unfinished.
