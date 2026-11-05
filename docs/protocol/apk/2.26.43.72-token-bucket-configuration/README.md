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

## Adapter follow-up and returned-value inspection

**Confirmed static:** the existing [privacy configuration trace](../../privacy-mex-config.md#effective-integer-configuration-has-two-authorities)
already retains the exact same 00E integer getter and 1XN/1XO adapters under
current-APK REA records. Reuse that evidence rather than treating this as a
newly discovered general resolver. 00G.A03 has firstAccessValue=false and
selects legacy cache A0P. The integer result adapter is 1XN selector 0;
0A7.B8Q zero yields integer zero with INVALID_SPECIFIER_MISSING_IN_WA_LOOKUP_MAP,
nonzero calls Arl and narrows Java long to int. Direct adapter 1XO selector 1
likewise resolves B8Q and narrows Arl; its extra mode argument covers sampling.
These are not AB-response integer coercion. 0A7 implementation mapping for the
four token IDs and readiness transitions are still unresolved. The generic
superclass-only apk-virtual-overrides query for 0A7.B8Q returns only the abstract
interface declaration; it cannot enumerate interface implementations, so that
negative result does not prove the lookup is absent.

**Implemented offline:** existing abprops-query --show now includes
`token_bucket_values`, a map containing only returned integer property IDs
865/909/996/997. Decimal JSON keys map to signed i32 values; null is an invalid
last returned value, and an absent key is missing/inapplicable. Duplicate rows
use the last value even when invalid. Nonpositive values remain integers so
they can be inspected without silently selecting defaults. Full hashless replies
remain not_modified with an empty map, while deltas may carry values without a
hash. Unknown values, hashes and other private metadata remain excluded.
The map is observational and never read by token scheduling/reservation.
Legacy serialized inspections default it to empty; saved replies are reparsed
without mutation, so prior durable responses also gain this inspection.

Tests cover valid returned values across encrypted-store reopen and refusals,
last invalid duplicates, overflow/whitespace, zero/negative values, hash/delta
branches, exclusion of unrelated values, JSON roundtrip and legacy defaults.
Effective configuration resolution and the configured scheduling/admission port
remain unfinished; exposing raw rows is not evidence of native eligibility.

The subsequent [concrete MobileConfig binding trace](../2.26.43.72-token-mobileconfig-binding/README.md)
closes the four token specifiers, case66/252 DI bindings and one readiness/cache
transition. Effective SDK values and the complete lifecycle still remain open.
