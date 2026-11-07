# Media range consumers and result validation — APK 2.26.43.72

**Confirmed selected static / research only**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports via xtask fallback, not target execution or REA
Evidence IDs. No Rust runtime change, account operation or captured network traffic.
Byte PCs below are method-relative; branch displacements use16-bit code units.

## The streaming wrappers do not select a universal full-file profile

The earlier [shared HTTP boundary](../2.26.43.72-media-download-http/README.md)
found MuY.A03 and MuZ.A03 among four direct0lO.A01 callers. Exact [MuY census](muy-A03-callers.txt)
and [MuZ census](muz-A03-callers.txt) now each find THREE direct invocations, all
inside their own A00. These are invocation-site counts, not three distinct callers.
No reflection/JNI completeness is claimed.

[MuY.A00](muy-A00.txt) is marked StreamMediaDownloadHandler by its own log strings.
At07b6..07d8 it computes a range end through MC6.A01 and calls A03 with a selected
start/end. At07da..084a it conditionally extracts the Content-Range slash suffix,
trims it, treats `*` as unknown, and parses a long; another branch reads content
length. At0898..08d6 it derives another bound and calls A03; at0de4 it calls the
wrapper again between stream windows. Those paths cannot be collapsed into one
zero-offset full-file operation without tracing input/mode/state selection.

At08e0..0930 MuY constructs M29 from key material/sidecar bytes/object length and
packages it with the1gY response, MT7 range state and two longs in MJi. The
[complete MJi constructors](stream-response-wrapper.txt) simply retain these
fields or an error integer. They do not validate status or authenticate bytes.
Later MuY reads status for metrics at0b64 and obtains the input stream at0bb8;
that status observation alone does not introduce an additional200-only predicate.

[MuZ.A00](muz-A00.txt) is separately marked StreamMediaDownloadPushHandler and
uses Mp4StreamCheck, replay/chunk state and MJj. Its three direct A03 sites are
0742,0d5a,12b0.
It also parses Content-Range after the initial request. This export does NOT
establish MuZ uses M29: it is a separate producer family, not silently equated
with MuY. Its complete decrypter implementation remains untraced here.

## Selected chunk decrypter is not the whole-file receiver

[M29](stream-decrypter-M29.txt) constructor stores LvP, sidecar bytes and
object-length-minus10. A00 builds a MAC through MTj.A03, branches between
MTj.A02 and explicit AES/CBC/NoPadding cipher construction, processes input,
and calls Cipher.doFinal. At01d4..020c it truncates the computed MAC to10bytes,
selects10bytes from the retained sidecar and compares using MessageDigest.isEqual;
mismatch logs offsets and throws LUP. Key derivation, sidecar/index provenance,
window geometry and MTj helper internals are not established by these leaves.

Cipher.update output is written to the supplied OutputStream at0118..0120
BEFORE the final sidecar-MAC comparison. This is internal streaming behavior,
not evidence that unauthenticated plaintext may be published to a user. Caller
buffering, checkpoint visibility and output adoption must be traced separately.
Rust's existing whole-file image verification before plaintext handoff must not
be weakened to mimic this internal ordering, or replaced by a guessed chunk MAC.

## Normal downloader keeps status, length and result validation separate

The previously retained [MVv.A04](../2.26.43.72-media-download-http/caller-mvv.txt)
uses content length for200, while206 tries the Content-Range total after the last
slash using trim/Long.parseLong at04d8..0540. A NumberFormatException branch logs
`normaldownloadhandler/unparseable Content-Range total` at0542; its fallback
uses start+nonnegative content length, otherwise unknown(-1). Feature predicates
can select other length policy. This is not a general strict HTTP range parser.
MVv reads through1gY.AUq at081a and invokes AES.A06 on result paths. Here `AES`
is an OBFUSCATED CLASS NAME, not evidence of a Java cryptographic AES operation.

[Finalizer AES.A06](normal-finalize-A06.txt) first invokes A01; then checks
FutureTask cancellation before interpreting that result. A nonzero A01 result
becomes a failure result. If its two file objects compare equal, it returns
without A02. Otherwise A02 validates the calculated plaintext hash, followed by
another cancellation check; missing calculated plaintext hash maps to result31,
mismatch to7, cancellation to13. These numeric local result codes are not HTTP
or protocol ban/retry codes.

[A01](normal-validate-A01.txt) checks a calculated transfer hash against MX3.A0K;
when that expected hash is null it falls back to LvQ.A01. Missing calculated hash
returns27, mismatch32. [A02](normal-validate-A02.txt) compares the calculated
plaintext hash to MX3.A0L: null calculated value returns1, mismatch2, equality0.
Actual digest algorithms and exact descriptor-to-MX3 field construction still
need upstream evidence. No hash fallback is added to Rust: its supported encrypted
media descriptors already require explicit fixed-width expected hashes.

## Conclusion and next evidence required

**Confirmed:** native status acceptance, range processing, stream authentication
and final result adoption are separate layers. MuY/MuZ are stream/window families;
normal MVv also has partial-file and416 paths. This refines the prior leaf-policy
difference; it does NOT prove a full ciphertext GET must accept every206 or416.
Rust remains200-only with full ciphertext/MAC/plaintext validation and no replay.

**Unknown:** supported operation factory selection, start/end values for the
particular ordinary image/view-once profile, native200 response to a nonzero range,
complete Content-Range rejection/recovery semantics, native crypto/result source
binding, cancellation/account replacement, final viewer/export ownership and
runtime server interoperability. No mandatory-header, zero-ban or complete
media coverage claim is made.

Reproduce exact caller censuses with `apk-callers <apk> --class 'LX/MuY;'
--method A03` (and MuZ); export `apk-smali <apk> --class '<descriptor>'
--method <method> --signatures --exceptions` for MuY.A00, MuZ.A00, AES.A01/A02/A06;
export MJi and M29 without method filters. [checksums.json](checksums.json)
pins every retained file including this report.
