# Receipt-manager exclusions and revoke cleanup

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eight-topic recipe](apk/recipes/2.26.37.71-receipt-manager.json),
[full signatures and catch tables](apk/2.26.37.71-receipt-manager/targets.json),
[exact-field/caller scan and raw instruction evidence](receipt-manager-field-evidence.txt).
This closes the known producer of 13v.A03 in
[receipt admission](receipt-admission.md). It does not implement Rust revoke,
per-user native admission or identity cleanup.

## The excluded-key set is populated after revoke receipt deletion

**Confirmed:** 13v's constructor 0068..007e creates A03 with
Collections.newSetFromMap(new ConcurrentHashMap()). The exact-field scan finds
four physical references: this write, two 15A.A02 reads and one 8UM.run read.
The worker's packed-switch selects 06d4 for discriminator 44; 06d4..06f4
casts its A00 to 13v, A01 to the message, then adds that message's A0i key
to the set. These direct references contain no remove/clear or getter/return of
the set. This supports an instance-local exclusion set, not a durable server
revocation table. Reflection, external lifetime and every restart projection
are **unknown**; an absent direct remove is not proof of eternal exclusion.

13v.A02(message) deletes `receipt_user` rows by the supplied message's A0j
row ID (004a..008c), registers 8SJ discriminator 6 for the user cache, then
deletes ordinary `receipt_device` rows with that same ID through its 0Z4.A07
concrete 0Z6 store (00b0..0106), registering DLn discriminator 6. Both DELETE
predicates are message-row-only: there is no timestamp or device predicate.
This differs from identity-change cleanup's pending-primary intersection and
pending-companion-only deletion. No message body, original target/version
snapshot or user identity is erased by this function's two SQL statements.

After marking the enclosing token successful, 012e..0142 constructs 8UM
with discriminator 44 and registers it through 0ZN.A04. Its timing log is
`ReceiptManager/deleteMessageReceiptsOnRevoke`. The two exact DEX callers are
152.A00 at 100a and 153.A00 at 0fd0; both broader bodies are retained. Their
revoke/edit/placeholder predicates are not fully interpreted here. In particular,
this trace does not authorize applying receipt deletion to an arbitrary inbound
protocol message, a cancelled local intent or an uncertain original send.

## Successful close, rollback and admission filters

**Confirmed by previously retained evidence:** 0ZN.A04 registers Cpf
selector 1. Its Runnable runs through Bb1 after successful transaction close,
not the immediate SQLite onCommit notification. Rollback/failure uses Bax;
selector 1 does not run through that discard path. 0Gy requires no active
outer transaction and no rollback marker before delivering Bb1. See the
[existing full closure trace](peer-identity.md#cleanup-callback-commit-and-rollback-closure-2026-10-08).
This applies that already traced mechanism to the newly identified 8UM44
producer; no new FIFO or exactly-once completion guarantee is inferred.
The per-user/device cache workers selected above remain **untraced** here.

15A.A02 0914..0938 and 0970..0994 exclude manager-pool entries when either
the message's A0l is true or its A0i key is in 13v.A03. The checks happen
before the remaining entries reach 0Z4.A06 and the per-user writer. However,
the earlier device-only pool is dispatched at 07b4..07ba. Therefore this
manager exclusion cannot be substituted with a blanket rule rejecting every
receipt or every earlier device-only timestamp update. Preserve both branches;
complete revoked-message classification and later effects are still **unknown**.

## Message flag context and copy behavior

**Confirmed:** 0V9.A00 verifies storage type 1 (`MAIN`), evicts the supplied
key from 0pm, and sets message A0l=true at 0098..009e before its deletion
handler and transaction success/close. A later zero-deletion result logs
`FMessageDatabaseSubsystem/deletemsg/nothing-deleted`; the earlier flag write
still occurred. This establishes an in-memory deletion-path mark, not a proven
persisted column or a general "revoked" flag with complete lifecycle.

Cjb.A02 0078..0086 copies A0l=true from one message object to another; false
does not explicitly clear the destination. The exact-field write scan finds
seven producer methods. Only this copier and 0V9's deletion path were traced
in this chunk. The other five are listed in the evidence file and remain
**uninterpreted**. No default, persistence/reload behavior or canonical name is
invented from the obfuscated member.

## Renderer correction and remaining implementation work

**Confirmed local/static:** 1gB.close 0092..0096 adjusts its nesting count with
DEX bytes `d8 00 00 ff`, independently reread from classes2.dex at 0x12a6da.
The 22b literal is signed -1, not +255. The renderer previously retained only
its raw hexadecimal spelling. It now appends signed decimal for negative
22b/22s arithmetic literals, preserving positive rendering, registers, widths
and raw bits. Boundary fixtures exercise all eleven 22b and eight 22s opcodes;
the existing positive 16-bit mask value 255 remains `#0xff`. This corrects
inspection output only; no WhatsApp arithmetic value changes.

Existing checksum generations remain immutable. The predecessor executable is
retained privately for reproducing their prior rendering; the new generation
uses the corrected formatter. The CLI capability is existing apk-smali and
apk-research-bundle, with no new flag or HTTP route and x-live=false.

**Unknown/unported:** complete per-user manager admission, effective own-device
and namespace predicates, remaining flag writers/reload, cache workers, revoke
parse/authorization/projection and mutable identity cleanup/resend. A lookup of
0bA methods containing BO yielded no bodies and did not identify 08Z.BO0/BO1's
implementation. Do not use that negative result to assume a predicate value.
Rust currently stores original pending device rows and supported root times;
its independent DeliveryFacts are not native receipt_user existence or a
revocation-aware mutable ledger. No retained account, identity, send ID, wait,
receipt or message changed; no live request was made.

Validation: all 67 xtask tests, strict xtask all-target/all-feature Clippy,
rebuilt research CLI, CLI-reference regeneration, API/library-symbol check,
formatting and diff hygiene passed. The eight topics / 13 files / 301,312 bytes
reproduce exactly from the pinned APK. Cached self-diff verifies eight unchanged
topics and four metadata artifacts, not cross-release equivalence. The four raw
signed-decrement bytes were independently reread at their recorded ZIP-entry
offset. Hygiene checked 1,416 local links and 34 bundles / 8,808,333 bytes.
No fresh shared-runtime or live account test is claimed for this tooling/research
change; the preceding receipt-row runtime validation remains separate.
