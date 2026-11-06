# Blocklist set result reconciliation: REA

Complete unmodified data Evidence from REA6.5.0/JADX0.7.1, pinned APK43.72.
Complete Java export, no fallback; no APK/account execution. Existing CLI uses
installed Java21 to avoid the known missing Java path in the connected MCP.

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.C8R7 CK4 --overload-index 0 --format json --full-output
```

Retain the complete data object; per-run metadata may vary.
[DEX branch/callback/source census and correction](../../apk/2.26.43.72-blocklist-set-reconciliation/README.md).
