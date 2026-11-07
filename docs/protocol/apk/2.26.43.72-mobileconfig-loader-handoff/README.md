# MobileConfig caller diagnostics and configuration state handoff

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution, live request or account state read.

## The finalizer's callers retain diagnostics and continue

The preceding [record finalizer](../2.26.43.72-mobileconfig-record-finalization/README.md)
has direct calls at 0x5a284 and 0x5a544 in the selected enclosing native region.
Before each, the caller tests loader byte +0x50. A clear low bit adds
native_error__parse_main_params_map to its diagnostic accumulator through
0x5c6e8; it then joins the same finalizer call as the set-bit path. This is not
an early return on that individual diagnostic. Earlier package-map loops also
carry native_error__parse_package_params_maps. Their complete aggregation and
external error reporting remain untraced.

After finalization, the caller invokes shared-record accessor 0xbfc6c and retains
its result in a local pair at stack+0x1c8. It also selects a string from the loader
or its child and retains another shared pair through 0x5c900. Their complete
semantic roles are not established here. One selected path requires a nonnull
loader and byte +0x50 equal to 1 before additional loads; later it rechecks the
low bit for diagnostics. Preserve those different checks rather than treating all
loader flag use as one global boolean admission rule.

The subsequent selected check distinguishes null and empty record vectors:
empty records add the diagnostic Params List is empty, typically means we failed
to parse params map file; null records add No valid schema hash and param list
provided and invoke reportSoftError. Both join later normal continuation at
0x5a600 in the retained region. This does not prove that all outer callers accept
such state or that no exception/logging path can interrupt construction.

## Construction retains the vector and seeds deferred work

At 0x5a6cc the local shared record pair is passed as constructor x1 to 0xa9430.
That constructor preserves x1 as x22. It allocates an 88-byte state object,
copies the supplied pair into its first two words, and increments the shared
control count when nonnull. It stores the new state pointer at object+0x258.
This links selected finalization/accessor output directly to the source getter,
[grouping](../2.26.43.72-mobileconfig-schema-grouping/README.md) and
[selection](../2.26.43.72-mobileconfig-schema-selection/README.md) state used later
for request construction. It is initialization evidence, not a file commit or
proof that configuration values have been activated.

The constructor sets state bytes +0x40 through +0x46 to 1 using overlapping
32-bit stores at +0x40/+0x43. It zeros selected range/index/cache pointers. Those
initial bytes connect to the previously traced deferred getters; a retained
source pair is not proof their derived tables have already been built.
Constructor argument w4 is masked to its low bit before object byte +0x194 is
stored. A halfword store of 1 at object+0x208 establishes byte +0x208=1 and
+0x209=0 on this pinned little-endian path. Thus their **initial** values are
canonical, narrowing the previous raw-byte uncertainty at construction. Later
writers and reuse can still change them, so this is not a lifetime invariant.

A selected late constructor branch computes object+0x198 from the last record's
word +0x34 plus one when a supplied low-bit flag enables that behavior and the
input vector is nonempty; otherwise it uses another supplied/default value.
Full flag meanings and bounds are unresolved. A completion marker, soft error,
shared pointer or constructor return does not independently prove account
ownership, server acceptance, persistence, crash recovery or race safety.

## Reproduction and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-loader-handoff/inspect-handoff.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-loader-handoff.json
cmp /tmp/mc-loader-handoff.json docs/protocol/apk/2.26.43.72-mobileconfig-loader-handoff/handoff.json
```

The inspector reuses pinned ELF validation and checks selected BLs and diagnostics.
LLVM retains bounded caller regions, the selected constructor normal path and
outlined setup/vector helpers. The enclosing-function setup neighborhood is a
bounded discovery read; its full identity and branches are not claimed. Nearest
JNI_OnLoad headings do not identify these local functions. The constructor's
later exception paths are outside the retained normal-path window.

rea-loader-handoff.json retains complete REA Evidence
`ev_028d4877643fe9f0bf160a667752b997e7c87ddc73bbaa28ae58f227baeb524a`:
`e scr.color=0; pd 48 @ 0x5a51c`, exit 0, untruncated output, unchanged member hash,
with an entrypoint warning. This is a bounded handoff window; LLVM extends through
constructor invocation and seeding. Register-relative annotations are not runtime
memory.

**Remaining:** full caller admission/error propagation, path and flag producers,
later configuration writes, complete constructor/exception contract, account
ownership, selected live request derivation and response persistence/cache
activation. No Rust runtime or saved account state changed.

Type refinement: [RequestData RTTI binding](../2.26.43.72-mobileconfig-request-data-type/README.md)
identifies 0xa9430 as constructing FBMobileConfigRequestData. The generic
configuration-state wording above denotes that request-data object and does not
identify the separate response-processing receiver or prove configuration adoption.
