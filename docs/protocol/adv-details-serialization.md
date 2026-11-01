# Companion ADV signature details serialization

2026-10-09. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Confirmed static / implemented offline:** companion ADV signature input uses
parsed-and-serialized GsB details, not the original wire bytes. The prior Rust
exact-original-details assumption is **refuted** for this consumer. Signed
key-index lists and non-E2EE attestation have separate consumers; this result
must not be generalized to their signing inputs.

## Exact consumer and descriptor

`11T.A0D` parses Grz at 0160–0170, then its details into GsB at 0170–0184.
At **024c–0254** it calls GsB.toByteArray and assigns v19. Account input
(02ae–02cc) is the selected account prefix + **v19** + companion raw key.
Device input (02ee–030a) is device prefix + **v19** + companion raw key +
account raw key. Both B7H.A09 verifications are required (030a–0374).
A signature over reordered original bytes can fail even when the parsed
metadata is identical; conversely canonical signatures can survive wire reorder.

GsB.dynamicMethod 005e–006c declares proto2 fields rawId=1 uint32,
timestamp=2 uint64, keyIndex=3 uint32, accountType=4 enum and deviceType=5
enum, with distinct presence bits. The field-array helpers 6al.A1W,
6aj.A1U and Ayj.A1J close bitField0/enum-verifier positions. IQg.isInRange
calls Azc.forNumber; **Azc recognizes three values**, not just the two in
the vendored reference schema:

| Numeric | Azc field | Name | clinit offsets |
| --- | --- | --- | --- |
| 0 | A01 | E2EE | 0000–0014 |
| 1 | A02 | HOSTED | 0014–0028 |
| 2 | A03 | NON_E2EE | 0028–003c |

The local WAAdv.proto adds NON_E2EE=2 with APK attribution; generated Rust and
schema hash are regenerated through xtask proto-gen/proto-check. The upstream
revision remains the reference snapshot; this annotated enum addition is local
APK-derived evidence, not an assertion the upstream file already contained it.
Missing or unknown enum values project as default E2EE for this verifier,
with unknown numerics retained as lite unknown fields. Only HOSTED device type
selects hosted signature domains; NON_E2EE uses the other domain. The mismatch
refusal is HOSTED device with E2EE account. Cryptographic verification of a
NON_E2EE value is separate from receiving admission, whose is_e2ee gate still
refuses it.

## Independent serialization oracle

**Observed reference / inferred APK runtime details:**
[AdvDetailsProtoOracle.java](AdvDetailsProtoOracle.java) reconstructs the exact
GsB descriptor and closed enum on desktop protobuf-javalite **4.36.1**. The JAR
hash is `91d3dba2521322103230c509d41a017da3a5e6766610adcb10c5bb127cbb26c5`,
matching the [existing pinned oracle dependency](attestation-verification.md).
The APK's exact protobuf runtime version and recursion limit remain **unknown**;
this is not execution of the APK or proof of every future runtime variant.

The [637 synthetic vectors](../../crates/whatsapp-api/fixtures/prekeys/2.26.37.71-adv-details-oracle.json)
record serialized bytes and field-presence/value projections from Java. They
cover permutations, explicit zeros, missing/duplicate fields, truncated/long
varints, uint32 wrapping, uint64 high bits, closed enums, mismatched wire types,
unknown scalar/fixed/length-delimited/group fields and malformed/deep groups.
Known fields serialize in ascending tag order with minimal varints; unknowns
follow in lite encounter order, with normalized varints/lengths and recursively
retained groups. Unknown enum int32 values are sign-extended for unknown varint
serialization. Dropping unknowns with a plain prost re-encode is also incorrect.

Reproduce after compiling Java with the verified JAR into LOCAL_CLASS_DIR:

```sh
cargo run --locked -p xtask -- java-oracle adv-details \
  --classpath 'LOCAL_JAR:LOCAL_CLASS_DIR' \
  --output /tmp/adv-details-oracle.json
cmp crates/whatsapp-api/fixtures/prekeys/2.26.37.71-adv-details-oracle.json \
  /tmp/adv-details-oracle.json
```

The [generator](../../xtask/src/java_oracle.rs) uses deterministic inputs and
Java outputs, not expected values copied from the Rust parser. JAR/class files
stay local. Exact regeneration of all vectors was verified, first with the
original Python generator and again (2026-11-01) with its Rust port, which
reproduces CPython's `random.Random(3771)` for the 250 random cases.

## Receiving and remaining boundaries

`VerifiedDeviceIdentity::verify/verify_prekey` now uses the bounded GsB projection
for both signed inputs. Its existing independent primary-pin check, candidate
key from the prekey envelope, two signatures, metadata presence/bounds and
local duplicate refusals remain. Production singleton checks are stricter than
Java duplicate acceptance; the oracle parser tests intentionally include a
broader set than that bounded verifier admission surface.

`listen` and offline `lab-inbox` use this through existing companion admission.
No new HTTP route, request flag, identity rotation, query or retry authority is
added. Existing guards still check current authenticated signed membership,
raw ID/key index/timestamp, own identities, teardown, primary-key fingerprint
and the nested message MAC before atomic first-session/message commit. The
NON_E2EE enum addition does not turn such a device into ordinary E2EE admission.
Companion identity-notification reply adoption and full raw-ID/teardown/state
policy remain unfinished.

Signature tests distinguish canonical input from original noncanonical bytes,
verify preservation of unknown fields/groups, and retain wrong-key/prefix,
damaged signature and malformed/duplicate refusals. The existing real-crypto
known-companion receive fixture now includes reordered LID details with unchanged
signatures/ciphertext and proves decryption, exact companion storage, sender-key
control and duplicate/reopen behavior. No fresh live companion fixture has been
observed for this correction; protocol parity remains bounded by the evidence.

[Hash-pinned recipe](apk/recipes/2.26.37.71-adv-details-serialization.json) and
[retained source mapping](apk/2.26.37.71-adv-details-serialization/targets.json)
contain 10 topics / 15 files / 113,397 bytes. Cached self-diff only checks
retention; it is not a comparison with another APK release.

Validation: 874 workspace/all-feature tests passed (API 733, codec properties 4,
crash matrix 9, CLI 54, API symbols 1, tooling 73). Strict Clippy and warning-denying
Rustdoc passed, as did default CLI (25 plus API-symbol), experimental CLI-reference
regeneration, proto-check, formatting and whitespace. All 637 Java vectors
reproduced byte-for-byte. Exact APK export and cached self-diff passed; hygiene
checked 2,423 links / 122 bundles. Initial stale two-value enum assumption was
corrected after tracing Azc, then vectors/tests regenerated and checks rerun.
The final incoming regression ran in the complete workspace suite.

## Bounded saved-account validation

**Observed:** [redacted validation](adv-details-serialization-live.json) used one
45-second receive-only window (57.05 seconds total) on the rebuilt binary.
Preflight had no queued actions/receipts/eligible outbox; outbox, automatic
reads and reconnects were disabled. Authentication/activation and clean stop
succeeded. Actions (21), receipts (9), eligible outbox (0), account generation,
registration and journal head stayed equal. Inbound checkpoints rose
848→856; other table counts stayed equal. This is an aggregate check,
not byte equality of every private encrypted record.

Incoming outcome logs contained 10 Unsupported controls and zero decrypted,
duplicate or undecryptable messages. No qualifying companion ADV arrived;
live parity remains **unknown**. Historical identity-refresh inspection remains
verified/matching but untracked, force_override_e2ee_state=false, applied=false.
No saved request or identity was reset, no historical guard backfilled, and no
chat was sent. The real-crypto synthetic receive test supplies positive coverage
of the changed branch; the live window proves ordinary receive startup/stop.

The redacted incoming stanza counts are five notifications and five ib controls,
zero message/receipt stanzas. Follow-up CLI-reference/API-symbol, whitespace
and hygiene checks passed (2,425 links / 122 bundles). No Rust code changed in
this validation-record follow-up; the complete implementation checks above
remain its verification. Private key/config/logs and account state stay ignored.
