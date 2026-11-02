# Answering a peer's retry receipt in the normal runtime

2026-10-13 design note; **2026-11-01: the no-fetch branch is implemented
offline** (`transport::retry_answer`, called by `listen` for every accepted
receipt). Not live-verified: a retry cannot be provoked without deliberately
breaking a peer's session, which we do not do; it will be confirmed the first
time a peer asks during a normal `listen`.

## APK answer policy (confirmed static, 2.26.37.71, 2026-11-01)

Anchored by the `RetryReceiptHandler/…` strings:

- `150.A04` (`RetryReceiptHandler`): `0270..027e` loads `4` and skips the retry
  ("axolotl skipping retry for") when `count > 4`. `0416..043c` compares the
  original's timestamp (`178.A0F`) with now − `0x134fd9000` ms (**60 days**) and
  refuses older originals ("cannot send retry to older message"). Also refused:
  a TEE bot device (`01a6`), a transient message (`03a0`), a deleted original
  (`0754`).
- `DNP.run` (the resend runnable): `0118..0136` refuses a device that "isn't an
  originally intended recipient"; `01c6..01f4` refuses when the original is
  "already marked delivered" for that device; otherwise it logs "axolotl
  resending" and re-injects a `SendE2EMessageJob` built over the original
  message object (`02aa..0390`, `Cn6.<init>`): the same message, re-encrypted.
- `151.A00(address, key, …, count, registration)`: when a session exists and
  its `remoteRegistrationId_` differs from the receipt's `<registration>`, the
  session is deleted ("deleting session due to registration id change", `0036`)
  and the send waits for a fresh bundle. With a matching registration:
  `count == 2` records the base key used ("will record the base key", `0102`,
  `0jc.A0u`) and resends; `count > 2` with the same base key (`0jc.A17`), or a
  type-3 session behind ab-prop `AzZ.A09`, "will wait to send … until a new
  prekey has been fetched" (`00ca`). Otherwise (count 1) nothing changes and
  the resend uses the existing session.
- `151.A01`: a receipt carrying `<keys>` can build the session from the receipt
  itself ("processing keys from the receipt"), refused when the registration
  does not match or no session exists.

## What `retry_answer` does

Answers only the branch that needs no key fetch: a server-accepted, not
failed, direct send of ours (outbox, any namespace), not delivered to that
device, at most 60 days old, `count` 1 or 2, the asking device equal to the
addressed peer (PN or its saved LID, same device), and a `<registration>` equal
to the saved session's remote registration id. The resend is the original
stanza with only the `enc` replaced (re-encrypted from the retained plaintext
under the saved session), the same message id. It is reserved before the write
(`retry-answer:<id>:<device>:<count>` checkpoint, a new
`AttemptRecord{re_encryption: true}`, the advanced session), so a crash never
causes a second answer. Every other case writes
`retry-unanswered:<id>:<device>:<count>` with one reason and sends nothing:
`group_retry_unsupported`, `count_out_of_range`, `new_session_needed_unsupported`
(count 3–4), `not_an_original_recipient`, `original_not_server_accepted`,
`already_delivered`, `original_older_than_60_days`, `plaintext_not_retained`
(sends reserved before 2026-11-01), `no_session`,
`registration_changed_needs_new_session`. `listen` logs the reason;
`store inspect` counts both kinds.

The plaintext `Message` of each direct send is now retained at reservation as
`retry-source:<wire id>` (store-encrypted at rest like the stanza).

Not reproduced: the count-2 base-key record and the count > 2 comparison, the
fresh-bundle path after a registration change, `<keys>` in the receipt, group
and broadcast retries in the normal runtime (the comparison group-send window
still answers its own), and any `enc count` attribute: `Civ`'s stanza writer
has no string constants and remains untraced, so the resend carries none, like
the comparison `answer_one`.

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
- **Native evidence retained:** `SendE2EMessageJob`'s
  `forceSenderKeyDistribution` / `targetDeviceRawJids` names (groups.md). The
  related [sync-and-resend job](sync-resend.md) resends to *newly added*
  devices; it is not the retry-receipt answer.
- **Rechecked 2026-10-13 (confirmed static, retained bundles; corrected the
  same day by an independent verification pass):** the job also carries
  `retryCount:I`, `messageSendRetrySource:Integer`, `isRetryReceiptLid:Z` and
  `includeSenderKeysInMessage:Z`. **The job bounds the count:** in
  [send-job-run](apk/2.26.37.71-send-job-state/send-job-run.txt)
  `0c90..0c9e`, `retryCount > 4` ("SendE2EMessageJob/aborting e2e message send
  job due to high retry count") logs a warning (`0998`) and exits, so at most
  four retries are answered by one job. Other branches: `if-nez retryCount` at
  `08c2..08c6`, and in
  [group-send-job-hash-check](apk/2.26.37.71-groups/group-send-job-hash-check.txt)
  `172a..172e`, where a retry skips the `Can.A03` PQ session-upgrade block.
  `includeSenderKeysInMessage` sets `job.A01` (`0b70..0b88`). The values are
  also forwarded: `isRetryReceiptLid` and `includeSenderKeysInMessage` into
  `CaB` (`11ee..1256`), `retryCount` into `Civ` (from `0bd6`) and the
  job-state logging (`067c..0684`), with operands elided. **Corrected:** an
  earlier version said no branch limits the count. Who creates the job from an
  incoming retry receipt, and with which count and flags, is still not
  retained.

## What a normal-runtime answer needed that was not stored (2026-10-13; direct plaintext now retained)

- **Direct messages:** the original plaintext `Message`. The stored stanza holds
  only ciphertext under the old session, which the peer could not decrypt.
  Regenerating from the intent is not faithful for every kind (a reaction's
  sender timestamp, a poll's secret, a quoted reply's parent preview can differ),
  so the plaintext would have to be retained per attempt, encrypted at rest.
- **Group messages:** the `skmsg` is recoverable from the stored stanza, but the
  sender key distribution *at the iteration the skmsg used* is not: the chain has
  advanced and the stored stanza holds only per-device encrypted copies. It
  would have to be retained per send.

## Not established (2026-10-13 list; partly answered above)

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
