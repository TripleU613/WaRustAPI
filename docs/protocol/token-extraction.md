# Checked APK token extraction and comparison codec

**Confirmed static, 2026-10-07:** APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [`1Rl.<clinit>` initializer](apk/2.26.37.71-core/token-dictionary-initializer.txt)
allocates a 236-entry single page, assigns index zero null, then allocates four
256-entry pages. String constants are assigned by actual `aput-object` indices;
the two resulting arrays are published with `sput-object` at 0b8e and 3b82.
`xmlstreamstart` is at method offset 0016 / classes.dex 0x3b9062.

The old extractor searched every byte for values resembling `const-string`,
without instruction boundaries or array indices. Operand bytes can look like
opcodes. Its 37.70 runtime fixture has four refuted page/index entries:

| Page/index | Historical heuristic value | Checked 37.71 assignment | Method offset |
| --- | --- | --- | --- |
| 0/1 | B3b | active | 0baa |
| 1/1 | LX/K5y; | dirty | 17c8 |
| 2/1 | LX/4Tw; | ptt_playback_speed_enabled | 23f8 |
| 3/1 | java.lang.Character | profile_picture | 2fdc |

These are extraction errors, **not confirmed release changes**. The available
37.71 APK establishes the corrected values. The raw 37.70 APK is unavailable
for re-extraction; its committed snapshot/fixture are retained with this caveat.
The intermediate new-release heuristic export also produced bogus values at
1/1 and 2/1. Both local pre-correction exports remain in ignored `lab/releases/`.
Do not use the historical dictionary to choose current wire token values.

`apk-tokens` now uses checked class/method/instruction traversal to find the
unique static initializer loading `xmlstreamstart`. A bounded evaluator follows
`const/4`, `const/16`, `const`, string/jumbo loads, typed string-array allocation,
indexed object writes and static publication. It requires one final return, one
single-page field and one double-page field belonging to that class, complete
slots and the supported 236 + 4x256 shape. Wrong registers/types/indices,
repeated writes, branches, payloads, unsupported opcodes, partial instructions,
multiple initializers and malformed traversed DEX structure refuse. It never
falls back to a longest plausible run. Bounds of 256 registers, eight arrays,
256 slots per array and 1 MiB initializer code are local tooling limits.
This is a straight-line initializer evaluator, **not a VM verifier** or proof
of reflective/native/dynamically modified dictionaries.

All 1,260 recovered entries match the independently held web-v3 fixture.
Compared with the historical runtime fixture, the only value differences are
the four rows above; provenance also changes. The comparison connection, lab
paths and stored binary-node inspection now use
`TokenDictionary::comparison_2_26_37_71`. The old helper remains available only
for exact historical reproduction. The pinned Android profile is not changed.
The new fixture is labelled `Derived` and names its static APK source; there is
no claim of fresh live validation from extraction alone.

Regression coverage includes out-of-order indexed writes, jumbo strings,
opcode-like bytes inside an integer operand, all truncated prefixes, unsupported
control flow and an invalid index. Codec tests decode independently constructed
`LIST_8/page/index` nodes for all four corrected entries and check encoding back
to those bytes, plus complete dictionary equality with the held reference.
No test copies expected values from the implementation being tested.
Since 2026-10-13 an exhaustive codec test also decodes every one of the 1,259
non-null entries from wire built from its fixture position and checks that the
token-first encoder reproduces those bytes (no entry is duplicated).

```sh
target/debug/xtask apk-tokens /path/to/WhatsApp.apk \
  --check crates/whatsapp-api/fixtures/tokens/comparison-2.26.37.71.json
```

Saved identities, ratchets, request IDs, reservations, prior encoded actions and
event markers are not migrated or deleted by this codec correction. A saved
node decoded with the old dictionary can retain its old projection; this change
does not silently replay consumed captures or clear projection markers. A live
session with the corrected codec and server-originated use of the affected
tokens require their own observed record.

**Observed live, 2026-10-07:** the existing saved account completed a bounded
30-second receive-only session with the checked comparison codec, reporting
`authenticated and active` and clean exit 0. Preflight had zero eligible chat
intents and zero queued actions. Existing sixteen action outcomes remained
unchanged (fourteen ok, one error, one unresolved sent two-factor action).
One automatic group-refresh action was added and settled ok, leaving seventeen
outcomes (fifteen ok, one error, one sent); no queued action/chat intent remained.
Supported ordinary frame processing worked; this does not establish that the
server exercised each corrected dictionary slot. Fourteen incoming stanzas
were classified unsupported: five ib, one group retry receipt, five psa,
one encrypt, one w:gp2 and one mex notification. These classifications remain
receive gaps; no user-message delivery or full protocol parity is claimed.
