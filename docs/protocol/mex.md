# MEX persisted queries and group-info construction

Static comparison APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This trace is separate from the working classic group IQ path. Sources are
[selected request instructions](mex-request-evidence.txt),
[response excerpts](mex-response-evidence.txt), and the previously recorded
[context/dispatch trace](groups.md#subject-recovery-mex-dispatch-and-local-guard-2026-10-07).
Offsets below are method-relative bytes; displayed prototypes are DEX shorties.

## Release data, not scattered Rust constants

**Confirmed (asset):** the APK ZIP contains
`assets/whatsapp-android-mex_client_persist_ids.json` (66,144 bytes), SHA-256
`a528a786209342b3e1d8cf8a78dc046599731c3842ea4bc3b316c64647bdfec0`.
Its selected `QueryGroupInfo` record is retained in the small
[release fixture](../../crates/whatsapp-api/fixtures/mex/2.26.37.71-group-queries.json).
The server document ID is `37759635623635554`; the distinct client ID is
`373629564112111573297758520297`. Operation-name hash is `3736295641`,
operation-text hash `12111573297758520297`, schema `whatsapp_mex`.
The source stores server `doc_id` as a JSON integer. Export converts exact u64
to a decimal string without floating-point precision loss (this value exceeds
2^53). These are public protocol identifiers, not account credentials.

```sh
target/debug/xtask apk-mex-queries /path/to/WhatsApp.apk \
  --operation QueryGroupInfo > lab/releases/next-group-queries.json

git diff --no-index \
  crates/whatsapp-api/fixtures/mex/2.26.37.71-group-queries.json \
  lab/releases/next-group-queries.json

# Reproduce this exact source and asset, including provenance:
target/debug/xtask apk-mex-queries /path/to/WhatsApp.apk \
  --operation QueryGroupInfo \
  --check crates/whatsapp-api/fixtures/mex/2.26.37.71-group-queries.json
```

The new offline extractor accepts positional APK, repeatable required
`--operation`, optional `--namespace` (default `whatsapp-android-mex`) and
`--check`. Names are ASCII identifiers up to 128 bytes; traversal is refused.
It selects at most 64 operations, reads at most 4 MiB of this asset, rejects
missing/duplicate ZIP members, mismatched operation names and missing/invalid
fields, and emits nothing until selection and provenance extraction succeed.
Check compares the complete export. ZIP entry inspection uses `by_index_raw`,
not `file_names`: zip 0.6 implements the latter with deduplicated name-map keys.
An actual two-entry ZIP regression proves the old check could miss duplicates
and the current check refuses them without decompressing unrelated assets. These are local tooling bounds, not server
requirements. No all-operation asset dump or media export is committed.
Format version 1 contains APK version/hash, namespace, asset path/hash and a
sorted operation map with all six selected source fields. Unknown additional
source fields are not exported; review changed source schema separately.

**Confirmed (reader):** `Lae.<init>`+0026..004c obtains operation name and
resolved build config and calls `15t.A00`. That resolves `161.A00(namespace)`
then `162.CFb(operation)`. `164.CFb`+0018..0024 reads `doc_id` with
`JSONObject.getString`; `164.AFf` independently reads `client_doc_id` with
feature/provider branches (`0x5cae`, `0x6285`). Those branches do not establish
that the client ID belongs in this JSON/XMPP request. `161.A00` returns `5dT`
when the namespace is unregistered; its `CFb` returns null. `15t` catches
lookup failure and returns null, rather than guessing an ID.

**Confirmed/unknown boundary:** `164.<init>` creates lazy `165`, whose `invoke`
loads the supplied filename through `Context.assets.open` and parses UTF-8 JSON.
The actual namespace-registration producer and filename supplied to this
constructor are **not yet traced**. Asset discovery plus matching schema is
direct evidence for the extracted record; the complete registration linkage
is an **inference**, not a proven active installation/feature choice.
Exact string search found one `client_doc_id` use (`164.AFf`, classes.dex
absolute 0x7f5e92); `_client_persist_ids.json` as a standalone constant yielded
none. Do not repeat that exact-constant search assuming it will locate an
assembled filename. Related ZIP entry names include the namespace's
`_argo_wire_types.argo` and `_pando_flatbuffer_data.bin`; their schemas were not
decoded here and their existence does not establish an active branch.

## Confirmed JSON/XMPP envelope and Rust port

`0Xm.A0F` passes `0Xo.A02`'s request through `15s.A01` to `GAw.AOK`.
`J8p` stores the call name/build namespace/parameters and response model `2FU`;
its `hasAcsToken` and `hasOhaiConfig` return false. The traced group caller
supplies null secondary model/callback and false request flag. This does not
rule out other transport branches or establish every feature setting.
`15s.A01` creates the wrapper. `GAw.<init>` stores timeout literal `0x7d00`
(32,000); its units remain **inferred** until the timeout consumer is traced.
The default `A04` flag is false; `GAw.A00`+00c4..014a constructs `Lae` for
that branch, while the true branch constructs `MEh`. Methods such as `BS7`
can change the flag, but no such mutation occurs in the traced normal caller.
`Laf.A02` refuses a missing persisted ID before dispatch, then can select a
Pando branch through `168`; that feature choice is **unknown** here. Its
ordinary XMPP fallback +01ac..0208 uses `0h8.A0T` with the constructed node,
fresh IQ ID, literal selector `0x155` and stored timeout. The alternate token,
header and MEh branches are not treated as required invented inputs.

**Confirmed:** `0Xo.A02` writes `group_jid` and `query_context` into a
GraphQlCallInput and `J8H.A02` stores it under `input`. `J8H.Asg` /
`J81.A1B` build a TreeMap; `GraphQlCallInput.A01` recursively transforms its
values. Its private `A00`+0140..015c handles a nested `GBB` as another TreeMap.
It therefore produces a nested object, not the input object's string rendering.
`Lai.<init>`+003e..005a creates JSON `queryId` and `variables`;
`1xg.A1P` invokes the JSONObject's `toString`.

`Lae.<init>`+004e..0074 converts empty/missing ID to null and creates a fresh
IQ ID. `Lai` passes the ID to `BpQ(selector 3)`, which at +004e..008e makes
`trace/flow_id` containing that **same document ID**. It passes the trace, IQ
ID, document ID and JSON string to the unique `9QZ.<init>(VLLLL)` overload:

```xml
<iq to="s.whatsapp.net" xmlns="w:mex" type="get" id="{fresh-iq-id}">
  <trace><flow_id>37759635623635554</flow_id></trace>
  <query query_id="37759635623635554">{"queryId":"37759635623635554","variables":{"input":{"group_jid":"123-456@g.us","query_context":"INTERACTIVE"}}}</query>
</iq>
```

This display uses a synthetic JID. `8kV.A0l` builds `iq`, `8kY.A1H` supplies
`1S7.A00`; its constructor uses an empty user and `getServer` returns
`s.whatsapp.net`. `8kX.A0T` supplies `type=get`; `8ka.A0a` adds `id`.
`9QZ` preserves the displayed attribute order and appends trace before query.
JSON key order is not established as a protocol requirement; the Rust port
preserves values/types/nesting and does not claim byte-identical JSONObject
serialization. Query text is JSON, not base64 or a query source string.

**Confirmed with qualification:** `9QZ` invokes `0hp.A06` for query ID length
1..20 and body length 1..`0x80e800` (8,448,000) Java UTF-16 code units.
`BpQ` checks flow ID length 1..36. Out-of-range lengths invoke `00K.A0D`, but
`0hp.A06` then returns true on that path. Its assertion's fatal/log behavior
is **unknown**; do not claim the APK always refuses/omits such values.

**Implemented (offline, live unverified):**
`whatsapp_api::transport::mex::group_info_request` constructs this node using
an explicit caller-supplied server ID and typed group context. It revalidates
group input, refuses empty/overlong/nondecimal document IDs and uses a local
1..256-byte/control-free IQ-ID bound. These strict local guards are not
additional server requirements. The twelve explicit context variants preserve
the previous source trace, including the private-set override and `UNKNOWN`;
unknown reason text is never arbitrarily uppercased into a context.

```sh
target/debug/xtask mex-group-preview \
  --queries crates/whatsapp-api/fixtures/mex/2.26.37.71-group-queries.json \
  --chat 123-456@g.us --id fixture-iq --context INTERACTIVE
```

Preview outputs ordered node attributes, child nodes, JSON text and the supplied
file's APK provenance. It does not authenticate the provenance independently:
use the extraction `--check` above for that. Input is bounded to 128 KiB,
format/namespace/operation/schema must match, and the context must be one of
the exact APK spellings. Output contains the supplied JID and ID; use synthetic
values for shared diagnostics. No account/store is opened, ID reserved, action
queued, group state overwritten or live request sent. There is no HTTP route.

## Response boundary and next port

**Confirmed, partial:** `LaT.A02`+0000..007a selects child `result`, otherwise
`update`, and throws `MEn` when neither exists. A string of length >=2 beginning
with `{` enters the JSON path through `LaV`, `Lab` and inherited `LaZ.A00`;
the alternate path looks up an Argo wire type by namespace/operation. `Lab`
gets one parser from provider `0x1763` and O4r selector 2/1/4 callbacks.
`LaZ` runs a first parser then separate callbacks; its diagnostics explicitly
distinguish failing `data`, `errors`, both, and JSON/Argo/OOM failures.
Their complete parser semantics, result/error schema and partial-data policy
remain **unknown**. An IQ `type=result` alone is not proof of GraphQL success.

`2FU` itself is only a JSONObject-backed `LbE` constructor, with no schema
getters. Initial `0Xo.A03`+000c..0054 reads `xwa2_group_query_by_id` as `2FT`,
wraps its object in `90x`, then uses `ADv` getters and `90x.A0J` followed by
a `value` lookup. The latter's subject interpretation is **inferred** until that
getter is traced.
The method also references `total_participants_count` and missing-participant
identification, but its full conversion/union/member semantics are not traced.
Do not replace a complete saved group with a guessed interpretation of these
field names. Required next trace: namespace/provider registration, LaV/O4r
JSON error handling, 90x/ADv/2FT/2H8 response schema, complete 0Xo.A03 state
conversion, Pando/Argo selection and native dependencies. Durable dispatch and
the separately traced monotonic rate guard must follow that response work.
Full group recovery, subject application and MEX live behavior are unfinished.

## JSON response field port (2026-10-07)

This closes the earlier **unknown** JSON provider/field-reader boundary, while
leaving the error-model/callback and full group conversion boundary open.
[Selected parser instructions](mex-json-parser-evidence.txt) and
[single provider-switch entry](mex-json-provider-switch.json) retain the source.
All claims in this section refer to the same SHA-pinned 37.71 APK above.

**Confirmed:** `00S.A03` -> `00S.A04`+0062 -> `1UE.A00` dispatches provider
`0x1763` (5987). The high seven-bit group selector is zero, so it enters A01;
the masked low fourteen bits fall in `0x16e7..0x17fd`, selecting A0E.
A0E's packed switch begins at +0004 with a 279-entry payload at +0bc4,
first key 5863. The selected index is `5987-5863=124`. The payload uniquely
matches header `00011701e7160000` in classes.dex at absolute 0x909c78;
selected entry at 0x909e70 is `fd010000`, signed code-unit offset 509.
Thus the target is `4 + 2*509 = 0x03fe`, which invokes `0yN.A00`.
That method creates `O4r(selector 0)`, closing the provider linkage without
assuming that the class name implies its role. The raw entry was checked
against the APK ZIP DEX bytes; it is not a server response or guessed selector.
The current disassembler prints only the first 64 switch cases, so key 5987
was absent from its abbreviated case list. Preserve this negative tooling
finding rather than interpreting an omitted case as an absent binding.

**Confirmed:** `LaV.Axk` supplies the original node. `O4r.Cf4` selector 0
(+0090..00f0) chooses `result`, else `update`, reads its string, and invokes
`Ayk.A18` (new JSONObject from the string) when nonempty; missing child throws
MEn, empty/null string yields an empty JSONObject. At the higher `LaT.A02`
JSON/Argo discriminator, however, only length >=2 and first character `{`
enters this JSON parser. Do not remove that distinction or retry malformed
`result` by falling back to an available `update`.

`LaZ.<init>` ties its fields to `Lab`'s callback arguments: A03 is that provider,
A00 is O4r selector 2 (`data`), A02 selector 1 (`errors`), A01 selector 4
(error-parser failure conversion). Selector 2 (+002a..0056) returns null for
absent/explicit-null data, otherwise requires JSONObject. Selector 1
(+0058..008e) returns a new empty JSONArray for absent/null errors, otherwise
requires JSONArray. It does not constrain the array elements here.
`LaZ.A00` runs errors and data independently, catches each failure and preserves
valid counterparts. Both valid become `LZw(data, errors)`. Invalid data with
valid errors becomes a structured MEq failure; both invalid become a two-cause
MEq failure. Invalid errors with valid data are converted by selector 4/default
into a one-element array containing `String.valueOf(exception)`, then returned
with the data. Therefore malformed errors cannot be safely treated as no error.
Exception text itself is runtime-dependent, and no Java exception text is
invented by the Rust port.

`LZw.<init>` stores A00=data/A01=errors. The later typed `LZv.<init>` stores
both again; for LbE data it derives a nonempty-data flag from A01 JSONObject
length, and a nonempty-errors flag from JSONArray length. Other model types
have separate branches; null/wrong model handling throws. These flags are
**not** a traced group success policy. Caller decisions after model conversion,
per-error code/retry schema and partial-group application remain **unknown**.
`90x.A0J`+0000..0014 explicitly reads `subject` as `8yq`, confirming the
previously inferred subject getter. Full subject union/timestamp/member schema
and the complete ADv/0Xo.A03 conversion remain unfinished.

**Implemented, offline/live unverified:** `transport::mex::parse_json_response`
selects result/update, preserves the discriminator and parses a bounded strict
UTF-8 JSON object. Its local 1-MiB cap and serde_json lexical acceptance are
implementation choices, not additional server requirements or proof of complete
Android JSONObject parser equivalence. It handles the selected fields separately:
valid data and error entries are retained even when their counterpart has an
invalid type. Field states `missing`, `null`, `valid`, `invalid` distinguish
unknown malformed values from accepted empty/missing values. Instead of
fabricating Java fallback strings, invalid fields stay explicit. This is a
response-parts port, not a reproduced full MEq exception/callback hierarchy.
No semantic-success method is exposed; `error_count=0` alone proves nothing.

`MexJsonResponse.data()` / `.errors()` expose private content deliberately;
its Debug and `.summary()` reveal only payload child, field state and counts.
[Sixteen synthetic field-type cases](../../crates/whatsapp-api/fixtures/mex/2.26.37.71-response-parts.json)
are derived from the actual field getters, not live captures. Every case tests
both payload children, independent preservation and content redaction; selection,
invalid JSON/UTF-8, size bounds and content-free errors have separate tests.

```sh
target/debug/xtask mex-response-inspect --input /private/mex-result.json
# If the saved payload came from an update child:
target/debug/xtask mex-response-inspect --input /private/mex-update.json --child update
```

The command invokes the actual Rust node parser. It prints only
`payload_child`, `data_state`, `data_field_count`, `errors_state`, `error_count`;
it does not print group IDs, subject text, server error messages or raw JSON.
Argo/Pando and malformed/oversized input refuse without partial stdout. No
account/store is opened, action settled, group snapshot overwritten, or network
request sent. A direct Laf.A00 method lookup found no matching bodies; full
Laf contains only its constructor/A02, and Lal only its constructor. Those
lookups do not establish that response callbacks are absent from the APK.
The next trace belongs to actual IQ callback/model-conversion callers.

## Versioned parser/request files

[The MEX release recipe](apk/recipes/2.26.37.71-mex.json) exports thirty-eight
method topics to [the 37.71 baseline](apk/2.26.37.71-mex/manifest.json):
43 files, 162,435 bytes including the checksum index. Each topic names
`crates/whatsapp-api/src/transport/mex.rs` as its Rust owner. The baseline is
additional source coverage, not a new APK release or full MEX parity.

**Confirmed source coverage, 2026-10-07:** the callback expansion adds Lac.BEE
(response handler), LaT.A00 (typed model construction), BaseMexCallback.A03
(inherited gate), 9Np.A06 (group-specific error decision), IWF.AYS (numeric
error getter), and I1H.A06 (retryable getter). These method files retain actual
DEX shorties and instruction offsets. The last two are source-only where their
error selection/coercion behavior remains unported. Their existence does not
authorize automatic retry or infer server waits.

[The cached coverage comparison](apk/mex-callback-coverage-diff.json) took
**0.0069 seconds**: six added topics, twelve unchanged topics; targets changed
while inventory, manifest and tokens stayed unchanged. Its old baseline is Git
revision `cdd4056`; old/new file paths in that report refer to their respective
revisions. Both APK hashes are identical. This measures cached comparison only,
not a release change, APK scanning, full-app parity or behavioral differences.

```sh
target/debug/xtask apk-research-bundle /path/to/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-mex.json \
  --output docs/protocol/apk/2.26.37.71-mex --check

# After re-anchoring and exporting the next actual APK:
target/debug/xtask apk-research-diff \
  docs/protocol/apk/2.26.37.71-mex lab/releases/next-mex
```

Use `--check` only with an existing complete export; for a new output directory,
omit it. [Actual remapping](apk/remap-evidence/2.26.37.71-mex.json) on this APK
has fourteen unique, seventeen unanchored and seven ambiguous topics. IWF.AYS is unique
only after shorty `I` removes four other code/error_code methods. `is_retryable`
plus shorty `Z` still matches I1H.A06 and 916.BGC/BQ2: one string and a return
type do not establish the intended owner. Common doc_id and subject anchors
also remain ambiguous. String-free construction/provider/storage/model/gate
methods remain unanchored and require caller-based/manual review on a new APK.
A full proposed recipe remains blocked until every mapping is resolved; class
names alone are not a safe patch. Selected persisted-ID asset files remain
separately diffable.

## Group callback gate and inherited dispatch (2026-10-07)

**Confirmed static**, same SHA-pinned APK 37.71:
[callback/source evidence](mex-callback-evidence.txt) closes the path from the
ordinary IQ handler to group-specific success/error decisions. `Laf.A02`
+01cc..01ec creates Lac, then LbK. LbK delegates response handling to 8lU;
8lU classifies through 9pA/1xl and calls Lac's BEE/BEC/BEF/BEW. Complete
transport classification remains untraced, but BEE is the traced successful-IQ
path into LaT.A01/A02 and typed model construction through LaT.A00.

LaT.A00 uses the requested class constructor accepting JSONObject (2FU for
group info), creates LZv with the parsed errors, and wraps constructor failures
as MEj. Missing/null data cannot construct 2FU: its constructor asserts a
nonnull JSONObject. LZv's flags describe nonempty model/errors, not complete
group validity. Lac.BEE+00e4..0116 constructs I1H when errors are nonempty,
records its flags and invokes Okj.C2c. A nonempty error array therefore does
not alone stop this transport handler from calling C2c.

The direct superclass of 9Np is
`com.whatsapp.infra.graphql.pando.BaseMexCallback` (read from DEX class_def_item,
not guessed from naming). Its C2c invokes A03. A03 constructs I1H for errors,
calls the subclass A06, and invokes subclass A04 only if A06 permits it and the
model is nonempty. With no errors it also requires the nonempty-model flag.
`I1H.<init>`+004c..0064 calls JSONArray.getJSONObject for **every** error entry;
an array of strings/null/primitives accepted by the earlier field reader is
not a valid typed callback error summary. Exceptions have separate telemetry/
product-failure paths; they are not converted into successful group data.

**Confirmed group policy:** 9Np selector 0 A06 returns false at +0028..002c,
including after its normal cleanup, numeric code inspection, 429 timestamp
update and scheduled runnable. The +0088 branch returns to +0028. Thus any
nonempty valid error array blocks this group's A04, even with partial data.
Do not import another callback's partial-data policy. When A04 does run, it
cleans up again and consults 0yw.A0x at +0b1a. Only an enabled gate invokes
0Xo.A03 and applies its result via 0yw.A0X; disabled goes to return-void.
The feature predicate, complete converter and resulting state transitions
remain unfinished. An invocation of A04 is not a successful snapshot update.

**Implemented offline, live unverified:** MexJsonResponse now exposes
`group_info_disposition()`, with states `unsupported_parts`, `missing_model`,
`invalid_error_entry`, `server_errors`, `empty_data`, `invoke_group_callback`.
Invalid local field states stay unsupported; Java exceptions/fallback text
are not fabricated. On supported shapes, model presence precedes error-summary
construction, all error entries must be objects, any valid nonempty errors
block the callback, and empty error-free model skips it. Nonempty error-free
model reaches the callback, even if it lacks xwa2_group_query_by_id: downstream
conversion must still validate that field and all required group structure.
[Ten synthetic gate cases](../../crates/whatsapp-api/fixtures/mex/2.26.37.71-group-callback.json)
include partial data plus errors and unrelated nonempty data to distinguish
callback invocation from full group validity.

```sh
target/debug/xtask mex-response-inspect --input /private/mex-result.json --group-info
```

The optional flag adds only `group_info_disposition` to the existing redacted
summary. It does not run callbacks, change rate guards, settle actions, open an
account, apply group state or send anything. Complete Java exception/model/
Android coercion behavior and durable MEX dispatch remain unfinished.

### Error-code schema and converter getter boundaries

**Confirmed static, not yet ported:** `1xn.A05` selects through I1H.A00, then
AuM.AYS; it does not simply read the first error. I1H.A00's ranking references
`summary`, critical severity and MGQ exception classes; complete selection
semantics remain unported. `IWF` stores root and optional extensions object.
AYS uses extensions.error_code when that object exists; otherwise root code
with explicit default `0x1a1` (417). A missing extensions.error_code does not
fall back to root code. A00 reads extensions.backoff as a string, A02 compares
extensions.severity with critical ignoring case, A01 reads root path array
(default empty), Act reads root message. Android optInt/optString coercion and
implicit defaults, ranking and backoff duration are not reproduced here.
I1H.A06 reads the selected IWF extensions.is_retryable with explicit default
true (also true when no eligible extensions object exists). That source flag
is not wired into this runtime. These getters establish neither an automatic
retry nor a server wait duration.

[Additional group getters](mex-group-getters-evidence.txt) preserve partial
converter tracing: ADv.A06 reads id and calls 1bo.A01; A04 reads creation_time
through LbE.A0F/8kY.A0B; A05 reads subject.creation_time through nullable A0E;
A03 reads announcement_version through A0E/0Gv.A01 with literal default zero.
A07 reads creator-related pn through 90x.A0H; A08 reads subject.creator.pn;
A09 follows A0H into 8kU.A1A/A0d. At this stage, the underlying 90x.A0H field name and producer/parsing helper
semantics were untraced; the later converter section confirms creator/id and
1000 multiplication. No time/storage units are established by these calls alone. LbE.A04 uses optJSONObject and
reflective construction, returning null on construction failure; A0E treats
missing/null as nullable while A0F uses optString. A03 has a string-specific
0CK.A08 long parse then optLong fallback, and A0G combines integer/boolean
coercion. LbE.A01 error_code enum and A02 retry_after_seconds getters are not
proof that these fields occur in the top-level GraphQL error array or group
reply. This partial research adds no numeric-code or retry-duration CLI output.

### Inspecting inherited methods

`apk-smali` now prints `# extends: <descriptor>` from the checked direct
superclass_idx at DEX class_def_item+8; NO_INDEX prints `<none>`. Malformed type
indices refuse rather than inventing a parent. This is metadata, not method
resolution: inspect the printed parent separately for a missing inherited body.
A method filter that matches no bodies retains the existing explicit diagnostic.
A synthetic class-table regression covers distinct parent, NO_INDEX and invalid
index; the real 9Np constructor inspection demonstrated the callback linkage.
Versioned bundle formatting remains unchanged by this CLI-only header addition.

## Group converter branches and time arithmetic (2026-10-07)

**Confirmed static**, same SHA-pinned APK 2.26.37.71:
[full selected converter/helper evidence](mex-group-converter-evidence.txt)
now includes 0Xo.A03 rather than just its initial field reads. It requires a
non-null xwa2_group_query_by_id object (null goes to IllegalStateException at
+0a9c). It wraps the object as 90x, reads typed group id, creator, creation time,
optional subject/value/time, announcement_version and total_participants_count,
then required participants. 90x.A0H is **creator/8yb**, A0I is
**participants/8ym through LbE.A05**, and A0J is subject/8yq through nullable
LbE.A04. LbE.A05 returns the nested model or throws; it is not a nullable
getter. A missing subject instead takes +03d4 and preserves null value; its
creation-time helper defaults through the null-string path.

### Variant branches

0Xo.A03 compares **String.hashCode of __typename**, with no corresponding
String.equals in this method. These are branch selectors, not discovered wire
spellings. After shared field reads, it has the following conversion paths:

| Hash bits | Wrapper | Required properties | Constructor integer slot |
| --- | --- | --- | --- |
| ec1046f2 | 2H8 | 2FS via A0H/LbE.A05 | 0 at v27 |
| b01c45b8 | 2H7 | 2FO via A0H/LbE.A05 | 2, or 6 if general_chat |
| 1550b5c7 | 2FI | 2FH via LbE.A05 | 3 at v61 |
| c3e26f86 | 2FK | 2FJ via LbE.A05 | 1 at v61 |

Other hashes throw UnsupportedOperationException at +0a8c. A4m's full
constructor/field mapping and wire spellings remain unverified here; the
integer slots above must not alone be treated as a completed group type port.
**Negative result:** exact Java-style hash comparison of all 453,954 unique
escaped DEX strings returned no matches for these four hashes; narrower Group/
Community/group/GROUP searches also returned none. These checks do not prove
server-returned names do not exist: escaped/UTF-16 representation and strings
assembled at runtime are outside that direct search. Do not invent names.

The 2H8/2H7 branches read support (2H8 only), locked, announcement,
growth_locked2.locked, ephemeral.expiration_time_in_sec, limit_sharing.
limit_sharing_enabled, membership_approval_mode_enabled, member_add_mode,
member_link_mode, member_share_group_history_mode, ack, allow_admin_reports,
group_safety_check, internal, appeal_status and optional appeal_update_time.
2H7 also selects general_chat and parses parent_group_jid through 0c6/1bo.A00,
plus hidden_group. The 2FI branch has growth_locked2/ephemeral/limit_sharing,
parent_group_jid, member_add_mode, ack, group_safety_check and appeal fields;
2FK has closed_by_membership_approval_mode, allow_non_admin_sub_group_creation
and appeal fields. Missing optional ephemeral selects literal zero expiration;
missing optional growth state retains null, not necessarily unlocked. Integer
and boolean coercion plus enum conversions remain unported. Exact resulting
flags and all A4m slots need constructor interpretation before state application.

All branches read missing_participant_identification. Seeing this flag causes
logging, not an early rejection in this method. ADv.A0D separately returns null
when participants_phash_match is true, rather than replacing membership with
an empty list. Full edge/role/participant-union conversion remains unfinished;
its source is preserved with this converter evidence, not inferred from counts.

### Time and creator helpers

ADv.A04 (group creation_time) and A05 (subject.creation_time) call 8kY.A0B.
It passes literal default zero to 0Gv.A01 then multiplies by **1000** using
mul-long/2addr. ADv.A03 announcement_version calls 0Gv.A01 directly, with no
multiplier. Thus these fields do not share identical numeric conversion.
0Gv.A01 checks TextUtils.isEmpty, returns the supplied default on that path,
and calls Long.parseLong otherwise; another return-default site exists.
The initial default dump did not identify its exception-table catch type;
the later explicit exception section resolves the NumberFormatException path. Android optString coercion,
Java signed-long/overflow behavior, resulting storage units and all constructor
slots are not ported by this arithmetic finding alone.

**Implemented tooling:** opcode bd now names mul-long/2addr, preserving the
existing 12x operand/width decoder. The literal eight-byte extent at
classes6.dex **0x26374c** (8kY.A0B+000c..0014) is independently read as
`16 00 e8 03 bd 02 10 02`: load 1000, multiply v2 by v0, return-wide v2.
The regression covers exact operands, boundaries and truncated prefixes.
This reports bytecode; it does not execute a timestamp converter. The initial search for CLI-style ??(0xbd) markers missed the bundle syntax
op=bd ??. The following full core-bundle audit found one such instruction
in noise-handshake; the later presentation audit below records its refresh.

Creator getter ADv.A09 follows 90x.A0H -> 8kU.A1A (nullable **id**) -> A0d;
1bo.A01 parses group id and requires subtype 1bn, throwing 08j otherwise.
0Xo.A01 and private A00 both consult feature **0x36fe**. Enabled A01 calls
ADv.A0H; disabled returns an empty map. Enabled A00 prefers creator.lid via
UserJid.Companion.A02, falls back to creator.id if needed, and requires parsed
creator.country_code as well before constructing a singleton CTQ metadata
map; otherwise it returns empty. ADv.A0H and full creator/JID/country parsing
remain unfinished. Feature-enabled identity metadata must not be silently
substituted by PN-only assumptions. No saved identity/key/group state changed,
account opened, MEX request sent or server response recognized live here.

## Result constructor and explicit exception inspection (2026-10-07)

**Confirmed static**, same APK 2.26.37.71/hash:
[selected constructor/catch source](mex-group-constructor-catch-evidence.txt)
resolves A4m's constructor writes. With 45 registers and 43 incoming register
words, this is v2 and arguments start at v3. Four longs consume two words each;
ignoring their widths would mislabel every later boolean. Cross-reading
0Xo.A03's ordinary-branch invoke range v11..v53 gives the following mapping:

| Constructor register | A4m field | Traced input |
| --- | --- | --- |
| v3 | A0C | A9r request context |
| v4 | A0D | typed group id |
| v5 | A0E | linked parent (null in ordinary branch) |
| v6 | A0F | creator user JID |
| v7 | A0G | description metadata from ADv.A0B |
| v8 | A0H | nullable growth_locked2 metadata |
| v9 | A0I | group_safety_check boxed boolean |
| v10 | A0J | nullable integer, passed null in ordinary branch |
| v11 | A0K | optional subject.value |
| v12 | A0O | ADv.A0D participant map (null on phash match) |
| v13 | A0L | ADv.A0F map, complete meaning untraced |
| v14 | A0N | feature-gated 0Xo.A01 map |
| v15 | A0M | feature-gated creator LID/id/country map |
| v16 | A07 | total_participants_count |
| v17 | A02 | ephemeral expiration |
| v18 | A03 | group type integer: 0 / 2 or 6 / 3 / 1 across branches |
| v19 | A00 | literal zero in ordinary branch, meaning untraced |
| v20 | A04 | converted member_add_mode |
| v21 | A05 | converted member_link_mode |
| v22 | A06 | converted member_share_group_history_mode |
| v23 | A01 | capi-derived integer |
| v24/v25 | A0A | creation_time parsed and multiplied by 1000 |
| v26/v27 | A0B | subject.creation_time parsed and multiplied by 1000 |
| v28/v29 | A08 | unscaled announcement_version |
| v30/v31 | A09 | optional appeal_update_time integer widened to long |
| v32 | A0S | support |
| v33 | A0b | locked |
| v34 | A0R | announcement |
| v35 | A0Z | ADv.A0J boolean, complete meaning untraced |
| v36 | A0a | limit_sharing_enabled |
| v37 | A0V | false ordinary; true 2FI branch; meaning untraced |
| v38 | A0W | membership_approval_mode_enabled |
| v39 | A0P | ack ordinary; literal true 2FK branch |
| v40 | A0Y | allow_admin_reports ordinary; non-admin subgroup creation 2FK |
| v41 | A0Q | literal false ordinary, meaning untraced |
| v42 | A0U | false ordinary; hidden_group in 2H7 |
| v43 | A0X | missing_participant_identification |
| v44 | A0T | internal ordinary/2H7; false other traced branches |

This confirms argument-slot provenance and group type field A03; it does not
complete every enum/coercion or downstream database write. Full application
0yw.A0X and alias/participant side effects remain unfinished. The type-name
hashes still have no established wire spelling; alternate branch flag reuse
must be interpreted per branch, not assumed identical to the ordinary case.

**Resolved parse failure:** 0Gv.A01's try_item covers code units 7..12
(byte offsets +000e..0018), with typed NumberFormatException handler at unit
12 (+0018). That instruction returns the supplied default v2; the traced
creation/subject/version callers supply zero. This is a typed catch, not
catch-all. Empty strings follow the separate TextUtils.isEmpty/default path.
[Literal code_item fixture](mex-long-parser-catch-fixture.json), independently
read from classes.dex offset 5390684, carries the real catch index/descriptor
and byte extent. Its regression reconstructs only the needed string/type
tables and proves range, handler and return-default bytes. Java Long.parseLong
lexical/coercion details remain unported; callers do not preserve malformed
numeric input as an unknown raw number.

LbE.A04 has four typed catches over code units 0c..21, all to unit 21:
IllegalAccessException, NoSuchMethodException, InstantiationException and
InvocationTargetException. Its handler stores the exception and returns null.
This closes the earlier broad statement about construction failure: arbitrary
Throwable is not caught locally by this table. 0Xo.A03 has no local try ranges;
this does not establish absence of caller-level exception handling.

**Implemented tooling:** `apk-smali --exceptions` prints each ordered typed
catch and final catch-all, with explicitly labeled code-unit half-open ranges
and targets, plus an explicit try-range count (including zero). Matching method metadata is checked whether or not the flag is
printed. Malformed padding/ranges/overlap/handler offsets/types/addresses,
truncation and signed/unsigned LEB overflow refuse without partial method
output. The prior scanner warned on a later DEX parse error and returned
earlier methods; it now propagates the named DEX error and prints no method
results. A two-DEX regression covers malformed bytes and invalid catch tables
after valid methods. No class hierarchy or instruction-boundary/whole-DEX verifier is
claimed. Existing default annotated output and versioned bundle format remain
unchanged. The format follows [AOSP DEX code_item and catch tables](https://source.android.com/docs/core/runtime/dex-format).

```sh
target/debug/xtask apk-smali /path/to/WhatsApp.apk \
  --class 'LX/0Gv;' --method A01 --exceptions
```

Synthetic regressions cover typed plus catch-all negative size, catch-all-only
zero size, odd instruction padding, code-unit addresses and malformed tables.
The actual APK fixture separately establishes this time parser's exception
path. No account opened, intent queued, group snapshot/identity overwritten or
live server response obtained by these offline inspections.

### Core bundle presentation audit

The strict scanner also rechecked all current 37.71 bundles. MEX and group
files reproduced unchanged. The core check exposed one stale annotation from
the preceding bd mnemonic fix: noise-handshake +045e..0460 now says
`op=bd mul-long/2addr v1, v10` instead of `op=bd ?? v1, v10`. Exact textual
replacement proves this is the only method-content change. The instruction
opcode, offsets and operands are unchanged; no handshake behavior is changed
or inferred here. The prior marker search used CLI spelling, whereas bundle
files have a separate op= column; that negative result did not establish full
bundle reproducibility.

[Cached rendering comparison](apk/core-mul-long-rendering-diff.json) against
old Git revision **34b5ab5** took **0.0328 seconds**: one changed topic, 21
unchanged topics, four unchanged metadata artifacts, same APK hash. Report
old/new paths refer to their respective Git revisions. Only noise-handshake
and its checksum were refreshed. The core now has 27 files/731,494 bytes,
12 bytes larger from naming the existing opcode. This is not a release or
protocol delta, and the older 37.70 evidence remains untouched.

## Numeric group projection into Rust (2026-10-07)

**Confirmed APK call chain**, same 2.26.37.71 SHA:
ADv.A04/A05 -> 8kY.A0B -> 0Gv.A01 parse/default-zero followed by signed-long
multiplication by 1000; ADv.A03 skips multiplication. A4m stores the results
separately in A0A, A0B and A08. The typed NumberFormatException/default return
is independently preserved in the earlier raw code_item fixture. The new
`MexJsonResponse::group_numeric_metadata()` ports these three fields as
`GroupNumericMetadata`: creation_scaled, subject_creation_scaled and
announcement_version, all i64. Names do not assert downstream storage units.

**Implemented, offline verified:** the method first requires the error-free
nonempty callback disposition and an object xwa2_group_query_by_id. Missing/
nonobject group data refuses with a named diagnostic. Subject is a nullable
object through 90x.A0J/LbE.A04; missing/null/nonobject subject gives zero time.
It does not validate group id, __typename/properties, membership, feature gate
or every converter field. Numeric projection is not full group validity and
must not be applied as a complete saved GroupSnapshot.

Numeric strings use signed decimal parsing without trimming, radix prefixes,
underscores, exponents or fractions. Optional ASCII signs and BMP decimal
digits are supported; UTF-16 surrogate units reject supplementary digit
scalars. Overflow/invalid/empty input returns zero. Multiplication uses explicit
wrapping signed-long arithmetic, not saturating or Rust debug panic behavior.
In particular, MAX_VALUE times 1000 is -1000 and MIN_VALUE times 1000 is zero;
announcement_version retains its unscaled signed value. Missing/null/boolean/
array/object values also produce zero. Signed integral JSON values retain
precision; floating/wider-unsigned numeric values produce zero rather than
being truncated into integers. The APK uses optString followed by parseLong,
not optLong; those conversions are not interchangeable.

**Platform reference / inferred coercion, not Android execution:** checked
[AOSP JSONObject.optString](https://android.googlesource.com/platform/libcore/+/refs/heads/main/json/src/main/java/org/json/JSONObject.java)
and [JSON.toString](https://android.googlesource.com/platform/libcore/+/refs/heads/main/json/src/main/java/org/json/JSON.java)
show string conversion via String.valueOf for other nonnull objects. Combined
with Java numeric string formatting and the APK's parseLong/default-zero call,
this supports the scalar results above. Full Android JSONTokener lexical
acceptance is not reproduced by strict serde_json; its runtime was not
executed here. For string parsing, the
[Java Long API](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Long.html)
and [Android Character API](https://developer.android.com/reference/java/lang/Character)
provide the reference semantics. The exact Android SDK/Unicode table used by
a live peer/build remains unverified, especially for non-ASCII input.

[Independent Java oracle](MexJavaNumericOracle.java) was executed on host
Temurin **21.0.12.1+1 LTS**, not on an Android device or inside the APK. It calls
actual Long.parseLong and long multiplication. Its
[479-case output](../../crates/whatsapp-api/fixtures/mex/2.26.37.71-numeric-reference.json)
covers signs, leading zeroes, boundaries, overflow, whitespace, punctuation,
BMP/supplementary digits and all 370 recognized BMP digits across 37 blocks.
The oracle enumerates all 65,536 UTF-16 units; Rust checks the full domain
against the resulting independent digit-block table. This is host-reference
verification, not observed live Android/client equivalence. Values and tests
are synthetic and contain no account data.

To reproduce the reference digit enumeration (with a local JDK):

```sh
javac -d /tmp/mex-java-reference docs/protocol/MexJavaNumericOracle.java
java -cp /tmp/mex-java-reference MexJavaNumericOracle digits
```

Without digits mode the oracle reads one Base64-encoded UTF-8 synthetic string
per input line and prints parsed/default-zero and wrapped-times-1000 decimal
values separated by a tab. The retained source prevents expectations from
being regenerated from the Rust implementation itself.

**CLI:** default `mex-response-inspect` and --group-info still print redacted
counts/gate only. New **--group-numbers requires --group-info** and explicitly
opts into private numeric content. It adds group_numeric_metadata with the
three **signed decimal strings**, preserving values beyond JSON consumers'
53-bit numeric precision. This flag must not be used as a redacted diagnostic
for real private group data. Library numeric Debug is redacted; diagnostics
never include input values or server error text. Blocked callback/wrong group
shape refuses before any partial stdout.

```sh
target/debug/xtask mex-response-inspect --input /private/mex-result.json \
  --group-info --group-numbers
```

No HTTP route, account access, action settlement, retry, live MEX dispatch,
group-state mutation or feature selection is added. Full converter/state
application remains unfinished; this ports its confirmed numeric component.

## Participant role conversion and member-map boundaries (2026-10-07)

**Confirmed static, APK 2.26.37.71 / SHA above:** selected instructions are
preserved in [participant/role evidence](mex-participant-role-evidence.txt),
with the two consuming branches in
[ADv.A0D](mex-group-converter-evidence.txt).
`2hu.<clinit>` defines ordinal 0 `UNSET_OR_UNRECOGNIZED_ENUM_VALUE` (A05),
1 `ADMIN_MEMBER` (A02), 2 `MEMBER` (A03), 3 `SUPERADMIN_MEMBER` (A04).
`LbE.A0C` reads `JSONObject.optString`, then `4R7.A00` uppercases using
`Locale.US` and calls `Enum.valueOf`. Null/empty returns the supplied default;
IllegalArgumentException at code units 0008..0017 goes to default at 0017.
There is **no trimming**. `3c5.A0C` returns ordinal. In both ADv.A0D branches,
ordinal 1 becomes `admin`, 3 `superadmin`, and 0/2 an empty string. In particular
+00ce's ordinal-2 branch skips the assignments; treating ordinal 2 as admin
would grant the wrong role.

**Implemented:** `GroupParticipantRole::from_server_value(Option<&str>)`
and `member_role()` preserve four enum states and the exact stored strings.
Unknown and ordinary members stay distinguishable. This is a nullable-string
conversion, not a port of arbitrary JSONObject.optString value coercion.
It will be reused by the full converter; no existing membership is mutated.
Inspect the actual Rust helper without account access:

```sh
cargo run --locked -p xtask -- mex-role-inspect --value admin_member
cargo run --locked -p xtask -- mex-role-inspect
```

Output is `offline_only`, recognized `role`, and `member_role`; supplied text
is not echoed. Missing, empty and unknown resolve to `unrecognized`.

**Executed host reference / inferred platform compatibility:** the
[Java oracle](MexJavaNumericOracle.java) `roles` mode uses APK-derived enum
names and ordinals but executes Java Locale.US/Enum.valueOf independently of
Rust. [91 cases](../../crates/whatsapp-api/fixtures/mex/2.26.37.71-role-reference.json)
include mixed/per-letter casing, missing-string equivalent empty, padding,
control characters, unknown values, dotless i, long s, dotted capital I,
sharp s, fullwidth letters and supplementary characters. Actual host runtime
is Temurin **21.0.12.1+1 LTS**. Rust's Unicode uppercase matches this corpus.
This is not Android/APK execution; complete Android SDK/Unicode parity remains
**unknown**. Regenerate with javac, then Base64 UTF-8 inputs one per line:
`java -cp <compiled-directory> MexJavaNumericOracle roles`.

**Confirmed guard, not yet implemented:** ADv.A0D returns null immediately
when `participants_phash_match` is true. Returning null rather than an empty
replacement is confirmed. Retaining existing members is the inferred intent;
0yw.A0X downstream application remains untraced. `LbE.A0G` derives a default from JSONObject.optInt via
`Ayj.A1R` (**integer exactly 1**, not any nonzero value), then calls optBoolean
with that default. Full Android integer/boolean coercion remains **unknown in
our Rust port**; a simplistic nonzero rule would contradict the APK.

**Confirmed participant routing:** ADv.A0D iterates edges, skips missing
participant models and unknown typename hashes, then uses two branches:
`0xc745b457` reads `jid` through nullable `0hj.A03`, whose strict parser
`0hj.A00` requires BotJid (`0hh`). `0x2179873b` reads `id` through strict
`0D8.A01` requiring UserJid; it also carries optional lid, pn and display_name.
Role is read from the **edge**, not the nested participant. Positive join_time
and nullable group_history_sent are carried. LinkedHashMap.put means duplicate
primary JIDs replace values while retaining insertion position. All 453,954
escaped DEX strings were previously searched and none matched either hash;
wire typename strings remain **unknown**, not invented from class names.

`0hh.getServer` returns **bot**, `getType` returns 26. Its constructor accepts
1..20 ASCII digits with **no leading zero**. This is distinct from the Unicode
decimal-digit support of the numeric metadata helper. Generic `0D1.A00`
recognizes the bot domain by hash then exact string equality, and also has a
bot device-wrapper branch (`Ble`); its complete agent/device semantics remain
unported. Existing Rust JID modeling is unchanged in this role chunk.
`32q.A00` filters optional alias objects to their concrete LID/phone/interop
classes before passing reordered arguments to `31c.<init>`. The constructor's
field mapping, edge-list helper, history coercion, alias feature maps, full
participant converter and durable application remain **unknown/unimplemented**.
No alias observations or identity pins may be manufactured from this trace.
No saved account was opened and no live request was sent.

## Exception-aware numeric and participant release coverage (2026-10-07)

**Confirmed static / reproducible tooling**, same 37.71 SHA: the
[recipe](apk/recipes/2.26.37.71-mex.json) adds twenty selected methods to the
previous eighteen. This includes the full 0Xo.A03 and ADv.A0D converters,
ADv.A03/A04/A05 numbers, 0Gv.A01 and 8kY.A0B numeric helpers, 2hu enum,
LbE.A0C/4R7.A00 role lookup, LbE.A0G/Ayj.A1R phash boolean, BotJid constructor
and domain, nullable/strict bot and strict user parsers, 32q alias builder,
A4m result constructor and 0D1 generic JID routing. These are source topics,
not a claim that their complete Rust ports exist. All topics point to the
Rust MEX implementation owner, including currently unimplemented prerequisites.

The recipe's optional root `include_exception_tables` boolean defaults to
false. When true, each method records ordered typed/catch-all handlers with
half-open **code-unit** ranges and code-unit targets, plus explicit zero counts.
Instruction offsets remain bytes. Targets metadata records the option as true;
resolved remap proposals retain the recipe setting. Invalid nonboolean values
refuse. Previously exported instruction-only formats stay reproducible.
This closes a concrete diff blind spot: unchanged instructions can have a
changed catch type or destination. A regression based on the independent
0Gv code_item fixture changes only its catch type to catch-all and verifies the
cached diff reports the owning topic changed.

Current baseline **43 files/162,435 bytes**, all **38** methods. The
[cached coverage diff](apk/mex-participant-coverage-diff.json) took
**0.023364 seconds**, one actual command with no APK scanning. Old baseline
is Git **78a7218**; old_file paths refer to that revision. Twenty topics added;
the old eighteen changed only because exception annotations were enabled.
Every prior instruction text was independently compared after stripping
`# exception table` / `# catch` lines and is identical. Inventory, manifest
and tokens unchanged; targets metadata changed. This is same-APK coverage
and formatting, not a new release or demonstrated behavioral change.
APK re-export still requires scanning and is not claimed subsecond.

**Confirmed relocation limitations:** [actual remapping](apk/remap-evidence/2.26.37.71-mex.json)
finds fourteen unique, seventeen unanchored, seven ambiguous. `creation_time`
with shorty JL still matches **both ADv.A04 and A05**; the locator cannot tell
creation from subject time from this anchor. `invalid bot jid: ` with LL
matches **0hj.A00 and A01**. `bot` with L matches **0hh.getServer,
Ble.getRawString, Ble.getServer, CSe.A00**. These are negative results to
preserve, not permission to pick the old class. String-free helpers require
caller-based/manual evidence. Full proposed recipe remains unresolved.

**Confirmed newly visible catch detail:** nullable `0hj.A03` catches **all**
from code units 0009..000e at 000e, wraps failure as 0Zd and returns null;
strict 0hj.A00 does not have that local fallback. `0D1.A00` has a typed
NumberFormatException handler at units 0106..010e -> 0191 in device parsing.
Complete device semantics and exception propagation beyond these local
handlers remain **unknown/unported**. Numeric/role typed catches retain their
previously documented behavior. No account/state access or live request.

Use the existing `apk-research-bundle --recipe ... --check` and
`apk-research-diff <old-cache> <new-cache>` commands above. Compare fully
checksum-verified snapshots; review annotation/scope changes separately from
protocol changes before applying a Rust patch.
