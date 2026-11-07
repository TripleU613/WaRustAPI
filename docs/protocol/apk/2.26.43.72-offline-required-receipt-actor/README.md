# Required DeviceJid in offline receipt aggregates — APK 2.26.43.72

**Confirmed selected static / fixed offline**, build 264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX methods with catch tables, xtask
fallback; no native runtime execution or REA Evidence envelope.

## Required, nullable and convertible are separate

The retained [complete F10.A03](../2.26.43.72-receipt-null-fact-actor/receipt-parser.txt)
uses 0Tp.A0B(DeviceJid,jid) at byte PC0098 for its first-child message_id branch.
This precedes each item's type and required timestamp. Its key branch instead
uses A0A(Jid,jid) and skips a null at0366; do not transfer that skip policy here.

[Complete A0B](required-accessor.txt) calls A0A at0000, returns only a nonnull
result, and constructs/throws LIL at000e..004a for null. The existing
[complete A0A](../2.26.43.72-ack-address-accessors/typed-address.txt) first uses
the typed object or nullable text construction. When DeviceJid is requested,
it converts only objects passing 0Dn.A0k through 0EB.A00, then calls Class.cast.
A failed cast is caught and returns null, triggering A0B's LIL.
[Complete A0k](convertible-user-predicate.txt) accepts types0/7/11/18/22/26,
not arbitrary UserJid subclasses. Existing DeviceJid objects pass Class.cast
directly without needing this conversion predicate.

The [current type census](../2.26.43.72-message-ack-chat/jid-types.txt) and
[known class mapping](../2.26.43.72-jid-write-compliance/README.md) distinguish
service type8 and gdpr type9 from an explicit service-backed Device17/24.
Group, broadcast/status, newsletter, call and temp classes do not become
DeviceJid through A0k. A successfully parsed generic address is therefore not
proof that a required actor exists. The
[constructor evidence](../2.26.43.72-lid-device-ack-text/README.md) separately
binds text constructor-null devices; the
existing binary_ack_user_class uses those retained parser/type proofs for a
bounded binary subset; its Invalid result is not generalized to opaque encodings.

## Local correction and validation

offline_aggregate now refuses known constructor-null or failed required-device
casts before selecting the item's type/time and before expanding any facts.
For text, this covers existing PN/LID/hosted Null, service and gdpr classification.
Known generic group/broadcast/newsletter/call/temp classes are refused as actors.
For ordinary zero-agent Plain/AgentDevice binary PN/LID/hosted input, the existing
native classifier also refuses known invalid, nonuser/service and server-only
objects. Opaque other binary classes and full constructor/cache fidelity remain
outside this bounded guard. Explicit service-backed device text is not a user
string blacklist: its Device class remains distinct from service UserJid.

The expanded `offline_aggregated_receipts_reject_malformed_batch_without_partial_facts`
test failed before correction because validate accepted a constructor-invalid
actor after a valid sibling. It now codec-roundtrips each malformed batch and
checks that validate and record both refuse, checkpoint/outbox rows and journal
stay identical, and error strings disclose neither synthetic private markers nor
the sender value. Cases include PN100, LID128, hosted-LID355, service/gdpr,
group/newsletter/status/call/temp and binary PN100/group actors. Existing valid
aggregate persistence and key-aggregate skip tests remain separate controls.

This guard prevents ordinary processing/ACK after a known malformed required
actor. It does NOT implement native LIL-to-error-response dispatch for all
ordinary receipts, prove a universal NACK code, or port native asynchronous
partial-write/recovery behavior. Native per-handler error response/dispatch needs
its own trace. Unknown types, typed nonzero legacy agents/opaque encodings,
required sender grammar, full valid-device projection, groups and live acceptance
remain separate. No global decoder/model, stored selector, identity, new request
family or live account operation change.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0B --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0k --signatures --exceptions
cargo test --locked -p whatsapp-api --all-features offline_aggregated_receipts
cargo test --locked -p whatsapp-api --all-features key_aggregates
```

checksums.json binds every other file by bytes/SHA256. Existing complete parser,
nullable accessor, type census and constructor evidence is linked, not duplicated.
