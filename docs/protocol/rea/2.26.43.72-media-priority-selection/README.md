# Calculator selection: complete REA evidence

REA6.5.0/JADX0.7.1, pinned APK2.26.43.72. Complete data Evidence objects from
successful class-name search and method inspection, including raw/normalized
results and subject digest. Method source is complete Java with no markers.
No target execution, runtime flag value or server acceptance observed.

Reproduce from repository root, with the existing engine:

```sh
export JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS
export REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar
npx --offline --no-install rea-agents@6.5.0 search-android-classes ../android-lab/apks/WhatsApp.1.apk 1Wv --format json --full-output
npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.C1Wv invoke --overload-index 0 --format json --full-output
```

Extract each complete data object. Raw DEX LX/1Wv maps to provider alias X.C1Wv;
X.1Wv was rejected. The complete failed envelope is retained in the APK bundle,
not counted as successful Evidence. Fresh CLI avoids the known stale MCP Java
environment. Native observations use ELF/LLVM, not unavailable native engines.

SHA-256:
- class-search.json: 3f84d9ee9dc23d560da08a39c5bfaf445a90a07f37d6391fd7743a61bd5dcb94
- lazy-producer.json: 5d3459bdf1ceab22f56c2745d7755b7476b60b6bec6395a6b54a662c0bed007a
