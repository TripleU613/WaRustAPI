# MobileConfig text records and field producers

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Static reads only; no native execution, account data or requests.

## A fallback text path constructs the record layout

The [native binary parameter reader](../2.26.43.72-mobileconfig-native-param-reader/README.md)
produces eight-byte entries, which are not themselves the 0x50-byte schema records.
A separate text path inside 0xbb28c constructs records in a keyed map. It reads
lines through 0xbc3d0, removes one trailing carriage return through 0xbc410,
compares a line to literal END, and branches on leading `?`, `*` or ordinary
content. Their complete grammar and input provenance remain untraced; none of
these marker characters identifies the network protocol by itself.

A `*` line has comma-separated header fields; the selected path copies its first
field (excluding the marker), updates loader fields +0x160/+0x178/+0x184 and
possibly its version word from subsequent fields. Missing selected fields have
distinct zero/default paths. New group-ID allocation at 0xbbaa4 uses the previous
loader+0x17c counter as +0x180, increments the counter, and records a mapping.
Existing groups can reuse prior mapping state. Thus a group ordinal is not simply
the wire config ID or the number written in a header.

Ordinary parameter lines split into comma spans and require a count from 2 to 4
on the selected path. The first span becomes a parameter string. The numeric
converter 0x81a24 receives the second span and arguments 1/16; the third, when
present, receives 4/16 and otherwise yields local flags value 4. Complete numeric
conversion semantics are not established merely from those arguments. Do not
call this a complete hex parser or invent defaults outside this branch.

The second parsed value is added to loader+0x178 with 32-bit arithmetic. The
lookup key becomes that accumulated word OR loader+0x184 shifted left 12, also
in 32-bit arithmetic. No masking of the accumulated word to 12 bits appears here;
OR must not be replaced by addition or assumed disjoint without upstream bounds.
Existing-key handling can follow a different path. On the new-record branch at
0xbbae0 it increments a count, updates group bookkeeping, then builds this layout:

| Record offset | Producer on selected new-record path |
| --- | --- |
| 0x00 | Copy of current group string at loader+0x160 |
| 0x18 | Copy of parameter string |
| 0x30 | Accumulated parameter word at loader+0x178 |
| 0x34 | Group ordinal at loader+0x180 |
| 0x38 | Same accumulated parameter word |
| 0x3c | Previous counter for the selected type, then that counter increments |
| 0x40 | Flags bits 2..4; values 6/7 branch away at 0xbbe58 |
| 0x44 | Local selected word w22; semantic name unproven |
| 0x48 | Boolean result of `(flags & 0x80000001) == 1` |
| 0x49 | Flags bit 6 |
| 0x4c | Wire config ID from loader+0x184 |

0xbe52c transfers the two temporary strings using move-shaped helper 0x5ce30
and copies the following 32 scalar bytes into the map record. The string helper
releases an existing long destination, transfers storage and clears the source
tag; it must not be treated as preserving the temporary string contents. This matches the field geometry used by the preceding
[grouping](../2.26.43.72-mobileconfig-schema-grouping/README.md),
[selection](../2.26.43.72-mobileconfig-schema-selection/README.md) and
[fragment/checksum](../2.26.43.72-mobileconfig-schema-fragments/README.md) traces.
It connects a producer to those fields without yet proving the map's final
conversion to the shared vector or the selected runtime text source.

Local w22 uses the loader child's +0x60 word when nonzero; otherwise the retained
code chooses 1 or 2 using arithmetic on the flags. Its exact domain meaning and
all branch conditions remain open. The +0x48 boolean is not flags bit0 alone:
the sign bit participates in the equality. Per-type counters are indexed from
loader+0x188; previous counter values become record slots. Counter initialization,
merging and persistence still need tracing. Flags bit5 enables additional work
that can install loader+0x48 as a pending object, later consumed by a `?` line;
that object's semantics are unresolved.

## Completion indicators are not interchangeable

The selected text loop has distinct END, empty-line, wrong-token-count and stream
termination paths. END sets loader byte +0x50 to 1 and clears its diagnostic
string. Empty lines write empty_line_while_parsing and set the byte to 0. Other
branches write wrong_token_size_while_parsing or wrong_token_size_while_parsing_param.
The loop began with failed_to_find_end_2 as its diagnostic, and stream termination
can also assign that byte from a local stream-state result. Thus byte +0x50 alone
is not established as a universal success flag. Full status/diagnostic handling,
partial insertion and rollback are unproven. Do not clear an error because a
single completion byte is set.

## Reproduction and limitations

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-text-records/derive-record-fields.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-text-fields.json
cmp /tmp/mc-text-fields.json docs/protocol/apk/2.26.43.72-mobileconfig-text-records/fields.json
```

The script reuses pinned ELF validation and selected call/string checks, then
projects scalar type/slot/flag fields over supplied synthetic inputs. It does not
parse a real map or execute target code. Out-of-domain type cases remain marked
unmodeled; Python results are not native error or rollback emulation. LLVM retains
the text loop, termination branches, record copy and CR trim, plus bounded
numeric-converter and header-adjustment entries. Nearest JNI_OnLoad headings do
not identify local functions. Retaining a region does not imply all its branches
have been semantically modeled.

rea-record-construction.json retains complete REA Evidence
`ev_82749bb1cefed2f8fbeefe6526284dbbf602991be037c18c0a845070f5bf117d`:
`e scr.color=0; pd 64 @ 0xbbae0`, exit 0, untruncated output, unchanged member hash,
with an entrypoint warning. It is a bounded construction window; LLVM retains the
wider source path. Register-relative annotations are not runtime memory.

**Remaining:** selected input file and real text content, numeric conversion and
complete grammar, existing-record merges, map-to-vector finalization, alternate
binary route, counter lifecycle, loader status/rollback and response persistence.
No Rust runtime or saved account behavior changed.
