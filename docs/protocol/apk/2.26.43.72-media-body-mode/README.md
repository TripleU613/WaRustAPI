# Upload body mode and fixed-length input — APK2.26.43.72

**Confirmed selected static**, build264307238, pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target or account execution. Complete
[REA method records](../../rea/2.26.43.72-media-body-mode/README.md)
corroborate the previously retained checked DEX for
[JAS.A04](../../rea/2.26.43.72-media-upload-http/urlconnection-execute.txt),
[JAT.A04](../../rea/2.26.43.72-media-upload-http/tigon-execute.txt),
[15Q.A01](../../rea/2.26.43.72-media-upload-http/transport-selector.txt) and
[K2k.A06 body writer](../../rea/2.26.43.72-media-upload-http/body-writer.txt).
REA Java contains decompiler control-flow warnings; DEX branch destinations
are authoritative. In these exports PC spans are bytes, branch deltas code units.

## Shared fixed-length configuration

15Q constructor reads00E.A14(0x666) intoA09 at0042..0052. Physical writes-only
field census finds this sole initializer. A14 passes00G.A03 and null Boolean
fallback to00E.A0D; null is not proof the effective result is false. Effective
runtime configuration/default resolution has NOT been established here.

15Q.A01 loadsA09 intoJAT constructor argument13 (v18 for invoke v5..v20),
which stores it inJAT.A0G. Its JAS branch loadsA09 intoargument15 (v33 for
invoke v18..v33), storingJAS.A0A. Thus both selected fixed-length tests read
the same factory snapshot of0x666, not independent constants or stream-kind flags.
15Q's two0VW.A01 calls feed separate JAS fieldsA01/A09 (socket/TLS selection),
notA0A. Complete0VW.A01 synchronizes and returns !A01. Do not mistake those
neighboring Boolean inputs for the fixed-length gate.

## Selected route-bound single-stream branch

Let n be firstJiH.A01 length, o firstJiH.A02 offset. With nonnull M3d and
one registered stream, K2k.A06 copies the stream throughA01 and optional EOF
processing; it returns before multipart fields. The fixed-length flag does
not change this raw-body choice. The existing processor/seal boundaries still
apply; raw does not mean a voice body has no seal.

| Condition | JAS explicit construction | JAT explicit construction |
|---|---|---|
| o > 0 | Content-Range bytes o-*/*; chunked mode | Same range; carries offset and length to body provider |
| o <= 0, gate true, n > 0 | setFixedLengthStreamingMode(n); skips chunked setup | Adds Content-Length n; carries length to body provider |
| o <= 0, gate false or n <= 0 | chunked mode, raw body, no multipart header from this branch | No explicit Content-Length here; still carries length to body provider |

JAS selects chunking at0282; the fixed-length path instead sets its selector
false at0260 and jumps past that assignment. At024e (gate false), +0x1a targets0282, NOT0264's
multipart Content-Type block. At0256 (nonpositive length), +0x16 also targets
0282. Treating a disabled gate as multipart would therefore be incorrect.
JAT at0330 (gate false) branches to015a; at0338 (nonpositive length) to015a.
It does not enter0352's multipart header block on either condition.

With null route, HTTP setup emits multipart/form-data boundary and the body
writer enters multipart. Empty-stream cases require separate care: writer also
enters multipart, while nonnull-route header paths do not necessarily set its
Content-Type. This is not the supported one-stream operation and is not used
to infer an ordinary-upload requirement. Multipart is not a universal header.

JAT wraps request/offset/length in0SU and constructs its native body provider.
Native-stack synthesis of Content-Length or transfer framing is not proven
by absence of an explicit addHeader; downstream provider internals remain open.

## Rust comparison and limits

image_upload::upload_prepared uses a known byte body, including the separately
implemented finished-voice seal, and reqwest's length-aware POST. It has no
resume offset, guessed Content-Range or unconditional multipart header. Raw
shape is supported by the traced branch. This is NOT proof its fixed-length
choice equals the effective APK configuration, or that declared native length
includes all processor bytes. No Rust behavior change is justified solely from
this static gate trace. Preserve existing uncertain reservations and no replay.

Remaining: effective0x666/default resolution, complete native body-provider
length/framing behavior, all stream-length/processor inputs and transport
selection. Configuration evidence from another APK build is not current proof.

## Reproduction

From repository root:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/JAS;' --method '<init>' --signatures --exceptions > url-ctor.txt
target/debug/xtask apk-smali "$APK" --class 'LX/JAT;' --method '<init>' --signatures --exceptions > tigon-ctor.txt
target/debug/xtask apk-smali "$APK" --class 'LX/0VW;' --method A01 --signatures --exceptions > platform-flag.txt
target/debug/xtask apk-smali "$APK" --class 'LX/15Q;' --method '<init>' --signatures --exceptions > factory-ctor.txt
target/debug/xtask apk-field-refs "$APK" --class 'LX/15Q;' --field A09 --type Z --writes-only > fixed-flag-writes.txt
target/debug/xtask apk-smali "$APK" --class 'LX/00E;' --method A14 --signatures --exceptions > flag-reader.txt
```

checksums.json covers every other file with byte counts and SHA256.
