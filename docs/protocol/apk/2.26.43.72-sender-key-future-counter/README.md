# Sender-key future-counter classification and limits

**Confirmed static:** APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reuses complete checked-DEX [Eyj.A01](../2.26.43.72-group-counter-error-paths/README.md),
[Fj2 constructor](../2.26.43.72-message-version-error-paths/README.md) and
[EtY/F2q/DxB](../2.26.43.72-outgoing-retry-error-map/README.md) exports.
No new APK extraction, target execution or live account operation. These are
local checked exports, not REA Evidence envelopes.

## Native path and ordering

Eyj.A01 selects matching sender-key state and callsFj2.A00 to verify the message
signature at007a..0080, before obtaining incoming iteration v4 and current chain
iteration v2 at0080..0090. Its if-le v2,v4 at0090 selects old-counter/skipped-key
handling versus the forward path. Forward path0192 subtracts v4-v2 into int v1,
loads0x7d0 (2000) at0196, and019a permits v1<=2000. Above2000 branches to0370,
EDY(String) exact Over 2000 messages into the future!. 0Yl.A00 maps EDY to
-1005; EtY refines this exact string to -1202; F2q kind10 and DxB wire6 follow.
A counter gate applied before signature verification would change cause precedence.

## Backend gap: cause and selected state

Backend identities and precise source facts are retained in backend-source-identity.json.
Its group get_sender_key uses u32 counters and MAX_FORWARD_JUMPS=2000, verifies
signatures first through group_decrypt, and returns InvalidMessage(SenderKey,
static reason) on an excessive jump. CBC decryption failure also returns the
same enum variant with another reason. There is no dedicated future-counter
variant. No arbitrary provider error-string parser was added to Rust.

The matching-state accessor is pub(crate) inside the dependency, so our adapter
cannot directly query all saved chain positions by ID. The public front-state
accessor is insufficient for a message targeting an older retained chain.
The public into_components projection is not a passive alternative: it first
fast-forwards the front state to its durable counter-reservation ceiling. A
clone avoids durable mutation but still changes the position being inspected.
Do not derive actual receive eligibility from that projected position or assume
the front state is the matching state. A dependency change or an independently
validated state view would be needed for a complete typed mapping.

## Integer-domain boundary

Native Fj2 copies protobuf iteration_:I into its int field; Eyj uses signed int
if-le/sub-int. Backend SenderKeyMessage and sender chain iteration are u32.
**Inferred from confirmed arithmetic**, with current=0 and incoming bit pattern
0x80000000, the selected native comparison takes the old-counter path (incoming
is negative), whereas backend unsigned comparison takes the forward path and
rejects a jump above2000. Therefore backend excessive-jump alone does not prove
native wire6 at every32-bit input. This is an arithmetic boundary inference,
not observed traffic and not proof of complete native overflow/skipped-key policy.
At small nonnegative counters, differences2000 and2001 select the expected
allowed-forward and future-rejection sides of the selected checks.

**Unknown:** complete high-bit/overflow behavior, skipped-key/version/chain
admission, full direct/self-device equivalence, all generic InvalidMessage causes
and full retry count/key/routing/ACK policy. No runtime code or request changed.
The next implementation must preserve cause precedence, exact matching chain and
integer domain rather than serializing6 for every generic group error.

Reproduction: inspect the linked full Eyj.A01 body and Fj2([B) constructor with
xtask apk-smali APK --class 'LX/Eyj;' --method A01 --signatures --exceptions
(and classFj2 method '<init>'). Recheck backend file hashes before reusing source
line references after dependency updates. This bundle is source/static evidence,
not a backend/native execution harness.

Validation:31 filtered retry tests, APK coverage, staged whitespace and research
checksum/link hygiene passed. No Rust source changed; the full workspace suite
was not rerun for this research-only chunk.

Follow-up: [effective loaded-record metadata view and bounded typed implementation](../2.26.43.72-sender-key-future-retry/README.md) now resolve matching-state access
for confirmed nonnegative signed-int jumps. High-bit/overflow and complete native
parity remain open. The earlier projection warning still applies to choosing an
unadjusted/raw or component-export position without matching load semantics.
