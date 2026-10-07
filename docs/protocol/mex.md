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

[The MEX release recipe](apk/recipes/2.26.37.71-mex.json) exports twelve
method topics to [the 37.71 baseline](apk/2.26.37.71-mex/manifest.json):
17 files, 58,761 bytes, reproduced exactly with `--check`. Each topic names
`crates/whatsapp-api/src/transport/mex.rs` as its Rust owner. The baseline is
additional source coverage, not a new APK release or full MEX parity.
A cached same-baseline comparison took **0.0064 seconds**, twelve unchanged
topics and four unchanged artifacts; this measures cached comparison only.

```sh
target/debug/xtask apk-research-bundle /path/to/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-mex.json \
  --output lab/releases/mex --check

# After re-anchoring and exporting the next actual APK:
target/debug/xtask apk-research-diff \
  docs/protocol/apk/2.26.37.71-mex lab/releases/next-mex
```

Use `--check` only with an existing complete export; for a new output directory,
omit it. Recipe relocation on this APK has six unique, four unanchored and two
ambiguous topics ([actual remap report](apk/remap-evidence/2.26.37.71-mex.json)).
Common `doc_id` and `subject` anchors remain ambiguous despite method shorties;
string-free constructors/provider/storage helpers remain unanchored. Those
require caller-based/manual review on a new APK. A full proposed recipe must
remain blocked until every mapping is resolved; class names alone are not a
safe patch. Selected persisted-ID asset files remain separately diffable.
