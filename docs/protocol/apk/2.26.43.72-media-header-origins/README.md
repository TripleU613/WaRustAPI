# Original media descriptor headers — APK2.26.43.72

**Confirmed selected static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No account, network or APK execution. Follow-through of the
[header provider/copy investigation](../2.26.43.72-media-header-producers/README.md).
[Complete independent REA method](../../rea/2.26.43.72-media-header-origins/README.md)
corroborates the checked DEX for ACp.A01; its Java carries decompiler type-inference
warnings, so bytecode register binding is the primary argument authority.

## Original map binding

MJe.A02's sole physical field write is its constructor, assigning argument2.
Six physical MJe constructor callers are retained; this is not reflection or
runtime reachability completeness. ACp.A00/A02/A03 and history-sync0jK.A01 use
MFB.A00 to construct that descriptor. MFB.A00 supplies null to constructor
argument14 (v16 from nullv3); kind is newsletter or optimistic. MFB.A01 also
supplies null argument14 (v14 from nullv1), kind newsletter or mms. We exported
the complete factory, not merely an initializer snippet.

The ordinary message assembly ACp.A01 directly constructs M5T with null
argument14: v51 is zero at01fe, copied to v60 at047c before invoke v46..v74
at04a0. No intervening v51 reassignment appears in the complete body. Its kind
argument10 is newsletter or mms. It wraps this descriptor in MJe at04c0.
M5T.A00 copies oldA0I (007c, v3) to constructor argument14 (v28 at0102), just as
Mtt.A05 preserved it in the previous bundle. These copies do not create headers.
The already-retained BaseMediaUploadPlugin.A09 direct MetaAI constructors supply
null maps too, but they are not the basis of the ordinary-message finding.

**Confirmed scoped conclusion:** for this traced ordinary-message descriptor
chain, absent extra headers are consistent with the APK. Together with the
newsletter-only provider aggregation, it supplies no evidence for adding backup
or newsletter metadata to Rust's ordinary image/PTT upload. This says nothing
about unrelated HTTP default headers, effective transport, or server acceptance.

## Separate nonempty backup map

M5I.A00 is explicitly a backup API (signature BackupCycleMetadata; use-case
wa_backup, descriptor kind backup). It builds a map, passes it as constructor
argument14 (v27 from v2), then wraps that descriptor in MJe. Map names include
X-WA-Backup-Custom-Metadata, X-WA-Upload-Authorization-Token,
X-WA-Backup-Use-Case, X-WA-Backup-Container-Id, X-WA-Backup-File-Key and
X-WA-Backup-Idempotency-Key. Transaction ID is conditional on nonnull input;
waffle token on nonnull/nonempty input; file priority on nonnull value; cycle
metadata on nonnull BackupCycleMetadata. Values come from this backup method's
arguments/objects, not invented constants for ordinary uploads. The complete
method also contains backup result/error handling; this chunk does not port or
claim that separate subsystem is complete.

**Unknown:** runtime provider contents/lambda callbacks, native platform default
headers, complete transport selection/fixed-length binding, all backup behavior,
and server-side mandatory checks. Physical constructor/field censuses do not
prove dynamic reachability, alias absence or all runtime paths. No Rust behavior
changed or live validation performed. Continue with the native one-shot
fixed-length/body-mode inputs; do not infer them solely from extra-map absence.

## Reproduction

From repository root:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-field-refs "$APK" --class 'LX/MJe;' --field A02 --type 'LX/M5T;' --writes-only > descriptor-writes.txt
target/debug/xtask apk-smali "$APK" --class 'LX/MJe;' --method '<init>' --signatures --exceptions > descriptor-holder.txt
target/debug/xtask apk-callers "$APK" --class 'LX/MJe;' --method '<init>' > descriptor-holder-callers.txt
target/debug/xtask apk-smali "$APK" --class 'LX/MFB;' --signatures --exceptions > descriptor-factory.txt
target/debug/xtask apk-smali "$APK" --class 'LX/ACp;' --signatures --exceptions > upload-assembly.txt
target/debug/xtask apk-smali "$APK" --class 'LX/0jK;' --method A01 --signatures --exceptions > upload-dispatch.txt
target/debug/xtask apk-smali "$APK" --class 'LX/M5I;' --method A00 --signatures --exceptions > upload-alternate.txt
target/debug/xtask apk-smali "$APK" --class 'LX/M5T;' --method A00 --signatures --exceptions > descriptor-copy.txt
```

checksums.json covers every other file with byte count and SHA256.
