# 2.26.43.72 against what this client relies on (2026-11-01)

The owner's pre-release asset `WhatsApp.1.apk` (2.26.43.72, version code
264307238, SHA-256 `887a6f94…0c2e`) was compared with the 2.26.37.71 APK on the
workstation (`1755111a…7d5c`). Scope: the inputs the current client depends on
to register, log in and exchange messages. It is not a full re-trace of every
research topic.

## Results

| What we rely on | Result | How it was checked |
| --- | --- | --- |
| Signing certificate (token input, `IMPORTED.DSA`) | **same** (`39:87:D0:43:…:5D:44`) | `xtask reference-manifest` |
| Binary-node token dictionary | **identical**, 236 + 4×256 entries; the initializer moved from `classes.dex 1Rl.<clinit>` to `classes2.dex 1ro.<clinit>` | `xtask apk-tokens` on both, diff of the outputs |
| `about_logo.png` (token key input) | **unchanged** (both densities absent from the file diff) | `xtask apk-file-index` + `apk-file-diff` |
| `classes.dex` (its MD5 is a token input) | **changed** (`a73988a6…` → `96fd9ecb…`, SHA-256) | same |
| String constants our Rust code uses | 3 of our literals are absent in 2.26.43.72: `credentials`, `edited`, `parent_consent_url`. None is sent on the wire: the first is a debug field name, the others occur only in tests. | `xtask apk-strings` on both; set difference intersected with every 2–80 character string literal in `crates/**/*.rs` |
| Research topic anchors | the strict `apk-research-remap` resolves the token-dictionary initializer uniquely; `core`, `signal-wire`, `prekey-kem-wire`, `message-ack`, `token-issue` each have topics left unanchored, so no remapped recipe was written | `xtask apk-research-remap --recipe …2.26.37.71-<topic>.json` |

Overall: 12,537 files changed, 2,928 added and 2,167 removed; 20,754 dex
strings disappeared and 65,321 appeared. The native `libs.so` changed.

## What this means

- **Registration tokens.** The key derivation inputs (package name,
  `about_logo.png`, signing certificate) are unchanged, but the `classes.dex`
  MD5 differs. So a 2.26.43.72 token differs from a 2.26.37.70 token for the
  same number. It belongs with the 2.26.43.72 User-Agent; mixing a token and a
  User-Agent from different builds is the known `bad_token` failure.
  **Observed live 2026-11-02:** one `/v2/code` for lab account `m`
  (`lab-signup retry-after-block`, same identity, `enc_empty_h`, the eight
  observed `code_bytes`) with User-Agent `WhatsApp/2.26.43.72 Android/unknown
  Device/unknown-unknown` and a token derived by `xtask apk-registration-token`
  from the 2.26.43.72 APK was answered `status: sent` (SMS; `sms_wait` 125 s,
  `voice_wait` 65 s). The 2.26.43.72 token derivation is therefore accepted.
  `xtask apk-registration-token --country-code … --national-number …
  /tmp/wa-apk/WhatsApp.1.apk` derives it directly from the APK.
- **Login and messaging.** Nothing we encode by name was removed and the token
  dictionary is unchanged, so the 2.26.43.72 build gives no evidence of a
  wire-format break. Login still presents app version 2.26.37.70
  (`transport/login.rs`). The server can retire that version at any time.
- **Diagnosability fix made with this comparison.** A refused login used to
  drop the server's `<failure reason=…>` (`Error::Authentication { code: None }`).
  It now keeps the number in the error and in the
  `comparison-primary-login-v1` checkpoint. Observed: 403 for the banned
  account `l`. 405 is the outdated-client code in public multi-device clients;
  this is **reference only**, not traced in the APK. If `listen` starts failing
  with `authentication failed (Some(405))`, the version presented at login
  (and then the registration recipe) is what needs to move to a newer build.

## Not done

- No remapped research recipes, and no re-trace of the 194 retained topics on
  2.26.43.72. Obfuscated names moved; the strict remap needs per-topic anchor
  work.
- Protobuf field numbers were not compared descriptor by descriptor. The
  string-level check cannot see a renumbered field.
- The Noise certificate trust root (a byte array, not a string) was not
  compared. A changed root would fail login visibly, not silently.
- No 2.26.43.72 request was sent.
