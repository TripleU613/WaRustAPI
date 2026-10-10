# Companion addition: native group state and Rust boundary

2026-10-09. **Confirmed static**, WhatsApp **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [hash-pinned recipe](apk/recipes/2.26.37.71-companion-addition-group-state.json)
and [source bundle](apk/2.26.37.71-companion-addition-group-state/targets.json)
retain 13 topics / 18 files / 103,661 bytes. This is a static follow-up to
[returning companion admission](companion-directory-addition.md), not a new
account operation or a claim of full native observer parity.

## Exact addition inputs

**Confirmed:** 1IW.A04 receives old, added and removed sets separately. The
returning-companion caller has an empty removed set; its removal worker is
skipped. At 0194–0216, the added set may be filtered under hosted-device
predicate Cbb.A01 and property 0x70ba. The effective property and filter
predicate remain **unknown**. Subsequent calls use v12, the resulting set,
not necessarily the original v7 set. Empty v12 skips group updates.

At 0216–028a, the caller obtains groups from 0bI.A0J, loads each 1jL via
0bI.A0G, calls A04(added, store, user, group), then A07(null, added, null,
store, user, group). A07's Boolean result is ignored. Every map value is
explicitly false. A06 receives this map and a null removal set. Offsets are
method-relative bytes; branch operands are 16-bit code units.

## Group membership and device records

**Confirmed:** 0bI.A0J selects group_jid_row_id from group_participant_user
where user_jid_row_id matches the supplied user. It resolves returned IDs as
0c3 and skips null resolutions. This discovers existing participant groups;
it does not add the user to another group.

0bI.A04 obtains the participant through 1jL.A0G(user, false). If nonnull, it
sets 1jL.A0H=true and inserts into that participant's 1uU.A07 map only when
containsKey(device) is false (00d8–00f2). Existing records are retained.
New records use 1tn(device, null, false, false). Constructor assignments map
these inputs to A03=device, A00=null, A02=false and A01=false.

**Confirmed branch:** LID devices for a non-LID user in a non-LID group are
logged and skipped (00a8–00c6). Other LID mismatches are also diagnosed earlier;
logging alone is not proof of an unconditional refusal. A nonempty added set
calls 1jL.A05 even if no participant was returned. That helper synchronizes on
A0G, recomputes A04/A03 hashes through A0O/A0N and AzR.A03, then calls A0K/A0I.
The latter two helpers' complete cache effects remain **unknown here**.

**Confirmed conditional counterpart path:** 0bI.A07 requires exactly one
nonnull operation set. Only an LID-based group and LID user can reach its
counterpart branch, with additional participant/A0b conditions. If 0Xw.A0H
returns a PhoneUserJid, A01 rebinds each device number under that user and the
addition branch calls A04 on the rebound set. A01 catches 08j and returns the
set accumulated so far. This is not authority to invent a PN/LID mapping in
Rust; effective mapping, group conditions and caller persistence continuity
for the counterpart remain **unknown/unported**.

## Persistence and sender-key flags

**Confirmed for this remote, false-map/null-removal call:** A06's
containsValue(true) is false, so its A0Q loop is skipped. Own-device bulk
removal requires an own user and a nonnull removal set; it does not apply.
The per-group A05 receives false as its first Boolean and null as its set.
Its 0010 branch lands at 002c, bypassing the preceding A0Q/A04 path; null at
004c returns to 002c, bypassing bucket-removal logic. With a returned
participant it calls 1uU.A00 then 0bK.A03.

1uU.A00 copies the map's values, including retained old records. 0bK.A03
transactionally deletes group_participant_device rows for that exact group
and user, then calls A02 with those values. A02 rejects mismatched owners and
inserts via the matching group_participant_user row. Its named columns and
bindings are device_jid_row_id, sent_sender_key (A02),
sent_add_on_sender_key (A01) and sent_sender_key_bucket (A00).
The new record therefore has **0, 0, NULL** for those three state columns;
retained records carry their existing values through replacement. An empty
input performs no insert. This traced call does not establish that the own
cryptographic sender-key chain rotates.

## Rust comparison and remaining work

**Confirmed implementation inspection:** group_send::plan_in expands saved
participant users through current device lists and recomputes the participant
hash. group_fanout::Snapshot::prepare_policy distributes the sender-key message to
all frozen targets; it does not implement native per-group sent_sender_key
omission rows. sender_key_distribution tracks exposure for primary-change
rotation, a different purpose. Adding a device to the shared list therefore
affects a subsequently prepared plan without requiring a second native-style
participant-device cache. Already reserved or uncertain sends remain frozen.
This comparison does not prove all native cache/counterpart behavior equivalent.

**Unknown/unported:** concrete JhE/Kx8 registered consumers, the DNd
addition worker, complete 1IW.A05 coordinator effects, effective hosted and
counterpart gates, and native omission-cache parity. JhE.A0L only establishes
that a nonempty added or removed set dispatches Kx8(user, added, removed)
through 078.A00; it does not identify every consumer's effect. No live group
addition, omission optimization or key rotation is newly claimed.

## Reproduce and compare a release

Use the existing CLI tooling, with the local APK and recipe paths above:

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --output docs/protocol/apk/2.26.37.71-companion-addition-group-state \
  --recipe docs/protocol/apk/recipes/2.26.37.71-companion-addition-group-state.json --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-companion-addition-group-state \
  docs/protocol/apk/2.26.37.71-companion-addition-group-state
```

The self-diff checks unchanged retained topics, not another APK's semantics.
A new release requires re-anchored class/method selections and its actual hash;
a recipe hash must not be changed and treated as evidence of equivalent code.

Validation: exact APK export check verified 13 topics / 18 files / 103,661
bytes; cached self-diff reports unchanged retained topics and metadata.
CLI-reference regeneration, OpenAPI/library-symbol validation, formatting and
whitespace checks passed. Repository hygiene checked 2,463 local links and
126 bundles / 26,527,619 bytes. No Rust runtime changed; no fresh workspace
runtime test or live account operation is claimed.
