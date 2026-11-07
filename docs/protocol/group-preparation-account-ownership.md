# Group preparation account ownership

## Confirmed local defect and correction (2026-11-07)

**Confirmed source/offline:** normal outgoing group preparation retained its
intent and phase but did not retain the account that began preparation. A red
regression began membership preparation under registration ID31, saved ID32,
and demonstrated that timeout handling still changed the durable preparation.
The corrected test refuses that timeout with unchanged rows and journal; reopen
preserves the uncertain preparation and does not reserve it again.

New private Row.account_authority captures the shared account/store fingerprint
inside begin_selected's transaction. Pending checks saved authority before reply
handling and key preparation. Preparation saves, device-query reservation and
final sender-key/session/outbox reservation recheck inside their write transaction.
Distribution advancement checks authority before returning another node. When a
saved account exists, supplied crypto account authority must also match. Only a
digest is retained; no new wire field or credential is transmitted.

The group passes its original owner into directory reserve_next_owned,
settle_owned and assembled_owned. Each checks that owner in the same transaction
or immutable snapshot as the child operation. This prevents the first child page
from adopting a replacement account between the parent's read and child creation.
A separate regression replaces the owner before first-page creation and verifies
no batch or state mutation; the original fixture owner can complete exactly once.
Existing public directory APIs retain their own documented batch ownership rules.

## Limits and verification

**Unknown legacy provenance:** absent Row metadata remains readable for historical
inspection, but is not manufactured into matching authority. Existing rows are
not resumed or replayed by begin_selected. Account-less offline adapters bind
explicit absence; they do not prove a live connection account. These fingerprints
are local comparisons, not protection against malicious local metadata rewriting.

This closes the preparation cuts described above, complementing the normal
[group writer guard](group-writer-account-ownership.md) and
[key-batch binding](key-batch-account-ownership.md). It does not prove atomicity
between separate parent/child transactions, all read-to-socket timing, generic ACK
settlement, every adapter, or full APK/network parity. No reset or replay added.

No new APK export was needed: this is a local Rust ownership defect, not a new
server rule. Pinned APK 2.26.43.72 evidence is unchanged. No live account operation,
server acceptance or restriction outcome was observed; zero bans is not proven.

Verification: **1,607 workspace/all-feature tests passed**, plus strict
workspace/all-target/all-feature Clippy. Formatting, CLI/OpenAPI checks, APK
coverage freshness and staged repository hygiene are required before commit.
