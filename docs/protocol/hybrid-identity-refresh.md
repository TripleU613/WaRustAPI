# Hybrid primary identity refresh

2026-10-13. Identity-refresh preview and application now stage a version-4
hybrid session from a verified primary `pqkey` directory reply instead of
deferring every PQ reply as `pq_session_unported`. **Implemented/offline.**
No live PQ directory reply has been applied by this client.

## Evidence reused (no new APK trace)

This session had no APK on disk, so this change rests only on previously
retained static evidence for pinned 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:

- **Confirmed:** `CQc.A00` is the bundle-processing path reached from
  `0jc.A0L`. At `0068..00b4` it verifies the Kyber key signature over the
  unchanged `KEMPublicKey` bytes ([pq-signature.md](pq-signature.md)), and at
  `028c..0296` it selects the hybrid HKDF label ([hybrid-kdf.md](hybrid-kdf.md)).
- **Confirmed:** `CQc.A00` archives a held current state at `0124..014c`
  before deriving ([peer-identity.md](peer-identity.md),
  [session-refresh-preview.md](session-refresh-preview.md)).
- **Confirmed:** `0jc.A0L` `0096/00a2` resets a changed-key session via
  `A0E/A0s -> A0D` before derivation
  ([identity-refresh-application.md](identity-refresh-application.md)).
- **Confirmed:** version-4 Alice state, pending KEM fields and the `0x44`
  wire form ([hybrid-session.md](hybrid-session.md), [signal-wire.md](signal-wire.md)).

**Inferred from those pieces:** a refresh reply reaches the same `CQc.A00`
path as first contact, branching only on Kyber presence; none of the retained
traces gives the refresh path a separate PQ rule. Native admission (`CPB.A00`)
is not re-derived here; the existing same-key/attestation admission rules apply
unchanged.

## Behavior

`signal::hybrid::bundle_for` must succeed: verified classic signed key,
exactly selected `pqkey` with a 3-byte ID, a valid signature over the original
value bytes and a valid ML-KEM-1024 encoding. Then
`HybridBundle::prepare_session_refresh` encapsulates to the fetched key and
stages a fresh version-4 Alice record with the KEM ID/ciphertext pending. It
encrypts nothing, allocates no ID and touches no store or socket.

| Reply | Preview | Application |
|---|---|---|
| Primary, changed identity, valid PQ | `prepared`, 0 archives | Applied: fresh v4 record, no old state; old pin/session go to the existing transition archive |
| Primary, same identity, valid PQ, `force_override_e2ee_state` | `prepared`, 1 archive | Applied: held current state (classic or v4) archived inside the v4 record |
| Primary, same identity, no override/verified attestation | `prepared` (preview does not evaluate admission) | `same_identity_admission_unported` (unchanged rule) |
| Invalid KEM signature, encoding or shape | `pq_bundle_invalid` | `pq_bundle_invalid`; field/reason logged |
| Companion peer with `pqkey` | existing companion authority result | `pq_session_unported` |
| `pqkey` with `non_e2ee_attestation` | `pq_session_unported` | `pq_session_unported` |

A held record used for same-key archival must carry the bundle's identity as
its remote identity and the saved account's local identity and registration,
and is bounded to 1 MiB; otherwise derivation refuses. These are local
integrity checks, not claimed native checks.

The commit is the existing one: unchanged reply, original refresh guard,
pin/session OCC, account fingerprint, own PN/LID exclusion, once-only marker.
The next normal send continues the held version-4 record
([held-hybrid-send.md](held-hybrid-send.md)) and so emits a `0x44` `pkmsg`
carrying the pending KEM fields until the peer replies.

Inspection's `pq_signature.key_validation` is now `valid_ml_kem_1024`,
`invalid_ml_kem_1024` or `not_checked` (value absent or above the local 64 KiB
inspection bound); previously it was always `unported`. A valid encoding is
not trust or adoption.

## Validation

`primary_pq_refresh_stages_hybrid_sessions_the_peer_can_open` runs PN and LID
peers, changed and same identities. Preview is read-only with the expected
archive count; application commits once (a repeat is `AlreadyApplied` with no
table change); non-identity/session/checkpoint tables are unchanged. The stored
record is version 4 with zero (changed) or one (same) archived state. It then
emits a `0x44` `pkmsg` naming KEM ID 9, which the peer side authenticates with
`hybrid::receive_prekey` using its signed and KEM private keys.

`primary_pq_refresh_keeps_invalid_and_unported_inputs_unchanged` covers a
tampered KEM signature, an invalid ML-KEM encoding, attestation plus PQ, a PQ
companion, same identity without override and the pinned policy; every table
and the journal head stay unchanged. Existing PQ-signature inspection tests now
expect `pq_bundle_invalid` for their non-ML-KEM values.

## Remaining

Companion and attestation-bearing PQ refresh; native `CPB.A00` admission
state persistence; outbox same-key classic-to-hybrid migration (the outbox
still refuses it); live interoperability. Ban immunity is not established.
