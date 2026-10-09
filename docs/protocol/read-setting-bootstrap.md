# Privacy setting fetch, callback and observer boundaries

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[13-topic recipe](apk/recipes/2.26.37.71-read-setting-bootstrap.json),
[source signatures, operands and catches](apk/2.26.37.71-read-setting-bootstrap/targets.json).
This extends [read-setting synchronization](read-setting-sync.md), not a claim
that bootstrap runs on every login or that Rust now adopts privacy settings.
Offsets are method-relative bytes. Unrelated lambda branches are retained but
not interpreted; their retention is not a complete semantic port.

## Fetch selection differs from set selection

**Confirmed:** 0sv.A0M reads integer property **23463** (0x5ba7) through
00D.A0b (0000..001c). It chooses:

| Returned integer | Selected fetch path |
| --- | --- |
| 1 | MEX coroutine FwT with selector 24 and a Dey completion callback |
| 2 | Shadow FwP selector 16 plus SMAX A02 with LNQ completion object and true flag |
| Other | SMAX A02 with null completion object and false flag |

Source branches are 0022..0060, 0060..00dc and 00dc..0154. The shadow path
constructs both operations; execution/comparison ordering inside its coroutine
and LNQ remains **unknown**. This is not permission to choose whichever result
is convenient. Set requests use a different Boolean property **23639** in A0P.
Their effective values, constructor/default authority and native configuration
resolution remain **unknown** here; an observed set mode would not prove fetch mode.

**Confirmed:** private 0sv.A02 constructs Dey and FdY, generates a request ID,
and builds an IQ to 1S7.A00 with type=get, xmlns=privacy and an empty privacy
child (0032..00c0). It dispatches 0h8.A0T with request code **70** and long
argument **0** (00c0..00ce). No per-category request or guessed read-setting
value appears in this body. The zero argument is not a server wait or proof
of retry/timeout semantics. Full callback correlation and delivery lifecycle
remain **unknown**, not established by constructing FdY.

## SMAX result parsing precedes preference adoption

**Confirmed:** FdY.C8f obtains the response's first child and invokes the
privacy-tag check (0008..0020). No children produces an empty map forwarded
to Dey.A01. With children, it enumerates indices, invokes Fsf selector **27**
to obtain each child, then Fsa selector **45**, and builds a map before Dey.A01.

Fsf.invoke case 27 at 145a..1478 indexes the original node's child array.
Fsa.invoke case 45 starts at 0018, invokes the category-tag check and reads
nullable `name` and `value`. Missing either returns null. A normal value produces
07o(name,value) at 0708..0712. A value equal to `error` instead reads the first
child, invokes the error-tag check and uses that child's nullable `code` as the
pair value; missing code returns null (0052..0088). This is not adoption of the
literal setting `error` and not an invented global false value.
Full sequence/map null handling and malformed-node check failure behavior remain
**unknown** here. The prior [A0R validator/pending guard](read-setting-sync.md#category-maps-have-a-different-adoption-path)
remains the preference-write authority after Dey.A01; a map/callback does not
prove every returned setting was adopted.

FdY.BmU examines error children, takes the first with a nonnull code, converts
through 0CK.A06 and passes the nullable Integer to Dey.A00. If none has a code,
it passes null (0008..0068). Dey.A00 only conditionally reports FGX(code,null)
through J80.AGE when its completion flag/object permit. FdY.BjL similarly
reports FGX(null,null) on delivery failure, conditionally. These selected bodies
do not update a preference, clear A0E pending entries or transmit retries.
That scoped negative does not establish their absence in transitive callbacks.

## Actual callers and local setter negative result

The [physical call inventory](read-setting-bootstrap-inventory.txt) finds six
A0M callsites: 6Aa.invokeSuspend, GL3 constructor, SettingsPrivacy.onCreate,
Igm.run, IhS.run and 1Tc.A01. Only the last caller is interpreted here.
1Tc.A01's AccountSyncManager/fetchAllAccountInfo path reads request object
1Td.A06; a separate A00(manager,3,time,force) admission determines its local
v22. When admitted it records a time in A0E and calls A0M (0568..0590).
This narrows scheduling to an account-sync request/admission, **not** every
login. Effective request flags, A00's interval/retry authority and the other
five caller workflows remain **unknown**.

**Confirmed negative:** 0sv.A0O(String,int) is an integer local setter, not a
fetch builder. It explicitly refuses Boolean-backed `readreceipts` and `defense`
with a diagnostic (0000..0020 -> 0076..0096). Other categories validate, write
an integer preference and emit observers. Do not use it to set or initialize
the global read Boolean. 0sv.A0Q remains observer dispatch, not a direct query.

**Confirmed:** 0sv.A0S gates on 079.A0T; false shows its local diagnostic/UI
resource and returns false. True calls A0P, cancels a runnable through 0GS.A01,
schedules that runnable with argument **20,000 ms** through 0GS.A02 and returns
true (002c..0064). True is local submission, not server acknowledgement.
The runnable's effect and the predicate's precise meaning remain **unknown**;
20 seconds is not a server cooldown or established network timeout. The manager
constructor creates synchronized HashMap A0E and lazy runnable supplier 1Xt
selector 40 (005c..0074, 0114..012e). Pending state in this constructor is
in memory, not proof of durable recovery across process restart.

## Read-setting observers can schedule receipt work

**Confirmed scoped branch:** DK9.Bsa returns unless category equals
`readreceipts` and value differs from exact `none` (0010..0030).
It calls 0zH.A0X with a shared helper result; that method's downstream effects
and this helper's Boolean meaning were not traced here. It then queries
`played_self_receipt`, ordered by message_row_id descending, limited to **4096**,
resolves saved destination/optional participant and builds CPp records.
No message body or new message ID is fabricated by this query.

It passes those records through DK9.A00 (untraced), enqueues
SendPlayedReceiptJobV2(record,**false**) through 0ls.A01 (0172..01d6), then
collects selected row IDs and deletes matching played_self_receipt rows in a
local transaction (01d6..02e8). Queueing precedes that deletion; full queue/store
atomicity and actual job transmission remain **unknown**. The query limit is
an APK local batch limit, not a WhatsApp rate limit. Observer registration and
which IUr events invoke Bsa remain **unknown**: this does not assert that every
server enable invokes this branch. It does establish why adopting settings
and importing historical receipt rows can have outbound consequences.

## MEX translation boundary

**Confirmed:** MexSetPrivacySettingsHandler.A01 maps `readreceipts` to EiU.A0J,
`all` to EiS.A02 and `none` to EiS.A09. Unknown category/value returns a failure
object before constructing FwQ. Known inputs enter a coroutine; its payload,
query ID, response translation and wire enum spelling remain **unknown**.
FCL.A01 separately returns `readreceipts` for feature ordinal 18, and null with
a diagnostic for unsupported features. Java enum field names/ordinal 18 are
not protobuf field numbers or guaranteed wire enum values. This does not finish
the MEX fetch/set protocol.

**Remaining authority:** full bootstrap scheduling, effective properties,
private account preference, observer registration/side effects and transactional
receipt-user admission. Primary identity cleanup and final resend remain
unported. No runtime, account, key, request ID, server wait or live request changed.
Existing APK research commands expose reproduction; no HTTP settings route or
live CLI setting mutation capability was added.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-read-setting-bootstrap.json \
  --output docs/protocol/apk/2.26.37.71-read-setting-bootstrap --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-read-setting-bootstrap \
  docs/protocol/apk/2.26.37.71-read-setting-bootstrap
```

Verification: exact regeneration matched 18 files / 273,498 bytes. Cached
self-diff reports 13 unchanged topics and four unchanged metadata artifacts.
CLI-reference regeneration/coverage, OpenAPI library-symbol coverage, formatting,
whitespace and repository hygiene passed (1,856 local links; 79 checksum bundles).
This is source/documentation work; workspace runtime tests were not repeated,
and no live parity is claimed from these checks.
