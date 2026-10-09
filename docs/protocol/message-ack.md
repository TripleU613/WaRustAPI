# Outgoing message ACK admission and native status persistence

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Twenty-topic recipe](apk/recipes/2.26.37.71-message-ack.json),
[complete methods, signatures and catches](apk/2.26.37.71-message-ack/targets.json),
[exact anchor and physical-caller inventory](message-ack-evidence.txt).
Offsets below are method-relative bytes. This closes a Rust ACK admission gap;
it does not establish complete native status, user-receipt or resend parity.

## Wire participant reaches the success handler

**Confirmed:** 1Qy.A02 reads wire `participant` through 0hs.A0A(Jid.class,...)
into CSu.A01 at 0036..0046. CSu.A00 passes that as the second JID argument to
Cb1's constructor, which stores it in Cb1.A01; Cb1.A02 is `from`. This assignment
trace prevents guessing the meaning of obfuscated fields.

1Qy.A02 tests the raw `error` attribute at 054a. With no error, it converts
Cb1.A01 through 0DK.A00 at 0552..0560 and passes that result as the first
DeviceJid argument to 1Dz.A10 at 0572..0578. A10 forwards it unchanged to A0H,
which obtains Android Message **what=0, arg1=19, arg2=0** and stores that first argument as
Bundle `participant` through A0g. A0g serializes the supplied JID through
0DE.A09; it does not manufacture a device. BFC's event-19 branch parses that
Bundle member through 0D1.A02 at 00b4..00cc. At 023e..024c a nonnull converted
participant logs the exact ignored-due-to-participant message and returns true
before the ordinary accepted-message worker is invoked.

0DK.A00 returns a DeviceJid unchanged, converts a UserJid recognized by
0DE.A0l to its primary device, and otherwise returns null. A0l recognizes native
numeric types `{0,7,11,18,22,26}`. Therefore a plain supported PN/LID participant
also triggers the guard; checking only `:device` would miss it. **Unknown:** the
complete mapping and native parser behavior for every other user subtype in
Rust's broader server enum. Presence of any arbitrary participant attribute is
not proof of a nonnull native DeviceJid.

**Confirmed:** error-present ACKs instead take 1Qy.A02's 057c..05aa A0z path.
The success-handler participant rule must not suppress that failure branch.
Some newsletter/scheduling branches and the complete Android dispatcher remain
untraced. The inventory has one physical caller each for A0H and A10, not a
claim about reflection, virtual dispatch or all workflows.

## Rust correction and diagnostics

**Implemented, tested offline:** outgoing::acknowledge now ignores success
message ACKs whose parsed participant is a nonempty PN or LID user, including
primary and companion forms and decoded-JID/text attributes. ID and recipient
matching, failure ACK handling and once-only terminal facts retain their existing
behavior. The guard runs before any outbox transaction: no acceptance fact,
identity mutation, new reservation or replay eligibility is introduced.

Restart tests keep the original uncertain intent reserved after these ignored
ACKs; a subsequent matching participant-free ACK can establish acceptance once.
An error ACK with a LID companion participant records the original error code
as Failed and remains nonreplayable across restart. Native additional user kinds,
malformed-JID rejection, counter/token persistence and special native
message classes remain **unknown or unimplemented** here. This bounded guard
must not be described as complete native ACK parity. Success ACK server time
is now retained separately; see [ACK timestamp evidence](ack-timestamps.md).

Use existing `status` to inspect the unchanged uncertainty/facts and `listen`
for authenticated processing. The actor already saves the authenticated node
before ACK handling, so existing encrypted inbox inspection retains the evidence.
No new command or HTTP route is introduced. This correction has no qualifying
live ACK test: no new message was sent, no account was reset, and no uncertain
login was retried.

## Ordinary native accepted-message worker

**Confirmed:** BFC's ordinary non-1iZ branch calls 0zC.A0U at 093a..096c with
the message/key, token, count and timestamp. A0U removes the key from its A1I
map and queues DMZ through 1f0.A01 with discriminator **23**. DMZ's constructor
copies those inputs directly. 1Qy.A02 parses wire `t` through 0Gv.A01 with zero
default then multiplies by 1000 at 049e..04be; forwarding/constructors add no
further conversion. This is DEX long arithmetic, not saturating Rust arithmetic.
Other fields include `server_id`, `count`, `refresh_lid`, `phash`, `edit` and
`rcat`; their complete semantics and persistence are not all implemented.

DMZ.run looks up an existing message at 002c..0034. A missing ordinary message
logs and returns; a special fallback can call 0ze.A04, which is not proof of
ordinary row recreation. Its ownership checks at 008e..00a6 and status-rank
comparison against **4** at 00ce..0114 can return before metadata/SQL writes.
Equal or nonadvancing status takes that early return; a raw numeric ordering
must not replace the native comparator.

At 0206..021c the ordinary worker chooses **13** when 1gd.A0f is true,
otherwise **4**, then sets message A0E to the supplied timestamp. A0f is exactly
key.from_me && status!=6 && own-account BO0(chat). This native own-chat state
13 is not evidence of a human recipient reading. D7J repeats its own selection
at 0230..0258. A separate 6an.A1Z flag check (message bit `0x4000000`) and DdR
per-type hook can instead reach state **21** and return. Flag producers and
complete subtype meaning remain **unknown**.

Later branches at 0282..029a can preserve an old positive A0E. Nevertheless,
the ordinary SQL construction at 07e6..0856 writes `status=B3b()`,
`receipt_server_timestamp=supplied timestamp`, and `recipient_count=message.A06`
to **message**, selected by chat_row_id/from_me/key_id. Thus a universal minimum
rule for memory and SQL would be incorrect. The shown ordinary UPDATE is not
an INSERT into receipt_user; broadcast clones/tables and all transitive effects
remain untraced. Complete native worker effects are retained for further work.

## Other status writes and scoped negative findings

**Confirmed:** 15A.A01(178,int) calls the message setter before updating message
status with the same chat/from_me/key selection. A row count other than one logs
failure; no rollback or ordinary insertion is established. It then continues
metrics and 8o5.A03. Do not assume memory and SQL stay identical under subtype
setter overrides; see [message status](message-status.md).

17J.A01 is a **failure helper**, despite its generic name: it logs permanent
failure and the ordinary fallback sets the supplied status plus 0zC.A0R(...,23).
17J.A03 handles an ACK-processing exception, tries that helper with **20**, logs
another exception if setting the red icon fails, then calls A02. Twenty here
is local processing failure, not an identified server reason or resend authority.
Full A02 effects remain unknown.

0ZG.A04/A06 and 0Y5.A0H were inspected and retained: their relevant log/flow
contexts are poll-vote storage, reverting delivered snapshots and message-add-on
poll-end handling. They do not establish ordinary initial outgoing status or
receipt_user seeding. Their complete poll semantics remain untraced. The exact
lowercase `serverack` string search had no matches; `ReceivedByServer` identified
22 strings including the actual handler/worker anchors. This negative search
is case-specific, not absence of an ACK implementation.

Next work remains actual ordinary outgoing initialization, stored status decoding,
user-row creation and primary-cleanup authority. No receipt_user membership is
inferred from an ACK, device target, DeliveryFact or native status integer.

Validation: all **811** workspace/all-feature tests pass (678 API unit, four
codec properties, nine crash tests, 52 CLI, one API-symbol check, 67 xtask).
Strict workspace/all-target/all-feature Clippy, Rustdoc with warnings denied,
formatting, default CLI tests (25 plus one API-symbol check), and CLI reference
regeneration pass. The twenty-topic bundle is 25 files / 279,970 bytes and
reproduces exactly from APK 37.71; cached self-diff reports unchanged artifacts.
No qualifying live ACK was exercised.

Follow-up: [paginated ACK correlation](ack-correlation-pages.md) removes our
10,000-row history refusal and rejects duplicate wire authority atomically.
It also corrects the Android selector description above: event 19 is arg1.
Complete native status/user-row admission remains unported.
