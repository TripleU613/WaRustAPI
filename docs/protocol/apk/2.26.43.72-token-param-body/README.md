# Token parameter body and selected translation rows

**Confirmed static**, APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
xtask DEX exports plus pinned ZIP/DEX derivation, not REA Evidence envelopes or
APK execution. Follows [header/schema gating](../2.26.43.72-token-param-header/README.md).

## The body is a packed pair array

Pcn.A02 readsPHU.A02 as parameter count, requires positive count, allocates count*2
ints and requests count*8 body bytes. A short read returns a failure/absent array,
not a partial translation. The successful branch flips a direct buffer, selects
LITTLE_ENDIAN and reads the int array. Errors and optional names/report generation
have separate branches and retained catches. One unsupported0xb9 rendering appears
at041e in the later reporting path; this bundle does not claim full Pcn execution
or derive selected body-loading/translation semantics from that unknown instruction.

All four packaged bodies have exactly count*8 bytes after their62-byte header.
The decoder verifies unique encoded config/parameter keys, sorted search partitions
and distinct config-key counts againstPHU.A00: universe1=635 configs/645 pairs,
2=22398/22458,4=2/72,6=38/38. PHU.A01 is used as the split point forPLr's two
binary-search partitions, viaPCc.A00 in the already retainedBPZ installation path.
For these packaged maps it equals the entire parameter count; this does not mean
other loaded map versions cannot use a second partition.

Each pair's second int encodes config key in bits12..31 and parameter ID in
bits0..11. The selected lookup returns the first int's signed high16 bits as the
translated slot. Other flag/detail bits of the first word are not token timing
values and are not fully interpreted here.

## Compiled config-key binding

NhI.A00 is the sole direct declared constructor caller found for09u. It passes
09o.A0B as the compiled config-key table;09u stores that argument inA07, whichPLr
reads.09o.<clinit> constructs unit2's config-key array by concatenating five
segments; the four selected config indices all fall in its first5000-entry
literal segment at payload0x0d00, referenced by instruction004e. The final outer
array places this segment's concatenation at unit index2.

The retained09o disassembly abbreviates large payload bytes. The derivation
therefore independently locates exact09o.<clinit> through DEX class_data/method
indices, reads its code_item and verifies the selected fill-array reference,
payload signature,width4 and5000-element count before extracting bytes directly.
It records DEX, payload and asset hashes; raw DEX and full arrays are not committed.
This is a bounded selected binding, not an interpreter proving every initializer,
reflection, shared-array mutation or subsequent runtime map replacement.

## Four token rows

| Property | Compiled config index | Config key | Parameter ID | Pair index | Translated slot |
| --- | ---: | ---: | ---: | ---: | ---: |
| 865 | 423 | 865 | 0 | 423 | 101 |
| 909 | 444 | 909 | 0 | 444 | 110 |
| 996 | 483 | 996 | 0 | 483 | 125 |
| 997 | 484 | 997 | 0 | 484 | 126 |

The selected packaged slots equal the original specifier slots. This confirms
identity translation for these four entries **when this same packaged mapping is
used**, not that every effective lookup always uses this mapping. Pair words and
hashes are retained in token-rows.json. The config-key/property equality here is
observed for these four selected entries, not assumed as a global mapping rule.

PLr.A00 returns the input unchanged when bit62 is set; the broader meaning of
that marker is not established here. Otherwise it obtains a
parameter ID using0Ep.A00, resolves config key by unit/config index, searches the
two partitions lexicographically by config key/parameter ID and reconstructs a
specifier using0Ep.A01. Missing cases use the sentinel slot65535. This does not
create a usable value or prove a later table read succeeded. The wrapperQAh also
has a silent-push path returning typed Long or the generated default; it can bypass
translation, so these rows alone do not resolve effective configuration.

## Reproduction and limits

```sh
python3 docs/protocol/apk/2.26.43.72-token-param-body/derive-token-rows.py --apk ../android-lab/apks/WhatsApp.1.apk > /tmp/token-rows.json
cmp /tmp/token-rows.json docs/protocol/apk/2.26.43.72-token-param-body/token-rows.json
```

The script pins the whole APK and previous asset identities, verifies all four
body sizes/sorted partitions/uniqueness, and requires exactly one selected row
per property. Exports use apk-smali --signatures --exceptions forPcn.A02,PLr.A00,
QAh.A01,0Ep.A00/A01,NhI.A00,09u.<init>,09o.<clinit>;apk-callers supplies the
constructor caller inventory. No raw binary asset or private account state is read
from outside the pinned APK.

Still open: dynamically loaded table/schema values, complete manager/flag choice,
translation versions across updates, cache readiness and effective00E resolution.
No Rust timing/admission, network request, identity reset or live account operation
changed. These are mapping coordinates, not evidence that the182-day Rust policy
matches the official client's configured timing or avoids restrictions.
