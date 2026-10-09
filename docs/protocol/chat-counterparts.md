# Chat-store counterpart mapping producers

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Twelve-topic recipe](apk/recipes/2.26.37.71-chat-counterparts.json),
[complete methods](apk/2.26.37.71-chat-counterparts/targets.json),
[physical references and bulk-update callers](chat-counterparts-evidence.txt).
Offsets are method-relative bytes unless the inventory says DEX byte offsets.
This extends [receipt counterpart selection](receipt-counterparts.md).

## Concrete store and SQL hydration

**Confirmed:** 0VS's constructor assigns a new HashBiMap to A08 at
00fe..010a. The exact field scan finds ten physical references, including that
one assignment. Collection mutations are reads of the field, so a writes-only
scan would miss their effect. This inventory is not proof of all possible
mutations through copied references or reflection.

The already retained [0Fs.A02 loader](apk/2.26.37.71-receipt-counterparts/chat-cache-loader.txt)
has two corresponding hydration sites (0634..068e and 1226..1280).
For the first site, v12 comes from 0ZR.A03, v57 indexes `jid_row_id`, and
v68 indexes `original_jid_row_id`. On the admitted row it reads jid_row_id
into v95, then, only with the v12 gate and 0DE.A0Z(chat), reads the original
row. A distinct original row greater than zero is inserted under the chat's
JID row in A08, inside the monitor. The second site repeats this relationship.
The loader first parses raw chat JID through 0D7.A02 and skips parse failures,
0DE.A0i chats, hidden==1 rows and 0DE.A0b chats. Those helper subtype domains
and the full remainder of loader processing are not established here.

0DE.A0Z accepts type 18, 19 or 25. The retained 0ZY class establishes ordinary
`lid` server/type 18; it extends 0ZX. Its constructor requires 1..15 ASCII
digits with no leading zero, with explicit invalid-LID exceptions. The scanner
found no method bodies for 0ZX, which does not prove absence of that abstract
class or establish all its implementations. Types 19/25 remain untraced here.

**Confirmed query construction:** 0Fs.A01 builds the SELECT against chat_view,
left joins jid on chat_view.jid_row_id=jid._id, and left joins deleted_chat_job.
The first Boolean adds original_jid_row_id; the second adds descending
sort_timestamp order. The SQL filters hidden=0. Selected chat_view columns
come from 0ZT.A00 through 0ZU.A01. Complete view definition, column-list
construction and 0ZR's effective migration state are **unknown**. This trace
identifies stored chat provenance; it does not prove server-authenticated alias
admission or grant Rust permission to create a mapping from arbitrary input.

## Bulk population from conversation records

**Confirmed:** 0pq.A0D consumes BZA.conversations_. For each nonnull BZK,
it parses nonempty accountLid_ through 0D1.A00 and requires an instance of
0ZX; parse exceptions become a failure wrapper and leave the selected value
null. It parses id_ through PhoneUserJid.Companion.A04. Only when both values
are nonnull does it obtain both JID rows through 0VT.A07 and stage
`account-LID-row -> phone-row` in a temporary HashMap (00a4..00ce).
A nonempty temporary map is passed to A08.putAll inside a monitor
(00dc..00ee). The method does not itself verify a signature, pin an identity,
check either row against -1, or persist receipt_user rows.

The physical caller inventory contains four invokes in three methods:
CnI.A08 twice, CoO.A02 once and CpC.A05 once. The complete CpC.A05 is
retained; its relevant branch logs IncrementalBackup/MessagesImporter,
opens a file input stream, parses delimited BZA records, and calls A0D
at 246a..2470 before iterating conversations. This establishes a local backup
restore source, not a direct authenticated wire-notification parser. Upstream
backup verification, the other two caller methods and full restore transaction
semantics remain **unknown**. Duplicate LID keys overwrite in the temporary
HashMap; downstream HashBiMap duplicate-value/error behavior is not analyzed.

## Consumers, deletion and cache invalidation

**Confirmed:** 0VS.A0J initializes the chat cache, then reads A08.inverse()
inside the monitor. That reverse lookup returns a nullable Long. In contrast,
0VS.A04 mutates A0F/A0G JID lookup caches, not A08. Its name appearing in the
loader is not a counterpart-map insertion source; this negative is retained.

0VS.A0W deletes chat rows by account_jid_row_id when 0ZR.A03 is true,
otherwise by jid_row_id. After chat-cache removal it removes A0F/A0G entries
and removes A08's forward key obtained through A07 (0098..00d2), then calls
the transaction's A00. This is chat deletion, not receipt-user cleanup or a
refusal reset. 13u.A00 clears A08 alongside the JID, message and receipt caches
(0092..009c for A08); its caller/trigger authority remains untraced. These
operations are evidence only and were not invoked on the saved account.

0W8.A0c reads the forward mapping and resolves its UserJid (035e..039c)
in a notification-channel/settings path. It does not insert into A08 at that
reference. The full method is retained; complete settings behavior is untraced.

SendLidMigrationMappingSyncJob.A0G initializes the chat cache, copies A08 to
a HashMap under the monitor, and has diagnostics for no paired devices and
empty mapping data. Its later mapping conversion resolves keys through
0VT.A0E(0ZY.class, keys) and values through A0E(PhoneUserJid.class, values),
compares the LID against 0Xw.A0Q's phone mapping, and builds CWA records
(03e4..04e0). It later stores an encodedMappingPayload_ in BPb and stages a
peer message. Full serialization, limits, job retry/receiver admission and
linked-device authentication remain **unknown**. No sync job has been queued.

## Implementation boundary and reproduction

**Inferred:** this receipt path's namespace expansion depends on stored chat
migration provenance that can be populated by SQL hydration and restored
conversation records. It is not equivalent to equality of raw PN/LID strings
or a contact-directory match. Native primary cleanup still requires actual
receipt_user membership, whose complete admission/creation is unfinished.
No Rust alias adoption, account identity, reservation or live request changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-chat-counterparts.json \
  --output docs/protocol/apk/2.26.37.71-chat-counterparts --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-chat-counterparts \
  docs/protocol/apk/2.26.37.71-chat-counterparts
```

Static artifact agreement does not establish live migration or sync parity.

## Import caller follow-up (2026-10-09)

CnI.A08 and CoO.A02 now have retained source and immediate input-path traces in
[mapping import entry points](mapping-admission.md). The history update is in
the initial-bootstrap branch; transfer parsing and bootstrap nonce presence
are not full upstream authentication proofs. The earlier snapshots are unchanged.
