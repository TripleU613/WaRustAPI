# App-state patch header audit

2026-11-04. **Confirmed static** uses APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rechecked against the actual local artifact. [Checked field exports](rea/2.26.43.72-patch-header-audit/README.md)
extend [fetch completion](app-state-fetch-completion.md). No network request,
APK execution, source implementation edit or account operation occurred in this
audit. Findings below distinguish existing safe refusal from future processing
work; there is no newly demonstrated active state-promotion bug here.

## Refuted: current crypto decoder silently drops external/exit fields

**Confirmed local source; hypothesis refuted:**
`signal/app_state_collection/patch.rs::CollectionPatch::decode` delegates to
`decode_versioned`. Before Prost parses the payload, its bounded wire scan
returns an error on **any** field3 (`externalMutations`) or field7 (`exitCode`).
It does not silently skip these recognized fields as unknown protobuf data.
This refusal happens before key lookup or MAC verification.

`decode_prepared` uses exactly the same scan. It supplies a claimed version only
for an original locally prepared request that omits the read-only wire version;
if that original protobuf carries a version, it refuses. Its only production
caller is original-request proof in `transport/app_state_sync.rs`, rather than
the opaque incoming fetch classifier. It is not a route around the external or
exit checks. No relaxation of either decoder is justified by this audit.

Both decoders require their supported MAC/key fields, enforce local size/count
and singleton bounds, and reject malformed protobuf. The ordinary decoder also
requires an outer version wrapper with its scalar present. These are stricter
supported-inline-crypto boundaries than the native structural parser; they are
not a full native response classifier.

The existing regression `unsupported_branches_and_malformed_fields_fail` was
run unchanged with `cargo test --locked -p whatsapp-api --all-features
unsupported_branches_and_malformed_fields_fail`: one selected test passed. Its
literal input suffixes include external-reference field3, exit field7 and a
duplicate version. The shared source path establishes the prepared decoder's
same pre-scan; no new automated test or source change was added.

## Exact current field bindings

The extractor follows current `newMessageInfo` objects and helpers:

| Current class | Fields |
| --- | --- |
| `Dh0` (SyncdPatch) | 1version, 2mutations, 3externalMutations, 4snapshotMac, 5patchMac, 6keyId, 7exitCode, 8deviceIndex, 9clientDebugData |
| `DYK` (version wrapper) | 1version |
| `Dba` (exit message) | 1code, 2text |
| `Dg5` (external reference) | 1mediaKey, 2directPath, 3handle, 4fileSizeBytes, 5fileSha256, 6fileEncSha256 |
| `Dbb` (mutation) | 1operation, 2record |

`Dh0.dynamicMethod` additionally identifies repeated mutation element class
`Dbb` at checked0050..0054. Field numbers alone do not establish each nested
scalar/message type, validation policy or current external-download admission.
The retained current F2T/accessor bodies supply the parser branches below.

## Native pre-application validation

**Confirmed, current `F2T.A00` checked DEX:** selected patch bodies are all
parsed into `Dh0` messages at0052..0088, before the later semantic loop beginning
00e8. That loop checks:

| Condition | Native fatal reason |
| --- | --- |
| exitCode present with code100 | 36 (`0x24`) |
| exitCode present with code101 | 37 (`0x25`) |
| other exit code, or exitCode without its code | 38 (`0x26`) |
| version wrapper or nested scalar absent | 8 |
| external mutation ref plus nonempty inline mutations | 14 |
| repeated patch version within the same collection | 9 |
| malformed patch protobuf during earlier parse loop | 18 |

An external-only patch is structurally permitted by this parser, then requires
the separately traced authenticated external-mutation download. Current Rust's
blanket external refusal safely defers an unsupported branch; do not reinterpret
that as native fatal14 or ignore the reference while verifying an incomplete
inline list. Duplicate versions are a collection-level rule; a decoder of one
patch cannot enforce them alone.

The two-pass order matters: a malformed later patch protobuf is found before
semantic checks on an earlier parsed exit patch. Snapshot-reference protobuf
parsing precedes patch-list parsing when both are selected, so complete F2T
failure precedence also depends on the external snapshot-reference parser.
None of these structural checks substitutes for MAC verification, held-state
version sequencing, key authorization or caller/collection/role authority.

## Proposed next structural stage (not implemented by this audit)

The subsequent pure implementation is now retained in
[structural header inspection](app-state-structural-headers.md). It keeps the
cryptographic decoder's existing strict refusal unchanged. The proposal below
records the audit's requirements; actual support and parser-parity limits are
documented with that implementation.

Add a separate bounded header inspector over the structural classifier's
`Data` and error409 `conflict_data` only. Version acknowledgements, already-synced
summaries and errors whose data branch native parsing does not enter must keep
their existing precedence; do not validate ignored opaque payloads as if selected.

1. Preserve original bodies/order and parse **all** selected patch protobufs
   (including recognized nested mutation/record/reference/exit messages) before
   semantic checks. A `skip_field`-only top-level scan cannot claim native
   protobuf-parse parity: malformed nested data would otherwise fail too late.
2. Return typed, redacted structural outcomes: inline/version; external-only
   retrieval deferred; fatal exit100/exit101/other; missing-version, mixed-source
   and duplicate-version failures. Keep numeric native fatal reason metadata
   distinct from actual server IQ error codes; do not invent meanings for36/37/38.
3. Check all collection patch versions for uniqueness before the first per-patch
   mutation-state commit. Ideally validate all selected collection categories
   for an IQ before starting the worker, as F2T construction precedes dispatch.
   Snapshot-reference protobuf validation remains a separately required stage.
4. Keep the existing cryptographic `CollectionPatch` decoder strict. A structurally
   valid inline header still needs complete supported decoding, all held record
   and signing keys, MAC verification and atomic current-state application.
   An external-only header must not reach inline verification by dropping data.
5. Preserve raw originals and unresolved outcomes; report only static reason,
   count/presence and safe version metadata. Never print exit text, references,
   keys or arbitrary names. No download, queue/retry, state promotion or generic
   success/ACK belongs in this structural-only inspector.

Future tests should be handwritten protobuf/Node cases: absent outer/inner
version, exit100/101/other/empty exit, external-only versus external+inline,
duplicate versions across inline/external bodies, malformed later/nested messages,
snapshot-before-patch failure order, source-byte immutability and redacted output.

**Current safety boundary:** structural fetch inspection intentionally leaves
patches opaque and authenticated fetch application is not implemented, so it
cannot currently promote their state. Existing conflict catch-up decodes inside
one write transaction and rolls later duplicate/decoder failures back. A future
per-patch fetch applier must not rely only on sequential crypto failure after
previous commits: native header rejection occurs before that application begins.
