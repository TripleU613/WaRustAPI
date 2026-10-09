# Scheduled metadata deadline arithmetic

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-scheduled-deadline.json),
[complete methods](apk/2.26.37.71-scheduled-deadline/targets.json).
This follows the [final after-hook map](text-final-after-hooks.md); no account
was opened and no scheduling/receipt runtime capability was added.

## Decoder correction and evidence

**Confirmed:** the developer disassembler and research exporter now name DEX
21c opcodes **0x61 sget-wide** and **0x68 sput-wide**. Their field references,
register operands and instruction widths were already rendered; the mnemonic
lookup was missing. This was a limitation in our tooling. The instruction
semantics and eight-bit register operand follow the
[AOSP bytecode table](https://source.android.com/docs/core/runtime/dalvik-bytecode).
It does not execute static initializers or turn all unknown instructions into Java.

The regression uses literal 37.71 classes7.dex bytes:

- D7g.CHJ at absolute **0x75e962**, method **016e**: `61 00 53 60`,
  followed by `71 40 0f 17 32 10` (the existing A18 invocation).
- CBc.<clinit> at absolute **0x3a582c**, method **0010**:
  `68 00 53 60`, followed by `0e 00` (return).

Field pool index **24659** resolves to **CBc.A00:J** in that actual DEX.
The minimal unit fixture deliberately leaves field/method pools unresolved and
checks both mnemonic/operand rendering and the following instruction boundary.
A separate synthetic register-254 operand checks the complete eight-bit address,
not a fabricated APK instruction.

## What the newly readable path establishes

**Confirmed static:** CBc.<clinit> loads TimeUnit.DAYS and constant **30**, calls
`toMillis`, then stores the result in CBc.A00:J. D7g.CHJ loads **D6e.A00:J**
at **00ee**, writes it as **scheduled_time**, then retains that pair through the
selected SQL path. At **016e** it loads CBc.A00; **6ak.A18(JJ)** adds the two
longs and boxes the result without another branch. The boxed sum is passed to
**DGf.A03**, together with the original D6e timestamp, key, bytes and 0UX.A03.
Thus this bounded path supplies **scheduled_time + 2,592,000,000 milliseconds**
as a selected argument. It does not derive the sum from message chat_row_id;
that earlier register value was overwritten at 00ee.

**Inferred:** this argument appears to be a deadline associated with scheduled
conditional-reveal storage. **Unknown/unported:** its complete meaning and
validation in DGf.A03, timestamp origin/unit guarantees at all producers,
overflow handling outside the retained helper, actual server enforcement,
expiry/deletion workers, receipt-user admission and cross-release parity.
A 30-day calculation in this method is not proof of a universal server expiry.

## Reproduction and structural comparison

Use the existing `xtask apk-research-bundle` command with the recipe above, then
`--check` against its output; existing `apk-smali` can inspect each exact class
and method. `apk-research-diff` accepts the old final-after-hook bundle and this
new bundle without loading an APK. Both are the **same APK hash**: the changed
D7g topic is a renderer correction, not a protocol or release change. The other
24 removed/two added topics reflect the different selected scope. D7g instruction
text is otherwise identical after substituting the single corrected mnemonic;
Rust owner metadata additionally names the decoder. Historical evidence remains
unchanged. The ordered after-hook index now links D7g to this corrected source.

## Validation

Scheduled-decoder validation: 827 workspace/all-feature tests, strict Clippy,
strict rustdoc, formatting, CLI reference and API/library-symbol checks passed.
Exact three-topic export (eight files/36,973 bytes), cached self-diff and the
single-mnemonic comparison passed. Hygiene checked 2,097 local links and 103 APK
bundles. No live account test ran.
