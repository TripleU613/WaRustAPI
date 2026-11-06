# Bulk-key missing-LID preflight, APK 2.26.43.72

Confirmed static for APK code264307238, SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected method exports from the supported xtask DEX fallback; no APK execution, live account traffic or REA Evidence ID claim. Android provider setup remains unavailable as previously diagnosed.

## Reproduce

From repository root:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/3KB;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/3KB;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/3KB;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ob;' --method A0A --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ob;' --method A06 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ob;' --method A08 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0x9;' --method A03 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0g --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0YG;' --method A05 --signatures --exceptions
```

The sibling [bulk job evidence](../2.26.43.72-bulk-key-job-completion/README.md) retains the caller and future implementation. These exports extend that trace only through sync request queue submission. The wire renderer, enum meaning, queue batching, success/error acceptance, mapping adoption and effective metadata are untraced here. checksums.json binds all other files by byte count and SHA-256.
