# Generic receipt ACK key and independent ID maps — APK 2.26.43.72

**Confirmed selected static**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete DEX methods/signatures/catches; xtask fallback,
not REA Evidence envelopes or live execution. No account use.

## Full-key callback route

[Retained1rH.A03](../2.26.43.72-distribution-ack-admission/native.txt)
constructs Efa/Eoh from typed from, nullable participant/recipient, required id,
class/type/edit before numeric error handling. Its ordinary generic route
calls1lZ.A1D(node,key), which forwards1lW.Big. Concrete1lY.Big resolves0ln and
calls A09; A09 wraps the pair in2E3 and sends handler what6.

[Complete0m2.handleMessage](../2.26.43.72-distribution-ack-admission/handler-callback.txt)
was re-exported and compared byte-for-byte against the retained file. What6
selects000e. At0048..005e it removes0Th.A04[key] under that map's monitor.
Eoh.equals compares nullable class(A06), from(A02), id(A08), participant(A01),
recipient(A03), type(A09), edit(A07), plus listA0A, nodeA04 and stringA05.
Those extra fields' full producer semantics are not interpreted here. This is
not an ID-only key or a key omitting nullable participant/type.

When no callback was found, the optional fallback at0062..012c is gated by
class=message, empty edit and error-present raw text. It looks for exactly one
edited key whose edit-cleared copy equals the received key, then removes that
callback. It is not a receipt-family fallback. At012e..013c a found1ue receives
BsT(node), regardless of whether node contains an error. This normal callback
method invocation alone does not mean server success; callback interpretation
must be traced separately.

## Independent ID-based cleanup/callbacks also execute

Do not generalize full-key lookup to all handler state. With no full-key
callback,012e branches directly to0162; after a found callback the shown flow
also reaches0162 if intervening work does not throw. From0162 it takes Eoh.A08 id, removes the handler's ConcurrentMap entry and timeout8,
then calls0TV.A02(id). A returned Eo7 can invoke its1ue.BsT(id) at01b0..01c8.
If none is returned, the branch01ca..01e8 removes0TV.A04[id] under its monitor.
Those map producers, family membership, account policy and durable authority
are not established by this trace. Full-key mismatch therefore does not prove
that the whole native manager leaves every ID-based record/callback untouched.
Neither does it justify completing a Rust receipt from ID alone.

## Rust comparison / negative finding

receipts::record_ack already requires receipt class, original/batch id, exact
type, logical to/from equality and nullable participant presence/equality. It
rejects currently unsupported recipient/edit attributes. No confirmed local
construction/ordering mismatch was established in this bounded audit, and no
runtime behavior is changed. Strict local receipt correlation is retained as
its explicit supported adapter, not claimed as a complete emulation of every
native map. Native callback BsT(node) and Rust diagnostic refused status are
different layers; raw numeric error459/malformed admission remains separate.

Other address-subtype/coercion cases, full callback registration, independent
ID maps' producers, specialized receipt state, retry/recovery and live server
acceptance remain open. No new request, error response, resend or release.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/Eoh;' --method equals --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1lZ;' --method A1D --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1lY;' --method Big --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ln;' --method A09 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0m2;' --method handleMessage --signatures --exceptions > /tmp/receipt-ack-handler.txt
cmp /tmp/receipt-ack-handler.txt docs/protocol/apk/2.26.43.72-distribution-ack-admission/handler-callback.txt
```

checksums.json covers every other file here. Existing complete handler and ACK
parser are linked rather than duplicated. Static evidence cannot prove bans.
