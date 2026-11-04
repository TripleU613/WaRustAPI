# Current primary pairing input and reply authority

2026-11-04. **Confirmed static** unless labelled unknown, WhatsApp Android
**2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Evidence](rea/2.26.43.72-primary-pairing-auth/README.md).
This continues [own-companion ingress](own-companion-ingress-current.md) and
[read-self authority](read-self-authority-current.md). No Rust code, account
state, pairing request or receipt send changed. This closes selected input and
request-correlation mechanics; authenticated own-device authority is still incomplete.

## QR input is structural input, not an authenticated own-device record

EyO.A02 splits a QR string on commas, drops trailing empty components and
requires at least three remaining components. It rejects a reference beginning
with `CAPI_`; C3q1.A1Y confirms this is startsWith. It decodes component1 as
the public key with Android Base64 flag0. With at least four components it
also decodes component2 as identity bytes, prefixes the Signal type byte5,
parses through F3k.A02 and wraps Ee1; component3 is decoded as the prior secret.
A later component can select platform metadata. Invalid Base64 or identity
parsing returns null on the selected caught paths. Exactly three components
can produce Esd with absent identity and secret plus its legacy third string.

The selected parser does not establish remote ownership, ref freshness,
operator approval, a fixed public-key length, or a 32-byte secret requirement.
Its Esd constructor has other callers (DGr.Bpd and Fcs.run), so the QR-string
path is not the universal origin of all pairing inputs.

F1G.A01 copies Esd's ref, public key, identity, secret and platform into EXq.
When a prior secret is present it requires nonempty bytes and derives a
32-byte handoff value with 1oO.A00. The selected primary path rejects absent
identity or secret with diagnostic -9 and mismatched hosted encryption/platform
classification with -13. The old three-component parse is therefore not enough
for this primary branch. This does not prove it is rejected everywhere.
EXq's constructor merely assigns fields; constructing it is no validation.

EGv.A00 supplies an alternative EXq origin from a DiD message carrying ref,
companionPublicKey, companionIdentityKey and advSecret. It parses the typed
identity and returns null on ECt. The preceding message decryption, producer
and user authorization for this alternative remain **unknown**.

F1G.A04 refuses another active FUY or existing 0Vy pending Eo9, requires
10V.A01's timestamp to differ from -1, refuses the selected hosted type on
consumer, and calls A05 before scheduling the preparation worker. These are
specific gates, not permission to invent registration or pairing state.
The preceding bundle retains A05's old-FUY deactivation and the later
FeN12 active-generation check before Eo9 creation.

F1g.A01 derives an AES-GCM SecretKeySpec through 1oO.A02 with length32.
It does not prove that every recovered Fdy Java variable named Key denotes an
RSA public key. The full Fdy field-to-crypto-to-wire mapping is **unknown**;
the apparent pem/rsa2048 branch needs a checked constructor and writer trace
before implementation. No cryptographic algorithm is selected from that label.

## Request ID correlation is now traced through callback consumption

The preceding bundle's Fdy nonzero branch registers its specific FUY with a
generated request ID through 0Sf.A0Q, discriminator219 and timeout32000.
Current 0Sf.A0Q calls A06. When connected, A06 registers through
0ln.A08 before queueing the node; when disconnected it removes the reservation
through A0E and calls BsS. It does not establish a successful write.

0ln.A08 synchronizes registration, refuses an ID already in active A0H, and
allows an existing waiting A0I value only when it is the reserved placeholder.
It schedules the keyed timeout when positive. The writer's existing checked
0v0.A00 at00a0..00b6 calls 0uv.C0e for a nonnull IQ ID before writer dispatch.
0uw.C0e forwards to 0ln.A0C. A0C moves the waiting callback from A0I into A0H,
asserting that no active entry exists, and attaches elapsedRealtime. Activation
therefore occurs before downstream wire writing; it is not delivery proof.

For the IQ parser subclass1rD, 1rA.A03 reads type/id/xmlns. It requires type,
requires a nonnull ID for result, handles result/error-specific local callbacks,
then forwards the node and same ID through 1lZ.A1E ->1lW.C0g ->1lY.C0g ->
0ln.A0H. Checked DEX at0000..00e6 confirms the selected result/error routing;
recovered Java warnings elsewhere in the method are not interpreted.

A0H finds the active callback by that ID and dispatches or posts handler message2.
0m2.handleMessage case2 calls 0ln.A02, which removes the active callback under
the lock before dispatch; missing callbacks are ignored. A01 invokes CK4 only
for exact type=result, Bvj for exact error, and diagnoses unexpected types.
A03 and A0D retain timeout/cancellation removal paths. This is concrete
ID-and-active-request correlation, including single-consumption behavior.

**Source authority remains unknown.** No explicit from comparison appears in
the selected 1rD result branch, 1lZ/1lY forwarding or 0ln callback lookup.
This bounded negative is not proof the transport accepts arbitrary sources:
the connection reader, stanza demultiplexer, session/account generation and
server authentication before Ay8 were not closed here. Result type plus ID
must not be described as complete authenticated pairing authority.

0ln.A09/A0A/A0B and handler cases6/7 concern message acknowledgement/unacked
message bookkeeping. They are not waiting-IQ activation; those earlier search
branches are retained to prevent repeating the false lead. A00 handles
registered incoming message types, not the IQ result correlation above.
1lY's concrete C0g has zero direct static callers because this path uses the
1lW interface; the interface trace finds 1lZ.A1E. Zero concrete callers does
not mean this callback is unused.

## Port boundary and next exact questions

The existing Rust VerifiedDeviceIdentity signature verifier and contact
companion_admission are not this authenticated pairing state machine.
The former verifies ADV identity material against independently supplied keys;
the latter uses a contact DeviceList. login.rs explicitly omits companion
pairing data. Neither proves current primary account ownership, a QR attempt,
ref/secret agreement, active generation, or completed own-device bootstrap.
A nonempty contact DeviceList cannot enable automatic read-self.

The [preceding note](own-companion-ingress-current.md) retains the important
Me distinction: 0D0/09O can load/install Me, 13N can create it from saved
registration preferences, and Mrw temporarily installs it during export.
BUx's loaded-Me predicate is consequently insufficient as authentication.
This chunk does not close primary verified-preference provenance or
logout/account-switch isolation. No broader completion claim is made.

Required next work, before an integrated own-device/read-self port:

1. Follow the producer/authorization of Esd and EGv's decrypted DiD branch;
   bind ref, remote public identity, prior secret and user-approved attempt.
   Decode the checked Fdy constructor/crypto path before choosing key encoding.
2. Follow 1rA.A03's selected subclass dispatch back through the connection reader
   to authenticated session/account and sender admission. Preserve A0C activation,
   exact ID, result/error and timeout outcomes without automatic retry.
3. Close the successful primary registration callback writing the preferences
   consumed by 13N.A06, then 09O.CrQ. Existing 107.BLn evidence is companion
   onboarding and cannot stand in for this primary origin.
4. Close logout, account switch, timeout and restart invalidation of FUY,
   EXq/Eo9, Me, PN/LID aliases, own devices and Signal pins as one account
   generation. Existing 0WB.A0E/A0F are namespace migration, not logout.
5. Define durable authenticated own inventory only after those owners are known.
   Acceptance must cover stale replies, wrong account/source, reused IDs,
   cancelled/replaced attempts, malformed identity/ref/secret, held identity-pin
   conflicts and restart between metadata/identity/bootstrap milestones.
   Keep unknown outcomes unresolved and block receipt eligibility until proven.

All findings are static. There is no live pairing validation, account restriction
causation claim, automatic read-self implementation, or new inspection capability.
