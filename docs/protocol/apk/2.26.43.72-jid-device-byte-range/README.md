# Original device getter versus byte-writer range — APK 2.26.43.72

**Confirmed selected static / fixed offline**, build 264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX method with exception table, exported
by the existing xtask fallback. Not runtime APK execution or a REA Evidence ID.

## Writer rejects, rather than truncates, original integers

[1r2.A03](byte-writer.txt) branches from byte PC 0000 on negative values and
0008 on values >=256 to byte PC 0018, constructing IOException. Only 0..255
reaches the mask and OutputStream.write. DEX relative offsets are code units
(two bytes), while displayed PCs are bytes; the negative +0xc target is 0018.

The retained [1r2.A00 writer](../2.26.43.72-group-missing-identification/current-jid-writer.txt)
selects AD-JID when getAgent()>0 or getDevice()>0. At 0160 it obtains the
original device getter and calls A03; it does not use the DeviceJid byte field.
The [current getter census](../2.26.43.72-group-missing-identification/current-device-overrides.txt)
and [constructor/parser trace](../2.26.43.72-lid-device-ack-text/README.md)
show 0EE/0EM retain the original int even though their base constructor checks
a narrowed signed byte. Thus an accepted constructor is not writer success.

- PN/LID device 256 or 355: constructors accept narrowed 0/99; positive device
  selects AD, then A03 rejects the original >=256.
- LID device -256: constructor accepts narrowed 0; 0EM.getAgent()==1 selects AD,
  then A03 rejects the original negative int.
- PN device -256 or -157: constructor accepts narrowed 0/99; base agent 0 and
  nonpositive original device select the compliant JID_PAIR branch. The original
  device is not written or reinterpreted as a positive device.
- Exact LID device 0 retains the previously fixed AD-JID flags 1/device 0;
  binary AD0 is reconstructed by the reader as UserJid and replies with a pair.
- Hosted devices require original integer 99 independently of byte narrowing.
  Constructor-null hosted strings are not covered by this writer-success guard.

## Bounded correction and verification

The shared known-unwritable predicate now checks accepted explicit DeviceJid
text for out-of-range original getters only when the native writer selects AD.
Notification routes reject these fields before constructing/writing ACKs;
receipt ACKs check only selected output fields. A DeviceJid-valued recipient
fails UserJid casting and is omitted, so it does not suppress a writable ACK.
Negative PN device text uses source-aware pair projection, including a bounded
fallback for raw text that the generic u16 parser cannot represent. Existing
parsed metadata, frozen selectors, raw captures and binary parsing are unchanged.

`wrapped_device_numbers_cannot_create_unwritable_ack_fields` failed before the
fix on from/12345:256@lid producing an ACK. It now covers from/participant,
notification refusal, omitted recipient and valid negative PN pair controls.
The picture receive actor checks capture-before-write for negative PN and exact
LID zero; positive/wrapped LID or PN and negative LID fields are refused, followed
by a successful ping barrier and clean shutdown. This proves receive survival
and supported ACK output, not native exception recovery or remote acceptance.

General nullable-constructor handling (including hosted 355), unknown subclasses,
all request families, native writer recovery and live acceptance remain open.
No new request family, identity/trust merge, device generation or account use.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/1r2;' --method A03 --signatures --exceptions
cargo test --locked -p whatsapp-api --all-features wrapped_device_numbers_cannot
cargo test --locked -p whatsapp-api --all-features picture_notification_ack_follows_capture
```

checksums.json binds every other file by bytes/SHA256. Reuse linked complete
constructor, getter and outer-writer exports rather than duplicate them.
