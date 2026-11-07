# Native parameter-map header and exact body consumption

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native code executes; byte projections below are analyst transcriptions.

## Native header gates differ from the Java header helper

The selected [loader branch](../2.26.43.72-mobileconfig-json-adapter/README.md)
0xbf604 calls 0xbf690, which delegates to header reader 0xbf870. It first reads
30 bytes and requires the stream's state word to be zero. It then checks:

| Raw file offset | Selected native check |
| --- | --- |
| +2 | Unsigned 16-bit declared length between 10 and 500 inclusive |
| +4 | Unsigned 16-bit magic equals 32767 |
| +6 and +8 | Each unsigned count at most 60000 |
| +28 | Unsigned hash-byte length at most 128 |

The hash length is read into a zero-initialized 129-byte temporary. A terminator
is written at the declared hash length and the result assigned as a C string to
an internal header-object string. That string's object offset +0x20 is **not**
a raw file offset. The selected header routine does not recheck stream state
after this hash read before returning its length-admission boolean. Later body
processing has its own stream check; header-value admission alone is insufficient.

The complete selected helper contains no comparison of file bytes +0/+1 to ASCII
v4 and no requirement declared_length == 30 + hash_length. It consumes 30+hash_length
bytes sequentially rather than seeking to the declared length. These are bounded
helper observations, not permission to send modified formats or proof the entire
loader accepts an arbitrary version prefix.

The earlier [Java PfF header](../2.26.43.72-token-param-header/README.md) selects v4
from initial bytes and requires the length equality. Preserve the distinction:
matching field offsets and packaged values do not make the validation contracts
identical. No universal validator is inferred from either branch.

## The body reader requires exact consumption

On header success, 0xbf690 copies counts +6/+8 to its loader fields and copies the
header hash string. 0xbf6ec resizes a vector of 8-byte entries to header count +8,
then reads exactly count*8 bytes into it. It calls peek and requires EOF (-1),
then tests stream state with mask 5 and requires those bits clear. Trailing bytes,
short reads and relevant stream errors therefore fail this selected body path.
EOF's state alone must not be conflated with the rejected mask bits.

Failure clears the selected vector through 0xa8264 and returns false; success
returns true. The calling 0xbf604 has already written mode/status fields before
these checks and invokes more helpers after header admission, so this is not a
whole-loader transaction or rollback proof. The vector is an 8-byte body table,
not the 0x50-byte schema-record vector previously used by selection and checksums.
Original schema-record population is still a separate trace.

## Packaged maps and qualified fault projections

inspect-reader.py verifies both pinned identities and reads only the four selected
ZIP assets. Their native byte projections satisfy header bounds and exact
30+hash_length+count8*8 consumption, consistent with the earlier
[Java body evidence](../2.26.43.72-token-param-body/README.md). reader.json retains
asset byte counts/digests and parsed numeric fields; no raw assets are committed.
The projection decodes little-endian fields for this pinned arm64 library/assets.
It is not a target execution oracle or proof of every loader branch.

Synthetic variants of the u4 bytes show the distinct boundaries: replacing the
first two bytes or changing declared length to 10/500 leaves these selected
helper checks admitted; 9/501, bad magic, count>60000 and hash length>128 fail.
Changing the unused +26 word or count +6 to zero has no additional body effect
in this selected helper. A trailing byte or truncated body fails exact consumption.
Changing hash length to zero can pass header-value checks yet fail body size.
These are analyst-derived byte projections under ordinary complete reads, not
observed native acceptance, a suggested production format or a malformed-input
recovery API. External I/O faults and caller state/routing are not simulated.

## Reproduction and remaining boundary

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-native-param-reader/inspect-reader.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so ../android-lab/apks/WhatsApp.1.apk > /tmp/mc-native-param-reader.json
cmp /tmp/mc-native-param-reader.json docs/protocol/apk/2.26.43.72-mobileconfig-native-param-reader/reader.json
```

The script validates selected BL targets, APK/member hashes and asset reads,
then emits the explicitly synthetic projections. LLVM windows retain complete
selected header/body/forwarding logic and its immediate caller. Nearest exported
JNI_OnLoad headings are not local function identities.
rea-native-header.json retains complete REA Evidence
`ev_2b5272cc722ddf8c7af0412e09a70f057240bfd94dc397fff38f54543d3b68a9`:
`e scr.color=0; pd 62 @ 0xbf870`, exit 0, untruncated output, unchanged member hash,
with an entrypoint warning. It is a bounded window; LLVM retains through the
selected helper boundary. Register-relative annotations are not runtime memory.

**Remaining:** exact caller/version routing, body-entry interpretation in native
code, original 0x50-byte record population, configured path producer, I/O failures,
replacement/account ownership and response persistence/cache activation. No Rust
runtime, sender, timing or account state changed.
