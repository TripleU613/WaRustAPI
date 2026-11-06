# Unordered count queue hold and release

**Confirmed static**, WhatsApp 2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete xtask DEX exports, not REA Evidence IDs or Android execution. The known
REA Android provider limitation was not retried. Only the named hold branches
are interpreted; unrelated FeQ selectors/metrics paths remain outside this audit.

## Reproduction and scope

For each method row, from repository root run
`target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'CLASS' --method 'METHOD' --signatures --exceptions > OUTPUT`.
For the whole-class row omit --method. OUTPUT is the bundle-local file name.

| CLASS | METHOD | OUTPUT |
| --- | --- | --- |
| LX/1k7; | whole class | hold-owner-methods.txt |
| LX/FeQ; | run | escape-timer.txt |
| LX/FeQ; | <init> | timer-constructors.txt |
| LX/1ic; | A0I | ordered-pending.txt |
| LX/1iq; | A0E | ordered-count.txt |
| LX/1k9; | whole class | head-pair.txt |
| LX/1id; | A0A | head-selector.txt |
| LX/07Z; | A0v | head-list-helper.txt |

Field inventory command:
`target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/1k7;' --field A0A --type 'Ljava/util/concurrent/atomic/AtomicBoolean;' > hold-field-references.txt`.
It found seven direct DEX field references: constructor, scheduler1il.A01,
owner1k7.A03, three in2JI.run and one inFeQ.run. This is not a reflective/alias
mutation census. APK SHA256 was rechecked directly during this pass.

## What ordered-pending means

The previously resolved [consumer/hold binding](../2.26.43.72-count-consumer-binding/README.md)
constructs1iz with1k7 hold owner. [1ic.A0I](ordered-pending.txt) checks1if and1iu:
only a load latch with count0 proceeds to cache1iq.A0E; positive size returns
true. Unloaded queues are skipped. [A0E](ordered-count.txt) is the key metadata
map's size under its monitor, not SQL row count or an in-flight state filter.
Thus false is not proof that unloaded persistent work does not exist.

[1k7.A00](hold-owner-methods.txt) snapshots1if/1iu keys through
[1id.A0A](head-selector.txt): only loaded caches are inspected; under the map
monitor it takes listA09's first element and returns its2DG key, or null.
[07Z.A0v](head-list-helper.txt) proves first-or-null. [1k9](head-pair.txt)
stores these two keys and compares both null-safely. The static1k7.A0D sentinel
is1k9(null,null), constructed in <clinit>.

## Runner sets hold, detects a concurrent release and permits a timed bypass

Existing [2JI.run selector32](../2.26.43.72-count-queue-processing/consumer-runner.txt),
0370..0482, resolves1k7 and applies this logic only when0av.A0U is true.
It samples change counterA0B and checks1ic.A0I. If no loaded ordered metadata
is pending it clears A09/A01, incrementsA0B and clearsA0A under monitorA08.
If ordered work is pending it snapshots heads and enters that monitor:

- A01 already true: clear that bypass flag and continue toward one unordered
  batch (the selected log says ordered side stuck).
- Otherwise getAndSet(true) on holdA0A; a newly acquired hold increments arm
  numberA00 and invokes1k7.A01(snapshot,owner,arm). Already-held work keeps the
  existing timer instead of adding another selected timer here.
- After the monitor, if the sampledA0B changed, clear hold and continue; the
  equal-counter path exits this pull. This is a separate local change counter,
  not an account generation or a server acknowledgement.

A01 obtains0av.A06 and schedulesFeQ(owner,arm,21,snapshot) through08Z.Cc2,
cancelling a previous stored Runnable viaCYL and saving the new one. The log
labelsA06 as milliseconds; the configured numeric value and08Z scheduling
implementation are not traced here. Constructor field mapping is retained in
[timer-constructors.txt](timer-constructors.txt).

## Ordered removal and timer release are distinct stimuli

[1k7.A03](hold-owner-methods.txt) runs only when0av.A0U is true and1ic.A0I
is false. Under monitorA08 it clearsA09/A01, incrementsA0B and uses
hold.getAndSet(false). Outside the monitor A02 asks1il.A01 to schedule only
if the old hold was true. Earlier [1in/1iw completion overrides](../2.26.43.72-count-completion-rescheduling/completion-declarations.txt)
callA03 after their completion bookkeeping. Therefore ordered completion can
release unordered work, subject to the scheduler's remaining gates.

[FeQ selector21](escape-timer.txt),03f2..05a0 and1c0c..1c5e:

- If hold is already false, return. If1ic.A0I is false, require captured arm
  equals currentA00, clear flags/incrementA0B/clear hold under the monitor,
  then callA02 with the old hold (onEscapeTimer). A stale arm does not release.
- If ordered work remains, snapshot heads again and check the arm under the
  monitor. Escape when either nonnull saved head still equals its current
  corresponding head, OR both saved/current pairs equal the null sentinel.
  This is an OR across unchanged nonnull heads, not a requirement that both
  queues remain unchanged.
- The escape path clears hold; if it was true, setA01=true for the next pull,
  updateA09 by CAS and call the scheduler outside the monitor. The CAS result
  gates the subsequent metric path, not that scheduler invocation.
- If no escape predicate matched, rearmA01 with the new snapshot and same arm.
  Arm mismatch returns without this rearm. Retained exception paths release
  monitors and rethrow; no crash-recovery guarantee is inferred.

## Port consequences and limits

A future port needs distinct ordered-pending snapshots, hold ownership, arm
identity, change-counter handling and one-batch bypass. Replacing this with
immediate draining on every completion, a guessed constant delay, or an
unconditional replay timer would discard traced conditions. This evidence does
not authorize such changes. EffectiveA0U/config6f06/A06, full queue-load/disk
semantics, timer cancellation guarantees, executor rejection, starvation/crash
behavior and socket ACK ordering remain open. No Rust network behavior, retry
permission or live account operation changed.
