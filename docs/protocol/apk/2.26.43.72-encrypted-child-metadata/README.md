# Encrypted child metadata before receive authority — APK2.26.43.72

**Confirmed selected static plus Rust regression**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX fallback exports. PCs are bytes; branch displacements
are16-bit units. No new REA Evidence ID, APK execution or account traffic.

## Secondary descriptor is sender-key ciphertext in this parser branch

[Exact Dxs.A0A writer census](secondary-context-writers.txt) finds its constructor
alone; [constructor caller census](chat-context-callers.txt) finds Ewa.A01 alone.
[Builder](chat-context-builder.txt) loads Ewa.A0A into v22 and A0B into v23,
passes them to Dxs at01ae, and the [constructor](chat-context-constructor.txt)
stores them as Dxs.A09 (first Ezj parameter v19) and Dxs.A0A (second v20).
[Exact Ewa.A0B writer census](secondary-builder-writers.txt) has two sites, both
EuN.A01. The [complete parser](chat-stanza-parser.txt) stores a descriptor in
A0B only when its Ezj.A00 type equals2, at08fe or0ca4. Other selected descriptor
types go into A0A at0930 or0cda. [EIM.A00](enc-descriptor-parser.txt) maps msg=0,
pkmsg=1,skmsg=2,frskmsg=3,msmsg=4 (also handles otsk);
[Ezj constructor](enc-descriptor-constructors.txt) preserves the type into A00
and version into A01. Thus the earlier secondary descriptor in this selected
chat parser is skmsg, not a second arbitrary direct-session message.

The previous retained [F2q.A04](../2.26.43.72-media-error-retry-producers/decrypt-retry-processing.txt)
first pass uses F1Z.A09 and enables sendReceipt when F1Z.A0A is absent. For this
selected Dxs mapping that means the direct ciphertext pass suppresses its receipt
when a following skmsg descriptor exists. A conditional later call at0a84 passes
that skmsg descriptor and current v2 boolean. The complete reaching definition
of v2 and all first-pass failure/stop transitions are still unclassified; no new
claim of unconditional later-pass receipt emission is made. Direct/converted
error491 writer and durable action ownership remain separate unknowns.

## Parser validates outer versions and counts before decryption

EIM.A00 requires the exact text version `2`; other spellings throw LIL at0116.
This is the stanza enc version, not the internal Signal ciphertext version.
The descriptor records version2, independent of ciphertext leading bytes.

EuN.A01 reads `count` with [0Tp.A05](integer-default.txt), default0 when absent;
[0Tp.A06](integer-parser.txt) delegates to Java Integer.parseInt and converts
NumberFormatException to LIL. At090a..093e and0cb4..0ce8 the parser compares
against its already stored count ONLY if that previous count is positive, then
stores the current count. It preserves input traversal order. Therefore:

| Counts in traversal order | Selected comparison result |
| --- | --- |
| absent, absent | admit (0,0) |
| 0,1 | admit |
| 1,0 or1,absent | refuse |
| 1,2 | refuse |
| +01,1 | admit |
| negative,positive | admit by this check |

Do not replace this with an unordered equality check or unsigned-only parsing.
The parser also compares paired descriptor versions at0942..096a and0cec..0d16.
In the selected EIM branch, exact version2 admission already precedes this pair
comparison. Other parser branches/providers remain separately bounded.

## Confirmed Rust mismatch and correction

Before this change, the group receive selector filtered children for version2
and selected one direct and one skmsg child. A conflicting outer version could
be silently ignored while the supported child decrypted; count was not validated
or compared. That could advance state and authorize a response for a stanza the
selected native parser refuses.

`inbox::encrypted_children_metadata_valid` now checks all immediate enc children
before process_direct routes to direct/group/recipient handling. Version must be
exact2; missing count defaults0; the existing Java decimal helper plus checked
i32 conversion retains signed/leading-zero/BMP Unicode digit behavior and bounds.
Nontext decoded values remain outside this supported Rust adapter. Count
comparison preserves input order and the previous-positive condition. Refusal
returns IncomingResult::Unsupported before decryption, duplicate lookup or ACK
permission. Capture preservation remains caller-owned and is not erased. This
adds no native error response or retry and does not infer that native LIL itself
is unacknowledged; its complete response handling remains untraced.

The regression extends a real encrypted-store group fixture with valid skmsg
ciphertext. Overflowing/malformed counts, conflicting paired counts and an extra
wrong-version enc return Unsupported with unchanged sender keys and journal after
capture preservation. The corrected original still decrypts, proving the rejected
variants did not consume its ratchet. Existing consumed/redelivery behavior stays
covered. A table test covers ordered defaults, signed boundaries, Unicode digits,
invalid spellings and mixed versions. Broader native duplicate/type selection,
frskmsg/msmsg/otsk support, nested recipient parsing, transactions and response
ownership are not claimed complete. No server/restriction causality is established.

Reproduce `apk-smali <apk> --class '<descriptor>' --method <name> --signatures
--exceptions` for Dxs.<init>,Ewa.A01,EuN.A01,EIM.A00,Ezj.<init>,0Tp.A05/A06;
`apk-field-refs` selects exact Dxs.A0A or Ewa.A0B with type LX/Ezj; and writes-only;
`apk-callers` selects Dxs.<init>. [checksums.json](checksums.json) pins all files.
