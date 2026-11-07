# MobileConfig parameter-map persistence scheduling

Follow-up: [default executor binding](../2.26.43.72-mobileconfig-inline-executor/README.md)
resolves the selected imported dispatch in pinned libwhatsappmerged.so to direct
callable invocation. Supplied executor overrides, runtime binding and persistence
outcomes remain separate; the original external-boundary unknown below is now
narrowed for this default construction chain.

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Static inspection only; no task, file write or live account/network execution.

## Follow-up identity narrows the remaining installation boundary

The [upgrade replacement](../2.26.43.72-mobileconfig-upgrade-replacement/README.md)
finishing path calls **0x8ada0 at 0x85c04**. This helper's diagnostic names it
`FBMobileConfigStorageManager::scheduleWritingLatestParamsMapToDisk`. It is
parameter-map persistence scheduling, **not established active-table installation**.
The earlier open question about this call must not be converted into an activation
claim merely because it follows successful table-file replacement.

It first requires manager member **+0x118** to be nonnull. On null it logs
`PersistParamsMapError` and
`Failed to persist params map because mcThreadPool_ is null`, then calls
**0x87f90** with value **3** on the supplied result object. That helper appends
one byte to its vector at +0x10/+0x18/+0x20; it does not replace the result's
first status byte. The guard precedes the later direct/scheduled choice.
The selected upgrade caller subsequently continues its own finishing path;
the append is not itself a false upgrade return or server refusal.

## Direct mode and captured-task mode

With a nonnull pool, the helper creates two paths through **0x8b030** and
**0x8b0b0**, using the current base, manager word+0x408 and byte+0x2a5.
Their filename builders 0xc5e0c/0xc5d00 select parameter-map names; retained
literals include `params_map_v4_` and named sessionless/u1/u0 forms. Full mode
and filename semantics remain unmodeled rather than inferred from the names.
It retains the RequestData shared pair at +0x98 and logger pair at +0xf8.

Manager byte **+0x230 exactly 1** takes the direct mode:

- if byte+0x212 bit 0 is clear, call file helper **0x8b130** for the first path;
- call that helper for the second path regardless of the first call's effects;
- supply `(argument1 XOR 1) & 1` as the helper's first flag.

Other +0x230 values allocate an **88-byte captured object** with the two path
strings, retained RequestData/logger pairs, flag +0x50 = argument1 & 1 and
flag +0x51 = manager byte+0x212. It installs invoke pointer **0x8b384** and
manager pointer **0x8b428**, then calls pool virtual slot **+0x10 at 0x8afcc**
with the function holder. After dispatch returns it destroys any remaining
local function target through the manager and releases local captures.
The callable's eventual ownership, ordering, thread and completion depend on
the receiver implementation; submission does not prove asynchronous execution,
completion, once-only invocation or persisted data.

The retained invoke routine mirrors the direct mode: skip the first path when
capture+0x51 bit 0 is set, always process the second, and use inverse low bit
of capture+0x50 as the file-helper flag. No result or completion callback is
propagated from these calls in this selected invoke body. Manager 0x8b428 has
separate object-management operations, including destruction through 0x8b484;
those operations are not calls to the invoke pointer.

## Pool origin: construction versus imported implementation

In the selected construction chain, storage object address is allocation+0x20.
At **0x776b8**, a pointer in x23 is stored at allocation+0x138, hence **object
+0x118**. It comes from the supplied executor pointer with a vtable-relative
base adjustment at 0x775c4–0x775d0. If the supplied shared pointer is null,
construction calls factory **0x770bc** and reuses its returned pointer.

That factory imports the vtable and VTT for
`facebook::mobile::xplat::executor::InlineSerialExecutor`. These are
**R_AARCH64_GLOB_DAT** relocations at **0xd6ba0/0xd6ba8**, not relative addends
to local implementation addresses. The inspector parses their linked symbol
names separately from local RELA targets. The imported executor's slot body,
virtual-base layout at runtime and supplied overrides are not resolved here;
the class name alone does not establish actual execution timing.

Do not confuse the adjacent logger: allocation+0x118 corresponds to storage
object+0xf8, not object+0x118. Its pointer comes from the separately constructed
FBMobileConfigLogger object at address point 0xd6500. This offset distinction
prevents binding the scheduling call to the logger's same-numbered slot.

## File helper is not a configuration-owner setter

Helper **0x8b130** works with a target path, RequestData and logger. Selected
branches check existing file/status and parameter-map content, compare basenames,
copy files through 0xc1f4c, or generate through 0xc044c. Existing source/content
that compares equal can skip writing on a selected flag path. The copy helper's
previously retained unchecked write/close results still apply.

The complete generation and filename-probe semantics remain untraced. A selected
false write result can log `writeToFile() returned false for: `; the helper has
no demonstrated status return used by 0x8ada0 or its captured invoke. This
follow-up therefore does not establish a successful active-configuration swap,
network readiness or complete on-disk parameter maps.

## Reproduction and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-params-scheduling/inspect-scheduling.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-scheduling.json
cmp /tmp/mc-scheduling.json docs/protocol/apk/2.26.43.72-mobileconfig-params-scheduling/scheduling.json
```

The inspector pins member/ELF identity, strings, selected calls and imported
executor symbol relocations. Its summaries are static transcriptions, not task
execution or persistence tests. LLVM retains the scheduling/callback/file helper
and bounded construction/filename/generation windows. Nearest JNI_OnLoad headings
are not local identities. Exceptions, imported dispatch and all helpers are not
fully modeled.

rea-scheduling.json retains complete REA Evidence
`ev_f4cc865e87caf49f7a6134e14bc338f509e664ff1f3b61a246e3ab1dddc6d762`,
command `e scr.color=0; pd 36 @ 0x8af94`, exit 0, untruncated output, unchanged
member hash and an entrypoint warning. This is a bounded function-holder handoff;
provider version/external inputs are not bound and annotations are not runtime
memory observations.

**Remaining:** runtime executor dispatch/overrides, generation and content-check
helpers, actual task completion and byte durability, other configuration-owner
installation paths and retry/account ownership. No Rust runtime or saved account
state changed.
