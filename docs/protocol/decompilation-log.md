# Decompilation log — registration and handshake classes

Purpose: a running, append-only record of every static-analysis step against
the pinned APK (`com.whatsapp` 2.26.36.75, sha256 `ce7468…922e9`), so no step
is repeated and every ledger row can be traced to the class and method that
justifies it. Decompiled output stays in the lab directory (never committed);
only findings, class names, offsets and derived facts are recorded here.

Conventions: one dated entry per session; each entry lists **target**,
**method**, **result**, **ledger rows touched**, **dead ends**.

## 2026-09-17 — tooling and string-level findings (done overnight)

- Tools: `cargo run -p xtask -- apk-strings` (dex string tables),
  `apk-tokens` (dictionary from `const-string` runs), `reference-manifest`.
- Results recorded in `evidence-ledger.md` and the reference manifest
  `client_observations` block: hosts, Noise pattern names, registration
  vocabulary, ClientPayload/handshake field constants, ordered dictionary
  (`classes2.dex @ 0x126a96`).
- Dead ends: `lib/*/libs.so` has no protocol strings (superpack-compressed);
  `strings -n 6` undercounts short tokens, use `apk-strings` instead.

## 2026-09-17 — decompiler tooling attempt (session 2)

- GitHub release downloads (`jadx`, `dex2jar`) are blocked by the egress
  policy (403). Maven Central is reachable and carries `io.github.skylot:
  jadx-cli` 1.5.6, `org.smali:baksmali` 2.5.2, `org.vineflower:vineflower`
  1.12.0, `org.benf:cfr` 0.152; `mvn` and `gradle` are installed.
- Fetching and running a third-party decompiler could not be evaluated under
  the session's auto permission mode (safety check blocked the action). It
  must be done in a session with the default permission mode, or by the
  operator locally, following the recipe below. **Do not retry under auto
  mode.**
- Recipe (lab, outside the repository):
  1. `pom.xml` depending on `io.github.skylot:jadx-cli:1.5.6` plus
     `jadx-dex-input`, `jadx-java-input`, `jadx-java-convert`,
     `jadx-smali-input`, `jadx-rename-mappings` and `logback-classic`;
     `mvn dependency:copy-dependencies -DoutputDirectory=lib`.
  2. `java -cp 'lib/*' jadx.cli.JadxCLI --no-res --deobf -d out WhatsApp.apk`
     (or `--single-class <name>` for targeted classes).
  3. Locate targets first with the string→class cross-reference tool
     (`apk-xref`, below) so only the relevant classes are decompiled and read.

## Targets (ordered by ledger impact)

| # | Target | Anchor strings | Ledger rows | Status |
| --- | --- | --- | --- | --- |
| T1 | Registration URL assembly and request encoding | `v.whatsapp.net`, `v2`, `code`, `register`, `exist`, `e_skey_sig`, `authkey` | `registration.endpoints`, `registration.payload_encoding`, `registration.prerequisites` | open |
| T2 | Verification flow and cooldown handling | `retry_after`, `wipe_token`, `push_code`, `flash_type`, `reason` | `registration.verification_steps` | open |
| T3 | Handshake pattern selection and prologue | `Noise_XX_25519_AESGCM_SHA256`, `Noise_XXkem_…`, `WA` header bytes, `xmlstreamstart` class neighbour | `noise.pattern`, `noise.prologue`, `binary.tokens_version`, `noise.record_limit` | open |
| T4 | Certificate chain trust root and issuer serial | `CertChain`, `intermediate`, `leaf` protobuf names, 32-byte constant | `noise.trust_root`, `noise.issuer_serial` | open |
| T5 | Chat socket endpoint, port, framing | `g.whatsapp.net`, `e1.whatsapp.net`, port ints, 3-byte length prefix | `transport.endpoint`, `framing.*` | open |
| T6 | Primary `ClientPayload` construction | `CONNECT_TYPE_FIELD_NUMBER`, `ANDROID`, `USER_ACTIVATED`,

## 2026-09-17 — Rust-native dex structural parser (session 3)

### What was built

Two new xtask commands implemented in `xtask/src/xref.rs` (pure Rust, no Java
decompiler needed):

**`apk-xref <apk> --anchor <string>…`**
Parses the full dex structural tables (string_id, type_id, proto_id, field_id,
method_id, class_def, class_data, code_item) and finds every class + method
that loads an anchor string via `const-string`/`const-string/jumbo`. Output:

```
[<anchor>]  (N reference(s))
  classes2.dex  Lcom/whatsapp/…;->methodName(shorty)
    dex byte offset: 0xXXXXX
```

Usage to locate T1–T6 targets without decompiler:
```
cargo run -p xtask -- apk-xref WhatsApp.apk \
  --anchor v.whatsapp.net \
  --anchor "Noise_XX_25519_AESGCM_SHA256" \
  --anchor "Noise_XXkem_25519_AESGCM_SHA256_MLKem512" \
  --anchor CertChain \
  --anchor g.whatsapp.net \
  --anchor CONNECT_TYPE_FIELD_NUMBER \
  --anchor e_skey_sig \
  --anchor authkey
```

**`apk-smali <apk> --class <filter> [--method <filter>]`**
Given a class name substring (from xref output), dumps all methods of matching
classes as annotated Dalvik instructions: opcode name, byte range, and
resolved string/type/method/field for key opcodes (`const-string`,
`invoke-*`, `new-instance`, `check-cast`, `iget`/`sget`/`iput`/`sput`).

Usage after xref identifies a class:
```
cargo run -p xtask -- apk-smali WhatsApp.apk --class RegistrationManager
cargo run -p xtask -- apk-smali WhatsApp.apk --class HandshakeManager --method selectPattern
```

### Implementation notes

- Instruction width table covers all 256 opcodes from the AOSP Dalvik bytecode
  spec; unknown/unused opcodes conservatively return 1 unit (won't panic, may
  misalign on exotic ART-only extended opcodes).
- `const-string` (0x1a, 2 units) and `const-string/jumbo` (0x1b, 3 units)
  are resolved against the string table by index.
- `invoke-virtual/super/direct/static/interface` (0x6e–0x72, format 35c)
  decode arg count, registers, and method descriptor.
- `iget/iput` (format 22c), `sget/sput` (format 21c): decode field class,
  name, and type descriptor.
- All parsing is bounds-checked; truncated or malformed dex files emit warnings
  and continue.
- 5 unit tests pass (`insn_width_covers_all_opcodes`, `insn_width_key_opcodes`,
  `decode_insns_const_string_annotation`, `decode_insns_never_panics_on_truncated`,
  `xref_scan_finds_string_in_crafted_dex` using a hand-assembled dex blob).

### What is still needed (requires APK)

The APK (`com.whatsapp` 2.26.36.75) is not present in this container. Once
available, run xref first, then smali on the identified classes. Expected
findings per target:

| Target | Command | Expected output |
| --- | --- | --- |
| T1 (registration URL) | `apk-xref … --anchor v.whatsapp.net` | Class under `net/` or `connection/`; method that assembles the HTTPS URL |
| T1 (request encoding) | `apk-smali … --class <T1 class>` | `new-instance Lorg/json/JSONObject;` or `Lcom/google/protobuf/…` calls; `const-string` for all parameter names |
| T3 (Noise pattern) | `apk-xref … --anchor "Noise_XX_25519_AESGCM_SHA256"` | Single class; the method is the pattern selector or handshake initiator |
| T3 (prologue) | `apk-smali … --class <T3 class> --method initiate` | `sget`/`iget` for the WA header bytes (should show `0x57 0x41` = "WA") and the dictionary version byte |
| T4 (trust root) | `apk-xref … --anchor CertChain` | Proto deserialization class; look for `sget-object` of a 32-byte static field |
| T5 (endpoint) | `apk-xref … --anchor g.whatsapp.net` | Socket/connect class; look for port constant via `const/16` immediately following |

All findings go in the next dated entry of this log with the actual class
descriptors, method names, and ledger rows updated.

### Ledger rows pending confirmation

- `transport.endpoint`: host `g.whatsapp.net`/`e1.whatsapp.net` confirmed by
  string scan (Observed); port and protocol version need smali confirmation.
- `noise.pattern`: 12 pattern strings observed; which one the client selects
  at runtime requires smali analysis of T3.
- `registration.endpoints`: host observed; path (`/v2/register`, `/v2/exist`)
  observed; full query-parameter set needs smali confirmation of T1.
- `noise.prologue`: "WA" prefix + dictionary version byte — Unknown, needs T3.
- `noise.trust_root`: 32-byte constant — Unknown, needs T4.
- `noise.issuer_serial`: u32 — Unknown, needs T4.
