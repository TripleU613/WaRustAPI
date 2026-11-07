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
No new API route, field or CLI flag is introduced. The later literal-zero
section below narrows the former text-device-zero gap. Hosted primary aliases,
interop JNI codecs and complete native message-to-DeviceJid conversion remain
separate unfinished work.

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


### Current-build ACK zero-device normalization (2026-11-05)

**Confirmed static, APK2.26.43.72 / offline regression:**123.A01/A02/A04
reconstruct binary zero-device PN/LID without a device suffix;1r2.A00 chooses
pair for zero compliant getters and AD for positive device/agent. The existing
picture actor test now covers zero/nonzero PN/LID sender and participant forms.
Native domain flags are distinct from Rust's agent field. No runtime policy
change; all subtype/hosted/interop behavior and live acceptance are not claimed.
[Current source and census](apk/2.26.43.72-group-missing-identification/README.md).


## Writer compliance follows selected receipt context (2026-11-06)

**Confirmed local fix / offline:** [known writer compliance evidence](apk/2.26.43.72-jid-write-compliance/README.md)
now guards selected ACK to/participant and copied recipient fields. Plain service,
gdpr and valid temp generic fields no longer produce unwritable ACKs. Nonuser
gdpr/temp recipients fail the UserJid cast and are omitted; they do not suppress
an otherwise supported ACK. A parse-error retry487 removes recipient/edit in its
fresh context, whereas unsupported-version487 copies original metadata. The new
regression checks this distinction with a service recipient and passes only when
compliance is checked after context selection.

The earlier generic-participant test's gdpr expectation certified only field
presence, not writer success. It is replaced by explicit unwritable-participant
and omitted-recipient controls. Native writer recovery, complete address grammar
and live acceptance remain open. No fact/identity/replay policy, request family
or global encoder behavior changed.

## Literal zero-device ACK wire class (2026-11-06)

**Confirmed static / fixed offline, APK 2.26.43.72:** literal text
`12345:0@lid` is a DeviceLid (0EM), whose `getAgent()` returns 1.
The native 1r2.A00 writer therefore emits AD-JID flags 1/device 0. In contrast,
123.A01/A02/A04 reconstruct incoming binary AD0 as an ordinary user, whose
reply uses JID_PAIR. A prior generic normalization incorrectly flattened both.
PN Device0 has agent 0 and still writes JID_PAIR.

Sources: [parser](apk/2.26.43.72-message-ack-chat/jid-parser.txt),
[current getter census and writer](apk/2.26.43.72-group-missing-identification/README.md),
and [original integer device storage](apk/2.26.43.72-lid-device-ack-text/README.md).
Source-aware projection now applies to selected receipt ACK from/participant
and generic picture, identity, devices, count and groups_dirty ACK fields.
It recognizes original exact-zero signed/BMP decimal forms and ignored dot
qualifiers using the existing native text classifier. Device recipients still
fail the receipt UserJid cast; token authority admission is unchanged.

The regression first failed on the Plain versus AgentDevice kind, then passed
with actual AD-JID bytes `f7 01 00`. Additional notification-builder tests cover
literal, ignored-dot, signed and Unicode zero forms. Existing binary-zero tests
retain the opposite pair outcome. Incoming attributes remain unchanged;
already-parseable metadata and persisted dirty-notification selectors are not
normalized. A bounded metadata fallback admits formerly unparseable exact-zero
forms. This is ACK output projection, not full sender/fact canonicalization.
Original device integers 256/-256, hosted-zero validity, unknown subclasses,
complete grammar, runtime writer recovery and live acceptance remain unverified.
No account operation or new request family.

## Original device getter range at ACK writing (2026-11-06)

**Confirmed static / fixed offline:** [complete byte writer and outer path](apk/2.26.43.72-jid-device-byte-range/README.md)
resolve the wrapped-device writer gap. 1r2.A03 requires 0..255; 1r2.A00 passes
original getter integers, not narrowed constructor bytes. Positive 256/355 PN/LID
and negative LID values that pass construction can therefore be unwritable.
The selected-output guard now refuses these ACK fields. Omitted device-valued
recipients remain harmless; a valid negative PN device instead takes JID_PAIR
because its original getter is nonpositive and agent is zero. A bounded raw-text
projection reproduces that pair without changing captures or existing selectors.

The field regression failed on an ACK built with 12345:256@lid before correction.
The receive actor exercises refusal followed by a ping barrier, and capture-before-
write for valid negative PN and exact LID zero. Global encoder/model, native
exception recovery, constructor-null hosted strings, other classes and live
acceptance remain separate; no account operation or new request family.

## Nullable receipt context and participant output (2026-11-06)

**Confirmed static / fixed offline:** [complete descriptor construction trace](apk/2.26.43.72-ack-nullable-context/README.md)
shows F0R.A00/Efa.A00/Eoh do not assert a sender or participant; F3e.A05 omits
null fields. This does not establish every handler's null-sender admission.
The supported Rust ACK path retains its sender requirement. Its optional
participant now honors known constructor-null PN/LID/hosted text rather than
copying the broader generic parser's result. Nonnull Device256 writer failures
remain distinct and still refuse ACK construction. Regression/actor checks cover
actual encoding, durable capture and receive continuation. Fact attribution,
notification handlers, typed constructors and live acceptance remain open;
no account operation, stored selector or new request-family change.

## Related media application error ACK (2026-11-06; research only)

A distinct native media/plaintext validation path can produce message ACK error491
and meta.failure_reason, subject to processing/retry/BX2 predicates. It is separate
from receipt parsing's487 and does not authorize a generic Rust unsupported-message
ACK. No runtime integration was added.
[Full evidence and missing bindings](apk/2.26.43.72-media-parser-error-boundary/README.md).
