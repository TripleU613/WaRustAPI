# Self-read virtual eligibility and an exact override census

2026-10-11, **confirmed static / implemented analysis tooling**, APK
**2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The new `xtask apk-virtual-overrides` command resolves exact declared instance
methods through all shipped DEX superclass links. It closes the virtual A0l
boundary in the [read submission trace](read-receipt-submission.md), without
claiming complete self-read message-state application.

## Predicate results and their actual scope

The previously retained [0zd.A09](apk/2.26.37.71-read-receipt-submission/individual-admission.txt)
requires message A0F at least **1415214000000**, a true virtual `178.A0l()` and
`0zd.A07(chat)`. The [Cmn.A04 receipt worker](apk/2.26.37.71-group-receipts/receipt-native-application.txt)
uses this helper in its status-17 branch at `0608..0668`, alongside chat/message
predicates. That branch collects qualifying messages into a separate pool; its
helper alone does not prove final receipt rejection, a recipient read fact, or
that every common message-state/observer path is skipped on false.

The [complete REA base method](rea/2.26.37.71-self-read-eligibility/message-eligibility.json)
returns true. The [complete structural census](rea/2.26.37.71-self-read-eligibility/virtual-eligibility.txt)
finds these **four declarations** across **101,453 physical DEX class definitions**:

| Declaring class | Direct superclass | A0l()Z |
| --- | --- | --- |
| 178 | java.lang.Object | true |
| 1iZ | 178 | false |
| 1jt | 178 | false |
| Bpp | 1ik | false |

The census also retains each ancestry chain, including Bpp → 1ik → 178, exact
DEX signature, complete selected instructions and exception tables. A0l's base
body cannot stand in for every subtype; inherited subtype behavior follows its
nearest declaration. These class names are not a complete mapping from public
Rust message kinds or protobuf fields to native runtime classes.

The [complete 0zd.A07 body](rea/2.26.37.71-self-read-eligibility/chat-eligibility.json)
allows its three special chat predicates, or requires the held C08l.A1H value
and both A00/A01 chat checks. A07 is therefore not an unconditional ordinary-chat
allowance. [0DE.A0m](rea/2.26.37.71-self-read-eligibility/special-a0m.json)
excludes A0W and otherwise recognizes JID types 1/2; the independent
[A0i predicate](rea/2.26.37.71-self-read-eligibility/special-a0i.json) recognizes
type 5. The [existing exact parser](rea/2.26.37.71-receipt-list-shape/parser-exact.txt)
uses those predicates for chat selection. Numeric type checks are not assumed
equivalent to arbitrary local server strings.

## Tooling and update workflow

```sh
cargo run -p xtask -- apk-virtual-overrides /path/to/WhatsApp.apk \
  --class 'LX/178;' --method A0l --signature '()Z' > eligibility.txt
```

The command selects the exact full parameter/return descriptor, reads direct
and virtual method tables separately and follows superclass links even when an
intermediate class declares no method. It emits the root and declared overrides,
with the existing checked instruction renderer. Unrelated classes, overloads,
name-prefix matches, static/private and nonvirtual declarations are excluded.
Package-private roots, constructors, missing/ambiguous declarations, superclass
cycles and attempts to override a final root refuse. Malformed later DEX files
cannot emit a partial census. The shared smali reader also now refuses malformed
class/method references, truncated class data and unavailable instruction bodies
instead of silently omitting them.

After [reviewed topic remapping](apk-file-diff.md), run the same command with the
new release's confirmed symbols and compare retained outputs. Reusing an old
obfuscated class name without remapping is not version authority. Interface
implementations, external classes, dynamic loaders and actual runtime dispatch
remain outside this superclass census. APK signatures are not verified by it.

The pinned APK hash was rechecked. Actual scan: **3.181989 seconds**; wrong exact
signature refused with empty stdout. This is a one-time APK scan, separate from
the fast cached whole-file diff. All 82 xtask tests, strict workspace Clippy,
formatting, build and repository link/evidence hygiene checks passed.

## Remaining runtime work

Next required evidence connects supported protobuf/message factories to these
native classes, resolves A07's effective ordinary-chat preference checks, and
separates the qualifying self-read submission pool from common status-17
persistence and observers. Existing direction evidence distinguishes read-self
from played-self; neither is relabelled as a recipient read/playback fact.
Incoming self receipts remain captured; no new receipt event, state mutation,
read policy, ACK, account operation or network request was introduced here.
