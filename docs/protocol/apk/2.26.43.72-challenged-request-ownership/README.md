# Challenged request ownership — APK2.26.43.72

**Confirmed static**, checked DEX TXT exports, not observed runtime or REA
Evidence envelopes. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
reverified2026-11-04. Commands: `xtask apk-field-refs APK --class CLASS
--field FIELD --type TYPE`, `xtask apk-callers APK --class CLASS --method METHOD`,
and `xtask apk-smali APK --class CLASS --method METHOD --signatures --exceptions`.
All inventories are exact physical references, not reflective/interface/runtime censuses.

## Map transfer is not ordinary ACK settlement

The preceding [auth-challenge trace](../2.26.43.72-ack-auth-challenge/README.md)
connects numeric459 to0m2 handler7: remove an Eo7 by stanza ID from0TV.A05,
then place it in A04 when one exists. A04 has7 exact field references; A05 has5.
Eo7 stores Android Message(A01), callback(A02) and integer count(A00). Its
A00 returns EV7 only for message discriminator206, explicitly a CallStanza.
These records are not themselves proof of durable frozen ciphertext or
ordinary group-message registration in this map.0Th's message ACK callback
registry remains distinct.

0TV.A02 first calls A01 to remove from A05, falling back to removal from A04
when absent. This provides a lookup/removal route for either map; it does not
authorize treating a challenged request as a successful or failed ordinary ACK.

## Selection keeps eligible entries; it is not a destructive drain

0TV.A03 creates a LinkedHashMap and calls A00 under A04's monitor. A00
iterates the source, copying eligible ID -> Android Message entries into the
new map at00d0..00ee. That branch does not remove the source entry. Entries
whose count fails the selected bound are removed through Iterator.remove at
00fc..0102; special call records can schedule cleanup under config0x3b15
and emit call_stanza_drop diagnostics.

The ordinary fallback condition selects count<3. EV7 offer/terminate records
can instead use a positive configured limit from0x3c4a; a nonpositive limit
falls back to the ordinary condition. Effective config and every message-family
producer remain unknown. These are client map-selection counters, not a
server rate limit, cooldown bypass or safe retry budget for Rust. Export file
names containing drain are search labels; eligible records remain in source.

## Two selected entry paths reach re-dispatch

The nonzero-selector FUT.CK4 token branch parses a token-bearing result, saves
its bytes Base64(flag3) under pref_client_auth_token, calls19i.A00, then invokes
0Sf.A08 with0TV.A03's selected map (009e..00c6). Token request construction,
reply converters and conditional eligibility remain prerequisites; no token
value or response is manufactured by this trace. Calling the key-rotation
manager is not proof its asynchronous key replacement has completed before
held-request dispatch. This closes the earlier post-token-helper unknown.

0YB.A01 is labelled MessageHandler/handleConnected. Its selected05xx block
first selects from A05 with0TV.A00 and dispatches that map, then selects A04
with A03 and dispatches it (055e..05ba). Complete entry predicates, callback
ownership and connection-generation lifecycle are not reconstructed here.
Other preference writes in the large connected method are outside this scoped
trace and are not proof that a server restriction was removed.

0Sf.A08 iterates the selected map and calls A0L(Message,id). A0L supplies a
null new callback, timeout0 and tracking=true to A04. In A04's ready branch,
an existing held entry is replaced with incremented count and the old callback
when no new callback is supplied (0060..0098). Otherwise the analogous A05
entry starts at1 or increments its old count (009a..00de). Replacing an A04
entry does not move it into A05 in this selected body. Positive durations
register through0ln.A06 before dispatch; the A0L wrapper's zero duration does
not request that positive-duration registration. When not ready, A04 queues an
EVD in0TX.A04; its later connected-list dispatch is separately visible in
0YB.A01. None of these facts imply exactly-once wire transmission.

## Rust port boundary and remaining work

**Unknown:** the complete families registered in0TV versus0Th, challenge-push
authenticity/eligibility, token writer/converters, map/callback generation and
shutdown ownership, cleanup semantics, durable restart reconciliation and
whether held Android Messages preserve identical wire ciphertext on resend.
A physical field census is not proof all those boundaries are covered.

Rust continues to keep459 out of ordinary terminal send settlement and retains
its uncertain reservations without automatic replay. Do not transplant the
count bound, treat any new connection or token-like value as replay authority,
clear a refusal, or repeat an uncertain ciphertext from these partial methods.
This evidence-only chunk changes no production behavior and performs no live
account or network operation.

## Checksums

- [connection-recovery.txt](connection-recovery.txt): `a19263185d577b5f199c76239976a4b82ec971c4cae74c047869d5b3db031f77`
- [held-drain-callers.txt](held-drain-callers.txt): `ca3f8c06ae406c651007c78b8e366ccb482206ea3f670729402e2ff07b2498c3`
- [held-drain.txt](held-drain.txt): `f420848b97fa52ae31d5fd61152971437a075ec3adb5c18d4c4a8db4e77ccdc5`
- [held-lookup.txt](held-lookup.txt): `bbb9ed8932b3dd67bf8b1d76c6a87487a47cedca7678a77df80af1825b575c67`
- [held-map-refs.txt](held-map-refs.txt): `52a5486e99db5496db735d12f00107ad63102f6152d5fd1cb4be112180b1f0a8`
- [map-dispatch.txt](map-dispatch.txt): `4548fc8e0721c68c2a007eb4107c343d9e73fa7e7e2835540893451c401da95f`
- [map-drain-policy.txt](map-drain-policy.txt): `8150417dd9baafd217467ad7a2246d144111404005763bf22abd01bedcdc5ef9`
- [redispatch-wrapper.txt](redispatch-wrapper.txt): `b0a219274d95e693a7ce386c9faac93c0c7c6dcb9d840030db47397860527aa9`
- [request-record.txt](request-record.txt): `830410f3eac6839d3b24c6ef566a226728d0db78ae9b0a3b80e7a42c0c4399f4`
- [request-registration.txt](request-registration.txt): `c209a338afe7d9010b7cfd1d13adf719075badb02e48b3e19f8bde3d3a812edc`
- [token-success.txt](token-success.txt): `6eaf99462a33932944533a617e1383433d1471f65dfe83074b02824da3ccd632`
- [unacked-map-refs.txt](unacked-map-refs.txt): `13cdf1176f2eb79bae89b5486a34af81b315da2c4c21bb5c648974025e6d9ffd`
