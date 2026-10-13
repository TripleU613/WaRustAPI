# Answering a peer's retry receipt in the normal runtime

2026-10-13. **Design note, nothing implemented.** Written so the next session
starts from facts. The normal runtime (`listen`, `outbox-send`) never answers a
`receipt type="retry"`; only the comparison group-send window does
([groups](groups.md#what-the-receipt-asks-for-and-what-we-do-about-it)).

## What is established

- **Observed 2026-09-24:** a handset that could not decrypt our group fan-out
  sent `receipt type=retry`, `<retry count=1 v=1>` and a four-byte
  `<registration>`. Two other devices decrypted the same send.
- **Comparison implementation (offline):** `answer_retries` in
  `crates/whatsapp-cli/src/transport_group_send.rs` answers once per device per
  send inside a 45 s window: fresh key fetch for that device, fresh pairwise
  session (hybrid when the row carries a verified `pqkey`), the *same* sender
  key distribution and the *same* `skmsg` bytes under the original message ID
  and stanza attributes, session committed before the write. No live retry
  answer has been made, so its acceptance by a handset is **unverified**.
- **Normal outbox storage:** each attempt keeps the encoded outgoing stanza
  (`AttemptRecord.payload`, encrypted at rest) and the model already has
  `AttemptRecord.re_encryption` for a protocol-directed re-encryption. Nothing
  sets it yet.
- **Native evidence retained:** only `SendE2EMessageJob`'s
  `forceSenderKeyDistribution` / `targetDeviceRawJids` names (groups.md). The
  related [sync-and-resend job](sync-resend.md) resends to *newly added*
  devices; it is not the retry-receipt answer.

## What a normal-runtime answer needs that is not stored

- **Direct messages:** the original plaintext `Message`. The stored stanza holds
  only ciphertext under the old session, which the peer could not decrypt.
  Regenerating from the intent is not faithful for every kind (a reaction's
  sender timestamp, a poll's secret, a quoted reply's parent preview can differ),
  so the plaintext would have to be retained per attempt, encrypted at rest.
- **Group messages:** the `skmsg` is recoverable from the stored stanza, but the
  sender key distribution *at the iteration the skmsg used* is not: the chain has
  advanced and the stored stanza holds only per-device encrypted copies. It
  would have to be retained per send.

## Not established (needs the APK)

The native answer policy: how many retries are answered and for which counts;
whether `count >= 2` receipts carry `keys` the sender must use instead of a key
fetch; when the registration ID mismatch forces a new session versus reusing
the existing one; how long after a send an answer is still allowed; and the
answer's own stanza attributes. Trace the retry branch of 2.26.37.71
`SendE2EMessageJob` and the incoming `receipt type=retry` handler before
implementing.

## Bounds any implementation must keep

Answer only server-acknowledged sends of ours, at most once per (message,
device), never a send whose outcome is uncertain or refused; reserve the answer
(and any key fetch) durably before the write; never change the message ID;
keep pinned-identity rules (a changed identity is not adopted to answer a
retry); and record the answer as a new attempt with `re_encryption: true`.
