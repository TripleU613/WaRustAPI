# MobileConfig schema fragments and ordered checksum

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native code executes. Synthetic projections below do not contain account data.

## Schema fragment tokens preserve selected order

The preceding [query setup trace](../2.26.43.72-mobileconfig-query-setup/README.md)
calls 0xab728 to build a vector of strings, then 0xadac4 to wrap those strings
for selected query_hash appends. 0xab728 resolves a schema table through 0xaa594,
walks the container's 32-bit selected indices from +0x08/+0x10, and indexes
0x18-byte table entries holding record ranges. An unsigned index outside the
available table count reaches BRK at 0xab8e4; this is not a silent omission.
The schema-table getter itself can initialize state; its complete lifecycle is
still untraced.

A selected range produces one fragment. The first record's word at +0x4c supplies
its group/config token through 0xc150c:

- Unsigned values 1 through 1048575 become unsigned decimal strings.
- Other 32-bit values use the stored string at the first record's base address.

Then each 0x50-byte record appends a literal colon and its parameter token through
0xc14e4. Its +0x30 word is interpreted as signed int: values other than -1 and
less than 16384 become signed decimal strings; -1 and values at least 16384 use
the stored string at record+0x18. The assembly has no general nonnegative check:
-2 follows the decimal branch. Do not broaden the -1 fallback into all negatives.
The +0x18 temporary pointer adjustment followed by +0x38 makes the full record
stride 0x50. The source does not sort or deduplicate selected indices here.

The wrapper 0xadac4 streams `[`, each fragment surrounded by literal double
quotes and separated by commas, then `]`. Its selected string-write helper
0x5e4b8 calls __put_character_sequence on the stored string bytes. No escaping
pass appears on this selected wrapper path. That does not prove upstream labels
can contain arbitrary characters or authorize injecting unvalidated labels.

## The checksum is ordered 32-bit arithmetic

0xab954 resolves the same table and traverses selected indices and records in
source order. Its accumulator starts at zero. For each admitted record, let
C be the word at +0x4c and P the word at +0x30. W-register arithmetic gives:

```
term = (P + 31*C + 16337) mod 2^32
acc  = ((acc * 397) XOR term) mod 2^32
```

The disassembly computes P-C followed by adding C shifted left by five, hence
31*C; it does not use 32*C. The accumulator is returned as a 32-bit word.
0xad9a0 always computes the first container's checksum. If the second container's
+0x20 word is nonzero and the supplied flag's low bit is clear, it also computes
that checksum with the same optional filter and XORs both results. It formats the
result with to_string(unsigned int), matching param_list_checksum's decimal value.
This is not a cryptographic hash and must not substitute for other hash fields.

Both fragment and checksum traversal optionally apply 0x9c754 to the first
record's +0x4c word **before the range loop**. A false lookup skips the whole
selected range. 0x9c754 reduces the pointer result of 0x9c974 to a boolean;
the retained lookup checks a stored integer key. The synthetic model takes the
lookup's admitted-ID set as input instead of pretending to reconstruct its bucket
state. It does not apply a separate filter to each later record. Record ranges
must satisfy the caller's invariants: this path can dereference the first record
before an empty-range check. The model refuses empty groups rather than claiming
native malformed-input behavior is a supported API.

## Reproducible synthetic projection and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-schema-fragments/derive-fragments.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-schema-derived.json
cmp /tmp/mc-schema-derived.json docs/protocol/apk/2.26.43.72-mobileconfig-schema-fragments/derived.json
```

The script reuses pinned ELF validation, checks selected call targets and delimiter
strings, and projects analyst-created groups. derived.json includes original,
reversed and duplicated selections; first-record filtering; signed token
boundaries; empty selection; and wrapping arithmetic. These are **analyst-derived
examples**, not execution of APK code, recovered real schema values or proof of
server acceptance. It models the selected numeric/string branches on valid
nonempty groups and supplied lookup outcomes. Actual table/selection/label
producers remain unresolved, so its outputs are not ready-to-send requests.

Bounded LLVM windows retain both loops, wrapper, token helpers, filter lookup,
outlined helpers and the checksum combiner already retained in the prior bundle.
The adjacent formatting function beginning 0xadbc4 was inspected during discovery;
followup-formatting-discovery.txt preserves that bounded read without claiming its
untraced enum meanings. Nearest JNI_OnLoad symbol headings do not identify locals.

rea-checksum-window.json retains complete REA Evidence
`ev_d9c19840aeb97eb9f46828bacf4227a43957cdf178b15c8647df964b29c9b82e`,
`e scr.color=0; pd 52 @ 0xab954`: exit 0, untruncated output and unchanged member
hash, with an entrypoint warning. It is a bounded instruction window, not complete
runtime coverage. LLVM independently presents the complete selected checksum loop.

**Remaining:** actual schema-table and index producers, label validity, query-mode
selection, remaining hash functions, Java route consumption and response
persistence/cache activation. No Rust runtime sender, timing or account state
changes are justified yet. Next follow the producer of selected schema indices
and the request-data constructor to connect these algorithms to real inputs.

Follow-up: [paired schema selection](../2.26.43.72-mobileconfig-schema-selection/README.md)
now identifies one ascending, unique index producer and its filtering rules.
The synthetic reversed/duplicated fragment examples above remain algorithm
projections, not selections emitted by that specific producer.
