# Upload category: complete REA evidence

REA6.5.0, JADX0.7.1; complete data object, raw and normalized results retained.
Pinned subject digest inside Evidence. Java A02 maps the checked K1R category
branches, including A06 -> MEDIA_UPLOAD. Full native analysis is separate ELF/
LLVM evidence; no invented native REA record and no target execution.
Fresh CLI uses existing Java21/JADX to avoid the known stale MCP Java environment.

Reproduce from repository root:

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.K1R A02 --overload-index 0 --format json --full-output
```

Extract complete data object. upload-category.json SHA-256:
d6d6ef6566bf42fd3d704cc41d3f3ee1e710fbf9e3aa5c34742043ec98419ac5
