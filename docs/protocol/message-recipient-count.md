# Incoming stanza recipient-count producer

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Three new helper topics](apk/recipes/2.26.37.71-message-recipient-count.json),
[complete signatures/register operands/catches](apk/2.26.37.71-message-recipient-count/targets.json)
and [exact field-write inventory](message-recipient-count-evidence.txt).
Existing group-envelope and count-preprocessing snapshots below are reused,
not rewritten. Offsets below are method-relative bytes.

## Root count parsing and constructor transfer

**Confirmed:** [CnN.A02](apk/2.26.37.71-group-envelope-own/attribute-parser.txt)
initializes v5 to zero at 0000..0002, then reads root attribute `count` through
`0hs.A05(name, default)` at 0078..0084 with that zero default. Nothing between
initialization and this call overwrites v5. The result v24 is the constructor's
integer argument in the range call v6..v29 at 0162..0168.

New `CaF.<init>` 002a..0032 copies its integer parameter v19 into `CaF.A00`.
The constructor has registers=25 and 24 parameter words including receiver;
the receiver is v1, and v19 corresponds to caller v24. This checks argument
placement rather than guessing meaning from the field name.

New `0hs.A05` returns the supplied default only when nullable A0M returns null.
A present value goes through `A06(value,name)`. A06 calls
`Integer.parseInt(String)` at 0000..0008. Its NumberFormatException handler
constructs and throws **B7l**, with an attribute/tag/value diagnostic. Therefore
present empty/malformed/overflow input does not become zero in this helper.
Java signed 32-bit parsing, including BMP decimal digits, differs from the
long/default-zero parser used for ACK and receipt time. Negative valid integers
are not clamped in these helpers. Complete upstream exception handling and
B7l classification remain **unknown**; no server refusal meaning is assigned.

## Transfer is gated by own sender and chat class

**Confirmed:** [CgC.A01](apk/2.26.37.71-group-envelope-own/network-message-parser.txt)
uses chat predicates A0m or A0i at 011c..0134. When both are false it goes to
0344, sets v15=0 and jumps back to 0174 (0346 + 2*(-0xe9)). In the other branch,
CnN.A04 checks a DeviceJid's user via BO0; otherwise a UserJid is tested directly
via BO0. A successful own-user test selects v15=1 at 0172. The existing
[own-envelope trace](group-envelope-own.md) retains those classification limits.

At 0204, false v15 skips to 0264 (0204 + 2*0x30). True v15 reaches
020e..0216: `CaF.A00` is copied to **Cih.A02**. The physical writes-only scan
finds exactly this one Cih.A02 write at classes7.dex byte offset 0x5fdb1e;
reflection is outside that inventory. No universal direct-message count copy
is established. The zero-default field when this assignment is skipped is a
**platform-default inference**, not a traced runtime observation.

The existing [Cih.A01](apk/2.26.37.71-group-envelope-own/stanza-builder.txt)
loads A02 at 00ac..00b4 and passes it as the Bp2 constructor integer at
016c..0170. The [Bp2 constructor](apk/2.26.37.71-group-envelope-own/message-stanza-constructor.txt)
stores it at 0050..0058. Finally [Bp2.A0P](apk/2.26.37.71-receipt-ownership/incoming-message-flag.txt)
under its independent C6a key.fromMe guard copies Bp2.A03 into message.A06
at 0058..0060. That completes the previously open root-count source in
[count preprocessing](receipt-count-preprocessing.md), without establishing
ordinary authored-message initialization or every preprocessing overwrite.

## Distinct nested count and runtime boundary

**Confirmed scoped distinction:** the same CgC method separately reads `count`
from an `enc` child at 07d6..07e2 and tests it for positivity. That local child
read is not the Cih.A02 field assignment above. Its complete retry/consumer
meaning is **untraced** here; it must not be substituted for root recipient count.

No encrypted-device cardinality, successful delivery count or receipt_user row
is inferred from either value. Primary identity cleanup still requires actual
per-user membership and complete admission/settings/namespace authority.
No Rust runtime, account, identity, reservation, message or network request
changed. Existing apk-field-refs, apk-smali and apk-research-bundle reproduce
the sources; no command, HTTP route or live capability was added. Future APKs
need reviewed anchors; cached file equality is not semantic compatibility.

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-message-recipient-count.json \
  --output docs/protocol/apk/2.26.37.71-message-recipient-count --check
```

Count-source validation: three topics / eight files / 34,489 bytes reproduce
exactly from APK 37.71. Cached self-diff verifies three unchanged topics and
four metadata artifacts; raw count-copy/false-rejoin branch operands and the
field-write opcode agree. CLI reference, API-symbol, formatting, whitespace and
hygiene passed (2,176 links/108 APK bundles). Source/docs only; no fresh Rust
runtime suite, account operation or live test is claimed.
