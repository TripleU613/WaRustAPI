# Legacy prekey publication binding audit

## Confirmed local defect, still open (2026-11-05)

Audited revision: `9b26f558fbdc8c2d670e64f921e840bce1a3197f`.
This is a synthetic local-store defect, not APK parity or observed server behavior.

`transport/prekeys.rs::begin_upload_selected` validates legacy private/public
pairs through `legacy_record_in`, freezes their public bytes in the request,
and reserves `Meta { status: OutcomeUnknown, batch: iq_id }`.
`PendingUpload::settle` later checks the batch ID list and metadata ownership,
but does not bind a legacy public key to that original publication. Replacing a
reserved row with a different valid pair under the same ID leaves those checks
satisfied and allows a result to mark the replacement ID Uploaded. The KEM
fingerprint guard added in 9b26f558 does not cover this legacy list.

## Reproduction retained

Temporarily insert the following test into `transport/prekeys.rs::tests`, whose
existing helpers create an encrypted synthetic store and registered test account.
Run `cargo test --locked -p whatsapp-api --all-features audit_legacy_upload_replacement_probe`.
At the audited revision it ran one test and failed the final assertion with
`replacement legacy public material was acknowledged` (exit 101); it compiled
successfully, so this was a behavior failure. The temporary test was removed
from production source after the probe; the complete reproducer remains below.
No live connection, credentials or account operation was used.

```rust
#[test]
    fn audit_legacy_upload_replacement_probe() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("legacy-probe.db"));
        let (account, signed) = account_and_signed(&store);
        generate(&store).unwrap();
        let (pending, _) = begin_upload(&store, &account, &signed).unwrap().unwrap();
        let id = pending.ids[0];
        let replacement = PreKeyRecord::new(id.into(), &KeyPair::generate(&mut rand::rng())).serialize().unwrap();
        store.write(|tx| tx.put_json(Table::Prekeys, record_key(id).as_bytes(), 1, &replacement)).unwrap();
        let reply = Node::new("iq").attr("id", pending.iq_id.as_str()).attr("type", "result");
        assert!(pending.settle(&store, &reply).is_err(), "replacement legacy public material was acknowledged");
    }
```

## Receive and storage constraints on the fix

Both `inbox.rs` direct receive and group member pairwise bootstrap load
`comparison-prekey:{id}` before decryption, compare the full original serialized
record inside the final write transaction, and delete it only on successful
commit. The two deletion sites are the `used_prekey` branches before the
`used_kem` branches. Unlike `kem_prekeys::consume_in`, these branches currently
write no dedicated consumed-public fingerprint marker. Their existing exact-row
comparison protects concurrent receive commit, but does not attest the missing
public material to later upload settlement.

Consequently, simply requiring every row to exist at upload ACK would break
legitimate consume-before-ACK. Simply allowing any missing row would leave key
loss indistinguishable from consumption. A complete fix needs immutable
reservation fingerprints plus atomic consumption evidence in both receive
branches, refusal of conflicting restored rows, and validation of the original
public identity on accepted settlement. Historical reservations/consumptions
must remain unknown rather than gain inferred hashes or replay authority.

Required regression coverage: same-ID well-formed replacement, missing row
without consumption evidence, mismatched consumed fingerprint, conflicting
restored row, legitimate direct and group consumption before ACK, transaction
rollback, reopen, and unchanged uncertainty/refusal holds. None of those new
legacy guarantees is implemented by this audit. Whole-store rollback remains
outside the local consistency model. This finding does not establish a ban
cause or server acceptance requirement.
