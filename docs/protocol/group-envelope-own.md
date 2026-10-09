# Group reaction envelope direction

2026-10-09. **Confirmed static** for APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [24-topic recipe](apk/recipes/2.26.37.71-group-envelope-own.json) reproduces
the [bounded register/exception bundle](apk/2.26.37.71-group-envelope-own/targets.json).
The [inventory](group-envelope-own-inventory.txt) retains global searches and
three additional scoped predicates. Offsets below are method-relative bytes.

This closes the ordinary-group source/envelope-own producer gap recorded in
[the previous mapper](group-reaction-mapper.md). It does not implement complete
own-device message synchronization, hosted/status references or linked-device
admission. Previously published events remain historical evidence.

## Source and flag producers

**Confirmed:** a writes-only scan of `1gc.A02:Z` finds two writes, both in
constructors. The `(VLLZ)` constructor stores its boolean at `000e..0012`; the
copy constructor copies it at `000e..0016`. The complete constructor-caller
inventory has 131 sites. Only the network path below was traced; do not infer
that the remaining callers are equivalent network entry points.

`CnN.A02` reads raw `from` (`0024..0030`), `participant` (`00bc..00c8`) and
`recipient` (`00cc..00dc`). `CaF.<init>` assigns these to A01, A02 and A07,
respectively. Its A0I flag comes from `0DE.A0R(from)`, which tests type 3; A0M
comes from `0DE.A0i(from)`, which tests status type 5. Normal g.us type 1 takes
neither flag. `is_sender="true"` becomes a separate A0K flag; it is not proof
that a normal group's reaction came from this account.

`CnN.A01`'s ordinary branch (`0096..00c8`) selects raw from as CXt.A00 and
raw participant as CXt.A01. `CXt.<init>` fixes those operand roles at
`0006..000e`. Its other source/status branches are outside this port.

`CgC.A01` feeds the selected chat through `CnN.A00` (`00e0..0118`) and reads
the selected sender (`0118..011c`). `CnN.A00` can normalize an own peer chat;
`08i.BO1` uses `0DK.A00`, which extracts a DeviceJid or constructs a primary
device only for supported UserJid types. Type-1 groups satisfy neither predicate
and retain their group chat. The group/status gate in CgC is `011c..0134`.
Within it, `CnN.A04` checks a DeviceJid's UserJid through BO0
(`0008..0034`); alternatively CgC accepts an actual UserJid with BO0 true
(`0148..0174`). This produces the own-sender decision.

`CgC.A01` writes the selected sender to `Cih.A06` (`01d0..01d4`). Its
own-sender branch supplies true to `Cih.A03` (`0204..020e`); the ordinary
remote branch supplies false (`032c..0332`). The additional `is_sender` branch
at `02a2..02b8` is gated by `0DE.A0b`, type 21, and does not apply to normal
type-1 groups. Cih.A03 stores the boxed flag in A0Y (`0000..000e`).

`Cih.A00` requires A0Y and obtains its boolean (`0026..0042`). The ordinary
key path passes it through `Ayj.A0x` to `1gc.<init>` (`0092..00a6`), keeping
original and normal keys together. Its recipient/special-chat normalization
branches are not implemented here. `Cih.A01` forwards A0X/A09 at
`004e..007c`, `0194..019c`; `Bp2.<init>` stores the original key in
C6a.A00 (`00f2..00fe`) and the normal key in Bp2.A07 (`0028..0030`).

`Cna.A01` feeds the original key to C9O.A00 (`0018..002a`), which constructs
CRD. CRD's constructor retains it as A0R (`000e..0012`). Cna also forwards
the normal key to CRD.A06 (`0114..011c`). The already exported CRD.A00 prefers
A06 over A0R (`0020..002c`) and CoT retains that key as A0A. This connects the
ordinary group sender decision to the envelope key read by Bal/BoQ.

**Confirmed:** Cna derives the reaction's source CoT.A06 from CjN.A04 via
0DE.A00 (`0040..0050`). The Cih/Bp2/base-stanza constructors establish that
field's selected-sender role. 0DE.A00 extracts a DeviceJid's UserJid
(`0000..0016`). The prior claim that the first CoT constructor argument alone
proved raw stanza participant authority was insufficient; this traced producer
supplies the missing evidence for the ordinary network group subset.

## Reaction decisions

The existing [BoQ disassembly](apk/2.26.37.71-group-reaction-mapper/ordinary-target-mapper.txt)
checks envelope 1gc.A02 at `0076..007e`. An own envelope goes to `00f6..00fe`,
retaining the reaction's payload fromMe flag and author. It does not run the
ordinary remote own-author classification. On a remote true-fromMe group
branch, a null participant jumps from `00d6` to `00b6`, selecting the source;
a present participant instead replaces it at `00b4`. Both clear the native key
boolean at `00b8`.

| Authenticated group sender | Payload fromMe | Author | Rust target |
|---|---|---|---|
| own PN/LID user or device | true | present or absent/empty | outgoing, no model sender |
| own PN/LID user or device | absent/false | supported explicit user | incoming explicit author |
| remote user or device | true | absent/empty | incoming authenticated sender user |
| remote user or device | true | supported explicit user | incoming explicit author |
| remote user or device | absent/false | known own user | outgoing |
| remote user or device | absent/false | another explicit user | incoming explicit author |

The model omits sender on outgoing keys; native CUq can additionally retain an
author. Incoming parents still resolve within the authenticated envelope group,
retaining their original saved device. No parent body is copied or opened.

**Confirmed local port:** `reactions::group_target_for_sender` derives this
decision from a sender already authenticated by the receive path and the held
login-identity PN/LID checkpoint. It does not authenticate a supplied sender or
accept a caller-provided own boolean. Receive projection and parent resolution
use it; `group_target` remains the explicitly remote-envelope reference parser.
`group_inbox::sender_for_record` loads a retained original route/capture and
checks its canonical or historical selector. CLI inspection now uses that
source rather than inventing a remote envelope from the record's group prefix.

**Local support limits:** sender-aware parsing needs a known own checkpoint,
even for a true-fromMe key, to distinguish the two envelope branches. A missing
checkpoint reports `own_identity_missing`; unsupported sender namespaces/agents
report `unsupported_group_sender`. Explicit malformed, agent/device-qualified
target authors remain unsupported. Remote false/default-fromMe without an
author remains unrepresentable. Native parsing can turn other invalid authors
into null; only absent/empty author fallback is ported. Missing/mismatched saved
routes refuse inspection with fixed redacted reasons. No runtime own-checkpoint
value, peer admission or directory alias is manufactured.

**Unknown/unported:** full native own-device synchronization and primary echo
reconciliation, hosted UserJid types, special recipient/chat normalization,
zero-device textual subtype parity and all non-network constructor callers.
The broader CoT/CRD writer inventory is not fully traced by this network subset.
Fresh live own-device/group reaction semantics remain unverified.

## Negative results and reproduction

`1gz.A00` dispatches already constructed domain messages; its two 1gc
constructor sites concern placeholder/history lookups, not this stanza's own
flag. `1fm.A04` processes orphan resolvers and copied target keys. `1gV.A0D`
loads media message keys from database `from_me`; `1gd.A0F` looks up an outgoing
associated message. `CoJ.A05` consumes CoT/domain keys downstream. Those scans
did not establish the network producer. Their scoped dumps remain private;
do not repeat those methods as evidence for a raw stanza flag.

Constructor scans narrowed Bp2 to one caller, Cih.A01; that builder has six
callers, and CaF has one caller, CnN.A02. These inventories are retained above.
Reproduce with `xtask apk-field-refs APK --class 'LX/1gc;' --field A02 --type Z
--writes-only`, `apk-callers` for the listed constructors, and the bundle recipe.
An untyped all-field query was refused by the tool; no complete Cih field-writer
inventory was established. An initial bundle was preserved locally before the
two-topic expansion; export refuses overwriting an existing bundle.

**Confirmed offline:** literal descriptor fixtures cover own PN/LID user and
device senders, both direction flags/default, null/empty-author fallback and
missing checkpoint/unsupported sender refusals. Twelve projection fixtures
cover remote/own reactors, original legacy/new parents, retained devices,
consumed body/outbox preservation and once-only markers across reopen. Retained
source inspection covers canonical/legacy selectors, capture-backed consumed
records, missing routes and mismatched participant selectors. One initial
negative fixture used underscore text instead of the parser's `.agent` syntax;
it was corrected without changing runtime parsing.

**Confirmed verification:** 779 workspace/all-feature tests passed (652 API,
4 property, 9 crash, 50 CLI, 1 API-symbol and 63 xtask tests), along with strict
all-target/all-feature Clippy, Rustdoc with warnings denied, default CLI tests,
API/library checks, CLI-reference regeneration and formatting. The final embedded
API/build check also passed. The 24-topic export and revalidation agree on
29 files / 127,370 bytes. Hygiene validates 17 APK checksum bundles totaling
6,028,707 bytes.

**Observed receive check (2026-10-09):** one 45-second window authenticated and
stopped cleanly with chat sends and automatic reconnect disabled. Original
captures were retained; capture rows grew from 481 to 487. Peer keys, actions,
receipts, outbox and durable events stayed unchanged. Both retained group keys
still inspected as target_validation=valid using their original sender routes.
No historical markers were repaired or removed. This checks the receive runtime
and retained references; it does not establish fresh live own-device/group
reaction handling or human handset rendering.
