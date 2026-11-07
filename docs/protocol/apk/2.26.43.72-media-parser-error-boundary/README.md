# Media parser failure reason to conditional error ACK — APK 2.26.43.72

**Confirmed selected static / research only**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports via xtask fallback and referenced same-build retained
writer evidence; no target execution, new REA Evidence IDs, account traffic or Rust
runtime change. Byte PCs are method-relative; branch displacements use16-bit units.

## Typed parser reasons are not HTTP status or generic retry causes

[Complete Dxp class](parser-errors.txt) extends EDP and stores the constructor's
integer directly in e2eFailureReason. Factory A01 returns13 (size), A02 returns16
(missing key), A03 returns17 (MIME), A04 preserves its caller integer. The selected
[image](../2.26.43.72-media-image-ingress/README.md),
[video](../2.26.43.72-media-video-ingress/README.md),
[audio](../2.26.43.72-media-audio-ingress/README.md),
[document](../2.26.43.72-media-document-ingress/README.md) and
[sticker](../2.26.43.72-media-sticker-ingress/README.md) parsers supply14 for
malformed hashes on selected paths. These integers are parser failure reasons;
they must not be sent directly as HTTP status or Signal retry-cause values.

[Exact field census](error-reason-refs.txt) finds23 direct DEX reads/writes of
Dxp.e2eFailureReason. This bounds direct static references, not all runtime/reflection
or native consumers. Early candidates [16C.A05](edited-message-consumer.txt),
[16C.A06](edited-message-report.txt) are EDITED-message paths; [1wC.A03](error-consumer-1wC.txt)
is an orphan resolver. Their complete outputs are retained but not substituted
for ordinary plaintext ingress merely because they read the same exception field.

## Shared plaintext processing preserves the reason

[EdS.A00](receive-error-consumer.txt) is marked SharedMessageProcessor/handlePlaintext
in its own diagnostics and has distinct parse/validation catches. Its Dxp catch
at130c..133e reads the original e2eFailureReason and calls F0b.A01 with it.
[F0b.A00](alternate-error-consumer.txt) also catches BadE2eMessageException, passes
its reason through a typed result, and distinguishes DiscardWithDeliveryReceipt
and other exceptions. Not every unknown/unsupported message follows the same
failure branch. Full upstream supported-profile binding remains untraced.

[Failure dispatcher F0b.A01](failure-dispatch.txt) clears FUe.A02 (current message)
and conditionally enters ExA reporting/action when F0b.A00 is true. It invokes
ExA.A02 then calls0x0.A0b ONLY on the selected path where F1Z.A03 (logged as
isSendRetry) is false and F0b.A06 is true. Other branches can retain the F0b.A00
flag or bypass action. Those fields' complete producer/lifecycle semantics are
unknown here; do not turn this conditional path into a universal ACK rule.

## Reason mapping and optional original reason are separate

[ExA.A00](reason-map.txt) maps the original reason:

| Original reason | Mapped error integer |
| --- | --- |
| 34 | 500 |
| 106 | 420 |
| 107 | 421 |
| every other value in this selected helper | 491 |

Thus13/14/16/17 map to491, not487, in this helper. These are protocol ACK error
attributes in the selected construction chain, not HTTP response statuses or
proof of an account-restriction cause. The helper itself does not validate the
entire reason domain. [ExA.A02](reason-observation.txt) separately records failure
observations, with special classifications for other reasons; its full telemetry
semantics are not classified as network protocol here.

F0b.A01 passes BOTH boxed original reason and mapped integer to
[0x0.A0b](reason-action.txt). That action records the mapped value through F0A.A05,
then separates a BX2-specific branch using2Dx/1ic.A0G from the ordinary envelope
branch. The former's full job behavior remains unknown; it is not equated with
the direct envelope path.

On the non-BX2 branch, mapped error becomes a decimal string. With a nonnull
original reason, [F1Z.A0B](error-envelope-builder.txt) builds Efa with class=message,
id/type/address/timestamp from its original context, sets the error attribute via
[Efa.A01](error-attribute-setter.txt), and attaches a `meta` child containing
`failure_reason=<original reason>`. With null reason it omits that child. Sender/
participant choices are context-controlled; no arbitrary address fabrication is
inferred. [Efa.A00](error-envelope-finalize.txt) preserves map attributes and child
in [Eoh StanzaKey](error-envelope.txt).

## Selected queue and ACK builder binding

[1i8.A02](ack-job-builder.txt) obtains Android Message with what0,arg1=129,obj=Eoh.
[0x0.A09](error-job-submit.txt) passes that message to0Sf.A0V. The previously
retained [1i8.A07 conversion](../2.26.43.72-prekey-upload-replies/prekey-ack-conversion.txt)
compares arg1=129 at001c..0024 and reaches0202, casts Eoh, and invokes F3e.A05
with its optional child at0214. Its field-byte math is0020 +0xf1*2 =0202.
The same-build [ACK builder](../2.26.43.72-prekey-upload-replies/prekey-ack-wire-builder.txt)
emits root ack with context class/type/addresses and appends extra attributes plus
optional child. Thus the selected converted ACK can carry error491 and meta
failure_reason17 for the MIME example, PROVIDED the upstream predicates selected
that action. Queue/readiness/conversion submission is not socket delivery or
server acceptance. Direct unconverted arg1=129 writer dispatch is not newly proven
by the previously retained arg1=76 direct writer path; do not conflate them.

This is separate from receipt-parser count/version error487 and separate from
decryption retry receipt construction. An invalid application media field must
not inherit a Signal missing-session/key retry simply because it has a numeric
reason, nor may every unsupported payload be called invalid native media.

## Rust comparison and next required proof

**Confirmed Rust source:** direct inbox processing atomically persists decrypted
plaintext, publishes its local event and returns IncomingResult::Decrypted. Its
UnsupportedMessage event projection is not a typed native Dxp classification.
No corresponding media application error491/meta reason branch was established
in Rust. This is an incomplete compatibility boundary, not justification to send
491 for every local unsupported feature or discard authenticated ratchet state.
The earlier MIME preflights protect explicit retrieval/cache authority; they did
NOT implement this receive-response family.

Before runtime integration, trace F0b.A00/A06 and F1Z.A03/BX2 input ownership,
exact supported native media parser conditions versus local feature limits,
authenticated processing/durability order, direct/converted writer behavior and
retry/defer lifecycle. Preserve capture/ratchet/result ownership and error-specific
no-replay rules. Full wrong-reason, header, server-acceptance and restriction
causality remains unknown. No ACK, retry, auto-recovery or live operation is added.

Reproduce `apk-smali <apk> --class '<descriptor>' --method <name> --signatures
--exceptions` for16C.A05/A06,1wC.A03,EdS.A00,F0b.A00/A01,ExA.A00/A02,0x0.A0b/A09,
F1Z.A0B,Efa.A00/A01,1i8.A02; export Dxp/Eoh without method filters.
Use `apk-field-refs <apk> --class 'LX/Dxp;' --field e2eFailureReason --type I`.
[checksums.json](checksums.json) pins all new files. Referenced writer files have
their own previously verified bundle checksum manifests. No ban/zero-ban claim.
