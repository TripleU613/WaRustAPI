# Broadcast ACK text construction — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX class exports using xtask fallback, not REA Evidence
or runtime execution. No live account operations.

[0Dd.A00](../2.26.43.72-message-ack-chat/jid-parser.txt)00b2..00e8 handles
broadcast user location asDv3, status as2p4, otherwise constructs2U8 with the
WHOLE user. These classes confirm canonical broadcast server and respective
types6,5,3.2U8.<init> requires1..20 ASCII digits with first1..9. It does not
parse dot/colon suffixes as agent/device. Unicode digits, leading0, whitespace,
letters and explicit device syntax fail; exact status/location bypass that
numeric constructor. The special strings are case-sensitive.

The query for2p5 method bodies returned no matches. This is not proof of missing
class metadata: retained [getType ancestry census](../2.26.43.72-message-ack-chat/jid-types.txt)
binds all three to2p5 ->0nT ->0DY ->Jid, not UserJid. No methods are invented
from the empty query. No full reflection/cache completeness is claimed.

The [optional JID parser](../2.26.43.72-ack-address-accessors/optional-parser.txt)
returns null on constructor failure. For call ACK this preserves the earlier
recoverable null-sender route. Valid types3/5/6 are all excluded by
[0Dn.A0R](../2.26.43.72-call-ack-admission/sender-types.txt) and therefore reach
the wrapped-LIL call rejection with nonempty id/type too.459 diversion and
required-id/numeric parsing precede this classification.

The [typed recipient accessor](../2.26.43.72-ack-address-accessors/typed-address.txt)
requests UserJid, so valid broadcast recipients cannot override from. Selected
broadcast from remains its original plain user; invalid construction cannot
establish a selected message key. Participant conversion excludes these plain
broadcast types already; no new broadcast participant semantics are introduced.

## Reproduce

From checkout root, pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2U8;' --signatures --exceptions > list-class.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Dv3;' --signatures --exceptions > location-class.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2p4;' --signatures --exceptions > status-class.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2p5;' --signatures --exceptions > base-method-query.txt 2>&1
```

Binary typed-JID admission, generic receipt routes, full cache/reader recovery
and actual server policy remain unproven. Static admission is not no-ban proof.
