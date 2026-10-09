# FStatus enum conversion is separate from message SQL decoding

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Ten-topic recipe](apk/recipes/2.26.37.71-status-enum.json),
[complete signatures, branches and catches](apk/2.26.37.71-status-enum/targets.json),
[physical decoder callers and enum-field writers](status-enum-evidence.txt).
Offsets below are method-relative bytes.

## Named enum values and conversion

**Confirmed:** 1cE.<clinit> constructs eight enum entries, with matching ordinal
and constructor value 0..7. Its constructor stores the third argument in `value`;
7z7.A00 dispatches on **ordinal**, not that field. 8BW.<init> explicitly puts
PREPARING in A06 at 003c..0044. This is a separate enum field from
[178.A08 message status](message-status.md).

| Ordinal | 1cE name | 7z7.A00 result |
| --- | --- | --- |
| 0 | PREPARING | Context-dependent 0, 1 or 2; see below |
| 1 | SENDING | Same branch as PREPARING |
| 2 | FAILED_TO_SEND | 20 |
| 3 | SENT | 4 |
| 4 | RECEIVED | 0 |
| 5 | VIEWING | 16 |
| 6 | VIEWED | 16 |
| 7 | PERMANENTLY_FAILED_TO_SEND | 21 |

The default switch branch returns **-1**. A null enum also reaches -1. For 8BW,
A00 reads A06; for non-8BW 74Z it reads inherited 1vx.A03; other types feed null
into the same branch. These are not unconditional numeric message-rank mappings.

For ordinals 0/1, A00 first checks 1iA and its Ao5 media state. A null media
state returns 0. A17=true returns 1. Otherwise A0q/A0r=true or A0J>=100 reaches
00f2: A0r=true returns 2, otherwise 0. With all those predicates false it
returns 1. The units and producer meaning of A0J/A17/A0q/A0r are **unknown**;
100 must not be described as an established percentage.

The non-1iA branch checks 743 and its 7pT/7pS-associated asset state. Non-743
returns 0. For either retained asset, a nonnull object with A0A=false or
A08=null returns 1; otherwise it returns 0. The complete asset-producer and
subtype meanings remain **unknown**. The full method retains these branches so
future work need not reconstruct them from an oversimplified status table.

## Consumer and storage authority

**Confirmed:** the physical invocation scan finds two calls to 7z7.A00:
7z7.A06 and 7zY.A01. A06's previously retained 02b0..02bc sequence converts
8BW and writes the returned integer directly to 178.A08. This proves a
conversion assignment, not a normal database row read.

7zY.A01 first checks 1iB and directly returns 178.B3b(). Otherwise it calls
7z7.A00 only for 8BW or 74Z; unsupported media types throw IllegalStateException
with `Unknown FMedia type`. It does not reinterpret a native 178 integer through
the enum. This is a second concrete reason not to treat enum conversion as a
universal message SQL decoder.

8BW.A0S diagnostics explicitly call this **FStatus** state change; 6ca.A05
contains `UserActionsMessageForwarding/userActionForwardMessage/failed to map
chat FMessage to FStatus`. **Inferred:** these retained enum/container paths are
FStatus/media conversion machinery. Complete domain, callers, serialized/schema
sources and ordinary stored-message decoder remain **unknown**. No new SQL
status loader, receipt_user seeding or universal ordinary send state is proved.
The earlier [message status](message-status.md) restoration entry is narrowed
by this new evidence; its direct assignment finding remains valid.

## Enum transition admission differs from 178's setter

**Confirmed:** 8BW.A0S accepts the first requested enum when A0P=false, stores
it and sets A0P=true. Otherwise, for 6cy.A07 it rejects VIEWING/VIEWED before
calling 1u0.A01. The meaning and active producers of 6cy.A07 remain unknown.
If the policy returns true it assigns the new enum; otherwise it logs and
reports `FStatus invalid state change`, returning false without assigning.
This differs from 178.A0H's downgrade logging followed by unconditional storage.

1u0.A01's complete allowed-target sets, derived from its switch branches to
00f6 (true) versus 00f4 (false), are:

| Current enum | Allowed target enums |
| --- | --- |
| PREPARING | PREPARING, SENDING, FAILED_TO_SEND, SENT, RECEIVED, PERMANENTLY_FAILED_TO_SEND |
| SENDING | SENDING, FAILED_TO_SEND, SENT, PERMANENTLY_FAILED_TO_SEND |
| FAILED_TO_SEND | PREPARING, SENDING, FAILED_TO_SEND, SENT, PERMANENTLY_FAILED_TO_SEND |
| SENT | SENT, VIEWING, VIEWED |
| RECEIVED | RECEIVED, VIEWING, VIEWED |
| VIEWING | VIEWING, VIEWED |
| VIEWED | VIEWED |
| PERMANENTLY_FAILED_TO_SEND | PERMANENTLY_FAILED_TO_SEND |

Unsupported ordinals throw OM8, and current is checked nonnull. A0S's first
assignment and special-container rejection occur outside this matrix. Do not
substitute this policy for ordinary outbox reservation or receipt admission.

## Direct enum writes and scoped limits

**Confirmed:** the all-DEX writes-only field scan finds **six writes in five
methods** to 8BW.A06: constructor, A0S twice, 6ca.A05, 8Si.run and 8U9.run.
The constructor writes PREPARING; 6ca's 0c30..0c38 write also uses PREPARING.
8Si discriminator **0** resets PREPARING at 03c4..03cc and sets A0P=true;
8U9 discriminator **2** resets it at 0236..023e and sets A0P=true. The workers'
other UI/media cases and transitive scheduling are retained but **untraced**.
A direct enum write bypasses A0S policy, just as direct A08 writes bypass the
message setter; no universal transition guarantee follows from one helper.

**Negative result:** the investigated 7z7.A00 path is not demonstrated to decode
SQL message.status. The constructor's PREPARING enum does not by itself prove
that every outgoing 178 starts with 0 or 1; context conversion is explicit above.
No row is seeded, account touched or send/receipt performed by this research.
Rust runtime remains at the preceding ACK guard, with existing inspection and
APK research CLI commands. The research API is CLI-only, x-live=false.

Validation: the ten-topic snapshot (15 files / 167,968 bytes) re-exports
exactly from APK 37.71 and cached self-diff reports unchanged artifacts. All
eight documented transition rows match the literal DEX true/false branch
destinations. API-symbol checks, CLI reference regeneration and formatting
pass. The preceding ACK runtime remains covered by its 811 passing workspace
tests; this follow-up changes research/documentation only. No live request ran.
