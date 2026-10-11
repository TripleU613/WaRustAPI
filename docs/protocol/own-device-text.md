# Own-device wrapped text receive

2026-10-11. **Confirmed static** for APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipient routing](recipient-routing.md) establishes source validation and
outgoing key direction; [DeviceSentMessage](device-sent-wrapper.md) establishes
wrapper admission, destination and outer context. This chunk adds
[a recipe](apk/recipes/2.26.37.71-own-device-text.json),
[DEX signatures/branches](apk/2.26.37.71-own-device-text/targets.json),
[complete recipient-mapping envelope](rea/2.26.37.71-own-device-text/recipient-namespace-selection.json),
and [direction caller census](rea/2.26.37.71-own-device-text/direction-callers.txt).

## Recipient mapping and receipt boundaries

**Confirmed:** `0Yo.A09` returns the original recipient except for supported
ordinary non-bot namespaces requiring migration conversion. LID with A0H false
selects supplied PN; PN with A0H true selects supplied LID. CgC.A01 uses the
original recipient when the returned value is not a nonnull UserJid, including
missing conversion aliases. Thus, absent alternate recipient attributes, its
fallback is unambiguous without inventing a migration flag.

`0Yo.A0H` depends on 0H9's cached state and message-store readiness. When cached
state is absent and the store predicate A0C is false, it returns true; otherwise
it returns 0H9.A01 after the retained readiness/logging branches. Persistence,
the complete 0H9 predicate and recovery state are untraced here. We do not infer
global migration completion from an authenticated own LID alone.

The complete Cih.A03 call census has 11 sites. The additional network parser
DE4.CEv is a bot-specific override guarded by a special source, configuration
28110 and meta fields. It does not make ordinary recipient presence equivalent
to an original fromMe flag. Bp2.A0P sets the domain source device from CjN.A00
and sets 178.A0y when the **original** key's fromMe flag is true. Cp2.A04 uses
that flag for `type=sender` and recipient selection, then passes the device
through CmC.A06. Normalized outgoing direction alone is insufficient evidence
for that receipt. The full domain/receipt decision and primary echo
reconciliation remain open; this chunk does not add a confirmation builder.

## Runtime receive and API/CLI behavior

**Implemented:** shared `inbox::process`/`process_with_policy` now route a
recipient-qualified text stanza into `own_device::process`. The supported
route requires a registered primary account, an agent-zero nonprimary source
in saved own PN or validated own LID authority, a primary PN/LID recipient,
and no participant, edit, peer category or alternate recipient metadata.
Device/agent-qualified textual recipients, including explicit `:0`, refuse.
This is a support boundary, not a claim that the APK refuses all other forms.

The exact own-device identity pin and Signal session must already exist.
Only one `enc type=msg v=2` is admitted; prekeys and unknown devices remain raw
captures. No device admission, alias learning, key replacement or directory
lookup occurs. Valid Signal authentication is followed by wrapper checks:
exact primary destination equality, explicit destination/body presence, and
one supported inner conversation or extended-text field. Outer Message context
replaces inner context as in CjY.A01. Unsupported/malformed/mismatched wrappers
do not advance the durable ratchet. Damaged ciphertext returns Unsupported so
the existing connection loop cannot emit an ordinary direct retry for it.

The write transaction rechecks account generation, registration ID, Signal
public identity, own-source authority, held session and pin, and row absence.
It commits the advanced session, original wrapper/route, projection marker and
`SyncEvent::NewMessage` together. The event key uses the recipient chat,
`Direction::Outgoing`, no sender, and the original stanza ID. The ordinary text
projector supplies quotes, expiry handling and timestamps. No send intent,
request ID, new account identity or server wait is manufactured.

Rows use `received-own:<recipient>:<id>` to keep outgoing synchronization apart
from incoming rows with the same ID. Redelivery from the same held own-device
source deduplicates, including after reopen. A different source cannot claim
that row without a future reconciliation implementation. The existing CLI
saved-inbox inspection includes these rows and uses redacted wrapper summaries.
The normal event API exposes the outgoing message; no new command or flag is
required.

Local `clear_chat` now accepts LID chats and strips authenticated plaintext
from incoming and synchronized own-device rows while retaining source/route
and cleared markers. Cleared synchronization does not republish on redelivery.
Other chats are untouched. This follows existing local clear semantics:
historical journal events remain historical, and no companion app-state clear
patch is transmitted.

Delivery confirmations, automatic reads and retry recovery for this route
remain unimplemented and emit no traffic. Server redelivery may therefore
continue; it does not duplicate the committed event. This is functioning offline
receive/projection through a held session, not complete own-device messaging.

## Tests and remaining work

A real Signal fixture failed with Unsupported before this route existed. It
now verifies authenticated wrapped text, exact outgoing chat/direction/body,
no sender, no outbox intent, original-wrapper retention, duplicate/reopen,
destination mismatch without ratchet loss, unknown/foreign/primary source
refusal, damaged-ciphertext refusal without a retry reservation, and clearing
without republication. Its sessions/pins are explicitly seeded offline; it
does not prove live linked-device bootstrap.

Independent unit fixtures cover PN/LID own-source selection, absent own LID,
agent/device/special-recipient rejection, alternate metadata refusal, missing
or malformed wrapper, nontext inner content, malformed context, and outer
context replacing an existing inner context. Local clear fixtures cover both
incoming and synchronized LID rows, retained markers, idempotence and untouched
foreign chat rows.

Validation: all 942 workspace tests passed with all features; strict workspace
Clippy, warnings-denied Rustdoc, 26 default CLI tests, experimental CLI build,
formatting and diff checks passed. The seven-topic APK bundle reproduced all
12 files (51,673 bytes) with `--check` against the pinned APK.

**Unported:** synchronized media and control effects, namespace mapping with
alternate attributes, migration-state import, special/hosted routes, admission
and lifecycle of new own devices, primary echo reconciliation with existing
outgoing intents, and native confirmation/read-self/retry behavior. No fresh
live own-device text sample was observed. No claim of ban immunity is made.
