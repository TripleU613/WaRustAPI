# Closed enum projection for protocol types and keep actions

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Shared decode now follows the pinned closed-enum rules for BZb.type and
BV3.keepType. Unknown numeric values leave the last recognized value intact;
wrong-wire enum fields stay unknown, and repeated singular submessages merge.
Missing or unknown-only types remain absent rather than becoming REVOKE=0.
Malformed tags, truncation and illegal/unmatched groups still refuse.
No encoder, saved payload, account state, request or wire ID is rewritten.

## Evidence and interpretation

The [seven-topic recipe](apk/recipes/2.26.37.71-control-enum.json) reproduces
[12 files / 70,497 bytes](apk/2.26.37.71-control-enum/targets.json), retaining
full signatures, register flow and catch tables. BZb.dynamicMethod supplies
CvY's verifier for type field 2; CvY.isInRange delegates to C4F.forNumber.
That function recognizes 35 values from 0 through 41, excluding
1, 2, 12, 13, 15, 23 and 33. These gaps must not be treated as recognized
unsupported operations that overwrite an earlier valid type. Recognized but
unported operations still replace an earlier type and decline its old behavior.

BV3.dynamicMethod supplies Cv0 for keepType field 2. Cv0 delegates to C2L,
which recognizes UNKNOWN=0, KEEP_FOR_ALL=1 and UNDO_KEEP_FOR_ALL=2. Explicit
UNKNOWN replaces a prior keep/unkeep value; an unrecognized number does not.
The complete independent REA envelopes retain their Evidence IDs and source:
[protocol number lookup](rea/2.26.37.71-control-enum/protocol-enum-0.json),
[protocol verifier](rea/2.26.37.71-control-enum/protocol-enum-1.json),
[keep number lookup](rea/2.26.37.71-control-enum/protocol-keep-enum-0.json),
[keep verifier](rea/2.26.37.71-control-enum/protocol-keep-enum-1.json).
These are static observations, not Android execution.

The independent [Java oracle](ProtocolEnumsOracle.java) reconstructs these
projected enum fields, presence bits and enclosing singular message fields
on desktop protobuf-javalite 4.36.1. It is a minimal projection schema, not a
full copy of every BZb/BV3/BZc field or proof of the APK runtime version.
[2,040 vectors](../../crates/whatsapp-api/fixtures/prekeys/2.26.37.71-control-enum-oracle.json)
cover all values 0..64, signed/high-bit varints, duplicate/unknown values,
unknown-only/absent fields, split submessages, other wire types, malformed groups
and recursion boundaries. Expected results come from GeneratedMessageLite,
not a second copy of the Rust decoder. The fixture reproduces byte for byte.

## Runtime change and preservation

The original prost integer field kept a final unknown value. A regression
reproduced timer type 3 followed by unknown 1 declining instead of retaining
the timer. A separate regression reproduced a wrong-wire enum field declining
an otherwise supported timer. Shared decode now reads the original field bytes
with native-style varint32 projection, retains enum presence and known values,
and merges non-enum metadata fields through the existing prost models. Enum
unknowns do not reach prost as scalar fields. Raw message/checkpoint/signature
bytes remain untouched; field slices are borrowed rather than copied into new
plaintext buffers.

Protocol settings, sender revoke decode, keep decode and group keep target
validation all use the normalized shared envelope. Duration clamping,
after-read refusal, privacy metadata, parent attribution and route/source guards
retain their existing behavior. Split protocol/keep submessages retain earlier
enum presence while later supplied metadata still merges. The sender-revoke
receive path keeps its existing one-content/one-protocol guard; this codec
correction does not claim every repeated-message receive form is admitted.

The existing BP2 attestation parser supplies the shared wire primitives and now
uses that module for its original field-1 enum projection. All 1,502 previous
attestation Java vectors still pass. Its default E2EE, signature over original
bytes, inspection bound and separate native admission policy are unchanged.
The shared standalone wrapper is gated with its stateful attestation caller;
normal chat-setting codecs still compile without Signal crypto.

The Signal regression now encrypts peer/own revokes and own keep/unkeep with
trailing unknown enum numbers. It checks target/actor, confirmation boundaries,
reopen deduplication, unchanged uncertain outbox/idempotency, edited-content
refusal and continued ratchet use. Group codec tests retain the original parent
and distinguish an explicit recognized UNKNOWN from unrecognized numbers.

## Reproduction and limits

Use the fixture's Maven URL to obtain protobuf-javalite 4.36.1 outside Git and
verify SHA-256 `91d3dba2521322103230c509d41a017da3a5e6766610adcb10c5bb127cbb26c5`.
The generator also verifies that pinned JAR before executing the oracle.

```sh
javac -cp /tmp/enum-oracle/protobuf-javalite-4.36.1.jar \
  -d /tmp/enum-oracle docs/protocol/ProtocolEnumsOracle.java
cargo run --locked -p xtask -- java-oracle protocol-enums \
  --classpath /tmp/enum-oracle/protobuf-javalite-4.36.1.jar:/tmp/enum-oracle \
  --output /tmp/control-enum-oracle.json
cmp crates/whatsapp-api/fixtures/prekeys/2.26.37.71-control-enum-oracle.json \
  /tmp/control-enum-oracle.json
```

The standalone Java root counts toward its default recursion limit of 100;
the nested oracle counts both envelope and child. Rust preserves those local
projection bounds. The APK runtime version/limit and full wrapper nesting
contexts are not established. Other metadata fields still use their existing
prost models; complete unknown-field/tag/length parity, nested scalar enums,
full schemas and configured native ProtobufValidator remain incomplete.
No admission property is guessed or exposed as a bypass switch.

Validation passed 962 workspace/all-feature tests, strict Clippy,
warnings-denied Rustdoc, the Signal regression with only TCP/Noise/Signal
enabled, 26 default CLI/API-symbol tests, TCP/Noise without Signal, and the
experimental CLI build. Existing light-build unused-helper warnings remain.
No real account operation, network request or new live sample ran for this
pure decoder change. Cross-release APK comparison and fresh live control
interoperability remain unverified; complete native parity is not claimed.
