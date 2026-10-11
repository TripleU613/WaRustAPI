2026-10-11 follow-up: [primary user ACK emission](receipt-primary-ack.md#all-supported-notification-ack-callers--2026-10-11)
now uses JID_PAIR for decoded zero-device PN/LID users. Preserving the descriptor
means preserving its native user class, not blindly retaining AD wire encoding.
Concrete nonzero device routes remain unchanged; earlier validation is historical.

# Identity notification acknowledgement — 2026-10-09

Evidence is **confirmed static** in WhatsApp **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [hash-pinned recipe](apk/recipes/2.26.37.71-identity-notification-ack.json)
and [bounded source bundle](apk/2.26.37.71-identity-notification-ack/targets.json)
retain signatures, register flow and exception tables. Earlier statements that
identity ACK must wait for bundle adoption are **refuted** by this trace.

## Dispatch and deferral

`14z.A09` chooses the first child. Count (005c–0062) and PQ-count
(00a4–00aa) call inherited `0yO.A07(true)`. Identity (00d0–01cc)
parses root from, passes optional metadata, schedules `DLp` discriminator 5
through `16Z.A00` (01b8–01ca), and returns without A07. Scheduling is not
worker completion or successful key adoption.

`0yO.A01` invokes A09 (0206–020c), clears the current descriptor and checks
the pending map (0296–02b8). With no deferred entry it calls A00
(02bc–02c6). `A07(true)` instead consumes the current descriptor, inserts it
into A0A (0040–0054), and schedules `AhT` discriminator 40 with 6000 ms
(0092–00b0). `AAR.A00` removes the deferred entry and ACKs through A00 when
its once-consumed callback completes. The watchdog `AhT.run` branch
10e6–1204 logs pending descriptors, clears A0A/A0B, and returns; **it does
not send an ACK or retry**. This deferral belongs to other dispatch branches.

The direct field-reference inventory for `0yO.A0A:Ljava/util/Map;` found five
references: constructor write (classes.dex 0x74fcc2), A01 read (0x75004c),
A07 read (0x75016c), AAR.A00 read (classes6.dex 0x4b0e1a), and AhT.run
read (0x5bbfc2). Those bodies were traced above. Reflection/native mutation
and complete alternative Android CCQ transport parity remain **unknown**.

## Wire descriptor

`CmG.A02/A00` selects class=notification and original root from/id/type,
with participant separate. `CSu.A00` passes these to Cb1. It does not copy
recipient/category/edit/lid/display_name. `0yO.A00` has separate BPe
transport branches; its legacy branch calls `0h8.A0N` → A07 → `1q7.A03`.
The latter uses Message.obtain with **arg1=76**, not what=76.
`1q7.A07` case 76 (0202–0220) renders the descriptor through `Cp2.A05`.
The complete legacy writer dispatcher was also searched and contains a
Cp2.A05 → 1Rd.A04 send path; mapping every dispatcher case and alternative
CCQ scheduling is **not established** here.

`Cp2.A05` creates ack with id/to/class/type/participant. Only class=receipt
and type=delivery suppress type (0008–002e); a notification keeps encrypt
(0116–011c). Optional extra fields in the generic renderer are absent from
this notification descriptor. Root device and address representation are
preserved for the ACK; key-query primary canonicalization is a separate step.

## Rust behavior and reproduction

**Implemented/offline:** the authenticated actor captures the control, validates
supported PN/LID metadata, durably queues or skips its key read, and writes
this ACK before dispatching the queued read. Missing held identity, own identity
and a blocked previous read can still be ACKed. This is handling of a control,
not trust in a replacement key and not a message read receipt. Default pinned
identity policy is unchanged.

`transport::identity_notifications::notification_ack` is a pure builder.
`listen --config PRIVATE --seconds 45 --allow-live-request` invokes the actor;
omit --send-outbox and use mark_read=false for receive-only validation.
`actions --show ID` reports the original read/guard/application; ACK handling
uses fixed redacted tracing messages. No HTTP identity-ACK route is added.

Malformed id/participant is captured with a fixed diagnostic and no new read
or ACK. The 128-byte ID bound is local validation, not a claimed APK limit.
Unsupported server-only/group/broadcast/hosted/interop routes remain outside
this implementation. Native malformed-UserJid null/assertion behavior was not
promoted into support for server-only encrypt identity controls.

A failed ACK write ends the epoch with the original queued read preserved.
Startup can send that queued read once but does not reconstruct an ACK.
An uncertain key-query write retains Sent and is never replayed. A newly
authenticated server redelivery can repeat the idempotent control ACK while
reusing the original read ID and baseline. No identity/session/outbox/event
change is implied by the ACK itself.

Tests cover native-shaped PN/LID primary/companion descriptors, typed/text
addresses, omitted extra fields, malformed metadata, first-child dispatch,
missing/own identity skips, ACK-write cut/reopen and uncertain query-write
cut/redelivery. Live identity-control ACK receipt remains **unknown** until a
qualifying control arrives; offline tests do not claim server acceptance.

Negative fixture result: the generic JID parser accepts a bare string as a
server address. A malformed participant test must use an invalid device suffix,
not assume a bare name is unparseable. Generic participant routing is retained;
it never selects the key-query target.

Validation: all 869 workspace/all-feature tests and strict Clippy passed;
14-topic / 19-file / 186,462-byte source export reproduces exactly and cached
self-diff is unchanged. This compares a bundle with itself, not two releases.

Warning-denying Rustdoc, default CLI (25 tests plus API-symbol check), CLI-reference
regeneration, formatting, whitespace and repository hygiene passed (2,395 links,
119 bundles). No fresh network request was made in this implementation chunk.

## Legacy writer closure — 2026-10-09

**Confirmed static**, same 37.71 hash: the previously unresolved legacy
writer mapping is now closed. [Recipe](apk/recipes/2.26.37.71-identity-ack-writer.json)
and [source mapping](apk/2.26.37.71-identity-ack-writer/targets.json) retain
1q6.A00, the complete 0y0.A00 dispatcher and 0y2.A02.

`1q6.A00` returns arg1 unchanged except codes 233/356/6/343/486,
which select arg2; 76 therefore stays 76. `0y0.A00` reads that code
(004a–0052); its non-43 path sets the prebuilt-node slot null
(0056 → 0288 → 0062). The active writer proceeds through 00be,
loads Message.arg1 at 00ce, and dispatches case 76 at 00f0 to 11c2.
That case casts obj to Cb1 and calls `0y2.A02` (11ca–11d0).
`0y2.A02` gets the descriptor child, calls Cp2.A05 (000c–0012),
and invokes `1Rd.A04` with the ACK node and descriptor timestamp
(0014–001e). This is the stanza-received ACK path. A separate Cp2.A05
call at 1a8e belongs to another dispatcher case; merely finding that call
was insufficient to prove the 76 route.

The native writer can queue Message copies in an in-memory ArrayDeque while
offline (0088–009e). That is not proof of durable ACK replay across process
restart. Rust continues to repeat an ACK only on authenticated redelivery;
no saved replay queue or new retry authority follows from this trace.
Alternative BPe/CCQ routing and downstream socket completion remain separate
**unknowns**, not claims of full Android scheduler parity.

## Bounded live validation — 2026-10-09

**Observed:** [redacted validation](identity-notification-ack-live.json) used the
saved account, existing keys and original action IDs. Fresh preflight had zero
queued actions/receipts/eligible chat intents. The 45-second receive window
(59.20 seconds total) authenticated, activated and stopped cleanly with outbox,
automatic reads and reconnects disabled. Actions (21), receipts (9), eligible
outbox (0), account generation, registration and journal head were unchanged.
Inbound checkpoints rose 830→838; other table counts stayed equal. This is a
summary/count check, not a byte comparison of every encrypted record. No identity
ACK success or invalid-metadata log appeared, and no new identity action was
created. **No qualifying identity control arrived; live ACK acceptance remains
unknown.** No repeated window or new chat send was attempted.

The follow-up writer bundle has 3 topics / 8 files / 177,076 bytes. Exact export
and cached self-diff passed; this does not establish parity with a new release.

Follow-up checks: source regeneration/self-diff, CLI-reference regeneration,
API/library symbols, formatting, whitespace and hygiene passed (2,401 local
links / 120 bundles). No Rust changes in this follow-up; workspace tests were
not repeated after the implementation chunk. Private logs/config/state stay
ignored and outside the commit.
