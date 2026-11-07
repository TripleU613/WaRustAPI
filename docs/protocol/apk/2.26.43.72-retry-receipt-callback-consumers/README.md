# Retry receipt callback completion and failure consumers

**Confirmed selected static DEX**, WhatsApp2.26.43.72, code264307238,
APK SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `5f4f258d`. Offline repository DEX inspector; no target/account execution,
runtime policy change or live traffic. REA's previously recorded Java-path failure
is not a successful analysis result. This follows the [registration bundle](../2.26.43.72-retry-receipt-callback-handoff/README.md).

## Physical-reference coverage

[Exact field scan](map-refs.txt) finds eight direct DEX references to
0Th.A04:Map: constructor, A01/A02 registration,0Sf.A0T,0ln.A0B,
two0m2.handleMessage sites and1rL.A03. This is exhausted direct field-reference
coverage for that exact descriptor, not reflection/dynamic/native alias coverage.
Selected consumers below account for every returned read site. The separate
A05 bulk-query map is not substituted for A04.

## ACK completion

[Caller scan](ack-post-callers.txt) finds1lY.Big and1lY.handleMessage call0ln.A09.
[Big](ack-listener.txt) directly obtains the router and forwards node plus Eoh.
[0ln.A09](ack-post.txt) wraps both in2E3 and posts handler message6.
[0m2.handleMessage](handler.txt) switches message6 to000e. At004c..005e it removes
the exact Eoh key from A04 under the map monitor. For a found callback it exits
the monitor and calls BsT with the original node at0132..013c.

An error attribute is not a BsR switch in that matched path. The extra fallback
search at0062..012c is guarded by class=message, absent edit selector and a
present error; it is not a generic receipt callback match by ID. It seeks a
unique edited-message key after stripping that selector. Do not apply it to
retry receipts or weaken Rust receipt route/member correlation from this branch.

The selected retry wrapper mode0 [forwards null](../2.26.43.72-retry-receipt-callback-handoff/callback-wrapper.txt),
and its FfK releases the job's untimed get normally. Thus a matched callback can
complete the retry job even for an error-bearing ACK. Normal job return proves
callback completion, not server acceptance, usable keys or permission to resend.
The upstream reader's complete parsing/challenge dispatch and effective provider
binding remain separate; not every arbitrary input node is admitted here.

## Disconnect/fail-all path

[0ln.A0F](fail-all-post.txt) posts message3 after checking its queued request maps
and a supplied boolean. The force/queue condition must pass; it also removes
messages4/8 and clears its timer bookkeeping. [Two callers](fail-all-callers.txt)
are0YB.A05 and0YU.handleMessage; their complete event classification is not newly
traced here. This is not a fixed response timeout established by the job itself.

Handler message3 enters055a. It drains other request bookkeeping, snapshots all
A04 callback values into an ArrayList at060c..061e, gathers message-class telemetry,
and clears A04 under the monitor at06ac..06b4. It then constructs E5V and calls
each retained callback's BsR at071c..074e on the normal path. [E5V](fail-all-exception.txt)
extends IOException with message `disconnected`. The wrapper forwards that same
exception; FfK.get wraps failure in ExecutionException. Map removal and callback
failure are distinct operations; interruption or callback exceptions are not
proven atomic/durable by this Java branch. No native crash recovery is inferred.

## Other exact reads and boundaries

[0Sf.A0T](client-event.txt) normalizes the supplied Eoh with0Sf.A02, removes its
A04 callback under the map monitor, and forwards a supplied exception via BsR
outside it. The supplied exception's producer/classification is not resolved here.
[0ln.A0B](router-event.txt) likewise removes an exact key and forwards its supplied
exception; its local log names received-message-too-large. No fresh receipt or
retry is constructed in these selected methods.

[1rL.A03](status-error-consumer.txt) is an **error stanza/status** consumer, not
the ordinary receipt ACK reader originally sought. Its selected A04 scan requires
class=status plus matching stanza ID, removes one callback and calls BsR with an
IOException. It does not establish retry receipt failure by ID alone. Retaining
this negative classification prevents widening its status-only rule to receipts.

Rust's existing server_acknowledged/refused outcomes, immutable callbacks and
uncertain-write no-replay rules remain unchanged. Full native job retry decisions,
all event producers, scheduling, key lifecycle and restart behavior remain open.
No automatic resend follows from callback failure or untimed get.

## Reproduction

```
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Th;' --field A04 --type 'Ljava/util/Map;'
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0m2;' --method handleMessage --signatures --exceptions
```

Use exported descriptors for the other selectors. Full selected methods retain
branch/exception context; no bulk unpacked tree is committed. Printed instruction
coordinates are inspector bytes, while exception comments use code units.
