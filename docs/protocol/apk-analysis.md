# Static APK analysis with xtask (lab procedure)

All tools are Rust (`xtask`). The APK never enters the repository; keep it
under a path listed in `.gitignore` (for example `lab/`).

```sh
# 1. Obtain the candidate build, then verify it before treating it as the
#    reference. The /current/ URL is mutable.
curl -sSL -o lab/WhatsApp.apk https://www.whatsapp.com/android/current/WhatsApp.apk
cargo run -p xtask -- reference-manifest lab/WhatsApp.apk \
    --check crates/whatsapp-api/fixtures/reference-manifest.json

# 2. Dump every dex string constant (449k for 2.26.36.75) and search it.
cargo run -p xtask -- apk-strings lab/WhatsApp.apk > lab/strings.txt
cargo run -p xtask -- apk-strings lab/WhatsApp.apk --contains Noise_ --contains whatsapp.net

# 3. Recover the ordered binary-node token dictionary from bytecode and
#    compare it with the committed fixture.
cargo run -p xtask -- apk-tokens lab/WhatsApp.apk \
    --check crates/whatsapp-api/fixtures/tokens/web-md-v3.json

# 4. Locate string users and inspect only the relevant Dalvik methods.
cargo run -p xtask -- apk-xref lab/WhatsApp.apk \
    --anchor v.whatsapp.net --anchor authkey --anchor e_skey_sig
cargo run -p xtask -- apk-smali lab/WhatsApp.apk --class 'LX/HpO;'
cargo run -p xtask -- apk-callers lab/WhatsApp.apk \
    --class 'Lcom/whatsapp/registration/core/http/KotlinRegistrationBridge;' \
    --method A09

# 5. For a controlled test number only, reproduce the inspected comparison
#    build's /v2/code token. This prints a phone-bound credential to stdout.
cargo run -p xtask -- apk-registration-token lab/WhatsApp.apk \
    --country-code 1 --national-number 5551234567
```

If step 1 reports a manifest mismatch, do not replace the pinned manifest or
token fixture. Record the candidate by exact version and sha256 as a separate
comparison. On 2026-09-17 the URL had already advanced from pinned 2.26.36.75
to 2.26.37.70; that comparison is documented in `decompilation-log.md`.

## What each result proves

For the exact 2.26.37.70 comparison, `xtask apk-research-bundle` exports a
reproducible, media-free selection of protocol code and metadata under a
2 MiB total limit. See the [bundle guide](apk/README.md) for regeneration,
verification, and decoder limitations. This snapshot is separate from the
active profile's fixtures.

| Tool | Proves | Does not prove |
| --- | --- | --- |
| `reference-manifest` | Package, version name/code, SDK levels, APK hash, v1 signing certificate fingerprint | That the server accepts this build today |
| `apk-strings` | A literal is compiled into the client | How, when or whether it is sent on the wire |
| `apk-tokens` | The ordered dictionary initialised by the client | The dictionary *version byte* sent in the header/prologue |
| `apk-xref` | Which Dalvik class and method loads an exact string | The runtime branch taken or values assembled without string literals |
| `apk-callers` | Which Dalvik methods invoke an exact class/method pair | Whether the invoke is reached at runtime |
| `apk-smali` | Bounds-checked structural bytecode and referenced symbols for selected methods | Server acceptance or the runtime path without control-flow analysis/capture |
| `apk-registration-token` | The exact comparison-build token derivation over APK-owned inputs and a controlled number | Server acceptance or compatibility with any other APK version |

## How `apk-tokens` works

Dalvik initialises string arrays with runs of `const-string` (opcode `0x1a`,
`1a AA BBBB`) / `const-string/jumbo` (`0x1b`) instructions. The tool scans
each `classes*.dex` for runs of plausible tokens separated by no more than 64
bytes, anchors on the stream-control token `xmlstreamstart`, stops at the
first repeated string (a dictionary has no duplicates), and splits the run
into the 235-entry single-byte page (index 0 is the empty token) and
256-entry double-byte pages. It decodes no other bytecode. For 2.26.36.75
the run is in `classes2.dex @ 0x126a96` and is identical to the web v3 list.

## Findings recorded so far

See `client_observations` in `crates/whatsapp-api/fixtures/reference-manifest.json`
and the `Observed (client, static)` rows of `evidence-ledger.md`.

## Token material archive

When reconstructing token material from an unpacked APK tree and resources from another archive:

```sh
cargo run -p xtask -- apk-token-material \
    --from-dir <unpacked-2.26.37.70-tree> \
    --logo-from ~/Downloads/WhatsApp.apk \
    --out lab/whatsapp-2.26.37.70-token-material.zip \
    [--verify-config <request.json>]
```

Copies `AndroidManifest.xml`, `classes.dex`, `resources.arsc` (if present), every `META-INF/*.DSA|*.RSA|*.EC`, and the first available logo candidate from `--logo-from` into a stored (uncompressed) ZIP. The output file is created with mode 0600 and refuses to overwrite existing files. With `--verify-config`, reads a `LabRegistrationConfig` JSON, derives the token from the output archive, and verifies it matches the config's token field.

## Next static targets for the pinned profile

- Re-run the newer-build `WA 06 03` and registration URL/body findings
  against the exact pinned 2.26.36.75 APK.
- Which Noise pattern the client selects for a fresh primary registration
  (classic XX vs ML-KEM512 hybrid) and the certificate trust root bytes.
