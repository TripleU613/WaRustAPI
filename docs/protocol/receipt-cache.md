# Receipt cache invalidation and incoming timestamp lookup

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eight-topic recipe](apk/recipes/2.26.37.71-receipt-cache.json),
[retained sources](apk/2.26.37.71-receipt-cache/targets.json).
This closes cache operations referenced by [identity receipt cleanup](identity-receipts.md).
It does not port the native mutable receipt ledger or authorize resending.

## Signed cache-clear sentinel

**Confirmed local/static:** the DEX 11n const/4 literal is a signed four-bit
value. 0Z8.A0G supplies -1 to 0D3.trimToSize. The previous renderer printed
the correctly decoded signed i8 only as `#0xff`, obscuring its meaning.
The renderer now adds `(-1)` (and analogous values -8..-2); raw hexadecimal
spelling, registers and instruction widths are preserved. All sixteen nibbles
are tested from independent literal byte encodings. Positive output is unchanged.
Existing immutable research generations retain their predecessor rendering;
this new generation uses the corrected formatter. The predecessor executable
is preserved privately for reproducing old generations. This is a display fix,
not a changed WhatsApp protocol value or a 255-entry cache threshold.

## Primary and companion invalidation

**Confirmed:** 0Z5.A07(long message row) boxes its argument and calls its
0Z8.A0E. A0E removes only that key from the underlying 0Z9/0D3 cache and
returns the unwrapped value, or null. In the optional 7cQ mode it removes under
a monitor and then runs A01's callback drain, including on exceptional exits.
This anchors the selected-message invalidation scheduled after primary cleanup;
it does not erase stored messages, receipt_user rows or immutable send snapshots.

0Z5.A06 calls 0Z8.A0G. Without optional 7cQ state, A0G invokes
trimToSize(-1). With 7cQ it delegates to A0F: trimToSize(-1) under the monitor,
clear 7cQ.A01 LinkedHashMap and A02 ConcurrentHashMap, then drain A01.
0D3.trimToSize removes entries while its size exceeds the bound and the map
is nonempty, decrements the measured size, increments evictionCount and invokes
entryRemoved(true,key,value,null) outside its lock. With -1, that loop clears
all entries, including any zero-size entry still present. Inconsistent size
bookkeeping throws instead of silently succeeding.

0Z8.A01 removes a thread-local callback list before iterating it, invokes
7qv.A01 Runnable values, releases callback records through 7qv.A00 and
accumulates RuntimeExceptions before throwing. Complete 7qv ownership and
0Z9/0ZA/70h eviction callback producers remain **unknown**. The shown clear
can have callbacks; it is not enough to claim every effect is a plain Map.clear.
The outer database commit callback scheduling remains untraced here.

These two operations explain why native receipt-cache membership can change
after identity cleanup. The companion path clears the store cache broadly;
the primary path invalidates selected message rows. Neither operation rewrites
Rust's immutable original-target capture into the native mutable set. Native
SQL deletion predicates remain those separately documented in identity-receipts.

## Incoming timestamp-only SQL fallback

**Confirmed:** 0Z6.A0N(Collection<CYV>) skips messages whose row is -1,
canonicalizes the device via 0Z5.A01 and expands row counterparts with A04.
It builds message/device row pairs and an OR query over `receipt_device`,
selecting only message_row_id, receipt_device_jid_row_id and
receipt_device_timestamp. The query does **not** select primary_device_version.
Each result resolves its actual device row through 0VT.A0D, canonicalizes it
for the message, gets the timestamp with Cursor.getLong and applies 7hf.A01
(the [earliest-positive policy](receipt-clock.md)) to a per-CYV result map.
No explicit Cursor.isNull check appears in this retained path; do not infer a
version value or complete null-loading policy from that omission.

A missing resolved DeviceJid logs and skips that row; a returned pair missing
its request-map entry throws IllegalStateException. SQLiteDatabaseCorruptException
logs, calls 0Fy.A03 and returns the accumulated map. Other exception/cleanup
paths remain in the full catch table; this is not permission to suppress all
storage failures in Rust. A0N's log says NoCaching: this is a transient fallback
for incoming timestamps, not proof of the general full-message cache loader.
The separate A0M version query retained in receipt-version.md remains distinct.

**Unknown/unported:** the general message cache load (0Z6.A00), complete 15A
message/status admission, all namespace/counterpart authority, cache-mode default
and eviction callbacks, and full resend worker/device-key/send recovery policy.
No retained account was read or changed, no send/receipt was queued and no live
validation is claimed for this offline research/tooling change.

Reproduce this bundle with the recipe, --output and --check. Export another
release to a separate anchored generation before apk-research-diff. The existing
apk-smali CLI and bundle tooling expose the corrected annotation; no new flag
or HTTP route was introduced. The OpenAPI research-tool entry records the output
change with CLI transport and x-live=false.

Receipt-cache validation: all 66 xtask tests, strict xtask Clippy, rebuilt
research CLI, CLI-reference regeneration, OpenAPI/library-symbol check, formatting
and diff hygiene passed. Eight source topics / 13 files / 57,611 bytes reproduce
exactly with the corrected renderer; cached self-diff verifies eight unchanged
topics and four metadata artifacts. Hygiene checked 1,359 local links and
30 APK bundles / 8,199,368 bytes. No transport code changed; previous 798-test
runtime validation is not claimed as a fresh run. No live account operation ran.

Follow-up: [general receipt loading](receipt-load.md) now establishes pending
row membership and distinct general-load namespace mapping. The mutable set
used by resend subtraction is not a set of delivered devices. Complete
receipt_user/status admission and namespace authority remain unported.
