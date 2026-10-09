# Ordinary cursor status loading and a media preparation source

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Five-topic recipe](apk/recipes/2.26.37.71-message-loader.json),
[full methods, signatures and catches](apk/2.26.37.71-message-loader/targets.json),
[exact searches and physical-call inventory](message-loader-evidence.txt).
Offsets are method-relative bytes. No account operation or Rust runtime change.

## Cursor status reaches the message setter

**Confirmed:** 0VA.A00(Cursor,1gc,0VA):178 reads `message_type`, `timestamp`
and `status` through getColumnIndexOrThrow, followed by getInt/getLong.
The first status read is at 0066..007c and contributes to type/status factory
selection. Types 80, 86 and 83, and the type-0/status-6 case, have separate
branches; complete subtype interpretation remains **unknown**.

The ordinary path calls 1ge.A00(key,type,timestamp) at 00ba..00c0. A nonnull
message branches from 00c2 to 01da. The failed/null construction path logs,
reports through 0AH and returns null at 0168..016a. Its diagnostic reuse of v5
as an integer therefore does not reach the later successful load path, where
v5 remains the literal `status` column name.

At **020c..0222** the successful path again gets that column index, reads its
integer and passes it to **178.A0H**. There is no FStatus ordinal conversion,
receipt-time-derived reconstruction or numeric status clamp in this sequence.
Virtual setter overrides still apply: the separately retained Bn0/Bn1 override
forces 6. A cursor integer is therefore not a universal assertion about the
resulting object's subtype-specific status. See [status setters](message-status.md)
and the distinct [FStatus conversion](status-enum.md).

The same loader reads `recipient_count` into 178.A06 at 0246..025e, and
`receipt_server_timestamp` into 178.A0E at 02c4..02d8. The latter is getLong
followed by direct assignment, without multiplying, minimum selection or
substitution of local observation time. This complements the
[native accepted-message UPDATE](message-ack.md), which writes supplied server
receipt time even when another branch preserves an older in-memory A0E.
No universal timestamp minimum is implied by the worker or loader.

## Entry points and subtype construction

**Confirmed:** the physical call scan finds two 0VA.A00 callers: 0V9.A05 and
0VA.A05. Both take cursors. The scan does not establish every reflection or
virtual caller. 0VA.A05(Cursor,chat,bool,bool) returns null for an empty cursor;
a one-column cursor instead loads its row ID through A04. On a full row it
obtains the message key through 0zc.A06, logs and returns null when that fails,
then conditionally tries A01 before calling A00 at 0086..008c.
An A01 hit returns immediately. Full cache authority, A04 row query, outer SQL
projection and all following decode/post-load hooks remain **untraced** here.
This proves a concrete successful cursor loader, not universal refresh of
already-cached objects or every database retrieval route.

1ge.A00 gets a registered type implementation through 1gj.A00(type), casts it
to 1gh and calls AIQ(key,timestamp). Its MTL catch covers 001c..0026,
not the earlier registry lookup. The handler distinguishes reported
registration/support cases and emits the retained factory diagnostics. Its
fallback block calls 00K.A0D(false,diagnostic) before constructing 1rW; whether
that assertion returns is **untraced**, so fallback success is not guaranteed.
Other exception classes are not swallowed by that catch. Full type registry
and fallback behavior remain **unknown**; a
nonstandard factory result must not be relabelled an ordinary supported type.

## One concrete outgoing initial-state producer

**Confirmed:** 0qA.A03 requests a key through 0od.A03(chat,true),
constructs a message through 1ge.A00 and requires a **1iB** result. An unsupported
result throws IllegalArgumentException with the retained requirement diagnostic.
Its successful branch calls CTc(media), sets 178.A01=0 and invokes
**178.A0H(1)** at 0148..014e. Register v9 is set to 1 at 0010 and not reassigned
before this invocation. Subsequent media/body/origin initialization is retained.

This is evidence for the shown media preparation source, not every outgoing
text, group, broadcast, newsletter, edit, restored or mirrored message. Complete
0qA callers, key-builder authority and subtype setter overrides remain **unknown**. Neither this
status-1 call nor the FStatus constructor's PREPARING enum proves a universal
outgoing initial state. It is not permission to set uncertain Rust reservations
back to a prepared state or to resend a message.

## Negative findings, Rust boundary and next trace

The case-sensitive `getMessageFromCursor` string search had zero matches;
`FMessageFactory` found nine strings. The existing
[physical setter inventory](message-status-evidence.txt), rather than a guessed
log spelling, led to the actual cursor signature. Those searches are scoped
negative results, not absence of SQL status loading.

**Unknown/unported:** native message row import, complete status initialization,
outer SQL/cache authority, ordinary receipt_user creation and primary cleanup's
user-row intersection. Neither a loaded status nor receipt_server_timestamp
creates a per-user receipt row. Rust's immutable send targets, mutable device
membership and DeliveryFacts remain distinct; no historical row is synthesized
from those records. The next required authority is actual live user-row creation
and its native status/ownership admission, before porting primary cleanup.

Existing apk-smali, apk-callers, apk-research-bundle and apk-research-diff expose
and reproduce the result. Research is CLI-only with x-live=false; no new HTTP
route or live validation is claimed. Two retained diagnostic-anchor sets aid
new-build relocation, while unanchored targets require reviewed selectors.
Structural changes are not automatic proof of protocol behavior changes.

Validation: five topics / 10 files / 93,812 bytes re-export exactly from the
pinned APK; cached self-diff reports unchanged artifacts. CLI reference
regeneration, API-symbol checks, formatting and diff hygiene pass. The prior
811-test runtime result applies to unchanged Rust; this evidence chunk has no
new runtime or live test claim.
