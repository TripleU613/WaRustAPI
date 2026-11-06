# Disconnect bookkeeping reaches message-send logging — APK 2.26.43.72

**Confirmed selected static**, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence or runtime observations.
Research only; no account operations or Rust behavior change.

## Correct the persistence assumption with the actual consumer

The [prior disconnect worker](../2.26.43.72-message-disconnect-callbacks/worker.txt)
case13 reloads a saved row and constructsF1z. F1z.A01 invokesA02 to buildEuU,
then calls0x7.A01(record). F1z itself stores builder fields and counts selected
sets; it does not contain a durable-row write or send invocation. The initial
hypothesis that this was necessarily a message-state persistence/retry handler
is not established: selected0x7.A01 blocks construct message-send telemetry.
The complete large consumer is retained; only identified field/event blocks
are interpreted here. Helpers, scheduled workers and unrelated branches remain
explicitly outside the absence claim.

## Exact fields from the disconnect producer

F1z.A02 plusEuU.<init> bind these copies:

| Disconnect builder input | EuU field | Selected0x7 consumer |
| --- | --- | --- |
| A05=4 | A06 | 007a..008e boxes intoDqZ.A0e |
| A04=selected local integer | A05 | 0aa6..0ac0 conditionally records excess over1 intoDqZ.A0y |
| A03=originalrow.A07 | A04 | 009a..00b0 converts toLong DqZ.A0z |
| A0C=originaldisconnect reason | A0G | 0272..0288, when result=4, maps through2Aj.A00 intoDqZ.A0a |
| A08=originalEoh | A0B | stored byconstructor; no direct read ofthis field in complete0x7.A01 |

The reason is NOT EuU.A0H: that is a different builder inputA0D, which is
not assigned by selected disconnectcase13. 2Aj.A00 is an explicit nullable
Integer mapper; recognized inputs can map differently (e.g.21->20 and22->21),
unrecognized/null returnsnull. No label for every native reason value, native
server code or Rust status enum is inferred from these integers.

At0222..0244 the consumer also checks its separate terminal-failure boolean
and asserts result!=1 when set, with the string `Message send result should
not be OK if the failure is terminal`. The disconnect builder does not set
that boolean in selectedcase13. Therefore result4 alone must not be relabeled
terminally failed, server refused, unsent or safe to replay. No new native
message-state enum was discovered by these logging fields.

## Telemetry boundary and real side effects

0x7.A01 obtains aB1D representation throughEuU.A00. That helper returns the
provided entity for its sentinel case, or lazily converts the saved18U through
A9H.A00. Complete conversion/persistence effects are not reconstructed here.
The initial B1D.BXT predicate gates the main DqZ construction branch; no
claim that every saved row produces this event follows. In that selected
branch the consumer populatesDqZ from the record and saved metadata.
At1706..172c it submits that event to0NX.CSJ with selected sampling arguments;
other conditional branch28dc..28e6 calls0NX.CSW. This is the established
metrics submission boundary, not a new final HTTP endpoint, required request
or server acceptance proof. Full sampling, buffering/dispatch and feature
conditions must retain their own prior evidence or remain unknown.

The consumer is NOT claimed side-effect-free: selected branches update
SharedPreferences usage counters such assticker_send_count and premium-sticker
time, emit other events and call additional helpers. Full3394-line method
transitive effects are untraced. A bounded direct instruction search found
no EuU.A0B read, no direct0Sf send, SendE2 job enqueue or SQLite operation in
this method; that does not prove no helper schedules work or writes storage.
Do not derive a blanket no-persistence/no-retry result from an index search.

## Rust comparison and next boundary

No confirmed Rust mismatch is changed here. Existing durable receipt ownership
and partial encrypted-write controls pass: native logging after disconnect
provides no authority to erase original uncertainty, roll back keys or replay.
These are Rust offline tests, not native execution or full lifecycle parity.
The saved-row reload is a logging input, not evidence that Rust needs to mutate
message delivery state or synthesize native telemetry. No invented counters,
telemetry, mandatory field or live operation is added.

Remaining: completeEuU/A9H entity conversion, other consumer helper effects,
actual native retry producers and durable transaction/current-owner ordering,
A05 callback-map producers and shutdown/ACK races. This narrows the previous
F1z persistence hypothesis without claiming the wider goal complete.

## Reproduce

From checkout root with pinned APK outside Git; compare complete outputs:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/F1z;' --signatures --exceptions > builder.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0x7;' --method 'A01' --signatures --exceptions > consumer.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/EuU;' --method '<init>' --signatures --exceptions > record-constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/EuU;' --method 'A00' --signatures --exceptions > record-entity.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2Aj;' --method 'A00' --signatures --exceptions > reason-map.txt
```

PCs are bytes, branch deltas code units. Consumer retained whole; selected
blocks and bounded negative direct searches do not establish transitive/runtime
completeness. No server behavior, account-restriction or complete RE claim.
