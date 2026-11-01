# Static APK analysis with xtask (lab procedure)

All tools are Rust (`xtask`). The APK never enters the repository; keep it
under a path listed in `.gitignore` (for example `lab/`).

## Obtaining the APK

The owner publishes the current official APK as the asset `WhatsApp.1.apk` of
the GitHub pre-release `nothing_yet` ("Not a release - Just the WA APK"), so a
cloud session without the owner's workstation can fetch it. It is research
input, not a project release:

```sh
mkdir -p /tmp/wa-apk && gh release download nothing_yet \
  -R TripleU613/WaRustAPI -p 'WhatsApp.1.apk' -D /tmp/wa-apk
cargo run --locked -p xtask -- reference-manifest /tmp/wa-apk/WhatsApp.1.apk
```

**Confirmed 2026-11-01** (`xtask reference-manifest` on the downloaded asset):

| Field | Value |
| --- | --- |
| Package / version | `com.whatsapp` **2.26.43.72**, version code 264307238 |
| APK SHA-256 | `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e` |
| Signing certificate SHA-256 | `39:87:D0:43:…:5D:44`, the same certificate as 2.26.37.70 and 2.26.37.71 |

No protocol finding is attributed to 2.26.43.72 yet. Its token inputs are not
verified against the working registration recipe, which still derives tokens
from the 2.26.37.70 material. The asset can be replaced: check the hash and
re-identify the build before citing it.

Other builds with recorded evidence: 2.26.37.70 (token material; SHA-256
`ce746890…922e9`) and 2.26.37.71 (owner workstation `~/Downloads/WhatsApp.apk`;
SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`).

## Procedure

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

The tool locates a unique `xmlstreamstart` initializer through decoded DEX
instructions, then follows its typed array allocations, register constants,
indexed stores and static-field publication. Unsupported instructions, malformed
DEX, ambiguous initializers and incomplete arrays refuse extraction. There is
no raw-byte scanning fallback. APK 37.71 `1Rl.<clinit>` yields 236 single and
four 256-entry double pages. See [token extraction evidence](token-extraction.md).

The earlier raw-byte heuristic could mistake operands for string instructions.
Four historical comparison slots were refuted; the immutable 37.70 snapshot
remains for reproduction. The earlier 36.75 location/equality claim was produced
by that heuristic and has not been revalidated with the corrected extractor.

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

**Changed 2026-10-13:** `apk-proto-fields` now pairs names the same exact way
as `retained-proto-fields` (below): it follows the objects array through
static helper bodies it reads from the same APK, instead of taking field-name
literals in method order, and refuses a class it cannot follow, naming the
call. `--literal-order` prints the old method-order pairing for such a class,
labelled unverified. **Not yet run against an APK:** this session had no APK;
the simulator is tested on retained 2.26.37.71 bodies, and the APK adapter
(DEX method to followed form, helper lookup) on synthetic input only.

**Refuted wording (2026-10-07)**: protobuf-lite classes do not universally lack
field-number symbols. Current `DA8.AD7` at `0068..006c` reads
`BV6.BOT_HISTORY_SHARE_SYNC_METADATA_FIELD_NUMBER`. The tool reads descriptors,
not dex encoded static values; that implementation choice must not be presented
as absence of constants in the APK.


## Field numbers from retained bundles (`retained-proto-fields`, 2026-10-13)

```sh
cargo run -p xtask -- retained-proto-fields \
  docs/protocol/apk/2.26.37.71-own-device-media/document-descriptor.txt
```

For sessions without the APK. It reads a committed `dynamicMethod`
disassembly and **follows the code that builds the `newMessageInfo` objects
array** instead of pairing name literals in method order: it tracks registers,
`new-array` and `aput-object` slots, and runs static helpers whose bodies are
retained from the *same* APK (matched by the file's SHA-256 header and the
call's **full descriptor**; a call rendered with only its shorty, such as
`A1K(VLL)`, names no single overload and is not followed; if retained copies of
one helper disagree, the call is treated as not retained and the class is
refused). The leading `packed-switch` fans out, the method must have
exactly one `newMessageInfo` call, and exactly one case must reach it in
straight-line code, so no `MethodToInvoke` ordinal is assumed. Every slot must
be written once with a name or a class/verifier object, the first slots must
be exactly the descriptor's presence-word count of `bitFieldN_` names (and no
hasbit name may appear later), every other name must look like `field_`, and
the count must equal the decoded descriptor's, or the class is refused with
the reason. An array stored into another array or a field, or handed to a call
that is not followed, is treated as possibly written by unseen code. Any
instruction rendered without its destination register (older renders print
some moves, `move-result-wide` and arithmetic that way) and any unknown opcode
(`??`) refuse the path rather than leaving a stale register. A refusal names
the helper whose body is missing.
A followed body must start at offset 0 and be contiguous (an excerpt or an
elided instruction is refused), and a file without an `APK SHA-256` header
never matches a helper. All 2,695 retained methods met both conditions on
2026-10-13.
`--literal-order` additionally prints, under a header marked
`LITERAL ORDER, unverified`, the method-order pairing (what `apk-proto-fields`
did before this change) for a refused class.

**Why literal order is not enough (confirmed, APK 2.26.37.71):**
`BZB.dynamicMethod` (DocumentMessage) calls `LX/Ayp;->A1K`, retained in
`2.26.37.71-own-device-media/image-name-helper.txt`: it stores its argument
(`title_`) at objects[3] and the literal `fileSha256_` at objects[4]. Method
order has 20 names against 21 descriptor fields; any pairing that ignored the
mismatch would shift every name after `title`. Followed exactly, `fileSha256`
is 4, `mediaKey` 7, `fileName` 8 and `caption` 20.

**Decoded exactly from retained files** (descriptor and every name accounted
for; `xtask` tests pin BZB and BZK). The last column lists every helper the
path followed; some, such as `6al.A1W`, write only `bitField0_`:

| Class | Message | Fields | Helpers followed |
| --- | --- | --- | --- |
| BZc | Message | 110 | 6al.A1W |
| BZD | ImageMessage | 29 | 6al.A1W, Ayp.A1K |
| BZB | DocumentMessage | 21 | 6al.A1W, Ayp.A1K |
| BZE | VideoMessage | 30 | 6al.A1W, Ayq.A12 |
| BZ0 | AudioMessage | 16 | 6al.A1W, Ayq.A12, Ayq.A13 |
| BZK | Conversation (history) | 59 | Ayl.A1b, Ayn.A1O |
| BY9 | PreKeySignalMessage | 8 | 6al.A1W |
| GsB | ADVDeviceIdentity | 5 | 6aj.A1U, 6al.A1W, Ayj.A1J |
| 6tK | UserReceipt | 6 | — |
| BSo | PollCreation option | 2 | 6ap.A1b |
| BP2 | (attestation payload) | 1 | 6aj.A1V, 6ap.A1b |

BZK reproduces, mechanically, the manual pairing in
[token-import](token-import.md#field-numbers-and-a-decoder-limitation) (id 1,
tcToken 21, tcTokenTimestamp 22, contactPrimaryIdentityKey 23,
tcTokenSenderTimestamp 28, identityVerification 63). Where both readers decode
a class (BZc, BY9, GsB, 6tK, BSo, BP2) their output is identical. **Corrected
the same day** after an independent review: BSp (PollEncValue) was listed as
exact, but its only retained copy renders the `6ap.A1b` call by shorty, so it
is now refused (literal order still pairs `encPayload` 1, `encIv` 2,
unverified); one of the retained BZc copies is refused for the same reason
while another, with full signatures, decodes.

**Refused, with the body to retain next time the APK is available** (these
decode only under `--literal-order`, unverified, or not at all):

| Class | Message | Missing body |
| --- | --- | --- |
| BZb | ProtocolMessage | 1xi.A1M receives the array |
| BW7 | DisappearingMode | 1xi.A1M receives the array |
| BZY | WebMessageInfo (history) | Ayj.A1I receives the array |
| BYa | PollCreationMessage | Ayj.A1K receives the array |
| BZC | StickerMessage | Ayq.A11 receives the array (literal order: 22 fields, 13 names) |
| BUs | DeviceSentMessage | Ayp.A1Z creates the array |
| BV3, BWY, BZa, Grz, BWI, BWH | KeepInChat, SignalMessage, MessageKey, ADVSignedDeviceIdentity, Reaction, PollUpdate | Ayp.A1a creates the array |
| BQ2 | one field, literal order `messageSecret` 3 | 6ap.A1a creates the array |
| BSp | PollEncValue | its `6ap.A1b` call is retained only as a shorty |

BWI (reaction) and BWH (poll update) have 4 descriptor fields against 3 method
literals. **Inferred:** the fourth name comes from `Ayo.A1P`, which receives the
array; the array itself comes from `Ayp.A1a`, also not retained, which could
pre-fill a slot as `6ap.A1b` does with `bitField0_`.

**Cross-check of this project's encoders (2026-10-13, local):** every
`prost` field whose struct corresponds to an exactly decoded class agrees with
it: `transport/media.rs` `Image`/`Audio`/`Video`/`Document`,
`transport/image_upload.rs` `ImageMessage`, `transport/voice_note.rs`
`AudioMessage`, `signal/wire.rs` PreKeySignalMessage (including
`kyberPreKeyId` 7 and `kyberCiphertext` 8), `proto/wa_adv.rs`
`AdvDeviceIdentity`, and `transport/polls.rs` `PollOption`. Structs whose class
is refused above (`KeepInChatMessage`, `DeviceSent`, SignalMessage,
`AdvSignedDeviceIdentity`, `PollCreation`, `EncValue`) agree with the
literal-order pairing only, which is weaker evidence. **Corrected:** the
sticker (`transport/media.rs` `Sticker`) has no retained pairing at all: even
literal order refuses BZC (22 fields, 13 names), so its tags rest only on the
earlier APK-session trace through `Ayq.A11`. The comparison matched struct
fields to descriptor names by hand-checked correspondence; it is not an
automated test.

**Pin in chat (edit thread):** BZc's exact decode gives `pinInChatMessage` = 63
(type `BVD`, from the retained field inventory). That is the envelope field
only: the `BVD` (PinInChatMessage) descriptor and its C2S type enum are not
retained, so pin/unpin sending stays blocked on the APK. See
[unknowns](unknowns.md).

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
