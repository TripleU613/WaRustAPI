# Selected registered after-insert hooks

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-text-after-hooks.json),
[complete methods](apk/2.26.37.71-text-after-hooks/targets.json) and
[exact provider mapping and negative results](text-after-hook-evidence.json)
continue [registered text insertion](text-type-inserter.md).
Offsets are method-relative bytes. No Rust/account/live behavior changed.

## Nineteen immediate hook boundaries

The previous 07A.A3B initializer supplies the 54 after-hook IDs. This trace
resolves the consecutive **0x14a4..0x14b4** entries through root case 0 /
1UE.A01 / A0B; each selected factory constructs the listed hook. Their array
positions are recorded from A3B, not assumed to equal message types. Factory
methods and CHJ bodies are both retained in the new bundle.

| Provider | Hook | Immediate confirmed boundary / remaining limit |
| --- | --- | --- |
| 0x14a4 | 8CB | 7RY.A00 qualification -> 8G6.A03; helper effects unknown |
| 0x14a5 | 3KJ | data_sharing_disclosure_metadata write |
| 0x14a6 | 3KK | message_external_ad_content write |
| 0x14a7 | D7R | message_biz_context_info update/insert/delete |
| 0x14a8 | 8CC | flag 0x4000 and A0c -> 6bf.A00 collection helper |
| 0x14a9 | D7l | message_details author_device_jid write |
| 0x14aa | AVG | flag 0x20000 and Cbt.A00 qualification -> 16U.A01 |
| 0x14ab | D7i | A16/extra-table path -> serialization/store helper; full effects unknown |
| 0x14ac | D7c | message_privacy_state write; reads B3b, no direct status setter |
| 0x14ad | D7S | nonnull A09() -> 1jQ.A01; helper effects unknown |
| 0x14ae | D87 | Cbj.A00 qualification -> 0zm.A04; helper effects unknown |
| 0x14af | D7Q | group_history_share_reporting_info write |
| 0x14b0 | 8CJ | status_attribution write |
| 0x14b1 | 8Fv | conditional 1iA/8Bw state changes and 7sM.A01; full helpers unknown |
| 0x14b2 | 8CD | 1iu subtype -> 7sx.A02; helper effects unknown |
| 0x14b3 | D7T | support_citation_metadata write |
| 0x14b4 | D7U | url_tracking_map_element update/insert; oversized lists diagnosed with maximum 32 |

The first two after-array entries use a different root bucket: **0x400a**
and **0x400b** select 1UE.A00 case 1 -> **1qQ.A00**, whose masked cases 10/11
construct **8CE/D7V**. 8CE.CHJ qualifies a chat/string and invokes 8G1 parsing
or queues 8UO selector 8 through 7av's executor; callback effects are unknown.
D7V.CHJ delegates to **Ce8.A00(message,helper)**. Its effects remain unknown.
These two entries are included in the 19 mapped after hooks, not additional
entries outside the original array.

**Confirmed negative:** none of these 19 selected CHJ bodies directly calls
178.A0H or writes receipt_user SQL. This is a source-boundary exclusion, not
proof that delegated, serialized, trigger or asynchronous effects preserve
status or never create user rows. A metadata table name does not establish
native receipt membership. Full activation conditions and concrete dependency
semantics are not ported or inferred from provider names.

The D7c body reads the existing B3b status for a privacy branch and writes
privacy metadata; reading status is not setting it. 8Fv can mutate subtype
8Bw.A0K, which is distinct from base message A08. Several hooks reject a
nonnull result argument; the retained normal 1hh.A01 invocation passes null.
Failures in provider construction or helper work can still abort iteration.
Complete guards/helper effects are retained but remain partially uninterpreted.

## Routing correction and next authority boundary

**Refuted:** using 1UE.A02 as the root-bucket-one dispatcher. The actual root
switch calls 1qQ.A00; A02 is a different low-ID partition. An exploratory 0AW
CHJ query returned no executable body and is not evidence for an insertion hook.
Exact numeric switch keys must be used; a suffix substring can match another
integer. The unselected A02 body and negative result are retained so this dead
end is not repeated. No incorrect provider adoption entered Rust or account state.

**Unknown/unported:** the remaining **35 after** entries, the other before hooks,
all delegated/asynchronous effects here, complete normal send progression,
settings/namespace admission, native user-row membership and primary identity
cleanup. This narrows source candidates without inventing a receipt row model.
Existing offline export/diff tools reproduce the scoped evidence; no CLI command,
HTTP route, live send, receive window or new request ID was added.
Validation: 39 topics / 44 files / 309,159 bytes reproduce exactly from the
pinned APK. Cached self-diff verifies 39 unchanged topics and four metadata
artifacts. Exact factory types/switch keys and direct-body exclusions were
checked. CLI reference regeneration, API/library-symbol, formatting, whitespace
and hygiene pass (2,031 local links; 95 bundles / 20,202,507 bytes). No fresh
workspace/runtime suite or live test is claimed.

Follow-up: [D7V link/span helper](text-link-after-hook.md) attributes Ce8
and its update queue; transitive update effects remain unknown.
