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
