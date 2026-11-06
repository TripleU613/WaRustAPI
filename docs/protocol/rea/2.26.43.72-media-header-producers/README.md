# Complete media header REA Evidence

Pinned APK2.26.43.72 SHA256887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e.
Complete data objects from successful REA6.5.0 inspect-android-method wrappers,
JADX0.7.1, selected overload0, full-output Java. No target execution.

Reproduce with installed Java21 JAVA_HOME and existing REA_JADX_MCP_JAR:

```sh
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.MvM A02 --overload-index 0 --format json --full-output
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.AnonymousClass845 A00 --overload-index 0 --format json --full-output
```

Extract complete data objects, preserving Evidence fields. Interpretation and
limits: [checked DEX bundle](../../apk/2.26.43.72-media-header-producers/README.md).

SHA256:
- header-aggregate.json: `b69d43acf2f262831472e69eb834c9a0baac44bf567057e249302fdd261d0552`
- header-input.json: `a47c48bda8707b431e8826b6ce873318b30f8b90840d392e82dd77c99c1cd7b0`
