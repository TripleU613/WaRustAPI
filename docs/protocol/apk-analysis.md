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
| `apk-callers` | Which Dalvik methods invoke an exact class/method pair | Whether the invoke is reached at runtime, **and whether any caller exists at all** — see the warning below |
| `apk-smali` | Bounds-checked structural bytecode and referenced symbols for selected methods | Server acceptance or the runtime path without control-flow analysis/capture |
| `apk-registration-token` | The exact comparison-build token derivation over APK-owned inputs and a controlled number | Server acceptance or compatibility with any other APK version |

## Two traps that produce silent empty output

Both of these look exactly like "the symbol is not in this APK". Neither is.

**The APK path is a positional argument, not a default.** Omitting it returns
nothing at all rather than an error:

```sh
cargo run --locked -p xtask -- apk-xref  <apk> --anchor skmsg
cargo run --locked -p xtask -- apk-smali <apk> --class 'LX/ADy' --method A0O
```

On 2026-09-22 this made `skmsg`, `SenderKeyDistribution` and `enc type` all look
absent from the comparison APK; with the path they return 6, several and 73
references. Before concluding a string is missing, check the invocation.

**`apk-callers` does not resolve virtual dispatch or constructors.** It reported
`0 caller(s)` for `LX/C9c->A00`, `LX/0hs->A0N` and `LX/BpZ-><init>`, all of which
are certainly called. **A zero result is not evidence of no callers**, and must
never be written into a document or the evidence ledger as one. To find callers
of an interface, virtual method or constructor, `apk-xref` a string constant used
near the call site instead.

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

## Reading protobuf field numbers (`apk-proto-fields`)

```sh
cargo run --release -p xtask -- apk-proto-fields /path/to/WhatsApp.apk --class 'LX/BZc;'
```

This tool derives numbers from the protobuf-lite descriptor, without assuming
encoded static field values were recovered. A class hands
`GeneratedMessageLite.newMessageInfo` an array of field names and an encoded
string pairing each name with its number and type. Every other way of getting a
number has produced a wrong answer here:

* **from memory** — `senderKeyDistributionMessage` was implemented as field 18
  (`liveLocationMessage`), matched nothing, and no group message decrypted for
  weeks;
* **from position in the name array** — the array holds the class's own
  bookkeeping strings first and the numbers are not contiguous, so counting gives
  `stickerMessage` = 22 rather than 26.

A wrong field number is self-consistent locally — our own encoder and decoder
agree — and unreadable by every real client, which is the worst failure mode
available.

The descriptor uses a ten-integer header and **variable-width entries**. Every
entry has a field number and type/flags; scalar presence bit `0x1000` adds a
presence index, while repeated fields do not. The earlier fixed-triple reader
was incorrect for mixed schemas. It could stop early and pair a partial prefix
with trailing names. The reader now validates the full declared field count,
header bounds/list/map/initialization counts, ascending wire numbers, presence
bounds and exact field-name count before printing any pairing. UTF-16 integer
continuations are decoded with overflow checks. `bitFieldN_` bookkeeping names
are excluded explicitly; oneof name mapping is refused rather than guessed.

**Confirmed, APK 2.26.37.71**: `com/google/protobuf/MessageSchema`'s
`newSchemaForRawMessageInfo` tests `0x1000` at `0262..026a`, limits presence
indices to scalar type <=17 at `026a..0272`, and reads the additional integer
at `0272..02c6`. `LX/BX5.dynamicMethod`'s literal at `0066..006a` mixes repeated
string fields with presence-bearing int64 fields. It now decodes completely:
`historyReceivers=1`, `oldestMessageTimestampInWindow=2`, `messageCount=3`,
`nonHistoryReceivers=4`, `oldestMessageTimestampInBundle=5`. The complete
`BZc` Message descriptor still decodes 110 fields. The APK runtime agrees with
the [official protobuf Java reader](https://github.com/protocolbuffers/protobuf/blob/main/java/core/src/main/java/com/google/protobuf/MessageSchema.java)
(accessed 2026-10-07); the APK offsets are the evidence for this build.
Literal APK fixtures test mixed entry widths and every truncated prefix;
additional fixtures test header corruption, presence bounds, integer overflow
and unsupported oneofs. No partial field pairing is printed.

**One trap worth naming.** The literal must be extracted honouring `\"`: the
encoding uses the raw character for field 34, which is `"`, so splitting the
disassembly line on quotes truncates there. That truncation is silent — a
110-field message decodes as a 34-field one and the names shift by 76.

Validated on `LX/BZc;` (`Message`) against every number this project already
knew independently: `conversation` 1, `senderKeyDistributionMessage` 2,
`imageMessage` 3, `extendedTextMessage` 6, `documentMessage` 7, `audioMessage`
8, `videoMessage` 9, `reactionMessage` 46. It then gives `stickerMessage` **26**
and `ptvMessage` **66**.

**Refuted wording (2026-10-07)**: protobuf-lite classes do not universally lack
field-number symbols. Current `DA8.AD7` at `0068..006c` reads
`BV6.BOT_HISTORY_SHARE_SYNC_METADATA_FIELD_NUMBER`. The tool reads descriptors,
not dex encoded static values; that implementation choice must not be presented
as absence of constants in the APK.


### Group-fetch double arithmetic names (2026-10-07)

**Confirmed/static, APK 2.26.37.71** (SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`):
`FetchTruncatedGroupsJob.A0G` 003c..0044 contains literal instruction bytes
`cd13 19014e40 ce13`. The [AOSP Dalvik table](https://source.android.com/docs/core/runtime/dalvik-bytecode)
identifies cd as `mul-double/2addr` and ce as `div-double/2addr`, both 12x.
Existing register/width decoding already handled them; the missing names
made the otherwise decoded operations look opaque. The correction changes
only mnemonic rendering. A literal APK regression checks both registers,
three instruction offsets/widths and every truncated prefix.

Reproduce with `cargo run --locked -p xtask -- apk-smali
~/Downloads/WhatsApp.apk --class FetchTruncatedGroupsJob --method A0G`.
The rebuilt tool rendered `mul-double/2addr v3, v1` at 003c and
`div-double/2addr v3, v1` at 0042. The surrounding constants establish the
memory-budget formula, recorded with its feature-cap unknown in
[groups](groups.md#dirty-refresh-request-and-inherited-dispatch-trace-2026-10-07).
[Scoped instruction evidence](group-refresh-request-evidence.txt) retains the
method context. All 32 xtask tests and strict all-target xtask Clippy passed.
This is offline tooling, not a runtime group-query capability or live validation.


### Empty method-filter diagnostics (2026-10-07)

**Confirmed/local tooling defect:** `apk-smali --class 9R2 --method C8h`
reported "no classes match" even though APK 37.71 contains 9R2's constructor
and its callbacks are inherited from AWW. The method visitor returns matching
bodies, so an empty result cannot establish class absence. The command now
reports "no method bodies match" and includes both active filters, preserving
its exit behavior. A rebuilt-tool smoke run on the actual APK returned empty
stdout and the expected two-filter diagnostic. 32 xtask tests and strict
all-target xtask Clippy passed. No flag or HTTP capability was introduced.

[Group response evidence](group-refresh-response-evidence.txt) retains the
actual constructor/base callbacks and identifies the current Cp2 ACK writer;
Cns is a stale 37.70 anchor. Full dump files and negative searches stay local.


2026-10-07: binary-operation 23x operands now cover the existing width range
0x90..0xaf, fixing named add-int without registers in APK 37.71
FetchTruncatedGroupsJob.A0G. The literal bytes at classes6.dex 0x673e42 plus
high-register/truncated fixtures verify formatting; mnemonic coverage is unchanged.
[Job partition and bounded round evidence](groups.md#batch-partition-and-bounded-follow-up-rounds-2026-10-07).
