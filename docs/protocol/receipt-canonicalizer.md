# Receipt chat canonicalization — 2026-10-09

**Confirmed static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [versioned source bundle](apk/2.26.37.71-receipt-canonicalizer/manifest.json)
keeps selected complete methods, register operands and exception tables.
Offsets are method-relative bytes. This follows the
[direct receipt correction](direct-receipts.md), and distinguishes native chat
storage policy from our conservative saved-intent association.

## Default is absence, not malformed presence

`0hs.A0C(LL)` scans the node's `0hq[]` attributes, compares names against
`0hq.A02`, returns the first matching attribute, and returns null only when no
attribute is found. `0hs.A0M(LLL)` calls that lookup; a found attribute returns
`0hq.A03` directly (`0008..0012`), whereas no attribute returns the supplied
default (`0012..0014`). A found null string therefore does not take the default.
`CmG.A03` supplies `delivery` as the receipt wrapper's default; `Cmk.A03` and
`Cmk.A00` separately classify status types. Empty or unknown type is not the
missing-attribute case. Native duplicate-attribute parsing policy and every
binary attribute conversion remain **unknown** beyond this getter trace.

**Confirmed local bug/correction:** Rust `receipts::record` used
`get_attr("type").and_then(as_text).unwrap_or("delivery")`. That turned a
present JID or invalid-UTF-8 byte attribute into recipient delivery. It now
defaults only on `None`; present nontext, empty and unknown values are ignored
without mutating any receipt facts, legacy send, outbox entry or journal event.
Supported absent type still records delivery. Native absent/default evidence
supports this separation; the JID/byte refusal is our Node representation guard,
not a claim that the APK stores those attributes with Rust's enum layout.

The wire regression encodes and decodes JID-valued, non-UTF-8 and empty-string
type attributes using the comparison binary dictionary and proper frame
unpacking before exercising the handler. Encrypted-store snapshots and journal
head stay identical for each malformed case. An absent-type control matches
both legacy and normal sends, survives reopen, and preserves prior uncertainty,
local progress and original send IDs. It does not manufacture read/playback.

## Receipt chat selection

`Cmk.A02(LL)` selects the remote user's JID unless a recipient is present and
the remote is outside `0DE.A0m/A0i`. Those group-like/status predicates were
already traced in the [primary-LID evidence](peer-identity.md#primary-lid-receive-and-legacy-record-stability-2026-10-09).
The selected chat goes to `Cmk.A01(LLL)`; actual participant/device fields remain
separate in the receipt event constructor. This is selection of the message's
chat, not authority to replace the actual fact actor or its identity pin.

`Cmk.A01` first tests the bot predicate `0DE.A0P` and inverted `Ayk.A1V` gate.
If true, `0hd.A00` may return an immediate converted address. All other paths
call `0Yo.A07(LL)` (including fallback when conversion returns null).

## Ordinary PN/LID migration branches

`0Yo.A07` first requires `0DE.A0l` and excludes `0hd.A02`; otherwise it returns
the original. `A0l` recognizes types `{0, 7, 11, 18, 22, 26}`. `0hd.A02` has
the already traced special-bot/phone-range predicate; it is not equivalent to
all phone addresses. `PhoneUserJid.getType` is 0; ordinary `0ZY.getType` is 18.
`0DE.A0a` tests exactly 18 and `A0e` exactly 0, both false for null.

For those ordinary classes, the normal-edge policy is:

| Selected chat | Global one-to-one LID migration (`0Yo.A0H`) | Additional condition | Result |
| --- | --- | --- | --- |
| LID (18) | false | Stored chat category is `0Zk.A04` / `PNH_CTWA` | Original LID |
| LID (18) | false | No stored chat, or any other stored category | Existing PN counterpart if available, otherwise original LID |
| LID (18) | true | No additional category check | Original LID |
| PN (0) | false | No counterpart lookup | Original PN |
| PN (0) | true | Counterpart lookup | Existing account counterpart if available, otherwise original PN |

The LID/not-migrated branch reads `0Fs.A0F` and `0Ze.A0l` at `0030..005c`.
`0Zk.<clinit>` explicitly initializes A04 as name `PNH_CTWA`, wire-like string
`ctwa`; A03 is `GROUP_STATUS`/`group_status` and A02 `GENERAL`/`general`.
The equality exemption jumps to `00c4`, returning the original. The
counterpart call at `0074..0088` returns its result only when nonnull. The
phone/migrated branch at `0088..00c6` has the analogous fallback. Neither
branch invents a counterpart from numeric similarity.

**Confirmed scope, unknown runtime:** `0Yo.A0H` consults cached `0H9.A00`,
message-store readiness and `0H9.A01`; its complete source is retained here and
was previously captured for [identity receipt cleanup](identity-receipts.md).
The current account's effective migration state, full stored-chat/category
producer and native mapping-write policy remain unknown. The normal-edge
matrix does not establish feature values or exception recovery behavior.

## Counterpart direction is subtype-specific

`0Xw.A0I(LL)` refuses null and `0DE.A0Y` types `{22, 23}`. For `A0Z` types
`{18, 19, 25}`, it casts to account-user base `0ZX` and calls reverse `A0H`.
For `A0d` types `{0, 17}`, it casts to PhoneUserJid and calls forward `A0C`.
Other types return null. This closes the classifiers left unresolved in the
older JID-map selection trace; it does not equate all Rust server names with
all Android subtypes. In particular, type-22 exclusion precedes conversion.
The [earlier mapping trace](peer-identity.md#canonical-key-mapping-and-row-selection-in-rust-2026-10-08)
records forward row ranking and reverse lookup boundaries separately.

## Bot conversion is a separate branch

`0DE.A0P` recognizes only types 26/27. `Ayk.A1V` returns the inverse of
`1Z3.A02`; `A02` is true exactly when `1Z3.A00` returns a positive long.
Thus the immediate bot conversion is selected when that value is zero or
negative. `A00` uses cached A07 when present, otherwise returns zero if the
message store is not ready, or obtains a Number from lazy A06. The lazy
producer, cached value's meaning and current effective value remain **unknown**;
do not rename this observation a server flag or a universal bot rule.

`0hd.A00` checks `A04` for one fixed public APK bot-user constant; an `0hh`
matching that constant returns a fixed `0ZY` address. Otherwise a bot is passed
to `0DE.A07`, which checks the type, looks up the user in the inverse of
`8oC.A00`'s HashBiMap, constructs a PhoneUserJid when found, and returns the
original when absent. This is a static bot dictionary branch, not ordinary
PN/LID account association, and not permission to reuse these public constants
as private account provenance. Dictionary initialization and full bot subtype
inventory remain **unknown/unported**.

## Rust application boundary and next requirements

The transactional exact/authenticated saved-chat association introduced in
`d18a01c` remains the current Rust policy. It does not mirror all native
canonicalization gates, persist inferred migration state, apply bot dictionaries,
rewrite legacy receipts or change identity adoption. The native matrix explains
why a global "convert every LID receipt to PN" rule would be incorrect.

A complete port still needs authenticated mapping producers, effective native
migration state, the stored chat category producer, complete special-address
classification, and bot dictionary/gate lifecycle. Each is a distinct required
input; mere field existence or directory alias cannot supply it. Receipt identity
cleanup and own-device/group aggregation remain separate unfinished work.

Reproduce and compare offline:

```sh
cargo run --locked -p xtask -- apk-research-bundle \
  ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-canonicalizer.json \
  --output docs/protocol/apk/2.26.37.71-receipt-canonicalizer --check
```

Export a separately re-anchored recipe for another APK, then use
`apk-research-diff` on cached bundles for rapid file/topic changes. A cached
self-diff verifies artifacts, not cross-release semantic equivalence.

## Validation

Exact APK/recipe re-export passed `--check`: 24 selected methods, 29 files and
67,062 bytes. A checksum-verified cached self-comparison took 0.007559 seconds;
this does not claim a second release was equivalent. Workspace/all-feature
checks passed 740 tests, strict all-target/all-feature Clippy, warning-free
Rustdoc, formatting, default CLI tests (24), API symbol coverage and regenerated
CLI reference. Repository hygiene passed 1,101 local links and 5,690,508 bytes
across 14 APK research checksum bundles.

Offline saved-profile reproject upgraded zero events. Peer keys, actions,
receipt-send outcomes, pending outbox, event summaries, retained inspection and
normalized capture shapes were unchanged; two saved reaction-key validations
remain valid and no poll-update sample is present. No new live connection,
network request, account mutation, registration attempt or receipt replay was
performed in this chunk. The previous chunk's live receive check remains its
own evidence; malformed incoming types and the native migration/bot branches
are not newly live verified here.

Later group extension (2026-10-09): [group participant receipts](group-receipts.md)
adds a separate original-target-bound group branch and offline comparison-send
inspection. Earlier direct-only scope statements describe this document’s
checkpoint; native aggregation, cleanup and normal group send preparation remain
unported.
