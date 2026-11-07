# Receipt-carried keys: AEA admission and later state update

**Confirmed selected static / research only**, APK2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX fallback exports; PCs are bytes, branch displacements
16-bit units. No target execution, new REA Evidence ID, key adoption, attestation
generation, account traffic or Rust runtime change. Continues the
[receipt-carried key boundary](../2.26.43.72-retry-receipt-keys/README.md).

## Caller binds verification before receipt-key processing

Retained [163.A02](../2.26.43.72-retry-receipt-keys/key-processing-policy.txt)
constructs EoQ at0104..0116 and invokes Em2.A01 at0116. The supplied identity
bytes v27 are copied into v14 at0008 and enter EoQ as its first byte-array
argument. [Constructor](request-constructor.txt) stores that argument in A0A;
[Em2.A01](verification-wrapper.txt) supplies A0A to Em2.A00 along with the
request's descriptor byte, Ejp attestation, device, address category and A08 flag.
The selected caller passes descriptor type5 and false for that first Boolean.
The byte store is retained in raw opcode0x5d form by the exporter; its typed
field reference is preserved rather than silently omitted.

Caller policy requires its own selected prerequisites/session-registration gates.
Its AEA branch is for device0 under feature0x6f6f and an additional missing-Ejp
condition. [DBc.A1S](verification-gate.txt) also reads0x6f6f in the inner helper.
No current value/default is asserted. [Em2.A00](verification-policy.txt) returns
Dtb (not applied on selected branches) for other device indices or its disabled
gate, rather than treating every such case as cryptographic rejection.

The caller checks Dta at011e, logs verification rejected, and returns false
before its subsequent key-processing block. Dtb and DtZ flow differently. The
outer [163.A05](../2.26.43.72-retry-receipt-keys/session-policy-entry.txt) has
additional session/base-key fallback logic, so this false result alone is not a
universal statement about all native resend decisions.

## Signature input is a separate attestation domain

[Accepted attestation type set](attestation-types.txt) contains exact hybrid_e2ee
and non_e2ee. In the selected verification block Em2 requires64 signature bytes,
parses the supplied attestation payload as DXR, resolves its accountType enum,
and compares the normalized enum name against the stanza type before signature
verification. Unknown/invalid account types and type mismatch have distinct
internal result classifications; they are not generic Signal retry wire codes.

[08u constants](domain-constants.txt) initializes A0L to bytes06 07 at0160..016a.
[2L7.A1T](signed-input-order.txt) places that prefix first and the original DXR
payload bytes second in a two-element array. [1nr.A05](signed-input-concatenation.txt)
concatenates in that order. Thus selected signed input is exactly:

`06 07 || original serialized attestation payload`

It is not a reserialized DXR, the signed-prekey public bytes, the whole receipt
or a newly synthesized timestamp. [F3k.A0A](signature-dispatch.txt) requires Ehd
public-key type5, then calls the selected1oA best provider with public bytes,
message bytes and signature. Other key types assert. The provider's full native
algorithm/encoding binding, DXR field schema and trusted identity provenance
remain separate proof requirements; no Rust verification fixture is generated
from an assumed primitive.

## Verification result depends on existing key and stored encryption state

[Actual enum initialization](encryption-states.txt) binds:
A05=NOT_SET,A02=E2EE,A03=HYBRID_E2EE,A04=NON_E2EE. Em2 compares incoming identity
bytes with the stored Ee1.Ehd bytes, and loads previous state from0YT.A0A. The
whole decision therefore cannot be reduced to signatureValid alone.

The complete method retains separate malformed/decode/account-type/type-mismatch/
missing-key/signature-failure classifications. Its policy can distinguish key
change from the same key, a non-attesting-chat result, and sticky non-E2EE state;
selected sticky-state paths log rejection of an implicit E2EE upgrade. Feature
0x7497 also participates in selected rejection/state behavior. Some branches
produce an E2EE policy outcome after an unsuccessful attestation classification;
that is NOT proof the signature verified, nor authority to implement a verification
bypass. Full branch-table parity, preference effects and authoritative state
migration are not claimed here. Rust's existing key-carrying-retry refusal stays
in place until the full policy and provenance are implemented.

## State-update helper follows status0 session-processing result

Back in163.A02, selected accepted paths reach0WJ.A0K at0236 with the derived
state and carried key material. Only its status0 path reaches the later Em2.A02
call at0272 (and only for a nonnull DtZ result). Other statuses have a separate
error path. Selected legacy/PQ cleanup can occur earlier in the caller and is
not conflated with the later AEA update or a safe generic session reset.

[Em2.A02](post-session-state-update.txt) extracts DtZ.Evl.E7b, rechecks the same
feature through DBc.A1S, maps the user address through ExG, and dispatches into
ExG/DDs state operations under further policy including0x7497. Their complete
storage schema, transaction boundaries, crash recovery and downstream effects
remain untraced here. Do not persist a claimed verified/non-E2EE state merely
because the parser supplied an attestation or this helper returned DtZ.

Follow-up: the callee also performs an earlier identity-table AEA write before
returning. The ordering above applies specifically to the later Em2.A02 helper,
not to every AEA mutation. [Session/storage trace and exception limits](../2.26.43.72-retry-session-aea-storage/README.md).

## Current Rust comparison and next proof

Confirmed source: retry_answer explicitly records receipt_keys_unsupported and
returns no send/query for any top-level keys child after its eligibility checks.
That prior tested boundary is retained. Existing peer_verification local marking
is not established as this native AEA state machine, and ordinary directory key
validation is not a substitute. No current runtime key-adoption defect is newly
demonstrated by this research pass.

Next integrate the real receipt-key parser/identity provenance, provider encoding,
complete state-result transitions,0WJ.A0K status/rollback behavior and ExG/DDs
persistence as one owned session workflow, preserving refusal, uncertain-write
and no-replay rules. Do not send fake attestations, disable gates, adopt keys from
an unsupported receipt or regenerate identity to change a result. Full native
receipt-carried-key parity and server/restriction causality remain unproven.

Reproduce apk-smali with signatures/exceptions for Em2.A00/A01/A02,EoQ.<init>,
E7b.<clinit>,F3k.A0A,F1d.<clinit>,08u.<clinit>,2L7.A1T,1nr.A05,DBc.A1S.
Referenced complete same-build caller methods retain their earlier manifests.
[checksums.json](checksums.json) pins all files here.

Follow-up: [later chat classification and local cooldown transaction](../2.26.43.72-retry-chat-aea-storage/README.md)
now retains ExG/DDs storage boundaries, transition guards and transaction paths.
Full policy, callback/cache recovery and whole-retry atomicity remain open.
