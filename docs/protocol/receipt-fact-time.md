# Preserve supplied receipt time in public send facts

2026-10-09. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Confirmed clock trace](receipt-clock.md) and its
[retained source](apk/2.26.37.71-receipt-clock/targets.json) ground the conversion.
Cmk.A03 04e0–04fe reads optional root t through 8kT.A18, calls
0Gv.A01(default 0), and uses signed DEX mul-long by 1,000. This change does
not claim a new APK release, timestamp clock authority or native user-row admission.

**Confirmed implementation gap:** authenticated direct/group receipt paths
previously stored public DeliveryFact.server_timestamp=None even when root t
was parsed for a saved device receipt. The field already existed for success
ACKs. Public send status now retains the valid supplied root time for newly
recorded delivery/read/played facts attributed to an existing original send.
All current route, namespace and original-target admission guards remain.

The shared receipts::server_timestamp helper uses the existing Java signed
BMP-decimal parser, then wrapping multiplication. Valid zero/negative/overflow
results remain signed evidence in public facts, not a positive native receipt
row. **Local projection choice:** missing, nontext or malformed input stays
None in public facts so it cannot look like a supplied zero timestamp.
send_versions::observe_in reuses this parser but still defaults None to zero
and admits only positive milliseconds, preserving its native device-cache path.
Success ACKs retain their separately documented default-zero policy.

Public facts retain the **first observation** per kind/actual device. A later
receipt does not rewrite that fact or its local observed_at time. Device rows
can independently retain an earlier positive timestamp even on an equal/lower
status. The two values can differ: first read t=20 seconds remains 20,000 in
the public fact while a subsequent read t=10 updates its device row to 10,000.
Historical missing facts are not backfilled; original reservations, identities,
attempts and saved payloads do not change. Comparison-lab Boolean summaries
remain Boolean summaries without invented historical timestamps.

## Inspect and verify

The existing `whatsapp-cli status --local-id ID --json true` returns send facts
including optional server_timestamp in milliseconds, separate from observed_at.
This is local inspection; it performs no HTTP or WhatsApp request. Existing
listen receive processing populates new facts; no new flag or route was added.
See [API transport declarations](../api/openapi.json).

Offline tests cover ASCII/BMP digits, absent/empty/malformed/out-of-range and
signed-long wrapping boundaries, all three fact kinds, reopen/idempotency,
historical fact preservation and immutable send state. Existing direct/group
integration tests distinguish first observed time from earliest device time.
Fresh live receipt timestamp projection remains **unknown** until a qualifying
new receipt is received; old captures or another unchanged receive window do
not establish that result. Full native receipt_user admission/primary cleanup
remains separately unported.

Validation: 887 workspace/all-feature tests passed (746 library, 4 codec,
9 crash, 54 CLI, 1 API-symbol and 73 tooling tests). Strict all-target/all-feature
Clippy, warnings-denied Rustdoc, default CLI tests, CLI-reference regeneration,
formatting and whitespace checks passed. Hygiene checked 2,479 local links and
127 APK bundles / 26,797,855 bytes. Existing APK evidence was reused; no new
export or cross-release parity is claimed.
