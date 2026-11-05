# Contact backoff read gates and reset callers

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution, live account traffic or invented REA Evidence envelope.
REA6.5.0 reports Android provider unavailable because its configured Java path
is missing. Existing xtask static DEX inspection supplies the exports here.
This continues the [observer/clock trace](../2.26.43.72-contact-protocol-backoff/README.md)
and [factory comparison](../2.26.43.72-contact-factory-selection/README.md).

## A scoped executor gate, then protocol filtering

0wB.run reads the request preset from0vt.A09 and captures requested protocol
flags into locals at0108..019e. Property0x92b7 (37559) is captured in v7 at
01de..01ec. The selected global gate at02a6..0328 compares the stored
`global_backoff_time` (absent default-1) with08q.A00 now. A deadline strictly
greater than now exits this contact executor dispatch path; equality passes.
This does not establish a process-wide network block or HxV query reachability.

At09e8..09f8,0vC.A01 selects FULL mode (0vE.A04). Within that branch,
requested contact flagv27 reads `full_sync_backoff` when property37559 is true,
otherwise `contact_sync_backoff` through the0e94 branch. Requested sidelist
flagv28 similarly reads `side_full_sync_backoff` or `sidelist_sync_backoff`
through0e8a. Absent defaults are-1. Each deadline>now clears its selected local
protocol flag; these instructions do not rename a contact to out. The executor
has separate queue-removal/completion handling outside these flag comparisons.
The property's effective account value remains unknown.

The remaining selected protocol gates similarly clear individual flags. The
all-false branch at0e36..0e72 declines dispatch with the log that no syncing
protocol should be requested due to error backoff. This is evidence against
implementing every per-protocol wait as a blanket rejection. The excerpt also
retains exceptions and control branches, including the DELTA-mode wait path;
it is not a complete executor reconstruction or exact wire oracle.

0vC.A00 is DELTA, A01 is FULL and A02 is FULL plus its context predicate.
0vC.<clinit> and0vE.<clinit> establish INTERACTIVE_QUERY and
INTERACTIVE_MULTI_PROTOCOL_QUERY use QUERY (A05), not FULL. Thus applying
these contact/sidelist FULL-branch gates to Rust's query solely from shared
protocol names would be incorrect. This is a bounded branch conclusion;
all possible query read gates/entry paths have not been proved absent.

## Reset is tied to selected native events

Exact direct-invoke census finds two callers of0pQ.A02 in this APK:

- Hbf.A02 starts RegistrationContactSync, invokes the reset before scheduling
  its contact-sync work, and catches an Exception from reset. Its two direct
  callers are RegisterNameManager.A02 and IC6.run; their full upstream
  registration authorization/state transitions remain untraced here.
- IBi.run selector16 reaches0f48..0fde. The constructor retains its supplied
  selector directly. This branch invokes0pQ.A02, then chooses a0ob.A01 contact
  sync request from its primary/companion predicate and associated data.

19V.A09 is ContactUpdateNotificationHandler. It selects the first child of
its input. A `sync` child at0600..0612 reads child `after` and outer `t` via
0HF.A01 with default0, converts both throughTimeUnit.SECONDS.toMillis, then
schedules IBi(this,16) via08Z.Cc2. The delay starts at0 and becomes
`after_ms - t_ms` only when after_ms>t_ms and t_ms>0. It does not subtract
local wall-clock now. This selected branch establishes event-bound scheduling,
not the full authenticated ingress/ACK authority or permission for a generic
retry command to clear waits. The0HF parser's full malformed-input semantics
and scheduler/restart persistence remain unknown; no Rust conversion is added.

The prior0pQ.A02 export resets its explicit list including contact, full,
side_full, sidelist, status, picture, business, devices, delta,
disappearing_mode, lid, reachability and global to-1. It does not mention
`text_status_sync_backoff` or `username_sync_backoff`, both read in the selected
executor. Therefore do not describe this reset as clearing every possible
protocol wait. The direct-call and literal-key censuses do not cover reflection,
aliases, dynamically generated preference keys or indirect lifecycle effects.

## Port obligations and verification

Runtime request eligibility is unchanged. The preceding Rust outer-refusal
metadata remains diagnostic only. Still required: choose the intended full
native query factory; establish its gate reachability/observer scope and
effective configuration; implement08q.A00-compatible clock authority; retain
protocol-result integrity/adoption policy; persist actual deadlines atomically
with owned replies and check them before reservation. An eventual notification
reset must be authorized by the actual ingress path and preserve once-only
processing. No historical replay, automatic retry or blanket wait reset follows
from this evidence.

Reproduce using existing xtask commands against the pinned APK:

```text
apk-callers APK --class LX/0pQ; --method A02
apk-callers APK --class LX/Hbf; --method A02
apk-xref APK --anchor global_backoff_time --anchor sidelist_sync_backoff --anchor contact_sync_backoff
apk-smali APK --class LX/0wB; --method run --signatures --exceptions
apk-smali APK --class LX/Hbf; --method A02 --signatures --exceptions
apk-smali APK --class LX/19V; --method A09 --signatures --exceptions
apk-smali APK --class LX/IBi; --method run --signatures --exceptions
apk-smali APK --class LX/IBi; --method <init> --signatures --exceptions
apk-smali APK --class LX/0vC; --signatures --exceptions
apk-smali APK --class LX/0vE; --signatures --exceptions
```

Quote shell descriptor/metacharacter arguments. Excerpts name source-export
hashes and exact byte-PC ranges; omitted branches are not inferred absent.
Checksums cover every other file. No raw APK or bulk decompiled tree is included.
