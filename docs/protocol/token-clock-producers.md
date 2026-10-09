# Clock producers for outbound token jobs

**Confirmed static, 2026-10-09:** APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [seven-topic recipe](apk/recipes/2.26.37.71-token-clock-producers.json) and
[snapshot](apk/2.26.37.71-token-clock-producers/targets.json) preserve declared
signatures, register flow and catches. This extends the
[token issue clock boundary](token-issue.md), not active token scheduling.

## Stored offset and elapsed baseline

`08B.<init>` loads SharedPreferences `client_server_time_diff` (default zero)
into `A02` at `0036..0046`. It separately checks cached NTP inputs
`last_ntp_client_time` / `client_ntp_time_diff`, requiring positive saved time
and an absolute wall-clock age below 86,400,000 ms before `A01`. This NTP
initialization is not proof of an offset source for token jobs.

**Confirmed:** `08B.A07(server_ms, device_ms)` only updates for positive
`server_ms`. It records `A04 = server_ms - elapsedRealtime()` (`0018..0028`)
and `A02 = device_ms - server_ms` (`0028..0030`), then persists A02 under
`client_server_time_diff` (`008a..009c`). It makes ancillary clock calls before
and after; their complete tracing/logging effects are not established here.
`08B.A04`, the token job's getter, prefers nonzero A04 plus current elapsed
realtime, otherwise wall clock minus A02. `08B.A03` is exactly
`System.currentTimeMillis()`.

Physical field-write inventory finds A02 only in the constructor and A07
(`classes.dex` `0x45abd2` / `0x45ae18`, opcode `5a`), and A04 only in A07
(`0x45ae10`). Physical callers of A07 are `0iS.A0p` (`0x651f7c`) and
`0iS.A0y` (`0x652ed0`). Reproduce these inventories with `apk-field-refs`
using `--class 'LX/08B;' --field A02` or `A04`, `--type J --writes-only`,
and `apk-callers --class 'LX/08B;' --method A07` on the pinned APK.
These are DEX physical inventories, not reflection/native reachability proofs.

## Login producer

**Confirmed:** `0iS.A12` identifies a `success` node and reads its `t` string
(`00b4..00cc`, `00ec..0108`). It uses Java `Long.parseLong` and stores seconds
in `1Rd.A02`. It samples `08B.A03()` and divides by 1000 into `1Rd.A01`
(`0108..011a`), then calls `08l.A0t` with the server seconds. The divisor
is loaded as 1000 at `00c2`; renderer opcode `0xbe` at `0114` is currently
unnamed. Its standard meaning, `div-long/2addr`, follows the
[AOSP instruction table](https://source.android.com/docs/core/runtime/dalvik-bytecode),
but this chunk does not change the renderer. Invalid server-time parsing reaches
its NumberFormatException handler and logging branch, rather than supplying an
invented successful time. Missing `t` skips this parsing block. Initial/default
`1Rd` values and all failed-login consequences remain outside this trace.

Physical 1Rd.A02 write inventory identifies this A12 write only
(`classes.dex` `0x653ee0`, opcode `5a`). Later `0iS.A0y` reads both stored
values, multiplies each by 1000 and calls `08B.A07` at `0d68..0d86`.
The stored local wall time therefore has whole-second precision. The selected
update does not itself calculate a round-trip correction; no such correction
was established elsewhere. Its surrounding connection lifecycle and all
alternate login/control paths are not claimed complete.

## Timestamp event producer

**Confirmed:** `0iS.A0p` uses `1UB.A00` to select Android message argument 4.
It reads Bundle `timestamp`, multiplies by 1000, samples `08B.A03()` and calls
A07 at `004c..007a`. Selector 87 is a separate Boolean handling branch; other
selectors delegate to `0fY.CDs`. This is not proof that every received stanza's
`t` is a clock update. The wire producer of selector 4 and its timestamp
provenance remain **unknown**. Do not generate this event from message/receipt
time merely because those attributes also look like seconds.

## Rust boundary

**Confirmed local:** `transport::clock::observe` currently samples only
plausible authenticated message/receipt `t` attributes and local wall time, and
rejects older samples. This is the earlier documented inference for resolving
chat-mute times. Primary login persists `success` and account identity metadata
without using `success@t` for this clock. Neither module currently ports A07's
elapsed baseline or the persisted native offset. No incoming token timestamp,
receipt server time or local outgoing `accepted_at` supplies native token-job
clock authority. No real saved account, keys, request IDs, waits or live session
was changed while establishing this source.

**Next implementation boundary:** any port must distinguish the confirmed
login time from inferred stanza samples, preserve clock-domain/source provenance,
handle missing/invalid login `t` without fabricating an offset, and reproduce
elapsed-time behavior without treating process restart as the same monotonic
epoch. This trace supplies producers, not a verified complete Rust clock.

**Verification:** exact regeneration verified 12 files / 159,623 bytes;
self-diff reports seven topics and four artifacts unchanged. CLI reference
regeneration, API-symbol integration, diff whitespace and repository hygiene
passed (1,799 links; 72 checksum-verified bundles). This source-only chunk
changes no Rust; the preceding 818-test/Clippy/Rustdoc result is not a new run
for this continuation. Bulk private dumps remain local and ignored.

## Server-ping wire producer (2026-10-09 continuation)

**Confirmed static:** `1Qt.A02` is the IQ parser whose own diagnostics identify
missing/unknown IQ type. It reads envelope `type`, `id` and `xmlns`. For
`type=get`, namespace `urn:xmpp:ping` selects `00bc..00f2`: it reads envelope
`t`, passes it to `0Gv.A01(text, 0)` and calls `1Dz.A0t` with the resulting
seconds. This branch does not require the first child to be named `ping`;
the first-child variable is used by other branches. Result/error IQs use a
separate pending callback/removal path and do not enter this time branch.

`0Gv.A01` returns its supplied fallback for null/empty text or caught
NumberFormatException; otherwise it returns Java signed `Long.parseLong`.
Here the fallback is zero. This parser is not a positive-time requirement:
`08B.A07` later applies its positive millisecond check. Multiplication by 1000
is ordinary long arithmetic in the native event handler; range/overflow behavior
must not be confused with the current Rust plausible-stanza-time bound.

`1Dz.A0t` logs `ConnectionThreadRequestsImpl/onServerPingReceived`, calls
`A0B(seconds)` and passes the Android message to `1Dw.CDr`. `A0B` constructs
`Message.obtain(null, 0, 4, 0)` and stores Bundle `timestamp`. Thus selector 4
is **arg1**, not `what`; this agrees with the `1UB.A00` dispatch already
traced. Physical callers show one A0B caller (A0t, `classes.dex` `0x841dca`)
and one A0t caller (1Qt.A02, `0x8dc5bc`). Earlier selector-4 wire provenance
unknown is closed for this build's physical branch; reflective/native call
closure is not claimed.

**Confirmed boundary/negative result:** after the clock update, `0iS.A0p`
schedules `0iS.A07()` through `0yE.CQW`, then calls `1Dp.A03`. The A07 factory
builds Android `Message.obtain(null, 0, 0, 0)`, not a network IQ result.
`1Qt.A02` subsequently invokes `1Dz.A1F(node,id)`, whose body delegates to
`1Dw.Bqw` after logging. Neither selected body constructs a network pong.
This does not prove there is no later writer or that a reply is unnecessary;
Bqw's downstream behavior and writer remain untraced. No speculative response
or server-ping ACK was added to Rust.

Physical `apk-callers` reports zero direct calls of `1Qt.A02`, whose class
extends `1Qp`. This is not evidence that the parser is unused: polymorphic
method dispatch is outside an exact declaring-class/method pair inventory.
The full wire parser and both dispatch helpers are preserved in the
[six-topic recipe](apk/recipes/2.26.37.71-token-clock-wire.json) and
[snapshot](apk/2.26.37.71-token-clock-wire/targets.json), together with the
numeric parser and selected post-event/followup methods.

**Confirmed local audit:** Rust `connection::drive` currently handles the reply
to its own outstanding `w:p` IQ by ID; that is not this unsolicited get/ping
branch. `clock::observe` still only samples message/receipt time. A proper port
must preserve the distinction between confirmed login/server-ping input and
inferred stanza samples, with elapsed-time/restart and missing/invalid-time
handling. No Rust behavior, saved clock, account identity or live request was
changed by this source continuation.

**Wire-chunk verification:** exact regeneration verified 11 files / 44,131
bytes; self-diff reports six topics/four artifacts unchanged. CLI reference
regeneration, API-symbol integration, diff whitespace and repository hygiene
passed (1,804 links; 73 checksum-verified bundles). This source-only chunk
changes no Rust; full workspace/Clippy/Rustdoc was not rerun. Private APK scans
and caller inventories stayed in ignored local research storage; their scoped
findings and reproduction commands are retained above.
