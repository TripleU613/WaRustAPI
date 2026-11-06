# Missing participant-identification context lifecycle — APK 2.26.43.72

**Confirmed static**, checked DEX exports, not observed runtime or REA Evidence
envelopes. APK SHA-256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands use `target/debug/xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions`, `apk-callers` for0cv.A00/A01 and0zu.A02,
`apk-field-refs` for82U.A0Y:Z writes and `apk-xref --anchor
groups_with_missing_participant_ids`. Offsets are method-relative bytes;
branch deltas are two-byte DEX code units.

## Request context consults persistent group state

The preceding [current group-info trace](../2.26.43.72-group-info-mex-timeout/README.md)
shows0co.A02 calling0cv.A00 then overriding query_context with
MISSING_PARTICIPANT_IDENTIFICATION when the supplied group is in that set.
Rust's offline MEX builder already has the same explicit GroupQueryContext
variant. This does not establish that Rust automatically maintains the native
set or selects that override under the same circumstances.

0cv.<init> lazily obtains SharedPreferences. A00 reads string set
"groups_with_missing_participant_ids", uses an empty set fallback, parses each
string through GroupJid.Companion.A03 and discards null parse results. A01
replaces that preference with supplied JIDs' getRawString values converted to
a set and calls Editor.apply. This is asynchronous preference persistence,
not proof of an atomic/fsynced group database transaction or an in-memory-only
flag. No expiry or time comparison appears in these two methods.
Exact preference-string references found only A00/A01. Physical A01 callers
are0wE.A0b and0zu.A02; readers are those two plus0co.A02. This is not proof
against reflection, native code, preferences reset or other indirect mutation.

## Marker presence adds; absence alone does not clear

0zu.A02 receives a GroupJid and node. It checks for a direct
missing_participant_identification child at0008..0018. If present it copies
the stored set, adds the supplied group and writes it back (002c..0048).
If absent it returns without removing anything. Presence, not parsed child
text or arbitrary missing member fields, controls this selected method.

Exact callers are0w9.A09 and19k.A09. The retained0w9 prefix shows a supported
sender conversion, feature gate0x36fe and nonnull group before the call at007a.
The rest of that large dispatcher is intentionally not exported as this selected
excerpt; its full-export hash is retained.19k.A09 is retained completely and
has conditional dispatch before its027a call. Full notification registration,
all admission/authentication guards and effective gate values remain unclosed;
these call sites do not authorize applying this marker to every inbound node.

## Group-info application both adds and removes

0wE.A0b starts by reading/copying the same set. It takes the group from v58
into v3. If v98 is true,002e..0034 adds it; otherwise002a's +0x5bb jump reaches
0ba0, which removes it. The remove branch jumps back from0ba6 by-0x5b9 to0034,
so both branches call0cv.A01 at0034..003a before the later group processing.
This ordering is not proof that subsequent store work cannot fail or that
preferences roll back together with it.

**Field provenance, not adjacent-field guessing:** 0wE.A0e reads82U.A0Y into
v9 at00fc..0100 and moves it to v58 at0150..0154. The call has instance v16;
A0b has instance v56, so the +42 register slot becomes v98.82U.<init> stores
constructor v44 in A0Y at00ea..00f2. The neighboring A0U candidate maps to the
next slot and is not this set-update flag; its negative field census is retained.

0co.A03 reads missing_participant_identification with A0G. Its four retained
constructor ranges carry that boolean in the corresponding +42 slot:
- call instance v11, source v53, read0348..0350, constructor03e2;
- instance v46, source v88, read0674..067c, constructor0746;
- instance v47, source v89, read08ea..08f2, constructor09d0;
- instance v46, source v88, read0a6c..0a74, constructor0b56.
Thus these converted group-info branches populate A0Y, which controls the
add/remove operation above. The initial true flag also logs at0164..016e;
logging is not an early rejection. A0G's complete coercion and all other
converter fields/branch wire type names are not re-established by this trace.

## Port boundary

**Confirmed local review:** Rust's explicit MEX input context covers the current
vocabulary; its offline builder takes context from the caller. No automatically
persisted missing-identification set is established by that API. Older37.71
notes saying set lifecycle was untraced remain historical; these current
add/persist/remove edges now qualify that gap. No Rust state or request was
changed on the strength of an incompletely bound ingress.

**Unknown:** full notification registration/authorization, all gate/default
values, preference reset/concurrency/transaction semantics, every native group
conversion side effect and complete Rust coordinator integration. This flag is
not a ban status or permission to replay a failed query. No live account
operation, automatic query, identifier invention or identity-policy relaxation.

`checksums.json` records bytes and SHA-256 for every other bundle file.

## Handler construction and numeric dispatch follow-up — 2026-11-05

**Confirmed static:** exact constructor caller searches find 1Uw.A0Q for
0w9 and 1Uw.A0I for19k. These are dependency-injection factories, not wire
notification registrations. Both mask the selector with0x3fff: retained switch
entries select0w9 at2518 ->0bf0 and19k at7001 ->0cba. Factory excerpts retain
the switch/header and selected branch plus complete-source SHA-256; unrelated
factory branches are omitted.

Complete constructors pass singleton integer arrays and true to0v6.<init>:
0w9 uses0xd1 (209),19k uses0xbd (189). The base constructor stores that array
in A0D; Amf returns it. BLn checks the supplied handler integer against A0D
before accepting Message.obj as IncomingStanza, obtains its node and builds
an8bl runnable (selector13). A01 obtains the IncomingStanza node and calls
virtual A09 at0204..020a. Its exact physical callers are8aj.run and8bl.run.
These sources establish numeric filtering and the shared processing boundary;
they do not yet map209/189 to wire type strings or prove transport authorization.
The runnable selector paths and upstream integer producer remain untraced.
Do not equate numeric handler codes with binary-node token IDs.

No Rust behavior changed. Unknown notification admission remains a port blocker
for a generic marker consumer, not a reason to invent a broader policy.

## Wire notification type binding follow-up — 2026-11-05

**Confirmed static:** EL2.<clinit> maps w:gp2 ->209 at000a..0022 and
picture ->189 at0110..0126. Exact map references are its initializer and two
reads in1rN.A05. In that method's1rc branch, IncomingStanza's node type is read
at004e..005c. Following special devices/update and business/bizai routes, the
map lookup019c..01b8 supplies the integer, jumps to0154 and calls2Ai.A00 with
the original IncomingStanza. The resulting Message goes to1rc.A00.accept.
2Ai.A00 passes the integer as Message.arg1; what is zero. These integers are
not binary-node token IDs. The1rc constructor stores its callback; concrete
callback/outer router wiring and transport authentication remain untraced.
This closes the earlier unknown type-to-integer mapping, not all ingress.

Base BLn constructs8bl selector13. Its retained0368..0412 arm stores
child/offline bookkeeping, then calls0v6.A01 only when its A09 executor is
null; otherwise it schedules8aj selector1 on098. Both handler constructors
pass true and initialize that executor. The8aj selector body and scheduling
guarantees remain unclosed; factory references do not prove synchronous ACKs.

**Selected conditions:** w:gp2's0w9 resolves a group sender, checks feature
0x36fe and obtains the first child before calling0zu.A02 on the whole node.
Picture's19k resolves first-child jid, checks group type and has an Eof.A03
refresh-and-return path. It handles set/delete photo changes; set parses its
photo ID with -1 fallback, delete supplies -1. Group/photo-ID checks and
0zP.A02 select normal photo work versus out-of-order logging. Both can reach
0224: normal work runs a loop (03d0 onward), exits via05bc and05f8 ->0224;
out-of-order work falls through021e. After feature0x36fe it re-reads first
child and permits set/delete before the marker call027a. At024a v2 is set
from0158, not create. Any earlier create/delete description of this range is
corrected to set/delete. Runtime gates and complete photo helper semantics
remain unknown. Picture type alone is insufficient marker admission.

**Confirmed Rust source review:** connection.rs and notifications.rs have no
picture-specific consumer/ACK. Existing w:gp2 handling ACKs after capture and
supported changes; neither path automatically maintains this missing-ID set.
This is a concrete parity gap. Native deferral, validation and group-state
ordering must be reconciled before adding a picture ACK or marker consumer.
No runtime behavior or live account operation changed here.

**Bounded negatives:** exact0v6.Amf caller search returned zero, which does not
exclude interface dispatch. Six w:gp2 string references were found; inspected
1ro is a token array, E88 an enum,2ER a type list,T3n a registry constructor,
and8B8.A0I a typed converter. These alone do not bind209/189; positive proof
is the retained complete EL2 map and1rN consumer. Unrelated tables are omitted.

## Picture base ACK integration — 2026-11-05

**Confirmed static:**8aj.run with nonzero selector (selector1 scheduled by8bl)
unpacks the captured handler/IncomingStanza/Eoh/child/code and calls0v6.A01
at0028..002e. This closes that deferred call edge. Complete19k declared method
inventory contains only constructor and A09; no ACK override exists there.
The exact0v6.A07 caller census contains45 physical calls and none in19k.
A07 is the selected base deferral helper: it removes the current atomic stanza
and inserts it into A0A. Exact A0A field references are retained; they identify
base initialization/processing/deferral and89Y/8bZ consumers. Reflection and
untraced indirect mutation are not disproved. Ordinary return from A09 reaches
base A01's atomic claim, A0A membership test and A00 ACK dispatch.

For non-BX2 stanzas, A00 calls0Sf.A0O ->private A07 ->1i8.A03 and queues its
Message through0Sf.A0V.1i8.A03 puts the Eoh into Message code76. BX2 has its
separate1ic/2Dx path; neither is a new Rust CCQ implementation. Final wire
serialization remains the existing shared-base evidence, not newly traced by
these selected methods. Scheduling latency and native database failure/error
ACK behavior are not ported from this trace.

**Implemented locally:** notifications::picture_ack uses the existing base_ack
for exact notification/type=picture. Connection dispatch invokes it after the
existing durable capture. Envelope guards retain the shared nonempty bounded ID,
valid sender and optional valid participant rules. InvalidInput is logged and
leaves receive running, with no ACK. Redelivery is ACKed again; no photo update,
query, marker persistence, identity adoption or outbound message is triggered.
This supersedes the previous no-picture-ACK source-review finding. Photo and
missing-identification state application remain unported; ACK support does not
claim their parity or any live server acceptance.

The actor regression checks exact ACK and participant routing, capture existence
before completing the write, repeated notification, malformed envelope followed
by a ping barrier, no extra output, and durable capture after store reopen.
