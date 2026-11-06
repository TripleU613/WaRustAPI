# Provider-cache initialization and repaired REA CLI — APK2.26.43.72

**Confirmed selected static**, code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete checked DEX exports and [complete fresh REA records](../../rea/2.26.43.72-provider-java-repair/README.md).
REA6.5.0/JADX0.7.1 now succeeds in a fresh CLI with the installed Java21 runtime;
the already-running MCP retains its stale environment. Host repair details and
private-backup boundary are in the linked README. No target/account execution.

## Direct cache references

[Exact physical field census](cache-field-refs.txt) of00D.A03 ConcurrentHashMap
finds four sites: class initializer write, resolver A03 read,0BH.A00 read and
0BH.A01 read. The latter's Map.put initially looked like a possible replacement
path. [Complete accessor](cache-accessor.txt) stores its integer key, gets that
entry in A00 and puts a supplied value in A01. This alone does not prove a
reset, account switch or arbitrary replacement is executed.

[Complete00D.A01 optional factory](optional-factory.txt) reads a0BF descriptor.
Key-1 constructs0Bh(null). A selected boolean branch creates
0BH(key,07N.A00(key)) and0BM with its0BJ nullable supplier; the other branch
uses a05h-backed wrapper. Effective descriptor values, provider0x90 selection
through this optional factory and every caller remain outside this trace.

[0BG](cache-abstraction.txt) holds that lock object and declares the get/put
abstraction. [Complete0BM.A01](optional-initializer.txt) checks its instance and
scope-cache checker, reads the cache, and enters the monitor only for null. It
rechecks cache under lock; only if still null does it invoke the supplier and
put the supplied value or its missing-value sentinel. This selected branch
calls put only after its own cache reads found null; complete concurrency and
lock-factory behavior are not proven here. This refutes interpreting these extra
field reads alone as proof of a reset. No selected Map.clear/remove occurs in
these retained accessors/resolver, but this physical inventory does not exclude
reflection, native code, class-loader/process reset or all external mutation.
The previous conditional provider0x90 identity proof remains conditional.

The [fresh complete Java resolver](../../rea/2.26.43.72-provider-java-repair/provider-resolve.json)
corroborates get/recheck/per-key monitor/nonnull put in00D.A03. No decompiler
output is upgraded to a runtime/account-lifecycle observation. The resolver's
full source was already retained as checked DEX in the
[prior failure-owner bundle](../2.26.43.72-full-key-failure-ownership/README.md).

## Reproduce checked DEX

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/00D;' --field A03 --type 'Ljava/util/concurrent/ConcurrentHashMap;'
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0BH;' --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/00D;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0BG;' --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0BM;' --method A01 --signatures --exceptions
```

PCs are bytes; branch/catch coordinates use code units. Complete selected
methods/classes are retained; only the cache-initialization blocks are interpreted.
checksums.json covers every other file here. No Rust behavior, timers, retries,
sender rules, durable reset or live account operation changed.
