# Outgoing read-receipt job and wire construction

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eleven-topic recipe](apk/recipes/2.26.37.71-read-receipt-job.json),
[full selected methods](apk/2.26.37.71-read-receipt-job/targets.json).
Offsets are method-relative bytes. Whole job/dispatch methods are retained;
only the selected read path below is interpreted. Other writer cases are not
claimed as complete protocol implementations.

## Locate the job and follow the handoff

**Confirmed / negative:** no method bodies matched the old guessed descriptor
com/whatsapp/jobqueue/job/SendReadReceiptJob. An exact-string xref of
SendReadReceiptJob returned no references; that is not proof the job is absent.
The apk-smali class substring SendRead located the actual class:
com/whatsapp/messaging/receipts/jobqueue/job/SendReadReceiptJob.
Keep this build's package separate from earlier attribution.

Its constructor retains chat, participant, recipient, remoteSender, receiptClass,
messageIds, originalMessageTimestamp, loggableStanzaId, server-store micros and
shouldForceReadSelfReceipt. readObject validates parsed JID/nonempty message IDs;
full constructor caller authority and serialized-job scheduling remain **unknown**.

A0G parses saved addresses, calls 0zd.A08 to admit the job (0098..00a4), then
0zd.A04 to create CM8 (00c4..00cc). It builds an ACK selector through CmC.A06
and CSu using receipt, selected type, first message ID, participant and recipient,
and passes Android Message.what=89 with the CM8 to 0h8.A0B before waiting on
AzY.get. Job completion is not itself proof of a handset read indicator.
Special type-7/status-class branches alter parsed participant; full type domains
and automatic job retry semantics remain **unknown** here.

## Sending policy differs from the factory policy

**Confirmed:** 0zd.A05(chat,force) starts with read-self. Force true preserves
that type. Otherwise it returns read exactly when:

(group predicate A0m OR read_receipts_enabled) AND
(type-21 A0b OR type-7 A0h OR (private A00 AND private A01)).

Other branches return read-self. This follows the branch targets in A05,
not a name-based reuse of A07. The incoming factory's A07 lets type 21/7
bypass the global preference; this outgoing A05 does not. Its private predicates
and preference getter reuse [read policy](receipt-read-policy.md) and
[classifier evidence](receipt-read-classifiers.md). Effective account state and
preference synchronization remain **unknown**.

A06 admits when A07 is true or 0bN.A02 returns a nonempty ImmutableSet.
A08 first requires A06, then rejects an ID when StringUtils.A0J returns true,
logging invalid message ids. Exact string-validation grammar is untraced.
For status-chat A0i, a positive original timestamp is rejected only when
original+86,400,000 is less than current 08B time; equal is accepted by this
comparison, and nonpositive original time skips it. Long addition uses native
wrapping arithmetic. That is a local status-age guard, not a server wait.
Finally read-self requires the same 0bN.A02 set nonempty. The
[saved-set trace](read-linked-set.md) now closes its immediate loader, logout
filter and own-namespace conversion. Full ingestion/deletion authority,
linked-device verification and read-self routing remain **unknown**.

## Handler, route, attributes and list

**Confirmed:** 0zd.A04 creates a false-fromMe 1gc using the first ID, chooses
type through A05 and populates CM8. For more than one ID, it copies the
remaining IDs, in supplied order, to CM8.A03; one ID leaves this field null.
CM8 constructor stores participant A04, remote sender A05, recipient A06,
key A07 and type A08, and initializes server-store micros A01=-1.
A04 supplies receiptClass A02 and overwrites both timestamp/logging fields.
Its status-participant flip and PN-report diagnostic branches are retained;
full producer correctness for those specialized paths remains **unknown**.

Cp2.A07 extracts the key chat/id, type, participant, remote sender, recipient,
extra IDs and server-store micros. A nonnull receiptClass becomes class;
mode argument is 0. It calls Cp2.A00, the actual outgoing receipt builder.
That builder obtains CmC.A06's route pair and emits:

- to and id; nonnull type.
- participant when nonnull and different from to; nonnull recipient.
- sts only when server-store micros is positive; optional supplied class.
- Extra IDs via Cp2.A0A: a list child containing item children with id
  attributes in array order. Null extras omit list; a nonnull empty array
  would create an empty list, though the traced A04 doesn't supply that case.

**Confirmed:** CmC.A06 preserves the group chat as to and can use remote
sender as participant; ordinary routes can instead use remote sender as to.
Its A0Q branches can flip addresses. Complete A0Q domains and remote-sender
producer trust remain **unknown**, so this is not a universal route equivalence.
The existing Rust authenticated direct/group receipt path implements a bounded
single-ID format; it does not implement every native route or this bulk builder.

0y0.A00's selected CM8 case at 1f38..200e calls Cp2.A07, diagnoses an
individual receipt carrying participant and passes the unchanged node to
1Rd.A04 with its loggable ID. The diagnostic does not strip participant.
The full dispatcher and its switch payload are retained; transitive framing,
compression/ACK timing and all other writer cases remain untraced here.

The exact field scan for CM8.A03:[Ljava/lang/String; found three physical
instructions: the 0zd.A04 writer and readers in Cp2.A07/0y0.A00. All three
methods are retained here. This scan does not prove transitive producer authority.
Reproduce with apk-field-refs APK --class LX/CM8; --field A03 --type
'[Ljava/lang/String;'.

## Rust scope and remaining work

**Confirmed improvement in evidence:** the read job uses the same Cp2 receipt
builder as other client receipts, and native extra-ID list construction is now
traced. Earlier vocabulary-only inference is superseded for that wire format.
Rust still queues one authenticated ID per receipt with an explicit read action;
it does not apply the full APK preference/contact policy, generate read-self,
batch lists, copy unknown server-store time or implement full job scheduling.
Existing acknowledged receipt IDs/reservations remain unchanged.
No runtime behavior, account state or live request changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-read-receipt-job.json \
  --output docs/protocol/apk/2.26.37.71-read-receipt-job --check
```
