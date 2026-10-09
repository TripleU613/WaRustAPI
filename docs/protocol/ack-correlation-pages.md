# ACK correlation across saved history

2026-10-09. **Confirmed local bug/fix**, APK context **2.26.37.71**,
SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [existing ACK admission trace](message-ack.md) retains the actual parser,
participant guard and worker. This change repairs local correlation; it does
not promote raw acceptance facts into native receipt_user membership.

## Runtime correction

`outgoing::acknowledge` previously scanned at most **10,000 outbox rows** and
returned `pagination required` when that count was reached. Completed/cancelled
history could therefore prevent a new matching server ACK from settling its
original uncertainty. This was our local limit, not a WhatsApp server rule.

**Implemented/offline tested:** it now uses the existing
`Outbox::by_wire_id_in_write` lookup, which scans complete history in 256-row
pages, across caller namespaces, and rejects duplicate wire IDs or malformed
saved entries. Lookup and once-only fact mutation share the original write
transaction. It never chooses a first storage-order match under ambiguous wire
authority. The page size is local memory bookkeeping, not a network limit.

The selected intent still requires **InFlight**, exact saved recipient and
matching original wire ID. Existing supported participant success exclusions
and error-ACK handling remain in place. An invalid wire-ID shape is ignored
without writing. Terminal ServerAccepted/Failed facts still suppress repeats;
no intent, attempt, identity, key, request ID or server wait is reset. An error
ACK still records Failed; success still records ServerAccepted, with local
observation time. Server ACK time is now retained separately in Rust facts; see
[ACK timestamp evidence](ack-timestamps.md). Native counter/token persistence
and status/user-row admission remain **unknown/unported**.

Existing `status` exposes the durable facts; `listen` processes authenticated
ACKs. No new command, flag or HTTP route exists. Native state 4 is not assigned
by this adapter, and an acceptance fact is not handset delivery or a read.

## Regression evidence

The synthetic encrypted-store regression puts an in-flight intent after
**10,000 cancelled history rows**. With the original implementation restored,
the test fails with `pagination required`; with the correction it accepts once,
keeps 10,001 rows, survives reopen, and has no eligible pending send. A separate
fixture duplicates the same wire ID under a different chat and proves fixed
ambiguity refusal with the original entry unchanged. Replacing that duplicate
with a malformed stored row also refuses without acceptance; reopen preserves
the original entry. These fixtures contain synthetic addresses and no network
request. Existing participant, error, uncertainty and recipient tests remain
part of validation. No qualifying fresh live ACK was generated: no existing
acknowledged JSON was resent or assigned a new ID for this check.

## Static recheck, correction and negative findings

**Confirmed static correction:** 1Dz.A0H calls
`Message.obtain(null,0,19,0)` in the retained
[producer](apk/2.26.37.71-message-ack/ack-android-message-producer.txt).
Thus **what=0, arg1=19, arg2=0**; the earlier prose saying what=19 was wrong.
[Dispatch evidence](receipt-ack-special.md#dispatch-reads-arg1-not-what)
shows 1UB/1Dy read arg1. This correction changes no runtime wire field.

The worker/parser recheck reused the existing message-ack bundle; it did not
require a second copy or establish new native admission. Candidate **152.A00**
and **153.A00** were rechecked against the existing
[revoke-consumer bundle](apk/2.26.37.71-receipt-manager/targets.json): diagnostics
identify edit/revoke processing, not a fresh text factory. **0qA.A03** is the
already retained [media factory](apk/2.26.37.71-message-loader/targets.json)
with explicit initial status 1; it does not establish text initialization.

The newly retained [two-method recipe](apk/recipes/2.26.37.71-sender-address-status.json)
and [bundle](apk/2.26.37.71-sender-address-status/targets.json) trace
**1sS.A00**: CW6 writes A0g and clears A0W, then the helper sets native status
**4** only when the message key is outgoing. The [exact caller inventory](sender-address-status-evidence.txt) finds **four
physical invokes**, all **1fm.A04**, at classes2.dex absolute
**0x12487e, 0x124980, 0x124f3c, 0x125382**. The previously retained orphan/add-on
resolution context is in [message status](message-status.md#separate-workflows-not-universal-initial-state-evidence).
This helper is not proof that all fresh text sends enter state 4 at construction.
Complete virtual dispatch, address authority and transitive effects remain unknown.

**Search scope:** case-sensitive `apk-strings --contains SendE2e --contains
AckHandler` returns 49 strings, predominantly the status job and named handlers;
that mixed-case search does not match `SendE2EMessageJob`. The broader Ack/ack/
search located the existing 1Qy.A02 and DMZ.run diagnostics. Three exact xrefs
(one per anchor) revalidated the known owner methods, not a newly discovered
normal send implementation. Future investigations should consult message-ack.md
before repeating that chain. Full primary cleanup remains unfinished.

## Validation

ACK pagination validation: 829 workspace/all-feature tests, strict all-target
Clippy, warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), CLI
reference, API-symbol, formatting and whitespace checks passed. The regression
fails on the old implementation with pagination required. Exact two-topic source
export/self-diff and hygiene (2,119 links/105 APK bundles) passed. No live ACK,
send, account migration or replay ran.

The new source bundle is seven files/29,470 bytes; its two topics and four
metadata artifacts are unchanged in cached self-comparison.
