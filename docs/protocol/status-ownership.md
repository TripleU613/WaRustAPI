# Status receipt sender type follows own identity

**Confirmed static / implemented offline**, 2026-10-11. WhatsApp 2.26.37.71,
SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The old builder emitted type=sender for every received status. The native flag
means fromMe; peer status takes a null type. The actor and legacy CLI now make that selection
using existing saved own PN/LID authority, separately from decryption admission.

## Holder and parser authority

[Cp2.A06](apk/2.26.37.71-status-receipt/status-receipt.txt) tests Bp1.A05 at
0000..0008: true selects sender, false selects null at 005c. Both branches keep
class=status, signed sts=-1 and mode=0.

The [Bp1.A05 field scan](rea/2.26.37.71-status-ownership/sender-field-refs.txt)
finds nine direct references and one write, in its constructor. The
[constructor](apk/2.26.37.71-status-ownership/status-holder.txt) copies the last
boolean to A05. The [constructor caller scan](rea/2.26.37.71-status-ownership/holder-constructors.txt)
finds only [CRF.A00](apk/2.26.37.71-status-ownership/parsed-builder.txt), which
requires CRF.A0D and explicitly diagnoses a missing value as fromMe.

[1RE.A01](apk/2.26.37.71-status-ownership/status-parser.txt) is the node parser.
It computes the remote and participant through CnN, tests group or status kind,
then tests participant ownership. At 00da..00e6,
[CnN.A04](apk/2.26.37.71-status-ownership/device-own.txt) accepts a DeviceJid
and invokes BO0 on its userJid. If that fails, 00f8..0110 tests UserJid and
BO0 directly. An owned participant selects boolean true; the unowned branch
sets false at 01c4. At 0160..0170 true is stored into CRF.A0D. The false path
at 019c..01c0 only reads CaF.A0K for newsletter type 21; other types store false.
No peer from_me/sender_pn claim is ownership authority for status@broadcast.

The retained [REA Java parser](rea/2.26.37.71-status-ownership/status-parser.json)
has duplicated/uninitialized-looking assignments; branch conclusions above use
complete structural DEX, not that Java formatting. The constructor and CRF
builder's independent complete REA methods agree with the field flow.

[0DE.A0i](apk/2.26.37.71-status-ownership/status-type.txt) tests type 5, which
matches 8oR status in the complete [getType census](rea/2.26.37.71-receipt-user-recipient/jid-type-ancestry.txt).
A0n/A0m/A0W separately cover group types 1/2 and interop group 28. A0b tests
newsletter 21; A0Q's broadcast set is 3/5/6. A broadcast-list object has type 3
and must not be assumed to use the status parser merely because its domain is
broadcast. [Ordinary list confirmations](broadcast-receipt.md) now omit status
class; their complete receive admission remains unported.

[BO0/BO1 implementation closure](own-identity.md) establishes stored self PN/LID
comparison, not contact/directory aliases. Device numbers are stripped by the
native own-user predicate; an own companion still has fromMe=true.

## Runtime callers and guards

The context-free received_confirmation builder now renders remote status without
type. The actor and legacy CLI receive loop use received_confirmation_for_account after durable capture
and authenticated message processing; it adds type=sender when the participant
matches AccountRecord.own_jid or the existing validated group_send::own_lid.
The latter reads authenticated login/legacy own-LID authority and refuses
malformed/conflicting records. It never consults a peer mapping. Sender type is
inserted before participant, matching native append order. Other confirmation
branches and original status/device/ID fields keep their existing behavior.

When saved own-LID authority must be read, corrupt/conflicting records cause
the contextual builder to return an error;
the actor stops that epoch and the CLI returns an error after capture, without guessing ownership or writing
the control. The saved authority is not repaired, cleared or replaced. A direct
confirmation does not read this unrelated LID state. The existing helper is
public so the separate CLI crate can share it. Its Rustdoc requires durable
capture and authenticated/decrypted message admission before confirmation;
it does not write or queue. No ownership override, API route or CLI option is
added. The context-free builder remains available for existing offline callers.

The CLI boundary regression initially failed to compile because the helper
was crate-private. It now checks own companion PN and a remote status carrying
a forged from_me=true claim through the public helper. Existing contextual
API tests cover own/remote LID and refusal on conflicting saved authority.

The remote full-shape regression failed before the type removal. Contextual
tests cover own and remote PN/LID, primary and companion identities, forged
from_me/sender_pn metadata, native attribute order, absent sts, conflicting own
LID refusal and unchanged direct behavior. Raw nodes and saved keys/IDs/waits
are not changed. No live status sample is claimed.

## Complete field audit and limits

[CRF.A0D field references](rea/2.26.37.71-status-ownership/from-me-field-refs.txt)
find seven writes plus CRF.A00's read. All writing methods were inspected and
retained. 1RE.A00 reconstructs status from BO0(participant); Cj1.A02 uses the
same device/user ownership split. Other bodies are retained as bounded evidence:
1RY.A06 and 9wp.A00 use newsletter input's true string; D28.Bz0 copies an addon
message key's A02, and DG0.CEt copies imported protobuf key.fromMe. Full import,
newsletter and callback admission is not inferred from these assignments.

Recipe reproduction:

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-status-ownership.json \
  --output docs/protocol/apk/2.26.37.71-status-ownership --check
cargo test -p whatsapp-api --all-features status_
```

All eighteen full REA envelopes and three physical inventories are retained.
Native callback registration/properties, alternative transport, full broadcast-list
admission, own ordinary-message policy and full newsletter/import support
remain separate work. A second APK is still needed for real release comparison;
these recipe topics have no automatic anchors. No fresh live window ran.

Original actor validation: 932 workspace/all-feature tests passed, plus strict workspace
Clippy, warnings-denied Rustdoc, default CLI/API symbols, experimental CLI build,
CLI-reference/OpenAPI coverage, formatting and diff checks. Twelve topics
reproduced exactly: seventeen files / 68,094 bytes. All eighteen full REA
envelopes match the rechecked subject digest. Hygiene checked 2,770 local links
and 133 checksum bundles / 27,618,160 bytes. No fresh live result is claimed.

CLI alignment validation: 935 workspace/all-feature tests passed, including
the cross-crate account-confirmation regression. Strict workspace Clippy,
warnings-denied Rustdoc, default CLI/API symbol tests, experimental CLI build,
formatting and diff checks passed. The CLI keeps its existing decrypted/duplicate
gate and durable capture before confirmation; errors propagate before its write.
No new wire policy or live sample was introduced by sharing the helper.
