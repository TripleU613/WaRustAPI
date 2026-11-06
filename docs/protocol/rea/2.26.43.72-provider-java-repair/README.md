# Verified REA Android CLI after Java-path repair

**Observed host repair / confirmed selected static**, APK2.26.43.72,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Direct complete REA Evidence records extracted without modification from each
successful CLI wrapper's data object; wrapper command timing stays in the local
run logs. APK never executed. No account or device operation.

The Codex REA entry referenced a missing Java21 directory. Existing installed
Temurin21.0.13 was verified with java -version. A private backup preceded changing
only mcp_servers.rea.env.JAVA_HOME; parsed remaining settings were verified equal.
No config/backup contents or credentials are committed. Existing JADX0.7.1 JAR
was reused. A fresh pinned REA6.5.0 CLI with both JAVA_HOME and REA_JADX_MCP_JAR
succeeded. The first fresh CLI without the JAR environment correctly refused;
Codex's configured JAR path already existed and required no change.

The already-running MCP connection still returned its old missing-Java path
after configuration changed. This is a fresh-CLI repair proof and a persisted
future-launch configuration correction, NOT proof the current MCP connection
reloaded. No shared MCP process or desktop app was killed/restarted. No Java
installation, global shell runtime activation or unrelated settings changed.

## Retained records

- [constructor.json](constructor.json): ev_23215900d710aef030a8d28bd82982ff910828df48fd45b581b1ff2470b2895e; SHA256 `d14f95f403d8a8b4939f623a86ac07f4fe87bc27519f94cac633bfe704811e7d`.
- [provider-class-search.json](provider-class-search.json): ev_7a71e04cad38951490e73ef00b75ece768ec3f607d70c92d76c2b28421ee68eb; SHA256 `1ae93fb86afa1c5dcf4e9211f4f6e5d53c300d83316ed0b3bf42c114947af705`.
- [provider-resolve.json](provider-resolve.json): ev_607f1e3b1917980291712d634bbf7fb99abff8ba9059d1ee3d993eff672370e8; SHA256 `b0ba2a0ebd5c51fb8e67d23e888f39944e8e8ba0946da67bdcabbd6995e08dc9`.

Dxh constructor and C00D.A03 are available, complete Java bodies. The exact
class-name search returns X.C00D for query00D; provider display aliases remain
distinct from raw LX/00D DEX descriptors. Engine identity, source revision,
worker/heap settings, artifact digest and limitations remain inside each record.
No explicit Rust ownership is assigned to these provider-helper observations;
retaining them does not mean a Rust DI port or full APK parity.

[Checked-DEX cache-initialization trace and host limits](../../apk/2.26.43.72-provider-cache-initialization/README.md).

From checkout root, repeat the exact selected method with the existing local
runtime/JAR (outside Git):

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS REA_JADX_MCP_JAR=/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/tools/jadx-headless-mcp-0.7.1-all.jar npx --offline --no-install rea-agents@6.5.0 inspect-android-method ../android-lab/apks/WhatsApp.1.apk X.C00D A03 --overload-index 0 --format json --full-output
```

Use inspect-android-method APK X.Dxh '<init>' --overload-index 0 for the
other body, or search-android-classes APK 00D for the exact alias census,
with the same environment and --format json --full-output options. Full static decompiler limitations remain, including no APK
signature/runtime/whole-call-flow proof. No traffic-reliability or ban guarantee.
