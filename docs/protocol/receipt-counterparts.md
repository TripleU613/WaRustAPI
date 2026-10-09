# Receipt-user counterpart row selection

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eleven-topic recipe](apk/recipes/2.26.37.71-receipt-counterparts.json),
[complete retained methods](apk/2.26.37.71-receipt-counterparts/targets.json).
Offsets below are method-relative bytes. This extends the
[live receipt-user writer](receipt-user.md) and
[route prerequisites](receipt-route.md); it does not implement native primary cleanup.

## Row set used by the live writer

**Confirmed:** the previously retained 15A.A02 calls 8G8.A02 after 8G8.A01
selected its user, only on the message-chat 0DE.A0l branch (0c24..0c3e).
It uses the resulting set in the receipt_user UPDATE's
`message_row_id=? AND receipt_user_jid_row_id IN ...` predicate.
The other branch uses the single selected row. This is not universal expansion
of every receipt actor.

8G8.A02 first calls 0VT.A07 on the supplied user, checks that the result is
not -1 with the diagnostic `invalid jid`, and adds its decimal string to a new
HashSet (0000..003c). Only if 0Yo.A0H returns true does it call 0Yo.A02.
The comparison at 006c is **reference identity**, not JID value equality.
If the returned object differs, it calls A07 again and adds that row's decimal
string. HashSet deduplicates equal strings. There is no second explicit -1
assertion in this method, no receipt timestamp test, and no query proving that
either corresponding receipt_user row already exists.

## Mapping path and migration gate

**Confirmed:** 0Yo.A02 constructs 1wk with discriminator 2. Its invoke switch
selects 00b8..00d4, calling 0Yo.A0B(input, false). Do not follow invoke's default
branch, which has different mapping/fallback behavior.

A0B returns the original input when null, when 0DE.A0O rejects its type, when
A0H is false, or when the mapped row does not resolve. A0O accepts types
22, 18 and 26; their complete subtype domains are not established here.
When admitted, A0B obtains the input's JID row via A07, invokes
0Fs.A02(chatCache, null), then reads 0VS.A08's Long-to-Long mapping inside a
monitor (005c..0078). A mapped Long is resolved through
0VT.A0D(UserJid.class, rowId, false). A nonnull result is returned.
Only the boolean-true branch tries 0Xw.A0I; **this caller passes false**,
so that fallback does not authorize expanding this receipt row set.

**Confirmed:** A0H consults cached 0H9.A00. When absent, it calls both
0Gc.A0D and A0C; A0C=false returns true directly. Otherwise, when A0D=false
and 0KP.A03 is true, it logs the retained main-thread/database-not-ready
exception diagnostic, then proceeds to 0H9.A01. With a cached Boolean it
proceeds directly to A01. A01 returns the cached value; its uninitialized
path diagnoses `isGlobalChatDbMigrated is not initialized`, reports
`msgStore not ready`, and assigns Boolean true if those diagnostics return.
Assertion/diagnostic throwing behavior and complete migration initialization
are **unknown**, so the trace is not a license to assume this gate always true.

## JID rows are not receipt-user membership

**Confirmed:** A07 reads property 0x3c64 as the integer argument to private
A01. A01 checks a JID-to-row cache, queries A00, and, when needed, opens a
write transaction, rechecks, and inserts into **jid** using user, server,
agent, device, type and raw_string (00e4..0174). It contains constraint recovery,
a gated null-device repair, mismatch deletion and bounded recursive retry;
nonpositive insertion results can return -1. Complete cache callbacks, A00
lookup, A0D resolution, effective properties and database wrapper semantics
remain **unknown**. The retained full exception table supports further tracing.

**Inferred consequence:** calling A07 may establish a JID row without establishing
a receipt_user row. Neither a mapped JID row nor an original send target can
stand in for the actual receipt_user membership required by
[native primary cleanup](identity-receipts.md#counterparts-versions-and-companion-cleanup).

## Preserved boundaries and reproduction

The complete 0Fs.A02 chat-cache loader is retained because A0B invokes it before
reading the mapping. Its fast path returns the existing map, while other paths
have load/wait branches. Its full SQL hydration, mapping producers and concurrent
mutation authority have **not** been traced. 0VU declares abstract/native
inverse, put and putAll methods; that interface alone does not prove which
implementation populates A08 or authenticates a counterpart association.
No mapping has been adopted into Rust from this trace. Self-receipt direction,
complete receipt-user admission and primary cleanup remain unported; no account
state, reservation or live request changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-counterparts.json \
  --output docs/protocol/apk/2.26.37.71-receipt-counterparts --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-receipt-counterparts \
  docs/protocol/apk/2.26.37.71-receipt-counterparts
```

These checks establish reproducible static artifacts, not live protocol parity.
