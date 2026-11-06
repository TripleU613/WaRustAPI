# Played job exception hook: REA

Complete unmodified data Evidence from REA6.5.0/JADX0.7.1, pinned APK43.72;
complete Java source without fallback. Static only, no job or account execution.
Existing installed Java/JADX CLI avoids the known stale MCP Java path.

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk com.whatsapp.messaging.receipts.jobqueue.job.SendPlayedReceiptJobV2 A0N --overload-index 0 --format json --full-output
```

Extract the complete data object; per-run metadata may vary.
[DEX methods, caller/runner comparison and limits](../../apk/2.26.43.72-played-job-retry/README.md).
