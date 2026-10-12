# Staged hybrid prekey receive

2026-10-12. Experimental `signal::hybrid::receive_prekey` now authenticates a
version-4 prekey envelope using the selected local EC/KEM keys and returns a
session/plaintext candidate plus consumption IDs. It performs no durable write,
key deletion, request reservation or network operation.

## Native evidence

Pinned APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [three-topic recipe](apk/recipes/2.26.37.71-hybrid-incoming.json) reproduces
[versioned DEX evidence](apk/2.26.37.71-hybrid-incoming/targets.json), including
full signatures, register flow and catch tables, mapped to the new Rust file.

Cnp.A04 first compares version and Alice base key against the current and
archived sessions. A matching record bypasses local prekey lookup. Fresh bootstrap
loads the signed EC key, optional ordinary EC key and KEM record, decapsulates,
and derives the ordered hybrid state. Successful nested decrypt precedes session
storage and key consumption. Ordinary ID 16777215 is reusable and is not deleted.
The existing [session evidence](hybrid-session.md) covers the derivation and
native pending-clear behavior.

[REA KEM lookup](rea/2.26.37.71-hybrid-incoming/kem-lookup.json)
`ev_c50d2cc5cc85563480dcea3d845a84487fb494cec3c21b234163b1405f4b9935` observes 0kh.BTS selecting
record from kyber_prekeys by prekey_id and throwing when absent. This supplies
CG3's stored public/private pair to Cnp.A04's KEM decapsulation.
[REA KEM deletion](rea/2.26.37.71-hybrid-incoming/kem-delete.json) `ev_4e8169a86236cb8d8335eb1f3e93aca0dae5990168ba2de6b99f9c6fc073dc11` observes
0kh.BWa deleting only rows matching prekey_id and last_resort_key=0.
Thus successful last-resort use must retain the key. These are static observations;
no Android database or account was executed/imported.

## API and preservation rules

`LocalKemPrekey { id, secret, last_resort }` borrows trusted local inventory
material. It must not be constructed from sender-controlled last-resort metadata.
The expanded secret is borrowed and Debug is redacted. `ReceivedHybridPrekey`
contains the existing ReceivedPrekey plus optional consumed_kem; Debug is redacted.
The caller must atomically commit plaintext, identity/session, any identity archive
and both key-consumption decisions after comparing the original saved inputs.

A local 1 MiB held-record bound and account/pin integrity guard apply before
promotion or refresh. The parser requires paired KEM metadata and version 4 at outer and nested layers.
Matching held/archived sessions require an unchanged saved pin and matching local
identity/registration; no signed/ordinary/KEM key need be supplied. Authenticated
replay still fails the ratchet counter check. Cross-identity archive promotion is
explicitly unsupported to prevent a delayed envelope from rolling back a pin.

Fresh bootstrap validates account identity pair, exact signed key ID and its
signature/pair, optional ordinary key ID/pair, selected local KEM ID and expanded
secret/ciphertext encoding. Pinned policy refuses changed identities; explicit
APK-authenticated policy stages a fresh candidate. Changed identities do not carry
old states into the new ratchet; same-identity refresh retains the canonical archive
with existing counter-reservation handling. The previous bytes remain untouched.
These local integrity guards are not claims of additional native admission rules.

Only successful nested MAC and APK padding validation return consumption. Ordinary
ID 16777215 and local last-resort KEM keys remain reusable. Matching sessions report
neither key consumed. Implicit KEM rejection alone never authenticates a message.

## Validation and remaining integration

Actual encrypted fixtures exercise bootstrap with/without an ordinary EC key,
last-resort KEM retention, reusable ordinary ID, pinned refusal and explicit changed
identity staging. Subsequent pending prekey messages decrypt from the held record
without any local keys. An archived matching session is promoted and decrypts;
duplicated counters, modified KEM ciphertext, missing/wrong-ID keys, version mismatch
and another local registration refuse. Input session bytes remain unchanged.

Normal direct/group inbox guards are still active. Saved account-bound KEM inventory,
publication/upload, transactional lookup/consumption, own-device and companion
admission, crash/reopen inbox tests and live Android/server interoperability remain
unfinished. No existing saved identity, key, request ID or server wait changed.

Validation passed 978 workspace/all-feature tests, the expanded crypto-only
hybrid exchange, strict all-target Clippy, warnings-denied Rustdoc, 26 default
CLI/API-symbol tests, experimental CLI build and formatting. The final ownership
guards were rechecked by the focused encrypted exchange and strict Clippy.
