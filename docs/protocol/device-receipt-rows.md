# Original-device pending rows and receipt timestamps

2026-10-09. **Confirmed static** evidence uses APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Implemented/offline tested** behavior below is a bounded port of original
pending rows and earliest-positive timestamp retention. Primary/counterpart cleanup, per-user status admission and resend remain
unfinished. Exact companion cleanup is described in the follow-up below.

## Evidence and scope

0Z5.A0D → DPo.invoke selector 1 → 0Z6.A0L seeds pending device rows with
nullable primary versions before receipts arrive. See the full methods and
branches in [receipt-version](receipt-version.md). This makes pending membership
independent of a positive delivery timestamp. [Receipt-load](receipt-load.md)
traces reconstruction of every saved device row and sync's key-set subtraction,
including pending rows. Immutable targets alone and observed delivery facts
therefore cannot implement the native mutable ledger.

Cmk.A03 04e0..04fe reads optional root `t` through 8kT.A18 and
0Gv.A01(string,0), then DEX mul-long multiplies seconds by 1,000. DDH
carries that value separately from actor and status. 7hf.A01 rejects nonpositive
times and keeps the earliest positive time; an existing row's nullable version
is retained. These are **confirmed**, with methods in
[receipt-clock](receipt-clock.md). 15A.A02 07b4..07ba routes equal/lower
status events to the device store too, so fact deduplication must not prevent an
earlier timestamp from being retained. See [receipt-admission](receipt-admission.md).
A device timestamp is not a read/played timestamp or proof of message status
advancement; [receipt-user](receipt-user.md) traces those separate columns.

## Rust storage and inspection

**Implemented:** `send_versions::capture_in` seeds encrypted schema-1
`outbox-device-receipts:{local_id}` in the same original reservation transaction
as `outbox-primary-versions:{local_id}`. It includes every actual direct/group
plan target, even when no new sender-key child is needed. Each row stores its
exact normalized original device, original nullable version and null timestamp.
The original version/target snapshot remains immutable. This adds no standalone
writer, migration, historical reconstruction or new send permission.

The existing authenticated direct and targeted group receipt paths call
`observe_in` before deduplicating delivery/read/played facts. An exact original
device row receives the minimum positive root `t` value, parsed as signed ASCII
seconds and multiplied with signed 64-bit wrapping semantics. Missing, empty,
malformed or out-of-range input defaults to zero. Nonpositive results leave rows
unchanged; no local clock is substituted. **Parity limit:** Java's Unicode-digit
parsing is not ported. Aggregated participant receipt branches use different
native parsing/default policies and are not handled by this root-receipt port.

A repeated fact can update its timestamp while returning zero new facts and
emitting no duplicate event. Fact timestamps retain their existing local
observation-time semantics. Receipt metadata, facts and journal writes share one
transaction; a later failure rolls all of them back. Original attempts, payload,
wire/local IDs, progress, account identity and trust state are not rewritten.

```sh
whatsapp-cli --profile l --json outbox-versions --local-id ORIGINAL_LOCAL_ID
```

`device_receipts` exposes only device hashes, components, `at_send` nullable
versions and nullable server-millisecond `timestamp`, plus current `present`
membership and nullable row `version`. Missing historical rows
are null, including historical snapshots with known exact targets: inspection
and receiving never backfill them. Pending rows have null timestamps. This is
an offline CLI/library capability; the OpenAPI path is a contract with no served
HTTP route and `x-live=false`.

Malformed/missing required row members, duplicate/unplanned devices, altered
initial versions, wrong provenance counts and nonpositive stored timestamps refuse with
fixed redacted diagnostics. Rows require exact immutable provenance; failures
neither repair private state nor reveal input values. Full original provenance coverage remains required; mutable membership is
now separate under the companion cleanup rules linked below.

## Remaining boundaries and validation

**Unknown/unported:** native conditional PN/LID canonicalization authority,
message/settings status gates, aggregated receipt parsing, per-user timestamp
rows, primary/counterpart invalidation/deletion, cache lifecycle and full sync/resend.
Existing verified alias attribution may establish a fact, but this port does
not merge another namespace into an original device row. A receipt for an
unplanned device cannot add a row. Pending rows are not yet used to subtract
resend targets, and nullable version equality remains diagnostic only.

Offline regressions cover pending null/version capture, exact namespace/device
selection, earliest-positive duplicate handling, invalid/missing/overflow times,
restart, immutable provenance, historical unknowns, redacted malformed-row
refusal and transactional rollback after a later group record fails. Both normal
direct and group receipt entry points are exercised. No fresh chat send or live
receipt-row capture is claimed; retained historical sends cannot validate a
new reservation-time capability without an authorized new stable intent.

Validation completed: workspace/all-feature tests passed 802 tests (API 671,
properties 4, crash matrix 9, CLI 51, API symbols 1, xtask 66). Strict all-target
all-feature Clippy, warning-denying Rustdoc, formatting, default CLI (25 plus
one API-symbol test), CLI-reference regeneration and experimental CLI build
passed. Hygiene checked 1,405 local links and 33 bundles / 8,507,021 bytes.
The retained profile's store inspection and eligible queued intents were unchanged;
invalid/unknown selectors refused without echo. No retained group preparations
were available to inspect, so no historical live send row was demonstrated.
Zero network commands ran. A repeated workspace run exposed a pre-existing
immediate-close ownership refusal; the shared-descriptor teardown regression
and normal owner-drop correction are documented in
[verification](../design/verification.md#temporary-store-ownership-refusal-observed-2026-10-09).

Follow-up: [authenticated companion cleanup](companion-receipt-cleanup.md) now implements
pending exact-device membership removal on ordinary direct/group rows and late
positive receipt reinsertion with the current nullable user version. Original
at_send remains immutable; new present/version diagnostics distinguish mutable
membership. This supersedes the earlier full-membership/no-cleanup limitation
for this branch only. Primary and namespace/counterpart cleanup remain unported.


## Java receipt time and companion cleanup correction (2026-10-09)

**Confirmed static:** APK 2.26.37.71 / SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The existing [clock recipe](apk/recipes/2.26.37.71-receipt-clock.json) and
[snapshot](apk/2.26.37.71-receipt-clock/targets.json) retain Cmk.A03's direct
`t -> 0Gv.A01(text,0) -> wrapping mul-long 1000` chain. The more recent
[wire-clock numeric helper](apk/2.26.37.71-token-clock-wire/targets.json)
retains 0Gv.A01; its success path uses Java Long.parseLong. Missing, empty or
invalid text returns zero; valid signed long multiplication wraps before the
existing 7hf.A01 positive-millisecond admission. Those distinct stages remain
unchanged. This is direct-root receipt time, not the separate strict per-item
aggregated/participants parser or an inferred local time.

**Confirmed local bug and correction:** send_versions::observe_in had still
used Rust parse::<i64>, which rejects Java BMP decimal digit inputs. A matching
receipt could therefore leave a device timestamp null and later be selected by
our pending-companion cleanup. The new failing-before regression refused its
first update for `+２０`. The observer now reuses mex::java_decimal_long,
keeping zero fallback, wrapping milliseconds conversion, positive admission,
exact original device/namespace match and earliest-positive replacement.
No stored schema, version counters or original reservation are rewritten.

**Regression scope:** synthetic encrypted-store input `+２０` establishes
20,000 ms, then `١٠` lowers it to 10,000 ms. Whitespace, supplementary digits,
nonpositive values and out-of-range longs leave it unchanged. An aborted
companion cleanup rolls back. A subsequent committed cleanup retains this
received companion's row and removes the separate still-pending companion's
membership, while immutable original user/version snapshots stay unchanged.
Reopening preserves that result. The existing normal-outbox receipt integration
now exercises the same BMP times through receipts::record, preserving peer
scope, idempotence, one Read fact and no invented server acceptance. Independent
Java numeric fixtures are [documented separately](token-notification-peer.md#timestamp-parser-correction-2026-10-09).

**Remaining boundary:** this correction prevents parser-created false pending
state inside the existing device-row port. It does not implement native
receipt_user membership, primary identity cleanup, counterpart authority,
status/settings admission, aggregate parsing or resend. No actual peer identity
was replaced and no real account, keys, saved request IDs, waits or session
changed. A fresh live Unicode receipt has not been observed or requested.

**Completed checks:** 822 workspace/all-feature tests passed (686 API unit,
four codec, nine crash matrix, 52 CLI unit, one API-symbol and 70 xtask).
Strict workspace/all-target/all-feature Clippy, warnings-denied Rustdoc,
default CLI tests, regenerated CLI reference, formatting and diff whitespace
passed. The new companion-cleanup regression failed before the parser change
and passed afterward, as did the modified normal-outbox entry-point test.
Repository hygiene validates 1,830 local links and all 76 existing APK checksum
bundles; no new source generation or old-snapshot rewrite was needed.
