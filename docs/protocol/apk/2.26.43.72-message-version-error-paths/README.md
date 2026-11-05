# Message version errors and producer boundaries

**Confirmed static:** APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked-DEX exports, not REA Evidence envelopes; no APK/account execution.

## ED6 is not established as an identity refusal

Five constructor callers were found: FKZ.BLW and constructorsFj0/Fj1/Fj2/Fj3.
The selected four message constructors explicitly create ED6 for legacy versions.
This rules out using its label or -1007 result as proof of UntrustedIdentity.
FKZ.BLW and complete identity-change handling remain untraced in this chunk.
No native identity-refusal equivalence was established and no policy changed.

## Different version ranges and exceptions

All these branches examine the high nibble of the version byte. Minimum byte
length, required fields, key decoding and nested parsing still apply; the table
only describes the selected version gate, not complete message admission.

| Constructor | Version gate | Low version | High version |
| --- | --- | --- | --- |
| Fj3 direct Signal message | 2..4 | <=1: ED6, Legacy message | >4: EDY, Unknown version |
| Fj1 prekey message | 3..4 | <3: ED6, Legacy version | >4: ED5, Unknown version |
| Fj2 sender-key message | exactly3 | <3: ED6, Legacy message | >3: EDY, Unknown version |
| Fj0 sender-key distribution | exactly3 | <3: ED6, Legacy message | >3: EDY, Unknown version |

Fj3 version gate0038..0042 selects ED6 at00c0 orEDY at00d4.
Fj1 gate0020..002c selects ED6 at0170 orED5 at0184.
Fj2 gate0038..0042 selects ED6 at00ac orEDY at00c0.
Fj0 complete selected constructor is retained separately; this distribution
producer is not automatically a decryption-result producer.

[Ordinary direct0WJ.A0N](../2.26.43.72-direct-retry-cause-paths/README.md)
catches ED6 as -1007 (excluded by selectedEtY retry predicate) and EDY as -1005
(eligible and potentially refined). Group0Yl.A00 and fast-ratchet0WJ.A0P have
the same selected ED6/EDY status split. Thus treating all unsupported versions
as no-retry would incorrectly merge low and high native version branches.

[Prekey0WJ.A0O](../2.26.43.72-outgoing-retry-error-map/README.md) has a separate
early constructor region: catches ED5 -> -1006 and EDY -> -1005, but its early
catch table does not list ED6. ED6 -> -1007 handlers apply to later protected
regions, not to every operation in the method. Fj1's nested parse catch table
can also wrap ED6 as EDY. Do not apply the later ED6 handler to the outer
constructor or infer universal retry/connection behavior from this bounded trace.

## Current backend comparison

The exact installed wacore-libsignal0.7.0 protocol.rs identity and line references
are retained in backend-source-identity.json; this is source inspection, not
runtime observation. Its direct/prekey parsers accept versions3..4 and use
UnrecognizedCiphertextVersion for both lower and higher versions. Its sender-key
message/distribution parsers accept exactly3, split low LegacyCiphertextVersion
from high UnrecognizedCiphertextVersion, and check byte length first.

**Confirmed gap:** native Fj3's version2 gate differs from the backend's direct
version3 minimum. This does not establish safe version2 crypto support: full
primitive/key/MAC/session semantics are unported. Rust currently merges these
backend version variants into generic diagnostics/Undecryptable in relevant
receive paths; blanket mapping would discard native producer and range context.
**Unknown:** all native low-version consumers, native short-input exception
semantics, distribution ingestion policy, identity refusals, provider equivalence
and complete count/key/routing/ACK behavior. No request shape/behavior changed.

Reproduce APK exports: target/debug/xtask apk-smali APK --class 'LX/Fj3;'
--method '<init>' --signatures --exceptions; substituteFj1/Fj2/Fj0.
Inventory: apk-callers APK --class 'LX/ED6;' --method '<init>'. Recheck installed
backend identity before reusing its line references after a dependency update.

Validation:31 filtered retry tests, APK coverage, staged whitespace and research
checksum/link hygiene passed. No Rust source changed; the full workspace suite
was not rerun for this research-only chunk.
