# Message ACK diversion admission — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected predicate exports through xtask fallback, not REA Evidence
or runtime execution. No account operations. The complete selected handler is
already retained as[1rH.A03](../2.26.43.72-distribution-ack-admission/native.txt).

At0294..02c2 the handler prefers recipient UserJid, otherwise normalizesfrom,
and applies its existing selected-chat conversion. At02c2..02ce, A0h(type5)
status chat diverts to0568. Otherwise02ce..02ee reads is_group_status with
text defaultfalse and diverts only exacttrue to0568, BEFORE ordinary error/
success routing. The dispatch at0568..0588 again selects recipient or normalized
from, with a status singleton fallback only if none exists.058c..05b4 calls
A0h, A0b, A0a; their full predicates here permit only types5,1,21 respectively.
If none matches,05b0 branches -0x26a code units to00dc, the common1lZ.A1D(node,Eoh) generic callback path, then return00e6.
It does not scheduleFcQ or call ordinaryA10/A0z. Generic callback ownership/
possible effects remain separate and unproven; this is NOT proof that the
whole ACK has no callback or native side effects.
When permitted,05b4..05e4 schedulesFcQ with the selected chat, node andEoh.

Thus ordinary PN/type0 and LID/type18 selected chats with exacttrue are ignored
on this branch. They cannot become ordinary terminal ACK outcomes merely because
id and recipient match. The check uses SELECTED chat: a groupfrom plus direct
recipient is still direct. Numeric459 diversion precedes all of this. Other
flag spellings or absence do not take this branch. The Rust change specifically
ports this proved ignored PN/LID case, shared by outbox/expected-chat paths.

## Other observed branches remain open

Status/type5 and flagged group/type1/newsletter/type21 scheduleFcQ; its downstream
application/settlement contract is NOT traced here. No claim that every diverted
ACK must be discarded or that existing status sends should stop working.

At0378..039c, error absence plus selected newsletter/type21, nonnull textual
scheduled_server_id and absent server_id enters a separate scheduled-newsletter
branch. Empty scheduled text is still nonnull; no integer validation is shown
at this test.03a8..03c2 builds two1dW keys with booleantrue/false.03ea..0420
invokes17f.A05(key,null,4), then0TX.A04(null,key,null) for each and loops,
returning without ordinaryA10. Meanings of state4, callbacks and durable
newsletter effects require tracing17f/0TX. No Rust scheduled-ACK classification
change or fabricated terminal outcome is made from these calls alone.

## Reproduce

From checkout root, pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Dn;' --method A0h --signatures --exceptions > status-type.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Dn;' --method A0b --signatures --exceptions > group-type.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Dn;' --method A0a --signatures --exceptions > newsletter-type.txt
```

DEX displayed offsets are bytes; branch deltas are code units, multiplied by2.
Binary admission, status/group/newsletter asynchronous processing, scheduled
state transitions, generic callback/receipt authority and server policy remain
open. No live operations, replay authority or no-ban proof.
