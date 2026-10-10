# Encrypt identity notifications

APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
All offsets below are method-relative bytes. **Confirmed static** means the
retained annotated DEX establishes the selected path; it is not execution on
Android or a live server observation. Other releases need separate anchoring.

## Notification selection and metadata

The [14z.A09 handler](apk/2.26.37.71-prekey-count-notifications/count-notification-handler.txt)
selects child zero, then tests `identity` at `00d0..00e0`. It resolves root
`from` as UserJid (`00e0..00f4`) and rejoins device work at `0188` when null
(`00f4`). Thus absent metadata peer does **not** establish that
all identity work is skipped. This distinction follows relative-branch arithmetic:
`00f4 + 0x4a * 2 = 0188`.

For a nonnull metadata peer it reads root `type` (default empty), optional `lid`
as UserJid and a nonblank `display_name`; passes these to
[32q.A00](apk/2.26.37.71-mex/mex-participant-alias-builder.txt), then calls
[17H.A04](apk/2.26.37.71-identity-notifications/identity-peer-metadata.txt)
with the singleton metadata map and **false** (`0150..0188`). 32q classifies
typed PN/LID metadata; 17H gathers mapping/display-name inputs, delegates a
map to `0Xw.A0P`, and schedules `3QH` selector 2. **Unknown:** those downstream
storage/observer effects and their authority across identity aliases. This is
not a demonstrated replacement of a Signal pin.

At `0188..0198`, even after the null-metadata-peer rejoin, the handler obtains
a DeviceJid through `0DK.A00` from base-handler `0yO.A01`. The
[base handler](apk/2.26.37.71-identity-notifications/identity-handler-ack.txt)
sets that field from `Cb1.A02` (`0146..014e`) before dispatching `A09`
(`0206..020c`). **Unknown:** the complete outer Cb1 producer/address
normalization and participant/recipient routing. Do not assume the metadata
`from` parser selects the exact same device as this callback field.

The handler queues `DLp` selector **5** (`01b8..01ca`) on `16Z.A00`.
[Constructor](apk/2.26.37.71-identity-notifications/identity-worker-constructor.txt)
assigns `$t` directly, A00=handler and A01=device. The
[run switch](apk/2.26.37.71-identity-notifications/identity-worker-run.txt)
selects `01fa` for 5. No identity-child body/key is consumed in this handler
path. That is evidence about this consumer, not proof that arbitrary identity
notifications never contain other payload.

## Existing identity gate and prekey job

DLp.run `0214..023a` converts the callback DeviceJid with `B7H.A04` and acquires
its `0jp` lock. `0246..0252` calls
[0jc.A0W](apk/2.26.37.71-identity-notifications/identity-stored-key.txt)
and branches to lock closure at `02e4` when null:
`024e + 0x4b * 2 = 02e4`. A0W reads an existing serialized identity via
`0kc.A0J` and decodes it as B7D; missing bytes return null. It also has
normalization and invalid-key cleanup paths. **Unknown:** complete recipient
remapping and A0H cleanup effects; this does not authorize deleting our saved
pins or sessions when a lookup fails.

With an existing identity, `0266..027c` creates a one-device array and invokes
`14L.A09(devices, 8, true)`. The call chain is retained:

| Method | Selected forwarding |
| --- | --- |
| [14L.A09](apk/2.26.37.71-identity-notifications/identity-fetch-entry.txt) | A08 with null context, reason 8, remaining integer inputs zero, first boolean true and second false |
| [14L.A08](apk/2.26.37.71-identity-notifications/identity-fetch-wrapper.txt) | A07, adding null B78 |
| [14L.A07](apk/2.26.37.71-identity-notifications/identity-fetch-core-wrapper.txt) | A00, adding final false |
| [14L.A00](apk/2.26.37.71-identity-notifications/identity-fetch-dispatch.txt) | filters/devices/in-flight bookkeeping, then A06 (`0280..02a6`) |
| [14L.A06](apk/2.26.37.71-identity-notifications/identity-fetch-job.txt) | filtered device lists, BulkGetPreKeyJob construction and queue (`01c0..01e4` for an ordinary small list) |

A00's own log names the first boolean `fromIdentityChange`, the last boolean
`isPqSessionUpgrade`. The middle selects B77 and reaches A06's
`fetchPqPrekey` log/parameter. A00 removes expired in-flight entries only when
`stored_uptime + 60000 < current_uptime` (`00da..00f4`). Non-upgrade requests
skip device keys already in that map (`0236..0242`). This is native transient
bookkeeping, **not** permission to expire/replay our durable outcome-unknown
actions or ignore server waits. TEE filtering, configured PN normalization,
B78/B77 key selection and callback/response semantics remain **unported**.

A06 filters devices again, returns for empty lists, and enqueues the named
bulk prekey job. Large-list splitting uses separate size/config guards. This
trace ends at job construction: **unknown** wire request, attribute use,
response verification/adoption and failure/retry policy for this exact job.
The Rust `peer-keys` full-bundle inspection and explicit `identity-query`
commands do not automatically implement this job.

After scheduling the legacy request, DLp also checks `0Fs.A0U(user)` and
`Can.A04(user)` (`027c..02b4`). Only both true reach a second A08 call with
the middle boolean true (`02b4..02d6`): PQ fetching, not unconditional key
replacement. Lock closes at `02e4`. **Unknown:** predicate semantics and
complete PQ response strategy.

## Acknowledgement and Rust boundary

0yO.A01 dispatches synchronously to A09, then takes `020c` to `0296` and
acknowledges through A00 only if its retained notification key is absent from
A0A (`02a4..02c6`). A deferred key in that map skips to return `0352`.
**Unknown:** A0A insertion/removal lifetimes and identity-specific deferred ACK
authority. Neither scheduling a worker nor receiving the notification proves
that refresh completed or that the user identity actually changed.

**Confirmed local gap:** the Rust actor durably captures these controls, but
its supported handlers cover privacy tokens, dirty groups, device lists and
prekey counts. It does not dispatch this identity worker, migrate metadata,
queue this native prekey refresh or apply its ACK lifecycle. Current successful
message/bundle authentication and explicit identity-change policies continue to
control pin adoption. This research changes no account, key, reservation or
runtime behavior. No fresh live notification was requested or inspected here.

The next implementation prerequisite is the BulkGetPreKeyJob wire/response path,
then exact callback-device resolution and once-only refresh/ACK state. Preserve
own-account protections, lock-equivalent transactions, existing pin authentication
and durable reservations rather than treating the first `identity` tag as trust.

## Reproduce and compare

The [ten-topic recipe](apk/recipes/2.26.37.71-identity-notifications.json) retains
15 files / 141,669 bytes with signatures, register flow and exception tables.
A09 and 32q sources are reused
from existing bundles. Bulk APK/decompiled trees remain local.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-identity-notifications.json \
  --output docs/protocol/apk/2.26.37.71-identity-notifications --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-identity-notifications \
  docs/protocol/apk/2.26.37.71-identity-notifications
```

Cached self-diff checks retention; it is not a comparison against a new release.
Use separate release recipes and exact-string remapping when names change.
