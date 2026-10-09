# Privacy-token canonicalization and counterpart gates

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Six-topic recipe](apk/recipes/2.26.37.71-token-canonicalization.json),
[selected source](apk/2.26.37.71-token-canonicalization/targets.json).
Offsets are method-relative bytes. This narrows the mapping prerequisites left
open by [identity token refresh](identity-token-refresh.md), without enabling
refresh requests or changing saved identities.

## Normalization and counterpart selection are separate

**Confirmed:** 0q2.A0L calls 0DE.A0e; that predicate is true only for a nonnull
JID whose getType() is zero. Only that branch casts PhoneUserJid and asks
0Xw.A0C for an account-user mapping. A nonnull mapping replaces the supplied
user; null mapping or another subtype returns the original user. A0L itself
does not read property 15491 or 14303.

The existing [forward mapping trace](peer-identity.md#canonical-key-mapping-and-row-selection-in-rust-2026-10-08)
establishes readiness, own-account handling, stored row ranking and cache lookup
inside A0C. The [counterpart subtype trace](receipt-canonicalizer.md#counterpart-direction-is-subtype-specific)
establishes A0I's null/type exclusions and forward/reverse dispatch. Those
methods were rechecked, not newly reconstructed here. Current mapping authority,
readiness and complete typed row materialization remain **unknown**.

**Confirmed:** 0q2.A02 receives original and normalized users. Property
**0x3c83 / 15491** true returns null before equality or counterpart lookup.
False compares those two users: if different, returns the original directly;
if equal, calls 0Xw.A0I(original). Its branch at 0020 targets 0016 when
areEqual is false, retaining v3 (the original argument).

| Property 15491 | original versus normalized | counterpart result |
| --- | --- | --- |
| true | either | null; normalization remains independent |
| false | different | original user; no A0I call |
| false | equal | A0I(original), possibly null |

Thus mapped PN input can select canonical account-user plus original PN without
reverse lookup. Unchanged account-user input can require the reverse mapper.
**Refuted shortcut:** suppressing counterpart lookup is not permission to keep
an original PN as canonical when A0L found its account-user mapping. Nor does
an authenticated Rust PN/LID alias establish the APK's mapping-store selection.
The effective property's value is still **unknown**, not an inferred default.

## Read and successful-write consumers

**Confirmed:** the all-DEX physical invocation scan finds four A02 callers:

- 0q2.A0M at classes.dex **0x6d8a0a**.
- 0q2.A0N at **0x6d8c0e**.
- 0q2.A0O at **0x6d8ca2**.
- 0q2.A0V at **0x6d9840**.

Reproduce with `xtask apk-callers APK --class 'LX/0q2;' --method A02`.
These are physical calls, not a transitive closure or all possible mapping uses.
No direct A02 invocation was found in A0P/A0W; this does not prove their complete
transitive independence. Their own A0L normalization remains separately traced.

A0M normalizes, checks its incoming-token cache, then on miss reads canonical
alone if A02 returns null. Otherwise its SQL reads both names from
wa_trusted_contacts and selects `ORDER BY incoming_tc_token_timestamp DESC LIMIT 1`.
It materializes token bytes/time through 0q4.A06 and caches a nonnull result under
the normalized user. Equal timestamps have no secondary SQL ordering here;
selection, cache invalidation and full row-decoder constraints remain **unknown**.
This incoming-byte query is distinct from outbound issue/send state.

A0N normalizes and uses a nonnull outbound-cache hit directly. On miss it reads
canonical alone through 0q4.A0J or both names through A0M/HOF.A00 aggregation,
then returns. It contains no new cache put or expiry check in this body. The
already retained [A0O reader](apk/2.26.37.71-identity-token-refresh/token-read.txt)
uses containsKey and expiry handling; do not treat the two getters as identical.
A cached explicit null and a cache miss can differ between their tests.

The [successful writer](apk/2.26.37.71-identity-token-refresh/token-success.txt)
0q2.A0V normalizes and commits its outbound timestamp operation first. On its
successful outcome it evicts the canonical cache and calls A02 at 0194..019a;
nonnull selects the counterpart row for deletion on a later database handle.
The same gate therefore affects aggregation and this post-write cleanup, not
just request addressing. Full cache lifecycle and effective property changes
remain untraced. No incoming token is promoted into outbound history.

## Pending-job namespace has another gate

**Confirmed:** HxT.A00 reads **0x37df / 14303**. True uses 0q2.A0L; false checks
0DE.A0Z and tries reverse 0Xw.A0H for admitted account-user types, returning the
original if that fails, or for other types. This property is separate from
15491. It can choose a different pending-set/job user from the manager's later
A0L result. The retained source names its cast LidUserJid; the broader A0Z
subtype domain must not be equated with every Rust LID encoding.

The [refresh lifecycle](identity-token-refresh.md) separately traces membership,
rehydration, threshold and IQ construction. **Unknown/unported:** effective
14303/15491 and expiry properties, adjusted clock, complete mapping authority,
property/cache transition behavior and durable outbound scheduling/refresh.
This chunk closes the helper's static branch conditions, not those runtime inputs.
No fabricated flag or unconditional PN/LID merge is introduced.

## Repository/runtime boundary

Existing Rust authenticated replacement archives and device refresh remain
intact. Incoming notification storage still keeps received token bytes separate
from outbound state. No Rust runtime, account, key, request ID, reservation,
server wait, database reset or live operation changed. The selected methods and
existing offline APK CLI reproduce this trace; no HTTP route or live capability
is added. This is progress toward the refresh port, not completion of it.

Validation: exact re-export verifies 11 files / 45,470 bytes; checksum-validated
self-diff reports six topics and four metadata artifacts unchanged. CLI-reference
regeneration, API-symbol checks, formatting and whitespace passed. This static
chunk retains the preceding 814-test baseline; no fresh workspace or live test
is claimed.
