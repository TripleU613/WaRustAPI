# Unsupported retry version error ACK — APK 2.26.43.72

**Confirmed static / bounded implementation correction.** Version code264307238,
APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete checked DEX method exports retain signatures/exception tables. These
are local developer-tool exports, not REA Evidence envelopes. **Observed:** no
account traffic, Android execution or runtime ACK. **Inferred:** the ordinary
non-BX2 receipt branch is the selected compatibility target for Rust's decoded
receipt processing. **Unknown:** full upstream retry handler registration,
BX2 semantics, complete retry-success policy, ACK recovery/replay parity,
remote acceptance and physical socket delivery order across threads.

## Previously missing writer binding

[Prior base/context/error trace](../2.26.43.72-retry-ack-dispatch/README.md)
identifies0Sf.A07(false) →1i8.A03 → Android Message(arg1=76,obj=Eoh) →0Sf.A0V.
A0V invokesA09(message,this,false,false). In its ready branch A09 invokesA05;
its not-ready branch usesA00 for pre-ACK bookkeeping. A05 likewise callsA00
when not bypassing that bookkeeping, then hands an obtained message to1A0.CdY.
A00 calls1i8.A07; the previously retained
[conversion](../2.26.43.72-prekey-upload-replies/prekey-ack-conversion.txt)
selects76 at0014..001c →0202, usesF3e.A05(context.child,context) at0214, and packs
its output/loggable ID into1vZ. The original message can instead reach its writer
when conversion returns null; do not assume every send remains Message76.

The selected ready channel is1A3:0ZJ.A15 stores its0vH writer channel and constructs
1A3 for0YS.BqT. Its CdY sets what=4 and submits to its Handler; handleMessage
case4 at014a forwards to0ZJ.A0t →A0r. A0r obtains the message and invokes its
0vH.CdY. Writer0v0.onLooperPrepared constructs0vI and calls0uv.CPl;0uw.CPl posts
what0 with that channel;0uw.handleMessage calls0ZJ.A16 →A15. 0vI.CdY sets what2;
its handler case2 invokes0v0.A00. Writer A00 handles arg1=76 at11b8..11c8,
casts obj toEoh and calls0v2.A02. The previously retained
[producer](../2.26.43.72-prekey-upload-replies/prekey-ack-wire-producer.txt)
callsF3e.A05 then1rj.A04. Both direct and converted paths therefore retain the
ACK-builder semantics for the selected branch. Existing channel-ready owner
plumbing is retained in the iq-shutdown-eligibility bundle. Readiness, callback
and writer state can defer/drop work; this is submission, not server acceptance.

## Ordering correction to the investigation

160.A04 buildsFe5 at0668..06a2, puts it in14h.A09 at06a6..06ac, and returns via
its true path. 14h.A00(non-BX2) calls0Sf.A0O at0156, then jumps to00fe, removes
that context's Runnable fromA09 and runs it at0112. The selected handler thus
submits its ACK before invoking this deferred retry runnable. It does not prove
that socket ACK bytes finish before every independently queued operation. No
universal resend-before-ACK rule follows from this evidence; successful retry
ACK integration and durable work handoff remain separate work.

## Narrow version-refusal correction

F10.A04 first requires retry and registration, validates registration length4,
then reads v with absent default1. A non-1 version returnsnull at004e before
recipient/count/t/id/keys parsing. 160 returnsfalse;14h calls2Cx, which adds
error487 to the receipt context. F3e copies its extra attributes into the ACK.
Do not confuse2Cx's separate diagnostic490 with the error code. The unhandled
context preserves root id/from/participant/recipient/edit; it differs from the
LIL parse-error context. Sources are retained in prior linked bundles.

Rust now constructs this error ACK through the shared confirmation helper only
for explicit text v !=1 with four binary registration bytes and supported root
correlation fields. It does not reuse parse_retry(None) as an error classifier.
Nested id/t/count/keys are intentionally not prerequisites for this branch.
Absent/v1, malformed registration, non-text versions and unsupported context
fields remain unacknowledged by this correction. Supported retry success and
other malformed paths remain incomplete. Existing callers retain capture/store
admission before confirmation; this change adds no key query/session mutation.

## Reproduction and validation scope

Use `target/debug/xtask apk-smali <pinned-apk> --class 'LX/CLASS;' --method METHOD
--signatures --exceptions` for each exported signature. channel-send/ready-observer
use class filterLX/ with methodsCdY/CPl to retain concrete alternatives.
`apk-callers` inventories0v2.A02 and0ZJ.A15/A16; `apk-field-refs --class 'LX/0ZJ;'
--field A04 --type 'LX/0vH;' --writes-only` inventories its selected writer field.
The failed attempt to omit apk-smali's required class filter produced no evidence;
it was replaced by the explicitLX/ filter, not interpreted as class absence.

Regressions compare ACK fields and codec round-trips for versions empty/0/2/01,
including invalid later count and missing nested id/t, and preserve no-ACK policy
for missing/v1, local carried-key/high-count limits, invalid registration and
invalid root correlation fields. Codec round-trips use the existing comparison
37.71 dictionary (identical tokens in43.72); they do not execute the APK or prove
server acceptance. Checksums cover every other file in this bundle.
