# Native metrics snapshot validator — APK 2.26.43.72

**Confirmed static plus observed synthetic ARM64 emulation.** This checks the
local snapshot validator, not metrics network requests or server responses.
Reconstructed libwhatsapp.so SHA256:
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Reconstruction evidence](../2.26.43.72-superpack-reconstructed-members/README.md)
binds the image. Selected validator instructions executed only in Unicorn with
synthetic objects; no Android runtime, saved account state, target I/O or live
traffic. These are local experiments, not REA Evidence envelopes.

## Four groups, exact ranges and error priority

The [metadata recovery body](../2.26.43.72-metrics-native-metadata-file/README.md)
calls **0x71fc84** after decoding the fixed record. It iterates selectors 0..3
through helper **0x71fd70**. The exact byte jump table selects group pointers
from snapshot-state offsets **0x30, 0x28, 0x18, 0x20**, in that order. These are
local group selectors; their complete relationship to channel policy remains
unproven. Each group supplies four signed 32-bit values via getters:

| Raw group field | Accepted values | Failure code |
| --- | --- | --- |
| A, offset 0x18 | 0..2147483647 | 9 when negative |
| B, offset 0x1c | 0..2147483647 | 10 when negative |
| C, offset 0x20 | 1..65535 | 11 below 1; 12 at least 65536 |
| D, offset 0x24 | 1..65535 | 13 below 1; 14 at least 65536 |

The native comparisons run through every check of a selected group. Later
failing checks overwrite that group's earlier error code. It then stops at the
first group with a nonzero result. Passing all four groups returns zero. These
codes are native local validation results, not HTTP/stanza errors, server rate
limits or account restriction signals. Full semantic names and authority of the
fields remain unknown; no Rust counter limits are changed from this evidence.

If given a nonnull error-output pointer, the function may construct/store an
MCFError for the selected failure. The synthetic run passes a null error pointer,
so error-object contents and lifetime have not been exercised. Null or invalid
group pointers reach separate abort guards, also not exercised by these fixtures.

## Rejected recovery is replaced locally

In the selected recovery continuation, a missing/invalid snapshot reaches
constructor **0x71fb58**. After validation, a nonzero result releases the rejected
snapshot and constructs a replacement; zero retains the decoded snapshot for
return. The selected constructor creates four default groups and a dictionary;
its outlined argument setup includes A/B zero and C/D one. Additional fields and
all constructor dependencies are not fully interpreted here. This is a local
metrics snapshot fallback, not authority to reset credentials, ignore cooldowns,
or replay pending message/request intents. No saved user state was modified.

## Bounded execution evidence

check-validator.py loads the pinned ELF's PT_LOAD bytes into Unicorn 2.1.4,
applies supported AArch64 RELATIVE/ABS64/GLOB_DAT/JUMP_SLOT relocations and
supplies synthetic snapshot/group pointers. It executes the actual entry at
0x71fc84 with a return sentinel. Only called MCFRetain/MCFRelease are accepted:
retain returns the synthetic pointer and release is a no-op stub. Any other
called import or target interrupt stops the run. The model does not reproduce
real MCF object allocation, reference counting, JNI, error creation or concurrency.

**43 cases passed**, with a 100000-instruction / one-second engine bound per
case. Cases cover each group's six failure categories, signed maxima for A/B,
65535 acceptance for C/D, a valid baseline, within-group error precedence and
first-failing-group behavior. Exact synthetic inputs, expected/native results and
called imports are retained in validator-results.json. No actual metadata files
were parsed or altered by this harness, and no account operations were involved.
This provides bounded native-code behavior evidence, not exhaustive malformed
input or whole-recovery execution coverage.

## Reproduction and remaining work

Run `python inspect-validator.py RECONSTRUCTED_LIBWHATSAPP.so` for section-mapped
byte hashes, selector and range declarations; stdout must match
validator-boundaries.json. Under the isolated Unicorn venv, run
`python check-validator.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
validator-results.json. Both reuse the adjacent digest-checked image resolver.
Assembly reproduction uses Rizin `pD SIZE @ VA` from the retained windows.
The validator, selector, default constructor and recovery continuation are
complete selected bodies; small field/register helpers are retained as bounded
windows. [Section mapping](../2.26.43.72-metrics-native-event-routing/elf-sections.txt)
and [Rizin version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt) apply.

Next trace event buffer ownership, serialization, flush/send triggers and replies.
The metrics manager's broader persistence/termination policy and native transport
remain unfinished. No Rust behavior, HTTP route, telemetry, sender or retry added.
