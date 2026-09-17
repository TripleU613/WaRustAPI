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
| T1 | Registration URL assembly and request encoding | `v.whatsapp.net`, `v2`, `code`, `register`, `exist`, `e_skey_sig`, `authkey` | `registration.endpoints`, `registration.payload_encoding`, `registration.prerequisites` | newer-build comparison complete; pinned build open |
| T2 | Verification flow and cooldown handling | `retry_after`, `wipe_token`, `push_code`, `flash_type`, `reason` | `registration.verification_steps` | open |
| T3 | Handshake pattern selection and prologue | `Noise_XX_25519_AESGCM_SHA256`, `Noise_XXkem_…`, `WA` header bytes, `xmlstreamstart` class neighbour | `noise.pattern`, `noise.prologue`, `binary.tokens_version`, `noise.record_limit` | header and selector map resolved in newer build; active config and pinned build open |
| T4 | Certificate chain trust root and issuer serial | `CertChain`, `intermediate`, `leaf` protobuf names, 32-byte constant | `noise.trust_root`, `noise.issuer_serial` | newer-build comparison complete; pinned build open |
| T5 | Chat socket endpoint, port, framing | `g.whatsapp.net`, `e1.whatsapp.net`, port ints, 3-byte length prefix | `transport.endpoint`, `framing.*` | ports resolved in newer build; layer/framing and pinned build open |
| T6 | Primary `ClientPayload` construction | `CONNECT_TYPE_FIELD_NUMBER`, `ANDROID`, `USER_ACTIVATED` | `login.client_payload` | open |

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
- 8 unit tests pass, including a hand-assembled dex xref test and regression
  tests for direct/virtual method-index deltas, array-data payloads, and
  literal/conversion operands.

### What is still needed (requires the pinned APK)

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

## 2026-09-17 — signed newer-build comparison (session 4)

The mutable official download URL no longer serves the pinned 2.26.36.75
artifact. It served `com.whatsapp` 2.26.37.70 (code 263707033), sha256
`b2090b0207e9b97449a5b48e98eb4d1649ae8e5e738e9b7431d6242717fee732`,
146380260 bytes. Its signing-certificate sha256 is the same as the pinned
client. This build was used only as a dated comparison: none of the findings
below silently changes a ledger row for 2.26.36.75.

The Rust-native parser was corrected before analysis:

- DEX method-index deltas now reset between direct and virtual method lists.
- `filled-new-array` widths and packed/sparse/array-data payload widths are
  decoded, so payload bytes cannot be mistaken for instructions.
- array operations, unary conversions, and `*-/lit8` operands are annotated.
- `apk-callers` now finds invoke sites for an exact class/method pair. It
  traced `KotlinRegistrationBridge->A09` through its blocking coroutine to
  `RequestCodeRepository->A00` without relying on identifier guesses.

### T1 — registration URL and request body

- `LX/HpO;-><clinit>` stores XOR-0x12-obfuscated paths and
  `LX/HpO;->A00(LL)` decodes every character. The decoded paths are:
  `/v2/reg_onboard_abprop`, `/v2/exist`, `/v2/code`,
  `/v2/acverify_request`, `/v2/acverify`, `/v2/register`, `/v2/security`,
  `/v2/autoconf`, `/v2/autoconf_verifier`, `/v2/device_logout_send`,
  `/v2/device_logout_fetch`, `/v2/client_log`, `/v2/pre_pn_client_log`,
  `/v2/consent`, `/v2/challenge`, `/v2/passkey_auth`, and `/v2/wfs`.
- `LX/0k8;-><clinit>` decodes the base URL
  `https://v.whatsapp.net`. `LX/1y7;->A0T(LLL)` concatenates the base and
  path, giving URLs such as `https://v.whatsapp.net/v2/register`.
- `RetryingHttpClient` joins map entries with `&`. The case-20 lambda in
  `LX/Ifj;` URL-encodes names and, except for an explicitly pre-encoded set,
  values with UTF-8, producing `name=value` pairs. It POSTs the resulting
  bytes with `Content-Type: application/x-www-form-urlencoded` and an exact
  `Content-Length`.
- The same request builder can replace the clear form with an encrypted
  envelope. It uses an ephemeral key agreement plus AES-GCM and prefixes the
  encoded result with `ENC=`; encryption failure falls back to the clear form.
- `LX/Cdl;->A00(ZL)` adds the pre-registration key bundle `authkey`,
  `e_ident`, `e_keytype`, `e_regid`, `e_skey_id`, `e_skey_val`, and
  `e_skey_sig`. Static analysis establishes the bundle construction, not the
  exact first request on which every field is mandatory.
- The bundle helper sources `authkey` from the 32-byte client Noise static
  public key. `LX/CcG;` fixes `e_keytype` to byte `05`; `LX/1Lz;` encodes the
  registration ID as four-byte big-endian and signed-prekey IDs as three-byte
  big-endian. `LX/Hvq;->A04` encodes each byte string with Android Base64 flag
  value 11 (`URL_SAFE | NO_WRAP | NO_PADDING`) before form serialization.
- This proves public-bundle serialization for the comparison build. It does
  not establish accepted registration-ID ranges, signed-prekey generation or
  signature semantics; those remain behind the Signal backend gate.
- `KotlinRegistrationBridge` splits common request fields across three
  helpers. `A0T` appends `cc` and `in`; `A0U` appends `lg`, `lc`, `fdid`, and
  `expid`; `A0V` optionally appends `access_session_id`, then `id` and
  `backup_token`. Both UUID-backed fields serialize the UUID's canonical 16
  bytes with unpadded URL-safe Base64. `LX/Hvq;->A05` passes the raw device-ID
  and backup-token bytes through `LX/HNM;->A00`, which preserves RFC 3986
  unreserved bytes (`A-Z`, `a-z`, `0-9`, `-`, `.`, `_`, `~`), percent-encodes
  all other bytes as uppercase `%HH`, and marks the result pre-encoded before
  the surrounding form serializer runs.
- `KotlinRegistrationBridge->A09` constructs the generate-auth-code request
  at the path held by `LX/HSP;->A06` (`/v2/code`). After the three common
  helpers it appends required `token` and `method`, optional `context`, then
  tri-state `clicked_education_link`, `manage_call_permission`, and
  `call_log_permission`. `LX/Hvq;->A00` maps internal integer 0 to `false`, 1
  to `true`, and omits other values. The method next adds an optional
  `client_start_message` with unpadded URL-safe Base64, optional
  `advertising_id`, then `A0S` adds optional `login` and the decimal value of
  registration enum `type`. Finally it attempts `A0W`, the public key-bundle
  helper; if keys are unavailable it logs that fact and still sends the
  request.
- The required `token` is produced before that coroutine by
  `LX/HQO;->A00:LX/Hre;` calling `LX/Hre;->A01(Context, phoneNumber)`. The
  argument is the national number (`in`) without `cc`. The helper derives a
  64-byte key with `PBKDF2WithHmacSHA1And8BIT`, 128 iterations, the exact
  `package-name || selected hdpi about_logo.png` bytes as password, and the
  90-byte salt held by `LX/HQP;->A00`. It then computes HMAC-SHA1 over the
  first Android signing-certificate DER, MD5 of uncompressed `classes.dex`,
  and the UTF-8 national number, returning standard padded Base64 with
  Android's `NO_WRAP` flag. `registration::token` reproduces this bounded,
  version-scoped algorithm; `xtask apk-registration-token` extracts its APK
  inputs. This remains a comparison-build observation, not a cross-version
  invariant.
- The same caller obtains `fdid` from `LX/1GO;->AuB()`: on first use this is
  `UUID.randomUUID().toString()` persisted as `phoneid_id`. It obtains `expid`
  from `LX/1GR;->A03()`: absent an Android-derived source, this also falls back
  to a persisted random UUID. `LX/0wl;` generates and persists
  `access_session_id` with `UUID.randomUUID()`.
- `LX/I4D;->A0v(cc, in)` loads a phone-bound recovery token or generates one
  with `LX/00L;->A0G()`. `A0u` does the same for the shared backup token when
  preferences are empty. `A0G()` uses an AES key generator initialized to 160
  bits with the client's secure RNG, so both fresh fallback values are 20
  random bytes. `ComparisonRegistrationPrerequisites` now generates these
  UUIDv4 and 20-byte fallback values as one serializable, redacted unit for
  encrypted pre-request persistence.
- `KotlinRegistrationBridge->A08` constructs the phone-number completion
  request at `LX/HSP;->A0A` (`/v2/register`). After common fields it appends
  required `code`, optional binary `auth_response` using unpadded URL-safe
  Base64, then optional `context`, `method`, `advertising_id`, `login`, and
  decimal enum `type`, followed by the same optional public key bundle.
- `KotlinRegistrationBridge->A0D` constructs account security-code requests
  at `LX/HSP;->A0F` (`/v2/security`). After common fields it conditionally
  appends `code`, adds `reset` only when its string is non-empty,
  conditionally passes raw `wipe_token` bytes through the same uppercase
  RFC-3986 percent encoder/pre-encoded path, then adds optional
  `advertising_id` and the public key bundle. Its response parser treats
  `ok` as success and reads `login`; handled failures include `stale`,
  `blocked`, `too_many_guesses`, `temporarily_unavailable`, and `incorrect`.

### T2 — registration response envelope

- `KotlinRegistrationBridge` reads an HTTP result's `JSONObject` and handles
  the response statuses `ok`, `sent`, `fail`, and `attached` across its code
  request and phone-registration parsers.
- It branches on the reason tokens recorded in the comparison fixture,
  including `challenge`, `security_code`, `second_code`, cooldown/provider
  failures, and malformed-key/input cases. This establishes vocabulary, not
  the state transition appropriate for an unobserved server response.
- Cooldown and verification controls include `retry_after`, `sms_wait`,
  `voice_wait`, `flash_wait`, `wipe_wait`, `length`, `sms_length`, and
  `voice_length`. Credential/routing values such as `login`, `code`,
  `wipe_token`, `passkey_credential`, `appeal_token`, and
  `edge_routing_info` are treated as secret-bearing by the Rust boundary.
- No response was obtained from a server. The parser is comparison-scoped and
  cannot promote `registration.verification_steps` for the active profile.

### T3 — connection header and token dictionary

- `LX/1N7;->A04(L)` returns `57 41 05 03` for protocol version 5 and
  constructs `57 41 06 03` for version 6. The constructor selects version 6
  and writes this four-byte header before the Noise handshake (after any
  optional routing header), statically observing dictionary version byte 3
  in this comparison build.
- `apk-tokens` recovered 235 single-byte entries plus four 256-entry
  double-byte pages from `classes.dex @ 0x3b8d06`. The dictionary differs
  from the pinned web-v3 fixture at double-byte page 0 index 1, so it was not
  substituted for the pinned profile.
- `LX/1NZ;` maps every static field to its pattern string. In
  `LX/1N7;-><init>`, PQ mode `NONE` selects classic XX for a full handshake,
  classic IK for resume, and classic XXfallback after a failed resume. With
  PQ enabled/enforced, the protocol variant selects `XXkem`/`IKkem`,
  `XXkem-FS`/`IKkem-FS`, or `XXkemEph`/`IKkem2`, with the corresponding
  hybrid fallback pattern. `LX/19p;->A05()` reads the variant with `IKKEM` as
  its local default. The effective remotely configured PQ mode/variant still
  requires a runtime observation, so no single active pattern is claimed.

### T4 — certificate trust root

- `LX/Aw1;-><clinit>(V)` initialises the 32-byte `WhatsAppLongTerm1` root as
  `14 23 75 57 4d 0a 58 71 66 aa e7 1e be 51 64 37 c4 a2 8b 73 e3 69 5c 6c e1 f7 f9 54 5d a8 ee 6b`.
- `LX/1N7;->A03(LLLLL)` parses intermediate and leaf certificate details,
  requires the intermediate issuer serial to be zero, checks the serial link,
  binds the leaf key to the Noise server static key, verifies the leaf with
  the intermediate key, and verifies the intermediate with
  `WhatsAppLongTerm1`. This matches the reference-only root and issuer serial
  already used by the library, for this newer comparison build.

### T5 — chat endpoint ports

- `LX/1BH;->A00(I)` selects port 443 or 5222 at random unless a valid prior
  connection supplies another non-80 port. Other connection-sequence states
  explicitly try `g.whatsapp.net`, `g-fallback.whatsapp.net`, or an `e1`–`e16`
  edge on 443/5222; the fallback sequence also contains port 80.
- This identifies candidate socket ports in the newer build. It does not yet
  establish the active profile's TCP/WebSocket/TLS layering or the framing
  used on every connection-sequence branch.

Machine-readable metadata and findings are recorded in
`crates/whatsapp-api/fixtures/comparisons/whatsapp-2.26.37.70.json`.
