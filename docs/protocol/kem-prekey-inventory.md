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
over the KEM public key, last-resort creation/rotation, the inner element
shape of `pq_list`/`pq_last_resort_key` and the server's acceptance are all
**unknown**. Because no KEM key is ever uploaded, a correctly behaving server
has no KEM key for this account to hand peers; in normal operation a fresh
hybrid bootstrap should not arrive. The inbox path exists so that, once an
evidence-backed producer and upload are implemented, receive is already
transactional.

## Inbox routing and classification

| Input | Result | State |
|---|---|---|
| Own-device (`category=peer`) `pkmsg`, hybrid or not | Unsupported | refused by the existing own-device route before KEM parsing |
| Direct companion or group companion participant hybrid `pkmsg` | Unsupported | unchanged; checked before companion admission |
| Primary PN/LID, envelope base key matches held/archived session, pin unchanged | Decrypted | session advanced; no key consumed |
| Primary PN/LID, fresh bootstrap, local row present | Decrypted | session, pin, plaintext, ordinary key and non-last-resort KEM row committed atomically |
| Fresh bootstrap with no local row, invalid row, malformed KEM ciphertext, version mismatch, or matching session with a changed pin | Unsupported | unchanged; reason logged (`hybrid prekey preserved`) |
| Pinned policy and changed identity | Undecryptable | identity conflict observation only; session, pin and KEM row unchanged |
| Nested MAC/padding/counter failure | Undecryptable | unchanged |

The pinned changed-identity case is reported as the classic backend's
untrusted-identity failure (`direct_message`), so the existing once-only
retry receipt applies to both paths alike. Unsupported outcomes stay
unacknowledged and do not queue a retry receipt, the same conservative
behavior the earlier blanket PQ refusal had. These are **local coverage
choices**: how the native outer caller handles a thrown `0kh.BTS` lookup
(retry receipt, error stanza or silent drop) is **unknown**.

A group stanza whose pairwise hybrid child is Unsupported leaves the whole
stanza unprocessed, as the earlier refusal did. After a pairwise hybrid child
decrypts, the existing classic code installs a contained sender-key
distribution or publishes an addressed payload in the same commit as the
session and key consumption; the test below exercises the addressed-payload
case only.

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
  unchanged;
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
