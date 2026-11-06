# Current group-info MEX request and timeout boundary

**Confirmed static**, APK 2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX text exports, not REA Evidence envelopes or observed runtime.
Commands: `target/debug/xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions`; omit method for complete helper/model/operation
classes. `apk-callers` targets the helper A00. `apk-field-refs --class 'LX/0iq;'
--field A00 --type J --writes-only` is the physical timeout-writer census.
Offsets below are method-relative bytes; jumps are two-byte DEX code units.

## Helper builds GraphQL group info, not the classic query

GetGroupInfoProtocolHelper.A00 calls 0co.A02(group,"interactive") at0090..009a,
then 2LA.A0Y ->0ig.A01 constructs a new0iq operation. 2LE.A0Q ->0iq.ARv/AQY/AQV
connects it to the callback and dispatch path. After await, the helper converts
72N using 0co.A03 with 87J(...,"group_response",false,false), calls 0wE.A0e,
and invokes onSuccess. Its exception branch catches Exception, extracts a Giw
error list's first code when applicable, and invokes BvV. Full conversion,
callback implementation and scheduler/retry policy remain untraced.

0co.A02 creates input.group_jid from getRawString and input.query_context.
Ordinary "interactive" maps to "INTERACTIVE". A selected 0cv.A00-set membership
branch substitutes "MISSING_PARTICIPANT_IDENTIFICATION". It constructs0iT with
call name QueryGroupInfo, resolved build config whatsapp-android-mex, tree type
72N and false final boolean. Other recognized context strings are retained in
request.txt; no claim of complete runtime group eligibility follows.

## The tempting30-second literal is discarded

Helper.<clinit> calculates SECONDS.toMillis(30), then returns without storing
or using the returned long. This retained bytecode cannot establish its timeout.
The actual newly constructed0iq initializes A00 to0x7d00 =32000 at0080..0088.
The selected helper does not call a timeout override;0iT.setNetworkTimeoutSeconds
throws UnsupportedOperationException. The exact physical A00:J writer census
contains8 writes across6 methods, including construction and unrelated
producer overrides. It is not a reflection/native/runtime mutation census or
proof every GraphQL operation always has this duration.

## Selected operation reaches MEX IQ with32000ms

0iq.A00 selects RsH if A03 is true; BZr sets that flag. A new operation defaults
false, and the retained helper path does not call BZr, so this selected path
constructs0j3 with A00 at00dc..017a.0j3 stores the passed long as its A00.
0j2.A04 checks that A05().A02() has a query identifier before invoking A06;
otherwise it reports LI6 through Bx5 and does not dispatch in that branch.
0mF.A02 returns its A04 identifier field.0j2.A03 obtains that identifier
through0ih.A00(callName, resolvedBuildConfigName), rejects absent/empty output,
and0ih resolves0ik.A00(config).CRh(callName), logging and returning null on
Exception. Concrete config implementation/identifier data is not re-traced here;
existing Rust MEX query support remains a separate comparison boundary.

0j3.A06 has another conditional Pando route selected through0io. That branch's
transport/timing is not established by the MEX call below. In its non-Pando
branch,0132..0148 calls0Sf.A0W(callback,node,generatedID,341,A00). Thus the
selected unchanged default is32000ms in that MEX branch, forwarded to the
existing native IQ timer mechanism. It is not proof every group info operation
or Pando path has the same effective deadline.

0mF.<init> serializes JSONObject {queryId,variables} from the request's parameter
map and constructs0mK.0mK builds iq/type=get/xmlns=w:mex with to=0mL.A00,
optional query metadata, a query child with query_id when validated, and JSON
text when admitted. The concrete destination singleton serialization, resolved
QueryGroupInfo identifier, every0Tk string rule and Pando metadata remain
outside this trace. Do not invent a query ID or HTTP endpoint from call name.

## Caller scope and Rust port boundary

The exact helper A00 caller census has2 physical methods:8fK.invokeSuspend
(the continuation) and8iq.invokeSuspend. The selected8iq selector7 call excerpt
is retained with its full-export hash; it is explicitly partial, not a full
caller/cancellation proof. Source-file string anchors had no instruction xrefs;
class descriptor inspection resolved the real method directly.

Rust groups::info_request still builds its existing inferred classic w:g2
query/request=interactive shape. This trace does not authenticate that shape
as the current helper's native writer and does not justify simply applying the
MEX32000ms duration to it. Normal-outbox preparation's30-second guard remains
an explicitly local policy; generic LiveReplies claim-based expiry remains
separate. No wire/timer change, request replay or live account operation was made.

**Unknown / next work:** complete native classic group-info producer/callers if
present, MEX identifier/config selection, response conversion and comparison
with existing Rust MEX support, all branch gates and native recovery/cancellation.
Older37.71 vocabulary evidence remains attributed to its build; this current
trace qualifies any implication that vocabulary alone proves classic parity.

`checksums.json` records bytes and SHA-256 for every other bundle file.
