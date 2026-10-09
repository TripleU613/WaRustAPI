# Group participant receipt attribution — 2026-10-09

Comparison APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [source bundle](apk/2.26.37.71-group-receipts/manifest.json) preserves
21 methods in ten semantic topics, 15 files / 124,482 bytes, including complete
selected operands and exception tables. The
[recipe](apk/recipes/2.26.37.71-group-receipts.json) selects shorties and classes
for this build; a later APK must be re-anchored before comparing those topics.
Offsets below are method-relative bytes. This extends
[direct attribution](direct-receipts.md); native chat migration gates remain
separate [unfinished work](receipt-canonicalizer.md).

## What the APK keeps separate

**Confirmed static:** `Cmk.A00(IL)` distinguishes ordinary delivery (5), read
(13), played (8), read-self (17), played-self (18), inactive (15) and sender (5).
`Cmk.A02(LL)` retains the remote group/status chat rather than selecting the
wrapper recipient in its special-chat branch. `Cmk.A03(LLLL)` reads the root
message ID at `04aa..04be` and constructs `DDH` at `051a..05bc` with the selected
message keys, remote chat, `DeviceJid.Companion.A00(participant)` and selected
recipient as separate arguments. `0DK.A00(LL)` is retained as the selected
participant-device factory; this does not prove every runtime JID subtype is
supported by our ordinary group handler.

**Confirmed static:** `DDH.<init>(VLLLLLLIJ)` stores keys in `A07`, remote
chat in `A03`, actual device in `A02`, selected user in `A04`, and independent
status/time fields. `BSJ(LI)` reads an indexed message key, `CFC(LI)` returns the
actual device, `CKw(L)` returns remote chat, and `CJI(L)` returns selected user.
The exact `A07` field-reference scan found only constructor write, `BSJ` read,
`CPB` read and `size` read; it did not reveal another direct array consumer.

**Confirmed static:** `DDH.CPB(VLIIJZ)` copies the unprocessed message-key
suffix using `System.arraycopy`, constructs
`ReceiptProcessingJob.<init>(VLLLLLIIJJZ)`, and submits it to the job queue.
`ReceiptProcessingJob.A0G(V)` rebuilds keys from saved chat/from-me/ID arrays
and separately parses remote chat, selected recipient and participant-device
strings before recreating `DDH` without the original wrapper. Its ordinary
branch calls `Cmn.A03(LLIJJ)` with the saved duration and a 10,000-ms processing
budget; its alternate status-infrastructure branch uses a different future.
`Cmn.A03` schedules `DMc`, and `DMc.run(V)` invokes `Cmn.A04(VLIJJ)` before
settling the future. These traces establish durable actor/chat separation;
Rust does not implement that native job scheduler or budget.

**Confirmed static:** `Cmn.A04` obtains actual device (`CFC`), timestamp,
message key (`BSJ`) and remote chat (`CKw`) separately at `02a0..02be`; it later
obtains selected recipient (`CJI`) at `032c`. One group branch rejects a selected
recipient inconsistent with the message sender (`0498..0548`); the subsequent
common branch diagnoses a null remote device (`0548..0560`). Those branches
are not proof that every receipt subtype has identical validation.

**Confirmed static / unknown policy:** when original message lookup fails,
`Cmn.A04` can retry an outgoing message lookup using `0Xw.A0I` and the same ID,
logging `recovered outgoing message by counterpart chat (receipt chat-form
mismatch)` at `0476`. Helper and feature conditions were not fully traced.
Our existing authenticated association is a local authority boundary, not a
claim to reproduce that native gate. The worker also contains orphaned-receipt
storage and later application branches. Their complete row mutation, cache,
privacy, broadcast/status/coexistence handling and deferred replay remain
**unknown / unported**.

**Unknown native batch parity:** `Cmk.A03` reads list/item IDs, but the selected
loop alone did not establish the resulting native array assignments. Rust
retains its existing bounded deduplicated root/list ID handling; it does not
claim complete native batch equivalence.

## Rust behavior and local authority

**Confirmed local implementation:** `receipts::record` dispatches an ordinary
`g.us` receipt to `group_receipts::record`. It requires a primary group address,
a separate PN/LID participant device, and absent or recognized delivery/read/
played type. Companion participant devices are supported; hosted, interop,
nonzero-agent, self-status, recipient-qualified, broadcast and status routes
are excluded. As in the direct handler, missing type defaults to delivery;
present nontext, empty or unknown type does not. Call the library only with an
authenticated transport receipt; parsing an arbitrary node is not authentication.

The saved group, wire ID and original prepared envelope must agree. A participant
must match a device in the original send's frozen target set. PN/LID crossover
uses only the existing authenticated association, in either direction, with the
same device index. Current membership or a directory alias cannot supply that
authority. This frozen-target check is **our conservative implementation guard**,
not a traced mandatory server/APK rule. The local binary child bound limits a
target/fact collection to 4,096; this is not a claimed WhatsApp group limit.

Comparison sends now save `recipient_devices` alongside the original encrypted
wire in the pre-write reservation. This retains the actual addressed devices
when sender-key distributions went out of band. Historical in-band sends can
derive targets from their original `participants/to@jid` list. Historical
out-of-band sends lacking that snapshot have **unknown** addressed devices and
are not attributed using current membership. A new snapshot must agree with an
embedded target list when both exist. An explicitly empty frozen set reports
known zero, rather than missing historical provenance. No historical reservation
is reset.

Comparison records keep independent facts in `device_receipts`, keyed by the
actual PN/LID device, without changing the saved send outcome. A read does not
invent delivery; played does not invent read. Counts do not establish that every
member received a message. Mapped namespaces retain their observed actors and
can produce separate device rows; this is not a count of unique human members.

For normal outbox records, the unique saved wire intent and original first
prepared envelope supply the association. A new per-device fact and its
`SendStatusChanged` journal event commit together. Original intent, attempt,
progress, wire ID and `DeliveryUnknown` stay intact; a receipt does not synthesize
`ServerAccepted`. **Normal group send preparation remains unimplemented**;
this branch is tested with isolated prepared-envelope fixtures, not presented
as a newly working normal group sender.

The entire root/list batch, checkpoint updates and normal outbox events use one
write transaction. Duplicate actor/kind facts return locally without another
event. Ambiguous duplicate wire IDs, corrupt matching facts or inconsistent
saved wire/targets produce fixed redacted diagnostics and roll back prior
updates in the same batch. Checkpoints page in groups of 256 rather than a
10,000-row truncation. Unsupported or unknown targets remain preserved captures;
there is no orphaned-receipt replay implementation. Receipt-send ACK reservations,
peer keys, ratchets and account identities are separate and untouched.

The comparison group-send post-send window, normal `listen` and comparison
receive share this handler. Recording a fact itself sends neither a chat nor a
receipt ACK; existing authenticated receive confirmation logic is separate.

## Offline inspection

```sh
whatsapp-cli --profile NAME group receipts --request-id EXISTING_REQUEST_ID
```

Use the existing private message JSON's `request_id`. The command reads one
comparison reservation and prints only `send_status`, nullable `addressed_devices`,
`receipt_devices`, `delivered_devices`, `read_devices` and `played_devices`.
A missing/non-group reservation returns JSON `null`, not a server refusal.
No addresses, wire IDs, contents or ciphertext are printed. Inspection does not
queue work, connect, resend or change reservations. Normal outbox facts remain
inspectable through the existing `status --local-id` command. The API description
is a CLI/library contract; `/v1/groups/receipts` is **not served over HTTP**.

## Validation and remaining live boundary

**Confirmed local:** eight group-handler regressions cover actual wire decoding,
primary and companion PN/LID attribution, authenticated crossover in both
directions, wrong-device/alias-only refusals, independent facts, original-target
agreement, old out-of-band unknowns, reopen deduplication, more than 10,000
checkpoint rows, grouped IDs and transaction rollback after ambiguous or corrupt
late matches. A CLI subprocess test verifies real command parsing, offline
summary/null behavior, redaction and unchanged encrypted state/journal.

Fresh live participant receipt attribution, group companion sending, native
receipt cleanup/aggregation and historical orphaned application remain
**unverified**. Saved account and receive-only regression results are recorded
in the messaging checklist after verification; no new group send is authorized
by inspection or by this implementation.

**Observed offline:** twelve distinct existing private group request files were
inspected without network requests. Seven had saved group reservations, five
had none, and the seven had known original target sets and zero tracked receipt
devices. Offline projection upgraded zero events. Saved peer keys, action
outcomes, receipt ACKs, outbox, events and normalized retained inspection were
unchanged. Inspection was serialized after projection: overlapping processes
opening the same encrypted database can fail locally and are not server refusals.
No existing capture was replayed as a receipt or counted as a new message.

All 749 workspace/all-feature tests passed, together with strict Clippy,
warning-free Rustdoc, default CLI tests, CLI/API symbol and reference checks,
formatting and repository hygiene. The APK/recipe re-export matched all 15 files;
cached self-diff took 0.019472 seconds. This is cached same-build reproducibility,
not evidence that a new release is unchanged or that obfuscated selectors will
survive it.

**Observed live regression:** a 45-second receive-only session authenticated,
activated and stopped cleanly with chat sending and reconnects disabled.
Peer keys, action outcomes, receipt ACKs, empty outbox, events and retained
noncapture inspection stayed unchanged. Transport capture rows increased from
442 to 446; these are not four user messages. All seven saved group summaries
stayed unchanged with zero tracked participant receipt devices. No fresh group
receipt established live attribution, and no uncertain send was replayed.

2026-10-09 — **Confirmed local/static:** normal-outbox receipt attribution now
prefers [exact original targets](send-version-snapshot.md#exact-original-device-provenance-subsequent-port)
saved with the original reservation. Envelope sender-key recipients must be a
subset. Historical count-only/missing snapshots retain wire-only attribution.
The current Rust builder distributes SKDMs to every plan device; omitted SKDM
recipients are covered by native-shaped fixtures, not a claimed current live bug.
Full native receipt cache/cleanup remains unported.
