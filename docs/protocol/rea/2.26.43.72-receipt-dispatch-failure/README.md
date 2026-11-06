# Receipt dispatch readiness: REA

Complete unmodified data Evidence from REA6.5.0/JADX0.7.1, pinned APK43.72.
The connected MCP failed because its Java path did not exist; the existing CLI
with installed Java21 produced this complete Java method without fallback or
truncation. No engine/configuration installation or account execution.

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.C0Sf A05 --overload-index 0 --format json --full-output
```

Retain the complete data object; per-run evidence/runtime metadata can vary.
[Exact DEX exports and comparison](../../apk/2.26.43.72-receipt-dispatch-failure/README.md).
