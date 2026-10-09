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

## Property-to-MobileConfig mapping (2026-10-09)

**Confirmed static:** `07t.<init>` obtains its `09M` argument with
`00C.A02(233)` (`0080..0090` in the
[default constructor](apk/2.26.37.71-token-feature-inputs/default-map-constructor.txt)).
`00C.A02` calls `00C.A03`; the latter caches by boxed provider ID and, on a
cache miss, calls `1UE.A00(id, context, application)`. Its family selector
`(id >> 14) & 0x7f` sends 233 to `1UE.A01`. That method masks `0x3fff`, selects
233 (`0xbf6`), constructs `09N` and returns it (`0bf6..0c02`). Thus `09N` is
the concrete constructor binding, not an inference from interface names.
`00D.<init>` stores that supplied argument in `A0L`; physical write inspection
finds one constructor write, `classes.dex` offset `0x3bf0ea`, opcode `5b`.

`09M.B2y(I)J` itself has no implementation body. `09N.B2y` gets static
`0E9.A00:[J`, lazily calls `0E9.A0I` if null, and reads the long at the property
ID. Negative or out-of-bounds IDs return zero. `0E9.A0I` locks/rechecks, allocates
`0x9125` (37,157) longs, writes constants and calls helpers `A0A..A0H` then
`A00..A09`, before publishing the array at `a79c..a7a0`. The physical static
field writer is this initializer only (`classes.dex` `0x5094a0`, opcode `69`).
Field publication is not a proof that all array elements are immutable.

**Confirmed static initial entries:** both IDs are in helper `0E9.A0F`:

| Property | Write offsets | Long specifier |
| --- | --- | --- |
| 14303 / `0x37df` | `072c..073e` | `0x811af9000014f5` / 36339928744858869 |
| 15491 / `0x3c83` | `35b8..35ca` | `0x811d83000016e2` / 36342720473601762 |

Each sequence loads a literal index into `v2`, a literal wide value into `v0`,
then writes `aput-wide v0, v3, v2`. The private complete `0E9` dump contains
20 methods; the eighteen helper bodies contain only constant loads, wide array
writes and return. The class scan finds one literal write index for each of
these IDs. Neither is an out-of-range or missing-zero entry in the generated
initial table. Those nonzero keys reach `AVn` when the already documented
MobileConfig branch is selected; that does not prove the branch is active.

**Negative/limited results:** searching only `0E9.A0I` did not find these ID
literals because its helper `A0F` holds them. A direct DEX class-definition
interface-list scan found `09N` implementing `09M`; the constructor binding
trace above supplies the stronger evidence for this actual use. The scan is
not a transitive implementation or dynamic-loading closure. `09M.<clinit>`
references `PfA`, whose inspected bodies only initialize the singleton; they
do not implement `B2y`. No native values were recovered. Physical field reads
also exist in `099.A0C` and `0Dt.A00/A03`; their effects on table contents have
not been traced. Cache replacement/reflection/native mutation is not excluded.

The [eight-topic mapping recipe](apk/recipes/2.26.37.71-token-feature-mapping.json)
and [snapshot](apk/2.26.37.71-token-feature-mapping/targets.json) preserve the
interface getter, concrete lookup, initializer, token entries and full provider
cache/family/binding methods, with signatures/register flow/catches. Reproduce
with `apk-research-bundle` and `--recipe`; compare releases with the existing
checksum-checked `apk-research-diff` after matching selectors. No new CLI flag,
HTTP route or runtime adoption was introduced.

**Unknown:** effective MobileConfig Boolean results, readiness, account-specific
cache/preferences and full transition precedence remain unresolved. Earlier
sections' mapping/binding unknowns are closed for these two initial APK entries
by this continuation, not for all runtime inputs. No real account state, identity,
keys, queued request or live behavior was changed.

**Mapping-chunk verification:** exact regeneration verified 13 files / 600,923
bytes; self-diff reports eight topics and four artifacts unchanged. CLI reference
regeneration, API-symbol integration, diff whitespace and repository hygiene
passed (1,778 links; 69 checksum-verified bundles). This source-only continuation
did not alter Rust; the preceding 816-test/Clippy/Rustdoc baseline is not claimed
as a new run for this chunk. Private bulk dumps stayed outside Git.

## Mapping consumers and shadow comparison (2026-10-09)

**Confirmed static:** the remaining physical `0E9.A00` readers inspected in
this chunk do not directly rewrite that table. `0Dt.A00` resolves an index
through `A01`, checks the generated long is nonzero, and returns
`(specifier >>> 48) & 0x3f`; missing/invalid entries return zero. This gives
both token specifiers the value 1, without independently naming a native type.
`0Dt.A03` iterates its separate `A07` map, resolves each index and nonzero
specifier, then inserts the associated object into separate `09E` cache via
`A0A(specifier, value)`. Its initial `09E.A07` and insertion affect that cache,
not `0E9.A00`. `09E.A0A` maintains sorted long keys and object values with
replacement, tombstone compaction and capacity growth. It accepts an object;
this trace alone does not establish its origin or native adoption.

`099.A0C` initializes a MobileConfig manager and iterates the generated long
array. For every nonzero entry it places the corresponding stable ID into a
new int array of size `0x4f73` (20,339), then calls
`setWaStableIdsInLookupMap`. It does not pass the long array to that call.
The size is an APK constant, not a count derived or checked by our runtime.
The constructor/callback/native manager and `0Dt.A01` index mapping remain
outside this chunk's complete semantic proof. `0E9.A0I` publicly returns the
array; this physical-reader inspection does not prove global immutability or
close all transitive callers of that getter.

**Confirmed static precedence boundary:** `00D.A0z` returns
`A0A && A0b && !A0Q.get()`. In the previously saved
[constructor](apk/2.26.37.71-token-feature-providers/provider-bindings.txt),
`A0A` requires the supplied `084` dependency and either preference 23048
(`0x5a08`, Boolean) or positive preference 27668 (`0x6c14`, integer).
`A0b` reads Boolean 24896 (`0x6140`); `A0Q` starts from Boolean 25403
(`0x633b`). These preference defaults are false/zero. `A0C` starts from
Boolean 22646 (`0x5876`, false default). Physical flag-write inspection finds
only constructor writes for `A0A` / `A0C`, respectively `classes.dex` offsets
`0x3bf29c` / `0x3bf2cc`, opcode `5c`; atomic updates and other readiness inputs
are not thereby closed.

**Confirmed static / negative result:** the method `00D.A0C` called during
provider evaluation is a shadow comparison/logger, not a value-adoption
routine. It compares the supplied fallback and `Hln.A02`, records source/exposure
metadata, updates `2tO` sets/counters and may send a `2VW` logging event. It
returns void. The caller `00D.A07` selects its return independently: the branch
at `0192` obtains an `Hln`, evaluates fallback for comparison and returns
`Hln.A02` at `01de`; the later branch obtains fallback at `0224`, optionally
calls the provider/comparison again, then returns that original fallback at
`028a`. Thus obtaining a native value does not itself prove adoption. Earlier
wording that called this generic result processing did not establish adoption.
Full readiness changes, cache precedence and logger side effects remain unknown.

**Confirmed local renderer correction:** opcode `0xc5` in `0Dt.A00+0058`
and `0E6.A00+0074` is now rendered `ushr-long/2addr`; `0xb8` in
`09E.A0A+00bc` is `shl-int/2addr`. The
[AOSP Dalvik table](https://source.android.com/docs/core/runtime/dalvik-bytecode)
defines their existing 12x format: destination in the low register nibble and
shift-distance register in the high nibble. The regression uses literal APK
instructions and a high-register case; before the fix all four mnemonics were
unknown. Width/register decoding was already correct and remains unchanged.
The previously unknown `0E6.A00` instruction now decodes unsigned extraction
of bits 32..47 before optional metadata lookup; this is a renderer limitation
resolved, not a change to APK code or proof of full metadata provenance.

The [seven-topic consumer recipe](apk/recipes/2.26.37.71-token-feature-consumers.json)
and [snapshot](apk/2.26.37.71-token-feature-consumers/targets.json) preserve these
methods and exception tables using the corrected renderer. Older immutable
snapshots are not regenerated: comparisons against them can show annotation
changes on identical APK bytes, so use the same renderer for release comparisons.
No new CLI command, gate adoption, real account change or live request occurred.

**Consumer-chunk verification:** 817 workspace/all-feature tests passed (681 API
unit, four codec, nine crash matrix, 52 CLI unit, one API-symbol, 70 xtask).
Strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
CLI reference regeneration, API-symbol integration, formatting and whitespace
checks passed. Exact regeneration verified 12 files / 71,120 bytes; self-diff
reports seven topics/four artifacts unchanged. Repository hygiene passed:
1,784 links and 70 checksum-verified research bundles. The added regression
failed before the opcode fix; private bulk dumps stayed outside Git.
