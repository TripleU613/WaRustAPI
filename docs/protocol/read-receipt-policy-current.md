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
