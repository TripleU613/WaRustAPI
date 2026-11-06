# Explicit LID device ACK text — APK 2.26.43.72

**Confirmed selected static**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX methods/classes with catch tables,
xtask fallback, not REA Evidence envelopes or runtime execution.0Dd.A00 was
re-exported and matched its retained complete parser exactly. No account use.

## Parser chooses class separately from numeric device value

[Complete0Dd.A00](../2.26.43.72-message-ack-chat/jid-parser.txt) at0196..021a
finds last dot/colon. With neither it constructs a UserJid. Otherwise it rejects
trailing dot/colon, selects the base prefix using the earlier of last dot/colon,
and constructs the base through0De.A04. A colon suffix is Integer.parseInt;
with no colon device defaults0. The dot suffix is not parsed as a numeric agent
here. A colon before a later dot leaves that dot in the integer suffix and
fails parseInt. Multiple earlier separators can instead invalidate the base.

For lid it constructs0EM; hosted.lid constructsDv6. Even explicit:0 or a
dot-only suffix constructs a DeviceJid, not a plain UserJid.0EM.getAgent returns
fixed1, getServer returns lid and getDevice returns its stored original int.
A query for0EL method bodies yielded none; the previously retained superclass
census and complete0EM methods supply the selected inheritance/override evidence.
No meaning is guessed from the rendered text's numeric agent alone.

DeviceJid.<init> begins int-to-byte of the supplied integer into v2 and checks
that narrowed signed byte is0..99 (for this LID base, the status singleton
exception does not apply). Thus100/128/255 fail,256 and-256 narrow to0,355
narrows to99,356 fails.0EM additionally stores the original int; no u16 parsing
or modulo-only full-device model is inferred. The disassembler renders the
byte-field store as unknown opcode0x5d; the admission result depends on the
explicit int-to-byte and later v2 checks, not interpreting that store.

Dv6 invokes0EM/base checks then independently requires original integer99,
not merely low byte99.355@hosted.lid therefore fails despite its narrowed byte;
99 succeeds. GetServer preserves hosted.lid on that device, while its underlying
[0Dx user](../2.26.43.72-message-ack-chat/lid-user.txt) has canonical server lid.
Native0Dn.A00 unwraps DeviceJid.userJid for message-key chat selection. The
[optional UserJid accessor](../2.26.43.72-ack-address-accessors/typed-address.txt)
returns null on a DeviceJid class-cast failure, including explicit device0.
Constructor/parse failures become null through0Dd.A02.

## Bounded Rust use

An internal LID ACK-text parser now reports null/user/device and the canonical
base user. It uses the existing pinned Java UTF16/BMP decimal parser with i32
range checking, byte narrowing and hosted99 condition. It does not rewrite the
global Jid model or emit any of these unusual device strings.

Call admission uses null classification before later class-field checks.
Selected message ACK chat ignores a device-valued recipient and unwraps a valid
device-valued from, preserving explicit zero's class identity. Ordinary success
participant admission uses the same class/null result: valid user/device
participants retain the guard, constructor-null text does not. Refusal/challenge
handling remains separate. The special bot-feedback exception remains unported.

Other address families, opaque typed/binary JIDs, complete cache/provider/runtime
mutation, all receipt parsers and native account recovery remain separate.
No identity trust merge, call feature, request, device generation or replay.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Dd;' --method A00 --signatures --exceptions > /tmp/lid-device-parser.txt
cmp /tmp/lid-device-parser.txt docs/protocol/apk/2.26.43.72-message-ack-chat/jid-parser.txt
target/debug/xtask apk-smali "$APK" --class 'LX/0EM;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Dv6;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/DeviceJid;' --method '<init>' --signatures --exceptions
```

checksums.json covers every other file. Existing parser/user/type/cast exports
are linked instead of duplicated. Static parsing is not zero-ban proof.
