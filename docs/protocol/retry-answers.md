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

## Current correction (2026-11-04)

The normal direct retry driver supports count 1/2 with the saved session or a
reserved fresh-bundle fetch. Both paths now emit the incoming count unchanged
on the new `enc` child; the complete current producer/writer chain is retained
in [retry-enc-count](retry-enc-count.md). The original message stays unchanged.
The no-fetch description below is the older baseline, not the whole runtime.
Count-2 base-key persistence, higher-count policy, receipt keys and ordinary
group/broadcast retry remain unported; root routing normalization has separate
unresolved gates. No live retry is claimed for this correction.

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
`registration_changed_needs_new_session`, `identity_changed`. `listen` logs the reason;
`store inspect` counts both kinds.

The plaintext `Message` of each direct send is now retained at reservation as
`retry-source:<wire id>` (store-encrypted at rest like the stanza).

Not reproduced: the count-2 base-key record and the count > 2 comparison,
`<keys>` in the receipt, and group/broadcast retries in the normal runtime
(the comparison group-send window still answers its own). The fresh-bundle
path below and the later `enc count` correction supersede the older statement
that these were absent. Counts above two remain refused until the base-key
policy is ported.

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

## Fresh-bundle answer (2026-11-01)

A retry whose `<registration>` differs from the saved session (or with no
session) is no longer only recorded: following `151.A00` ("will wait to send …
until a new prekey has been fetched"), `answer` reserves
`retry-answer:<id>:<device>:<count>` as `fetching`, writes
`retry-refetch:<query id>` and returns an `encrypt` key query for the device.
`complete_refetch` handles that query's reply in `listen`: from the bundle it
derives a new session (pinned identity unchanged, else `identity_changed`),
re-encrypts the retained plaintext under the original message id, commits the
new session and attempt, and returns the resend (a `pkmsg`). A refused query
or unusable bundle records `refetch_refused` / `refetch_bundle_unusable`. One
fetch per (message, device, count). Offline-tested, including the new
session's remote registration; not live (a retry cannot be forced).


## Refetch reply admission correction (2026-11-04)

**Confirmed / implemented offline:** same-ID non-reply IQs no longer delete
the pending fetch or record refetch_refused. Exact result/error is required
before any checkpoint mutation. [APK 43.72 0ln.A01 and durable regressions](network-correctness-audit.md#retry-refetch-reply-type-admission-2026-11-04).

## Atomic refetch settlement (2026-11-04)

**Confirmed local correction:** retiring a pending fetch now commits together
with both refusal records or the reserved replacement send/session. Later errors
leave the job held instead of stranding fetching history. [Fault regression and evidence limits](network-correctness-audit.md#retry-refetch-atomic-settlement-2026-11-04).

## Refetch peer-state guard (2026-11-04)

**Confirmed local correction:** the reservation transaction rechecks the identity
and session captured before async preparation, refusing stale replacement without
retiring its job. [Guard regression and remaining concurrency scope](network-correctness-audit.md#retry-refetch-peer-state-reservation-guard-2026-11-04).

## Delivery during refetch (2026-11-04)

**Confirmed / fixed offline:** completion repeats acceptance/delivery checks
when a reserved key query answers and verifies the original send row is unchanged
at reservation. A delivery arriving while keys are pending suppresses resend.
[Native DNP basis, workflow regression and remaining scope](network-correctness-audit.md#retry-refetch-delivery-recheck-2026-11-04).

## Age while refetch waits (2026-11-04)

**Confirmed local correction:** completion and its resend transaction recheck
the original first-attempt age; overflow-safe comparison preserves strict >60-day
refusal. [Native timestamp distinction and tests](network-correctness-audit.md#retry-refetch-age-eligibility-recheck-2026-11-04).

## Saved-session retry reservation guard (2026-11-05)

**Confirmed local correction / implemented offline:** `answer` prepared crypto
outside the write transaction, then checked only the saved session and duplicate
answer marker. A concurrently committed delivery/failure/attempt update could
therefore be ignored, or a changed identity pin could be paired with the prepared
old-session result. The fresh-bundle path already guarded the original row and
peer state; both paths now share the same original/age and identity/session guards.

Immediately before reservation, the saved-session path requires the outbox row
to equal the snapshot used for eligibility/preparation, the current identity and
session to equal the captured inputs, the first attempt still to be no older
than60 days, and the answer marker to be absent. Any failed guard returns a
local error and rolls back the write transaction: no retry attempt, reserved
answer marker or prepared ratchet is committed. It does not overwrite newer
state or silently retry. The exact60-day boundary remains eligible.

The retained APK DNP.run delivery exclusion and150.A04 age checks motivate
rechecking eligibility before resend; transactional equality/CAS is our local
concurrency mechanism, not a claimed Android database implementation or new
server policy. Native count/root/addressing rules remain separate.

Regression tests simulate a captured snapshot followed by a committed delivery,
failure or extra attempt, then try the same reservation guard/transaction. They
verify unchanged tables on refusal and success at exactly60 days; another case
crosses the age boundary without a row change. Existing peer-state tests cover
identity/session changes and retained pending-job state, and the complete
saved/fresh retry workflow remains covered. These are deterministic store/guard
simulations, not an end-to-end scheduled thread race or live retry test.

The guard assumes every outbox/pin/session change uses the same ProtocolStore
transaction boundary. Other retry families, higher counts and complete current
APK root normalization remain open. No live account operation occurred.

Validation: `cargo test --locked --workspace --all-features` passed1311 tests;
strict workspace/all-target/all-feature Clippy and formatting passed. CLI
reference regenerated/checked without a reference diff. Pinned root-gate export
reproduced; coverage and repository hygiene checked before push.

## Refetch query reservation inputs (2026-11-05)

**Confirmed local correction / implemented offline:** the initial retry key-query
reservation previously checked only for an existing answer marker. The eligibility
and peer snapshots came from earlier read transactions, so competing store work
could make them stale before the fetching marker/job were committed. Even though
this branch does not await crypto, other store users can commit between those
transactions. The later refetch-completion guard did not prevent that unnecessary
query reservation.

`reserve_refetch` now reads the current original via its wire-id index within
the same write transaction as the two pending records. It requires the original
outbox row to equal the eligibility snapshot, repeats the strict >60-day age
check and compares identity/session bytes with the captured inputs (including
an absent session). A stale/missing original or changed peer state returns a local
error with no fetching marker or job committed. An existing answer marker still
returns false without another job. Only a successful reservation allows `answer`
to return the key-request node to its caller; no socket write occurs here.

This uses the same local guards as saved-session/fresh-bundle retry reservation.
It preserves the retained APK original-recipient/delivery/age exclusions without
claiming Android uses Rust's transaction mechanism. It does not delete/replace
a session, relax counts, rotate identity or add automatic retry after refusal.

A deterministic regression calls the actual reservation helper after a competing
transaction commits delivery, failure, an extra attempt, identity/session change,
an absent→present session, or deletion of the original. An aged captured original
also refuses at this boundary. Each refusal preserves all checked outbox,
checkpoint, identity and session rows. The unchanged case commits one pending
query; a second query ID with the same answer key returns false and adds no job.
The existing full saved/fresh retry workflow remains covered. These are store
interleaving simulations, not scheduled multithread races or live key requests.

**Observed live:** none. Full current-APK root normalization and higher-count
base-key policy remain unresolved. No new CLI command, route or protocol feature.

Validation: `1312` workspace/all-feature tests passed, along with strict
workspace/all-target/all-feature Clippy, formatting and CLI-reference regeneration.
Repository coverage/hygiene checked before push.
