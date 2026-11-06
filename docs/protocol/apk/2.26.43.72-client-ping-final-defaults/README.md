# Final compiled client-ping integer defaults

Confirmed static source, APK 2.26.43.72 (264307238), SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
These are xtask DEX exports and a bounded argument-flow proof, not REA Evidence
envelopes or live observations.

The preceding [selected timing entries](../2.26.43.72-client-ping-scheduling/README.md)
showed assignments but did not exclude subsequent replacement. This bundle closes
that question for three integer keys, through construction of `LX/08Y;->A02`.

| Key | Final compiled value (ms) | A0I put PC |
| --- | ---: | --- |
| 15103 | 32000 | 07ec |
| 15183 | 240000 | 07ce |
| 15270 | 15000 | 07ae |

## Proof scope

`08Y.<init>` creates five distinct fresh builders. Its integer builder (model
identity 1, constructor v13) reaches only A0B, A0I and A0Q at PCs 43b2, 43b8 and
43be. Other helpers receive distinct builders. The constructor and those three
helpers are linear and have zero exception ranges. The model visits 5551 puts:
2551 in the constructor and 1000 per helper. Of these, 4158 write the integer
builder, representing 3585 distinct keys. The difference is excess writes, not
a count of distinct duplicated keys. Every integer key/value resolves to a
boxed constant. Each selected key occurs exactly once across this whole path.
The built snapshot is assigned to the constructor's actual `this.A02` at
cc24..cc28, with no later target write. Thus these three entries are not replaced
by subsequent default-builder writes.

`fresh-builder.txt` retains the complete small ImmutableMap.builder method;
`build-entry.txt` retains buildKeepingLast's entry delegating to build(false).
The selected keys are unique, so their result does not depend on resolving
other keys' duplicate handling.

The Python model tracks constants, moves, boxed integers, builder identities,
helper arguments, snapshots and final owner binding. It rejects unsupported
instructions/control flow, unknown integer arguments, unexpected target-builder
escapes, wrong final owner/map and writes after snapshot. It pins complete source
export hashes in `defaults-proof.json`. Bulk constructor/helper exports stay
outside Git and can be regenerated from the pinned APK. This is a scoped normal
construction proof, not a general Dalvik interpreter or dynamic/reflection,
native-code, concurrency or process-lifetime proof.

## Reproduce from the repository root

```sh
python3 docs/protocol/apk/2.26.43.72-client-ping-final-defaults/derive-defaults.py --apk ../android-lab/apks/WhatsApp.1.apk > /tmp/ping-final-proof.json
cmp /tmp/ping-final-proof.json docs/protocol/apk/2.26.43.72-client-ping-final-defaults/defaults-proof.json
python3 docs/protocol/apk/2.26.43.72-client-ping-final-defaults/check-model-guards.py --apk ../android-lab/apks/WhatsApp.1.apk > /tmp/ping-final-guards.json
cmp /tmp/ping-final-guards.json docs/protocol/apk/2.26.43.72-client-ping-final-defaults/model-guard-checks.json
```

Build `target/debug/xtask` first if absent. The guard script injects four mutations
after verifying export identity: control flow, unresolved key, unexpected helper
escape and wrong owner. All are rejected. These are model guard checks, not
Android execution tests.

## Remaining boundary

Final compiled defaults are not effective account configuration. The
[first-access trace](../2.26.43.72-client-ping-first-access/README.md) still applies:
saved preferences, SDK readiness/selection, direct versus comparison precedence,
first-access caching and owner lifetime must be respected. This research does not
change Rust timer values or claim server acceptance, native effective timing,
complete protocol parity or a guarantee against bans.
