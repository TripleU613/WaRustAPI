# Specialized receipt ACK state versus peer receipt time

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Six-topic recipe](apk/recipes/2.26.37.71-receipt-ack-state.json),
[selected methods](apk/2.26.37.71-receipt-ack-state/targets.json).
This connects the [concrete ACK handlers](receipt-ack-handlers.md) to existing
[state-only admission](receipt-state-only.md), reusing the full
[Cmn.A04 application](apk/2.26.37.71-receipt-ack-handlers/state-application-boundary.txt)
and [Cmn.A00 store](apk/2.26.37.71-receipt-apply/apply-store.txt).
It does not establish every subtype/canonicalization path or live parity.
Offsets are method-relative bytes.

## Model getters preserve the producer's zero time

**Confirmed:** DDH.BSJ indexes its supplied key array. CFC returns A02 device,
CJI returns A04 original user, CKw returns A03 chat, CJB returns A06 metadata,
CbP returns A05 stanza key, Ccf returns A00 status and CeD returns A01 time;
size returns array length. Ccf/CeD/CFC ignore their supplied index, sharing
those values across the model's keys. AT2 returns MULTI_MESSAGES.

Consequently the previously traced A16/A17 ACK producers retain status **10/12**,
time **0**, null device/original-user/metadata/stanza, and their one false-direction
key. No getter fills in the current time or recasts these as peer delivery/read
observations. The flag stays false even though generic ACK correlation separately
settles an outgoing receipt reservation.

## Conditional path to state-only admission

**Confirmed:** Cmn.A04 obtains per-item device/time/key at 02a0..02b8. Its routing
can normalize a device or derive one from the remote chat. The ordinary fallback
at 0aa6 calls Ayl.A0V, which delegates to 0DK.A00. That helper returns an existing
DeviceJid unchanged, converts an admitted UserJid to its primary device, otherwise
returns null. Complete upstream chat branches remain **unknown** here; this is
not unconditional substitution of a missing participant on every ACK.

After message lookup and recipient validation, A04 constructs CZ4 at
0568..0586. Its constructor stores status A00, time A01, device A02, original
user A03, message A04 and metadata A05. Ordinary selected items reach the v30
pool at 06ba..06c4 and Cmn.A00 at 0cdc..0ce6. Other message subtypes use different
pools; absent/invalid messages can skip application. Key direction and lookup
must remain part of any eventual Rust port.

**Confirmed from existing source:** 1hg.A03 recognizes only statuses
**5, 8, 13, 15, 17, 18**. Both 10 and 12 are false. In Cmn.A00, selected items
with a false predicate take the branch at 00de. Message status **21** logs
permanent send failure, records a true result and skips the setter path.
Otherwise 0130..0136 calls **15A.A07(message, null callback, supplied status)**.
The true map entry on the skip is not evidence that the status changed.

A07 separately rejects equal/backward rank, requires played eligibility for
10, then mutates the virtual message status and calls its persistence path.
Its true return does not prove every core write succeeded. Status 12 skips its
0Yq hook; status 10 does not. Full subtype eligibility, updater/hook behavior
and execution of this path on any current account remain **unknown**.
The shown non-six-status branch does not route these selected 10/12 items through
CNn's timestamp value factory or 15A.A02's per-user writer. This is a bounded
branch claim, not global absence of SQL or other callback effects.

## Deferred work preserves keys, status and time

**Confirmed:** DDH.CPB copies only the unprocessed suffix of its key array,
then constructs ReceiptProcessingJob with the same device/chat/original-user,
metadata, status and timestamp; caller-supplied deferral count, total duration
and Boolean remain distinct. It does not copy stanza A05 into the job constructor.
The job constructor uses ReceiptProcessingGroup and stores aligned key ID,
fromMe and chat-string arrays plus status/timestamp, optional address strings,
privacy mode, deferral count, process duration and useStatusInfra. Actual jobqueue
serialization/durability and effective group scheduling remain **unknown**.

ReceiptProcessingJob.A0G reconstructs keys with their saved direction, skipping
entries whose chat parse is null, builds DDH with saved status/time and null
stanza key, then chooses by useStatusInfra. True calls Cif.A01 and waits on AzY;
false calls Cmn.A03 with the saved deferral count/duration and budget **10000**,
then waits on AzY. The ACK handler's original budget is **5000**. Neither number
is a WhatsApp server wait or permission to repeat a network request. Complete
Cif routing, parsing exceptions and jobqueue retry policy remain unknown.

## Rust boundary

**Confirmed local inspection:** receipts::record_ack settles only its matching
outcome_unknown receipt reservation. Peer ReceiptFacts/DeliveryFacts are populated
separately by incoming receipt handling. Keeping ACK acceptance separate from
peer played/read facts agrees with the zero-time source boundary above. Rust has
not implemented the complete native incoming-message status/hook pipeline, so no
state transition is invented from this research. Primary identity cleanup still
requires actual per-user row authority, not ACKs or raw delivery observations.

No runtime code, account identity, session, request ID, reservation or server wait
changed. No live request ran. The full Cmn A00/A01/A02 bodies and 1hg.A03 were
rechecked against their existing snapshots; they are reused rather than duplicated.

Validation: exact re-export verifies 11 files / 48,877 bytes; checksum-validated
self-diff reports six topics and four metadata artifacts unchanged. CLI-reference
regeneration, API symbol check, formatting and whitespace checks passed. This
research-only chunk uses the preceding 813-test runtime/tool baseline; no fresh
workspace suite or live behavior is claimed.
