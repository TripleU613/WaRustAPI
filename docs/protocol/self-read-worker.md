# Incoming self-read worker: submission and state paths

2026-10-11, **confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Four-topic recipe](apk/recipes/2.26.37.71-self-read-worker.json) and
[complete structural methods](apk/2.26.37.71-self-read-worker/targets.json)
retain the dispatcher, collection helper and immediate guards. Offsets below
are method-relative bytes; branch deltas count two-byte DEX code units.
This follows [virtual self-read eligibility](self-read-eligibility.md).

## The status-17 split

For a message reaching Cmn.A04's status classification, the status-17 branch
starts at 0608. The three following conditions select the common CZ4 pool v30:

| Condition | Branch | Target |
| --- | --- | --- |
| 0DE.A0h(message-key chat) true | 0630 + 2 × 0x45 | 06ba |
| Ayn.A1S(message) true | 063c + 2 × 0x3f | 06ba |
| 0zd.A09(message) false | 0648 + 2 × 0x39 | 06ba |

[0DE.A0h](rea/2.26.37.71-self-read-worker/chat-guard.json) is the nonnull
JID type-7 predicate. [Ayn.A1S](rea/2.26.37.71-self-read-worker/message-guard.json)
delegates to 0hd.A02(message.B1Q()); it does not test message-key direction.
The complete B1Q producer, subtype dispatch and 0hd.A02 authority remain
separate boundaries. Numeric JID types are not inferred from local server strings.

When all three tests permit submission, 064c..0656 adds the message to v26.
0656..0668 invokes Cmn.A01(key,CZ4,v23), then jumps back to the loop.
The [complete A01](rea/2.26.37.71-self-read-worker/worker-observers.json)
adds the key only if C8L.A00(key,CZ4.status) is true. The
[previously traced C8L predicate](receipt-state-suppression.md) requires
fromMe false and status 13, 16 or 17. This helper only collects keys; it neither
sets message status nor commits a row. The full v23 consumer is not claimed here.

## Failed submission eligibility still permits state processing

At 06ba..06c4 the original CZ4 enters v30. A nonempty v30 later calls
Cmn.A00 at 0cdc..0ce6. Its returned map controls another Cmn.A01 collection
call at 0d18..0d36; Cmn.A02 builds auxiliary work and CPh.A00 consumes it.
The entire auxiliary observer chain remains unported.

The [factory/store dispatch](receipt-factory.md) and
[state-only admission](receipt-state-only.md) distinguish this path from
submission: status 17 survives CNn's settings downgrade, goes through CYV,
and ordinary messages reach 15A.A02. Its state-only pool maps 17 to 17,
then applies the saved last-read-row boundary and A07 status ordering before
mutation. Reaching v30 therefore does not prove a write, but a false A09
does **not** establish whole-receipt rejection or permission to discard the
receipt's local state effects. The 1iZ store branch remains distinct.

The qualifying v26 pool instead calls 0zH.A0V(collection,false) at
0d7a..0d92. The [retained submission trace](read-receipt-submission.md)
shows that A0I grouping/A0W queueing happens before the later individual A09
check. In that later loop, status 17 invokes A0J and continues without another
individual job in that branch. A0J skips 1iZ and true virtual A0V; otherwise
it sets status **16** and schedules DNe discriminator 2. Its
[existing-row persistence](read-receipt-persistence.md) is asynchronous and
can fail after the in-memory mutation; no server acceptance is awaited.
Do not collapse these two pools into one status-17 write or one privacy gate.

## Policy correction and port boundary

Rechecking A07's private A00 predicate exposed a prose error in the
[contact policy](receipt-read-policy.md#contact-predicate-a00): absent contact
returns true; a present smb: contact runs the premium-message query. A present
ent: contact is allowed only on the separate non-smb branch. Original structural
bytes and REA's full smali fallback agree. The old source bundle is preserved.

Before runtime self-read application, resolve authenticated target/chat authority,
supported message factories and virtual predicates, saved policy inputs,
last-read-row/position state, failure reconciliation and special ACK dispatch.
The existing parser's read-self fromMe=false rule remains separate from
played-self. Neither receipt is a remote outbox recipient read/playback fact.
No Rust behavior, public API, saved identity, key, request ID, wait, reservation
or live request changes in this chunk. Full client parity and ban immunity are
not established by static source agreement.

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-self-read-worker.json \
  --output docs/protocol/apk/2.26.37.71-self-read-worker --check
```

For a new release, review/remap the exact methods before exporting a new pinned
generation and using apk-research-diff. These four topics have no automatic
relocation anchors; old obfuscated names alone are not cross-release authority.

Validation: all nine generated files / 86,208 bytes reproduced exactly from the
pinned APK. Four complete REA envelopes have the same subject SHA-256; five
critical branch targets were checked against retained structural instructions.
This source/documentation chunk does not repeat Rust runtime tests or claim
live self-read application.
