# MobileConfig schema selection and paired refresh containers

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution or live request; modeled inputs below are entirely synthetic.

## Refresh constructs the two selection containers

The [query setup getters](../2.26.43.72-mobileconfig-query-setup/README.md)
can call 0xaa760 when a deferred byte is set. Its outlined helper chain preserves
the configuration holder and byte pointer. At 0xaa784–0xaa794, a byte compare-and-
exchange requests state 1 to 2 via 0x4c630. An observed previous value 1 enters
construction; 0 goes to return; other values invoke 0xaa018 and loop. That wait
helper and the full concurrent refresh lifecycle remain untraced.

On the selected construction path, configuration byte +0x194 selects mode 0 when
zero. Otherwise mode is 1 plus the low bit of byte +0x208. Configuration word
+0x190 is passed as the version comparison word. It allocates two 40-byte objects
and calls 0xab3e4 twice: first with side=1, then side=0, both with the version-gate
flag=1 and the same mode/version. The resulting objects replace configuration
+0x258's fields +0x18 and +0x20 through 0xa9980. The final local completion byte is
set to 1 before 0xa9f0c. A bounded entry into that release helper shows atomic
byte publication and wait machinery; it does not establish file persistence,
crash atomicity, account ownership, or every observer's activation order.

This mode calculation tests +0x208's **low bit**, whereas the preceding native
bool_opt_policy calculation tests that byte for **nonzero**. Do not silently
collapse those assembly conditions into one general boolean rule. Whether the
producer always restricts that byte to 0/1 is still unproven.

## Index selection is an ascending filtered walk

0xab3e4 stores the configuration pointer at object+0, initializes a vector at +8
and a record counter at +0x20, and obtains the schema table through 0xaa594.
It visits table indices 0,1,2,... in order, using a 0x18-byte table-entry stride.
For valid record ranges (0x50-byte stride), it skips:

- Table entry +0x10 low bit set.
- Zero records.
- When the supplied version-gate low bit is set, first record word +0x44 unequal
  to the supplied version word.

The remaining predicate uses the mode and side low bit. Let P be the helper
0xab5a8's predicate described below, and A mean every record byte +0x40 equals 1.

| Mode | side=1 admits | side=0 admits |
| --- | --- | --- |
| 0 | not P | none |
| 1 | not P | P |
| 2 | not A | A |
| Other | none | none |

Each admission appends the current index through 0xab65c, then adds that group's
record count to object+0x20. The append helper preserves order through both
available-capacity and reallocation paths. There is no sorting or deduplication
step in this selector; each table index is visited once. Therefore this selected
producer narrows the previous [fragment/checksum synthetic freedom](../2.26.43.72-mobileconfig-schema-fragments/README.md):
its generated index vector is ascending and unique. That does not establish that
no other producer can supply a different vector to those algorithms.

The selector reloads the table after append and checks the admitted index again;
an invalid index reaches BRK. Valid-table invariants and concurrency effects are
not replaced with forgiving behavior in the projection script.

## The selected special predicate has concrete string conditions

For the valid nonempty, unflagged groups reaching it, 0xab5a8 returns true only
for a one-record group whose first record:

- Has a stored group label starting with the three bytes `gk_`.
- Has word +0x30 equal to 1.
- Has stored parameter label at +0x18 equal to the empty string or `_`.

The equality path 0x70a28 obtains string pointer/length, uses strlen on the static
comparison literal, compares lengths and then bytes through 0x5df14/0x5d9f8.
Outlined literals at 0x25876 and 0x1b192 are empty and underscore respectively.
This is not merely an underscore-prefix test or any negative parameter ID test.
The helper has additional behavior for flagged/empty ranges outside this valid
selector domain; malformed-memory behavior is not modeled as a supported input.
We do not assign protocol meaning to record byte +0x40, word +0x44 or the numeric
mode beyond these observed conditions.

## Reproduce and remaining boundary

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-schema-selection/derive-selection.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-selection.json
cmp /tmp/mc-selection.json docs/protocol/apk/2.26.43.72-mobileconfig-schema-selection/selection.json
```

The script reuses pinned ELF/member validation, checks selected direct calls and
literals, and emits an analyst-derived selection matrix over synthetic valid
groups. It covers both sides, modes 0/1/2/other, table skip/empty/version gates,
a multi-record group and both special parameter-label alternatives. It does not
execute the APK, extract real schema tables, or prove server acceptance.

LLVM windows retain refresh construction, selector, predicate, vector append,
string comparison, atomic byte CAS and the bounded release-helper entry.
Nearest exported JNI_OnLoad symbol headings do not identify these local functions.
rea-selector-window.json retains complete REA Evidence
`ev_a551884b7359ea623457769bec6c1c7363b083227a53dcfefb97fdd8331ccd70`:
`e scr.color=0; pd 72 @ 0xab3e4`, exit 0, untruncated output, unchanged artifact
hash, with an entrypoint warning. This window ends inside selection; the LLVM
window retains the remaining selector and its helpers. Register-relative Rizin
annotations are not runtime memory observations.

**Remaining:** actual schema-table construction, mode/version/record-field
producers, initialization and refresh synchronization, request-data construction,
Java route consumption, and response parsing/file commit/cache activation. No
Rust runtime, sender, timing or saved account behavior changes are justified yet.
