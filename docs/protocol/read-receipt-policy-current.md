# Current automatic-read privacy guard

2026-11-04, **confirmed static** WhatsApp **2.26.43.72**, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Current evidence](rea/2.26.43.72-read-receipt-policy/README.md).
The [37.71 job](read-receipt-job.md) remains historical evidence; its selected
type7/type21 exceptions must not be copied into the current outgoing selector.

## Current outgoing selection

`SendReadReceiptJob.Cf0` binds policy `0xW`; `A0L` calls `A09` admission,
then `A04` construction. Checked `0xW.A05` 0000..0068 returns `read` exactly
when force=false and all of these hold:

1. `0Dn.A0l(chat)` or preference `09S.A1H()` is true.
2. Neither `0Dn.A0a(chat)` nor `0Dn.A0g(chat)` is true.
3. Both private `A00(chat)` and `A01(chat)` are true.

Otherwise it returns `read-self`. The preference getter reads
`read_receipts_enabled` with compiled defaulttrue; default is not effective
account state. `A0l` admits types1/2 excluding type28; `A0a/A0g` test numeric
types21/7. Their full human-domain names are not inferred. Unlike the older
outgoing selector, these types do not bypass the preference into normal read.

Private A00 inspects nullable UserJid, allowlist/contact records, addon flags
and `2Qb.A00`; it can suppress possible spam. A01 checks dependent-account
policy and a contact/chat predicate. Those complete authorities remain unknown.
Thus a true preference alone does not prove the full native normal-read policy.

Checked `A09` selects the recipient as policy chat only for the special
`Exz.A00` chat; otherwise it uses the original chat. `A07` admits when `A08`
or the eligible-own-companion set is nonempty. It rejects an invalid message ID,
then applies a status-age guard: positive originaltime plus86,400,000 must not
be older than current clock (native long addition). Finally `read-self` also
requires a nonempty eligible-own-companion set. These are local admission
rules, not server wait/retry authority.

`0x0` is the constructor producer. Its A0V uses force=A0H(message), which
forces self for C1eq, type7, a special-user predicate, or A0G(message) with
preference disabled. A0E has two forced-self job constructions; A0d copies
its batch descriptor's force flag. Status A0P sets receiptClass=status and
has separate property39224/special-user conditions. Full producer predicates
and specialized status/recipient routes remain unported.

## Implemented bounded negative gate

The existing operator opt-in `transport.mark_read` now retains a newly
automatic direct normal-read intent when the durable validated server
preference is known disabled. It creates no read-self substitute. This follows
a necessary condition in current A05, not an assertion that every positive
native contact/force condition has been implemented. Unknown preferences keep
the existing operator-authorized behavior; no default is synthesized from the
APK. Existing group behavior and public explicit `queue_read` remain separate.

New read outbox rows carry explicit provenance `automatic=true/false`; old rows
remain unknown. The preference check and intent queue happen in one encrypted
store transaction. Blocked intent stays pending under its original ID/wire with
fixed reason `server_read_receipts_disabled`. Queue count is zero for held work,
so the receive loop does not claim it was marked read. The inbox's existing
read_receipt=queued marker still means a durable intent, not a socket write.

Reservation checks the known server preference again in the same transaction
that reserves a pending send. Blocked automatic direct rows stay pending,
do not consume the16-ID reservation batch, and retain their fixed reason.
An eligible later reservation clears that reason and reserves the original
wire once. A privacy change never resets sent/uncertain rows. Unclassified
legacy intent is not relabeled or blocked through guessed provenance.

An explicit user read can promote only a still-pending automatic intent to
explicit using the same selector/wire. It removes deferral and cannot reset
an already reserved or acknowledged outcome. This preserves the explicit
user-action semantics without inventing a new ID to resend.

Existing `receipts` text output now shows origin=automatic/explicit/unknown
and deferred=<fixed reason>/none. Existing JSON/HTTP ReceiptOutcome adds
nullable `automatic` and `deferred_reason`; no new route or switch. These
fields report local queue state, not handset observation. The server preference
comes only from the supported [privacy category adoption](account-privacy-notifications.md).

## Own-companion and read-self continuation

Current `0jD.A02` returns empty when `09F.BUx` is false, otherwise obtains
`1nG.A08().keySet()`. BUx is an interface method; its concrete binding is
not established here. `1nG.A08` filters `1nH.A06` values through `F0I.A04`;
that tests whether A02 is positive. The current field's loader binding and full
ingestion/deletion/key authority remain unclosed; the analogous historical
[37.71 logout-time trace](read-linked-set.md) is not reused as current proof.

`1nH.A06` preserves the two-pass own-LID-to-own-PN conversion: retain other
entries first, then convert own entries preserving device number and metadata;
an existing PN entry wins. It caches the converted map. A current PN identity
alone is not an eligible companion set. `Exf.A05` preserves specialized route
selection and optional remote sender, but its full classifier domains remain
unknown; no arbitrary peer/device inventory supplies read-self authority.

Full read-self processing, own-companion source verification, current writer
receipt type and route, status/recipient/class handling, contact/force policy,
and automatic native job lifecycle remain required work. This negative guard
does not fulfill those obligations. No live requests, account resets, fabricated
provenance or account-restriction claims occurred.

Offline verification: three new encrypted-store regressions cover deferral and
explicit promotion, privacy changes between queue/reservation with reopen,
group/explicit exemption, later release without replay, legacy provenance and
malformed-flag rollback. All39 receipt-module tests passed; the existing actor
automatic-read write-cut regression also passed.

## Writer-bound preference recheck (2026-11-06)

The normal actor and CLI now retain all members of a reserved receipt batch and
recheck the existing automatic direct-read negative gate at writer admission. A
late disabled preference blocks a mixed batch containing an automatic direct
read; explicit manual/group policy remains unchanged. Refused unknown rows are
held, not requeued. [Owned API, batch/journal/reopen controls and limits](replay-safety-audit.md#receipt-writer-owns-every-reserved-batch-member-2026-11-06).
No full native classifier/read-self or live claim.

## Automatic played privacy correction (2026-11-06)

**Confirmed pinned mismatch / Rust correction:** [complete current played job,
checked DEX policy branches and REA limitations](apk/2.26.43.72-played-privacy/README.md)
show normal direct played requires enabled read-receipt privacy; otherwise the
native job selects played-self. Rust queue_opened had always emitted played,
including when a validated server preference was known disabled. The private
contact/block/addon gates are additional conditions, not overridden by a true
preference. Full played-self/companion routing remains unported.

Viewer completion now creates automatic=true provenance. The shared negative
automatic privacy gate applies to direct read/played at queue, reservation and
the owned writer. Known-disabled played stays pending under the original wire,
selector and consumed inbox marker with server_read_receipts_disabled. It does
not consume a reservation slot. Enabling privacy can reserve the same pending
wire once and clear deferral. If privacy changes after reservation, the writer
refuses before encoding without releasing outcome_unknown. Reopen never
reconsumes the image or resets/retries that reservation. CLI completion now
says privacy policy may defer sending; receipts/outcomes expose the fixed reason.

Two new encrypted-store regressions failed before the fix, including the real
NodeLink -> Outbound.prepare handoff admitting late-disabled played. They cover
PN/LID queue-time and reservation-time holds, reopen, later release with identical
wire, unknown preference, legacy unknown provenance, group controls, late writer
refusal preserving every checkpoint and journal, and consumed ticket refusal.
Only synthetic consumed fixtures are used; no image/viewer/account is opened.
Existing read and callback ownership regressions remain applicable.

Compatibility: unknown preference retains existing behavior; old unclassified
played rows are not relabeled. Explicit read override and group behavior stay
separate. This is a necessary negative privacy gate, not full positive native
admission, blocklist synchronization, played-self implementation, cross-device
opened synchronization or safe native retry. No server acceptance or ban-rate
claim. The first parallel workspace run hit an unrelated handshake cancellation
test failure; it passed alone and in the four-thread API rerun. That failure's
scheduling cause remains unproven. The subsequent workspace run passed API/CLI
suites but found the coverage report stale after adding new retained evidence;
the report was then regenerated before the final full check.

Final validation:1,515 workspace tests pass in9 result suites with four test
threads; strict workspace/all-target/all-feature Clippy, formatting, CLI
reference regeneration, OpenAPI library coverage, APK coverage freshness and
repository hygiene pass. Three selected DEX exports reproduce byte-for-byte.
REA subject hash and complete source are retained with its duplicated-code
warning; checked DEX determines the selection branches.

## Pending UI views versus the receipt Boolean (2026-11-06)

**Confirmed selected static:** [separate getter/observer paths](apk/2.26.43.72-privacy-optimistic-state/README.md)
show0oe.A0L's optimistic integer overlay and8Y1's pending-entry test are not the
same operation as09S.A1H reading read_receipts_enabled. The latter's stored
Boolean branch remains the retained current0xW.A05 input. Downstream observer
callbacks, every writer and full caller domains remain open; no receipt policy
or inferred local-setting default is changed from this trace. No live operation.

## Current ACK-triggered preference writer (2026-11-06)

**Confirmed selected static:** [current227 synchronization](apk/2.26.43.72-read-setting-sync/README.md)
re-anchors another real read_receipts_enabled writer in15E.BLn. Its read-self
ACK path preserves localfalse on receivedall and can queue0oe.A0P(none), unlike
category-map adoption. The outgoing preference getter is unchanged; full mode,
legacy registration, dispatch/order and corrective-send recovery are unported.
No new Rust preference or read/read-self selection rule follows from this
bounded source trace, and no missing preference is replaced with compiledtrue.

## Stored payload shape errors in automatic selection

**Confirmed Rust diagnostic defect**, baseline `32b4a7a0`, corrected offline.
`queue_read_for_incoming` treated a decrypted row whose protobuf field could
not decode as a byte vector as a successful zero-work result. This hid a stored
record fault behind ordinary read ineligibility. It now returns a fixed redacted
InvalidInput error before receipt construction or writes. Missing/null fields,
text and out-of-range byte elements are invalid saved payload shapes.

The actual queue entry regression uses direct and group fixture routes, corrupts
only the stored payload field, closes/reopens the encrypted store, and verifies
an error with unchanged checkpoint rows/journal and no receipt intent. No network
is used. The existing successful fixture tests remain controls. Byte-vector
payload inspection still skips unsupported/malformed protobuf and the inspection
size bound as before; this change does not broaden supported message types or
normal-read authority. This is source/store-shape evidence, not a new APK rule,
server acceptance result or proof of every runtime caller's error handling.

## Read/played preparation account binding

**Confirmed Rust ownership defect, corrected offline**, baseline `47cf4c22`.
New read/played intents previously acquired account authority only at first
reservation. A changed account could therefore adopt old pending work. The
new real-store regression failed on old code with read/jid: stale prepared
receipt was reserved. This is a local ownership failure, not a demonstrated
native server rejection or restriction trigger.

When an account exists, queue_read_with_origin and queue_opened now record the
existing receipt authority fingerprint as prepared_account_authority_sha256
inside the same transaction as intent preparation. reserve_pending_owned's
existing preparation check then refuses a changed/missing account before
mutating pending rows or creating callbacks. Explicit promotion of an existing
automatic read intent also checks its retained preparation marker before
changing origin/deferral fields. It never rebinds the old row to current authority.
The fixed error now says prepared receipt account changed for all families.

Offline regression covers explicit direct read, automatic direct/group read and
played preparation, then changed own JID, Signal registration ID or account
removal, encrypted-store close/reopen, and reservation refusal without checkpoint/
journal mutation. Automatic-direct explicit promotion is also refused after
those changes. Same-account and observation-timestamp-only controls reserve once
and cannot replay. No socket is used and encoded receipt bytes are unchanged.

Account-less offline adapters remain explicitly unbound, consistent with the
existing retry adapter. Historical rows lacking the preparation marker are not
backfilled; original historical inbox/account provenance remains unknown.
Preparation ownership does not itself prove original message ownership or
complete native policy. Reserved callback/write/ACK authority checks remain
independent. No receipt is sent and no account operation is performed.

## Read/played prepared wire integrity

**Confirmed Rust preparation defect, corrected offline**, baseline `b506089f`.
The account preparation marker did not freeze read/played wire bytes. A new
real-store regression changed a queued read's destination to another valid JID,
reopened the store and reproduced first reservation adopting that changed wire.
This is a local preparation integrity defect, not an observed server restriction.

New read/played intents with a saved account now retain prepared_wire_sha256
alongside their preparation account marker. The shared existing reservation
check validates that digest before decoding/adopting pending bytes; its redacted
error now says prepared receipt wire changed for all receipt families. Explicit
promotion of an owned automatic read intent uses the same check before modifying
origin/deferral fields. Stored intent digests are not recalculated to bless changed
bytes. Encoding itself, reserved callback snapshots and retry policy are unchanged.

Offline regression queues two real intents per explicit-read, automatic-direct,
automatic-group and played family. It changes the last physical receipt row's
wire destination, closes/reopens the encrypted store and proves reservation fails
with checkpoints/journal unchanged. Direct automatic/played controls disable
privacy after preparation, so rollback also preserves any earlier row deferral
changes attempted in the reservation transaction. Explicit automatic-read
promotion likewise refuses the changed wire without mutation. Unchanged controls
reserve both members once without replay. No socket is used.

Account-less offline adapters and historical rows without preparation digests
keep their explicit unbound/unknown provenance; no proof is backfilled. This
checks mutation after local preparation, not the original authenticated message's
historical account ownership or complete native receipt parity. No live operation.
