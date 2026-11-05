# Native metrics credential persistence — APK 2.26.43.72

**Confirmed static / derived; observed synthetic header admission.** Pinned APK
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
No real credential, filesystem I/O, Android process or server request was used.
Local Rizin exports/Unicorn fixtures are not REA Evidence envelopes. No Rust
production behavior changed. This is local metrics state, not account/message
replay authority or server acceptance evidence.

## Paired path and observer

Helper **0x6fe6a0** copies the store's configured root string +0x28 and appends
`/WAMMetadataPrivateCredential-%d`, formatting its raw signed byte +0x30.
A direct-BL census of executable sections finds exactly two calls: save
0x729684 and load0x72a7c4. This census excludes indirect calls and does not
assign a broader semantic name to that mode byte.

The previously traced observer scheduler0x728bfc passes manager/credential in
MCIParams to thunk0xbd8fb0 → **save0x729540**. It checks execution contexts,
allocates a separate credential object and copies data slots +0x18/+0x20 and
optional +0x28, then copies expiry +0x30 and remaining-use count +0x38.
Copying occurs when the worker executes, not a proved atomic disk snapshot at
request admission. Admission/use consumption and observer scheduling are traced
in [credential lifecycle](../2.26.43.72-metrics-native-credential-lifecycle/README.md)
and [the complete sender](../2.26.43.72-metrics-native-transport-responses/buffer-sender-assembly.txt).
The existence of a queued save is not proof it executed or completed.

## Version2 storage envelope

Writer0x7296ac..0x7297b8 emits this25-byte fixed header in ARM64 little-endian
order, followed by the three data segments in slot order:

| Offset | Bytes | Value |
| --- | --- | --- |
| 0 | 1 | version2 |
| 1 | 4 | remaining-use count, object+0x38 |
| 5 | 8 | raw expiry, object+0x30 |
| 13 | 4 | data+0x18 length |
| 17 | 4 | data+0x20 length |
| 21 | 4 | optional data+0x28 length, zero when absent |

These are raw persistence fields, not server quota/timeout values. The selected
writer uses lengths obtained from MCFData; native payload meanings remain
unresolved. No real payload bytes are retained in this bundle. Test inputs use
explicit synthetic repeated ASCII bytes, not valid credentials.

## Write/close result and limits

Save creates the root directory with mode0x1fd (0775), continuing on success or
errno17/EEXIST. It opens the paired path through0x733ce0 with raw mode1, which
the [shared mode table](../2.26.43.72-metrics-native-metadata-file/README.md)
resolves to **wb**. This can truncate an existing file before the new bytes are
fully written. It writes the full serialized range through0x734210, then closes
through **0x734448**, even when the write status was nonzero. Ordinary selected
success requires write raw status0, close raw status2 and the written length
matching the serialized length. Otherwise it takes diagnostic handling.

The [shared write helper](../2.26.43.72-metrics-native-drain-persistence/README.md)
uses fwrite/ferror; the retained close helper calls fclose and marks its object
closed. fclose success is encoded as low raw status2; a close failure encodes1
with errno in upper bits. These are local helper statuses, not HTTP/native
credential-result enums. No temporary-file rename or fsync appears in the
selected save/close path; power-loss durability and whole-framework storage
policy remain unknown. No local counter rollback, send cancellation or durable
commit acknowledgement is established by a successful observer enqueue.

## Load guards and restoration

**Load0x72a740** uses the same path and raw open mode0/rb. It requests0x219
(**537**) bytes through0x734174, then closes the handle. This is a selected read
request, not a proved semantic maximum credential size or whole-file length
check. Before reconstructing it requires:

- Read status0 and close status2.
- At least25 data bytes and first byte version2.
- Nonnegative signed32 count and three lengths; nonnegative signed64 expiry.
- Enough remaining bytes for each segment in sequence.

The fixed-header/length checks run at0x72a818..0x72a8bc, with success transfer
to0x72a9cc. They do **not** require positive segment lengths, exact payload
length, positive count or an expiry later than now. Extra trailing bytes are
allowed by these checks. Other stages may impose stricter requirements; header
admission alone is not credential/signature validation.

On admission it creates/copies the three segment objects, invokes credential
constructor0x7212f4, then **overwrites** the constructor's default expiry with
the stored expiry and calls setter0x721368 with the stored counter. It installs
the reconstructed object into manager+0x78. Thus this load does not simply
refresh expiry or reset the use count to64/180. Actual expiry/count usability
is checked separately by sender admission. A zero third length reconstructs
null, not an asserted empty optional data object.

A malformed header logs invalid-format handling, releases objects and requests
renewal; the selected malformed-header branch does not remove the file. A
read/close error branch instead calls remove(path) before renewal. Missing/open
failure also falls back to renewal. Distinguish those branches; removal here
concerns the local metrics credential file, not account identity or message
outcome history. The separately retained body0x72ab10 was initially labelled load-completion;
subsequent log/argument/timer analysis identifies connection-status handling.
Its file is now connection-status-assembly.txt; see the
[status/timer trace](../2.26.43.72-metrics-native-recovery-timer/README.md).
It must not be treated as proof of credential-file load success.

## Fixture scope and reproduction

**23 bounded native cases** cover valid segments, zero-length/header-only,
trailing bytes, version/short-header rejection, negative and maximal count/expiry,
negative/oversized individual lengths and truncation. The native header-admission
instructions, including its version-byte helper, execute with controlled MCFData
length/byte/range adapters. They match an independent Python signed-field and
length predicate. No file open, full restoration, cryptography or real MCF
allocation is executed. Even a header-only admitted case is not claimed usable.

Run `python inspect-file.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
file-boundaries.json. It reuses the pinned ELF resolver, hashes windows, extracts
literals and independently enumerates direct path-helper calls. Rizin windows
use `pD SIZE @ VA`. With Unicorn2.1.4, run
`python check-header.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
header-results.json. Each case has100000-instruction/one-second caps; unexpected
imports/interrupts or unsupported loader relocations fail closed. Execution
stops at the header reject/success boundary before downstream processing.
Checksums cover all other files.

**Inferred:** quota/expiry persistence and the timing of its writes matter to a
future metrics port; resetting these fields at restart would differ from this
selected load. **Observed live:** none. **Unknown:** complete credential algorithm,
atomicity/recovery guarantees, load scheduling/timer semantics and deployment
necessity. Endpoint-list production is still open; the callback followed here
proved to be persistence, not endpoint construction. No mandatory telemetry,
automatic retry or restriction-avoidance guarantee follows from this evidence.
