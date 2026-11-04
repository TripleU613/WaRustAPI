# Current startup caller evidence and remaining boundaries

Retained 2026-11-04 from APK **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
This preserves the startup exploration preceding the
[bounded server-pong correction](../../server-pong.md) and complements the
[active/passive IQ builder](../2.26.43.72-active-iq-audit/README.md).
No new code change, APK execution, server request or live outcome is claimed.

## Confirmed static observations

- [0ZJ.A13](login-sequence.json) is the current login-sequence reader, located
  by the exact `ConnectionThread/lid-lifecycle/login-success isFirstLidLogin=`
  diagnostic. It distinguishes `web`, `success`, `failure` and unexpected nodes;
  a conditional expectation can require `web` before `success`. On success it
  reads the server `t` and a local-seconds sample, and contains conditional
  LID-lifecycle handling. The full producer of the web/passive/identity gates
  and complete post-success bootstrap contract were not closed by this trace.
- [0ZJ.A1P](active-mode-selection.json), anchored by
  `ConnectionThread/sendActiveModeIq`, invokes `0bq.A00`, then selects either
  writer command `1i8.A06(true)` when its input is true or
  [0ZJ.A0f](active-message-client.json) when false. The [command builder](active-writer-command.json) confirms: `Message.obtain(null, 0, 37, 0, Boolean.valueOf(z))`.
  The effects of the first helper and `0uu.A02` are not interpreted here.
- A0f obtains an ID through the message client and submits
  `277.A00(id,true)` with callback `2IT`. Callback success posts reader-handler
  message 8 with the captured native integer when the handler is present;
  error and delivery-failure callbacks log warnings. Exact builder/JID/callback
  source is retained in the separate active-IQ evidence linked above.
- [0uw.handleMessage](writer-ready-handler.json) on `what=0` logs
  `WriterThreadObserver/sending_channel_ready` and passes the supplied writer
  handle to [0ZJ.A15](writer-ready-consumer.json). A15 assigns that handle and
  invokes `A1B.BqT(new 1A3(this))`. Its `what=1` path invokes
  [0ZJ.A10](writer-closed-consumer.json), which calls A1R only when its supplied
  connection object is still the selected object. A1R's entire recovery policy
  is not established by that call-site condition.
- [1A3.handleMessage](message-service-handler.json) `what=0` invokes the native
  connect path with `available` and `forcePassiveMode` inputs. Its other shown
  cases route disconnect/network-block and forwarding events. These are
  internal scheduling inputs, not extra wire IQs to add to Rust.
  [1A3.A00](queued-connect-removal.json) removes a pending message 0 and invokes
  `BqF`; complete caller lifetime and scheduling are untraced.

## Provider limitations and negative results

REA's [A16 result](writer-ready-forwarder-rea.json) reports no decompiled body.
That **does not mean this method is actually native, abstract or absent**:
[checked DEX](writer-ready-forwarder-checked.txt) shows the concrete two-instruction
forwarder calling A15, then returning. Keep the raw method identity and the
provider's unavailable-body result distinct; do not infer a missing native
implementation from the display alone.

The [A0y result](connect-forwarder-rea.json) similarly reports no decompiled
body, but its checked DEX was not exported in this exploration. No conclusion
about its body or semantics follows from that display. No repeated decompile
attempt or guessed replacement was used.

These records do not establish that Rust's post-login active/presence/prekey/
device-query ordering is the complete current native bootstrap order. Conditional
pre-ack draining, effective remote configuration, reconnect-specific login flags,
keepalive cadence, full passive exit and the downstream BqT/A1S/A1R helpers remain
follow-up questions. A decoded success node or one matching IQ builder cannot
promote the entire startup path to parity. No causal link to account restrictions
is established.

## Reproduction and source identities

`diagnostic-xrefs.txt` is a complete exact-anchor `xtask apk-xref` result for
these five strings in this APK:

```text
ConnectionThread/sendActiveModeIq
ConnectionThread/lid-lifecycle/login-success isFirstLidLogin=
ConnectionThread/WriterThreadObserver/sending_channel_ready
ConnectionThread/sendActiveModeIqThroughMessageClient onDeliveryFailure
ConnectionThread/send preacks, size=
```

It also locates `0ZJ.A1V` for pre-ack sending; that body was not interpreted
in this exploration. Xref offsets are DEX byte offsets, not method-relative
coordinates. Reproduce the independently checked forwarder with:

```sh
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/0ZJ;' --method A16 --signatures --exceptions
```

| Checked output | SHA-256 |
| --- | --- |
| diagnostic-xrefs.txt | `ee33b1640396321620269c1d4b84ffe5b12f67fbdd0a3451bc2a0304fc87981d` |
| writer-ready-forwarder-checked.txt | `84212b86e0c35bdb0ae1047f9f145da3795d01e6ffc7799076e2a2b38ffd9d70` |

The JSON files preserve returned REA Evidence and limitations. The explicit
ownership sidecar identifies the existing Rust connection integration surface;
it is not a declaration that all retained native branches are implemented.
