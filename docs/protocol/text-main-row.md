# Main message row before the type-specific insert

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-main-row.json) and
[complete methods](apk/2.26.37.71-text-main-row/targets.json) continue
[store insertion](text-store-insertion.md). Offsets are method-relative bytes.

## Callback continuity and two different state fields

AiX's constructor stores argument 1 in A00, argument 2 in A01 and argument 3
in $t. Its invoke switch selects **0494** for discriminator **10**. That block
casts A00 to 0VB and A01 to the original message, resolves 0VB.A02 as 0zc,
and calls **0zc.A0A(message)** at 04b4..04ba. It then returns Kotlin Unit through
0028. The caller and registry hook order therefore establish that main-row
insertion precedes 1hj.BJK for this supported 0V9.A06 path.

0zc.A0A calls **178.A0h(1)** at 0012 and 0416. The lowercase-h method sets
**A0f**, with a setStorageType diagnostic when a defined value differs. It
passes the compatibility condition into 00K.A0D before assignment; the full
assertion policy is not traced here. It is distinct from uppercase **A0H**, the
native status setter. Consequently these calls do not establish status 1.
The storage type is already set before opening the database, so a later failure
is not proof that the in-memory message remained unchanged.

0zc.A03 supplies ContentValues for the common row. It writes SQL **status**
from the virtual **B3b()** getter at 0048..0064, not from the callback selector,
insertion reason or storage-type argument. The retained base getter returns
A08 ([status evidence](message-status.md)); subtype overrides and earlier
preprocessing still determine the actual value. No fresh status assignment
appears in the selected main-row callback/writer methods.

Other directly attributed A03 columns include timestamp/A0F,
receipt_server_timestamp/A0E, message_type/A0h, origin/A05 and sort_id/A0k.
origination_flags uses A08() (a different, long-returning method), text_data
uses A0g(), and lookup_tables uses Cjb.A00; complete helper/flag semantics
remain **unknown**. These column mappings are local database evidence, not wire
field numbers or server requirements.

## Main-row write and failure boundaries

A0A resolves JID/chat rows, applies PN/LID creation guards using 0Yo.A0H and
JID predicates, and emits IOException diagnostics for the selected forbidden
creation branches. Full effective namespace setting and all helper effects
remain **unknown**; the diagnostic does not authorize fabricated aliases.
It adds chat_row_id, from_me, key_id, sender_jid_row_id, broadcast and
recipient_count (message.A06 unchanged) to the common ContentValues.

The selected row insertion at 0354..035a calls 0IY.A06 for table **message**
with INSERT_MESSAGE_MAIN_SQL or INSERT_MESSAGE_MAIN_WITH_ROW_ID_SQL. A negative
returned ID branches to an IOException; an existing positive requested ID
that differs from the returned ID also throws. Otherwise the returned ID is
assigned to message.A0j at 038c. This is a main message row, **not receipt_user**.
The selected sort-id update has its own caught SQLException diagnostic and
continues; it must not be described as guaranteeing successful sort persistence.
Success marking and transaction/store closes are retained, as are rethrow and
suppressed close failures. Actual 0IY SQL definitions and complete nested
transaction guarantees remain **unknown**. No network send or ACK is established.

## Registry resolution retained, concrete type-zero owner still unknown

1gj's constructor initializes range arguments 0 and 0x95, stores the API
class provider, and uses 1wj selectors **28/29/30** for fallback, provider array
and AtomicReferenceArray. Selector 30 allocates **0x96** cache entries; selector
29 obtains the requested class and calls **1gi.A00** for its provider array.
Selector 28 selects a fallback using class comparisons and provider IDs; those
fallback classes are not inferred to be normal text inserters.

1gj.A02 checks range and a nonnull provider-array slot. A00 first returns a
cached implementation, otherwise gets the slot's provider or fallback, attempts
atomic publication through 0EE.A00 and returns the cache entry. This preserves
the registry lookup boundary without guessing the type-zero provider. The
1gi provider map, 1hj concrete BJK, before/after hook membership and 0EE atomic
helper semantics remain **unknown/unported**.

Full ordinary send progression, user-row admission and primary identity cleanup
remain unfinished. No Rust runtime, account, message ID or live operation changed.
The existing offline CLI export/diff tools reproduce these source topics; no
HTTP or live capability was added.

Validation: seven topics / twelve files / 166,001 bytes reproduce exactly from
the pinned APK. Cached self-diff verifies seven unchanged topics and four
metadata artifacts. CLI reference regeneration, API/library-symbol validation,
formatting, whitespace and repository hygiene pass (2,011 local links; 93 bundles
/ 18,935,752 bytes). No fresh workspace/runtime suite or live test is claimed.

Follow-up: [text type inserter](text-type-inserter.md) identifies 1hl.BJK and
confirms D7J membership in the before set; other hook effects remain unknown.
