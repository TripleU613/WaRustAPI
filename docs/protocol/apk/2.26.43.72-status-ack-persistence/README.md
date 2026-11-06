# Status ACK persistence and monotonic state — APK 2.26.43.72

**Confirmed selected static**, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports through xtask fallback; not REA Evidence envelopes
or runtime execution. No account operations. This is a research/negative-result
chunk; existing Rust settlement, once-only facts and no-replay policy stay intact.

## Native status identity and row update

Previous [FcQ/A03 evidence](../2.26.43.72-status-ack-async-edit/README.md)
binds nullpart status success to private1r6.A01. Dwc stores sender, remote chat,
id and derived1dW. It records whether sender equals0EG (fromMe). ItsA00 returns
sender when remotechat is status/type5, remotechat otherwise. Key producer FcQ
uses0EG sender, selected chat and normalized baseid. This is more than an id-only
row lookup. Dwc validates supported remote families through assertions; asserted
helper behavior beyond the retained methods is not guessed.

1r6.A01 receives participantDevice,Dwc,phash,addressing_mode,rcat,edit,count,
serverTime,serverId. If participant is null ANDphash is nonempty,0000..0084 loads
Dto.A0H(key), adds the ownLID device when available, recomputesF2N.A03 and compares
phash. A mismatch invokesEcN.A00(key,revokePredicate) andExJ.A01(chat,revokePredicate)
at0084..00c8, then CONTINUES to row lookup. These calls are reconciliation
candidates; no outbound request, replay, membership set or atomic repair contract
is established here. Missing saved row returns; no arbitrary row is created by
this selected method. Hash mismatch is not an early hard ACK rejection.

With savedDyI,0wE.A0g(chat,addressing_mode) occurs before state handling. Null
participant takes state predicates and can callDyI.A0D(count,serverId,rcat,time).
The state predicates below protect higher receipt/view states, with additional
DyI.A0F/feature conditions. Nonnull participant skips this whole-row update in
this selectedA01. No generic native once-only guarantee follows: an existing
SENT state is NOT excluded byACg.A01. Rust's once-only fact rule is separate
local reservation policy and is not weakened by this finding.

## Exact state names and predicates

1uI.<init> stores its final integer asvalue. <clinit> binds:

| Field | Value | Name |
|---|---:|---|
| A05 | 0 | PREPARING |
| A07 | 1 | SENDING |
| A03 | 2 | FAILED_TO_SEND |
| A08 | 3 | SENT |
| A06 | 4 | RECEIVED |
| A0A | 5 | VIEWING |
| A09 | 6 | VIEWED |
| A04 | 7 | PERMANENTLY_FAILED_TO_SEND |

TheA00 set contains SENT,RECEIVED,VIEWED,VIEWING. ACg.A00 mapsvalue through2EA
and tests that set.2EA searches enum values and defaults unknown integers to
SENDING. ACg.A01 returns false only for RECEIVED,VIEWING,VIEWED; A02 matches
FAILED_TO_SEND/PERMANENTLY_FAILED_TO_SEND; A03 matchesVIEWING/VIEWED; A04 matches
PERMANENTLY_FAILED_TO_SEND. This enum is unrelated to17f's integer4 in the
separate scheduled-newsletter ACK branch: do not transfer meanings across types.

The previously retained refusal1r6.A00 usesACg.A00 to skip failure-state update
for these already-progressed states. Other refusal side effects remain possible.
DyI.A0D dispatches E37 to2IU.state=SENT then1uP.A03, E36 to1uJ.A06(SENT,row),
andE35 to1tj.A0a(AQ5,count,serverId,rcat,time,false). Different storage backends
must not be conflated into one inferred transaction or Rust schema.

## SQL path and verification boundary

1tj.A0a has feature/storage branches. Its SQL-related path reads pending-server-
ACK metadata from status.content_proto by row_id; it delegates toA0C or performs
another locked/transactional update path. A0C obtains a DB handle/transaction,
locks the supplied object and rereads savedstate/content_proto byrow_id. When
both saved/in-memory states are inACg.A00 it compares their integer values.
For memory ahead of saved, the selected branch loads the saved receipt timestamp
while retaining the higher in-memory state; the other comparison branch applies
saved state throughAQ5.A0P. Saved flags/archive/content metadata are also reread.
These selected comparisons precede further update logic. It also
checks serverAckPending and condition flags. This confirms state-aware reread,
not that all native writes are equivalent to a single generic ACK fact.

CompleteA0a/A0C exports preserve callbacks/commit/cleanup paths. Their helper
A0B/A0A/A06,AB2 writer, flags, AQ5 identity ownership, callback lifecycle and full
transaction rollback contract remain untraced. No Rust persistence migration or
native schema port is inferred from partial coverage.

**Confirmed negative:**6ol.A06 used byFcQ delegates0HF.A01(text,0), multiplies
by1000 with Dalvik long wrapping. This matches the already-ported ordinary ACK
timestamp parser/scaling; no separate strict status timestamp parser is needed.
Existing executed-Java oracle regressions check the Rust parser/wrapping, not
native runtime execution or persistence/callback completeness.

## Reproduce

From checkout root with pinned APK outside Git, compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r6;' --method 'A01' --signatures --exceptions > plain-success.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Dwc;' --signatures --exceptions > status-key.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/DyI;' --method 'A0D' --signatures --exceptions > row-update.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/ACg;' --method 'A0' --signatures --exceptions > state-predicates.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1uI;' --method '<clinit>' --signatures --exceptions > state-init.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1uI;' --method '<init>' --signatures --exceptions > state-constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2EA;' --method 'A00' --signatures --exceptions > state-lookup.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/6ol;' --method 'A06' --signatures --exceptions > time-helper.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1tj;' --method 'A0a' --signatures --exceptions > storage-dispatch.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1tj;' --method 'A0C' --signatures --exceptions > storage-transaction.txt
```

TheA0 method filter deliberately retains all fiveACg state predicates. DEX PCs
are bytes; branch deltas are code units. Pure/local regression checks:

```sh
cargo test --locked -p whatsapp-api --all-features acknowledgement_timestamp_uses_java_oracle_and_survives_reopen
cargo test --locked -p whatsapp-api --all-features status_edit_
```

No new tests purport to reproduce the unported native DB. Remaining work includes
hash repair producers, complete storage/callback authority, multipart operations,
scheduled-newsletter state and operation-bound404 handling. No live/no-ban claim.
