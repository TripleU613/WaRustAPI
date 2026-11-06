# Message callback shutdown and separate native maps — APK 2.26.43.72

**Confirmed selected static**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence/runtime observations.
Research only; no account operations or Rust behavior change.

## Posted message3 clears the full-key callback map

[Existing shutdown eligibility](../2.26.43.72-iq-shutdown-eligibility/README.md)
binds ordinary0YB.A05 to0ln.A0F with original reason and channel queue predicate.
[CompleteA0F](../2.26.43.72-iq-promotion-shutdown/registry-A0F.txt) postsmessage3
regardless of its conditional empty-IQ timer-reset block. The [complete0m2
handler](../2.26.43.72-iq-promotion-shutdown/handler.txt) at055a first flushes
sentIQ mapA0H as previously traced, then at05f8 handles0Th.A04 fullEoh keys.
These are distinct maps and callback families, not interchangeable ownership.

At060a..06b4 it locksA04, snapshots ALL callback values, iterates keys to build
message-class-only bookkeeping pairs, clearsA04, then unlocks. A04 is the same
map used by status submission0Th.A01 and alternate0Th.A02; A02 wraps callbacks
with2Gc discriminator1, while statusA01 uses0. Alternate registration is NOT
registration inA05. Key projection usesfrom (0Dn.A00), originalid andfromMe
(edit!=8); edit null/empty becomes0, other values useInteger.parseInt. This
is saved local callback-key projection, not parsing an inbound ACK. A thrown
parse/conversion beforeclear prevents assuming unconditional successful cleanup.

After unlocking it schedulesFcN discriminator13 with those pairs and original
messagearg2/reason, constructsE5V(IOException withmessage `disconnected`), and
callsBsR on every snapshotted callback. The reason integer belongs to the
separate worker; E5V's constructor carries no such field. Metrics reset follows
normal callback iteration. A callback exception could interrupt that iteration;
no whole-system infallibility or observed race is claimed. Map removal occurs
before failure delivery, not after it. For the selected status future, the
previously retained2Gc.BsR/FfK.BsR propagate local exceptional completion,
not a server refusal or proof that no write occurred.

## Message bookkeeping reloads an existing row

FcN.<init> binds case13 and the supplied reason, list and handler. Its run
case13 at0314..0416 iterates pairs, callsEaT.A00(messageKey,false), skips missing
rows and obtains0fY.A04(row.key)'s set. For1eq rows it supplies1 as a selected
local argument; otherwise obtains0xw.A01(row). It constructsF1z(row,null), sets
A05=4, A04=the selected argument, A03=row.A07, callsF1z.A00(context,builder,set),
stores that set, originalEoh and original reason and dispatchesF1z.A01.
No row is created in this selected worker. The meaning of fields4/1, complete
subtype classification, EaT/set providers and F1z consumer/persistence/retry
policy remain untraced; they are not labeled server state or permission to resend.

## Direct cleanup uses a different map

0Th.A03(exception), called directly by both0YB.A05 and0YU.handleMessage,
locksA05, invokesBsR while still holding that lock, then clearsA05. It does
NOT referenceA04. Unlike message3 it does not snapshot/unlock before callbacks;
a thrown callback can leaveclear unexecuted. The directA05 field census has
four physical references: initializer, A03,0YU.handleMessage and
BulkGetPreKeyJob.A0L. Full producers/aliases remain untraced; no universal
callback type or lifecycle is inferred from that bounded inventory.

0uw.C6j (writer offline drop) calls0ln.A0D(stringIQid), the already retained
IQ timer/failure mechanism. It has no direct fullEoh/A04 removal. For a queued
status type8 with noIQid this is not evidence of immediate status-future failure,
ACK settlement, retransmission or cancellation. Posted shutdown processing,
timeouts and separately retained actual ACK dispatch have their own authority.

## Rust comparison and remaining scope

Existing durable receipt callback ownership survives reopen and can settle
unchanged original uncertain rows on a later matching ACK without resend.
Partial encrypted-write controls consume failed channels and retain durable
uncertainty. Both targeted tests passed. Native in-memory callback clearing
must not erase those durable reservations or imply the wire was never sent.
No confirmed Rust mismatch is changed here; these tests are Rust controls,
not native Android execution or full reconnect equivalence.

Open: completeF1z persistence/retry effects, A05 producers, shutdown/ACK races,
account/generation binding, replacement/restart and all reconnect paths. No
live operation, new retry, acceptance result or account-restriction guarantee.

## Reproduce

From checkout root, pinned APK outside Git; compare complete selected outputs:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/E5V;' --signatures --exceptions > exception.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/FcN;' --method 'run' --signatures --exceptions > worker.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/FcN;' --method '<init>' --signatures --exceptions > worker-constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0uw;' --method 'C6j' --signatures --exceptions > offline-drop.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Th;' --method 'A03' --signatures --exceptions > direct-flush.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Th;' --method 'A02' --signatures --exceptions > alternate-registration.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Th;' --method A03 > direct-callers.txt
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Th;' --field A05 --type 'Ljava/util/Map;' > other-map-refs.txt
```

PCs are bytes; branch deltas code units. LargeFcN retains its whole method;
only case13 interpreted. Direct-reference censuses exclude runtime/reflection
and alias completeness. Existing complete handler/eligibility exports are linked.
