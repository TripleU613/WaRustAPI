# Final registered after-hook mappings

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-final-after-hooks.json),
[complete methods](apk/2.26.37.71-text-final-after-hooks/targets.json) and
[ordered 54-entry index](text-after-hook-index.json) continue
[association-hook coverage](text-association-after-hooks.md). Offsets are
method-relative bytes. No Rust/account/live behavior changes.

## Thirteen remaining entries

These are actual indices in the retained 07A.A3B initializer. Root selection
uses **(ID >> 14) & 0x7f**; selected factories use **ID & 0x3fff**. Root
2/4/5/6 select **1xr/6as/8kb/Ayr.A00**, respectively. Root-zero low partitions
use the previously retained A01 dispatch. Factory/CHJ continuity is retained.

| ID / array index | Dispatch / masked key / offset | Hook |
| --- | --- | --- |
| 0x18299 / 13 | Ayr.A00 / 665 / 2848 | D7g |
| 0x1413b / 24 | 8kb.A00 / 315 / 0f0e | D7n |
| 0x180e3 / 42 | Ayr.A00 / 227 / 1590 | D7Z |
| 0x81d4 / 43 | 1xr.A00 / 468 / 197a | 3KP |
| 0x1412c / 44 | 8kb.A00 / 300 / 0e5a | AVH |
| 0x77b / 45 | 1UE.A0N / 1915 / 05e8 -> 0HZ.A0g | D7m |
| 0x8450 / 46 | 1xr.A00 / 1104 / 4de0 | 3KQ |
| 0x131b / 47 | 1UE.A0A / 4891 / 0502 -> 0pz.A0A | 3KL |
| 0x10253 / 48 | 6as.A00 / 595 / 1fc8 | 8CH |
| 0x142ef / 50 | 8kb.A00 / 751 / 242e | D7O |
| 0x858e / 51 | 1xr.A00 / 1422 / 5eee | 3KM |
| 0x84b3 / 52 | 1xr.A00 / 1203 / 5424 | 3KO |
| 0x848a / 53 | 1xr.A00 / 1162 / 5164 | 3KN |

**Confirmed static:** all **54 registered after entries** now have immediate
factory and executable CHJ attribution. The index keeps ordered IDs, hook names,
source bundle/topic, direct setter presence and evidence documents. It enables
inspection without redoing dispatch discovery. It does **not** establish complete
transitive helper interpretation, cross-release parity or Rust implementation.
The remaining before-hook set and downstream effects still require work.

## Bounded effects and exclusions

D7g requires CB5 qualification, D6e extension and a selected nonnull 0ZY.
It updates **message_conditional_reveal** by message_row_id, then inserts if
update count is zero; insert result **-1 throws SQLException**. Values include
message_row_id, null proto_data/stanza_data, key_id/key_jid, conditional_reveal_type,
chat_row_id, from_me and scheduled_time. It then calls **DGf.A03** with selected
bytes/key/time inputs. Full scheduled-send behavior is **unknown/unported**.
The renderer leaves opcode **0x61** at 016e as an explicit unknown marker;
the referenced CBc.A00 value and ensuing deadline arithmetic are **not decoded
or assumed** here. This is a source-tool limitation, not a server requirement.

D7n reads D5z appointment metadata or a flagged 1jl/CrW AppointmentInfo path.
The body writes **message_appointment** by message_row_id, with zero-count
insert fallback inside a transaction, and contains chat-scoped appointment
repaint paths through DQy, DMH and its A00 helper. Columns include appointment_id,
status, format, start/end seconds, call_link, client_note and canceled_by_business.
**status here is an appointment-table column**, not a demonstrated 178.A08
assignment. Its missing-ID/update-input/chat diagnostics are retained. Complete
update predicates, callback effects and interpretation of enum cases remain
**unknown**; no mandatory registration or receipt field is inferred from them.

D7Z requires origination flag **0x800000000** and CmJ/D6O. It writes
**tee_chat_request_table** from CQK.A00 values plus message_row_id through
A09 numeric argument **3**. Negative returned ID logs and skips the selected
transaction success mark; nonnegative result marks it before close. Full CQK
values and TEE behavior remain unknown; this is not attestation provenance.

AVH requires origination flag **0x400000000**, AV1 metadata and its key.
It writes **group_history_bundle_association**: message_row_id, message_sort_id,
bundle_message_row_id, bundle_sender_jid_row_id and bundle key ID/from-me/chat
row fields. It diagnoses a negative SQL result; other helper/namespace behavior
remains unknown. This association is not the previously traced receipt import.

D7O delegates to the message-add-on manager. Its selected collection branches
set **native status 17** on related **BnG/1iZ** children without selected bytes
and on **BnD** poll-vote children, then delegate to 0Y5.A00; other branches
use 1fV, 0Z2, BnF and BnI helpers. Poll-option lists may be rewritten. These
are setters on iterated add-on objects, not a proven ordinary text-send state
transition. Full admission/storage/callback effects and enum meanings remain
unknown. A blanket absence of status setters in the after set would be false.

3KP operates on A09-related message/thread metadata with 35w/23h/3LW/1zk
helpers, can publish a 25r observer value and call **0V9.A07(input,reason)**.
Its full qualification, thread persistence and update effects remain **unknown**.
3KQ.CHJ delegates to **3KQ.A00**, which checks property 0x18d6 and input
branches before taking a 3K8 prompt list and queuing **3R5 selector 7** under
**BotMessagePromptsProcessor.STORE_PROMPTS_WORKER_TOKEN**. The worker is
untraced; scheduling does not prove a durable prompt row or receipt state.

3KM diagnoses selected dependent-send/render feature paths through 0AH; complete
meaning of predicates/settings and logging side effects remains unknown. 3KO
selects text from media/text subtypes and guarded translation settings, defers
when CPU-intensive call/camera work is detected, then invokes **Lbd.A00**.
Translation result persistence and all worker effects remain unknown. 3KN
attaches/updates AI-thread metadata, delegates through 1em/3LV, stages a
**3R5 selector 4** callback, and can OR a lookup flag through A0I then call
**0V9.A07(input,reason)**. Full AI-thread policy and downstream update effects
remain unknown. None is interpreted as general peer identity authority.

8CH handles 8Bi experience-ID lists through 7ql/7f7 and **8G7.A03**.
The selected store filters to **positive integer IDs**, joins them as a comma
string and writes **experience_id** with message_row_id, is_from_me,
experience_ids and timestamp. It updates by message_row_id/is_from_me, inserting
when affected count is not positive; SQL exceptions are passed to a diagnostic
helper and boolean result handling. This list is **not receipt-user membership**.
Full experience policy and boolean interpretation at all callers remain unknown.

D7m's payment path delegates through Fdp, Cg7, 12K, 0zj, 0zl, Cgi and CPo;
it can modify checkout metadata and related stored messages. Full transaction,
permission/settings and callback behavior remain unknown. No payment runtime
capability or new server response is claimed.

3KL delegates qualifying comment messages to **0q0.A03** then
**31a.A03(input,false)**; a separate 2Yz path calls 31a.A01. The selected
comment writer requires nonnull 323/3K0 parent ID unequal to **-1**, writes
**message_comment** (message_row_id,parent_message_row_id) through A09 numeric
argument **4**, discards the returned ID and marks/closes a transaction.
The guard is not positive-ID validation. 31a.A03 can look up the parent, OR
parent lookup flag **0x10**, call **0V9.A07(parent,-1)**, update 323/2Yz
metadata and post a 3R3 Handler callback. Full parent selection/counting and
callback effects remain unknown. This is not a read receipt or user-row seed.

## What complete mapping proves

**Confirmed bounded negative:** all 13 newly selected CHJ bodies have no literal
**receipt_user** SQL. Their sole direct **178.A0H** sites are D7O's two
add-on-object setters; AVI's new-system-message setter was traced separately.
Four selected delegates here contain no direct receipt_user or 178.A0H call.
This only excludes direct instructions in those retained methods. Shared SQL
helpers, predicates, workers, nested insertions and triggers remain untraced
and cannot be declared absent or status preserving.

**Unknown/unported:** other before hooks; transitive receipt/user-row lifecycle;
complete normal send status progression; effective receipt policy and namespaces;
primary identity cleanup. The complete after registry is a research milestone,
not proof of functional client completion. No account was opened, no live request
ran and no request ID changed. Existing offline APK export/diff CLI commands
reproduce the evidence; no new flag, HTTP route or live capability was introduced.

## Validation

Final-hook validation: exact 25-topic export (30 files/1,106,965 bytes), cached
self-diff (25 topics/four metadata artifacts unchanged), complete 54-entry source
index assertions, CLI reference, API/library-symbol, formatting and whitespace
checks passed. Hygiene checked 2,090 local links and 102 APK bundles. No fresh
runtime suite or live account test is claimed.
