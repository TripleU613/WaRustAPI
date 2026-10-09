# Registered bot metadata after-insert hooks

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-bot-after-hooks.json) and
[complete methods](apk/2.26.37.71-text-bot-after-hooks/targets.json)
continue [the registered after-hook trace](text-after-hooks.md).
Offsets below are method-relative bytes, not DEX file offsets. This chunk
changes source evidence only; it does not change Rust receipt admission.

## Provider continuity

The retained [07A initializer](apk/2.26.37.71-text-type-inserter/hook-provider-id-arrays.txt)
places **0xf4b** and **0xf53** at after-array indices **2** and **3**.
Both select root bucket zero through 1UE.A00, then the low-ID A01 partition
whose interval is **0xf44 < ID <= 0x105b**, selecting **A07**. A07 masks with
0x3fff; exact packed-switch cases **3915 -> 0066** and **3923 -> 00b6**
call **0cI.A0K -> new D7a** and **0cI.A0C -> new D7j**, respectively.
Factories and executable CHJ bodies are retained, so this establishes registry
membership rather than relying on suggestive class names.

**Refuted/unselected exploration:** 0cI.A0G constructs Cd6; it is not the
factory selected by case 3915. Its short body is retained to prevent reuse of
that incorrect candidate. Numeric switch matching must preserve token boundaries.

Together with the previous 19 mapped entries, this gives **21 of 54** immediate
after-hook boundaries; **33 remain unmapped**. Mapping is not full helper parity.
The registered insertion wrapper supplies a null second CHJ argument; D7a's
nonnull-result rejection therefore does not characterize normal insertion.

## D7a: bot information and plugin metadata

**Confirmed static:** D7a.CHJ collects typed extension values (000c..0094),
conditional 1hu strings/bytes, and Bmo collection JSON. The JSON uses
collection_id, expected_media_count and has_global_caption. Its tests at
0128..017c may synthesize a D65 with null invoker, B2I.A04 and empty target
string, then attach it via B34.A02. Complete predicate/enum semantics are
**unknown**; this mutation is not a base A08 message-status assignment.

With nonnull D65, 018e..01c8 calls **DGm.A05** with the collected values and
message **A0j** row ID. Independently, nonnull B2y.A00/D6B at 01ea..0210
calls **DGY.A01**. Neither path runs unconditionally for every text message.

DGm.A05 constructs ContentValues: message_row_id from the supplied row ID,
target_id from D65.A02 and **message_state from D65.A01's enum ordinal**.
This column is in **bot_message_info** (03fa..040a), not the main message
status or receipt_user. Optional columns include invoker_jid_row_id,
model_type, message_disclaimer, keyword_json, promotion_message, bot_fbid,
participants_metadata, verification_metadata, response_viewed,
ai_media_collection_metadata_json, signature_validation_status and tool_calls_blob.
The complete source retains guards, JSON/protobuf conversions and other columns;
not all nested serializers/producers have been traced. Tool calls are written
as delimited protobuf records into a byte array before the SQL helper.

DGY.A01 writes **bot_plugin_metadata** with message_row_id and optional
search_provider, plugin_type, thumbnail_cdn_url, profile_photo_cdn_url,
search_provider_url, reference_index, search_query and favicon_cdn_url.
Both selected stores call **0IY.A09 with numeric argument 5** and discard its
returned row ID. Their selected boundaries do not validate a positive message
row ID or a successful returned SQL ID. Effective conflict/trigger semantics
of the shared SQL helper are **unknown here**; diagnostic names alone do not
prove update behavior or durable success. Store close/exception tables are retained.

## D7j: sharing information and cached prompt association

**Confirmed static:** both **CXY.A02(message)** and **8lz.A01()** must pass
(0024..004c). Their effective settings/predicate semantics are **unknown**.
The body then reads message A0j and key A01. Conditional branches use 1ht
predicates, D65 target strings and C9n/D6c metadata. One branch remembers
(key ID, C1K) in **D7j.A03**; matching a later target string may reuse it.
This cached association is not an identity pin or receipt-user membership.

Several branches call **DGj.A01** directly. Another queries
**bot_message_sharing_info** by **message_id = ?**, using the D65 target
string (0230..0254), within a transaction; a missing query result skips the
selected insertion, whereas a result supplies C1K for the next DGj.A01 call.
It marks/closes the transaction and may cache (row ID, D6c) in field DGj.A01.
The CHJ catch branches log fixed BotMessageSharingInfoStore diagnostics and
can continue to attach D6c using 6al.A1M. Thus return/attachment is not proof
that an SQL write succeeded. Full DGj.A00 query and nested helper behavior
remain **unknown**. The diagnostic mentioning DeletePrompt does not itself
establish a deletion: no direct SQL delete occurs in this retained CHJ body.

DGj.A01 writes **bot_message_sharing_info** with message_row_id, message_id,
nullable bot_entry_point_origin (C1K.A01/C4G numeric value) and forward_score.
It calls **0IY.A09 with numeric argument 5**, discarding the returned ID.
The writer itself has no positive-row-ID guard; callers contain row-ID branches.
Do not infer a universal guard from the convenience writer signature.

## Receipt authority and remaining limits

**Confirmed bounded negative:** the two selected CHJ bodies and three selected
store methods contain no direct **178.A0H** call and no literal **receipt_user**
SQL destination. Their immediate SQL destinations are the three bot tables
above. This does **not** exclude effects from untraced extension helpers,
serializers, shared SQL helpers/triggers, predicates or callbacks.

**Unknown/unported:** 33 remaining after entries, the other before hooks,
full normal send progression, effective policy/namespace admission, native
receipt_user row existence and primary identity cleanup. This investigation
narrows candidates; it does not justify creating receipt membership from
metadata, target lists, message row IDs or bot message_state. No account was
opened, no network request ran and no request ID was created or changed.
Existing apk-smali/export/diff CLI commands reproduce the work; no new command,
flag, HTTP route or live capability was introduced.

Validation: all nine source topics / 14 files / 121,667 bytes reproduce
exactly from the pinned APK. Cached self-diff verifies nine unchanged topics
and four metadata artifacts; it is not a comparison with another APK release.
CLI-reference regeneration, API/library-symbol checks, formatting, whitespace
and repository hygiene passed (2,056 local links; 98 bundles / 20,520,676 bytes).
This source/docs chunk claims no fresh workspace/runtime suite or live test.

Follow-up: [eight more metadata hooks](text-metadata-after-hooks.md) brings
the current immediate mapping to 29/54, leaving 25 entries. Historical counts
above describe this bundle; native receipt admission remains unresolved.
