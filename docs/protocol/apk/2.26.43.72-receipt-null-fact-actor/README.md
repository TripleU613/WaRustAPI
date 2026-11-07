# Native null participant to direct receipt fact actor — APK 2.26.43.72

**Confirmed selected static / fixed offline**, build 264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected checked-DEX methods, catch tables and
reference searches from xtask fallback. Not runtime/native execution or REA IDs.

## Re-anchor and follow the event, not just its ACK

The [full exact anchor](parser-anchor.txt) maps the offline aggregate error to
F10.A03 in classes8.dex. Two initial exact queries, `empty participant list`
and `empty participant`, returned no references: apk-xref does not perform
substring matching. These negatives do not mean the longer string is absent.

[Complete F10.A03](receipt-parser.txt) at 0508..051c reads Eoh remote A02 and
participant A01 separately, converting participant through 0EB.A00. At 0580 a
null remote skips event construction; a nonnull remote and nullable converted
participant reach FSI.<init> at 05a2. For ordinary delivery/read/played with no
recipient, the key direction flag is true (0220..0230); sender/read-self paths
are distinct and are not newly supported here. The existing
[context and accessor proof](../2.26.43.72-ack-nullable-context/README.md)
establishes that failed PN/LID/hosted text constructors become null.

[0EB.A00](device-converter.txt) returns null for null input; [DBc.A0Z](device-wrapper.txt)
is its direct wrapper. [FSI constructor](event-constructor.txt) stores nullable
actor A02 and remote A03 without replacing the actor. The complete
[field-reference census](actor-readers.txt) finds only the constructor write,
[CRF getter](actor-getter.txt) and [Cc5 job dispatch](job-dispatch.txt). CRF
returns A02; [CXN](remote-getter.txt) independently returns A03.

[Interface caller search](actor-consumers.txt) finds Ens.A01 and F21.A05.
The [status path](status-apply.txt) preserves null actor at 002c ->005c and has
separate status-key/recipient handling; no status fact behavior is ported here.
The [job constructor](job-constructor.txt) serializes the optional actor and
remote separately; [job run](job-run.txt) reconstructs FSI and chooses status
or legacy processing. Serialization/recovery equivalence for every subclass
is not established by this selected trace.

## Ordinary outgoing direct legacy branch

[Complete F21.A05](legacy-apply.txt) at 02a8 reads CRF and at 02c0 reads CXN.
For an ordinary PN/LID message key (types 0/18), the branch at 02d6 reaches
0a48, then 0ad8. The retained complete predicate methods bind these exclusions:
A0Q tests type3; A0h type5; A0m delegates A0l/A0V (types1/2/28);
A0P tests3/5/6; A0a tests21. Ordinary types0/18 are outside all these sets.
[Current LID constructor/type](../2.26.43.72-message-ack-chat/lid-user.txt)
returns type18; the linked current
[type census](../2.26.43.72-message-ack-chat/jid-types.txt) supplies the PN mapping.

At 0afc the outgoing key direction flag selects 0b14. That block converts the
remote v5 with DBc.A0Z and places it in working actor v7, then goes back to 0308
for subsequent status processing. It does not resurrect constructor-invalid raw
participant text as a real target device. All DEX branch operands are code units;
printed PCs are bytes (e.g. 02d6 + 2*0x3b9 =0a48).
This selected branch is not a universal fallback rule for groups, broadcasts,
status, incoming keys, explicit recipient flows or feature-dependent storage.

## Bounded Rust correction and regression

The direct receipt target parser now honors known native Null classification
before its generic network Jid parser. A null optional participant enters the
existing authenticated sender/device fallback, just as an absent participant;
valid supplied participants keep this client's original-target/device checks.
The ACK path reuses the same internal text-null predicate. Raw nodes, capture,
identities, stored selectors, send IDs, attempts and local progress are unchanged.
No global Jid parser, aggregate conversion, group or retry admission change.

The expanded `unparseable_optional_receipt_addresses_follow_native_null_semantics`
regression first failed with 0 facts instead of 1 for PN participant device100.
It checks PN/LID peers, optional participant/recipient, delivery/read/played,
constructor-invalid device100/128/255, hosted2/355, invalid users, actual input
codec roundtrip, correct primary fact actor and timestamp, unchanged send
intent/attempts/progress, reopened fact persistence and duplicate idempotency.
Existing valid participant tests retain original-target/device binding.

**Unverified/separate:** complete F21 status/storage/feature-flag semantics;
valid participant normalization beyond current conservative checks; raw invalid
aggregate actor conversion; binary constructor validation; required sender grammar;
other families, cache behavior, native job recovery and live acceptance.
No fact is inferred from merely matching an ACK or local writer success.
No live account operation or new request family; no restriction guarantee.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-xref "$APK" --anchor 'ReceiptStanzaHelper/handleOfflineAggregatedReceipt: empty participant list'
target/debug/xtask apk-field-refs "$APK" --class 'LX/FSI;' --field A02 --type 'Lcom/whatsapp/infra/core/jid/DeviceJid;'
target/debug/xtask apk-callers "$APK" --class 'LX/FvR;' --method CRF
```

Each source file's full signature identifies its exact class and method; reproduce
with `target/debug/xtask apk-smali "$APK" --class '<declaring class>' --method
'<method>' --signatures --exceptions`. The exports retain complete selected
methods, not terminal slices. Job files select ReceiptProcessingJob.<init>/A0L;
event files select FSI.<init>/CRF/CXN/Cc5; predicate files select 0Dn methods.
Run `cargo test --locked -p whatsapp-api --all-features unparseable_optional_receipt_addresses`
and the complete workspace checks. checksums.json binds every other file's
byte length and SHA256. No APK, private state or bulk tree is included.
