# Main-row update and separate flag masks

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-main-update.json) and
[complete methods](apk/2.26.37.71-text-main-update/targets.json) continue
[the queued link update](text-link-after-hook.md). Offsets are method-relative
bytes; signed DEX branch displacements are multiplied by two.
No Rust/account/live behavior changed.

## Selector six and actual update arguments

3Rd.<init>(Object,int,int,Object) puts discriminator in $t, message in A02,
reason in A00 and updater in A01. Invoke's switch only lists 0..5, so **6**
selects the default at **000e**, not a missing case or a new message status.
It casts A01 to 0VC and A02 to the original message.

The selected special branch requires fromMe, message.A0Y, chat predicate
0DE.A0Q true, 0DE.A0i false and original reason neither **4** nor **1**. It
selects boolean true and obtains an integer from 2tw.A00 when positive,
otherwise -1. Failed guards select false/-1. Both join the same
**0zc.A0B(message,integer,boolean)** call. The original reason is used in the
guards; it is not simply forwarded as A0B's integer. Full 2tw producer semantics
remain unknown. A0B uses that integer to partition a participant list only in
its true branch; it is not assigned as native status.

The false branch starts at **025e** and sets storage type through lowercase
A0h(1), then invokes the retained
[common values writer](apk/2.26.37.71-text-main-row/main-row-values.txt).
SQL status therefore reads **B3b()**, while text_data and lookup_tables retain
their existing helper inputs. No direct uppercase A0H setter occurs in the
selected callback or main-row writer. Before/after update hooks and virtual
getter effects remain separate unknowns; full status preservation is not proven.

## SQL results do not establish row existence

The false branch also writes broadcast/A0Y, recipient_count/A06,
participant_hash/A0U, received_timestamp (A0C, or 08B.A00 when zero), server_sts
only when A0D>0, and view_mode (A0M, or integer zero). It updates table **message**
using **chat_row_id/from_me/key_id**, with arguments from the already retained
[0zc.A05](apk/2.26.37.71-receipt-projection/status-key-arguments.txt).

**UPDATE_MESSAGE_MAIN_SQL's affected-row count is ignored.** If in-memory
A0j is nonpositive, it queries FIND_MESSAGE_MAIN_SQL by the same key and assigns
_id only when a row is returned. An empty result has no inserted-row fallback
or refusal in this selected method. Store/cursor close and suppressed exception
handling are retained. Consequently outer wrapper true/observer notification
is not proof that this SQL updated or found a row, or that a receipt_user row
exists. This differs from the separately attributed receipt status writer's
row-count diagnostic; do not silently transfer that behavior here.

The true branch uses the participant list or a diagnosed empty-list scan.
Its scan predicate is **broadcast=1 AND from_me=1 AND key_id=?**; the other
path adds participant chat-row IN selection and supports positive-sized chunks.
It uses private A02, which delegates to common A03, then writes recipient_count
and received_timestamp. Full list provenance, row conversion and database
transaction/trigger effects remain unknown/unported. Retaining these branches
does not authorize cross-chat updates in Rust based on guessed participants.

## Lookup and origination masks are different fields

The common writer obtains **lookup_tables** from **Cjb.A00(message)**. Its
selected call at **011a..0126** passes bit **0x10000** and the accumulated mask
into Cjb.A01. A01 checks **178.A0a(bit)** and ORs that bit into the returned mask
when set. A0a reads **A0n** and tests (A0n & bit)==bit. This establishes how the
Ce8/A0I link flag reaches the computed lookup mask at this call site. Cjb also
derives other bits from payloads/predicates; the result is not asserted to be
an unchanged copy of A0n. Full helper semantics and all runtime branch inputs
remain unknown. Other flags in the complete method are retained without naming
untraced feature meanings.

**origination_flags** instead comes from **178.A08()J**, which returns **A0A**.
When Apk() is empty, that getter first invokes A0L(0x40); A0L clears the supplied
bits using A0A & ~mask. It does not OR them. **A0b(bit)** tests A0A, whereas
**A0a(bit)** tests A0n. This resolves the exploratory A0b/A0a ambiguity; it is
not evidence that the masks are interchangeable. Apk semantics and subtype
overrides remain unknown. The long-returning method named A08 is distinct from
int field A08/native status. Getter purity is not assumed: this getter can
change A0A. No native status change is shown by that flag-side effect.

## Remaining authority

**Unknown/unported:** all update-hook effects/provider membership, staged
callback effects, full ordinary send progression, settings/namespace admission,
actual native user-row membership and primary identity cleanup. Remaining
unmapped after-insert entries and delegated helpers still require attribution.
Existing offline APK CLI tools reproduce the source and cached diffs; no new
HTTP/live capability, account operation, receive window or message ID was added.
Validation: ten topics / fifteen files / 73,171 bytes reproduce exactly from
the pinned APK. Cached self-diff verifies ten unchanged topics and four metadata
artifacts. CLI reference regeneration, API/library-symbol, formatting, whitespace
and hygiene pass (2,047 local links; 97 bundles / 20,399,009 bytes). No fresh
workspace/runtime suite or live test is claimed.
