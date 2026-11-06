# Deferred ACK watchdog is diagnostic

**Confirmed static**, WhatsApp2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Complete8bZ.run](watchdog-run.txt) exported with xtask apk-smali APK
--class LX/8bZ; --method run --signatures --exceptions. DEX text, not a REA
Evidence ID. Only selector41 is audited; unrelated selectors are not attributed
to the notification handler.

## Schedule and fire

Existing [0v6.A07](../2.26.43.72-prekey-upload-replies/prekey-ack-handler-metadata.txt)
true mode consumes2DT, stores Eoh->IncomingStanza in A0A under its monitor,
removes the previous A02 Runnable through08Z.CYL when present, and schedules
8bZ(handler,41) through08Z.Cc2 with6000ms. It assigns the returned Runnable to
A02. This is a shared reset-on-deferral schedule, not an individual six-second
deadline for each Eoh and not the upload IQ response deadline.

8bZ.run selector41 enters090c. Under the A0A monitor it sets handler.A02=null
and returns immediately if the map is empty. Otherwise it builds diagnostics
from handler type, pending size, each Eoh and optional A0B metadata. It logs
failure-to-ACK messages, clears A0A and A0B at0a0c..0a1c, leaves the monitor,
then emits the aggregate failure diagnostic. There is no0v6.A00/89Y dispatch,
queue upload call, network write, worker interruption, or retained-token
mutation in this complete selected branch. Other concurrent actors and scheduler
implementation/lifecycle are outside this static branch claim.

## A later worker can still ACK

Existing [89Y.A00](../2.26.43.72-prekey-count-ack-source/deferred-ack-dispatch.txt)
takes and clears its own7p2 token under its own monitor. For a nonnull token,
it removes the original Eoh from handler.A0A at0020 but ignores the returned
value. Its subsequent original-IncomingStanza/null-override dispatch at003e
is not conditioned on map membership. It then removes the A0B record.
Therefore watchdog map clearing alone neither consumes the metadata object's
token nor prevents a later call from reaching ACK admission. The one-use token
still prevents a second dispatch by that same metadata object. This is not
permanent redelivery deduplication or proof that every worker eventually calls.

The [count worker](../2.26.43.72-prekey-upload-replies/count-worker-routing.txt)
requests89Y.A01 after its upload-work decision. The watchdog does not provide
an alternative successful ACK when that worker stalls. Native socket order,
count-specific tracked/untracked routing, effective queue configuration and
crash/reconnect recovery remain separate open obligations.

## Rust consequence

No Rust timeout, automatic ACK, cancellation or key replay was added. This
closes the old unknown about the scheduled Runnable's action. A future
faithful port must keep diagnostics, work ownership, metadata-token ownership,
send admission and server confirmation separate; expiring the tracking map
must not itself authorize transmission or invalidate the held token.
Static/offline only; no live account operation.
