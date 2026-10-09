# Registered storage, poll and indexing after-hooks

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-storage-after-hooks.json) and
[complete methods](apk/2.26.37.71-text-storage-after-hooks/targets.json)
continue [metadata hook coverage](text-metadata-after-hooks.md). Offsets below
are method-relative bytes. Source evidence only; no Rust/account/live change.

## Seven registry mappings

The [retained A3B initializer](apk/2.26.37.71-text-type-inserter/hook-provider-id-arrays.txt)
supplies these array indices. Root bucket zero selects 1UE.A01; its actual
interval dispatch, exact numeric case, factory and CHJ body establish continuity.

| Provider / array index | Partition / case / offset | Factory -> hook |
| --- | --- | --- |
| 0x9f3 / 12 | A0Q / 2547 / 0142 | 0VW.A09 -> D7h |
| 0x1344 / 14 | A0A / 4932 / 067e | 0q8.A0l -> D7d |
| 0x1345 / 15 | A0A / 4933 / 0688 | 0q8.A0p -> D7W |
| 0x1346 / 16 | A0A / 4934 / 0692 | 0q8.A0q -> 8CF |
| 0x1347 / 17 | A0A / 4935 / 069c | 0q8.A0u -> 8CG |
| 0x9cd / 21 | A0P / 2509 / 078e | 0VN.A0K -> D7Y |
| 0xa68 / 23 | A0Q / 2664 / 03f4 | 0WW.A04 -> D7f |

The relevant A01 intervals are **0x8b8 < ID <= 0x9cf -> A0P**,
**0x9cf < ID <= 0xae7 -> A0Q**, **0x1289 < ID <= 0x13a0 -> A0A**.
**Refuted candidate:** A0F is the low interval 0x45b < ID <= 0x573; its
selected dump has no case 2547 and cannot resolve 0x9f3. That exploratory
partition is retained separately, not adopted as a factory mapping.

Current immediate after-hook coverage is **36 of 54**, leaving **18 unmapped**.
This counts registered CHJ boundaries, not completed transitive porting.
Normal insertion supplies null to the second CHJ argument; conditional nonnull
rejections below do not prohibit the normal insertion path.

## Ephemeral and poll history paths

D7h.CHJ calls **15e.A0E** iff **CB5.A00 false and B35.A0B true**. The
writer calls untraced 15e.A01 for values, adds message_row_id and writes
**message_ephemeral** via 0IY.A05, discarding its returned ID. The selected
writer has no positive-row-ID validation. Full values/predicate meanings are
**unknown**. Normal completion also queues **DNy selector 41 (0x29)** through
08S.execute; the constructor/worker and its asynchronous effects remain unknown.
A direct writer return is not proof of receipt membership or expiry enforcement.

D7f.CHJ requires subtype **17A**, nonempty A0B pending-vote list and nonnull
B1V parent sender; missing sender logs and returns. Nonempty A0C delegates to
CS5.A01. If A0p() is null, it obtains a list from 0Y5.A0D(message, **0x43**)
and attaches it with A0t. For each BnD vote, missing option hashes or a null
Cjh.A01 mapping are diagnosed and skipped. Qualifying votes receive the poll
A0j in **1iZ.A02**, translated option lists via A0y/A01, then are passed to
**0ZG.A04(parent sender,vote,poll,02S.A01)**; its integer result is logged.
These are poll-history mutations, not base 178.A08 status assignments.
CS5/0Y5/Cjh/0ZG effects and full enum/option validation remain **unknown**.
Nothing here establishes normal text admission or native receipt-user rows.

## Media and thumbnail writers

D7d requires nonnull CmE/D6l. It obtains 0VS.A0F(message) and passes that
long plus message A0j to **6d3.A02**. The latter opens a transaction, iterates
D6l.A00 map values and processes **73W.A00 == -1** entries only. It calls
untraced **6d7.A00** for a media row ID, then writes **message_media_map**
(message_row_id, chat_row_id, media_row_id) via A09 numeric argument **5**.
The mapping SQL return is discarded. SQLiteConstraintException is logged and
swallowed after close handling; other exceptions follow retained catch tables.
0VS ID meaning, nested media writer behavior and full transaction semantics
remain **unknown**; a long helper result is not inferred to be receipt authority.

D7W requires **BnT**, casts to **Okm**, then calls **156.A00**. A null/empty
AwJ list returns. Otherwise each CLU entry writes **media_processed_video**:
message_row_id from nullable Ac5/F6T, direct_path, file_size, height, width,
bitrate, quality and comma-joined capabilities. A05 return IDs are discarded;
normal completion marks/closes the transaction. Nullable row metadata is not
an explicit positive-ID guard. Complete interface/producers remain unknown.

8CF's first branch requires **A0b(0x800)**, nonnull Cbl/8Bv and null
8Bv.A08. Its second requires **A0a(0x8000)**, a 1he Cbv/D6M nested 8Bv and
null nested A08. Both pass metadata and message A0j to **16V.A01**. A0b reads
A0A origination flags; A0a reads A0n lookup flags, as previously traced. These
are distinct masks and neither is superclass A08 status. The selected body
tracks whether either branch ran for its nonnull-result rejection.

16V.A01 branches on **8Bv.A0D == 1e3.A04**. The unequal branch writes
**mms_thumbnail_metadata** through A09 numeric argument **5**; the equal branch
writes **mms_metadata**. Both include message_row_id, direct_path, media_key,
media_key_timestamp, enc_thumb_hash, thumb_hash, thumb_width, thumb_height,
transferred, handle, micro_thumbnail and clock-derived insert_timestamp.
The equal branch additionally includes type. If **8Bv.A04 == -1**, A05's returned
ID is stored in that field; otherwise it updates **_id = ?**, discarding affected
count. The retained constraint catch paths log and **rethrow**, unlike 6d3's
swallowed constraint. Full enum meanings, producer validation and helper/trigger
semantics are **unknown**. APK constants in source are not private account keys.

8CG requires nonnull A0C()/1j7, **A04() true** and nonnull A05() bytes. It
calls **0qk.A06(message,bytes)** then **1j7.A01()**. The latter clears only
boolean **1j7.A01** under a monitor; it does not erase bytes or consume a
view-once message. A delegate exception prevents this subsequent clear.

0qk.A06 has a media-hash path for 72o/1iB with a nonempty AoD string, cached
thumbnail tests and supplied-byte decodability checks. Depending on guards it
returns, calls untraced **0qk.A00(bytes,row ID)**, or writes
**media_hash_thumbnail**. The latter updates thumbnail by **media_hash = ?**
when a cached value exists; affected count zero falls through to insert with
media_hash plus thumbnail. A cached-value absence also inserts. Insert return
ID is discarded. Decode/selection/string-normalization helpers remain unknown;
this boundary is not a new receipt, media-authentication or view-once rule.

## Search indexing boundary

D7Y skips subtype **1ii**; otherwise it calls **0VP.A0Q(message)** and then
A0T() for a conditional nonnull-result check. A0Q obtains an A0A() long and
calls **A0E(message,empty map,long,false)**, discarding the result.

0VP.A0T reads stored **fts_ready** with default zero and returns whether its
remainder modulo two is nonzero. Effective saved value is **unknown**.
Because the selected caller passes false, A0E returns a local **7bL(-5)** when
not ready; its A0U predicate can return **-6**. These are local search results,
not server refusal reasons. Other branches compute content/JID/namespace values,
may enqueue **AsyncMessageTokenizationJob**, or write **message_ftsv2** columns
content, fts_jid, fts_namespace and docid. A constraint catch updates by docid,
discarding the affected count. The source includes namespace/PN-LID helpers;
this specialized search path does not authorize Rust peer identity aliasing.
Complete tokenization, predicates and result/namespace semantics remain unknown.

## Receipt authority still missing

**Confirmed bounded negative:** the seven CHJ bodies and nine selected delegates
have no direct **178.A0H** invocation or literal **receipt_user** SQL. Other
classes' methods named A0H are present in indexing and are not that setter.
Immediate table writes above do not establish native user-row membership.
Untraced asynchronous/predicate/serialization/SQL-helper/trigger effects are
**unknown**, not excluded. Immutable targets, counts or metadata cannot substitute
for actual receipt-user row existence needed by primary identity cleanup.

**Unknown/unported:** remaining 18 after entries, other before hooks, full normal
send progression, effective receipt policy/namespace admission, native user-row
lifecycle and primary cleanup. Existing offline APK export/diff CLI commands
reproduce this trace; no new CLI flag, HTTP route or live capability was added.
No saved account was opened and no network request or changed request ID occurred.

Validation: 27 source topics / 32 files / 284,700 bytes reproduce exactly
from the pinned APK. Cached self-diff verifies 27 unchanged topics and four
metadata artifacts; no cross-release semantic comparison is claimed. Exact
provider cases and direct exclusions across seven hooks/nine delegates were
checked. CLI-reference regeneration, API/library-symbol checks, formatting,
whitespace and hygiene passed (2,073 local links; 100 bundles / 21,007,146 bytes).
No fresh workspace/runtime suite or live account test is claimed.

Follow-up: [association/sharing hooks](text-association-after-hooks.md) adds
five entries and resolves CS5's poll-name history writer. Current coverage is
41/54, leaving 13. AVI's native setter targets a newly created system message;
the previous bundle's bounded direct exclusions remain scoped to its methods.
