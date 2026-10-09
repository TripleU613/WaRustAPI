# Ordinary text inserter and registered recipient-count hook

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-type-inserter.json),
[complete source](apk/2.26.37.71-text-type-inserter/targets.json) and
[provider arrays / physical callers / negative results](text-insertion-provider-evidence.json)
continue [main-row insertion](text-main-row.md). Offsets are method-relative
bytes; branch displacements are signed DEX code units multiplied by two.
These are local insertion findings, not proof of network sending or user receipts.

## Type-zero provider continuity

1hh's constructor selects 1wj discriminator **31** for A0D. The previously
retained [1wj body](apk/2.26.37.71-text-main-row/registry-lazy-producers.txt)
maps 31 to 04cc, requests the **1go** API class through 09u, and calls 1gf.A00.
The synchronized factory caches 1gj by API-class provider. 1gi.A00 then caches
provider arrays by the resolved Class. Both branches create 150 slots and call
**1qR.A0k then LZq.A0I**: the computeIfAbsent branch uses 1wo selector 15,
which falls through the switch default to 000a; the older branch builds directly.
The selected runtime branch of 076.A00 remains **unknown**, but both populate
through the same two methods in that order.

For **1go**, 1qR.A0k at 09f4..0a06 passes **0x14c1** into A00. That helper
constructs 05E and assigns it to slot **0**. LZq's 1go block at 0a80..0f08
populates other slots and returns; it does not overwrite zero. Its 59 selected
indices are all nonzero. This establishes the normal text-type-zero provider,
not just a guessed class whose interface name resembles insertion.

05E.<init>(int) sets A03 to that ID and A02=true. get resolves its cached value
through **00S.A03 -> A04 -> 1UE.A00**. The root dispatch uses (ID >> 14) & 0x7f;
0x14c1 selects root case 0 -> A01. A01 masks 0x3fff and selects A0C because
0x14b8 < 0x14c1 <= 0x15cf. A0C case **5313** lands at **0070** and calls
0qn.A0S, whose body constructs **1hl**. This closes the ID-to-implementation
chain. 1YP.A0C is another constructor wrapper retained during tracing; it is
not needed by the direct slot-zero helper. 00C.A02 is an alternate global entry,
not the selected 05E.get path. A 1go body query returned no executable methods;
that result does not mean the class is absent.

## Actual BJK work and optional text extension

1hl.BJK casts the message to 1he and calls, in order:

1. **1tL.A00(message)** through its A00 dependency (constructor provider 0x101df).
2. **0ze.A03(message)** validation.
3. **0ze.A02(store,message,false)** through A01 (provider 0x14bf).

The runtime casts provide concrete dependency types; complete DI paths for
those two dependency IDs are not claimed here. CgJ uses the same helpers with
true rather than false; its callers and complete update workflow remain unknown.
APp/ABz load text information, with complete bodies retained but not analyzed
as insertion. No direct 178.A0H setter occurs in the selected BJK/helper bodies.

1tL.A00 clears **1he.A02** (a subtype field, not base message A08 status),
then conditionally derives a text offset from span bounds and following
whitespace. The branch includes nonempty text, property **0x49bc**, metadata
predicates and a nonnull byte-array prerequisite. Full span/parser/property
semantics are **unknown**; this is not a delivered/read status computation.

0ze.A03 checks positive A0j row ID and storage getter A0e()==1, passing
conditions into 00K.A0E diagnostics. Complete assertion policy and A0e override
semantics remain unknown. A02 stores optional extended text fields in table
**message_text** when its metadata predicate qualifies. With false input and
no qualifying metadata, its selected branch skips extension-row insertion.
That is not loss of ordinary text: the main message row's text_data was already
written by A03 in the previous trace. The complete predicate includes metadata
strings, style data, several booleans and positive text offset; defaults of all
virtual predicates are not asserted here.

The extension path uses INSERT_MESSAGE_TEXT_SQL. A caught SQLiteConstraintException
attempts UPDATE_MESSAGE_TEXT_SQL for the message row ID, requiring affected
count 1 or rethrowing the original exception. True/update input additionally
permits deletion of the extension row when metadata is absent. Thumbnail,
annotation and reporting helpers remain **unknown** beyond their retained call
sites. Neither message nor message_text membership proves receipt_user existence.

## Before/after hook membership is now attributable

The 1hh constructor supplies A03/before from **00S.A05(0x1cb4)** and A02/after
from **00S.A05(0x1c92)**. A05 calls the global provider dispatcher and casts
its result to Set. 1UE.A0K selects 07A.A3C for the former and A3B for the latter;
both land at 088a and construct 07B with the selected ID array. The static
initializer supplies **13 before** and **54 after** entries. Full integer lists
and derivation are retained in the linked provider evidence.

Before array **A3C index 8 = 0x3c5 (965)**. 07B.iterator constructs 08K;
next reads the selected ID through its 055 loader, and 07C.get passes that ID
to 1UE.A00. Case 0 / A01 / A04 resolves **965 -> 0234 -> 0Fv.A1Q -> D7J**.
The exact constructor/factory caller scans each found one physical invoke.
This establishes D7J's registration in the before set for this build, replacing
the earlier unknown. It is not based only on D7J implementing 1hq.

Together with [1hh.A01 hook order](text-store-insertion.md), D7J.CHJ receives
the original message and **null** before the main-row callback. Its previously
traced [recipient-count branches](receipt-count-preprocessing.md) therefore
run at this insertion boundary; its nonnull-result exception branch is not
selected here. Runtime provider injection can fail and abort iteration;
08K retains circular-dependency, invalid-binding and injection diagnostics.
Full semantics/membership of the other 66 entries remain **unknown/unported**.
No claim is made that every message follows this normal insertion path or that
all hooks succeed. No per-user receipt SQL or network ACK is inferred from it.

## Remaining work

**Unknown/unported:** all other before/after hooks, full normal text status
progression after insertion, settings/namespace admission, native user-row
membership and primary identity cleanup. The evidence is reproducible through
existing offline CLI tools and scoped cached diffs; no new HTTP/live capability,
Rust runtime change, account operation or message ID was introduced. Future APK
builds require separately anchored recipes; file equality is not semantic parity.
Validation: 29 topics / 34 files / 957,596 bytes reproduce exactly from the
pinned APK. Cached self-diff verifies 29 unchanged topics and four metadata
artifacts. CLI reference regeneration, API/library-symbol validation, formatting,
whitespace and repository hygiene pass (2,023 local links; 94 bundles /
19,893,348 bytes). No fresh workspace/runtime suite or live test is claimed.

Follow-up: [selected after hooks](text-after-hooks.md) maps 19 entries and
records direct source exclusions; transitive helper effects remain unknown.
