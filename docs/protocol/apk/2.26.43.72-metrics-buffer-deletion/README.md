# Metrics transport completion to buffer deletion — APK 2.26.43.72

**Confirmed static / bounded native deletion-worker execution.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
reconstructed ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction](../2.26.43.72-superpack-reconstructed-members/README.md).
No actual file was removed, no app/account/network was executed. Rizin/Unicorn
exports are local evidence, not REA Evidence envelopes.

## Selected construction binds the previously unknown owner callback

The retained engine-construction **excerpts**, not a complete constructor,
0x6f933c..0x6f9450 show callback-object constructor0x736578 receiving function
**0x7226d4** and an owner weak context. The callback is saved in engine+0x40,
then retained as the transport coordinator's **+0x50** completion callback.
The callback-object constructor stores function+0x18, context+0x20 and queue+0x28.

The regular construction excerpt0x6f957c..0x6f95c8 builds a four-function table:
setter0x727444, thunk0xbd8f78, task-add0x7274f4, thunk0xbd8f80. Constructor
0x72719c copies those function pointers into slots+0x18..+0x30 and retains the
regular manager as context+0x38. Factory0x727064 creates the owner-facing table
below with a weak coordinator context+0x38, then invokes the outer table's
setter. Setter0x727444 checks queue context, retains the new table at regular
manager+0x40 and releases the previous table. These exact selected writes bind
the regular task callback's table slots to:

| Slot | Function | Selected completion behavior |
| --- | --- | --- |
| +0x18 | 0x726bdc | resolve coordinator; if callback+0x50 exists, dispatch task with null error |
| +0x20 | 0x726c80 | resolve coordinator/check queue/release; no owner completion in this body |
| +0x28 | 0x726d14 | resolve coordinator; if callback+0x50 exists, create local error code2, dispatch task/error, release error |
| +0x30 | 0x726e18 | status callback; full semantics outside this chunk |

The task wrappers require live weak resolution and the expected execution
context. They do not delete files themselves. The regular task callback's
[success/retryable/non-retryable/failure outcome mapping](../2.26.43.72-metrics-native-transport-responses/README.md)
is distinct from these local error codes and HTTP status.

Dispatcher0x726964 reads the callback object's queue. If
MCIExecutionQueueGetExecutionContext returns0, it invokes the stored function
with **(task, error, stored context)** directly. Otherwise it creates typed
parameters and queues thunk0xbd8f48; body0x7269f8 unpacks and calls the same
function/arguments. Queue submission is not proof the callback ran. Full
lifetime/generation safety and all alternate construction paths remain open.

## Owner completion separates success from file deletion

Owner function **0x7226d4..0x72282c** requires a nonnull weak context, resolves
the owner and checks its global queue context. A **nonnull error** takes the
logging/cleanup branch and **does not schedule the deletion worker**. A null
error derives a buffer descriptor from the task, creates parameters containing
owner+0x90 storage context and that descriptor, and schedules
**0xbd8f28 →0x726580** on the storage execution queue with delay0.

Consequently, in this bound regular path:

- Success0 invokes table+0x18, which can eventually schedule file removal.
- Retryable1/failure3 invoke table+0x20, which does not complete to the owner.
- Non-retryable2 removes the task from the regular set and invokes table+0x28;
  its local error2 reaches the owner error branch, which skips this deletion.

Thus removal from the regular set is **not** equivalent to durable file
cleanup, including for the selected terminal outcome. Whether another cleanup
path removes those files or a later recovery scan admits/excludes them is
untraced; no permanent leak or automatic replay claim is made. This finding
must not be translated into a generic Rust retry after terminal refusal.

## Actual deletion is a separate fallible worker

Complete worker **0x726580..0x726660** unpacks storage owner and descriptor,
loads its store and completion callback, and requires the store's global queue
context. It calls path-builder0x6fe0d0 with root=store+0x28, the descriptor and
raw argument1. The builder's exact format is **`%s/%s-%d-%d-%d`**. Raw argument0
selects `wamt-a`; nonzero selects **`wamt-o`**, so this worker targets the latter.
The numeric fields are descriptor+0x1c, signed byte+0x20, then descriptor+0x18.
Their wider producer/domain constraints are not fully established here.

The worker converts the path object to a C string and calls **remove() once**:

| remove result | Selected completion |
| --- | --- |
| 0 | invoke0x71f248(completion, descriptor, null error) |
| nonzero | read errno; create MCFError(domain0xc30098, errno, null info); invoke0x71f248(completion, descriptor, error); release error |

It releases the path in both cases. ENOENT is not special-cased as success in
this body. There is no local retry loop, requeue, task-set mutation or fsync in
this worker. A successful syscall is not proof of crash-durable directory
metadata, and the downstream completion's storage bookkeeping remains open.
Transport success, deletion scheduling, remove success and durable recovery
must remain separate claims.

## Reproduction and limits

```sh
python inspect-deletion.py /path/to/libwhatsapp.so
python check-deletion.py /path/to/libwhatsapp.so
```

Compare stdout with `deletion-boundaries.json` and `deletion-results.json`.
The inspector hashes all retained windows, resolves three path literals and
independently decodes the deletion thunk. Full selected functions and explicitly
named construction excerpts are exported using `windows.json` and Rizin
`pD SIZE @ VA`. They are not complete engine/static program coverage.

The checker requires Unicorn2.1.4, maps pinned segments/relocations and executes
the complete selected worker, with controlled type/parameter/context/path,
remove/errno/error and completion adapters. Four cases cover successful remove,
ENOENT, EACCES and wrong-context abort. They assert the exact path-builder
arguments, single remove, error domain/code, completion error and release order.
No actual path formatting, file operation, callback execution, object destructor,
queue, concurrent completion or persistence is performed. All upstream chain
bindings above are static conclusions, not exercised end to end by these cases.

**Inferred:** this selected null-error completion is the regular success path
into file removal. **Observed live:** none. **Unknown:** alternate constructor
branches, full registration/lifetimes, downstream delete bookkeeping, failed
remove recovery, terminal-file scanning/cleanup and server restriction effects.
No Rust sender/retry behavior or live operation was added.
