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

## Confirmed runtime clock inputs (2026-10-09)

**Confirmed local implementation:** primary login now records authenticated
`success@t` in the same store transaction as success metadata/account changes.
The existing authenticated receive loop records `iq type=get` with
`xmlns=urn:xmpp:ping` through the clock observer. No child is required for this
namespace branch. Both use the existing Java decimal-long converter, ordinary
wrapping seconds-to-milliseconds multiplication and the APK's positive-ms
admission; missing, invalid or nonpositive results leave earlier confirmed
state untouched. Positive backward corrections replace confirmed state, as in
A07. The login local wall sample is truncated to seconds before being widened,
while ping samples use wall milliseconds. No historical capture replay occurs.

Confirmed state is stored separately at `server-clock-confirmed-v1` in encrypted
checkpoints, with source `login_success` or `server_ping`, client/server difference,
optional elapsed baseline and a process epoch. Linux/Android use rustix's
`CLOCK_BOOTTIME`, which includes suspend; Android
[elapsedRealtime](https://developer.android.com/reference/android/os/SystemClock#elapsedRealtime())
also includes deep sleep. The epoch is one local random nonce per process,
never an account identity/key or wire field. Its purpose is to prevent a
persisted in-memory baseline from being reused after restart. Same-process
nonzero baselines resolve with elapsed time; a different process, unsupported
host or unavailable epoch resolves wall time minus saved difference, matching
the traced A04 fallback. A zero baseline also uses that fallback.
Rustix was already present in Cargo.lock; its direct Linux/Android time dependency
adds no new resolved package version. No platform suspend/live parity is claimed
from synthetic clock tests alone.

**Local selection:** client chat-mute time resolution now prefers this confirmed
clock. Historical `server-clock-sample` message/receipt inference retains its
old format and update rule; it is used only when confirmed state is absent.
Even a newer inferred timestamp does not supersede confirmed time. Absence of
both sources remains no estimate. This improves the previously inferred mute
clock but does not port NTP-preferred `08B.A00`, NTP fetch/update or token-job
scheduling. Full native mute-clock parity remains unfinished.

**Offline inspection:** `whatsapp-cli clock-inspect` opens the existing encrypted
store without creating it, adopting captures, queueing requests or contacting
WhatsApp. Output has exactly `confirmed_source` (nullable), `resolution`
(`elapsed_realtime`, `saved_offset`, `inferred_stanza` or `unavailable`) and
`inferred_sample_present`. It omits timestamps, offsets, nonce, raw packets,
account addresses and keys. The API contract is CLI-only; no HTTP route exists.
An inspection in a separate CLI process normally reports saved_offset for a
previous process's confirmed update, rather than claiming a current elapsed epoch.

**Verification scope:** new synthetic-store tests cover get/ping versus result
and other namespaces, missing/bad/negative/wrapping-negative timestamps,
positive backwards login correction, precedence over newer inferred data,
whole-second login samples, wall jumps, simulated process restart/unsupported
host and zero-baseline fallback, transactional rollback, reopen and redacted
inspection. These are new-capability tests, not a claimed failing-before runtime
regression. Native numeric input conversion reuses independently tested Java
fixtures. No real saved account, keys, request IDs, cooldowns or live session
was changed during implementation.

**Completed checks:** workspace/all-feature tests passed (820 tests); strict
workspace/all-feature/all-target Clippy, warnings-denied Rustdoc, default CLI
tests (25 unit plus API-symbol integration), regenerated CLI reference and
experimental API-symbol integration passed. Formatting, diff whitespace and
repository hygiene passed (1,808 local links; 73 checksum bundles). A default
CLI build exposed an initially feature-gated rustix dependency; the dependency
is now Linux/Android target-gated and available to the unconditional public
clock module. Both default and all-feature configurations passed after that
correction. Separate-process CLI clock resolution and physical suspend remain
unverified; restart coverage uses a simulated changed process epoch.


## NTP preference and acquisition (2026-10-09 continuation)

**Confirmed static:** same 2.26.37.71 APK/hash as above. The
[nine-topic recipe](apk/recipes/2.26.37.71-ntp-clock.json) and
[snapshot](apk/2.26.37.71-ntp-clock/targets.json) retain full signatures,
register flow and catches. Reproduce individual bodies with `apk-smali`, using
exact class descriptors and method selectors from the recipe. This establishes
additional source coverage, not an NTP implementation or live clock validation.

### Clock domains and persistence

`08B.A00` selects nonzero NTP baseline A03 first (`0008..0020`), then nonzero
WhatsApp baseline A04 (`0020..0032`), adding the same sampled elapsedRealtime.
Otherwise it returns wall time minus persisted WhatsApp difference A02.
`08B.A02` is a different getter: only NTP baseline plus elapsed time, or zero.
The token-job getter A04 does not select NTP. Thus a port must not silently
substitute A00 everywhere merely because both getters can return epoch time.

`08B.A01(offset)` samples current wall time, adds the supplied signed offset,
subtracts elapsedRealtime and writes A03 (`0012..0028`). It does not use A07's
positive-server-time gate and does not itself persist the offset. Sign is
**NTP minus device**, unlike A07's persisted **device minus WhatsApp server**.
Physical writes find A03 only here (`classes.dex` `0x45acb4`, opcode 5a).
Physical A01 callers are the already traced constructor (`0x45ac22`) and
`CN1.A00` (`classes7.dex` `0x58ef3c`). These are physical DEX inventories,
not reflection/native reachability closure.

Successful CN1 acquisition calls A01 and stores `client_ntp_time_diff` plus
current wall `last_ntp_client_time` in the same preferences editor
(`0488..04bc`). `1xl.A1H` finishes putLong/apply. On process construction,
08B's earlier positive saved time / absolute age below 86,400,000 ms check
reconstructs a new elapsed baseline from that offset. No getter freshness check
is present in selected A00/A02 bodies: the constructor's cache admission is
not a claim that an in-process baseline automatically expires after a day.

### Acquisition and admission

`NtpSyncWorker.A0A` delegates to `CN1.A00`. CN1 suppresses another acquisition
when positive process-static A06 is less than 21,600,000 elapsed milliseconds
old (`0022..0064`). That is a six-hour recent-sync guard, separate from the
one-day constructor cache check; scheduling/enqueue/backoff remain **unknown**.
A06's sole physical write is CN1 `classes7.dex` `0x58eff2`, opcode 68, on its
GM9 success branch (`052e..054c`). Early recent-sync return skips this write.
Renderer 61/68 are still unnamed; their field/register references and widths
are retained, and `apk-field-refs --writes-only` identifies the latter as write.

CN1 selects `android:string/config_ntpServer`, falling back to
`2.android.pool.ntp.org` for absent/empty/unresolvable configuration
(`0076..00d0`). It resolves through 19x.A02 and tries resolved addresses with a
DatagramSocket timeout of 20,000 ms, port 123 (`00d8..0136`). Cll allocates a
48-byte packet; the first request byte is composed as version 3/mode 3
(`013a..0160`). CN1 writes the transmit timestamp in big-endian bytes 40..47,
then sends/receives (`0186..0228`). It requires reply originate timestamp bytes
24..31 to equal the exact sent timestamp (`022a..0252`); mismatch throws an
IOException and follows the address retry/cleanup branch. This is correlation,
not cryptographic time authentication. Packet length, source-address, leap,
stratum and response mode validation are **not established** by the selected
CN1 receive path; do not infer them from generic NTP expectations.

Cll.A01 reads eight bytes big-endian into DLh.ntpTime. DLh.A00 extracts unsigned
seconds and fraction, rounds `fraction * 1000 / 2^32` via Math.round, then adds
the era base: -2,208,988,800,000 ms when the seconds high bit is set,
2,085,978,496,000 ms otherwise. CN1's request conversion uses the same era
boundary and integer fraction construction (`0186..01e4`). Complete behavior
outside this bounded era interpretation is not established.

CN1 reads reply originate/receive/transmit at offsets 24/32/40 and records
local destination receive wall time. With nonzero originate/receive/transmit,
the selected path computes signed offset
`((receive - originate) + (transmit - destination)) / 2` (`03d2..03ec`), using
Java long arithmetic/truncation. Warning strings about processing/network
ordering are accumulated; those branches do not independently reject offset
adoption. Zero receive or transmit has a separate one-sided fallback;
`6ak.A19(a,b)` returns boxed `a-b`. Null offset takes the explicit failure
branch (`04be..04d4`). These fallback branches remain visible in full source;
no unsupported synthetic sample is being adopted by Rust.

Before adoption, CN1 computes `ntpTimeMs = destination + offset`, comparing it
to both WhatsApp A04 time and current wall time (`0402..044e`). It rejects only
when **both** absolute differences exceed 86,400,000 ms **and** Boolean
property 2225 (`0x8b1`, 00D.A10) is true (`044e..0488`). Exactly one day does
not pass the strict-greater rejection comparisons. The effective property and
upstream scheduling remain **unknown**; the guard is not an unconditional
mandatory one-day bound. No new Rust NTP fetch, settings/flag, retry, clock
state or wire request was added by this continuation. Confirmed login/ping
clock behavior from the preceding runtime chunk is unchanged.

**NTP-chunk verification:** exact regeneration verified 14 files / 79,950
bytes; self-diff reports nine topics and four artifacts unchanged. Regenerated
CLI reference, API-symbol integration, diff whitespace and repository hygiene
passed (1,813 local links; 74 checksum bundles). This source-only continuation
changes no Rust; the preceding 820-test/Clippy/Rustdoc run belongs to the runtime
chunk, not a new full run here. Bulk class dumps and physical inventories stay
local; the scoped findings and reproduction inputs are retained above.
