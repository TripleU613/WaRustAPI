# Interop tuple identity and legacy integrator loss — APK 2.26.43.72

**Confirmed selected static**, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence/runtime observations.
No live accounts. The implemented correction is a local lossless-identity
boundary for Rust's existing REFERENCE tuple, not a port of native interop.

## Native tag/schema differ from the current reference model

[Complete123.A0A string reader](../2.26.43.72-packed-nibble/string-reader.txt)
recognizes tag0xf4 (244) at0098 and reaches privateA03 at0262. A03 reads a
string account component, two big-endian16-bit integers (device, integrator),
and a one-byte domain discriminator. Domain0 constructs
`integrator-account[:device]@interop`, omitting device suffix only atzero.
Domain2 requires BOTH device and integratorzero, then constructs an
interop.g.us address throughA04. Other domains or invalid group fields throwLIL.
This is not tag245 followed by a text server; the tuple formats are distinct.
The [complete native writer](../2.26.43.72-string-context/jid-writer.txt)
uses244 and writes corresponding selected user/device/integrator/domain fields.
Full native parser/cast/device subtype and group authority remain unported.

2p3 extendsUserJid, stores original input, separate integrator integerA00 and
account stringA01, and reports serverinterop/type22. Its constructor splits
input at a hyphen, checks a1..3-character integrator prefix not starting0,
parses it throughInteger.parseInt, checks account length1..15/nonzero-leading/
ASCII digits and stores both components. This is not a proof of a positive
ASCII-only integrator range: Java signed/Unicode parse semantics and the split
helper's exact behavior require care. No new grammar is implemented here.
These retained fields and reader string construction establish the narrower
fact: changing integrator changes identity; it cannot be discarded.

## Confirmed Rust identity-loss bug and bounded correction

Current Rust tags245/246 originate from reference codecs, not this APK's244
schema. Decoder.read_interop_jid reads user,device,integrator and text server,
then previously discarded integrator and returned Jid{user,device,Interop}.
Encoder writes this legacy form with integratorzero. Thus distinct input
integrators1/2 could collapse to the same model and silently re-encode aszero.
The model has no independent integrator field. Treating arbitrary reference
input as a native-compatible JID would be unsupported.

Rust now returnsInvalidJid with a specific not-representable reason for any
nonzero legacy integrator. It does not silently discard, adopt, remap or
resend the identity. The zero-integrator legacy path retains prior behavior;
that compatibility control is NOT proof native2p3 accepts zero or that tag245
is a valid current-APK tuple. Tag244 remains unsupported until its complete
schema/address/subtype rules can be represented losslessly. No reference tag
constant is relabeled or native support falsely advertised.

Literal fixtures exercise the existing legacy decoder path with user123,
device3, serverinterop and integrators1/2/65535; each now refuses. The test
failed before the fix because that input decoded successfully. A separate
zero-integrator control verifies preserved user/device/kind. These are
synthetic offline Rust fixtures, not native packet executions, server errors
or account-restriction observations. Existing typed error propagation and
uncertain-outcome policies remain; no automatic retry or identity migration.

## Remaining work

Lossless native244 support requires full group/user/device class conversion,
namespace validation, identifier grammar, caller admission and outbound tuple
construction evidence; reference245/246 interoperability must stay separate.
Existing saved JIDs/rows are not rewritten by this decoder change. No guarantee
about older persisted identities already decoded under the lossy behavior is
claimed, and no speculative migration or live account action is performed.
The broader network audit and full native codec parity remain incomplete.

## Reproduce

From checkout root with pinned APK outside Git; compare complete outputs:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A03 --signatures --exceptions > reader.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2p3;' --signatures --exceptions > user-class.txt
```

Existing linked A0A/writer exports have their own exact recipes/checksums.
PCs are bytes; branch deltas code units. Static boundaries and local refusal
are not complete runtime/admission/parity or ban guarantees.
