# MobileConfig storage response preconditions and parse boundary

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Static reads only; no native parser, account operation or network request runs.

## Selected early-result precedence

The [resolved storage response method](../2.26.43.72-mobileconfig-storage-receiver/README.md)
0x82c4c reaches these checks in order. Internal result bytes are not HTTP status
codes or established Rust enums.

| Gate | Selected result construction |
| --- | --- |
| RequestData availability helper 0x880f8 returns false | Diagnostic Invalid StorageManager; result byte 2 |
| Response string has zero length | JSONDeserializeError / Attempted to persist without JSON; result byte 3 |
| Supplied selector equals 2 and context word +0x70 equals 5 | EPAndEmptyMetaSchemaError / Empty meta schema when handling EP response; result byte 12 |

Each selected branch calls initializer 0x60484 with its first byte, second byte
zero and an empty string, then clears caller-output byte +0x50 before returning
through common cleanup. The initializer sets multiple additional fields to zero
and copies the supplied string. Other logging/measurement virtual calls precede
these returns, so this does not claim the branches have no side effects. They
bypass the later parser call at 0x83008; file commit and cache behavior are not
established by a result byte alone. Numeric selector/context meanings are left
unproven despite the EP diagnostic naming. The previously retained updater call
at 0x7e69c supplies w3 through cset-eq (0 or 1), so reachability of the selector-2
arm from that particular call site is not established; other callers may differ.

## Availability is weaker than validated schema contents

The helper uses the RequestData pointer stored at storage-manager+0x98. It returns
true if RequestData+0x258's first source pointer is nonnull. It does **not** check
that vector's record count or validate its contents on that branch. Otherwise it
accepts nonempty strings at RequestData+0x130 and +0x148 together, or falls back
to the previously retained alternate-state predicate 0x5fb64 on RequestData+0x10.
Thus nonnull shared record ownership, available path/hash strings and alternate
state are different admission routes; none independently proves server acceptance
or that derived grouping/selection has completed. The first string's producer and
precise hash semantics still need tracing.

The byte-projection script takes the alternate predicate result as input rather
than inventing the external cache/state it represents. Its cases show precedence:
unavailable data wins before empty response; empty response wins before the 2/5
gate. Source-pointer presence alone can proceed even without path/hash strings.
Those examples are analyst transcriptions, not execution of the native helper.

## Parser outcome is not identical to early rejection

After these gates, the method sets up a parse context, derives a local numeric
value from selector/context, and calls the earlier [YAJL adapter](../2.26.43.72-mobileconfig-json-adapter/README.md)
0xc950c with a 50-byte diagnostic buffer. It stores the returned root pointer.
A null root copies the parser diagnostic into the context at 0x83080, but does
not immediately take the early result-byte-2/3/12 paths. It continues through
0x8308c into later processing whose validation and failure propagation remain
untraced. A nonnull root whose numeric tag is not 3 also joins that later path;
do not claim this selected check safely rejects every wrong JSON shape.

For tag 3 and a zero word at root+0x18, it sets a local marker and reaches a
separate result-byte-1 construction at 0x830cc, then helper calls, output marker
clear and cleanup. This report does not assign a JSON kind to tag 3 without the
callback mapping or treat that internal result as configuration activation.
Nonempty tag-3 content follows further branches. The retained code also contains
a defensive empty-input marker path even though the earlier gate handles the
selected zero-length response route.

At 0x83110, later processing begins by copying a string from the request context,
retaining its word +0x70 and allocating a shared object before calling 0xae568.
This is the next concrete boundary. Parser null, empty content, wrong root kind,
application rejection and file persistence must stay distinct until that path is
resolved. No retry, rollback or commit semantics are inferred from the entry.

## Reproduction and retained evidence

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-storage-preconditions/inspect-preconditions.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-storage-preconditions.json
cmp /tmp/mc-storage-preconditions.json docs/protocol/apk/2.26.43.72-mobileconfig-storage-preconditions/preconditions.json
```

The inspector reuses pinned ELF/member validation, checks selected BLs and
strings, then emits qualified synthetic early-result cases. LLVM retains the
precondition/parse-context region, availability helper, result initializer and
outlined helpers. Nearest JNI_OnLoad headings are not local function identities.
The retained region is not complete response or exception coverage.

rea-storage-gates.json retains complete REA Evidence
`ev_e24454f2da3dc1f35612d7055c32c6034b462d60490b20c9cc69f4c71d69950c`:
`e scr.color=0; pd 44 @ 0x82d14`, exit 0, untruncated output, unchanged member hash,
with an entrypoint warning. It is a bounded precondition window; LLVM retains the
additional early branches and parse handoff. Register-relative annotations are
not runtime memory observations.

**Remaining:** selector/context producers, YAJL callback root-kind mapping,
0xae568 and later validation/processing, application result propagation, file
commit ordering, retries, cache activation and account ownership. No Rust runtime
or saved account state changed.
