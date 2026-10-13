# Local KEM prekey inventory and hybrid inbox receive

2026-10-13. The normal direct and group inboxes now route a version-4 `pkmsg`
carrying paired KEM fields through the staged
[hybrid prekey receive](hybrid-incoming.md) instead of refusing it outright.
A matching held session needs no local key; a fresh bootstrap reads an
account-local ML-KEM-1024 row and deletes it in the same commit unless it is a
last-resort key. Replies on held version-4 sessions (`0x44` `msg`) decrypt
through the existing inbox `msg` branch. **Implemented/offline.** No live
hybrid message has been received by this client.

## Evidence reused (no new APK trace)

This session had **no APK on disk** (the documented
`~/Downloads/WhatsApp.apk` path does not exist in the cloud
container), so nothing below is a new static trace. Every native claim is the
previously retained evidence for pinned 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:

- **Confirmed (REA, [hybrid-incoming.md](hybrid-incoming.md)):** `0kh.BTS`
  selects `record` from `kyber_prekeys` by `prekey_id` and throws when absent;
  `0kh.BWa` deletes only rows matching `prekey_id` **and** `last_resort_key = 0`.
  Successful last-resort use therefore keeps the key.
- **Confirmed (`Cnp.A04`, [peer-identity.md](peer-identity.md)):** a session
  matching version and Alice base key is promoted before any local key lookup;
  otherwise the signed, optional ordinary and KEM keys are loaded and a new
  session is derived. Prekey deletion follows nested decryption.
- **Confirmed ([prekeys.rs](../../crates/whatsapp-api/src/transport/prekeys.rs)
  module trace):** the upload IQ `LX/0y0.A00` case 85 names optional
  `pq_list`/`pq_last_resort_key` children; `LX/13c.A03` logs "skipping kyber
  prekeys upload" without Kyber material; a `pq_count`-only encrypt
  notification takes no action (`LX/14z.A02` `01a0`).

## Inventory rows

`transport::kem_prekeys` keeps rows in the encrypted `prekeys` table under
`comparison-kem-prekey:<id>`, a prefix disjoint from ordinary
`comparison-prekey:` rows so the existing upload/count prefix scans never see
them. A row holds the native uint32 ID, the 1568-byte encapsulation key, the
3168-byte expanded decapsulation key (the form the APK stores; see
[rust-kem.md](rust-kem.md)) and `last_resort`. The secret is zeroized on drop
and Debug is redacted.

Loading validates the row ID against its key, both lengths, and that the
public key equals the copy embedded in the expanded secret (`dk_pke ‖ ek ‖
H(ek) ‖ z`, offset 1536). These are **local integrity checks**, not claims of
an additional native check. An invalid row preserves the message.

`last_resort` is read from the trusted local row, never from sender metadata.
Consumption re-reads the row inside the inbox write transaction and refuses
(rolling back the whole message commit) when it changed, when it is a
last-resort row, or when its ID differs; this mirrors the existing ordinary
one-time prekey guard.

**There is no producer.** Rows exist only in tests. Generation count, ID
allocation (no `next_kyber_prekey_id` equivalent is traced), any signature
over the KEM public key, last-resort creation/rotation and the server's
acceptance are all **unknown**. The inner element shape is now read from
retained evidence (next section); what goes into each element is not.

## Upload element shape (2026-10-13, retained evidence)

**Confirmed static**, from the retained upload writer `LX/0y0.A00` case 85
([writer-dispatch](apk/2.26.37.71-identity-ack-writer/writer-dispatch.txt)).
The IQ (`id` from `v2` at `1622..1630`, `xmlns=encrypt` at `1634..1648`,
`type=set` at `164c..165e`, and `to`) collects its children in
this order: an optional `op` (coexistence `mode`/`reason`/`coex_token`,
`13ea..1474`), `identity` (`1474..148e`), `registration` (`148e..14a6`),
`type` (`14ae..14c8`), `list` of ordinary keys (`14c8..14e0`), `skey`
(`14e2..153c`), then:

```text
pq_list                                  only if the KEM key array is non-empty (153c..1546)
  key            × n                     tag from v20 = "key" (1356, used at 159a)
    id         <CIv.A01 bytes>
    value      <CIv.A00 bytes>           v10 = "value" (135a)
    signature  <CIv.A02 bytes>           v12 = "signature" (150e)
pq_last_resort_key                       only if a last-resort key exists (15ca)
  id         <CIv.A01 bytes>
  value      <CIv.A00 bytes>
  signature  <CIv.A02 bytes>             (no inner key wrapper; 15ce..161c)
```

The `id` tag is **confirmed static** (upgraded the same day by an independent
verification pass): `const-string v2, "id"` at `00ec..00f0` directly precedes
the `sparse-switch` at `00f0` whose case 85 enters `12d0`; nothing else
branches into `12d0` or `00d0..00f6`, `v2` is not written inside
`12d0..1634`, and the same `v2` names the IQ's own `id` attribute. It also
matches the ordinary `<key><id/><value/>` shape acknowledged live on
2026-09-19. Each KEM entry is shaped
like `skey` (id, value, signature), unlike an ordinary one-time key (id,
value only).

**Element contents, derived from the receive side.** Peers parse and verify
our `pqkey` with the rules this client already implements for theirs
([pq-signature](pq-signature.md), [hybrid-bundle](hybrid-bundle.md)), so those
rules fix what an upload element must hold. **Confirmed static:**
`KEMPublicKey` wraps the `value` bytes unchanged (no type prefix) and `CQc.A00`
(`0068..00b4`) verifies the `signature` over exactly those bytes with the
peer's identity key, through the same adapter as the classic signed key; the
key must be a valid 1568-byte ML-KEM-1024 encapsulation key and the signature
64 bytes. **Inferred:** the server relays the uploaded `id`/`value`/`signature`
unchanged into peers' `pqkey` (it does so for ordinary keys, uploaded without
their type byte), and the `id` *value* is three bytes big-endian (the ordinary and
signed keys' `1Lz.A04` width, and this client's parser rule; the APK's `pqkey`
ID decoding is not traced).

`transport::prekeys::kem_upload_children` encodes local KEM rows that way:
`pq_list` of `key{id, value, signature}` and `pq_last_resort_key{id, value,
signature}`, the signature computed with the account's identity key. A test
(`kem_upload_entries_pass_the_receive_side_bundle_parser`) presents its
entries to `hybrid::bundle_for` as a peer would receive them, and they pass the
full receive-side check. It is **not** wired into `begin_upload`.

**Still unknown:** KEM key generation and counts, ID allocation (no
`next_kyber_prekey_id` equivalent), last-resort creation and rotation, when the
native client uploads (`13c.A03` is not retained), whether `Can.A04` must allow
it first, and the server's acceptance. So nothing post-quantum is uploaded. Because no KEM key is ever uploaded, a correctly behaving server
has no KEM key for this account to hand peers; in normal operation a fresh
hybrid bootstrap should not arrive. The inbox path exists so that, once an
evidence-backed producer and upload are implemented, receive is already
transactional.

## Inbox routing and classification

| Input | Result | State |
|---|---|---|
| Own-device (`category=peer`) `pkmsg`, hybrid or not | Unsupported | refused by the existing own-device route before KEM parsing |
| Direct companion hybrid `pkmsg` | Unsupported | unchanged; checked before companion admission |
| Group companion participant hybrid `pkmsg` | pairwise child skipped; a held sender key still decrypts the stanza's `skmsg` | no companion session, KEM row kept |
| Primary PN/LID, envelope base key matches held/archived session, pin unchanged | Decrypted | session advanced; no key consumed |
| Primary PN/LID, fresh bootstrap, local row present | Decrypted | session, pin, plaintext, ordinary key and non-last-resort KEM row committed atomically |
| Missing/mismatched/unusable signed, ordinary or KEM key (including a row that does not parse), or a wrong-length KEM ciphertext | Undecryptable (bounded retry, like classic `InvalidPreKeyId`) | unchanged |
| Version mismatch, matching session with a changed pin, held account/pin mismatch | Unsupported | unchanged; reason logged (`hybrid prekey preserved`) |
| Pinned policy and changed identity | Undecryptable | identity conflict observation only; session, pin and KEM row unchanged |
| Nested MAC/padding/counter failure | Undecryptable | unchanged |

The pinned changed-identity case is reported as the classic backend's
untrusted-identity failure (`direct_message`), so the existing once-only
retry receipt applies to both paths alike. **Corrected 2026-10-13 after an
independent review:** missing or unusable local keys were first classified
Unsupported (never acknowledged, no retry, redelivered at every login) while
the classic path makes the same conditions Undecryptable. They now use the
classic classification, which lets the sender re-fetch our (classic) bundle
and resend. A row that does not parse no longer surfaces as a store failure
that would end the session. Unsupported remains only for local coverage gaps.
How the native outer caller handles a thrown `0kh.BTS` lookup (retry receipt,
error stanza or silent drop) is still **unknown**; this mirrors the classic
backend, not a traced native rule.

A group stanza whose pairwise hybrid child is Unsupported leaves the whole
stanza unprocessed, as the earlier refusal did. After a pairwise hybrid child
decrypts, the existing classic code installs a contained sender-key
distribution or publishes an addressed payload in the same commit as the
session and key consumption; the test below exercises the addressed-payload
case only.

## Inspection

`lab-inbox --inspect` reports `prekey_kem` for the first preserved `pkmsg`:
`absent`, `invalid_input`, or `present` with `envelope_version` and
`local_kem_row` (`missing`, `held`, `last_resort`, `invalid`). It prints no KEM
ID or key bytes and changes nothing. The library classification is
`transport::prekeys::kem_row_state`. A matching held session needs no row, so
`missing` alone does not predict an Unsupported outcome; the inbox warning
names the actual reason when processing defers.

## Validation

`hybrid_prekeys_use_local_kem_inventory_and_continue_through_normal_inbox`
uses an encrypted store with the registered signed key and real ML-KEM-1024
encapsulations to a locally generated key:

- no row: Unsupported under both policies, every table and the journal head
  unchanged;
- with a row: Decrypted; ordinary key 11 and KEM row 9 deleted; held record is
  version 4; authenticated identity pinned; one event; replay is Duplicate;
- a second pending `pkmsg` with the same base key decrypts through the matching
  session after the KEM row is gone;
- our `0x44` reply (`prepare_payload` on the held record) decrypts on the
  peer, whose next `0x44` `msg` decrypts through the normal inbox `msg` branch;
- a last-resort row survives successful bootstrap; `prekeys::summary` reports
  `kem_local`/`kem_last_resort`;
- a changed identity is Undecryptable under the pinned policy with session, pin
  and KEM row unchanged, then Decrypted with archive/replacement under the
  explicit APK-authenticated policy, consuming that row;
- a group pairwise hybrid child from a primary participant decrypts and
  consumes its ordinary and KEM rows; replay is Duplicate;
- direct and group companion hybrid `pkmsg` are Unsupported with every table
  unchanged. **Limit (review, 2026-10-13):** these fixtures' companions have no
  saved device identity, so companion admission already refuses them and the
  tests would pass without the routing guards. The property that matters, that
  an *admitted* companion never reaches hybrid derivation, is now enforced
  inside `receive_hybrid_prekey` itself (`device != 0` returns the preserved
  `None`) and tested directly there: a companion address with a valid envelope
  and KEM row is refused, the same call for the primary opens, and neither
  commits. `admitted_companion_hybrid_prekeys_never_derive_or_consume_keys`
  (companion admission tests) now uses a companion whose classic prekey is
  admitted and decrypts; its hybrid 0x44 prekey with a valid local KEM row is
  not Decrypted, directly (PN and LID) or in a group, with every crypto table,
  the KEM row and the device list unchanged. With all three guards disabled the
  same message decrypts, so the test discriminates;
- after reopening, the three direct messages stay Duplicate with no table change.

`kem_prekeys` unit tests cover row validation, counts, refusal to consume a
changed/last-resort/mismatched row, and isolation from an ordinary prekey with
the same numeric ID. The existing `primary_lid_prekey_without_alias...` test
still shows a malformed version-3 envelope with KEM fields preserved unchanged.

## Remaining

KEM generation, signing, ID allocation, upload and server acceptance;
own-device and companion KEM admission; native outer-caller error handling and
retry behavior for hybrid failures; same-key classic-to-hybrid migration;
live Android/server interoperability. Ban immunity is not established.
