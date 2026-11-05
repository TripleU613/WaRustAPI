# Outgoing retry timestamp — APK 2.26.43.72

**Confirmed static / bounded local correction.** Version code264307238, APK
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected checked-DEX exports retain signatures/exception tables. These
are local exports, not REA Evidence envelopes. **Observed:** no Android runtime,
account traffic or retry acceptance. **Inferred:** supported Rust direct/group
retry receipts should carry their failed message's original seconds value.
**Unknown:** full upstream parser/admission registration, native numeric grammar/
range parity, error-code classification, extra keys, special routes and count/
recovery progression. This does not complete the outgoing retry protocol.

## Selected original timestamp chain

FVv.ABJ's Dyc-producing branch reads root t via8B6.A0J at06b6..06e8 (range inputs
0x5e0c5180..0xf486c780), takes its numeric seconds at06f2 and passes v38/39 to
Dyc's constructor at088c. Dyc stores that last long inA00. Ewu.A03 readsDyc.A00,
multiplies by1000 at00d8 and forwards it toEwu.A02. The Dyb-producing branch of
FVv likewise reads root t at0446..0488, forwards its value via v35/36 at0582,
and Dyb stores it inA00; Ewu.A04 multiplies this field by1000 at00ac.

Ewu.A02 boxes its supplied timestamp intoEdc.A0M. Edc.A00 takes that required
Long into v39/40 and passes it as Dxr's timestamp at0176..017c (see full signature/
register moves); Dxr forwards it toF1Z's constructor, which stores it inA05.
This is distinct from other observation/loggable timestamps in these envelopes.
The selected1rR.A00 helper likewise forwards its supplied long intoEdc.A0M;
its separate08q clock read is not evidence of replacing that message timestamp.

SendRetryReceiptJob's constructor copiesF1Z.A05 into its named timestamp field
at00f6..00fe. ItsA0L passes timestamp throughEYI's long argument (both retained
construction branches), andEYI stores it inA06. EuT.A01's direct EYI construction
also readsF1Z.A05 at02e2 and passes it forward. F3e.A08 readsEYI.A06, calls2LB.A06
at0134 and writes retry/@t from the result. 2LB.A06 performs signed long /1000.
No selected writer/job replaces this with the retry creation clock.

[Writer0v0.A00](../2.26.43.72-retry-version-error-ack/writer-dispatch.txt)
case11 at0448..046a castsEYI, callsF3e.A08 and sends via1rj.A04. The separate
call-rekey writer at1aac is not this message-retry path.

## Additional confirmed gaps, not guessed values

F3e.A08 always adds retry/@error fromEYI.A04 at014e..0162. EYI's constructor
stores its corresponding integer argument there, and job/direct producers carry
retryErrorCode from their own admission/classification. Rust's generic
Undecryptable result currently merges multiple causes. This chunk records the
missing error field and producer gap; it does **not** invent error0 or transplant
receipt ACK error487 into this unrelated field.

The job sends stored retryCount+1 and conditionally supplies extra key material.
Rust's current automatic path remains one bounded count1 request without those
unported producers. Registration stays exactly the saved four-byte value;
no key upload, identity change or new count progression is introduced.

## Search boundaries and negative results

EYI has three constructor callsites, andSendRetryReceiptJob has one, inEuT.A01.
F1Z has Dxr/Dxs constructors; only the message Dxr chain is traced here, not the
separate Dxs family. Edc timestamp field is **A0M:java.lang.Long**, notA01:J:
the initial incorrectly typed A01:J writer search returnedzero; the corrected
A0M inventory foundseven producers. Do not mistake that zero for absence of
message timestamp writes. Retained1rf.A05 is another protocol reader, using
7NU.A01*1000, not independent proof of the untraced7NU timestamp source. Unread
inventory producers remain unknown. The selected full bodies preserve these
boundaries without committing an APK or a bulk unpacked tree.

## Local correction and tests

queue_retry_for_incoming now parses the original root t as a signed64 decimal
using the existing Java-number helper. It refuses absent/unusable or millis-
conversion-overflow inputs before any queue transaction. It passes those
original seconds toretry_receipt; the public builder's parameter is named
message_seconds to make that meaning explicit. No current-clock fallback remains.
This is not full8B6 grammar/range admission parity: the selected parser range
above and its wider routing/error policy are not universally applied here.

The new store/reopen regression proves original seconds survive queueing and
serialization, a leading plus canonicalizes, invalid/missing inputs write no
checkpoint, and changed timestamp on redelivery cannot reset an uncertain held
request. The actual receive-loop write-cut test now asserts the emitted retry
contains the known original timestamp for direct/group variants, then retains
its no-replay guarantee. Existing queue/alias/historical/late-key fixtures now
supply valid timestamps. A historical changed-metadata fixture uses a changed
valid t instead of nonnumeric text, preserving its deduplication purpose;
malformed timestamps are covered separately. No live socket or APK execution.

Reproduce selected signatures with `target/debug/xtask apk-smali <apk> --class
'LX/CLASS;' --method METHOD --signatures --exceptions`; full Java job class names
are in exported signatures. apk-callers retains the selected constructor/factory
inventories; apk-field-refs onEdc.A0M with typeLjava/lang/Long; and--writes-only
reproduces the corrected field inventory. The initial wrong query is retained
and explicitly superseded. Checksums cover every other file here.
