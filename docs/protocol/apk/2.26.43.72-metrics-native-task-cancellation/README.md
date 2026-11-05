# Native metrics task cancellation and teardown — APK 2.26.43.72

**Confirmed static / derived; observed bounded synthetic cancellation.** Pinned
APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
No real network cancel, queue, account or Android process was executed. Local
Rizin/Unicorn outputs are not REA Evidence envelopes. No Rust behavior changed.

## Two direct callers, not a universal disconnect policy

Helper **0x727bd0** has two direct-BL call sites in executable image sections:
manager destructor0x727b58 and credential-completion raw2 branch0x729ef4.
The latter's log calls the signature-fetch error unretriable; see
[credential lifecycle](../2.26.43.72-metrics-native-credential-lifecycle/README.md).
This bounded census excludes indirect calls and does not prove every disconnect
uses this helper. The separately traced connection-status unavailable branch
signals callback+0x30; its complete caller/binding policy remains open.

## Cancellation ordering and local state

If manager network reference **+0x58** is nonnull, helper requires provider
**+0x48** nonnull. It snapshots active task dictionary **+0x60** keys/values.
Each value is classified by MCFGetTypeID against MCFNullGetTypeID:

- MCFNull placeholder: skips network cancellation.
- Other value: must be nonnull and contain backend+0x58. It passes backend
  fields+0x30/+0x58 to **MCINetworkSessionCancelTask**. Missing value/backend
  aborts. The selected helper does not inspect a cancellation result or wait
  for proof the network stopped.

Independently for each entry, if callback table manager+0x70 is nonnull, it
packages table and task key in MCIParams and queues thunk
**0xbd8f98 →0x727a10** on the utility execution queue with zero delay.
The deferred body unpacks those objects and invokes table function slot
**+0x28** with context+0x38 and that task key. This callback is not the ordinary
HTTP success slot+0x18. Full concrete table binding to higher-level outcome
handlers and ownership after queued work remain unproven.

After issuing cancels/queue submissions, the helper frees snapshot arrays,
removes all values from dictionary+0x60, releases/zeros provider+0x48 and
network+0x58, and clears busy byte+0xc0. It then asserts provider is null and
the active dictionary is empty. These operations occur after queue submission,
not after the queued callbacks or actual network cancellation complete.

If no network reference existed on entry, it skips that cleanup block and
only performs the final provider/dictionary assertions. In that path busy byte
is **not** cleared by this helper. A leftover provider or task aborts. This is
an invariant of this selected native implementation, not a Rust panic policy.

Endpoint dictionary **+0x68** is not cleared by the helper. Its pointer is
unchanged in the synthetic cases. Do not infer that endpoints remain usable
forever, that a task can be resent, or that destructor lifetime preserves them.
Their list construction/consumption is a [separate trace](../2.26.43.72-metrics-native-endpoints/README.md).

## Destructor suppresses this per-task callback path

Destructor **0x727b28** releases/zeros callback table+0x70 **before** calling
0x727bd0. Thus active network handles can receive cancel requests while the
helper does not enqueue per-task callbacks in this destructor path. It then
releases other owned fields, including task/endpoint dictionaries, current/spare
credentials, temporary credential objects, weak references, timer and product
string. Those releases are not deleting account/message history or proving
persistent buffers were removed.

Separately **0x72ae18** checks manager execution context, releases/zeros+0x70
and returns. It does not itself run this task-cancellation helper. Clearing
manager's pointer does not prove callbacks already retaining a table in queued
params cannot run. Reference-count, queued closure and generation fencing are
not fully reconstructed here; don't invent an exactly-once guarantee.

## Twelve bounded native cases

Cases cover absent/active empty state, placeholder-only/active/mixed entries,
no callback table, missing/orphan provider, orphan tasks, missing backend/null
value, and active-manager destruction. Native cancel helper instructions run
with controlled dictionaries, allocation/free, MCF type classification, params,
release, queue and network-cancel adapters. The destructor case additionally
runs its real native body with an explicit class/type adapter.

Fixtures assert exact synthetic backend cancel arguments and queued task keys,
active-map clearing, busy-byte branch behavior, provider/network clearing and
unchanged endpoint pointer. Destructor case checks callback release first and
no deferred per-task callback. Invalid state cases stop at native abort; they
are not claimed recoverable or leak-free. Queued callback execution, real cancel
results, concurrency and persistent buffer handling are **not executed**.

Run `python inspect-cancellation.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must
match cancellation-boundaries.json (pinned byte windows, branch targets and
direct-call census). Rizin windows.json uses `pD SIZE @ VA`. With Unicorn2.1.4,
`python check-cancellation.py RECONSTRUCTED_LIBWHATSAPP.so` must reproduce
cancellation-results.json. Each case has100000-instruction/one-second caps;
unexpected imports/interrupts or unsupported loader relocations fail closed.
Checksums cover all other files.

**Inferred:** local map removal, handle cancellation, callback completion and
durable buffer deletion must be separate stages in a future port. **Observed
live:** none. **Unknown:** full status/callback binding, stale completion fencing,
persistent retry/ownership, cancellation completion and deployment necessity.
No retry, re-send, identity reset or account-restriction guarantee follows.
