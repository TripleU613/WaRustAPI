# Receipt key direction and state-only message updates

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Ten-topic recipe](apk/recipes/2.26.37.71-receipt-state-only.json),
[complete new helper methods](apk/2.26.37.71-receipt-state-only/targets.json).
Offsets below are method-relative bytes. This extends
[receipt application](receipt-apply.md) and [admission](receipt-admission.md),
without promoting raw Rust facts to native message state.

## Parser key direction is separate from receipt status

**Confirmed:** the previously retained
[Cmk.A03 parser](apk/2.26.37.71-direct-receipts/receipt-status-parser.txt)
sets v13 at 0228..0238 using recipient presence, sender type and read-self type.
Following the actual branch targets yields:

| Receipt type | recipient absent | recipient present |
| --- | --- | --- |
| read-self | false | false |
| sender | true | true |
| other admitted types, including played-self | true | false |

The read-self test overrides the initial result to false. Played-self is not
part of that override. Thus two self-status names do not establish identical
key direction. The flag reaches Ayj.A0x for the key builder, including the
ordinary key-array helper path. Ayj.A0x passes it unchanged to
1gc(chat, id, boolean); that constructor stores it in A02 alongside chat A00
and ID A01. The retained copy constructor preserves all three fields.
The established A02 meaning is message-key fromMe; direction does not change
the independent parsed status 17/18.

Sender/read-self fallback diagnostics at 0238..028c do not modify this flag.
Full selected-user/chat canonicalization, all aggregated/business branches,
key-array helper validation and self-receipt trust remain separate trace
boundaries. This table is not authority to attribute self receipts to remote
outbox recipient facts. Current raw unsupported self captures stay preserved.

## A07 admission and played eligibility

**Confirmed:** 15A.A07(message, nullable Dbl callback, newStatus) reads B3b,
then calls 1hg.A04(current,new). A true result logs statusdowngrade and returns
false before the setter. The prior [status-order evidence](receipt-admission.md#ordering-is-not-numeric-status-comparison)
proves that known equal ranks are included. This state-only path differs from
the device-only pool that can still improve a timestamp on nonadvancement.

For advancing statuses 8, 9 or 10, A07 sets its played flag and calls
1gd.A0g. False logs status-played-non-ptt or view-once and returns false.
Other advancing statuses skip this eligibility check and leave the flag false.
A0g admits:

- Message type 2 when A05==1, virtual A0V()==true, or B2B.A00 returns a
  nonnull 8Bk descriptor.
- Message types 42/43 (1gd.A0M), 82, 78 or 81.

The base 178.A0V calls 0DE.A0b on the message-key chat; A0b is nonnull
JID type==21. **Negative:** this base method is not a view-once flag test.
The diagnostic text alone would give the wrong helper meaning. Virtual A0V
subtype overrides, A05 producer, descriptor semantics and complete public
message-kind domains remain **unknown** here. B2B.A00 obtains the 8Bk addon
through 178.A0A and casts its stored descriptor; existence alone does not
explain its lifecycle.

## Mutation, asynchronous hook and persistence result

**Confirmed:** admitted A07 calls virtual 178.A0H(newStatus) at 00bc..00c2.
When its played flag is true, it calls 8o5.A03; complete side effects are
untraced. Status 4 logs current wall-clock minus message A0F as a server-delay
diagnostic; it does not assign that elapsed value to receipt_user timestamps.

For statuses other than 11, 12, 16 and 17, it calls 0Yq.A01(message,-1).
That helper constructs DLx discriminator **16** and posts it to a Handler.
The earlier prose said 5; the retained source already showed 0x10. The
[observer follow-up](status-observers.md) resolves its immediate message/chat
callbacks while keeping registration, transitive effects and ordering unknown.
It is not proof of synchronous SQL or a receipt ACK. The 1iZ/status-20 branch separately calls 0Z0.A0C and
0Y5.A06 and returns true; full specialized persistence remains untraced.

Other messages call 0V9.A07(message,-1). A true result plus a nonnull callback
calls Dbl.COL(message). A false result or null callback skips COL, but A07
still reaches its true return. **Confirmed consequence:** A07's true result
means this method admitted the state update, not that every downstream database
operation returned true or that a network receipt was acknowledged. No rollback
of the already-mutated in-memory status is established on the false core result.

0V9.A07 checks its type registry and requires a registered 1hk updater.
Unsupported types log updateMessage skipping unsupported type and return false.
The supported branch opens a transaction, runs registered DbS hooks, invokes a
3Rd update lambda, calls 1hk.CgJ, runs DbR hooks, calls transaction A00, stages
a 3Pf callback and closes the transaction/store before returning true.
It retains an unsupported-update diagnostic inside this flow. Complete registry
selection, 3Rd's SQL/affected-row semantics, updater implementations, observer
vetoes, durable post-commit callbacks and database wrapper guarantees remain
**unknown**; this method is not a direct receipt_user row INSERT.

## Actual callers and remaining scope

**Confirmed from already retained sources:**
[Cmn.A00](apk/2.26.37.71-receipt-apply/apply-store.txt) calls A07 with a null
callback only on its non-1hg.A03 status branch (0130..0136). Statuses 17/18
satisfy that predicate, so they first go through CNn's CYV factory and 15A.A02,
not this direct A07 branch. CNn.A00 was rechecked against its
[existing factory trace](receipt-factory.md); it is still a value factory,
not an SQL writer.

[15A.A02](apk/2.26.37.71-receipt-admission/receipt-admission.txt)'s separate
state-only pool maps 8->10, 13->16 and 17->17 before A07; 18 skips that call.
Thus receipt status, mapped state argument, key direction and device timestamp
are distinct. The immediate [C8L/row suppression](receipt-state-suppression.md) is now traced.
Full chat read-state lifecycle, self admission, settings, selected-user
membership and primary cleanup remain unfinished. No Rust runtime, account
identity, key, request ID, reservation or live request changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-state-only.json \
  --output docs/protocol/apk/2.26.37.71-receipt-state-only --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-receipt-state-only \
  docs/protocol/apk/2.26.37.71-receipt-state-only
```

Static artifact agreement does not prove full receipt parity or live delivery.

Follow-up: [incoming self-read worker](self-read-worker.md) distinguishes the
status-17 common pool from qualifying submission/local status-16 marking.
Submission eligibility is not a blanket gate for common receipt state processing.

Follow-up: [receipt read policy](receipt-read-policy.md) traces the factory
preference/default and immediate contact/account predicates. Effective saved
settings and their synchronization remain unknown.

Follow-up: [receipt message projection](receipt-projection.md) traces native
per-user threshold counting, applied computed downgrades, the status-only SQL
update and conditional view-once hook. Complete threshold producers/admission
remain unknown; no Rust user-row projection or primary cleanup is claimed.
