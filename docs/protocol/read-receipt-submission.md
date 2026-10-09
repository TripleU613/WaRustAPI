# Saved read-receipt submission

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[13-topic recipe](apk/recipes/2.26.37.71-read-receipt-submission.json),
[complete selected signatures, operands and exception tables](apk/2.26.37.71-read-receipt-submission/targets.json).
Offsets are method-relative bytes. This follows the
[saved candidate scan and observer binding](read-setting-observers.md).
Queueing is distinct from the [job's later admission and wire construction](read-receipt-job.md).
No Rust behavior or live account state changes in this trace.

## Grouped work precedes individual admission

**Confirmed:** 0zH.A0V first calls A0I(collection), then A0W(groups)
(0008..0016). Its later ACB.A06/supplied-Boolean branch and per-message loop
therefore cannot retroactively suppress those queued groups. DOh selector 7,
ACB, Ajg selector 94 and their complete effects remain **unknown**. The auxiliary
branch collects A0D values for type 11 with positive A0D, and calls ACB.A03/A04;
these values are not fabricated current times. This is not a universal privacy gate.

A0I skips type 19/21 with diagnostic labels “hsm rejection message” and
“payment request declined message”, requires A0D(message), then skips A0B.
A0D rejects false message.A0l(), status 16/6, true 1gd.A1A, type 19/21/128,
or true CmC.A08. It also rejects false A0Z and false A0E: the branch at 005e with
code-unit delta +9 targets byte 0070 (const false), not the true return.
Only true A0Z proceeds through A0E and its true return at 0072.
The selected body does not exclude status 17. Full virtual predicates and
message-type domains remain **unknown**.

A0B rejects a nonnull chat equal to B1Q when chat fails 0DE.A0h, logging
`dropping_malformed_outgoing_read_receipt`. Equality is not evidence that B1Q
may be synthesized from the chat. A0C selects Ci4(chat,B1Q,true) for 1iZ,
a B1Q passing A0h, or a chat passing A0m with 23u.A00(derived B1Q UserJid)
true. Otherwise CN8.A00 selects Ci4(chat,B1Q,bool), with bool true for 1iZ
or chat passing 0DE.A0b. Complete 23u/classifier and participant authority
remain **unknown**. The Boolean is the job's shouldForceReadSelfReceipt;
true selects read-self through 0zd.A05, not permission to send peer read.

A0I appends original message-key IDs without sorting/deduplication in this
body. Chg initializes its time to -1; positive A00(message) values are combined
by Math.max. Chg.toString names it `maxServerStoreTimeMicros`. A00 returns
message.A0D only when positive and chat fails 0DE.A0i, otherwise -1.
This maximum is distinct from earliest-positive received-device timestamps;
A0D's complete producer authority remains **unknown**. HashMap group order
is not established as deterministic.

A0W partitions each group's supplied ID list into batches of **256**.
SendReadReceiptJob receives destination, Ci4 participant, null recipient,
null remote sender, null receiptClass, the batch IDs, original timestamp **-1**,
loggable stanza ID **0**, group maximum server-store micros, and Ci4's Boolean
(0172..01a2). It calls 0ls.A01 for each job. No server ACK is awaited here.

Destination selection depends on 0hd predicates, 1Z3.A02 and Boolean property
**27084**. For the A09(user)/A0d(chat) case it records a conversion flag and
uses the inverse A02 result. With A07(chat) and property 27084 true, that inverse
selects derived UserJid -> 0DE.A08 with nonnull checks; otherwise it preserves
chat. Outside that property branch, the conversion flag selects 0hd.A01 and a
required ChatJid cast. Effective properties, migration state and mapping
association remain **unknown**; arbitrary alias substitution is not justified.

## Later per-message policy and local marking

**Confirmed:** the loop first requires **0zd.A09(message)** (0246..0252).
A09 requires A0F >= **1,415,214,000,000**, true message.A0l(), and true
0zd.A07(chat). Its inclusive cutoff differs from A0X's strict greater-than
candidate cutoff. A07's preference/contact/account conditions are retained in
[read policy](receipt-read-policy.md); effective account inputs remain unknown.
This gate is downstream of grouped queueing, not the entire method's gate.

After admission, status **17** calls A0J then continues, without constructing
another individual job in that branch. Other statuses pass A0Z and status-16
checks. Selected B1V/chat 0hd.A02 predicates then lead to additional individual
jobs. The A0m(chat) path supplies chat and B1Q participant; the A0l(chat) path
requires B1Q and supplies it as destination with original chat as recipient.
Both use the original single key ID, A0F, A0m, A00(message), null remote sender
and receiptClass, and force **true** (02da..0320 and 03c6..041a). They call
0ls.A01, then A0J. This source does not establish that every candidate reaches
one of these paths, or authorize replaying IDs.

A0Z returns false early for flag **0x1000000000** or chat passing 0hd.A06.
Its remaining branches depend on 1Z3.A01, B1V/chat 0hd.A05/A02 and chat
A0l/A0m. These predicates and effective migration state remain **unknown**;
the retained branch operands are the reproduction surface, not a guessed
PN/LID policy. A0E returns inverse 0yy.A0B for the 7xQ.A02 plus PermanentGroupJid
case. Otherwise its status-chat branch requires nonnull B1Q, property **18389**
and B1Q passing A0l, then maps through 0Yo.A0C(user,null) and calls 0zd.A07.
Other branches return true. This nested policy is not a global Boolean check.

A0J returns immediately for 1iZ or true message.A0V(). Otherwise it calls
A0H(**16**), then schedules DNe(message,manager,**2**) via 07u.COM.
The selected code does not await server acceptance before this local mark.
DNe persistence/commit, the virtual predicate and crash atomicity remain
**unknown**. Status 16 here cannot be counted as handset read confirmation.

A0V also collects A0k maxima: type-5 chat with qualifying B1Q contributes to
one map; other chats retain a greatest-A0k message (equal replaces). It schedules
3RC selector **12** with these maps through 1f0.A01 code **43** (0454..0480).
Worker effects are **unknown**. The selected method has no catch ranges;
that does not establish all-or-nothing queue/persistence behavior.

## Reproduce and port boundary

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-read-receipt-submission.json \
  --output docs/protocol/apk/2.26.37.71-read-receipt-submission --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-read-receipt-submission \
  docs/protocol/apk/2.26.37.71-read-receipt-submission
```

**Inferred port consequence:** preserve the separate grouped and individual
admission stages, force-read-self bit, original IDs, maximum store-time and
local mark boundary. A single preference gate around A0V would erase confirmed
branch distinctions. Complete workers, effective settings, receipt_user
initialization and primary identity cleanup remain unported. No HTTP route,
new live capability, human-read event or automatic receipt replay is introduced.

Verification: exact regeneration matched **18 files / 84,398 bytes**; cached
self-diff reported 13 unchanged topics and four unchanged metadata artifacts.
This is same-APK structural reproducibility, not cross-release semantic parity.
CLI-reference regeneration/coverage, OpenAPI library-symbol coverage, formatting,
whitespace and repository hygiene passed (1,887 local links; 82 checksum bundles,
17,652,166 bytes). Workspace runtime tests were not repeated for this source-only
chunk; no live operation or handset-read confirmation is claimed.
