# Aggregate IDs: selector, required absence and local bounds — APK 2.26.43.72

**Confirmed selected static / fixed offline**, build 264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX methods with catch tables, xtask
fallback; no native execution, account traffic or REA Evidence envelopes.

## Identifier roles and read order are not interchangeable

The retained [complete F10.A03](../2.26.43.72-receipt-null-fact-actor/receipt-parser.txt)
first checks the first participants child's message_id through an optional string
reader at0044. Null chooses the other path; a nonnull string selects the offline
message_id branch. There it resolves the chat via F10.A02 at0052, then reads the
required message_id through A0L at005a. Literal absence of message_id therefore
does not itself mean a missing required offline message_id error: it selects the
key path. Opaque local string-carrier equivalence remains unported.

In the key path, the root kind is classified before key processing. A0L(key)
is invoked at0304; only AFTER that does F10.A02 run at030c. Thus required key
absence can throw LIL before chat resolution. An early generalization that chat
resolution precedes both required-ID reads was incorrect; comparing these exact
call sites refuted it. Non-delivery roots rejoin before this block, as already
[proved by the branch countercheck](../2.26.43.72-receipt-timestamp-errors/README.md).

[A0L](../2.26.43.72-receipt-timestamp-errors/required-string.txt) rejects null,
not empty text or a 128-byte threshold. [DBb.A0g](message-key-wrapper.txt) calls
[1dW.<init>](message-key-constructors.txt), whose ID check is nonnull; the selected
constructor adds no nonempty/length assertion. This is NOT proof of all native
storage admission or server acceptance for empty/long IDs. Rust retains its
supported nonempty/128-byte bounds as local refusals, not native487 authority.

## Preserve the chat dependency for future admission work

[F10.A02](chat-selector.txt) obtains optional recipient and converts remote from
via [0Dn.A00](chat-unwrapper.txt): a DeviceJid is unwrapped, otherwise
[0Di.A00](chat-safe-cast.txt) returns only an actual0DY or null. Depending on
A0l/A0h, the recipient may replace that remote chat. A02 asserts the result,
then delegates to [F10.A01](chat-canonicalizer.txt) and asserts its result again.
A01 has conditional0x9.A00 conversion and delegated0f5.A07 canonicalization.
This trace does not reproduce the complete feature/cache/store behavior or
assign new identity authority from arbitrary address text. It explains why the
relative position of each required-ID read matters.

## Local correction and negative controls

A missing key in the selected key branch now has a distinct pure-validation
cause, `receipt.aggregate.native_required_key`, accepted by the shared native
error predicate and fresh487 builder. The supported root-kind check precedes
that cause: opaque/unported root kinds remain separate rather than being
mislabelled as missing-key failures. Empty/overlong/opaque present keys retain
generic local refusals. The message_id branch does not require a key.
The response uses ROOT stanza ID and supported from/type/participant, without
inner ID, recipient/edit/category or aggregate children. No identifier is
synthesized. Normal and selected CLI callers reuse the existing capture-first,
no-fact/no-retry/error-write policy; no new network request family is introduced.

`missing_aggregate_key_gets_root_487_only_in_selected_key_branch` first failed
because there was no response. It now checks root487 and dropped recipient/edit,
a valid message_id branch without key, and unported root-kind exclusion. The
normal actor adds a missing-key response with durable capture before write and
unchanged outbox/identity/session snapshots. Existing response write-cut,
redelivery and no-local-replay controls remain in that actor.

`local_aggregate_id_bounds_do_not_authorize_native_error_ack_or_root_facts`
codec-roundtrips empty/129-byte IDs in both forms against a real saved ROOT-ID
send. Validation and recording refuse, no native error ACK is constructed, and
checkpoint/outbox/journal snapshots stay identical. Thus local bounds cannot
silently become native errors or root-ID fallback facts.

**Remaining:** unknown root-type native response policy, complete opaque/binary
string conversion, full native key/storage admission, chat canonicalization and
wrapper/queue recovery, other schema/read-order cases and live acceptance.
No new fact kind, identity merge, query/resend or account operation. Static/local
checks do not establish server acceptance or absence of account restrictions.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/F10;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/F10;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Di;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/DBb;' --method A0g --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1dW;' --method '<init>' --signatures --exceptions
cargo test --locked -p whatsapp-api --all-features missing_aggregate_key_gets
cargo test --locked -p whatsapp-api --all-features local_aggregate_id_bounds
```

checksums.json covers every other file by bytes/SHA256. Existing complete outer
parser and required-string evidence is linked, not duplicated. No APK/private state.
