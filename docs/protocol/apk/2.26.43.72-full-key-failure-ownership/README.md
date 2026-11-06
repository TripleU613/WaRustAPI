# Full key-query failure ownership — APK2.26.43.72

**Confirmed selected static**, code264307238, pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked xtask DEX exports, not runtime observations or REA Evidence. No APK
execution, account traffic, new timer or Rust runtime change.

## Writer callback and job future are distinct

The [complete writer](../2.26.43.72-retry-version-error-ack/writer-dispatch.txt)
case87 allocates Dxh and stores it in1rj.A03 at1726..1748 before writing IQ/get/
encrypt. [Complete concrete Dxh](../2.26.43.72-prekey-wire-error-attribution/callback.txt)
extends1ru and declares only its constructor/A01/A02. Fresh full export compared
byte-for-byte equal to that retained file. Constructor0014 calls Object.<init>
and only stores fields; it does not register a callback timer in this body.
The job's [previously traced FfK](../2.26.43.72-bulk-key-job-completion/README.md)
is separately inserted in0Th.A05 and awaited with untimed get(). Neither that
wait nor the selected writer registration establishes32000ms or a universal
absence of other watchdogs. Manager60000 expiry is still a separate mechanism.

## Reader map cleanup does not release the bulk future

[Complete1ru.A00](reader-cleanup-hook.txt) is a bare return. The concrete
Dxh allocation therefore inherits that no-op hook. [1s2.run](../2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
selected IOException/LIL initial-read cleanup invokes1ru.A00 for its reader-map
entries at1164..116a, then clears that map and rethrows. For a Dxh entry that
hook does not invoke FfK.BsT/BsR, settle keys, or clear0Th.A05. This is not a
universal no-op claim for every1ru subtype, nor proof a later disconnect cannot
fail the job future.

Result/error processing instead removes the lower-level callback and invokes
its family handler before forwarding to the ordinary registry/fallback. The
[fallback completion](../2.26.43.72-bulk-key-job-completion/README.md) removes
0Th.A05's entry and calls BsT(null). That releases a wait; it is not proof of
usable signed keys or successful state adoption. Exceptions before forwarding
need not reach this completion block.

## Failure callers and provider binding

The [exact physical caller census](future-failure-callers.txt) finds two direct
DEX callers of0Th.A03:0YB.A05 and0YU.handleMessage. This is not an alias,
reflection, native-code or dynamic execution census.

[0YB.A05](../2.26.43.72-iq-shutdown-eligibility/disconnected-handler.txt)
resolves A0W and calls A03 with IOException(disconnected) at01da..01fa, before
its later0ln.A0F call. The selected [0YU login-back failure block](../2.26.43.72-iq-shutdown-eligibility/message-handler.txt)
checks1sJ.A00 at05e2..05ee and, when admitted, passes its2AF exception to A03
at065c..0662. This is not a claim that every login failure executes that branch.
[Complete0Th.A03](../2.26.43.72-retry-refetch-callback-lifetime/callback-manager.txt)
iterates A05 under its monitor, invokes1ue.BsR on each reached callback, then
clears the map. A thrown callback can prevent reaching later entries/clear;
no universal cleanup success is inferred. Reached FfK.BsR counts down failure;
its untimed get wraps the saved exception in ExecutionException.

[0YB constructor](disconnect-constructor.txt) assigns A0W from00D.A00(0x90)
at0284..0294; its [write census](disconnect-provider-writes.txt) has one direct
constructor write. [0Sf constructor](message-client-constructor.txt) assigns
A0B from00D.A02(0x90), cast0Th, at00ee..0102. BulkGetPreKeyJob obtains its
future map from that0Sf.A0B. [A00](provider-lazy-factory.txt) constructs05h with
the key; [05h.get](lazy-provider.txt) resolves A02 once while its local cached
field is null. [A02](provider-value.txt) forwards to [A03](provider-resolve.txt),
whose static ConcurrentHashMap caches a nonnull value by integer key under a
per-key monitor. Both access paths therefore select the same global cached
0x90 object while that entry remains unchanged. Lazy caches can retain earlier
objects: account switches, provider invalidation/reset, aliases and global
initialization lifecycle remain outside this conditional binding proof.

## Tool limitation

The newly advertised REA inspect_android_method call for X.Dxh.<init> failed
before decompilation with unsupported_provider/capability_unavailable because
its configured JAVA_HOME executable /usr/lib/jvm/java-21-openjdk/bin/java is
missing. The [complete error result](rea-provider-error.json) has no Evidence ID
and is not counted as successful REA evidence. Local command discovery found a
mise java shim; that does not validate the REA server's Java environment. No
client/server configuration or system Java installation was changed. Supported
xtask disassembly continued the original target investigation.

## Reproduction

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1ru;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Dxh;' --signatures --exceptions > /tmp/full-key-Dxh.txt
cmp /tmp/full-key-Dxh.txt docs/protocol/apk/2.26.43.72-prekey-wire-error-attribution/callback.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Th;' --method A03
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/0YB;' --field A0W --type 'LX/00x;' --writes-only
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0YB;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Sf;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/00D;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/00D;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/00D;' --method A03 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/05h;' --signatures --exceptions
```

REA attempted inspect_android_method(class_name=X.Dxh, method_name=<init>,
overload_index=0) against the same pinned local APK. PCs are bytes; branches and
catch coordinates use code units. Complete methods are retained or linked;
checksums.json covers every other file here. No native whole-app deadline,
replay authority, runtime parity or restriction-prevention guarantee follows.
