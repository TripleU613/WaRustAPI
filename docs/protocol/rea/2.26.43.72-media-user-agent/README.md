# Media User-Agent holder: REA evidence

Complete `data` Evidence from REA 6.5.0, JADX headless 0.7.1. Static only;
APK never executed. Subject digest is retained inside the Evidence. Java body
is complete, corroborated by the checked DEX in the APK companion bundle.

Reproduce from repository root (existing Java/JADX installation):

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar \
npx --offline --no-install rea-agents@6.5.0 inspect-android-method \
 ../android-lab/apks/WhatsApp.1.apk X.RlW '<init>' \
 --overload-index 0 --format json --full-output
```

Extract the complete `data` object, without dropping raw or normalized results.
Fresh CLI avoided the previously recorded stale MCP Java environment; native
Ghidra/Hopper engines remain unavailable. Native claims use explicit ELF and
LLVM evidence, not a fabricated REA native record.

`holder-construction.json` SHA-256: af529b442ea1850a1a794810e2dcab9d4ebd4d5545b76d774440164e23d54475
