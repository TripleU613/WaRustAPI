# Lookup shared response-state lifecycle, APK43.72

Confirmed static for WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues the diagnosed unavailable Android provider. No APK execution, live traffic or REA Evidence envelope claim. Full selected methods and complete physical field/callsite inventories are retained; reflection, runtime aliases and scheduling are outside those inventories.

## Reproduce

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-field-refs "$APK" --class 'LX/0p7;' --field A00 --type 'LX/Gzg;'
target/debug/xtask apk-smali "$APK" --class 'LX/0p7;' --method A0G --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0p7;' --method A0U --signatures --exceptions
target/debug/xtask apk-callers "$APK" --class 'LX/0p7;' --method A0G
target/debug/xtask apk-callers "$APK" --class 'LX/0p7;' --method A0U
```

From repository root. checksums.json binds every other file. Reuse [complete executor](../2.26.43.72-prekey-lid-queue/executor.txt), [multi-query method](../2.26.43.72-prekey-lid-query-route/multi-query.txt) and [MEX callback/observer](../2.26.43.72-prekey-lid-mex-callback/README.md) exports.

## Physical write/read boundary

The exact0p7.A00:Gzg inventory has29 physical references. Write sites are A0G, A0U,0pA.BLh and three instructions in0wB.run; the remaining references read state. A0G and A0U store null at their identified reset/cleanup blocks; BLh stores the provided result. The three executor stores at49d8,53ce and6722 install newly constructed Gzg results; they are not null resets. Constructor Java default null is separate from an explicit field instruction. This does not prove global singleton identity, absence of reflection or freedom from races.

A0G returns true immediately for an empty input list (000c+0x2d*2=0066). On a nonempty list it clears A00 response and A01 error at0010..001c, clears A0e/A0f maps, obtains batch configuration and calls A0F at005e..0064. The caller census contains four sites, all inA0O; retained selected blocks0710/071e/0736/0740 belong to its masked follow-up read path. Do not describe this reset as an unconditional entry action for every A0O branch or a reset on an empty read.

A0U optionally completes telemetry, examines A00/users through396, then clears A0f/A0e, A00/A01 and nullable mapsA02/A03 at0062..0088. Its extra396 behavior is retained but not fully reconstructed. Four physical callers are all0wB.run, including multi-query drained cleanup6bc6 and its exception cleanup7ebe, plus other sync handling6baa/7f18. The drained call follows the executor's empty remaining-list branch at2a68+0x20a9*2=6bba. This is cleanup at that boundary, not proof of cleanup before every batch or transport request.

## What is closed and what is not

The prior callback audit left response-field clearing untraced. Explicit selected reset and drained/exception cleanup are now established; the field is not merely assumed to retain previous responses forever. However an immediate A0F/A0O route has no A00 write of its own in the physical census. Its initial state depends on upstream lifecycle. The earlier observer sets A00 before signaling its associated FfK; callback completion alone is still not a correlation fence for all later users of this shared field.

**Unknown:** effective executor serialization and object aliases, per-batch clearing guarantees, MEX callback cancellation/unregistration after timeout, suppression of late callbacks, callback identity checks and whether any real interleaving can install stale state. The selected success handler calls BLh and the observer stores the result, but no global race or account restriction is proven. Do not copy a shared response slot into Rust or relax its request-owned callback/no-replay rules from this partial native lifecycle. No Rust runtime change or live operation.
