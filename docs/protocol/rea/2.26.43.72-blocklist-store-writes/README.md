# Username writer: REA body unavailable

Complete unmodified data Evidence from REA6.5.0/JADX0.7.1, pinned APK43.72.
The provider reports body_status=not_available and source.status=complete with
only the comment that the method exists but has no decompiled body. That is
**not** a retained method body or proof it is native/abstract in the APK: the
complete DEX declaration export contains concrete instructions. Do not count
this REA result as a completed source port. No APK/device/account execution.

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.C0bZ BQ1 --overload-index 0 --format json --full-output
```

Retain complete data object. [DEX authority and limits](../../apk/2.26.43.72-blocklist-store-writes/README.md).
