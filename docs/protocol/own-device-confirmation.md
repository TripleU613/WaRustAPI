# Own-device text delivery confirmation

2026-10-11. **Confirmed static** for APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Extends [authenticated own-device text receive](own-device-text.md).
[Recipe](apk/recipes/2.26.37.71-own-device-confirmation.json),
[DEX signatures and register flow](apk/2.26.37.71-own-device-confirmation/targets.json),
[ordinary postprocess](rea/2.26.37.71-own-device-confirmation/ordinary-postprocess.json),
and [receipt encoder](rea/2.26.37.71-own-device-confirmation/receipt-node.json)
retain the evidence. Static inspection does not establish server acceptance.

## Key transfer correction

The earlier original/normalized labels for Bp2.A07 and Bp2.A08 were reversed.
Cih.A01 loads A0X into v9 (original) and A09 into v8 (normalized). At
0194..019c it transfers them into invocation registers v20/v21. With the
complete constructor descriptor, these are the fourth/fifth reference arguments.
Bp2.<init>'s v17/v18 are those arguments: 0028..0030 stores the original
in **A07**; 00f2..00fe stores the normalized in **A08/C6a.A00**.

Cna.A01 feeds the normalized key into CRD.A0R and the original into CRD.A06.
CRD.A00 selects A06, falling back to A0R, for the fifth CoT constructor
argument. CoT stores this as **A0B**, not A0A. The sixth argument remains
CRD.A0R and becomes **CoT.A0A**, the normalized key used by the ordinary
message mapper. The retained full signatures resolve both argument transfers.
[Group envelope notes](group-envelope-own.md) and
[wrapper validation notes](device-sent-wrapper.md) now use the corrected labels.

## Receipt shape

178.<init> initializes A0y to false. C1hF.A03 calls Bp2.A0O, which invokes
A0P. A0P sets A0p from CjN.A00 (actual source device). Under A08/C6a.A00's
fromMe guard, it sets status 4 and A0y=true. The separate false branch calls
CW6 with the selected sender user. Recipient-qualified ordinary own-device text
has the normalized recipient-chat/fromMe key established by Cih.A00, so it
takes the true branch.

Cp2.A04 selects `sender` when A0y is true. For an ordinary user chat, it
selects that chat as the receipt's recipient. CmC.A05's special flag route
is separate; this port refuses the alternate/special metadata that requires
it. Cp2.A00 passes the actual source device through CmC.A06. Its ordinary
branch replaces `to` with that device. Cp2 emits attributes in this order:
`to`, `id`, nonnull `type`, optional distinct `participant`, then `recipient`.
Its signed -1 server timestamp produces no `sts`.

On the ordinary text route, the sender field is absent: construction starts
with null A0g and the true A0P branch does not call CW6. B1Q reads A0g;
Cp2.A00 therefore emits no participant. The exact
[A0g write census](rea/2.26.37.71-own-device-confirmation/sender-field-writes.txt)
finds only 178.CW6; the complete
[setter caller census](rea/2.26.37.71-own-device-confirmation/sender-setter-callers.json)
retains 136 static sites, including separate history, control and rich-response
paths. Those are not admitted as ordinary text receipt routes here.

0zH.A0R also builds a receipt placeholder from A08/C6a.A00 and applies A0O.
0zH.A01 creates handler message 9; the existing
[writer dispatch](apk/2.26.37.71-message-primary-device/message-writer.txt)
calls Cp2.A04. This independently connects the key/flag to delivery encoding.

## Shared API and CLI behavior

`messaging::received_confirmation_for_account` now handles the supported
recipient-qualified own-device text route through a private own_device helper.
It reuses the receive shape and saved own PN/LID authority, and requires the
same-source `received-own:<chat>:<id>` checkpoint with status `decrypted`.
Unauthenticated captures, damaged or mismatched text, unknown/foreign sources,
different source markers, cleared rows and unsupported metadata return no
confirmation. Inspection makes no state changes.

The result is `<receipt to=<actual own device> id=<original ID> type=sender
recipient=<primary chat>/>`. The context-free confirmation builder continues
to refuse recipient-qualified messages. The delegation is gated on Signal
crypto. No new command, public operation, dependency, request ID, wait
or receipt reservation is introduced.

The existing API connection loop and CLI receive/send loops call the shared
account-aware builder only after a decrypted/duplicate result. Capture and
ratchet/message/event commit still precede the write. A genuine server
redelivery can be confirmed again, using the existing connection policy.
Read-self, retry recovery, media/control synchronization, namespace migration
with alternate attributes and new own-device admission remain unported.

## Validation

The real Signal fixture failed before this change because the builder returned
None. It now checks the exact sender receipt, no confirmation before commit,
the same receipt after reopen, and refusal after clearing. A PN-source unit
fixture covers pending/cleared/different-source markers, foreign source/chat,
unsupported metadata, and unchanged saved state. These are offline fixtures;
no live own-device delivery or server acceptance was checked in this chunk.

All 943 workspace tests with all features, strict Clippy, warnings-denied
Rustdoc, 26 default CLI tests, the experimental CLI build, formatting and diff
checks passed. The 17-topic DEX bundle reproduced all 22 files (102,788 bytes)
with `--check` against the pinned APK.

An additional `cargo check -p whatsapp-api --no-default-features --features
experimental-noise,experimental-transport` fails with 16 feature-gating errors
in existing group receipts, directory queries, receive, receipts and retries.
A clean archive of baseline commit 1e2b00a reproduces the same 16 missing-module
errors. This optional combination is not validated as a working build by the
receipt change; it needs a separate feature-boundary fix.
