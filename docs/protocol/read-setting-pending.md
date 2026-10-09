# Privacy pending requests, delayed clear and display state

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eight-topic recipe](apk/recipes/2.26.37.71-read-setting-pending.json),
[source signatures, operands and catches](apk/2.26.37.71-read-setting-pending/targets.json),
[physical field-reference inventory](read-setting-pending-inventory.txt).
This closes the immediate lazy-runnable boundary in
[fetch/callback evidence](read-setting-bootstrap.md), not a Rust settings port.
Offsets below are method-relative bytes. Unrelated lambda/UI branches are not
interpreted merely because their selected methods are retained.

## One delayed clear for the manager's pending map

**Confirmed:** 0sv's retained constructor supplies itself to 1Xt selector 40
and stores the lazy value in A0F. 1Xt.invoke case 40 (050c..0524) constructs
IhA(manager,14). This binds the actual runnable rather than inferring its role
from a 20-second constant.

IhA.run case 14 (0274..02bc) casts the captured object to 0sv, obtains A0E,
and checks isEmpty. An already empty map returns without notifying or showing
its diagnostic. Otherwise it **clears the entire map**, calls A0Q with the
empty list 01f.A00, and invokes 0Jr.A0A with resource 0x7f121975 and argument 1.
No resource-message wording is guessed from that integer.

**Confirmed scoped negative:** the selected branch does not apply a preference,
call A0P/A0S, manufacture a success response, or resend a request. A0Q remains
observer dispatch; its transitive effects/registration remain **unknown**.
Clearing pending state is not evidence of server failure, success, or safe replay.
This concerns in-memory UI settings operations, not registration reservations,
message request IDs, account identities or server waits.

0GS's constructor uses a Handler on Looper.getMainLooper. A01 calls
Handler.removeCallbacks(runnable). A02 invokes 0GU.CGS then
Handler.postDelayed(runnable,delay); it discards the Boolean result.
The prior A0S body removes this shared runnable and posts it again after
**20,000 milliseconds** after local submission. Thus the selected scheduling
boundary resets one manager-wide delayed clear, not a separate timer per
category. Actual firing time, Looper/process lifetime, CGS behavior and races
with callback adoption remain **unknown**; it is not a server-directed wait.
No executor ordering or network retry guarantee is inferred.

## Pending values are not committed preferences

**Confirmed:** A0P stores a pending Hj6(value) in A0E before dispatch. A0R
validates values and skips a received value differing from the current pending
value; a matching admitted value removes that category's entry before writing
its preference. Those [retained branches](read-setting-sync.md#category-maps-have-a-different-adoption-path)
and the runnable explain three different states: pending intent, committed
preference, and expired local pending marker. They are not interchangeable.

**Inferred narrowly:** once the delayed clear has removed an entry, A0R's
pending-value mismatch guard no longer rejects a subsequently admitted map value
for that category. This follows from its null-entry branch, not an observed late
server response. Full IQ correlation, stale response delivery and whether a
specific late reply executes A0R remain **unknown**. Do not implement durable
request replay or rotate request IDs from this observation.

**Confirmed:** SettingsPrivacy.A5J retrieves A0E[`readreceipts`]. A present
Hj6 supplies the displayed Boolean through I4X.A03; absent pending state reads
08l.A1H (0060..0084, 0114..0122). A present pending entry shows the progress bar,
hides the switch and skips setChecked; an absent entry hides the bar, shows the
switch and sets its checked value (0098..00c8). These display operations do not
establish a committed global read Boolean. Row enablement delegates to
1xi.A1V, whose precise predicate is **unknown** here.

H8g.A5M uses a similar pending display path for the separate `groupcreation`
category, with a stored 0SW-backed fallback. It is not the global read preference.
IZB.BP5 returns whether the selected descriptor category has a nonnull A0E
entry; it does not test an acknowledged outcome. GAZ.A0g obtains the manager
from a provider and returns its pending entry for the supplied key.

0sv.A0L's integer getter reads a category preference/default, then replaces its
returned integer with I4X.A00(pending.value) when an entry exists, and clamps
the result to at least zero (0026..0050). This does not commit that pending
value. Its other category branches and absence of a Boolean return must not be
used to infer the effective read Boolean or to replace 08l.A1H.

## Scope of the map inventory

The exact field scan finds **nine** references to 0sv.A0E: one constructor
field write, three manager reads (A0L/A0P/A0R), four helper/UI reads
(GAZ.A0g, H8g.A5M, SettingsPrivacy.A5J, IZB.BP5), and IhA.run's read.
Field reads can mutate the referenced collection: A0P puts, A0R removes and
IhA clears, so a single field-write count is not a mutation count. The four
examined helper/UI access sites use get and do not directly mutate the map.
This physical scan does not exclude indirect aliases, reflection or native
access. A synchronized map does not prove compound read/validate/remove/write
operations are atomic; full threading/locking authority remains **unknown**.

Effective saved-account preference, fetch/set mode properties, observer
registration/side effects and complete bootstrap remain **unknown**. Native
receipt_user admission, primary identity cleanup and final resend remain
unfinished. No runtime/private state/key/request ID/server wait or live request
changed. Existing offline tools expose reproduction; no HTTP setting route or
live mutation capability was added.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-read-setting-pending.json \
  --output docs/protocol/apk/2.26.37.71-read-setting-pending --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-read-setting-pending \
  docs/protocol/apk/2.26.37.71-read-setting-pending
```

Verification: exact regeneration matched 13 files / 141,743 bytes; cached
self-diff reports eight unchanged topics and four unchanged metadata artifacts.
CLI-reference regeneration/coverage, OpenAPI library-symbol coverage, formatting,
whitespace and repository hygiene passed (1,865 local links; 80 checksum bundles).
No Rust behavior changed, so workspace runtime tests were not repeated and these
static checks do not establish live settings or primary-cleanup parity.
