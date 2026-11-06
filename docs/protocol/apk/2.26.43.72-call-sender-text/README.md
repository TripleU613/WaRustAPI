# Call ACK sender text and factory/cache boundaries — APK 2.26.43.72

**Confirmed selected static**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX methods/catches and direct censuses;
xtask fallback, not REA Evidence envelopes or runtime observations.0Dd.A00 was
re-exported and matched the retained complete parser byte-for-byte. No account use.

## Text parse path before call-field admission

[Existing complete0Dd.A00](../2.26.43.72-message-ack-chat/jid-parser.txt)
first checks its JID cache, then finds the last@. Empty user branches from003a
to0338 and throws09P. For qualified input the case-sensitive domain switch at
0068 recognizes twelve names: s.whatsapp.net, broadcast, hosted, hosted.lid,
bot, lid, call, g.us, temp, newsletter, interop, interop.g.us. The default and
nonmatching equals branches throw09P at006e..007a. An empty/unknown domain is
not a valid native text JID merely because Rust's generic Jid parser preserves
an Other server.0Dd.A02 catches parser failures and returns null, as retained in
[optional address accessors](../2.26.43.72-ack-address-accessors/README.md).

Without@ the parser calls0De.A00 at02be. New complete A00 returns bare
s.whatsapp.net ->0mL, g.us ->7Ks and call ->J9P, otherwise null.0Dd.A00 then
recognizes status_me ->0EG and lid_me ->2LK, otherwise throws09P at0344.
[Declared type census](../2.26.43.72-message-ack-chat/jid-types.txt) gives these
selected bare results types13,16,20,11,11 respectively. They are nonnull on this
factory path; they must not all be classified as invalid/null bare strings.
Complete later call sender-subtype handling remains separate.

## Cache check prevents an overbroad grammar claim

The general0Df cache has124 direct put callers across many unrelated caches;
that list alone says nothing about JID-cache ownership. The exact0De.A00 field
census has four direct references: initializer,0Dd.A00 and0De.A03/A04. Constructor
creates an initially empty1000-entry0Df. A03 caches a validated newsletter;
A04 uses the retained [UserJid factory A02](../2.26.43.72-message-ack-chat/user-factory.txt).
That factory also knows status_me/lid_me as domains. Therefore the Rust definite-
null recognizer excludes those cache aliases as well as the parser's12 domain
names.0Dd.A01 cache-key formatting omits@ when user is empty. The selected
factories cannot use an empty user to create a leading@ cache key, or create
an arbitrary unknown-domain/bare-name entry through their shown routes.

This is a bounded direct field/call analysis, not an exhaustive reflection,
aliasing, native mutation or dynamic-code census. Recognized family constructor
failures and cache-dependent subtype/alias behavior are intentionally not
classified as null from a Rust parse failure. No broad bare-sender rejection is
added from a cache-miss singleton's type alone.

## Rust correction / limits

The [selected recoverable null-sender route](../2.26.43.72-call-null-sender/README.md)
precedes call id/type checks. Rust now uses a text-path definite-null helper
for missing/empty sender, empty user, unknown qualified domain and unknown bare
name outside the recognized parser/factory/cache names. These frames remain
captured without applying call state; reading continues. Global required-id/
numeric parsing still precedes this choice, and459 remains pre-class.

Known native bare values still require call fields; call with nonempty fields
passes this bounded validator. Other bare-family type decisions, malformed
recognized-family users, nontext/cached JID values, all child payloads, telemetry
and native account recovery remain open. This does not port full JID parsing,
merge identity namespaces, implement calls or add a request/response/retry.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Dd;' --method A00 --signatures --exceptions > /tmp/call-text-parser.txt
cmp /tmp/call-text-parser.txt docs/protocol/apk/2.26.43.72-message-ack-chat/jid-parser.txt
target/debug/xtask apk-smali "$APK" --class 'LX/0De;' --method A00 --signatures --exceptions
target/debug/xtask apk-field-refs "$APK" --class 'LX/0De;' --field A00 --type 'LX/0Df;'
target/debug/xtask apk-callers "$APK" --class 'LX/0Df;' --method put
target/debug/xtask apk-smali "$APK" --class 'LX/0Df;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0De;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0De;' --method A03 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0De;' --method A04 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dd;' --method A01 --signatures --exceptions
```

checksums.json covers every other file. Existing parser/type/factory/catch
exports are linked, not duplicated. Static parity does not establish zero bans.
