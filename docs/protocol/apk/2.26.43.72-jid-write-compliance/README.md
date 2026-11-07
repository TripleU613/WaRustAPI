# Known JID writer compliance classes — APK 2.26.43.72

**Confirmed selected static**, build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
freshly rehashed. Existing Android REA provider limitation remains; focused
xtask DEX fallback, no setup retry or new REA Evidence ID. No account traffic.

## Reproduce

From repository root:

```sh
target/debug/xtask apk-virtual-overrides ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/infra/core/jid/Jid;' --method isProtocolCompliant --signature '()Z' --include-interfaces > declared-compliance.txt
```

The retained complete census/method output covers114178 APK classes and finds
one declaration, the base Jid method. Its own scope explicitly leaves inherited
target/default/external/runtime dispatch unverified; it is not a runtime-wide
reflection/JNI completeness claim. The known selected class chains plus this
bounded census support using the base method for the cases below.

## Known false classes and final writer

The method returns false for type2/8/9/11. Reuse, without copying:

- [getType declaration/ancestry census](../2.26.43.72-message-ack-chat/jid-types.txt)
  maps7Kr ->2, Dv7 ->8, J9O ->9, and0EG/2LK ->11.
- [Temp constructor](../2.26.43.72-call-temp-ack-text/README.md) creates7Kr
  from temp user text containing a hyphen, retaining its user rather than
  applying PN/LID device syntax.
- [Special PN factory/classes](../2.26.43.72-special-pn-ack/README.md)
  maps Server to Dv7 and plain gdpr@s.whatsapp.net to J9O. Hosted Server
  aliases reach the same ServiceUser factory.
- [Self classes](../2.26.43.72-self-user-ack-conversion/README.md) bind11.
- [Final1r2 writer](../2.26.43.72-group-missing-identification/current-jid-writer.txt)
  tests positive agent/device first. Otherwise00b6..00c2 calls compliance;
  false reaches the IOException path0100..0132 instead of JID_PAIR output.
  These known nondevice classes use the zero-component path. The retained
  [agent declaration census](../2.26.43.72-group-missing-identification/current-agent-overrides.txt)
  and [device declaration census](../2.26.43.72-group-missing-identification/current-device-overrides.txt)
  bind the base zero getters and the distinct device-subclass overrides.

A literal user-string blacklist would be incorrect. [0EE/Dv4 constructors](../2.26.43.72-special-pn-device-ack/README.md)
can use a ServiceUser base; their own types17/24 are compliant. Explicit
Server:0@s.whatsapp.net and Server:99@hosted therefore remain separate cases.
Binary AD0 is reconstructed by123.A02/A04 before parsing and can instead become
the plain noncompliant service user. Native domain flags are not Rust legacy agent.

## Descriptor selection is part of compliance

[F0R.A03](../2.26.43.72-retry-ack-dispatch/receipt-context-factory.txt) requests
UserJid for recipient, but generic Jid for from/participant through A00. Temp
and gdpr fail the UserJid cast, so an omitted recipient cannot make the remaining
ACK unwritable. [F3e](../2.26.43.72-prekey-upload-replies/prekey-ack-wire-builder.txt)
copies only selected nonnull fields. [Retry contexts](../2.26.43.72-retry-ack-dispatch/README.md)
keep recipient for2Cx unsupported-version487 but remove recipient/edit for the
1rg fresh parse-error context. Test the fields actually selected for output.

## Implemented local correction and limits

A shared known-unwritable attribute predicate now refuses these mapped classes
in notification ACK routes and selected receipt ACK fields. Two regressions
failed before correction by producing an ACK containing Server@s.whatsapp.net.
Actual codec tests retain text and typed fields, allowed service-backed devices,
omitted nonuser recipients and fresh versus copied retry contexts. No ACK is
built for known-unwritable output; this does not port native writer-exception
recovery or change receipt facts, stored identities, aliases or retry authority.

A false predicate result is unclassified, not proof of complete native JID
validity. Unknown/cache-dependent subclasses, invalid nullable text grammar,
opaque caller-built metadata, full protocol encode admission and live acceptance
remain separate work. This is a supported automatic-ACK guard, not a new global
encoder policy or constructor failure/NACK rule.

`checksums.json` binds bytes/SHA256 for every other file in this bundle.
