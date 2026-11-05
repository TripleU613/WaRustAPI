# Native metrics transport responses — APK 2.26.43.72

**Confirmed static / derived branch flow**, reconstructed ARM64 libwhatsapp.so
SHA256 `be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
Pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Image reconstruction checks](../2.26.43.72-superpack-reconstructed-members/README.md)
bind these bytes. No requests, callback execution, account or live traffic was
performed. Local Rizin exports are not REA Evidence envelopes.

## Concrete request and response boundary

Buffer sender **0x7280dc** calls MCIURLRequestCreateWithHTTPBodyNoCopy, then
bridge **0x7345a4** with completion adapter **0x728c54**. The selected sender and
bridge bodies are retained, but endpoint/credential construction and every
header/body field have not yet been semantically reconstructed. A request object
and task bridge do not themselves prove successful connection or transmission.

The completion adapter unwraps a weak manager reference and schedules another
callback on its execution queue using thunk **0xbd8fa8**. That code thunk
branches to **0x728e18**; the inspector independently decodes the branch.
The worker unpacks typed manager/task, MCIURLResponse, MCFError, response-data
and timing objects. It requires manager queue context, clears local byte +0xc0,
then calls **MCIURLResponseGetStatusCode**. Full callback generation/ownership
fencing above this entry remains untraced; clearing that byte is not delivery.

## Preserve failure-branch ordering

The following order matters in the selected response worker:

1. Before ordinary status classification, if manager flag +0x40 bit zero is
   clear and the MCFError object is nonnull, it checks for another endpoint via
   0x729410. That helper obtains the task's array from dictionary +0x68 and
   returns true only for a present array with positive count. On this branch it
   calls the response-body parser 0x72943c, logs `try next endpoint`, invokes
   buffer sender 0x7280dc again, and exits this selected completion path.
2. The ordinary success predicate is **status 200 AND no MCFError object**.
   Success removes the task from dictionaries +0x60 and +0x68, invokes optional
   callback-table slot +0x18, logs success and calls scheduler 0x727d40.
3. If that predicate fails, a distinct **401** branch removes both dictionary
   entries, invokes optional callback slot +0x28 and calls 0x728050 with raw
   argument one. It logs `401, no more endpoints`. A bare 401 is therefore not
   the ordinary success branch, but the earlier error/endpoint branch can take
   precedence; do not flatten this into an unconditional rule for every 401.
4. Other failures follow manager-flag-specific branches. A selected regular
   branch parses response data through 0x72943c, invokes callback slot +0x20 and
   logs `failed temporarily`. A different branch parses failure-reason text and
   may try another endpoint or clear/renew a credential slot. Complete parser
   schema, credential lifetime and retry timing remain open.

These are native metrics transport rules. They do **not** authorize extending
Rust registration/login/message retries to HTTP 401, rotating account identity,
ignoring cooldowns or treating every API's HTTP 200 as application success.
No such Rust behavior changed. No server response was observed in this work.

The worker also produces an internal diagnostic event after selected outcomes.
Its numeric result fields are not identical to the regular task enum below and
are not server restriction codes. Complete diagnostic schema, persistence and
whether/how those events are later sent remain outside this trace.

## Regular task ownership callback has explicit outcome names

Related callback **0x7275d0**, named `_WAMTransportRegularSendTaskCallback` in
its diagnostic string, reads a four-entry relocated label table:

| Raw outcome | Exact native label | Selected local action |
| --- | --- | --- |
| 0 | success | callback slot +0x18; remove task from set +0x48; schedule 0x727294 |
| 1 | retryable | nonzero-outcome helper; optional callback slot +0x20 |
| 2 | non-retryable | nonzero-outcome helper; remove from set; optional callback slot +0x28 |
| 3 | failure | nonzero-outcome helper; optional callback slot +0x20 |

The nonzero helper0x7273a8 is now traced as a one-shot regular-drain status
notifier, not backoff or persistent task mutation; see the
[drain outcome trace](../2.26.43.72-metrics-native-drain-outcomes/README.md). The callback unwraps a weak reference, checks set membership and
can diagnose missing membership before continuing. Whether surrounding layers
exclude obsolete completions still requires proof. The callback-table bindings
between all transports and these interfaces remain incomplete; do not equate
this enum with HTTP status or the response worker's diagnostic result numbers.
A local removal callback is not a proved durable file deletion, user-message
acknowledgement or completed application operation.

## Reproduction and next work

Run `python inspect-transport.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
transport-boundaries.json. It reuses the pinned-image resolver, hashes complete
selected bodies, resolves the exact outcome labels and verifies the async thunk.
Rizin reproduction uses `pD SIZE @ VA` from those windows. [Section map](../2.26.43.72-metrics-native-event-routing/elf-sections.txt)
and [tool version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt) apply.
The sender, adapter, worker, endpoint predicate, regular response parser, bridge
and regular callback are retained as complete selected bodies, not declared
complete semantic coverage. Diagnostic Strings are kept with exact addresses.

Next trace request method/URL/header/body sources, callback-table bindings,
endpoint-list consumption, retry/backoff and actual buffer deletion/recovery.
No Rust runtime, HTTP route, sender, telemetry or live account operation added.
