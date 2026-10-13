# Hybrid bundles in device-key batches and group fan-out

2026-10-13. Directory-key batches now answer a selected device whose `pqkey`
verifies, and the shared group/status fan-out derives a native version-4
session for it. Previously any selected `pqkey` classified the whole batch as
`invalid_bundle`, blocking the group or status send for every member.
**Implemented/offline.** No live hybrid fan-out has been sent.

## Evidence reused (no new APK trace)

This session had no APK on disk. Pinned 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:

- **Confirmed:** batch callback `Bot.A03` reads a selected user's PQ
  id/value/signature into `CIv`; `14J.A04` hands admitted candidates to
  `0jc.A0L`, which constructs `KEMPublicKey` for `CQc.A00`
  ([pq-bundle-boundary.md](pq-bundle-boundary.md),
  [bulk-prekey-refresh.md](bulk-prekey-refresh.md)).
- **Confirmed:** `CQc.A00` verifies the signed device key and, on the Kyber
  branch, the KEM signature (`0068..00b4`), then derives with the hybrid label.
  The retained traces describe no companion-specific Kyber rule
  ([peer-identity.md](peer-identity.md), [hybrid-kdf.md](hybrid-kdf.md)); that
  absence was not re-checked against the APK here and is not proof none exists.
- **Confirmed static / unknown runtime:** the native batch builder can add
  `pqsupport="true"` from a supplied Boolean (`0y0.A00` `1806..1824`); its
  producer and effective value are unknown ([device-key-batches.md](device-key-batches.md)).
  This client still sends the bare classic query and never claims PQ support,
  so whether the server returns `pqkey` to such a query is **unknown**. This
  change makes a returned key usable; it does not request one.

## Behavior

`device_key_batches::classify` keeps every structural and classic signed-key
check, then runs `hybrid::bundle_for` for a row carrying `pqkey`. A KEM
signature that does not verify is `invalid_signature`; a missing/duplicate
field, wrong widths or an invalid ML-KEM-1024 encoding is `invalid_bundle`.
`hybrid::bundle_for` now reports a failed KEM signature under its own error
field (`hybrid_bundle_signature`) so the batch does not match reason text.

`group_fanout::Snapshot::prepare*` selects a `HybridBundle` for a device whose
row carries `pqkey`, else the existing classic bundle. Pin, replacement policy,
own-identity, companion-authority and rotation decisions use the classic part
unchanged. A fetched hybrid bundle exists only when no held ratchet is used
(fresh mode, no session, or an admitted replacement), and then
`HybridBundle::prepare_initial` encrypts the distribution: one APK padding,
`0x44` `pkmsg`, pending KEM ID/ciphertext. A held version-4 record continues
through the existing shared send ([held-hybrid-send.md](held-hybrid-send.md)).
The consumable classic parser still refuses selected PQ; nothing drops a KEM
key to derive a classic session.

Companion devices go through the same derivation: their admission in fan-out
is the existing device-list/routing authority, which this change does not
alter. The direct outbox still refuses PQ for companions
([hybrid-outbox.md](hybrid-outbox.md)); that is a separate path.

## Validation

`verified_pq_rows_answer_and_invalid_pq_rows_block_without_key_mutation`:
a valid signed ML-KEM-1024 row answers and assembles (the classic parser still
refuses it); a tampered KEM signature is `invalid_signature`; an invalid
encoding is `invalid_bundle`. Crypto tables and the journal head are unchanged.
The existing placeholder-`pqkey` case remains `invalid_bundle`.

`hybrid_primary_and_companion_distributions_open_natively_and_continue_once`:
a primary and a companion row carry signed KEM keys. A tampered signature
refuses the memory-only preparation. The valid reply yields `0x44` `pkmsg`
distributions that each device opens with `hybrid::receive_prekey` using its
signed and KEM private keys, consuming its KEM key, installing the sender key
and decrypting the `skmsg`. After the atomic reservation and a reopen, cached
version-4 records need no directory read and produce pending `0x44` `pkmsg`s
that open through the matching held session. Fresh mode with a signed primary
replacement refuses under the pinned policy and, under the explicit
APK-authenticated policy, derives a fresh hybrid session the replacement
identity opens.

## Remaining

`pqsupport` request semantics; the lab CLI retry answer still uses the classic
parser for a PQ device; an end-to-end normal group outbox test with PQ rows;
companion PQ in the direct outbox; live interoperability. Ban immunity is not
established.
