# Read receipt preference synchronization

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Ten-topic recipe](apk/recipes/2.26.37.71-read-setting-sync.json),
[signatures, operands and exception tables](apk/2.26.37.71-read-setting-sync/targets.json).
Offsets below are method-relative bytes. This narrows the writer/synchronization
unknown in [factory read policy](receipt-read-policy.md); it does not establish
our saved account's effective preference or finish primary identity cleanup.

## Discriminator 227 preserves a local disabled preference

**Confirmed:** 14c.BFC(Message,int) tests its supplied second argument for 227
(0026..002e), casts Message.obj to 0hs and reads its `readreceipts` attribute.
Exact String.equals comparisons accept `all` as true and `none` as false.
Missing/other values log an unknown-setting diagnostic and return handled true
(0072..00c4), without writing the preference. This is not a wire ACK.

08l.A1H reads `read_receipts_enabled` with default true. The handler compares
that local value with the received Boolean (00c4..00e4). Equal values return
handled true at 0388. Mismatches log `readReceipt-setting-inconsistency`, then:

| Existing local value | Received value | Selected effect |
| --- | --- | --- |
| true | none | Write false through 08l.A0Q/0FU.A01, apply, notify 0sv.A0Q |
| false | all | Call 0sv.A0P(`readreceipts`,`none`); retain local false |

The first branch is 012c..0188, the second 0188..01a4. Because only unequal
values reach these branches, the local-true branch cannot adopt true here.
A generic “server setting wins” implementation would incorrectly enable the
second case. The producer/wire origin of discriminator 227 remains **unknown**
in this trace; do not treat every privacy category notification as this event.

**Confirmed negative:** 0sv.A0Q(List) constructs IUu(list,17) and calls observer
dispatch 078.A00. Its selected body does not construct a network query. Transitive
observer effects remain **unknown**, so this does not prove no later fetch.

## Category maps have a different adoption path

**Confirmed:** 0sv.A0R(Map) validates each value using I4X.A00 and then
I4X.A04(category,code,accountBoolean,accountBoolean). Invalid/unsupported values
are skipped (0076..00b4). A pending Hj6 in A0E carries the requested value:
a different received value skips that entry, preserving the pending request
(00b4..00c8). A matching value removes the pending entry before category dispatch.
With no pending entry, the admitted category still reaches dispatch.

For `readreceipts`, 0238..0276 converts through I4X.A03 and writes/applies the
Boolean preference. A matched pending entry also emits IUr(category,value,4)
through 078.A00 (0276..0294). This differs from discriminator 227: an admitted
`all` map entry can enable the preference when no contradictory request is pending.
Do not apply the discriminator's local-off rule to every map.

**Confirmed:** I4X.A00 uses exact String.equals; `all` maps to 0, `none` to 2,
`on_standard` to 8. Those codes pass I4X.A04 for `readreceipts`, independent of
its two account Boolean arguments. I4X.A03 compares via 0CL.A0E against `all`
and `on_standard`; these admitted values produce true, and `none` produces false.
Other known category values and validations are retained in the full I4X dump;
they are not advertised as supported read-setting options. 0CL.A0E's behavior
on arbitrary, unvalidated strings remains **unknown** here. No unknown string
is treated as an explicit disable by the map's initial validator.

**Confirmed:** 17V.A09 obtains the envelope's first child and branches on its
tag `privacy`. `action=modify` takes a separate path. Otherwise it requires
nonempty `category` children, reads required `name`/`value` attributes, builds a
HashMap, and calls A0R (08d6..09ac). Duplicate names replace earlier map entries.
Dey.A01 also calls A0R, then conditionally reports completion through J80.AGE
(0018..004a). Completion does not prove every supplied category was adopted.
Full callback payload production, malformed-input behavior, observer lifecycle
and ACK/correlation authority remain **unknown** in this bounded trace.

## Outbound construction and UI boundaries

**Confirmed:** 0sv.A0P first saves Hj6(value) under the category in A0E
(0000..0020). Property 23639 selects a MEX branch versus the SMAX branch.
The MEX branch constructs Dey/Fw2 and calls 0tC.A02; its query ID/body and
current effective property remain **unknown**.

The SMAX branch builds `<iq type="set" xmlns="privacy" id="generated"
to="server"><privacy><category name="category" value="value"/></privacy></iq>`
(0100..01cc). The destination is 1S7.A00, not a peer. It dispatches through
0h8.A0T with request code 69 and long argument zero (01cc..01da). Zero here
does not establish an infinite timeout, no retry, or a server-directed wait.
Dispatch result/timeout handling and pending-state durability remain **unknown**.
This is source authority for the local-off correction request, not authorization
to send a request or add a settings mutation command now.

**Confirmed scoped boundaries:** SettingsPrivacy.C62 maps the local key
`read_receipts_enabled` to category `readreceipts`, converts an integer selection
through I4X.A02, then calls 0sv.A0S. A0S's full setter behavior is untraced here.
EcE.A01's relevant branch opens SettingsPrivacy with target_setting set to the
local key; that branch is navigation, not a preference write.

## Search inventory, limits and reproduction

The [exact-literal/physical-call inventory](read-setting-sync-inventory.txt)
records 23 references to `read_receipts_enabled`: seven state/UI references
and sixteen telemetry-named getFieldsMapForFalco methods. The inventory does
not exclude indirect writers, aliases or native code. SettingsPrivacy.A0a and
telemetry bodies were not traced in this change. There are two exact physical
A0R callsites, in 17V.A09 and Dey.A01; reflection/virtual reachability is not
proved by this count. All unrelated handler branches remain uninterpreted.

**Unknown:** effective private-account setting, bootstrap/query authority,
preference-store/account selection, complete synchronization lifecycle, and
full factory classifier/user-row authority. Rust does not infer true from the
APK default or false from an absent server value. No runtime, private state,
request ID or live request changed. No HTTP settings route was added.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-read-setting-sync.json \
  --output docs/protocol/apk/2.26.37.71-read-setting-sync --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-read-setting-sync \
  docs/protocol/apk/2.26.37.71-read-setting-sync
```

Verification for this source-only chunk: exact bundle regeneration matched all
15 files (179,869 bytes); cached self-diff reports ten unchanged topics and four
unchanged metadata artifacts. CLI reference regeneration/coverage and OpenAPI
library-symbol coverage passed. Formatting, whitespace and repository hygiene
passed (1,838 local links; 77 checksum bundles). No Rust behavior changed, so
workspace runtime tests were not repeated for this evidence-only change.
