# Status confirmation signed timestamp correction

**Confirmed static / implemented offline**, 2026-10-11. WhatsApp 2.26.37.71,
SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The prior implementation sent sts=65535 on a status delivery confirmation.
That value came from misreading the signed const-wide/16 literal 0xffff.

## Exact native producer and append gate

[Cp2.A06](apk/2.26.37.71-status-receipt/status-receipt.txt) loads -1 at 0048,
mode zero at 004c and null recipient at 004e. Its invoke-static/range v1..v11
at 0052 has the full descriptor retained. [Cp2.A00](apk/2.26.37.71-status-receipt/receipt-builder.txt)
uses cmp-long against zero (00ca), then if-lez at 00ce to skip sts. Thus neither
A06 branch emits sts. Sender type is selected by Bp1.A05; both branches carry
class=status. No positive timestamp/constant may be inferred from this path.

Independent [REA A06](rea/2.26.37.71-status-receipt/status-receipt.json) and
[REA A00](rea/2.26.37.71-status-receipt/receipt-builder.json) complete Java
representations agree with the structural DEX bytes. Full envelopes include
subject/engine metadata. Rust deletes the incorrect sts append and corrects
source comments and status documentation. No new API/CLI field or option exists;
this changes the existing receive confirmation on the wire.

The existing full status-shape regression was changed to require absent sts.
It failed against the old implementation and passed after the removal. It still
checks sender type, status class, original ID, destination, poster device,
absent recipient/meta and separate group shape. Saved identities, keys, session
state, raw capture, original IDs, reservations and server waits are unchanged.

## Reproduction

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-status-receipt.json \
  --output docs/protocol/apk/2.26.37.71-status-receipt --check
cargo test -p whatsapp-api --all-features remote_status_receipt_has
```

Two topics export seven files / 35,332 bytes. A later APK release requires
remapping/reviewing these selected methods before comparing cached bundles;
there are no automatic topic anchors. Only one release is available here.

## Trace boundaries retained

The initial device-route trace also retained full
[CmC.A06](rea/2.26.37.71-status-receipt/receipt-route.json),
[UserJid.getPrimaryDevice](rea/2.26.37.71-status-receipt/primary-device.json) and
[Cp2.A01 error builder](rea/2.26.37.71-status-receipt/error-builder.json).
getPrimaryDevice constructs device zero through 0DK.A01; CmC.A06 can use the
supplied DeviceJid for direct destination or group/broadcast participant. This
refutes the old claim that equal textual rendering proves equal wire encoding.
The [writer census](receipt-primary-ack.md) distinguishes DeviceLid from UserJid.
Complete device factory/subtype/transport normalization must be traced before
changing message receipt primary address encoding. This chunk makes no such
conversion. Cp2.A01's error shape is source evidence, not new error support.

Bp1.A05 producer authority, primary device encoding and full native status
receipt policy remain unported. No live window ran; server acceptance and ban
immunity are not established by static/offline checks.

Validation: 929 workspace/all-feature tests passed, plus strict workspace
Clippy, warnings-denied Rustdoc, default CLI/API-symbol checks, experimental CLI
build, CLI-reference/OpenAPI coverage, formatting and diff hygiene. Repository
hygiene checked 2,728 local links and 131 checksum bundles / 27,485,448 bytes.
The two-topic export reproduced exactly; all five complete REA envelopes have
the rechecked pinned APK subject digest. No fresh live result is claimed.

Follow-up: [primary device factory and writer closure](message-primary-device.md)
now establishes and implements PN/LID primary sender wire forms in the shared
confirmation builder. The earlier trace boundary above is historical; full
Bp1.A05 producer/admission remains unported.

Follow-up: [status ownership and sender selection](status-ownership.md) now
traces Bp1.A05 to parsed fromMe and the own-user predicate. Remote statuses omit
type; the authenticated actor supplies sender only for saved own PN/LID. The
earlier producer boundary above is superseded for this supported receive path.
