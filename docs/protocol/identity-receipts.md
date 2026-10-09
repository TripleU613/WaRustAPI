# Identity changes, device receipts and privacy-token refresh

## Static evidence (2026-10-09)

**Confirmed, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [recipe](apk/recipes/2.26.37.71-identity-receipts.json) and
[23 checksummed topics](apk/2.26.37.71-identity-receipts/targets.json) retain
53 selected declarations, 28 files/323,094 bytes, register operands and catch
metadata. Whole-class store/chunk topics include bodies not individually
interpreted below; inclusion is not a complete call-graph or semantic port.
The existing [identity cleanup trace](peer-identity.md#companion-key-removal-scope-and-callback-boundaries-2026-10-08)
and [sender-key rotation](cross-device-sync.md#durable-own-sender-key-exposure-and-rotation-2026-10-07)
remain separate evidence.

`D4x.BqC` passes first Boolean false and preserves its caller's second Boolean
into `D4x.A00`; `BqD` passes null key, true, false. These source arguments matter:
the first Boolean suppresses the later privacy-token refresh on deletion.
Earlier native save/deletion traces establish the observer meanings. Complete
registration, callback ordering, canonical mapping and effective feature inputs
remain **unknown**.

## Primary receipt selection

**Confirmed:** `D4x.A00 0476..0496` derives userJid and isPrimary, then tests
`1xj.A1X -> 08Z.BO0`. Nonprimary jumps directly to `082a`; primary with BO0
true jumps to `0814`. The BO0/own-user interpretation follows the prior observer
context; its complete implementation is not newly traced here. Primary with
BO0 false executes the `identity_changed` refresh and receipt selection:

1. `04c0..057e`: derive the user's primary DeviceJid and call `0Z5.A04` for a
   set of device JID row IDs. Read message IDs from `receipt_device` whose
   `receipt_device_jid_row_id` belongs to that set and whose
   `receipt_device_timestamp IS NULL`.
2. `0584..06b4`: `8G8.A02` selects user JID row IDs. Intersect the candidate
   message IDs with rows in `receipt_user` for those user IDs. This query has
   no timestamp predicate; its diagnostic is `getDeliveredMessageIds`.
   Preserve the actual row-existence predicate rather than inventing one.
3. `06b4..07f4`: recompute primary device row IDs, then delete only
   `receipt_device` rows with both those device IDs and the selected message
   IDs. The DELETE itself has no timestamp predicate. It therefore removes
   matching device rows for the selected message, not just one null-timestamp
   candidate row. It does not delete `receipt_user` or the message body.

Both message-ID query/deletion passes use `0eK` with `0x3cf` (975).
`0eK.<init>` requires a positive chunk size; `0eL.next` returns the whole array
when it fits, otherwise consecutive `Arrays.copyOfRange` slices, advancing by
that size. This is local SQL batching, not a network limit, retry or server wait.

Deletion commits its inner token, then registers `DNd` discriminator 9 on
`0ZN.A04` (`07ac..07c6`). `DNd.run 07b6..07f2` iterates the selected message
IDs and calls `0Z5.A07`, which delegates to the `0Z8.A0E` cache operation.
Full 0Z8 internals and commit/rollback callback scheduling remain **unknown**;
registration alone does not establish when every cache effect has completed.
The two selection reads and later deletion use separate store contexts; no
single atomic select/delete snapshot is inferred from this trace.

## Counterparts, versions and companion cleanup

**Confirmed:** `0Z5.A04` and `8G8.A02` first resolve the supplied address with
`0VT.A07`, asserting the initial row is not -1. If `0Yo.A0H` is true and
`0Yo.A02` supplies a different counterpart, they additionally select its row.
For device rows, the counterpart preserves the actual device number through
`0DK.A02`. The counterpart lookup has no second -1 assertion in these methods.
`0Yo.A0H` consults cached migration state, database readiness and `0H9.A01`;
`0Yo.A02` delegates to `1wk.invoke` discriminator 2. Its transitive mapping and
current migration state remain **unknown**. This is not evidence that a Rust
contact-directory alias alone authorizes merging/deleting namespaces.

`D4x.A00 0814..082a` calls synchronized `0Z7.A02` for primary changes, including
the BO0-true branch that skipped receipt deletion. `0Z7.A00` reads the exact
user's `primary_device_version.version`, caching a found value and returning
null when absent. `A02` stores 1 for missing state or old+1, using insert
conflict mode 5, and removes that user's cached version. This is persistent
per-user device-version bookkeeping, not merely a cache clear. It is not the
signed device-list timestamp, Signal registration ID or global account epoch.

`082a..0848` wraps the actual changed device in a `Collections.singleton`
(`1xg.A1R`) and calls `0Z4.A07`. Its Set path calls `0Z5.A0B` on both concrete
stores. `A0B` excludes primary devices and deletes only null-timestamp rows
for the selected companion/counterpart device row IDs. `0Z6.A0G/A0I/A0J`
name `receipt_device`, `receipt_device_jid_row_id`, `receipt_device_timestamp`;
`0ZE` names `message_add_on_receipt_device` with the same two column names.
This is an additional companion-specific path, not the earlier primary
message-ID intersection. After deletion, A0B calls A06; its full cache reset
semantics are not newly traced.

The same `0Z4.A07` path handles `status_receipt_device` for nonprimary mapped
devices; it also maps eligible PhoneUserJid users to LID row IDs and deletes
null-timestamp `receipt_coex` and `message_add_on_receipt_coex` rows. Both
coexistence tables use `receipt_coex_timestamp` in their shown DELETE
predicates; the ordinary device tables use `receipt_device_timestamp`.
Full type predicates (`0DE.A0a/A0e`), mapping authority, receipt insertion,
cache internals and coexistence semantics remain **unknown/unported**.
The SQLiteException catch at code units `0415..0425` covers the later device
cleanup call and logs `delete-companion-receipts-failed` before sender-key
handling; it is not proof that every earlier receipt SQL failure is swallowed.

## Privacy-token refresh

**Confirmed:** on the non-deletion branch (`07fa..0814`), `HxT.A02` normalizes
its user through `A00`, then `A05` admits only the first user entry into a
synchronized pending set. It loads an existing `HYS` via `0q2.A0O` and queues
`GeneratePrivacyTokenJob` only when its timestamp is at least `0q3.A01`'s
threshold. Missing/expired tokens unmark the pending user through `A03`.
`A03` removes only a pending-set entry; it does not erase the stored token.
Duplicate admitted users do not enqueue another job through this method.

`HxT.A00` uses property `0x37df` (14303) to select `0q2.A0L`; its other branch
may resolve a LID through `0Xw.A0H`, retaining the original when unresolved.
The effective canonicalization/freshness properties, full token scope and base
job-queue terminal dispatch remain **unknown**. Rehydration calls A05 to restore
pending membership; complete manager initialization is still untraced.
The subsequent [job lifecycle trace](identity-token-refresh.md) confirms the
threshold formula, outbound timestamp input, request, callbacks and explicit
unmark/retry branches. It corrects the assumption that incoming token bytes
supply the job's state.
The [status evidence](status.md#refuted-privacy-tokens-do-not-gate-status-delivery)
retains the earlier interpretation and its correction. The traced job requires
existing outbound issue/send state; incoming bytes cannot manufacture that state.

## Rust boundary and next port work

**Confirmed, local audit:** `identity_conflicts::archive_identity` preserves
old pin/session, invalidates local verification, requests remote-primary device
refresh and marks indexed own sender-key exposures pending, in the authenticated
identity transition. It does not implement these native receipt/version/token
side effects. `notifications::record` already stores supported incoming
trusted-contact tokens; there is no equivalent outbound refresh job here.

| Native state | Current Rust state and boundary |
| --- | --- |
| Per-message device/user receipt rows | Delivery facts are retained on saved sends/outbox entries; native row/alias/version policy is not ported. |
| `primary_device_version` | No native per-user counter/cache model; signed device-list timestamps must not be repurposed. |
| Token pending set and refresh job | Incoming token storage exists; canonical refresh scope and job lifecycle remain unfinished. |
| Receipt-send reservations | Rust `receipt-outbox:*` is a once-only wire reservation/ACK journal, not a native receipt-device cache. |

Do not clear acknowledged or uncertain receipt-send reservations to imitate
these SQL deletes. Native receipt/user row selection does not authorize send
replay, ratchet resets, identity adoption or history removal. This research adds
no cleanup worker, counter, network request or private-state migration.

**Corrected local receive finding, separate from cleanup:** the handler is
`receipts::record`. It already mapped some LID receipts to PN, so a PN original
could match. Same-LID originals without a map, reverse PN-to-LID attribution,
encoding equality and retention of the actual actor were the narrower gaps.
The subsequent [incoming receipt audit and correction](direct-receipts.md)
re-anchors the current APK parser to `Cmk`, uses a transactional saved-chat
association, and preserves actual receipt actors. Native cleanup remains
unported; this is not authority to clear receipt-send reservations.

Reproduce the source offline:

```sh
cargo run --locked -p xtask -- apk-research-bundle \
  ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-identity-receipts.json \
  --output docs/protocol/apk/2.26.37.71-identity-receipts --check
```

For another release, re-anchor classes/methods and export a separate recipe and
bundle, then run `apk-research-diff` on the two directories. Fast cached diff
is not proof of semantic equivalence, runtime feature selection or a complete
APK-to-Rust port.

## Validation

Exact APK/recipe re-export passed `apk-research-bundle --check`. A checksum-
verified cached self-comparison took 0.026378 seconds for 23 source topics and
four metadata artifacts. It reads no APK or account state and is not a
cross-release equivalence claim. This chunk changes evidence and documentation
only; no Rust runtime code, private account state or live connection was changed.

CLI reference regeneration and API/library-symbol validation passed; repository
hygiene validated 1,087 local links and 5,531,661 bytes across twelve APK checksum
bundles. No new runtime test is claimed for this static research chunk.
