# Selected snapshot consumer predicate

**Confirmed selected static evidence**, same pinned APK/native member as README.
REA `ev_de31349a252d43be8493904bb7d368f718caaaed63dffcd67b07121002b6fdd9`.
No native execution, live account operation or Rust runtime change.

StorageManager address point 0xd5580 has virtual **+0x58**, relocation slot
**0xd55d8 -> 0x83dc8**. Its selected normal predicate is:

1. Call RequestData availability helper 0x880f8 on manager+0x98. False returns
   false without taking the snapshot.
2. Use virtual+0x40 on the same manager (outlined helper 0xcb460 preserves
   receiver identity). With the pinned vtable, this is snapshot getter 0x83c7c.
   The caller supplies output storage sp+0x10 via x8.
3. Move only the first 16 bytes of that output (the shared pair) into local
   ownership. The getter also wrote status to output+0x10, hence **sp+0x20**;
   the selected 0x83dc8..0x83e68 body never reads that status.
4. Null handle returns false. A nonnull handle invokes virtual+0x10 using
   outlined 0xcb43c. For selected reader vtables d5880/d61c0 this binds to
   0x886ec: root pointer at reader+0x50 nonnull. Return masks bit zero.
5. Release the copied control block through 0x6f20c.

Thus this predicate checks availability, ownership and the root-pointer probe,
not status 2 versus 6 or a complete schema verifier. That does not establish
all other consumers' policy. The separate 0x83a54..0x83abc caller also takes a
snapshot, compares the nonnull handle's string+0x28 with another string and,
when unequal/missing, enters schema helper 0x8bda0; it must not be collapsed
into this predicate. Its enclosing admission/upgrade path remains incomplete.

**Unknown:** public semantic name for slot+0x58, all upstream callers, other
reader implementations, and final application acceptance. The numeric status
must remain attached to the snapshot in any future Rust port even though this
one predicate ignores it. No server requirement follows from a local probe.

Reproduction: bounded `llvm-objdump` windows retained in consumer-body.txt and
consumer-helpers.txt; selected pointer relocations retained in consumer-slots.txt.
