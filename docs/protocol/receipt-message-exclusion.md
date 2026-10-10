# Message exclusion flag: all physical writers and media round trips

2026-10-10. **Confirmed static**, WhatsApp APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-receipt-message-exclusion.json),
[complete selected source](apk/2.26.37.71-receipt-message-exclusion/targets.json)
and [fresh exact-field/caller scans](receipt-message-exclusion-scans.json)
retain 14 topics / 19 files / 509,571 bytes of source evidence. This follows the
[receipt-manager exclusion trace](receipt-manager.md), closing its five
previously uninterpreted physical A0l writers. It does not establish full
native user-row admission or port primary identity cleanup.

## Why the gate cannot simply mean revoked

**Confirmed:** 15A.A02 excludes both manager pools when message A0l is true
or its key is in 13v.A03 (0914–0938, 0970–0994). The earlier device-only
pool reaches 0Z4.A06 at 07b4–07ba before those checks. No blanket rejection
of every receipt or device timestamp follows from the later manager filter.
The new all-DEX writes-only scan again finds exactly seven physical A0l
writers. Those are physical references, not proof against reflection,
native/dynamically loaded writes or alternate field declarations.

| Writer | Confirmed assignment and context |
| --- | --- |
| 0V9.A00 | Sets true during the already traced MAIN-storage deletion path, before deletion outcome is known. |
| Cjb.A02 | Copies true from source to destination; source false does not clear destination. |
| 75e.CS5 | Obtains a message via A02 and sets true at 0008–000e. |
| 7z7.A06 | Copies true from 8BW.A0O at 025c–026a; false skips the message write. |
| DNt.run | Discriminator 1 marks the object whose key was evicted after inserting/updating another cached object. |
| NewsletterStatusView.A00 | Constructs a new 1he substitution and sets true at 04e2–04e8. |
| Fqh.run | Discriminator 24 constructs a new 1he substitution and sets true at 1150–1156. |

The seven shown physical writers set/copy true; none explicitly writes false
to A0l. **Inferred platform default:** a fresh object starts with false, but
that does not establish the flag after all preparation, conversion, deletion,
substitution and reload paths. Do not synthesize a durable native column or
assume an arbitrary saved Rust message has this provenance.

## Wrapper and replacement details

75e.A02 returns 72z.A00 for a 72z instance, otherwise 75e.A00. Its CS5
therefore marks the selected contained message. The exact physical invocation
scan for declaring class 75e and method CS5 finds **zero callers**. This does
not mean unreachable: superclass/interface dispatch, reflection and differently
declared invocation targets are outside that scan. Its full lifecycle name
and invocation authority remain **unknown**.

DNt.run's top switch maps discriminator 1 to **1640**. It loads 153 from
A00 and message objects from A01/A02, evicts A01's key through 0pm.A04,
adds A02 through 0pm.A03, updates 0Fs using A02, then marks A01.A0l=true.
The generic Object/Object/Object/int constructor stores arguments 2/3/1
into fields A00/A01/A02 respectively; the typed 14C/1B3/Cb1 constructor
uses ordinary 1/2/3 ordering. Confusing the two reverses object attribution.
153.A00 at 103e–1052 invokes the generic constructor with discriminator 1
and registers its Runnable with 0ZN.A04 after transaction success marking.
Full revoke/edit/placeholder admission remains **unknown here**; this is a
cache-replacement mark, not permission to delete any original Rust send.

## Media/status flag round trip is asymmetric

**Confirmed:** the fresh 8BW.A0O writes-only scan finds three writers.
89j.A03 at 031e–0326 reads message A0l and assigns its actual Boolean to
8BW.A0O, including false. 7z7.A06's opposite-direction copy only sets
message A0l when media A0O is true. Thus a false media source cannot clear
an already true destination message through this conversion. The round trip
is not an unconditional Boolean overwrite in both directions.

8GJ.CS5 obtains its 8BW through A03 and sets A0O=true. Complete A03 wrapper
selection/lifecycle is **untraced here**, so no universal wrapper authority is
claimed. 7wt.A02 also sets A0O=true before opening its transaction. It deletes
from table `status` where `row_id = ?`, using 8BW.A0J, commits/closes and
then notifies its 8iA observers. This is an actual status-store deletion path,
not ordinary receipt_user seeding. A failure after the in-memory write does
not imply the write was rolled back; SQL transaction behavior and the object
field are different forms of state.

## Newsletter substitution context

NewsletterStatusView.A00 obtains a referenced message key, checks its message
cache and then database, and on the retained 1ii branch constructs 1he from
that key/direction and current clock, assigns a resource string and marks it.
Fqh.run discriminator 24 similarly handles a DiJ plus two message inputs;
a null first message directly supplies the other to DiJ.A00 without this mark.
A nonnull 1ii first message selects the synthetic 1he branch, preserving its
key/direction, setting a resource string and marking before DiJ.A00.
Resource IDs and object types are **confirmed**; human text/complete domain
semantics and every later render/storage action remain **unknown**.
**Inferred:** these are substitution objects used in newsletter UI handling,
not proof that every marked message was revoked by a received protocol stanza.

## Rust and remaining primary-cleanup authority

No Rust runtime, account, queue, identity, reservation or message changed.
Rust DeliveryFacts, immutable original targets and mutable device receipt rows
remain distinct from receipt_user row existence. A native exclusion mark is
not inferred from a cancelled intent, unsupported capture, plaintext content,
delivery fact or a server timestamp. Complete ordinary message initialization,
revoke receive authorization/projection, subtype/namespace admission and
user-row persistence are still required before completing primary cleanup.

Existing apk-field-refs, apk-callers and apk-smali expose the scans; the linked
recipe with apk-research-bundle --check reproduces the source. Cached
apk-research-diff compares retained topics. No new CLI flag or HTTP capability
was added; these research commands are offline. No new live operation is needed
for these static findings, and no live classification is claimed.

Validation: exact regeneration verified 14 topics / 19 files / 509,571 bytes;
cached self-diff found 14 unchanged topics and four metadata artifacts.
CLI-reference regeneration, API/library-symbol validation, formatting,
whitespace and hygiene passed (2,491 local links; 128 bundles / 27,307,426
bytes). No new workspace runtime test or live account operation is claimed;
the prior functional chunk's 887-test result remains separate.
