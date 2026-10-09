# Registered association, UI and sharing-setting after-hooks

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-association-after-hooks.json),
[complete methods](apk/2.26.37.71-text-association-after-hooks/targets.json)
and [previous storage hooks](text-storage-after-hooks.md) retain the source.
Offsets are method-relative bytes. No Rust/account/live behavior changes.

## Five registered entries

The [retained A3B initializer](apk/2.26.37.71-text-type-inserter/hook-provider-id-arrays.txt)
and root-zero A01 interval routing establish these exact cases. Both factory
and CHJ executable bodies are retained; this is not class-name inference.

| Provider / array index | Partition / decimal key / offset | Factory -> hook |
| --- | --- | --- |
| 0x188d / 18 | A0G / 6285 / 0564 | 0zw.A06 -> D89 |
| 0x16e6 / 19 | A0D / 5862 / 08e0 | 0wQ.A0G -> AVI |
| 0x19ff / 20 | A0H / 6655 / 0764 | 15p.A07 -> D7X |
| 0x1b45 / 22 | A0J / 6981 / 0034 | 1NN.A1T -> 8CK |
| 0x1826 / 49 | A0G / 6182 / 0190 | 0zk.A0W -> D88 |

A01 routes **0x15cf < ID <= 0x16e6 -> A0D**, **0x17fd < ID <= 0x1914
-> A0G**, **0x1914 < ID <= 0x1a2c -> A0H**, and **0x1b43 < ID <=
0x1c5a -> A0J**. The selected factory cases mask IDs with 0x3fff.
Immediate after-hook mapping is now **41 of 54**, leaving **13 unmapped**.
Mapping is not complete transitive porting or native admission proof.

## Parent/child association and UI persistence

D89.CHJ calls its private **A00(message,true)**. With nonnull 7ym/D6j,
A00 first calls **DGa.A02** then uses D6j.A00 to obtain a parent message.
A present parent calls **0zy.ChU(child,parent,association type)** because the
selected boolean is true. The retained false branch instead obtains a DdR
callback and calls Chc; it is not selected by this CHJ caller. Lookup/callback
and association enum semantics remain **unknown**.

DGa.A02 returns only for **parent ID == -1**. Otherwise it writes
**message_association** with child_message_row_id from child A0j,
parent_message_row_id from D6j.A00 and association_type from D6j.A01.value.
A09 numeric argument **5** returned ID is discarded. The selected guard is
not a positive-parent or positive-child validation: zero and other negative
IDs are not excluded by that guard. Shared SQL/helper/trigger behavior remains
unknown; association rows cannot substitute for receipt-user membership.

D88.CHJ delegates to private **A00(message,result)**. If Ayj.A1V passes,
it calls CmF.A02 and **0zl.A05(metadata,store,"message_ui_elements",A0j)**.
A05 returns for null metadata. Otherwise it constructs message_row_id,
element_type **2** and conditional element_content JSON containing content,
footer and button/native-flow attributes. JSONExceptions are diagnosed; the
store call still follows with the surviving ContentValues. Complete converter
and predicate meanings remain unknown; a serialization warning is not a server
reason or proof that the row was written with every optional value.

**0zl.A03** performs a dynamic-table update matching
**element_type = ? AND message_row_id = ?**, with selected type **2** and
message row ID. Affected count **zero** falls through to A05 insertion into
the same table; its returned ID is discarded. Nonzero count skips insertion.
This attributes the selected message_ui_elements SQL destination without
assuming all generic callers use that table or inventing receipt semantics.

## Poll-name history

D7X.CHJ requires subtype **17A** with nonnull/nonempty **A0C** list, then
calls **CS5.A01(poll,list)**. That helper additionally requires property
**0x7b68 true** and **poll A0j != -1**. This is not a positive-row-ID check.
For each COX entry it requires nonnull edit stanza string and A00() hash bytes,
then calls **CTR.A00(bytes,stanza ID,poll row ID)**. Effective property value,
COX generation and full input validation remain **unknown**.

CTR.A00 first queries existing bytes through untraced CTR.A02. If nonnull,
it compares with Arrays.equals: mismatches log; both equal and unequal existing
values return without replacing them. An absent result inserts
**poll_name_hash_history** columns poll_message_row_id, edit_stanza_id and
poll_name_hash via 0IY.A06, discarding its returned ID. Hash-history preservation
is not peer identity verification or per-user receipt history. The query and
shared SQL/canonicalization semantics remain unknown. This also narrows the
CS5 delegate previously left unknown in [D7f](text-storage-after-hooks.md).

## Limit-sharing system message: a real setter on a different object

AVI.CHJ's selected guard rejects outgoing/from-me input, Bmy subtype, failed
0DE.A0l chat predicate, set origination bit **1**, or a disabled 8l4.A03 gate.
Its diagnostic identifies receiver disabling; effective settings and exact
chat-predicate meanings are **unknown here**. Later processing requires AUu
metadata, positive AUu.A01 timestamp newer than the selected chat A0Y value,
nonnull chat and repeated extension values. Missing-chat and skipped branches
are retained; successful return does not mean these guards passed.

When the AUu.A03 boolean differs from the existing chat's selected bit value,
the body allocates a **new Bmy** with message type **0x70 (112)**, a new key
from 0od.A03 and local clock timestamp. It copies the selected AUu boolean,
integer and timestamp into Bmy.A00/A01/A02, then calls
**178.A0H(6) on this new Bmy** at **0216..021e**, before
**0zC.A0K(new Bmy)** store submission. It subsequently invokes 0Ze.A0R with
selected values and 0VS.A07(chat). The new-key producer, complete chat updater
and downstream submission effects are **unknown/unported**.

**Confirmed distinction:** this is a direct native status assignment to a
new limit-sharing system message, not an assignment to the input message.
It refutes a blanket exclusion of status setters in all after hooks, but does
not establish ordinary text-send progression or receipt admission. The branch
can submit another message while processing insertion; reentrancy/full durable
behavior is not assumed from the void call.

## Embedded interactive annotation children

8CK.CHJ requires subtype **1iB**, property **0x2532 true**, media metadata and
nonnull InteractiveAnnotation array. It selects annotations whose data is 8EC
with a nonnull embedded message, then calls 0C6.A1I with integer **32**. The
helper's complete truncation/selection semantics remain **unknown here**;
32 is a static argument, not a newly inferred server limit.

For each selected child it creates D6j association metadata using parent A0j
and prior association type (default 15T.A04), attaches it through 7ym.A04,
then calls **child.A0J(0x4000000)**. The retained **178.A0J** directly ORs
**A0A origination flags**, not A0n lookup flags or native A08 status.
It calls the previously traced **0V9.A06(child)** insertion wrapper, then
replaces annotation data with an 8EC containing child and resulting A0j.
Annotation type is derived through 7ym.A00 when metadata is present.

Finally it updates **message_media_interactive_annotation** by **_id = ?**,
setting child_message_row_id and conditional type. The affected count is
discarded; no direct insert fallback appears in this selected update branch.
The 0V9 wrapper has previously documented registry/before/callback/inserter/
after boundaries, not guaranteed child insertion success or complete recursive
hook behavior. Effective property, child guards/type semantics and all nested
callbacks remain unknown. It does not establish receipt-user membership.

## Remaining authority boundary

**Confirmed bounded negative:** no literal **receipt_user** SQL occurs in these
five CHJ bodies or eight selected delegates. AVI's native status setter exists
and is attributed above; the other selected direct bodies have no 178.A0H call.
Delegated store submission and child insertion explicitly reach other methods,
so absence in these bodies cannot exclude transitive receipt or status effects.

**Unknown/unported:** remaining 13 after entries, other before hooks, transitive
settings/callback/SQL-helper/trigger behavior, full normal send progression,
native receipt policy/namespace admission, actual user-row existence and primary
identity cleanup. No account was opened, no live request ran and no request ID
changed. Existing offline APK export/diff CLI commands expose this evidence;
no new CLI flag, HTTP route or live capability was introduced.

Validation: 22 source topics / 27 files / 267,536 bytes reproduce exactly
from the pinned APK. Cached self-diff verifies 22 unchanged topics and four
metadata artifacts; no cross-release semantic comparison is claimed. Exact
provider cases, AVI's scoped setter and bounded table exclusions were checked.
CLI-reference regeneration, API/library-symbol checks, formatting, whitespace
and hygiene passed (2,082 local links; 101 bundles / 21,274,682 bytes).
No fresh workspace/runtime suite or live account test is claimed.

Follow-up: [final hooks and ordered index](text-final-after-hooks.md) complete
54/54 immediate after mappings. This historical bundle remains unchanged;
transitive effects and the before set are still incomplete.
