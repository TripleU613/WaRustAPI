# Node serializer content and size boundaries — APK 2.26.43.72

**Confirmed selected static**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence or runtime observations.
No live account operations.

## Confirmed mismatch: zero children are no content

1r2.A01 at0000..0026 counts children only if the array is nonnull AND
nonempty. A nonnull byte array always adds one source, even when empty.
More than one source throwsIOException before writing that node. It emits
list size1+2*attributes+contentCount, then tag and ordered attributes. At
00c4..00f2 it writes bytes if present; otherwise writes child-list content
only for a positive child count. Thus a zero-child node has NO content slot
or LIST_EMPTY suffix. A01 recursively applies the same rule to descendants.

Rust previously treated NodeContent::Children([]) as present content and
emitted an extra LIST_EMPTY. It now omits that slot and suffix, preserving
empty byte content and nonempty children. The model still retains the caller's
Children variant; this is wire canonicalization, not mutation of stored nodes.
Literal fixtures cover root omission, attributed omission, nested omission,
nonempty parent list and distinct empty bytes. With the existing web fixture
dictionary, empty iq must be `00 f8 01 19`; before the fix Rust produced
`00 f8 02 19 00`. The regression failed on that exact difference before fixing.
The structural change applies to every dictionary; this synthetic fixture is
not a native execution or proof the server rejects the alternate encoding.

## Confirmed compatible list and raw-byte size branches

A04 emits0 for list0, f8 for sizes1..255, f9 for256..65535 and throwsIOException
at65536 or greater. Rust list_start has the same positive-size boundaries.
A06's raw-byte branch usesfc below256, fd below2^20, fe at2^20 and above;
its length derives from Java's signed nonnegative array length (31 bits).
Rust's general Vec model also allowsu32 lengths, but actual transport bounds
reject much smaller records; no reachable oversized wire case is established
or speculative global allocation limit added here.

A06 also contains optional nibble/uppercase-hex packing, including native
colon nibble12 and low-nibble f odd padding. Rust currently does not select
colon nibble packing. That remains a separate open encoding-parity item:
its native reader, complete A05 context, dictionary lookup and actual producers
need corroboration before changing inbound or outbound string rules. Complete
A05 retained for follow-through; not all branch semantics are claimed audited.

## Serialization budget is conditional and precedes record encryption

The [previous codec-write](../2.26.43.72-status-final-writer/codec-write.txt)
selects29e only if its providerAsL returns a nonnullInteger; otherwise uses
ordinary ByteArrayOutputStream.29e allocates initial8192bytes, stores the
provider limit and a separate0x1000000 diagnostic max. Its synchronized
single/bulk writes throwE5X before appending when currentSize+writeLength
would exceed the provider limit; bulk negative length also throwsE5X. A
serialization may already have appended earlier bytes to this LOCAL buffer;
it has not reached the transport-counter/encryption/output block ofCyu.
E5X stores sizeAtAbort/limit/maxStanzaSize and increments its own diagnostic
counter. Do not equate that counter with a transport nonce.

1r2.<clinit> installs1r4 as default andAtomicReference initial value;1r4.AsL
returnsnull (unbounded serialization buffer, still framed-output limit).
The exact direct-field census has three refs: initializer, codec read and
T3d.Bk4. Bk4 installsRPU under its own installation guard. RPU.AsL returnsnull
when feature0x71b7 is false; otherwise chooses a limit with storedSff state,
configuration and optional runtime heap calculations. Its branches include
1MiB/16MiB bounds and configurableMiB multiplication, but runtime values,
feature defaults, Sff producer/update/clock authority and install lifecycle
are untraced. The complete provider is retained, not reimplemented as a
universal server limit or a mandatory Rust cap.

This narrows the prior size-failure attribution: selected29e E5X is pre-record-
encryption, whereas1oH E5W is checked AFTER counter consumption/encryption and
before its frame-header output. Other exception producers and whole connection
recovery remain open. Existing Rust consumed-channel/uncertain-outcome policy
is unchanged; neither finding authorizes resend or nonce rollback.

## Reproduce

From checkout root with the pinned APK outside Git; use a separate output
directory and compare complete bytes with these retained filenames:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --method 'A01' --signatures --exceptions > node.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --method 'A04' --signatures --exceptions > list.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --method 'A06' --signatures --exceptions > bytes.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --method 'A05' --signatures --exceptions > string.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/29e;' --signatures --exceptions > buffer.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/E5X;' --signatures --exceptions > size-exception.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r3;' --signatures --exceptions > provider-interface.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --method '<clinit>' --signatures --exceptions > provider-default.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r4;' --signatures --exceptions > provider.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/T3d;' --method 'Bk4' --signatures --exceptions > provider-install.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/RPU;' --signatures --exceptions > provider-installed.txt
target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --field A03 --type 'Ljava/util/concurrent/atomic/AtomicReference;' > provider-refs.txt
```

PCs are byte offsets; branch deltas are code units. Direct field references do
not establish alias/reflection or full runtime coverage. No observed server
result, native runtime test, ban guarantee or general serializer parity claim.
