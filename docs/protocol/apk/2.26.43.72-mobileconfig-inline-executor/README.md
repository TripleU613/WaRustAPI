# MobileConfig default executor resolves to direct invocation

**Confirmed selected static cross-member evidence**, APK 2.26.43.72 code
264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.

| Reconstructed arm64 member | Bytes | SHA256 |
| --- | --- | --- |
| libmobileconfig-jni.so | 852440 | `5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c` |
| libwhatsappmerged.so | 7871544 | `d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d` |

Both match the independently retained reconstructed-member validation. No library,
executor, task, filesystem write or network/account operation was executed.

## Imported boundary is present in another pinned member

The [parameter-map scheduling trace](../2.26.43.72-mobileconfig-params-scheduling/README.md)
resolved default factory 0x770bc to imports of InlineSerialExecutor's vtable/VTT.
libmobileconfig-jni.so declares libwhatsappmerged.so as DT_NEEDED. A bounded
dynamic-symbol search found the definitions in **libwhatsappmerged.so**:

- `_ZTVN8facebook6mobile5xplat8executor20InlineSerialExecutorE` at **0x757780**,
  size 136;
- `_ZTTN8facebook6mobile5xplat8executor20InlineSerialExecutorE` at **0x757808**,
  size 64.

The same symbol-name search found neither definition in selected libar-bundle3.so,
libc++_shared.so or libfb.so. This is not a complete process/loader search or a
claim about every library. Dynamic interposition and runtime symbol resolution
remain separate from the selected static dependency binding.

## Default address-point and call-slot binding

The factory's final vptr uses imported vtable base+**0x50**, giving address
point **0x7577d0**. The VTT's first relative relocation at 0x757808 also points
to 0x7577d0. The constructor obtains a base adjustment through vptr−0x48;
the pinned word at **0x757788** is zero with no relative relocation there.
Thus the selected default pointer adjustment is zero. The earlier allocation
and object offsets still matter: the factory returns allocation+0x18, while
the storage manager stores the adjusted pointer at its object+0x118.

At the address point, virtual slot+**0x10** is location **0x7577e0**. Its
R_AARCH64_RELATIVE addend resolves to **0x6679f0**. Raw file words for these
relocated pointer slots are zero; reading those zeros as function pointers
would miss the binding.

The entire selected method is four instructions:

```
bti c
ldr x16, [x1, #0x30]
mov x0, x1
br x16
```

It tail-invokes the function stored at holder+0x30, passing the holder itself.
There is **no queue, thread creation, deferred timer or scheduling loop in this
method**. In the MobileConfig call site at 0x8afcc, the holder's +0x30 entry is
**0x8b384**, previously traced to the parameter-map persistence task. The normal
selected default dispatch therefore executes that invoke directly before control
returns to the caller's post-dispatch cleanup. It does not establish file success
or durability: the task's file helper has no persistence result propagated by
that invoke body, and underlying copy/generation checks remain separate.

This conclusion is conditional on the constructed default executor and selected
dependency binding. It does not apply automatically to a supplied executor
override or prove once-only invocation across callers/retries. Exceptions and
runtime lifetime remain separate from normal-return control flow.

## Why this matters for the port

The mode that packages a captured task must not be assumed to defer parameter-map
work merely because its diagnostic says scheduling or its member says thread
pool. Under the selected default, local task side effects can occur during the
dispatch call. Conversely, completion of that call still cannot be promoted to
successful parameter-map persistence or active-table installation. This evidence
narrows timing; it does not introduce a Rust executor or new transport behavior.

## Reproduction and retained evidence

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-inline-executor/inspect-executor.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so ../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so > /tmp/mc-executor.json
cmp /tmp/mc-executor.json docs/protocol/apk/2.26.43.72-mobileconfig-inline-executor/executor.json
```

The inspector validates both member identities, ELF mappings, exported symbol
values/sizes, selected RELA targets, the unrelocated adjustment word and all four
dispatch instruction words. LLVM retains the method and bounded factory,
pool-selection, holder and task-body windows. Nearest exported symbol headings
in this stripped member are not identities for adjacent local methods.

rea-executor.json retains complete REA Evidence
`ev_c2a29398a37b5b3f570ce62f5919189851fd351450ade07ea7d02f9de5bebdea`,
command `e scr.color=0; pd 4 @ 0x6679f0` on the merged member, exit 0,
untruncated output, unchanged member hash and an entrypoint warning. This is a
static instruction observation. Provider version/external inputs are not bound,
and annotations are not runtime memory reads.

**Remaining:** actual loader binding and supplied overrides, exception/lifetime
behavior, parameter-map file outcomes, configuration-owner installation,
durability and retry/account ownership. No Rust runtime or private state changed.
