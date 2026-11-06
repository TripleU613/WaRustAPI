# Complete original upload descriptor REA Evidence

Pinned APK2.26.43.72, SHA256887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e.
REA6.5.0, JADX0.7.1, complete data object from successful fresh CLI wrapper.
Java output has type inference warnings; preserve them and corroborate with
[complete checked DEX](../../apk/2.26.43.72-media-header-origins/README.md).
No target/account execution. With installed Java21 JAVA_HOME and existing
REA_JADX_MCP_JAR, reproduce from repository root:

```sh
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.ACp A01 --overload-index 0 --format json --full-output
```

Extract the complete wrapper data object without modifying Evidence fields.

ordinary-message-assembly.json SHA256: `0922547a437cb8fa01210405836786892b78ebdc3926eb225f5239eeabd56805`
