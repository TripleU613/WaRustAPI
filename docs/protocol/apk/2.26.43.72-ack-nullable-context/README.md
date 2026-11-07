# Nullable ACK context versus nonnull unwritable fields — APK 2.26.43.72

**Confirmed selected static / fixed offline**, build 264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX methods with catch tables, xtask
fallback exports; not runtime execution or REA Evidence envelopes.

## Descriptor construction does not require a sender

A fresh F0R.A00 export matched the retained
[complete context-field builder](../2.26.43.72-retry-ack-dispatch/receipt-context-fields.txt)
byte-for-byte. With no explicit sender override, it calls 0Tp.A0A(Jid,from),
then independently calls that accessor for participant. It stores both nullable
results in Efa. [Efa.A00](context-build.txt) asserts only the id at byte PC 001e;
it forwards the address fields. [Eoh constructor](context-constructor.txt) stores
them without null checks. The outer
[receipt factory](../2.26.43.72-retry-ack-dispatch/receipt-context-factory.txt)
adds recipient through the narrower UserJid accessor.

The [typed accessor](../2.26.43.72-ack-address-accessors/typed-address.txt)
uses an existing typed object if present; otherwise it calls 0Dd.A02 on text.
The [parser and constructor evidence](../2.26.43.72-lid-device-ack-text/README.md)
establishes null for failed native text construction. A successful, noncompliant
object is logged by A0A and remains nonnull; a failed class cast returns null.
These outcomes must not be conflated with each other.

[F3e.A05](../2.26.43.72-prekey-upload-replies/prekey-ack-wire-builder.txt)
conditionally appends to at 004a and participant at 007c. Null fields are omitted.
It is therefore false that context construction universally requires a sender.
This does NOT prove an arbitrary receipt handler reaches the writer with a null
sender: concrete handler admission/exceptions and earlier dispatch still matter.
Rust's supported automatic receipt ACK policy retains its sender requirement.

## Confirmed receipt ACK correction

The generic Rust Jid parser accepts some text for which the APK constructor is
null: e.g. LID device 100/128/255, PN device 100, and hosted original 355 or 0.
The receipt ACK path formerly copied these generic objects as participant.
The new shared receipt-output helper first honors known native Null classification
for text, then applies existing wire projection. A null optional participant is
omitted; a null supported sender cannot construct this client's ACK. No incoming
attribute is rewritten. Typed/binary input does not receive this text rule.

A nonnull accepted DeviceJid with original 256 is a different case: its
[writer fails](../2.26.43.72-jid-device-byte-range/README.md), so the selected-field
guard still refuses the ACK rather than dropping that participant.
The field regression failed before correction on 12345:100@lid being copied.
An older generic-participant test incorrectly expected hosted device 2 to survive;
its valid-device controls now use original 99, and device 2 is covered as null.
The regression now checks null omission, encoded output, unchanged raw metadata, unsupported
sender gating, and the nonnull/unwritable control. A receive actor checks
capture-before-write, reopening captures, and a later ping after the unwritable
participant produced no ACK. Fixtures have no pending sends: these assertions
do not certify participant fact attribution for existing sends.

## Remaining boundaries

General notification admission, nullable sender policy per handler, ordinary/group
fact parsing and attribution, other address families, typed binary constructor
validation, cache-derived addresses, writer recovery and live acceptance remain
separate. In particular this change does not normalize frozen selectors, grant
new sender/identity authority, change receipt facts or implement a new request.
No live account operation. Static fidelity cannot guarantee no restrictions.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/F0R;' --method A00 --signatures --exceptions > /tmp/context-fields.txt
cmp /tmp/context-fields.txt docs/protocol/apk/2.26.43.72-retry-ack-dispatch/receipt-context-fields.txt
target/debug/xtask apk-smali "$APK" --class 'LX/Efa;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Eoh;' --method '<init>' --signatures --exceptions
cargo test --locked -p whatsapp-api --all-features receipt_ack_omits
```

checksums.json covers every other file by byte length/SHA256. Existing complete
factory/accessor/parser/writer evidence is linked instead of duplicated.
