# GraphQL intermediate map export: REA

Complete unmodified data Evidence from REA6.5.0/JADX0.7.1, pinned APK43.72.
Complete Java A03 source without fallback. Other mutation/conversion branches
are retained as complete selected DEX exports, not REA envelopes. No APK,
account or network execution. Existing CLI uses installed Java21, avoiding the
known missing Java path in the connected MCP.

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk com.facebook.graphql.calls.GraphQlCallInput A03 --overload-index 0 --format json --full-output
```

Extract complete data object; per-run metadata can vary.
[Values, callback discriminator checks and scoped deadline](../../apk/2.26.43.72-blocklist-mex-values-lifetime/README.md).
