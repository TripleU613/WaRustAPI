# Selected-device encryption attestation verification

2026-10-09. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This closes attestation byte/schema verification inputs for future native
[refresh admission](session-refresh-preview.md). It does not yet adopt a key,
store an attestation state or acknowledge an identity notification.

## Candidate and declared account type

**Confirmed static:** [Bot.A03](apk/2.26.37.71-attestation-verification/attestation-candidate.txt)
reads selected-user `non_e2ee_attestation` (`0248..0254`). With native property
`0x6f6f` enabled, its `account_type`, `value` and `signature` enter CWS
(`0278..02a6`). [CWS.<init>](apk/2.26.37.71-attestation-verification/attestation-carrier.txt)
stores constructor value bytes into A02, signature bytes into A01, and the
string into A00 (`000c..0018`). These field roles cannot be inferred from their
obfuscated names alone. Presence/property branches and explicit override
handling remain as recorded in the earlier session-preview trace.

[CPB.A00](apk/2.26.37.71-attestation-verification/primary-admission.txt)
checks the declared string against A61.A00 (`00fa..010e`).
[A61.<clinit>](apk/2.26.37.71-attestation-verification/allowed-account-types.txt)
constructs that set from `hybrid_e2ee` and `non_e2ee` (`0008..0028`).
[6aj.A1a](apk/2.26.37.71-attestation-verification/allowed-types-array.txt)
confirms the String array has exactly two entries (`0000..0006`).
The attestation signature must contain 64 bytes (`0112..011c`). An arbitrary
string, including `e2ee`, is not an accepted attestation account_type.

## Actual protobuf field and closed enum

**Confirmed static:** [BP2.dynamicMethod](apk/2.26.37.71-attestation-verification/payload-descriptor.txt)
uses the raw `newMessageInfo` descriptor at `0032..0040`, not a public-schema
guess. `xtask apk-proto-fields --class LX/BP2` decodes one field:
**accountType = 1**. [6ap.A1b](apk/2.26.37.71-attestation-verification/descriptor-field-array.txt)
creates the three-object field array with `bitField0_` at zero;
[6aj.A1V](apk/2.26.37.71-attestation-verification/descriptor-field-objects.txt)
stores `accountType_` at one and the verifier at two. Raw field info is `0x180c`.
[Cu0.isInRange](apk/2.26.37.71-attestation-verification/account-type-verifier.txt)
delegates to C29.forNumber and accepts only nonnull results.

[C29.<clinit>](apk/2.26.37.71-attestation-verification/account-type-enum.txt)
and [C29.forNumber](apk/2.26.37.71-attestation-verification/account-type-decode.txt)
establish these wire values:

| Protobuf value | Enum | CPB interpretation |
| --- | --- | --- |
| 0 | E2EE | Invalid attestation account type |
| 1 | HYBRID_E2EE | Must equal declared hybrid_e2ee |
| 2 | NON_E2EE | Must equal declared non_e2ee |
| Other | Not recognized by verifier | Unknown field; retains earlier recognized/default enum |

**These are not C0F stored-state values**, where NOT_SET=0, E2EE=1,
HYBRID_E2EE=2 and NON_E2EE=3. No new stored-state migration is performed here.
CPB parses BP2 (`0120..0130`), rejects default/fallback E2EE (`0130..0152`),
and compares the enum's lowercased name with the declared string (`0152..016c`).
[Ayp.A0S](apk/2.26.37.71-attestation-verification/account-type-string.txt)
uses Enum.name with Locale.ROOT lowercase. These three names are ASCII.

**Observed reference oracle / inferred APK parser behavior:** the retained
[Java oracle](AttestationProtoOracle.java) reconstructs the exact BP2 descriptor,
field array and verifier on desktop protobuf-javalite **4.36.1**. Unknown enum
numbers do not replace the last recognized field value. This follows the closed
enum rule described in the [official Protobuf documentation](https://protobuf.dev/programming-guides/enum/).
It is not equivalent to keeping the last arbitrary integer in a prost enum field.
Java varint32 truncates high bits of enum values, tags and lengths, and consumes
extra continuation bytes. Missing/unknown-only enum input projects to E2EE.
Unknown wire fields and groups are skipped; the signature input still includes
all their original bytes. [Official Java MessageSchema source](https://raw.githubusercontent.com/protocolbuffers/protobuf/main/java/core/src/main/java/com/google/protobuf/MessageSchema.java)
is supporting runtime context; the APK's exact protobuf runtime version is
**unknown**, so the desktop oracle is not claimed as execution of the APK.

The [1,502-case fixture](../../crates/whatsapp-api/fixtures/prekeys/2.26.37.71-attestation-proto-oracle.json)
contains exhaustive length-zero-through-three inputs from an 11-byte alphabet,
plus duplicate/unknown enum values, nonminimal/overflow varints, unknown wire
types, invalid groups, wrapped tags/lengths and recursion boundaries. Every
expected result comes from that separate Java runtime. The first added depth
boundary case exposed a Rust off-by-one: the Java root counts toward its default
limit, so 99 unknown groups pass and 100 refuse. The regression failed before
correcting this boundary and passes afterward. That exact boundary remains a
reference-oracle/local parser bound, not a proven APK runtime limit.

## Signature bytes and primitive

**Confirmed static:** [08F.<clinit>](apk/2.26.37.71-attestation-verification/signature-prefixes.txt)
fills A0J with bytes **06 07** at `0150..015a`, using payload `0250..025a`.
CPB.A00 builds `A0J || original_value_bytes` at `0172..0184` and passes it,
the returned B7E identity and attestation signature to B7H.A09 (`018a..0190`).
[1xh.A1Y](apk/2.26.37.71-attestation-verification/signature-parts-order.txt)
places prefix at index zero and body at one (`0000..000a`);
[1Lz.A05](apk/2.26.37.71-attestation-verification/signature-concatenate.txt)
writes each array in order into ByteArrayOutputStream (`000e..0028`), without
length prefixes. It does not decode/re-encode BP2 to construct the signed bytes.

[B7H.A09](apk/2.26.37.71-attestation-verification/signature-type-check.txt)
requires public-key type 5 (`0006..0010`) and calls the "best" 1NH verifier
with raw identity bytes, concatenated message and signature (`0010..0028`).
[1NH.A00](apk/2.26.37.71-attestation-verification/signature-provider.txt)
selects `org.whispersystems.curve25519.OpportunisticCurve25519Provider` for
that name (`0046..005c`). [1NH.A01](apk/2.26.37.71-attestation-verification/signature-widths.txt)
requires a nonnull 32-byte key, nonnull message and 64-byte signature before
calling 1NJ.verifySignature (`0000..002e`). Complete native/Java provider
internals were not traced in this chunk; it uses the same 1NH primitive family
retained in CQc signed-key verification. Rust uses the existing Signal public-key
verifier. Generated-key tests are not a new independent live attestation proof.

## Rust inspector and remaining admission policy

```sh
whatsapp-cli --profile PROFILE actions --show ACTION_ID
```

**Implemented/offline:** a complete supported selected-device bundle adds an
`attestation` object to the existing redacted inspection; queued, refused or
unsupported/incomplete bundles report null. It contains fixed `status`,
recognized `declared_account_type`, projected `payload_account_type` and nullable
`signature_valid`. No raw value, signature, identity, account/peer/action ID or
unknown account_type string is printed. Absent metadata is explicit absent,
not an attestation success or native stored-state assignment.

The inspector verifies original-byte signatures only after recognized account
types agree. Malformed fields/widths, unsupported declared type, invalid/default
payload type, malformed protobuf, mismatched type and invalid signature remain
distinct. Duplicate attestation/value/signature nodes refuse as a **local**
ambiguity guard; the APK uses its own node accessors, not this duplicate policy.
The payload bound is a **local 64 KiB inspector limit**, not an APK/server rule.
The bounded parser's recursion behavior is reference-oracle-tested as above.

Top-level signature_valid still describes the signed prekey. Nested attestation
signature_valid is a different check. A valid fetched-key/attestation pair can
still disagree with the held pin. Neither check replaces authenticated directory
provenance, [original refresh guards](identity-refresh-guards.md), own-account
protection, companion ADV authority, human verification or native admission.

CPB's existing fail-with-key-change fallback and unchanged-key/stored-state/
property/explicit-override matrix remain in the retained source. They are not
implemented as generic permission to ignore invalid attestation. **Unknown /
unported:** stored C0F provenance and schema defaults/producers, actual native
property values, full admission transitions, automatic atomic session/pin/state
adoption, alias updates and deferred identity ACKs. The existing session preview
still refuses present attestation with attestation_admission_unported, even when
this new mathematical verification succeeds.

Tests bind both supported types to identity, prefix, declared type and original
bytes, including unknown fields and nonminimal/duplicate enum encodings. Wrong
prefix/identity, shortened/re-encoded body and malformed/ambiguous/oversized fields
refuse. A generated full saved directory reply verifies attestation while
pin_matches remains false; the preview still refuses admission, and saved pins,
sessions and journal head compare equal. No real peer identity is replaced.

## Reproducing the independent parser oracle

Download the pinned protobuf-javalite 4.36.1 artifact from the fixture's official
Maven URL into local temporary storage. Verify SHA-256
`91d3dba2521322103230c509d41a017da3a5e6766610adcb10c5bb127cbb26c5`.
Compile AttestationProtoOracle.java with that JAR as the classpath; pipe each
fixture hex value as a separate stdin line into the compiled oracle. Each line
returns its projected integer or malformed. Compare with every fixture
account_type (null means malformed). The JAR and class files remain local;
only compact source and synthetic vectors are committed. No account or network
request is involved in running the oracle after the dependency is downloaded.

The [hash-pinned research recipe](apk/recipes/2.26.37.71-attestation-verification.json)
retains 18 topics / 23 files / 95,100 bytes with signatures, register flow and
exception tables. Exact regeneration and self-diff validate the export; they do
not establish cross-release compatibility or fresh server behavior.

## Retained-account offline inspection

**Observed locally:** the rebuilt experimental CLI inspected the authenticated
primary-LID key read saved by commit `41e66e3`. It still reports verified_bundle,
valid signed-key signature and matching pin, with the historical refresh guard
untracked. Attestation reports **absent**, with declared/payload account type and
attestation signature result null. Absence is not proof of native stored state,
attestation success or downgrade permission. Before/after store inspect JSON
compares equal. No connection, new query, baseline backfill, identity/session
commit or identity notification ACK occurred.
[Redacted aggregate](attestation-inspection-validation.json) contains no account,
peer or action IDs, public keys, signatures or raw wire bytes. The retained
reply did not carry an attestation, so no fresh live attestation signature
verification is claimed. Generated tests establish the positive verification
path; summary equality alone is not a byte-level live account-row comparison.

## Completed checks

862 workspace/all-feature tests passed (API 721, codec properties 4, crash
matrix 9, experimental CLI 54, API symbols 1, research tooling 73). Strict
workspace/all-target Clippy and warnings-denied Rustdoc passed. Default CLI
passed 25 tests plus one API-symbol test; experimental CLI reference regeneration,
formatting and staged whitespace passed. All 1,502 desktop Java oracle results
were independently reproduced. Exact APK export check verified 18 topics / 23
files / 95,100 bytes; checksum-verified self-diff found 18 unchanged topics and
four metadata artifacts. Hygiene checked 2,344 links and 116 bundles totaling
23,830,949 bytes. These checks do not establish the unknown APK parser runtime
version, cross-release parity, native admission or fresh live attestation proof.
