# Blocklist migration manager: REA

Complete unmodified data Evidence from REA6.5.0/JADX0.7.1, pinned APK43.72.
Complete Java source without fallback; no APK, account or request execution.
The existing CLI uses installed Java21, avoiding the known missing Java path
in the connected MCP. No configuration/engine installation was performed.

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.C88m A00 --overload-index 0 --format json --full-output
```

Extract the complete data object; per-run metadata may vary.
[Selected worker, schema/asset exports and limits](../../apk/2.26.43.72-blocklist-migration-obligation/README.md).
