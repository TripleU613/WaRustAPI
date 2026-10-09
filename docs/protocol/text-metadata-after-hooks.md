# More registered metadata after-insert hooks

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-metadata-after-hooks.json) and
[complete methods](apk/2.26.37.71-text-metadata-after-hooks/targets.json)
continue [bot after-hook coverage](text-bot-after-hooks.md).
Offsets are method-relative bytes. No Rust/account/network behavior changes.

## Eight registered entries

The [07A A3B initializer](apk/2.26.37.71-text-type-inserter/hook-provider-id-arrays.txt)
puts these entries at indices 4..11. All use root bucket one: **1UE.A00 ->
1qQ.A00**, whose **ID & 0x3fff** packed switch directly constructs each hook.
The complete dispatch method and eight CHJ bodies are retained. The null
second argument from normal 1hh.A01 insertion avoids the selected hooks'
nonnull-result rejection; guard/helper failures can still abort insertion.

| Provider / index | Masked key / factory offset | Hook | Immediate boundary |
| --- | --- | --- | --- |
| 0x41d0 / 4 | 464 / 1626 | D7e | payment_link_metadata |
| 0x41d2 / 5 | 466 / 163e | AVJ | payment_extended_metadata |
| 0x41e0 / 6 | 480 / 16e6 | 8CI | 0zf.A00 inline-video writer |
| 0x41e4 / 7 | 484 / 1716 | D7k | forwarded, newsletter, bot and business metadata |
| 0x41e5 / 8 | 485 / 1722 | D7b | mention JID lookup then 16S.A02 |
| 0x41e6 / 9 | 486 / 172e | 3KI | premium_message_info |
| 0x41e7 / 10 | 487 / 173a | AVF | AZl.A00 question writer |
| 0x41e8 / 11 | 488 / 1746 | D7P | question_reply_quoted_message |

This brings immediate after-hook coverage to **29 of 54**, leaving **25
unmapped**. It does not establish transitive helper parity or receipt admission.

## Guarded writes and failures

**Confirmed static:** D7e requires nonnull C9v/D6D and property **0x3a77
false**. It writes message_row_id, optional link_header_type, cta_button_text
and params_json into **payment_link_metadata** through 0IY.A05. A negative
returned ID is logged; the selected branch then closes/returns without throwing
solely for that result. Effective property/helper/conflict meanings are unknown.

AVJ requires an AUw extension and its own A01 boolean. If the message is 1he,
**178.A0b(0x4000000000)** must also pass. A0b tests the previously traced A0A
origination mask, not base A08 status. The synchronized extension reads are
retained. It writes message_row_id, type, platform and message_params_json into
**payment_extended_metadata** via 0IY.A05 and logs a negative returned ID.
Construction/effective A01 and AUw producer semantics remain **unknown**.

8CI requires **1he.A04 == 1** and property **0x4ea6 true**, then calls
**0zf.A00**. The helper repeats the subtype/A04 checks and additionally requires
nonnull **1he.A0C**. It writes **message_inline_video_metadata** columns
message_row_id, video_content_url (A0C), is_muted (1he.A08 Boolean) and caption
(A0B), through A09 numeric argument 5. This subtype A08 is **not** superclass
178.A08 integer status. A negative SQL result is logged, not thrown by that
check. Runtime property values and URL provenance remain **unknown**.

D7k is gated by **Ayj.A1W(message)**. It writes **message_forwarded** with
message_row_id, forward_score from 178.A02 and forward_origin from A0G.value
(default 1hw.A08 if null). Nonnull B2B/8Bk enables
**forwarded_newsletter_message_info** (newsletter_jid_row_id,
newsletter_server_message_id, newsletter_name, profile_name, message_row_id).
Nonnull 1hx metadata plus D65 delegates to the previously retained
[DGm.A05 bot writer](text-bot-after-hooks.md). Nonnull B2K/D62 enables
**business_message_forward_info** (message_row_id, business_owner_jid_row_id).
Repeated extension reads may throw explicit missing-information errors if the
second lookup is null. The selected direct SQL return IDs are discarded.
Forwarding predicate, enum and extension producer semantics are **unknown**.

D7b reads a 1ib.A01 list, uses 1ib.A03 for DLg JID lookups via 0VT.A07,
then calls **16S.A02**. The helper returns for null/empty lists; otherwise it
opens a transaction and writes **message_mentions** for each recognized entry:
DLg uses message_row_id, JID row ID, display_name and mention_type **0**;
D32 uses message_row_id, jid_row_id **-1** and mention_type **1**. Unknown
entry types are diagnosed and skipped, then normal completion marks/closes the
transaction. A09 numeric argument **4** return IDs are ignored. Neither JID
lookup nor mention membership is native receipt_user membership. List filtering,
all subtype meanings and shared SQL/transaction helper effects remain unknown.

3KI requires **A0b(0x400000)** and nonnull 324.A00 campaign string. It writes
**premium_message_info** with message_row_id, campaign_id, chat_row_id and
conditional account_jid_row_id. Property **0x324f true** selects 0Yo.A02 chat
mapping; a null original chat then throws. Otherwise the original chat is used.
Later lookup mode selects whether account_jid_row_id is included. The selected
0IY.A06 returned ID is discarded. Missing-chat/campaign error branches are
retained; these are local errors, not recognized WhatsApp server responses.
No new namespace aliasing rule is inferred from this specialized metadata path.

AVF requires B30/AUx, then **AZl.A00** rechecks that extension and writes
**message_question**: message_row_id, response_count from AUx.A00,
response_read_count from AUx.A01, is_enabled from AUx.A02. A09 numeric argument
5 returning exactly **-1** throws SQLException; other returned IDs reach close.
This differs from the diagnostic-only negative handling above. These aggregate
question counters are **not** evidence of per-user delivery/read timestamps.
Counter producers/effective feature behavior remain **unknown**.

D7P requires B2J/D6q and rechecks it before writing
**question_reply_quoted_message**: question_text, question_message_type,
response_text, server_question_id, plus values from **8kZ.A09(message)**.
A09 numeric argument 5 return ID is discarded. The complete common-values helper
has not been traced here; it cannot be presumed to create receipt rows.

## Bounded receipt exclusion

**Confirmed bounded negative:** none of these eight CHJ methods or the three
selected delegated writers directly calls **178.A0H** or uses literal
**receipt_user** SQL. Selected destinations and guard/result differences above
are confirmed from executable bodies, not diagnostic names alone. No positive
message-row-ID validation occurs at the selected writers' immediate boundaries.

**Unknown/unported:** remaining 25 after entries, other before hooks, transitive
extension/predicate/SQL-helper/trigger behavior, full normal send progression,
effective policy/namespace admission and actual native user-row membership.
Primary identity cleanup remains unfinished. Metadata rows and counters do not
supply the missing receipt authority. No saved account was opened, no network
request ran and no request ID changed. Existing offline APK CLI export/diff
commands reproduce the evidence; no new command, flag or HTTP route was added.

Validation: twelve source topics / 17 files / 201,770 bytes reproduce exactly
from the pinned APK. Cached self-diff verifies twelve unchanged topics and four
metadata artifacts; no cross-release semantic comparison is claimed. CLI reference,
API/library-symbol checks, formatting, whitespace and repository hygiene passed
(2,065 local links; 99 bundles / 20,722,446 bytes). This source/docs chunk claims
no fresh workspace/runtime suite or live account test.
