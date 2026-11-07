# MobileConfig response consumer gates

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution, account operation or network request.

## Consumer entry and caller rejection

Following the [storage preconditions](../2.26.43.72-mobileconfig-storage-preconditions/README.md),
the response method calls 0xae568 at 0x83184 with output storage and its parse
context. The consumer allocates a default 0x4c0-byte object through 0xb2ebc
**before** checking the root pointer and RequestData pointer. Either pointer
being null returns through 0xb22c4 without removing that output. Allocation is
not evidence that decoding succeeded.

For a nonnull root and RequestData, helper 0xcc370 compares the root tag to 3.
Tag 3 proceeds; tag 4 proceeds only when its word +0x10 is zero. Other selected
kinds assign `Invalid json root object` to context+0x10 and return. JSON kind
names remain unknown until the YAJL callback mapping is established. Empty
tag-4 admission through this first gate does not prove full response validity.

After the consumer returns, the caller checks context diagnostic length at
0x83198–0x831a4, then RequestData presence at 0x831a8–0x831ac. Either a nonempty
diagnostic or missing RequestData branches to 0x832d8. That branch logs
`JSONDeserializeError` and `Failed to parse delta response`, and calls the
previously traced result initializer 0x60484 at 0x83388 with first byte **10**,
second byte zero and an empty string. It copies decoded-object byte +0x60 into
caller-output byte +0x50 and takes cleanup. That byte's meaning is unknown.
The parser-null path previously shown to copy its diagnostic therefore has a
downstream error gate; parser null alone is not the actual predicate here.
Do not infer the result of a hypothetical null root with an empty diagnostic.

These are internal result bytes, not HTTP codes, Rust enums or server verdicts.
The error branch also calls logging/measurement helpers, including 0x89e8c;
their effects remain untraced. It bypasses the later 0x8d670 call, but this does
not establish universal rollback or absence of side effects.

## Separate flag result and subsequent processing

With no diagnostic and a nonnull RequestData, decoded-object byte +0x498 equal
to 1, byte +0x499 bit 0 clear, and byte +0x49a bit 0 clear take 0x83428. This
sets storage-manager byte +0x480 to 1 and constructs internal result byte **13**
at 0x8347c. It also copies decoded byte +0x60 to caller-output +0x50. Flag names
and result semantics remain unknown; no activation claim follows from the
manager-byte write. The allocator explicitly initializes all three decoded
flags to zero; the consumer body that can change them remains untraced.

Otherwise, the caller moves the decoded-object pointer into local ownership,
clears its earlier output slot, retains the request context pair and calls
**0x8d670 at 0x83210**. That is the next processing boundary. Its writes,
validation, persistence ordering and result propagation remain untraced.

## Reproduction and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-consumer-gates/inspect-consumer.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-consumer.json
cmp /tmp/mc-consumer.json docs/protocol/apk/2.26.43.72-mobileconfig-consumer-gates/consumer.json
```

The inspector reuses pinned member/ELF validation and checks selected BLs and
diagnostic strings. Its summaries are analyst transcriptions, not native tests.
LLVM windows retain caller gates, consumer entry/return, default initialization,
tag/length helpers and context cleanup. Nearest JNI_OnLoad headings are not
local function identities; these windows exclude most consumer and exception
paths. The context cleanup window is retained without a complete lifetime model.

rea-consumer.json retains complete REA Evidence
`ev_70317bd83c1a0da547155ad2747212d0442ce94f317a1f1463d8cf25c485563a`,
command `e scr.color=0; pd 34 @ 0xae568`, exit 0, untruncated, unchanged member
hash, with an entrypoint warning. This is a bounded static window. Provider
version and external inputs are not bound; annotations are not runtime reads.

**Remaining:** callback tag mapping; decoded-object type and field meanings;
consumer body; first lookup helper 0xc9de0 and key at global 0xdb8c8 (a raw
24-byte file read was all zero, not a resolved string or runtime value);
global initialization; 0x8d670 processing, persistence, retries, activation and
account ownership. No Rust runtime or saved account state changed.
