# Priority-context interceptor constructor: REA

Complete unmodified data Evidence object from REA6.5.0/JADX0.7.1. APK subject
digest and complete source retained; no execution or runtime activation claim.
Fresh CLI uses the existing Java21 and JADX engine, avoiding the known stale
MCP Java environment. Native tools Ghidra/Hopper remain unavailable; native
findings are separate pinned ELF/LLVM evidence, not REA native output.

Reproduce from repository root:

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk com.whatsapp.infra.tigon.WAHttpPriorityContextInterceptor '<init>' --overload-index 0 --format json --full-output
```

Extract complete data object. interceptor-constructor.json SHA-256:
c67758aa0ad44ef7d9a334f5668a44b9a25f1185620940a4142c9817c898a333
