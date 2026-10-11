# Direct receipt attribution — 2026-10-09

2026-10-11 follow-up: [nullable receipt fact routing](receipt-optional-addresses.md)
now treats namespace-invalid optional qualifiers as absent, following the native
accessor. Recognized qualifiers, exact original chat/device attribution and
unsupported self statuses retain their existing guards. Earlier validation below
is historical; fresh live projection remains unverified.

2026-10-11 follow-up: [receipt ACK namespace/type checks](receipt-ack-namespace.md)
now share the existing strict kind getter and aggregate-target namespace mask.
Invalid server-only lid/broadcast targets skip before optional time parsing;
unsupported root/type receipts remain captured without a control ACK. Native
special JID grammar and complete ACK dispatch remain separate boundaries.

Comparison APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [12-topic source bundle](apk/2.26.37.71-direct-receipts/manifest.json)
contains complete selected method bodies, operands and exception tables.
Offsets below are method-relative bytes; prototypes are DEX shorties.

## Incoming receipt evidence

**Confirmed static:** `CmG.A03(LLLJ)` reads `category`, typed `recipient`
and `edit`, then calls `A00` with class `receipt` and default type `delivery`.
`CmG.A00(LLLLLLLJ)` reads `from` when the supplied remote JID is null, plus
`id`, `participant` and `type`. `CSu.A00` and `Cb1.<init>` retain separate
remote-chat (`A02`), participant (`A01`), recipient (`A03`) and type (`A09`)
fields. These are not interchangeable actor/chat fields.

**Confirmed static:** `Cmk.A00(IL)` distinguishes delivery=5, read=13,
played=8, read-self=17, played-self=18, inactive=15 and sender=5. Unknown strings
throw `invalid type`; the sparse-switch sender path shares the delivery return.
`Cmk.A03(LLLL)` starts with `type` defaulting to `delivery`, recognizes
participant aggregation separately, and uses `Cb1.A09` for status conversion.
At `0208..0298`, read-self and sender choose recipient/participant fallbacks
and direction separately from ordinary recipient receipts. Self statuses must
not be relabelled recipient read/playback in our current direct-only model.

**Confirmed static:** `Cmk.A02(LL)` starts from remote-chat user, optionally
selects recipient except for the two special JID predicates `0DE.A0m/A0i`,
then invokes `Cmk.A01`. `A01` has a gated `0hd.A00` branch, otherwise invokes
`0Yo.A07`. The complete transitive predicate/mapping policy and effective gate
are **unknown** here; static mapping calls do not authorize adopting a mapping
from arbitrary receipt metadata.

**Confirmed static:** the ordinary target branch in `Cmk.A03` at
`04aa..04be` uses the root `id` and selected chat. At `05be..0668`, it recognizes one `list` child and reads `item` IDs.
**Unknown batch parity:** the captured loop initializes an array and reads item
IDs, but no item-ID array assignment appears in that loop. This trace alone
does not establish the resulting native batch contents; Rust continues its
existing bounded, deduplicated list handling. `051a..05bc` keeps remote chat and
`DeviceJid.Companion.A00(participant)` separate while constructing `DDH`.
`DDH.<init>(VLLLLLLIJ)` stores message-key array (`A07`), remote chat (`A03`),
actual device (`A02`), selected user (`A04`), status and timestamp independently.
`14e.A04` calls `Cmk.A03`, then `Cmn.A04`; `17q.A04` is the separate status
handler caller. Complete scheduler, receipt-row application, companion/group
aggregation, privacy metadata and own-device synchronization remain **unknown /
unported**. The captured constructor proves distinct fields, not full delivery
application parity.

**Confirmed negative:** the earlier module comment referred to `ClU.A03`;
in this APK, `ClU.A00(LLL)` instead queries `integrity_chat_info`. The receipt
anchors `read-self`/`played-self` re-anchor the current parser to `Cmk`. Older
37.70 citations remain evidence for that older build only. The source bundle
keeps this counterexample so the stale class is not reused as current evidence.

## Rust correction and authority boundary

**Confirmed local implementation:** the handler is `receipts::record`, not
`receipts::observe`. Before this correction it already mapped some incoming
LIDs to PN. That could match a PN original, but it missed same-LID original
intents without a map and the reverse PN-to-LID direction, required full JID
encoding equality afterward, and changed the fact actor to mapped PN. The
previous identity-cleanup note overstated the PN-original gap; this audit
corrects it rather than treating the map as absent.

Current normal-outbox correlation uses one unambiguous wire ID across all
namespaces, then compares the actual primary receipt actor with the original
saved chat inside the same write transaction. Same PN or LID matches without
an alias; cross-namespace matching requires an existing
`authenticated-lid:<normalized LID>` primary PN association. Ordinary and
zero-device AD encodings represent the same components. Contact/directory
aliases, coincident numeric users and raw `sender_pn` provide no authority.
The shared association guard lives in `jid_map` and also serves reaction/poll
projection; it does not port the APK's gated canonicalizer or write mappings.

Matched facts retain the original intent/chat/wire ID and record the actual
received actor namespace. Independent read, played and delivery observations
remain independent; a read does not fabricate delivery, playback or server
acceptance. Existing uncertainty facts and local attempt progress are retained.
Duplicate actor/kind observations add no second outbox event across restart.
Ambiguous wire IDs, malformed saved entries or malformed/non-primary maps
refuse with fixed diagnoses and roll back the whole receipt batch, including
legacy facts and newly appended events. Unsupported route encodings, nonzero
agent/device, group/recipient-qualified and self statuses remain raw captures.
The primary-only boundary is our current scope, not a server prohibition.

Comparison-lab records remain PN-only: their saved `to` must be numeric, and
an actual LID may match that PN only through the same authenticated association.
The checkpoint scan now pages 256 rows instead of failing at 10,000 records.
Normal-outbox lookup reuses the existing paged uniqueness check; IDs per receipt
remain bounded to 257 locally. Neither bound is claimed as an APK/server limit.
Standalone `receipt-facts:*` now keys the actual normalized receipt namespace;
older PN-keyed observations remain in place. The repository has no consumer of
those standalone keys; they are observations, not proof of a matching send.
No historical receipt replay, send replay, pin adoption, ratchet reset, native
identity cleanup or receipt-send reservation mutation is performed.

Existing `status --local-id <local-id>` inspects normal-outbox facts and actor;
`receipts` shows the separate receipt-send ACK journal. Legacy comparison-lab
facts remain encrypted checkpoint values with no dedicated CLI inspection
command; repeating `lab-transport --message` does not print their receipt facts.
These are different kinds of evidence. No nonexistent lab-status command is
claimed.

## Reproduce and compare

```sh
cargo run --locked -p xtask -- apk-research-bundle \
  ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-direct-receipts.json \
  --output docs/protocol/apk/2.26.37.71-direct-receipts --check
```

Re-anchor another release into its own recipe and bundle, then compare cached
bundles with `apk-research-diff`. No APK decompression or account state is needed
for that cached comparison. Full mapping/scheduler parity and fresh live
PN/LID receipt attribution remain unverified.

**Historical negative local feature check:** a no-default-features build enabling only
`experimental-noise,experimental-transport` failed in the existing `about` /
`actions` feature graph (six missing-module errors outside this correction).
At that correction, supported CLI and workspace feature combinations were
verified separately. The subsequent
[feature-boundary fix](../../crates/whatsapp-api/README.md#features) resolves
the lightweight transport build and checks it independently in CI.

## Local validation

The exact APK/recipe bundle re-export passed `--check`: 12 selected methods,
17 files and 91,785 bytes. A checksum-verified cached self-diff took 0.017892
seconds; it is not a cross-release semantic equivalence claim. Workspace/all-
feature tests passed 739 cases, including actual encrypted-store restart,
13 direct attribution scenarios, duplicate suppression, batch rollback,
redacted map corruption, unsupported/self routes and 10,000-row checkpoint
history. Strict all-target/all-feature Clippy, warning-free Rustdoc, formatting,
default CLI tests (24), API symbol coverage and CLI reference regeneration pass.

Saved profile offline projection upgraded zero events. Peer-key, action,
receipt-send, outbox and event summaries remained identical, as did normalized
capture shapes and retained inspection. No poll update sample is present;
the two saved reaction-key inspections remain valid. This is regression
validation, not fresh cross-namespace receipt proof.

**Observed live regression check:** a bounded 45-second session authenticated,
activated and stopped cleanly (exit 0), with chat sends and reconnect disabled.
No fresh user message or qualifying direct receipt arrived. Saved peer-key,
action, receipt-send, outbox and event summaries and retained inspection stayed
identical. There remain 20 actions (17 ok, one error, two historically sent),
nine saved receipt ACKs and zero pending outbox intents. Capture count grew
437 to 442; five transport captures are not five user messages. The original
two uncertain actions were not replayed. Fresh direct PN/LID receipt handling
is still unverified live. Repository hygiene passed 1,092 local links and
5,623,446 bytes across 13 checksum-verified APK research bundles.

The [subsequent canonicalizer trace](receipt-canonicalizer.md) closes ordinary
PN/LID migration conditions, PNH_CTWA exemption, subtype-specific counterpart
direction and the separate bot branch. Effective runtime values and complete
application remain unknown. It also corrects malformed present `type` falling
back to delivery; absence alone now selects that default.

Later group extension (2026-10-09): [group participant receipts](group-receipts.md)
adds a separate original-target-bound group branch and offline comparison-send
inspection. Earlier direct-only scope statements describe this document’s
checkpoint; native aggregation, cleanup and normal group send preparation remain
unported.

2026-10-09 — **Confirmed static:** receipt application now has an eight-topic
trace through 14e/Cmn and the DDH device/time/status/key getters. Cmn.A00
selects CNn.A00 for a status predicate and separately 15A.A07 for other
statuses. CNn/CYV row policy and complete clocks/mapping remain untraced;
no mutable receipt ledger or runtime status promotion is claimed.
[Writer boundary and next inputs](receipt-apply.md).

## Direct participant device receipts — 2026-10-10

**Confirmed static (2.26.37.71):** `Cmk.A03` at `051a..052e` reads
`Cb1.A02` (remote chat) separately from `Cb1.A01` (participant), converts
the latter with `DeviceJid.Companion.A00`, then passes both to `DDH` at
`0592..05bc`. These existing bundled operands support retaining a distinct
actual device; they do not make arbitrary participant metadata authoritative.

**Confirmed local implementation:** normal-outbox direct delivery/read/played
receipts with `participant` now correlate the root primary chat and participant
user independently, then require the participant device in the immutable
send-time target snapshot. Cross-namespace association requires the existing
authenticated PN/LID mapping at the same device ID. Current directory membership
cannot fill a missing original snapshot. Root device-qualified routes without
participant, recipient-qualified routes, own/self kinds and aggregated receipts
remain outside this extension.

Before this change every direct participant-qualified receipt returned without
recording a fact. A qualifying receipt now appends an independent device fact,
retaining actual namespace/device and supplied server time. Duplicate facts
remain immutable across reopening. Existing mutable receipt-row observation
still requires its exact saved namespace. Mapping a public fact does not rewrite
that row's namespace. Send intent, attempts, uncertainty facts and progress are
preserved; no transmission or replay is introduced. Primary-only standalone
summaries and legacy comparison-lab Boolean summaries are not updated by this
new branch. The earlier primary-only statements describe the previous boundary.

Inspect device facts using `whatsapp-cli status --local-id <local-id>` and
original targets using `whatsapp-cli outbox-versions --local-id <local-id>`.
Missing target instrumentation means attribution is unknown, not failed delivery.
**Unknown live:** no fresh qualifying participant receipt has validated this
extension; static construction plus encrypted-store fixtures are the evidence.

**Additional confirmed static finding:** the local full-class extraction of
`PhoneUserJid` found `getServer()` returning `s.whatsapp.net` (`0000..0008`)
and `getType()` returning zero (`0000..0004`). Therefore the traced `0DE.A0m`
1/2-type predicate cannot be assumed to select ordinary phone JIDs. Full-class
method extraction found no method bodies for `0ZX` or `0c3`; this does not
establish interface dispatch absence. A class-name Jid-filtered `getType`
search found the abstract base and phone implementation only; obfuscated
subclasses outside that name filter remain unknown. An exact `lid` string scan
found 138 physical references, including `0ZY.getServer` and `0e1.getServer`;
those two bodies were not traced in this chunk. None of these partial searches
is used as device receipt authority.

Validation: 888 workspace/all-feature tests passed (747 library, 4 codec,
9 crash, 54 CLI, 1 API-symbol and 73 tooling tests). The new encrypted-store
fixture covers seven authority/target cases, all three fact kinds, supplied
timestamps, legacy isolation, immutable send state and duplicate suppression
across reopening. Strict all-target/all-feature Clippy, warnings-denied
Rustdoc, default CLI tests and CLI-reference regeneration passed. No live
request was made for this chunk. Repository hygiene passed 2,494 local
links and 128 checksum-verified APK bundles totaling 27,307,426 bytes.

2026-10-10 — [Offline device aggregates](offline-aggregated-receipts.md) now
handle the first-child `participants message_id` form and resolve the earlier
native `list/item` assignment question using REA plus the existing DEX operands.
Root-ID, per-entry actor/time and transactional failure boundaries remain distinct.

## Single-list branch correction — 2026-10-10

**Implemented/offline:** ordinary direct and group receipts expand `list/item`
IDs only when `list` is the sole child. The APK Cmk.A03 checks the first `biz`
child at `04be..04d0`, otherwise requires exactly one child at `05be..05cc`
and its `list` name at `05d0..05e0`. Multiple children, including a list beside
business metadata or another list, select only the root ID. The existing
[exact parser](apk/2.26.37.71-direct-receipts/receipt-status-parser.txt) and
[complete REA parser](rea/2.26.37.71-offline-receipts/parser.json) independently
confirm that selection. The list assignment at `060a` repeats on the back-edge;
see the earlier [array correction](offline-aggregated-receipts.md).

Before this fix, the shared Rust `ids` helper scanned every list among the
children. It could attribute extra delivery/read/played facts to messages the
native branch did not select. The encrypted-store regression failed with two
matched intents instead of one, then passed after changing that one helper.
It covers duplicate lists, business metadata before/after a list, unrelated
children, reopening without duplicate journal writes, and subsequent admission
through a real single-list receipt. Both direct and group callers use this
helper. Existing item validation, local 256-item bound, ID deduplication, original
target guards and all-or-nothing writes remain in force. Native duplicate list
entries are retained by its parser; local fact deduplication is not a claim of
native scheduler equivalence. No key-based or self-receipt branch was enabled.

### Separate key aggregate clock follow-up

The connected REA MCP provider now successfully reads APK methods.
[1xj.A02 Evidence](rea/2.26.37.71-receipt-list-shape/clock-delegate.json),
`ev_cdc634951bc5013cd41b7e5924bce12822fbb57304ddb848b69820797f473165`,
confirms the aggregate clock delegate calls `08B.A00`.
[08B.A00 Evidence](rea/2.26.37.71-receipt-list-shape/clock-selector.json),
`ev_c8581bbd22a4b74d482959f21bcb7e28794bccc011d5b6a0112a182d68f16e95`,
confirms NTP elapsed baseline first, WhatsApp elapsed baseline second, otherwise
wall clock minus saved server offset. This corroborates the earlier
[NTP trace](token-clock-producers.md#ntp-preference-and-acquisition-2026-10-09-continuation);
it does not implement NTP or validate a live default timestamp.
[Exact DDG methods](rea/2.26.37.71-receipt-list-shape/key-aggregate-model.txt)
confirm one message key, per-entry device/time and root status. Using the current
Rust inferred-stanza clock for missing times would substitute a different
authority. The later [key multi-target follow-up](offline-aggregated-receipts.md#key-based-multi-target-receipts--2026-10-10)
uses confirmed/local time instead of inferred stanza time and applies supported
facts atomically. Native NTP preference, full admission and fresh live projection
remain incomplete. No APK or Android process was executed.

### Validation of the single-list correction

Validation: 894 workspace/all-feature tests passed (753 library, four codec,
nine crash, 54 CLI, one API-symbol and 73 tooling tests). Strict Clippy,
warnings-denied Rustdoc, 25 default CLI tests plus API-symbol coverage,
CLI-reference regeneration and the experimental CLI build passed. The default
build retains the existing unused `clock::observe_login` warning. No live request
was made after the list-selection change; the preceding window is recorded
separately above.
