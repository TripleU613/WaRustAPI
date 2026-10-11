# Durable outgoing read-receipt batching

2026-10-10. **Implemented/offline**, grounded in APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This extends the [native read-job trace](read-receipt-job.md). Queued matching
read routes now share a native extra-ID list at reservation. Public queue counts
and offline outcomes still describe individual message IDs, not stanza counts.

## Native wire and ACK evidence

**Confirmed static:** [0zd.A04](apk/2.26.37.71-read-receipt-job/read-handler-build.txt)
uses the first message ID for CM8.A07 and copies remaining supplied IDs to A03.
[Cp2.A07](apk/2.26.37.71-read-receipt-job/read-wire-adapter.txt) passes that array
to [Cp2.A00](apk/2.26.37.71-read-receipt-job/read-wire-builder.txt).
[Cp2.A0A](apk/2.26.37.71-read-receipt-job/read-list-builder.txt), `0016..0052`,
creates item/id children in array order, then one list child. The first ID remains
at the receipt root; only the remainder enters the list. A single ID omits list.

The complete [SendReadReceiptJob.A0G](apk/2.26.37.71-read-receipt-job/read-job.txt)
uses the receipt route/type and `messageIds[0]` (`00fa..0106`) for its CSu ACK
selector before waiting. Extra IDs are not independent wire ACK selectors.
Existing [participant ACK evidence](receipt-ack-correlation.md) keeps nullable
participant distinct from recipient; it does not allow a different group member
to settle the receipt. The checksummed topic bundle is unchanged by this port.

Example of the native shape for three authenticated IDs:

```xml
<receipt to="15555550100@s.whatsapp.net" id="first" type="read">
  <list><item id="second"/><item id="third"/></list>
</receipt>
```

## Reservation, recovery and API

`receipts::queue_read` still validates every authenticated incoming record and
atomically queues each original per-ID reservation. Unknown/unauthenticated
records or ambiguous group attribution refuse the entire queue operation.
Newly queued read attributes follow Cp2.A00 order: to, id, type, participant.
Existing pending, uncertain and terminal reservations keep their selectors and
wire bytes; repeats and overlapping read calls do not authorize another send.

`reserve_pending` selects at most sixteen per-message reservations, the existing
local bound, then groups plain read receipts with identical supported route
attributes apart from id. Saved attribute values and their order remain intact:
there is no PN/LID merge, device collapse or route canonicalization. Direct PN/LID
companion destinations are retained; group member devices remain separate.
Played/retry, metadata-bearing and existing child-bearing receipts stay separate.
The first selected ID stays at root; subsequent selected IDs enter list in the
local saved-reservation order. This implements the builder's ordered format,
not every Android job producer's scheduling or input ordering.

All selected rows become `outcome_unknown` in one transaction before any socket
write. For each newly batched row, encrypted checkpoint `batch_ack_id` stores
the selected root ID, while its original single-ID wire remains byte-for-byte
unchanged. Inspection continues to show the original message ID and validates
this optional saved selector using fixed, redacted errors. Historical rows
without it keep their original single-ID ACK correlation.

A matching root ACK must satisfy existing class/type/to/from and exact nullable
participant checks. It atomically marks every matching batch member
`server_acknowledged`, or `refused` when the ACK has an error. An extra-ID ACK,
wrong destination/type/member or repeated terminal ACK does not settle the batch.
Corrupt saved correlation causes a fixed error and rollback, without a partial
settlement. Server ACK is receipt-request acceptance, not proof of a UI indicator.

A failed or uncertain batch write is never replayed on reconnect or reopen.
A later authenticated root ACK can still settle the original reservations.
Reservations from other batches whose writes were never reached also remain
unknown, preserving the existing conservative socket-cut behavior. No request ID,
message ID, account key, message content, outbox intent, journal event or server
wait is fabricated or reset by batching.

Existing CLI `mark-read --chat CHAT --id ID --id OTHER` and library `send_receipt`
queue offline; `listen` sends pending receipts under the existing authorization.
`receipts` reports one outcome per original ID. There is no new switch, HTTP
route, response field or automatic read policy. Opt-in `mark_read` retains its
existing readable-content and authenticated-record checks.

## Checks and remaining scope

The updated authenticated multi-ID regression failed on the old two-stanza output
before implementation. It now proves the native list after binary encode/decode,
original IDs, duplicate-free queue/reopen, no uncertain replay, root-only ACK and
per-message terminal outcomes. Tests cover direct LID companion destinations,
group device separation, played/retry exclusion, batch refusal, the sixteen-ID
bound, singleton legacy ACK and all-table rollback for corrupt saved selectors.
The actor's real send interface is exercised with a two-ID write cut, new epoch,
no repeat write and later ACK settling both reservations.

Validation: 911 workspace all-feature tests, strict Clippy, warnings-denied
Rustdoc, default CLI/API-symbol tests, experimental CLI-reference check and CLI
build passed. The default build retains its existing `observe_login` unused
warning. A separate failing-before regression confirms the group attribute-order
correction.

Fresh live multi-ID read ACK and handset read indicators remain unverified; no
new network request was made. Complete native read/read-self privacy policy,
linked-device authority, specialized routes and Android scheduler parity remain
unported. This closes the wire batching and durable ACK lifecycle, not those
remaining client requirements.
