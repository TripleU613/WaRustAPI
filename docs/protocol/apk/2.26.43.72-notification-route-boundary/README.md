# Shared notification routing boundary — APK 2.26.43.72

**Confirmed static**, not observed runtime or REA Evidence records. APK SHA-256
freshly verified: `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
build264307238. Existing REA Android provider limitation remains; focused xtask
fallback, no setup retry or account traffic.

## Reproduce and reuse

From repository root:

```sh
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Tp;' --method A0L --signatures --exceptions > required-attribute.txt
```

This is the complete selected method. `checksums.json` binds this README and
export. Existing complete exports are reused, rather than copied:

- [F0R.A02 notification entry](../2.26.43.72-group-missing-identification/notification-key-entry.txt)
  supplies class notification and null override peer/optional metadata.
- [F0R.A00 fields](../2.26.43.72-group-missing-identification/notification-key-fields.txt)
  reads generic Jid from/participant via0Tp.A0A and id via0Tp.A0L; freezes via
  [Efa.A00](../2.26.43.72-group-missing-identification/notification-key-freeze.txt)
  into [Eoh constructor](../2.26.43.72-group-missing-identification/notification-key-constructor.txt).
- [Typed getter](../2.26.43.72-group-missing-identification/ack-typed-jid-read.txt)
  prefers cached typed attributes, otherwise calls
  [nullable parser](../2.26.43.72-group-missing-identification/ack-nullable-jid-parser.txt).
- [0Dd.A00 parser](../2.26.43.72-message-ack-chat/jid-parser.txt)
  takes lastIndexOf('@') at0028..0034; index0 branches at003a by+0x17f
  code units to0338, constructs09P and throws. A02 catches all parser throwables
  and returns null. A bare server string takes a separate no-@ branch.
- [F3e.A05 writer](../2.26.43.72-prekey-upload-replies/prekey-ack-wire-builder.txt)
  omits null to004a and participant007c. It suppresses type only for
  receipt/delivery, not notification. Other optional fields depend on the key.

## Confirmed mismatch and bounded fix

Rust generic Jid parsing accepts text `@s.whatsapp.net`, `@lid`, `@g.us`
as empty-user server routes. Shared notification base_ack previously echoed
such malformed text as to/participant. Native fresh text parsing does not
produce that route. The regression failed before the fix and passes after.
The shared helper now refuses text whose last @ is at index0 in either field,
using its existing InvalidInput behavior. Five supported kinds are tested;
the picture actor tests both fields followed by a ping barrier, retaining
capture-before-write, redelivery and zero/nonzero PN/LID routing coverage.

This is **not full native malformed-envelope parity**. Native optional getters
can yield null and the generic writer omits those addresses; Rust keeps its
existing stricter valid-route requirement. Noncompliance logging is not a
blanket native refusal. No arbitrary namespace whitelist or global JID parser,
binary decoder, stored identity, deduplication or replay policy changed.
Cached typed values and all other malformed grammar remain separate.

## Required ID boundary

Retained0Tp.A0L calls A0M(name,null), returns any nonnull result, and throws LIL
only for null. This accessor adds no empty-string or128-byte check. Rust's
nonempty/128-byte notification ID guard is a local supported-input limit,
not a demonstrated server/native requirement. A0M coercion and full upstream
limits are not established by this one method. No ID bound was relaxed.

**Unknown:** all subtype/parser parity, effective handler gates, downstream
state adoption and live ACK acceptance. This fix creates no requests, retries,
recipient authority, native work completion or ban-risk guarantee.
