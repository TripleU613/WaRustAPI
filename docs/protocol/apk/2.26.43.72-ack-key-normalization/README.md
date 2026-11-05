# ACK callback key normalization — APK2.26.43.72

**Confirmed static**, checked DEX text exports, not REA Evidence envelopes or
observed runtime. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
(the same pinned artifact reverified in the preceding ACK admission work).
Commands: `target/debug/xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions` and `target/debug/xtask apk-callers APK --class CLASS
--method METHOD`. Offsets below are method-relative bytes.

## Outbound key propagation

The retained [SendE2EMessageJob.A06](../2.26.43.72-sender-key-ack-confirmation/job.txt)
builds Efa at0b6e..0b9a, filling from(A02), class(A06), ID(A08),
participant(A01), recipient(A03) and edit(A07). The exact branch inputs for
every send type remain untraced. It passes the Eoh object into Ewq's constructor.
Ewq.<init> stores constructor argument5 in A01 (000e..0012); Ewq.A01 copies
A01 as constructor argument5 to Eng (009a..00d0); Eng.<init> stores it in A03
(000e..0012). Thus the key subsequently passed to0Sf.A02 and0Th.A02 at
1d88..1dc6 is the propagated callback key, not a key inferred from the final
serialized stanza ID alone. No mutation/replacement is shown in this selected
builder path. Unrelated arithmetic opcodes in Ewq.A01's large-stanza logging
branch are not decoded completely; no conclusion here depends on them.

## Primary-device canonicalization is bounded

0Sf.A02 inspects Eoh.A02(from). Only0Dn.A0S types
{17,24,25,19,23,27} enter the DeviceJid cast. If getDevice()==0, it clones
the key through Eoh.A00 and replaces from with DeviceJid.userJid. A nonzero
device leaves from unchanged. It separately converts participant through
0EB.A00: existing DeviceJid returns unchanged;0Dn.A0k-recognized UserJid
returns its primary device; unsupported/null inputs return null. A participant
with device0 is replaced by its userJid in a cloned key; nonzero companions
are preserved. The helper otherwise returns the original key.

0Dn.A0k recognizes numeric types{0,11,18,22,26,7}. These numeric sets do not
establish all server-name/type mappings. This helper performs no alias-store
lookup and is not evidence for arbitrary PN/LID equivalence, stripping every
companion suffix, or accepting a missing sender. Type predicates and field
assignment are retained so obfuscated field meanings can be rechecked.

The exact physical caller census has5 methods:0Sf.A0B/A0C/A0D/A0T and
SendE2EMessageJob.A06. This is not an interface/reflection/runtime census.

## Failed-request removal uses the same normalized key

The initial A0T candidate is a failure path, not successful inbound ACK handling.
0Sf.A0T normalizes Eoh with A02, removes that exact key from0Th.A04 under
lock, and invokes BsR(Exception) if a callback exists. Its physical callers
are8iv.invokeSuspend and SendE2eStatusJob.A0P; their complete failure policies
are untraced. Do not turn A0T into a generic success-confirmation authority.

## Generic inbound bridge now connected

The previously retained1lZ.A1D delegates generic ACKs to1lW.Big. The concrete
1lY.Big resolves0ln from0ZJ.A0C and calls A09 with the original node/key.
1lY.handleMessage case5 also extracts a2E3 pair and calls0ln.A09. Neither
selected bridge normalizes the received key with0Sf.A02.0ln.A09 then schedules
handler6 as retained in the [admission bundle](../2.26.43.72-distribution-ack-admission/README.md),
where Eoh equality checks class, from, ID, participant, recipient, type, edit
and additional fields before releasing a matching callback.

This connects the concrete fallback bridge, not every ordinary message ACK
dispatch or every out-of-band distribution caller. Outbound normalization is
not proof the native parser accepts every textual spelling as the same key.

## Port boundary

**Inferred consequence:** a raw outgoing-to / incoming-from comparison can be
too strict for a primary-device request, while ignoring addresses entirely is
weaker than the native Eoh callback identity. Neither arbitrary raw equality nor
broad address stripping is a justified replacement for the current distribution
waiter. Trace exact distribution key inputs and inbound parser type production
before implementing its normalized expected-address tuple.

**Unknown:** the out-of-band distribution producer and recipient alias choice,
all native numeric type mappings, ordinary-message ACK callback forwarding,
error459 failure routing and durable per-distribution confirmation. Existing
Rust class/ID admission and uncertainty reservations remain unchanged. No
production source change, live operation, recovery reset or replay authority.

## Checksums

- [normalization.txt](normalization.txt): `2f6edfd0a23db41930c28f6296426a37fba049c334d95e85e9550cb2347dcad1`
- [normalization-callers.txt](normalization-callers.txt): `762fd29cf5d6d1c07d55b5fbf03a8fde57df980abf2d0967a07de42577b40771`
- [device-predicate.txt](device-predicate.txt): `2df23cd04d4a21544a3036276201c0fd22aa991aceca37cc2cf72c29b8c0718a`
- [participant-conversion.txt](participant-conversion.txt): `c52a92cc1a5b7016342ea43399be0a0ece0ea1002c83c64e1cc9214905f86bef`
- [user-predicate.txt](user-predicate.txt): `15ac20b5755091ceb6fa5e8645b00557e528b2d7821e65e46c50130d8a622e6b`
- [builder-init.txt](builder-init.txt): `8d4d6b5cfec6292457db3b1861fe0d657031ff94c2bf76decc0bbc5722c8539f`
- [builder.txt](builder.txt): `1f714824a8f1e7d2287bf48a3ca0f8538c34c2c2e28886da0d0ecaaec2efd619`
- [message-init.txt](message-init.txt): `b7bc99681b6d7a58eabafd1e0bb27311ebca2f36fba80c897b2da9806999f82a`
- [reply-path.txt](reply-path.txt): `0762de0f2fd8f6cd33d1f0d9a92f748aebb17d9c086cf5d519dc0be5272a378c`
- [reply-callers.txt](reply-callers.txt): `58af0066a8600b51f74bd6f200b09663d0fffc9c7775b44a50a65426343ae258`
- [manager-callers.txt](manager-callers.txt): `ed106f37c8738722fcde5c163f8da8fb18ab89b446114b9168153ccc05f3b274`
- [inbound-bridge.txt](inbound-bridge.txt): `f8a13cfaa19178a442ef6a18e594f4c2e518924cd8e91ec7bd975d5e6651f671`
- [inbound-handler.txt](inbound-handler.txt): `ce5e0de9b6d49a0808374951e0dd1fa1323f1ea86fcc3dcedfef8664c002524b`
