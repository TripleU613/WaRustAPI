# Complete REA upload execution Evidence

Pinned APK2.26.43.72, SHA256887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e.
REA6.5.0, JADX0.7.1, complete data objects from successful CLI wrappers.
Java decompiler warnings are preserved; corroborate using checked DEX branch
coordinates. [Interpretation and limits](../../apk/2.26.43.72-media-body-mode/README.md).
No target/account execution. With installed Java21 JAVA_HOME and existing
REA_JADX_MCP_JAR, reproduce from repository root:

```sh
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.JAS A04 --overload-index 0 --format json --full-output
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.JAT A04 --overload-index 0 --format json --full-output
```

Extract complete data objects, preserving Evidence fields.

SHA256:
- tigon-execute.json: `a7b4f0ff9b812e77110c8e336ed7c8f77ba9835bfea93fcac74d3f04d17655ab`
- url-execute.json: `e65a52051b11844be5cd2fec044ab423239162ce8e01ad0336940261d0f87033`
