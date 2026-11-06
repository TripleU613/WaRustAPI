# Tracked count queue admission and disk scheduling

**Confirmed static**, WhatsApp2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Seven complete method exports from xtask apk-smali APK --class CLASS --method
METHOD --signatures --exceptions:1ic.A0J/A01,1id.A0P/A02/A0L,1k3.<init>,
1il.CHq. These are DEX text, not REA Evidence IDs. Claims below are bounded to
the selected tracked1k4 path, not all classes sharing these generic methods.

## Buffering-policy Boolean becomes A0D

The existing [ingress trace](../2.26.43.72-count-ingress-routing/README.md)
retains1s2.run fallback: Eza.A03 receives !0ZP.A04(node). Its first1k4
constructor Boolean passes unchanged to the first1k3 constructor Boolean.
[Base constructor](wrapper-base-constructor.txt) assigns that Boolean toA0D
at0060, initializes A0I=false and A0H=2A3.A05, and initializes A0C=-1.
Thus the reader inversion reaches a named field, not an unspecified flag.
It is not a database-commit indicator.

[1ic.A01](queue-selection.txt) selects1ix for1k4,1if for1kA and1iu for1k8;
unknown wrapper classes throw. [A0J](queue-manager-admission.txt) selected
notification tail at00e6 forwards both reader Boolean arguments to1id.A0P.
The separate message-class branch is outside this count interpretation.

The retained reader0c2e..0cae computes its first Boolean for1k4 as
!A0D AND offlineAbsent, and its second as A0D OR offlineAbsent, where absent
means parsed offline equals -1. These are local control-flow values, not a
new Rust configuration or sender-authority test. Malformed offline can fail
before admission; no replacement value is authorized.

## Admission is initially in memory

[1id.A0P](queue-admission.txt) at001e requires A0D=true OR a nonnull storage
backendA0B. It waits on the queue's load latch before checking the stanza-key
map under a monitor. A new key gets cache metadata and the stanza appended to
A09; a duplicate logs, invokes A0G and returns false. This duplicate behavior
is not proof of permanent account-wide deduplication or an ACK replay rule.

The selected first-stage promotion to2A3.A06 depends on the second admission
Boolean and A0L. [A0L](queue-inline-state-policy.txt) returns constantfalse
for1ix, so this promotion is unavailable for the fallback1k4 count queue,
even when the reader's second Boolean is true. Other queue types have their
own config-dependent policy. Do not generalize them to count notifications.

For a new stanza, if the first admission Boolean is true AND A0D=false, A0P
schedules a disk write. The1ix branch uses500ms;1if/1iu use separate configured
values. It then can call1im.CHq when the second Boolean is true and returns
true. This return establishes admission, not persisted bytes or server ACK.
Other later/periodic flush triggers remain outside this method's claim.

## Scheduler reaches the transactional writer

[1id.A02](disk-write-scheduler.txt) does nothing without a backend. Otherwise,
underA0D monitor, it cancels an existing0Sp job, creates Flr selector7 with
its supplied delay, launches it through0cl.A02 and retains the new job.
This is a resettable disk-write schedule, distinct from the6000ms ACK watchdog
and upload IQ response timers. Exact cancellation races are not established.

The previously traced [Flr selector7](../2.26.43.72-preack-transactional-dispatch/README.md)
flushes selected stanzas and, conditionally, pre-ACKs before send admission.
The scheduler now supplies a concrete edge into that trace, not a proof of
all count-specific pre-ACK-list entries or effective transaction configuration.

[1il.CHq](queue-arrival-hook.txt) either invokesA01 whenA07 is true or calls
2Ek.A01(stanza). Those helpers' processing/persistence gates remain the next
boundary. CHq itself neither confirms a disk commit nor directly writes ACK.

## Port requirements and open scope

A faithful Rust port needs separate admission, load ownership, persistence,
processing and ACK confirmation states. Its durable raw capture is not this
native queue. These exports do not justify treating admission as a successful
write, resending uncertain keys, copying500ms/6000ms into upload timeouts or
changing ACK/socket order. Effective flags/backend selection,2Ek/1il dispatch,
queue key ownership, reset/cancellation races and full count bulk ACK provenance
remain open. No Rust runtime change or live account operation.
