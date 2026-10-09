# Token feature inputs — APK 2.26.37.71

**Confirmed static, 2026-10-09.** Source is build 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [twelve-topic recipe](apk/recipes/2.26.37.71-token-feature-inputs.json) and
[checked snapshot](apk/2.26.37.71-token-feature-inputs/targets.json) preserve
17 files / 1,565,314 bytes, including register flow, signatures and catches.
These findings extend [token canonicalization](token-canonicalization.md) and
[feature configuration](feature-configuration.md); they do not establish effective
settings for any account.

## Default registrations and getter chain

**Confirmed static:** `07t.A0L` registers Boolean property 15491 (`0x3c83`)
with false: `v1` receives boxed zero at `000e..0014`, and the builder put at
`1594..15a6` uses that register. `07t.A0X` likewise registers property 14303
(`0x37df`) with false at `055a..056c`, using its boxed-zero `v1`. Nearby true
values use a different register. These are the counterpart-suppression and
pending-job namespace inputs used by `0q2.A02` and `HxT.A00`, respectively.

`07t.<init>` calls both helpers with Boolean builder `v14`, then calls
`buildKeepingLast` at `9f2c` and stores its map in `07t.A00` at `9f3e..9f42`.
`07t.A15` returns this immutable map. Physical write inspection finds one
`07t.A00` writer: the constructor, `classes.dex` offset `0x3fc312`, opcode `5b`.
Reproduce with `apk-field-refs --class 'LX/07t;' --field A00 --type
'Lcom/google/common/collect/ImmutableMap;' --writes-only` on the pinned APK.
This is physical-write scope, not proof about all runtime configuration sources.

**Negative result:** the two literal IDs are absent from the constructor itself;
they are in its helper methods. Searching only the constructor would incorrectly
report missing defaults. The private 26-method class scan found one literal
occurrence of each ID; computed duplicate keys were not exhaustively proved
absent. The snapshot preserves the full selected helpers and constructor rather
than interpreting a string/literal search as complete map authority.

**Confirmed static:** `00D.A10(I)Z` supplies mode `00F.A03` and a null nullable
fallback to `00D.A0E`. That resolver constructs `1Y1` with selector 2, retrieves
`09T`/`09U` providers and calls `00D.A07`; the result is cast to Boolean.
In `1Y1.invoke`, selector 2 takes the default switch branch and calls `00D.A0D`.
The float and other explicit branches are not this Boolean resolver.

`00F.<clinit>` constructs `A03` with a false Boolean, which the constructor
stores in `A01`. `00D.A0h` selects concurrent map `A0N` when that mode Boolean
is false and `A05` when true. Thus the false mode flag is a cache selection,
not a constant false feature value.

## Preferences, cache and MobileConfig boundary

**Confirmed static:** `00D.A0D` returns a cached Boolean if present, otherwise
locks and rechecks, invokes exposure helper `A0B` with `id + "_expo_key"`, and
looks up the virtual `A15` default map. A map value supersedes the nullable
caller fallback. If both are missing, it throws `IllegalArgumentException`
with `Unknown BooleanField: ` and the ID; unknown fields do not universally
become false. It calls SharedPreferences `getBoolean(String.valueOf(id),
selected_default)`. False-mode handling invokes `A0u` and `A0F`; true-mode
handling saves to `A05`. Their complete side effects remain untraced here.

**Confirmed static:** `00D.A07` can consult MobileConfig rather than simply
execute the fallback. Its full body includes provider/context/readiness gates,
`09U.invoke` at `012c..0132`, an alternative `09T.invoke` at `0192..0198`, and
fallback callback `09m.invoke` at `0224..022a`; some paths cache provider results.
A later branch can still invoke `09T` after obtaining a fallback.

**Unknown at this boundary:** native MobileConfig results, readiness atomic
writers, exposure effects, cache invalidation and full precedence during state
transitions. The enum meaning of the type argument `02S.A0N` is not established
by this trace. False APK registrations therefore cannot supply effective values
for the Rust token refresh path. Mapping authority and that runtime remain
unported; no defaults or returned properties are applied to live behavior.

## Offline inspection

**Confirmed local:** existing `abprops-query --show` now includes
`token_counterpart_value` / `token_counterpart_value_invalid` and
`token_pending_namespace_value` / `token_pending_namespace_value_invalid`.
These expose only selected values in an already saved correlated server reply;
show does not queue or send a query. No HTTP implementation is claimed.

[Existing APK parser evidence](feature-configuration.md) (`IWy.C8f`,
`0Ci.A00`) supplies last-duplicate retention and signed Java integer conversion:
zero is false, any other valid i32 is true. Invalid last values yield null with
an invalid flag, without fallback to an earlier row or the APK default. Missing
rows and non-applicable full responses without a nonempty hash yield null with
no invalid flag. A delta response can be applicable without that hash. Old
serialized inspections deserialize these new fields as unknown. Hashes, AB keys
and raw property values are not exposed.

**Verification:** the token-value regression failed before implementation
(expected true, got null), then passed. Coverage includes full/delta replies,
duplicate values, textual Boolean rejection, i32 overflow, missing rows,
hashless full replies, old JSON, durable reopen/redaction and preserved peer
pins/sessions/action IDs with no replay. All 816 workspace/all-feature tests,
strict all-target Clippy, warning-free Rustdoc, CLI reference regeneration and
API symbol checks passed. Snapshot exact regeneration passed; self-diff reports all twelve topics and
four indexed artifacts unchanged. Formatting and repository hygiene passed
(1,767 local links; 67 checksum-verified bundles). No real account state or
live request was changed.

## Boolean provider bindings (2026-10-09 continuation)

**Confirmed static:** `00D.<init>` constructs `1Y4(4)` at `0100..0110`
for `A0R:09T`, and `1Y5(0)` at `014c..015a` for `A0W:09U`. Each provider
constructor saves its selector to `$t`. Physical write inspection finds only
these constructor writes (`classes.dex` offsets `0x3bf198` / `0x3bf1e2`, opcode
`5b`), not a complete claim about reflection or native mutation.

`1Y4.invoke` selector 4 takes the default Boolean branch. It maps the property
ID with `09M.B2y(I)J` at `0024..002a`. A zero mapping returns an `Hln` with
false value, `INVALID_SPECIFIER_MISSING_IN_WA_LOOKUP_MAP` source, null string
and zero long. A nonzero mapping calls `MobileConfigUnsafeContext.AVn(0E8,J)`
at `00c8..00ce`, boxes its Boolean and delegates to `0E6.A00` for metadata.
This false missing-mapping result is distinct from the APK default-map lookup.

`1Y5.invoke` selector 0 takes the branch at `01a6`, delegates to `0E6.A04`
and boxes the Boolean. `0E6.A04` also maps through `09M.B2y`; zero returns
false immediately (`0014..0018`). Nonzero invokes `AVn` at `003c..0042`.
A positive final integer argument requests `N6u.A00` with type `02S.A0N`;
`0E6.A01` runs on both normal return and the catch-all rethrow path. The
meaning of this bookkeeping is not established here.

**Unknown:** concrete `09M` implementation and specifiers for 15491/14303,
actual native `AVn` results, readiness/provider precedence, and complete cache
adoption. `0E6.A00` was inspected privately: it assembles source and optional
metadata, but its optional metadata path contains renderer-unknown opcode
`0xc5`; complete decoding and metadata provenance are not claimed. No new
Rust runtime behavior or account request follows from these static branches.

The [six-topic provider recipe](apk/recipes/2.26.37.71-token-feature-providers.json)
and [snapshot](apk/2.26.37.71-token-feature-providers/targets.json) preserve the
bindings, both selector constructors, both dispatchers and direct resolver,
including signatures/register flow/catches: 11 files / 60,025 bytes. Exact
regeneration and self-diff passed (six topics/four artifacts unchanged); CLI
reference and API-symbol checks, diff whitespace and repository hygiene passed
(1,772 links; 68 checksum-verified bundles). The checked snapshot is the
reproducible inspection surface; this continuation adds no command or flag.
