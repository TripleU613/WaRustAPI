# Recovered metrics buffer consumer — APK 2.26.43.72

**Confirmed static / bounded native checksum decisions.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
reconstructed ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction](../2.26.43.72-superpack-reconstructed-members/README.md).
No real file/queue, app/account or network operation was executed. Local
Rizin/Unicorn exports are not REA Evidence envelopes.

## Selected initialization schedules the outgoing loader conditionally

The retained initialization continuation **excerpt0x7238f0..0x7239b4** checks
**owner byte+0xf8 exactly1** before creating parameters with owner+0x90 storage
context and scheduling **0xbd8f30 →0x726660** at delay0 on that context's store
queue. The [loader entry](../2.26.43.72-metrics-buffer-recovery/README.md) calls
0x6fe1a8 with raw mode1, selecting the `wamt-o` prefix. The excerpt proves that
selected conditional scheduling site, not every startup invocation or the
producer/meaning of byte+0xf8. A following different load job is not conflated
with this outgoing-buffer job.

Construction **excerpt0x6f96b0..0x6f97cc** creates the storage context, sets its
store+0x18 and creates queue-bound callback objects. In particular function
**0x722ce0** plus owner weak context is stored as storage-context **+0x38**;
the context is then stored as owner+0x90. The
[shared callback-object layout](../2.26.43.72-metrics-buffer-deletion/README.md)
stores function+0x18, context+0x20 and queue+0x28. This binds the outgoing loader's
selected callback to the consumer below. Full alternate construction/lifetime
policy remains open.

## Consumer adds the integrity gate absent from the file reader

Complete consumer **0x722ce0..0x723008** receives dictionary, raw mode, error and
weak context. It resolves the owner and requires its global queue context.
Nonnull error takes a logging/status path; null dictionary logs/returns; an
empty dictionary exits. For a nonempty dictionary it snapshots keys and values
into allocated arrays and visits each descriptor/buffer pair.

For buffer **version byte+0x34 >=3**, it computes:

1. `adler32(0, null, 0)` to obtain the initial state;
2. `adler32(initial, payload.byte_ptr, payload.length)` over buffer payload+0x18;
3. exact32-bit comparison against buffer **+0x30**.

The upstream loader only admits versions1/2/3, so version3 is the reachable
checksummed version through that loader. Versions1 and2 skip this consumer's
checksum comparison. The earlier file-reader finding was correctly bounded:
it restores state but does not validate payload **in that body**. The callback
consumer supplies this additional version-specific validation.

On mismatch it logs, increments owner32-bit counter **+0xe8**, then reaches array
cleanup and exits the remaining dictionary walk. It does **not** advance to the
next entry on this branch. Dictionary snapshot order is not established, so a
bad entry can prevent processing later snapshot entries in this invocation.
This branch does not delete the file, requeue work or complete a transport task.
Whether another invocation retries those entries is outside this trace.

## A passing outgoing entry reaches transport handoff

After the checksum gate, raw mode0 additionally calls0x72503c with descriptor
fields/buffer; mode1 skips that extra state handoff. Other raw modes abort at
this selected branch. Both valid modes call **0x7250c4** for task handoff.

Complete0x7250c4..0x72522c checks owner transport-coordinator+0x58. If absent it
logs and returns. Otherwise it creates a task retaining the buffer, checks the
coordinator queue, and routes task channel0/1 through its regular table+0x40,
slot+0x28, or channel2/3 through private table+0x48, slot+0x28. Out-of-range
channel aborts. A missing regular table skips the call; missing private table
logs. A callback invocation is not proof of network transmission.

The [selected regular construction](../2.26.43.72-metrics-buffer-deletion/README.md)
binds that table's+0x28 to task-add0x7274f4. Thus, **conditional on the selected
initialization flag, live callbacks, valid file/header/checksum and available
regular transport**, a recovered outgoing regular buffer reaches task admission.
No previous terminal outcome input is checked in this consumer. The prior
terminal path's set removal does not by itself exclude a surviving file here.
This is a static conditional path, not observed restart replay or permission to
retry any other API after terminal refusal. Full scheduler/provider/Java gates
still determine whether an admitted task actually sends.

## Reproduction and scope

```sh
python inspect-consumer.py /path/to/libwhatsapp.so
python check-checksum.py /path/to/libwhatsapp.so
```

Compare stdout with `consumer-boundaries.json` and `checksum-results.json`.
The inspector hashes complete selected consumer/handoff bodies and explicitly
named construction/initialization excerpts, and independently decodes the loader
thunk and consumer-function ADR. Exports follow `windows.json` using Rizin
`pD SIZE @ VA`; full engine initialization is not retained or claimed here.

The checker requires Unicorn2.1.4 and executes **0x722e80** through the checksum
pass boundary0x722eec or mismatch0x722fc0. Twelve cases use three positive-length
payloads, each with versions1/2 and arbitrary wrong stored checksum, version3
matching checksum and version3 mismatch. MCFData pointer/length and Adler32 are
controlled adapters; the latter uses host Python zlib on the actual synthetic
bytes/seed passed by native instructions. This checks native argument setup,
version branch and comparison, not the APK's zlib implementation.

Each mismatch also executes0x722fc0..0x722fec with a controlled logging adapter,
asserting counter17→18 and one log call. The enclosing dictionary iteration,
weak resolution, callback creation, file loader, downstream task creation and
transport are **not** executed. The stop-after-mismatch and handoff chain are
static conclusions from the retained complete body, not end-to-end fixture proof.

**Observed live:** none. **Unknown:** initialization flag producer, complete
startup/lifetime/order, recovery after bad checksum, alternate cleanup policy,
actual transmission and durability. No Rust/live behavior or generic retry rule
was introduced.
