# Prekey-count ACK optional source

**Confirmed static**, APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
New checked exports use xtask apk-smali APK --class LX/89Y; --method A01
(and A00) --signatures --exceptions. They are DEX text, not REA Evidence IDs.

The retained [15v notification dispatcher](../2.26.43.72-prekey-upload-replies/prekey-ack-count-dispatch.txt)
uses base A07(true) to obtain89Y; the [count worker](../2.26.43.72-prekey-upload-replies/count-worker-routing.txt)
invokes89Y.A01 after processing. A01 calls A00 with its original IncomingStanza
and null child/attribute overrides. A00 consumes the frozen7p2/Eoh and invokes
0v6.A00 with that key on the selected null-override path. It does not replace
its source with a fabricated server JID.

The common [notification key fields](../2.26.43.72-group-missing-identification/notification-key-fields.txt)
carry nullable from into Eoh. The [final wire builder](../2.26.43.72-group-missing-identification/picture-final-wire-builder.txt)
F3e.A05 emits id, optional to, class/type and other supplied metadata. Its
004a branch skips to when Eoh.A02 is null. These common bodies are not picture-
only functions. This establishes optional destination construction, not every
native malformed-source exception or prekey scheduling branch.

**Confirmed local defect/fix:** low_count_ack used unwrap_or(server) for both
absent and unparsable from. The actor therefore emitted an invented explicit
destination on absent source and silently redirected malformed explicit source.
It now omits to when from is absent, routes a parsed source through existing
ack_address, and returns InvalidInput for an explicit parse failure. That last
refusal is a local wire-integrity boundary, not a claim that native malformed
sources take an identical error ACK path. ID/local envelope bounds remain.

The actor source-less PQ-first count test fails on the old invented to, then
asserts omitted to and no legacy upload. Its existing valid-source legacy count
still ACKs/uploads under unchanged bounds. The refused-content actor matrix now
includes malformed explicit source, retaining capture/no-ACK and receive
continuation. No source authority, ID or count was fabricated. Native worker
ACK-after-processing versus Rust ACK-before-upload remains a separate unported
ordering issue; this change does not redefine that obligation or authorize
retrying uncertain batches. No live account operation ran.

Bounded research direction: a fresh class10a A09 check was a rejected candidate;
EL2 encrypt code241 and the existing15v dispatcher identify the correct handler.
No claims from10a were used. Exact count-worker/null-override chain is above.
Checksums cover every other bundle file.
