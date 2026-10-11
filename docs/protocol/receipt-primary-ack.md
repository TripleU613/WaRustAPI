# Primary address encoding in receipt ACKs

**Confirmed static, implemented/offline.** Subject: WhatsApp 2.26.37.71,
SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.

## Reader class and writer dispatch

The [Java AD reader and JID factory](ad-jid-decoding.md) reconstruct a PN/LID
user object from a binary zero-device address: 124.A03 omits the zero suffix,
124.A01 calls 0D1.A00 with that string, and caches the resulting JID. This differs
from literal text `user:0@server`, whose colon selects a device object.

[1Qe.A00](apk/2.26.37.71-ad-jid-decoding/ad-writer.txt) selects AD at 0132 when
getAgent or getDevice is positive (009c..00b6). Otherwise isProtocolCompliant
must pass, and the writer emits JID_PAIR (00c2..00fa). Its special interop
branches precede these tests.

Complete superclass override censuses cover 101,453 physical APK class definitions:

- [getAgent()I](rea/2.26.37.71-receipt-primary-ack/getAgent.txt): two declarations;
  Jid returns zero, DeviceLid subtype 0e1 returns one, inherited by Bli.
- [getDevice()I](rea/2.26.37.71-receipt-primary-ack/getDevice.txt): seven declarations;
  Jid returns zero, six device subtypes read the device field.
- [isProtocolCompliant()Z](rea/2.26.37.71-receipt-primary-ack/isProtocolCompliant.txt):
  only Jid declares it; getType values 2, 9, 11 and 8 fail, others pass.

**Inference from those confirmed branches:** a decoded binary zero PN/LID user
must use a pair in a generic receipt ACK. An explicitly constructed primary
DeviceLid may still use AD because its getAgent returns one. A global encoder
rule turning every zero AD address into a pair would therefore be incorrect.
Rust JidKind records wire encoding, not the APK's concrete runtime class.

## Runtime correction and scope

The shared receipt ACK renderer now converts binary AgentDevice PN/LID values
with agent/device both zero to plain user form for `to` and `participant`.
The specialized [UserJid recipient getter](receipt-user-recipient.md) already
returns that form. This fixes the runtime actor's outgoing control after its raw
capture and state commit. The input node and durable capture remain unchanged.
All callers use received_confirmation; message delivery/read/played producers
select its separate message branch and do not receive this conversion.

Nonzero device routing and the global encoder retain their existing behavior.
No new API route, field or CLI flag is introduced. Hosted primary aliases,
literal-text device-zero class validation, interop JNI codecs and complete
native message-to-DeviceJid conversion remain separate unfinished work.

## Reproduction and checks

```sh
for method in getAgent getDevice isProtocolCompliant; do
  signature='()I'
  if [ "$method" = isProtocolCompliant ]; then signature='()Z'; fi
  target/debug/xtask apk-virtual-overrides ~/Downloads/WhatsApp.apk \
    --class 'Lcom/whatsapp/infra/core/jid/Jid;' --method "$method" \
    --signature "$signature"
done
cargo test -p whatsapp-api --all-features receipt_ack
```

The independent literal binary regression carries LID zero in `from` and PN
zero in `participant`. It failed before the fix (`to` remained AgentDevice),
then passed with both ACK addresses Plain, unchanged ID/class and input node,
and an encoded/decoded ACK with matching user namespaces. Existing ACK tests
cover nonzero device, group, type, recipient and edit behavior.

No live request, account mutation, key/ID regeneration, reservation replay or
server-wait change ran. Offline checks do not establish server acceptance or
ban immunity. Real cross-release comparison still needs another APK.

Validation: 927 workspace/all-feature tests passed, plus default CLI/API-symbol
checks, strict workspace Clippy, warnings-denied Rustdoc, experimental CLI build,
formatting, CLI-reference/OpenAPI coverage and repository hygiene (2,708 local
links; 130 checksum bundles / 27,450,116 bytes). All three saved censuses
reproduced byte-for-byte from the rechecked pinned APK.

## All supported notification ACK callers — 2026-10-11

**Implemented/offline, same pinned subject.** The caller audit found the same
wire/class distinction in identity, device-list, count, group-dirty and
privacy-token ACKs. All six supported ACK paths now use receipts::ack_address
at their output fields. It is the existing receipt conversion moved into one
shared function, not a global parser or encoder rewrite.

**Confirmed static:** [CmG.A00](apk/2.26.37.71-identity-notification-ack/notification-address.txt)
reads root from and participant as Jid.class (0004..0014, 0020..0030), stores
those objects in CSu (004a, 0056), and
[CSu.A00](apk/2.26.37.71-identity-notification-ack/descriptor-build.txt)
copies them to Cb1. [Cp2.A05](apk/2.26.37.71-identity-notification-ack/ack-wire.txt)
renders those descriptor objects. The reader/writer/class census above therefore
applies to notification addresses too. This does not change the control's
original ID, type, dispatch ordering or admission policy.

The identity literal-wire regression failed before the change and now expects
plain LID to / PN participant with encrypt type retained and unchanged input.
A second independent literal LID fixture caught five mismatched output fields:
devices to, count to, group-dirty to/participant and privacy-token to. All now
return the plain user form. Existing fixtures cover server-only roots, concrete
device routes, strict input refusals, durable effects and uncertain query cuts.
Group-dirty deduplication/fingerprints, stored tokens, selected identity key-read
addresses and raw captures are untouched by the emission-only conversion.

The actor's stale reply diagnostic now says the notification ACK is handled
separately; it previously said deferred even though the traced identity dispatch
ACKs independently of adoption. No new API/CLI field or command is needed.
Hosted aliases, explicit text-zero class validation and other native codecs
remain unfinished; no fresh live window ran for this correction.

Follow-up validation: 929 workspace/all-feature tests, strict Clippy,
warnings-denied Rustdoc, default CLI/API symbols, experimental CLI build,
CLI-reference/OpenAPI coverage and formatting passed. Hygiene checked 2,714
local links and 130 APK checksum bundles / 27,450,116 bytes. The 14-topic
identity-notification ACK evidence reproduced exactly: 19 files / 186,462 bytes.
