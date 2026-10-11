# Message receipt primary device wire forms

**Confirmed static / implemented offline**, 2026-10-11. WhatsApp 2.26.37.71,
SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Copying a decoded user address did not always reproduce the device object
constructed by native message receipt builders. New confirmations now use
primary PN pairs and primary LID AD-zero addresses.

## Factory to writer

[UserJid.getPrimaryDevice](rea/2.26.37.71-status-receipt/primary-device.json)
calls 0DK.A01(this,0). [0DK.A01](apk/2.26.37.71-message-primary-device/device-builder.txt)
constructs user.agent:device@server, switching to hosted domains for device 99,
and calls [0DK.A03](apk/2.26.37.71-message-primary-device/device-parser.txt).
A03 calls [0D1.A00](apk/2.26.37.71-message-primary-device/jid-factory.txt)
and returns a DeviceJid unchanged, or obtains a UserJid primary device.

The generic factory sees the colon/dot, resolves its underlying user through
[0D2.A04](apk/2.26.37.71-message-primary-device/user-resolver.txt), parses device,
and constructs 0e1 for lid or 0DO for PN. Numeric device zero still constructs
that device class. Full [REA factory smali](rea/2.26.37.71-message-primary-device/jid-factory.json)
was returned because Java decompilation failed; no stub Java is treated as proof.

The complete [getAgent/getDevice/protocol census](receipt-primary-ack.md)
and [getType census](rea/2.26.37.71-receipt-user-recipient/jid-type-ancestry.txt)
resolve effective values. PN device 0DO has type 17, inherited agent zero and
its stored device value. LID device 0e1 has type 19, agent one and its stored
device. Both types pass inherited isProtocolCompliant. Thus
[1Qe.A00](apk/2.26.37.71-ad-jid-decoding/ad-writer.txt) selects:

| Native sender object | Wire form |
|---|---|
| PN primary device zero | JID_PAIR, because agent/device are zero |
| LID primary device zero | AD flag 1/device 0, because getAgent is one |
| PN/LID nonzero companion | AD with that device |

House Jid.agent is not this native class marker. It stays zero; the existing
encoder derives LID flag 1 from the namespace. No arbitrary agent flag is added.

## Native builder and socket path

[Cp2.A04](apk/2.26.37.71-message-primary-device/message-receipt.txt) receives
178.A0p DeviceJid and calls A00. [0y2.A00](apk/2.26.37.71-message-primary-device/message-writer.txt)
is the ordinary message-received writer: Cp2.A04(message,0) goes directly to
[1Rd.A04](apk/2.26.37.71-message-primary-device/node-send.txt).
The [physical caller inventory](rea/2.26.37.71-message-primary-device/message-builder-callers.txt)
finds 0y2.A00 and 1q7.A07. Complete initial message-class/178.A0p admission
remains outside this trace; the DeviceJid parameter-to-writer path is established.

[Status A06](apk/2.26.37.71-status-receipt/status-receipt.txt) supplies
Ayl.A0V(CjN.A04()) through DeviceJid.Companion.A00. The retained
[writer dispatcher](apk/2.26.37.71-identity-ack-writer/writer-dispatch.txt)
20e8..20fa calls A06 then 1Rd.A04, without a descriptor normalization between.
[Read A07](apk/2.26.37.71-message-primary-device/read-receipt.txt) supplies the
CM8.A05 DeviceJid; dispatcher 1f48..200e calls A07 then 1Rd.A04. There are
[two CM8 constructor sites](rea/2.26.37.71-message-primary-device/read-holder-callers.txt),
both in 0zd.A04; the complete producer policy is not claimed here.

[CmC.A06](rea/2.26.37.71-status-receipt/receipt-route.json) selects the supplied
device as direct destination or group/broadcast participant.
1Rd.A04 calls [1Qe.Ck7](apk/2.26.37.71-message-primary-device/frame-writer.txt)
with mode 1; it serializes the supplied node through A01, then encrypts/writes it.
[1Qe.A01](apk/2.26.37.71-core/node-writer.txt) passes actual attribute JID objects
to A00. No inspected call replaces the sender device object with a UserJid.

The separately inspected [0h8.A02](rea/2.26.37.71-message-primary-device/key-normalizer.json)
normalizes primary devices in Cb1 descriptor keys. That is not an instruction
to rewrite all already rendered message/read/status nodes. ACKs decoded as users
still use their [user pair form](receipt-primary-ack.md).

## Runtime and durable behavior

The shared received_confirmation message branch changes only encoding kind at
zero agent/device: PN becomes Plain, LID AgentDevice. Direct to and group/status
participant use that conversion. Existing nonzero devices, namespaces and user
components are preserved. Receipt ACKs take their separate earlier branch.
Automatic/explicit reads and opened receipts use the same message builder, so
newly generated receipts receive this native form too. Saved historical wires
are not migrated or replayed.

The PN/LID direct/group/status matrix failed before the fix (binary PN zero
remained AD), then checks both incoming Plain and AD-zero forms, unchanged input,
components, and encoded/decoded output. Existing device and ACK regressions
remain in place. A new primary-LID read test failed before the fix (Plain), then
checks AD-zero reservation, reopening, no uncertain replay, settlement by a plain
server ACK, duplicate rejection and unchanged saved incoming route.

Receipt ACK correlation already ignores encoding kind while retaining all
address components, ID, type and nullable participant; it needed no weakening.
No account identity/key, request ID, existing reservation or server wait changes.
No API/CLI surface is added; current receive/read behavior emits corrected bytes.

## Reproduction and limits

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-message-primary-device.json \
  --output docs/protocol/apk/2.26.37.71-message-primary-device --check
cargo test -p whatsapp-api --all-features primary
```

Nine topics export fourteen files / 64,618 bytes; twelve full REA envelopes and
two caller inventories accompany them. Remap/review topics for another APK
before comparing cached bundles; no automatic anchors or real second release
are claimed. Full native admission, hosted/interop sender classes, PN/LID
migration, Bp1 sender policy and alternative Android transport remain open.
No fresh live window ran, so server acceptance is unverified.

Validation: 931 workspace/all-feature tests passed. Three older decryption,
automatic-read and group-read fixtures expected a plain primary LID; only their
expected outgoing address was corrected to AD-zero, retaining their identity,
plaintext, suppression and ACK guards. Strict workspace Clippy, warnings-denied
Rustdoc, default CLI/API symbols, experimental CLI build, CLI-reference/OpenAPI
coverage, formatting and diff checks passed. The nine-topic export reproduced
exactly, all twelve REA envelopes match the rechecked APK digest, and repository
hygiene checked 2,753 local links / 132 checksum bundles / 27,550,066 bytes.
