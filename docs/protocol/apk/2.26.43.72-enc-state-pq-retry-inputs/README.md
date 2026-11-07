# Encrypted state/PQ inputs to retry preparation — APK2.26.43.72

**Confirmed selected static / research only**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX fallback exports and same-build retained methods.
No target execution, new REA Evidence ID, Rust runtime change or account traffic.
Method PCs are bytes; branch displacements16-bit units.

## Exact spellings produce distinct booleans

The retained [EIM.A00 parser](../2.26.43.72-encrypted-child-metadata/enc-descriptor-parser.txt)
reads state at0032 and compares it to exact false at00ee..00f8. It reads
session_type at00d2 and compares to exact pq at00de..00e8. It passes these values
as v8/v9 into the [Ezj constructor](../2.26.43.72-encrypted-child-metadata/enc-descriptor-constructors.txt):
state=false becomes A06; session_type=pq becomes A05. Missing/other spellings
produce false in these selected comparisons; they are not parser-invalid merely
for differing from those two special literals. No case-folding/boolean grammar
or correspondence to internal Signal version is inferred.

[Exact A06 census](stateless-descriptor-refs.txt) has15 direct sites, including
constructor, context copying, retry, outgoing and VoIP consumers. [A05 census](pq-descriptor-refs.txt)
has12 sites. These are bounded direct DEX references, not all runtime resolution.
The remaining consumers are not all newly traced. In particular these flags must
not be turned into generic refuse/decrypt switches based only on their names.

[Context copy Dxs.A0R](context-copy.txt) reads selected Dxs.A09 at0088; when A06
is true it sets18U.A15. A05 also conditionally adds a bit via feature policy.
This proves preservation into another selected context, not the full meaning,
transaction durability or wire effect of18U fields.

## Native retry job names the inputs

[F1Z.A0N](pq-predicate.txt) reads the A09() selected descriptor and returns its
A05==true. The [direct caller census](pq-predicate-callers.txt) has10 invokes,
including E5U, message logging, retry-job construction and other paths. It does
not inspect ciphertext bytes or derive this flag from the Signal version.

The previously retained [SendRetryReceiptJob constructor](../2.26.43.72-outgoing-retry-timestamp/job-constructor.txt)
reads selected descriptor A06 at011e, stores isStateless at012a, then uses A0N
to store isPqSession at0136. Thus these are native retry-job field names, not
names invented from obfuscation. New exact [isStateless refs](stateless-job-refs.txt)
find constructor write and job A0L read; [isPqSession refs](pq-job-refs.txt) find
constructor write and Ff4.call read.

The fresh E5U.A0B export was byte-identical to retained
[decrypt-retry-dispatch.txt](../2.26.43.72-outgoing-retry-error-map/decrypt-retry-dispatch.txt);
no duplicate was committed. It checks A06 at001e, obtains feature0x53e5 through
A0a and2L9.A1N, and conditionally checks0x5d4c. Along with F1Z.A04() retry count,
this controls whether key material is loaded before creating EXs/EuT and calling
EuT.A01 with mapped cause. Its PQ branch uses F1Z.A0N and Ex8.A01 before an
optional0WJ.A0a load. [2L9.A1N](positive-predicate.txt) returns its integer argument>0. For the
selected stateless policy, the extra preparation switch is
`isStateless && featureInt(0x53e5)>0 && featureBool(0x5d4c)`; the selected
key-load branch is used when this switch is true OR retryCount>0. The
minimal branch is used when retryCount<=0 and the switch is false. This
describes branch selection, not successful key availability. Actual feature
values/defaults, Ex8 enum semantics and
local key provisioning are not established here. Do not assume every stateless
message requires keys, forbids keys, or skips decryption.

## Job-time preparation rechecks policy

The retained [job A0L](../2.26.43.72-outgoing-retry-timestamp/job-run.txt) checks
isStateless at00aa, the same policy inputs, and retryCount. Its selected minimal
branch constructs EYI with null key inputs (0150..018c). The other branch creates
Ff4 with discriminator10 at01bc..01ca and executes it as retry_receipt_keys.
Job-time behavior is therefore not fully established merely by earlier receipt
queue admission or an input flag. Queue completion is not socket/server success.

The complete new [Ff4.call](retry-key-callable.txt) has many discriminator branches;
ONLY branch10 at004a is classified here. At008c it reads isPqSession; when true
it consults Ex8.A01 and selected02X values. Only the selected policy route calls
0WJ.A0a at00dc, otherwise its corresponding optional input is null. It loads
0WJ.A0c/A0d and builds EkF at0196, returning a pair with0WJ.A19 bytes. The job
unpacks that pair at01da onward before EYI construction. Exact key family,
encoding, signature input and regeneration/lifecycle effects need their own
source binding; no new KEM field or invented material is authorized by this trace.

## A rejected ordinary-message candidate is retained

[Eny.A00](voip-decrypt-candidate.txt) also reads both fields and constructs a
mode based on them. Its signature takes Dxg and its own diagnostics explicitly
say voip/encryption/decrypt-stanza and decryptCallPayload. It is a VoIP candidate,
not evidence that the ordinary chat Ed7 path uses the same mode selection. The
complete export is retained so this tempting but unproven substitution is not
repeated. Other census sites remain unclassified.

## Rust comparison and next proof

**Confirmed Rust source:** receipts::queue_retry_for_incoming_with_cause currently
builds the bounded first retry with saved registration, original timestamp and
selected cause through messaging::retry_receipt_with_error; it does not interpret
these enc attributes or implement this native key-preparation policy. This is a
compatibility gap, not proof its minimal branch is always wrong: native has a
conditional minimal branch too. The existing once-only route reservation and
uncertain-outcome no-replay policy must survive any future integration.

**Unknown / next:** feature defaults and current applicable values, Ex8 enums and0WJ key getters/lifecycle, EYI wire binding for
these selected values, and how the bounded Rust retry should select/persist its
native inputs. Do not add state/session_type attributes to the outgoing retry
just because they occur on incoming enc. Do not drop authenticated payloads or
infer server/account restrictions from these flags. No runtime gate, key
regeneration, retry replay, new ACK or live operation is added.

Reproduce apk-smali with signatures/exceptions for Dxs.A0R,F1Z.A0N,Eny.A00,Ff4.call,2L9.A1N;
apk-field-refs selects exact Ezj.A06/A05 and retry-job isStateless/isPqSession,
type Z; apk-callers selects F1Z.A0N. Earlier referenced bundles retain full
methods/manifests. [checksums.json](checksums.json) pins all files here.
