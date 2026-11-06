# Played receipt selection: REA

Complete unmodified data Evidence from REA6.5.0/JADX0.7.1, pinned APK43.72.
Java export is complete without fallback, but emits a duplicated-code warning
and omits an explicit assignment on one z branch. Use the linked complete DEX
as authority for branch targets/initialization; do not treat reconstructed Java
as executable or proof of every path. No job/account execution.

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk com.whatsapp.messaging.receipts.jobqueue.job.SendPlayedReceiptJobV2 A0L --overload-index 0 --format json --full-output
```

Extract complete data object; per-run metadata can vary.
[Checked DEX predicates, production correction and limits](../../apk/2.26.43.72-played-privacy/README.md).
