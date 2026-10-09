# Common link/span after-hook and asynchronous update boundary

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-link-after-hook.json) and
[complete methods](apk/2.26.37.71-text-link-after-hook/targets.json) continue
[D7V after-hook delegation](text-after-hooks.md). Offsets are method-relative
bytes; branch displacements are signed DEX code units multiplied by two.
No Rust, account, request ID or live behavior changed.

## Guarded span processing, distinct from native receipt status

D7V.CHJ is an unconditional Ce8.A00 caller, but Ce8 itself is guarded. Its
first path requires message implementing **1jl**, 1gd.A14(message), and property
**0x4ef2**. The precise predicate and effective property value remain unknown.
It reads AZX()/CrW fields A0I, nested A09/CrH.A02 and A0J, creates SpannableString
values and invokes **1ZB.A02**. Nonempty returned span lists are passed to
1jD.A00 with integer **1**, **2** or **3**, respectively. These integers become
SQL span_type, not native message or receipt statuses. Complete parser behavior,
field meaning and subtype interface continuity remain unknown.

The separate non-1jl **1he** path requires property **0x329f**, both chat
predicates 0DE.A0i/A0b false, a nonnull 7vS.A00 text and 0CM.A0p(text) false.
It parses the string and writes nonempty spans with span_type **1**. Do not
assume every ordinary text reaches this path: interface membership and effective
properties have not been established. A failed guard returns without a write.

When qualifying spans were handled, Ce8 ORs **0x10000** via **178.A0I(long)**
and invokes Ayo.A13. A0I reads **A0n**, ORs the supplied bits and writes A0n back;
it does not set base message **A08** status. The full semantic name/wire role of
this flag is unknown. The third 1jl span branch goes straight to the flag write;
other paths use v5 to remember an earlier qualifying list before joining it.
An unconditional hook call is therefore not an unconditional flag mutation.

7vS.A00 selects text by subtype: 1jl AZX.A0I first, otherwise Co4 helpers,
1he's A0f(), or 1jg/72u cast to 1iB.Ao1(); remaining inputs return null.
These selection priorities are retained without inventing all helper semantics.

## Span rows and actual failure scope

1jD.A00 returns immediately for an empty list. Otherwise it opens a store and
transaction, iterates the provided numeric pairs and inserts **message_span_indices**
rows with message_row_id/A0j, start_index/first, end_index/second and
span_type/supplied integer. The call uses **INSERT_MESSAGE_LINK_INDICES_TABLE**
and numeric 0IY.A09 argument **4**; complete insert/conflict semantics remain
unknown. Its returned ID is ignored. Success marking, close and rethrow paths
are retained. The method does not validate index bounds or positive message ID
at this boundary; parser/caller/database guarantees are separate unknowns.
No receipt_user SQL or direct native status setter appears here.

## Queued update and remaining transitive effects

Ayo.A13 resolves **0zC** and calls **A0M(message)**. A0M diagnoses **negative**
A0j with 00K.A0D and passes reason **-1** to A0R; its actual branch does not
reject zero. The assertion helper's full policy is unknown. A0R constructs
**Ahi(store,message,reason,5)** and submits it with local scheduling code **22**.
The separate reason-6 helper branch is not selected by this -1 call. Neither
selector, scheduling code nor reason is a receipt state or server retry wait.

The selected Ahi constructor stores discriminator 5, original message and
original reason. Run selector **5 -> 052a** resolves **0V9.A07(message,reason)**.
A true result sends Android Handler message **what=2**, preserving reason in
arg1 and zero in arg2. A false result returns without that notification.
This is an Android observer notification, not native status 2 or server ACK.

A07 checks the type registry for **1hk** support, logs unsupported types and
returns false. Its selected supported path opens a transaction, invokes before
**DbS.CHP(message,null,reason)** hooks from 1hh.A06, then **3Rd selector 6**,
then the concrete **1hk.CgJ**, then after **DbR.CHP** hooks from 1hh.A05.
The normal after-loop branch lands at **01b8**, skipping the unsupported-update
diagnostic. It marks success, stages **3Pf selector 27**, closes and selects
true. That boolean is not a demonstrated SQL affected-row count. The repeated
registry-check failure branch has its own diagnostic; full runtime reachability
is unknown. Exception handlers preserve close/rethrow behavior.

**Unknown/unported:** the update callback, staged callback, update-hook provider
membership/effects and full main-row update semantics. The previously resolved
text inserter also has CgJ, but all update interface/type/caller continuity is
not inferred from a shared method name alone. No claim of transitive status
preservation, native user-row creation or completed primary cleanup follows.
The remaining 35 after entries and other before hooks are still open. Existing
offline CLI source export/diff tools reproduce this boundary; no HTTP/live
capability was added.

Validation: ten topics / fifteen files / 123,331 bytes reproduce exactly from
the pinned APK. Cached self-diff verifies ten unchanged topics and four metadata
artifacts. CLI reference regeneration, API/library-symbol, formatting, whitespace
and hygiene pass (2,038 local links; 96 bundles / 20,325,838 bytes). No fresh
workspace/runtime suite or live test is claimed.
