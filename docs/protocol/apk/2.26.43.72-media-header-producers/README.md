# Media upload header producers — APK2.26.43.72

**Confirmed selected static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution or account operation. Complete checked DEX exports supplement
[complete REA records](../../rea/2.26.43.72-media-header-producers/README.md).

MvM.A02 selects provider845.A00 using descriptorM5T.A0G, substitutes an empty
map for a missing provider entry or null descriptorM5T.A0I, then merges provider
first and descriptor second. Complete05p.A08 copies the first map into a
LinkedHashMap and putAlls the second: descriptor values override matching keys.

845's constructor builds A00 with3ff discriminator8 and A01 with discriminator9
from injected set0x1da8. Complete845.A00 groups every set member under literal
newsletter, then folds callback maps.2LF.A0I gets/creates the group's ArrayList;
6oe.A1Q adds its member. Thus this aggregation has only a newsletter key (or no
keys for an empty set). A00 is the selected header input; A01 is a separate map,
not an additional unconditional header requirement. Effective set contents and
callback bodies are not established by this export.

The physical A0I field-write census finds only the M5T constructor write.
The physical constructor-call census finds eight sites, including copies;
it does not establish all dynamic paths. Constructor argument14 is A0I Map;
argument10 is A0G String. The standard voice plugin's existing retained A09
calls Mtt.A05. Complete Mtt.A05 retains the original descriptor map: at019a it
loads oldA0I into v5; at01e0 moves it to v27, the new constructor's argument14.
It can update A0G from Mtt.A0n (default optimistic), while retaining this map.
Consequently this method does not prove the standard upload map is empty.
The separate MetaAI direct-constructor null-map branches cannot establish that.

**Unknown:** original standard descriptor map producer, all other constructor
paths, effective newsletter entries and callbacks, JAT platform User-Agent
injection, complete runtime transport selection and mandatory server checks.
No Rust header changed: these findings do not establish a missing universal
header or ban prevention. Continue upstream from the original MJe.A02 descriptor.

## Reproduction

From repository root, APK=../android-lab/apks/WhatsApp.1.apk. Use:

```sh
target/debug/xtask apk-smali "$APK" --class 'LX/845;' --signatures --exceptions > header-provider.txt
target/debug/xtask apk-smali "$APK" --class 'LX/05p;' --method A08 --signatures --exceptions > header-map-merge.txt
target/debug/xtask apk-field-refs "$APK" --class 'LX/M5T;' --field A0I --type 'Ljava/util/Map;' --writes-only > header-map-writes.txt
target/debug/xtask apk-callers "$APK" --class 'LX/M5T;' --method '<init>' > header-descriptor-callers.txt
target/debug/xtask apk-smali "$APK" --class 'LX/M5T;' --method '<init>' --signatures --exceptions > header-descriptor-constructor.txt
target/debug/xtask apk-smali "$APK" --class 'LX/2LF;' --method A0I --signatures --exceptions > header-group-list.txt
target/debug/xtask apk-smali "$APK" --class 'LX/6oe;' --method A1Q --signatures --exceptions > header-group-add.txt
target/debug/xtask apk-smali "$APK" --class 'LX/Mtt;' --method A05 --signatures --exceptions > standard-descriptor.txt
```

checksums.json covers every other file with bytes and SHA256.
