# Recipient-qualified incoming message routing

2026-10-11. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eight-topic recipe](apk/recipes/2.26.37.71-recipient-routing.json),
[DEX signatures, register flow and exception tables](apk/2.26.37.71-recipient-routing/targets.json),
and [complete REA route-validation envelope](rea/2.26.37.71-recipient-routing/core-route-validation.json).
Offsets are method-relative bytes. This trace establishes why an ordinary
incoming-chat selector cannot ignore `recipient`; complete own-device
synchronization remains unfinished.

## Native source and direction

`CnN.A02` reads typed `recipient` at `00cc..00dc`, followed by
`peer_recipient_pn` and `peer_recipient_lid`. `CaF.<init>` retains these as
A07, A06 and A04. Ordinary `CnN.A01` selects raw from as the remote JID and
raw participant separately; recipient does not replace the cryptographic source.
`CnN.A00` normalizes an own source through BO1 and CMm/CMn, preserving its
device when appropriate. The peer-category/config-dependent namespace choice
must not be replaced by an arbitrary PN/LID alias.

`CgC.A01` constructs CWj from the recipient information. CWj's constructor
stores the UserJid in A02, PN in A01 and LID in A00. CgC invokes `0Yo.A09`
and stores the mapped UserJid, or the original recipient when no UserJid is
returned, in Cih.A08 (`029e..02a2`). Full alias-mapping behavior and its saved
directory authority are **untraced here**.

The DEs validation loop rejects simultaneous recipient and participant
(`05cc..05ea`). For a nonnull recipient, it derives the source user and invokes
DEs.A01/08Z.BO0 (`0634..064c`); false throws `Invalid recipient from non peer
device`. BO0 accepts stored own PN or LID, as established in
[stored own identity](own-identity.md). This is an own-user check, distinct from
`category="peer"`. The registration and completeness of the DEs set remain
outside this trace; the retained method shows the check inside its iteration.

`Cih.A00` keeps original and normalized message keys separately. On its ordinary
path it initially builds both from the remote JID and supplied fromMe flag
(`0092..00a6`). With a recipient and null normalized chat, `00dc..00ec` selects
the recipient and branches back to the key builder with true. With a distinct
normalized chat, a present recipient also prevents the false-direction branch
at `00cc..00da`. Cih.A01 forwards both keys. Simply decrypting under the own
device and publishing a normal incoming event would lose this chat/direction
distinction. The special A0T/participant branch is a separate path.

## Rust correction and remaining implementation

**Reproduced local bug:** `inbox::session_sender` ignored a present recipient
and selected the source user as an ordinary incoming chat. That selector also
feeds direct row lookup, historical source inspection, poll context and CLI
saved-message inspection. The library and CLI receive paths share it.
`messaging::received_confirmation` already refuses recipient-qualified messages;
this was an inconsistent inbox projection, not evidence of a sent receipt.

**Implemented:** the shared ordinary selector now refuses any present recipient,
including empty/malformed values, before reading an incoming row or performing
Signal operations. Raw preservation still occurs; no historical row is deleted,
renamed or migrated. Peer-control routing already refuses recipients separately.
This is an explicit temporary support boundary, not the native acceptance policy:
the APK supports validated own-source recipient routes.

The failing-before selector regression covers PN/LID primary and LID companion
sources. A real valid Signal prekey fixture verifies refusal under both identity
policies, retained raw capture and prekey, no new peer identity/session, and no
incoming message row. The same original ciphertext then exercises ordinary
authentication, showing refusal did not consume its cryptographic state.

**Next implementation boundary:** validate saved own PN/LID authority, preserve
the actual own-device Signal address, reproduce recipient mapping and key
direction, handle payload wrappers, reconcile synchronized outgoing messages
without inventing send intents, and trace sender/read-self receipt selection.
[DeviceSentMessage](device-sent-wrapper.md) now traces its descriptor,
peer-device admission gates, destination checks and outer-context handling;
runtime admission and reconciliation remain unported.
No new device admission, live delivery or complete own-device synchronization
is proved by this correction.

2026-10-11 validation: all 938 workspace/all-feature tests, strict workspace
Clippy, warning-denying Rustdoc, default CLI tests, experimental CLI build and
formatting passed. The eight-topic, 13-file, 101,233-byte DEX bundle reproduces
exactly; four complete REA envelopes retain the complementary Java renderings.
