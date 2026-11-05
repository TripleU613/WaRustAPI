# Group old counters and key-error producer boundaries

**Confirmed static**, APK 2.26.43.72, code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX exports, not REA Evidence envelopes. No runtime/account traffic.

## Sender-key old counter

Eyj.A01 is the sender-key decryptor: loads0YK sender-key record, verifies/decrypts
and writes updated record viaCrS. At016c..018e it builds “Received message with
old counter: ” and throws ED4. Its two selected callers are0Yl.A00, whose ED4
catch handler code-unit0056 (byte00ac) supplies -1001 at00ae toEHq.A00.
Ed7.A00 is the caller inventory boundary above0Yl, not a complete entrypoint map.
The [previous EtY/F2q trace](../2.26.43.72-outgoing-retry-error-map/README.md)
excludes -1001 from selected retry eligibility.

**Inferred compatibility mapping / implemented offline:** libsignal group
DuplicatedMessage now preserves SignalOldMessageCounter, reaching the existing
inbox Unsupported boundary: no automatic retry or delivery ACK, no committed
ratchet/event mutation. Authenticated durable same-ID duplicates are unchanged.
Group errors reuse direct's variant-only fixed diagnostics, relabelled
group_crypto, instead of formatting provider payloads. This sanitization does
not assign wire retry codes to other variants.

## Key errors are producer-specific

F22.A04 prekey decrypt setup creates EDR for absent one-time key (059a..05b2),
failed one-time record load (0570..059a), invalid signed-prekey record
(05e8..0614) and absent signed prekey (0614..062c). The previously retained
0WJ.A0O catches EDR as -1003, eligible internal3 -> wire3.
F22.A04 missing PQC key instead constructs EDQ (05b2..05c2).

Ecx.A00 session builder creates EDQ for invalid device-key signature
(0054..0064), invalid Kyber-prekey signature (00a4..00b4), and no signed prekey
in the supplied bundle (0510..0520). These are not automatically the same
producer as native exact “Invalid signature!” EDY/-1005 refinement to -1203.
No single Rust SignatureValidationFailed -> wire5 mapping is justified by these
exports. Session-builder call policy remains untraced here.

Eyj.A00 group encryption has EDR “No key state in record!” and EDQ “Session
missing signature key!”; these outbound failures do not establish receive retry
policy. Eyj.A01 group decrypt creates EDR “No keys for: ” (0344..035c), but its
exception table catches EDR and EDQ and wraps them as EDY at0380..038e. EHq/
EtY receives the wrapper's message at status -1005. EDY(Throwable) delegates
to Exception(Throwable), rather than explicitly forwarding the original message.
The inner “No keys for: ” text therefore does not establish a matching outer
startsWith refinement. This export does not justify wire2 for that branch.
A global EDR -> wire3 mapping would also lose the group wrapper distinction.
Eyj.A01 also constructs EDS for an absent sender-key record and EDY for a future
counter;0Yl maps EDS to -1008 and EDY to -1005. Full libsignal equivalence and
native producer coverage remain unknown; no new wire error serialized here.

## Reproduction and limits

Use target/debug/xtask apk-smali APK --class 'LX/Eyj;' --method A01
--signatures --exceptions; substitute classes/methods named by each export.
Use apk-callers APK --class 'LX/Eyj;' --method A01 (or0Yl/A00) for inventories.
Full selected methods/exception tables retained. Constructor inventories are
in the preceding old-counter bundle. No guessed names from class labels.

Offline tests replay actual sender-key ciphertext against the advanced record
and under a fresh inbox stanza ID, verifying typed refusal, unchanged sender
key/journal and existing consumed-message duplicate handling. Diagnostics test
covers address/application/backend markers. Complete native old-counter ACK
policy, full error-code/key/count/routing progression and server acceptance
remain unverified. Zero bans is not established.

Validation:1328 workspace/all-feature tests passed, as did formatting, strict
workspace/all-target/all-feature Clippy, CLI reference regeneration, APK coverage
and research checksum/link hygiene. Offline only.
