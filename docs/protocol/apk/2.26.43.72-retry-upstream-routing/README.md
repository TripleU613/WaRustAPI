# Upstream routing before ordinary retry — APK 2.26.43.72

**Confirmed selected static / implemented routing exclusion**, build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Existing xtask fallback; no Android runtime, live account operation or REA Evidence ID.

## Evidence and correction

The complete [1rU.A05 router](../2.26.43.72-receipt-router-boundary/receipt-route.txt)
reads its FIRST child at020c. At0250..0294, a participants child with NONNULL
message_id forwards through private A00 before any root-type set/map/retry test.
An empty string is nonnull. A later participants child or one with only key does
not select this branch. At055c..059e, a root type=retry selects code2 ordinarily,
but category=peer selects code263 (0x107). The latter is a separate consumer;
this change does not implement it or interpret its schema.

Previously parse_retry and the ordinary retry error classifier used the root type
without these selectors. A complete retry behind an early aggregate could reserve
a same-ID resend or key refetch; an incomplete one could produce a false ordinary
retry487. A peer-category receipt could do the same. The shared ordinary_retry_route
predicate now excludes both paths before parse_retry, retry error classification,
handled-success ACK and retry_answer's outbox/identity/session processing.
Nontext selector carriers are unsupported and cannot authorize an ordinary retry;
this is a local capability guard, not a claim that a native string getter throws.
Aggregate validation and its independently proven parse-error causes remain active.
No ordinary success ACK is granted merely because the router forwards a receipt.

Tests cover empty leading message_id, key-only and nonleading controls, peer and
other categories, malformed ordinary retry error ACK controls, and a saved-session
workflow containing a matching accepted outbox entry. The latter verifies that
both alternate routes produce no resend/query/confirmation and preserve all
outbox, checkpoint, ratchet and peer-identity rows, before a normal retry proceeds.

## Additional traced boundaries and negative results

[19t constructor](status-registration.txt) registers277/0x115 with14h, closing
private1rU.A00's selected status message code to the retained
[19t.A04 handler](../2.26.43.72-ordinary-receipt-parse-error-dispatch/status-handler.txt).
This closes a static registration link, not provider/runtime scheduling. The
status handler performs participant/alias work before F10 fallback at0558; its
newsletter branch is separate and may return false. It must not be treated as
an unconditional alias for the ordinary handler. Full side-effect parity is open.

[0Tp.A0H](optional-offline-getter.txt) catches NumberFormatException from
Integer.valueOf and returns null, as for absent offline. Thus malformed offline
is NOT evidence for adding a new parse-error487 cause. No runtime change was
needed for this negative result. [F0R.A03](receipt-context.txt) reads category,
UserJid recipient and edit before shared F0R.A00 context construction; these
remain distinct from the fresh1rg descriptor used for parse errors.

Correction to the earlier router prose: the special error branch at068e compares
ROOT TYPE to error, then requires an error child. Mere presence of an error child
under another type does not select it. The complete export already shows this;
canonical prose is corrected in this change.

## Remaining scope

The full router, unknown-type copied487 response, peer retry consumer, special
call/biz/view/simple/rekey/error processing, complete binary/string equivalence,
feature/provider state and queue scheduling remain unported or unverified. This
change fixes two confirmed ordinary-retry misroutes; it does not claim complete
receipt parity, server acceptance or freedom from account restrictions.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/1rU;' --method A05 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/19t;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0H --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/F0R;' --method A03 --signatures --exceptions
cargo test --locked --workspace --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
```

Selected method exports are complete. checksums.json covers every other file;
existing router/consumer evidence is linked without duplicating it. No APK,
private state or bulk decompilation is committed.
