# Receipt route predicates and selected metadata user

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Fourteen-topic recipe](apk/recipes/2.26.37.71-receipt-route.json),
[complete helper methods and signatures](apk/2.26.37.71-receipt-route/targets.json).
The parent is the retained
[15A.A02 receipt processor](apk/2.26.37.71-receipt-admission/receipt-admission.txt).
Offsets are method-relative bytes. This closes transitive classification inputs
in [receipt admission](receipt-admission.md), not complete native user-row parity.

## Chat types and the extra metadata condition

**Reconfirmed:** 0DE.A0m admits nonnull native types 1/2 after rejecting A0W's
type 28; A0i admits type 5 and A0R admits type 3. These numeric results were
already recorded in receipt-admission; the new snapshot keeps the helpers next
to the transitive metadata classification. Complete public JID class mappings
remain untraced; numeric type checks must not be replaced by guessed class names.
The type-3 branch's broadcast diagnostics are independently retained in 15A.

15A.A02 computes these three chat predicates at 00f4..010c, then at
0114..0134 computes:

`special = 1ht.A03(message) && 0DE.A0l(selected_user)`.

The selected user is the CYV original user when supplied, otherwise its device
user. A0l's native user types are `{0,7,11,18,22,26}`; its full numeric predicate
is [separately retained](apk/2.26.37.71-message-ack/ack-user-conversion-predicate.txt).
At 0136..014e the single-message candidate flag is true only when all three
chat predicates and special are false. This is more than a chat domain check.

At 02a0, that flag gates the individual status comparison; its branch target
is 0300 when false. At 0300 the type-3/broadcast branch can resolve an existing
individual copy and build a separate CYV with its original status/time. At
043a..047e the remaining multipart classification asserts that A0m, A0i or
special is true. The ordinary nonbroadcast multipart value enters v23 through
04c0..04d0. Complete downstream filtering, own-user settings and counterpart
mapping remain separate requirements from this route choice.

## The special-message predicate is not a recipient count

**Confirmed:** 1ht.A03 returns exactly this Boolean over its shown helpers:

`!0hd.A02(chat) && (message.A0z || selected_metadata_user != null)
 && message.A02 == 0 && 1id.A00(message) == null`.

At 001a an A0z=true branch targets **0032**, so it bypasses user-list selection
only. It still tests message A02 and the 1ie descriptor. Treating that flag as
an unconditional true result would be wrong. The chat exclusion runs first.
No check counts send targets or requires exactly one metadata entry.

1ib.A02 obtains the message's 1ic descriptor via 178.A0A, reads its A00 list
when present and passes it through A03; otherwise it returns null. A03 copies
only instanceof **DLg** values into a new ArrayList, preserving iteration order.
It does not deduplicate users. 1ht.A00 applies that same filter, then returns
the **first** DLg.A00 passing 0hd.A02, cast to UserJid. A null list or no
qualifying value yields null. This is a first qualifying metadata user, not a
single-participant proof or authority to merge PN/LID identities.

1id.A00 obtains a **1ie** descriptor through the same message descriptor access
pattern. Its presence suppresses special even when A0z is true. **Unknown:**
A0z producers/meaning, message A02's complete semantics, DLg/1ic/1ie's producer
and serialization authority, and all caller/subtype invariants. Those values
must not be invented from current Rust target lists or receipt facts.

## Shared user/chat exclusion predicate

**Confirmed:** 0hd.A02 returns false for null; true for instanceof 0hh;
otherwise it calls A05. A05 only accepts PhoneUserJid and applies A09 to its
`user` string. A09 uses 0ng.A00's nullable parsed wrapper, obtains 0ni.A00 and
checks two inclusive numeric bands through 0nh.A00:

- `0x30ef09630..0x30ef0bd3f`
- `0x310ba59b0..0x310ba5a13`

These are literal public APK constants, not account numbers from private state.
Their service/product meaning is **unknown** here; no arbitrary user or bot
classification is inferred from the name of 0hd or its callers.

0nh.A00 XORs both operands with `0x8000000000000000` then compares signed
longs, establishing unsigned ordering. Its two opcode-c2 operations at
0004/0006 previously rendered as unknown. `xor-long/2addr` is the standard
12x operation in the [AOSP bytecode table](https://source.android.com/docs/core/runtime/dalvik-bytecode).
The disassembler now names it while preserving the existing instruction width
and register decoding. A literal-byte regression checks both operand pairs,
the following cmp-long alignment, and the parser's high-register form.
Old snapshots remain unchanged; the new snapshot uses the corrected renderer.
Re-exporting an older recipe can therefore change text solely because its
opcode name is now rendered. That is a renderer correction, not evidence of
an APK release or protocol change; do not silently overwrite old snapshots.

0ng.A00 checks length and optional leading `+`, calls Character.digit with
radix 10 for subsequent characters, accumulates with unsigned comparisons and
overflow checks, and returns a nullable 0ni wrapper. Thus ASCII-only digit tests
or a signed-long parser are not automatically equivalent. Complete Unicode
character and boundary equivalence, wrapper construction and Android runtime
execution remain **unverified**. Its code is retained rather than guessed.

## User-row and Rust boundary

The new helpers establish route prerequisites, not a successful SQL INSERT.
[The live user writer](receipt-user.md) still updates exactly one 5/8/13 column
and inserts only after zero updated rows. Status ordering, message status,
original-user selection, native read settings, manager filters and counterpart
row resolution determine whether that writer runs. Self/inactive inputs cannot
be assumed to become ordinary user rows just because the broad factory admits
them. Existing targets, Delivered facts or a metadata match are not evidence
that a receipt_user row exists.

**Unknown/unported:** complete normal-send user-row seeding, metadata producers,
all live writer admission/settings, native aggregate projection, primary cleanup
and full cross-device recovery. No account state, identity, reservation or server
wait changed; no send or receive window ran. Rust changes are confined to the
APK disassembler mnemonic, not messaging runtime. Existing APK research CLI
commands reproduce and inspect the work; no new HTTP route is introduced.

Validation: **812** workspace/all-feature tests pass (including 68 xtask tests),
with strict all-target/all-feature Clippy, warnings-denied Rustdoc, formatting,
API-symbol checks and CLI reference regeneration. The fourteen-topic snapshot
is 19 files / 51,917 bytes and re-exports exactly from APK 37.71; cached
self-diff reports unchanged artifacts.
The retained comparison/parser now shows all three opcode-c2 XORs explicitly.
No messaging runtime or live account check is claimed by these tests.
