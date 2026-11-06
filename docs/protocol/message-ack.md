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

### ACK callback normalization and inbound bridge (2026-11-04)

**Confirmed static, APK43.72:** Ewq/Eng preserve the job's Eoh callback key.
0Sf.A02 canonicalizes supported device-zero from/participant fields to user
JIDs and retains nonzero companions. This helper does not establish PN/LID
alias equivalence. The generic1lZ.A1D ->1lY.Big ->0ln.A09 bridge is now
connected; the selected inbound bridge does not call that outbound normalizer.
A0T is exact normalized-key failure removal, not success handling.
[Field propagation, type predicates, callers and port limits](apk/2.26.43.72-ack-key-normalization/README.md).
Distribution producer inputs and parser/type mapping still need tracing before
changing expected-address admission. No production or live behavior changed.

### Auth-challenge ACKs preserve uncertainty (2026-11-04)

**Confirmed static / fixed offline:** APK43.72 1rH.A03 diverts numeric459
to onAuthChallengeReceived before ordinary ACK class processing. The traced
manager transfers an existing request between maps; it is not an ordinary
terminal message failure or proof of a ban. Rust direct/group ACK settlement
now excludes459, preserving original reservations and uncertainty. Java decimal
spellings are covered; no challenge bypass, retry or account-wide policy added.
[Methods, before/after regression and unported boundaries](apk/2.26.43.72-ack-auth-challenge/README.md).
No live operations. Full challenge-controller parity remains incomplete.

### Ordinary message ACK forwarding (2026-11-05)

**Confirmed static, APK43.72:** selected ordinary message success and refusal
event paths both jump back to generic A1D with the original node/Eoh key.
This closes those forwarding edges into the previously traced concrete callback
bridge. Error459 and malformed/early paths remain separately qualified. The
destination rewrite helper inspected here is FBID-migration-specific; Efa's
null type default does not prove the out-of-band producer leaves type unset.
[Exact offsets, retained exports and remaining producer gap](apk/2.26.43.72-message-ack-forwarding/README.md).
No runtime guard, replay policy or live account operation changed.

## Integral error admission before ordinary ACK outcomes (2026-11-06)

**Confirmed static, APK2.26.43.72 / Rust defect fixed offline:**
[1rH.A03,0Tp.A05/A06](apk/2.26.43.72-ack-auth-challenge/README.md)
parse root error with Integer.parseInt before special459 and ACK class dispatch.
Malformed strings and values outside signed32-bit range throw LIL; they do not
reach the ordinary message/receipt callback branch. Rust's error-presence checks
could instead create terminal failures from these values. A new classifier
regression failed on that old Refused behavior.

Shared ACK admission now distinguishes absent error, valid signed Java decimal
error, and malformed/unsupported error. It reuses the existing Java-long parser
with i32 range checking. Missing error preserves ordinary success semantics;
valid present errors preserve ordinary refusal semantics, including signed,
zero and Unicode decimal forms. Valid numeric459 retains its special challenge
diversion. Invalid/nontext values cannot become ordinary acceptance or refusal.
Nontext decoded attributes are a local unsupported boundary, not claimed native
A0M coercion parity.

The shared message_ack_outcome returns Unrelated/outcome_unknown for malformed
errors, so CLI direct/group comparison status remains uncertain. Normal direct
ACK settlement, group out-of-band distribution admission and receipt ACK
settlement use the same validity gate. They retain in-flight/preparation/receipt
reservations without recording failure, promoting sender-key recipients or
authorizing replay. Direct/receipt helpers emit redacted malformed-error
diagnostics. The actor's existing encrypted capture precedes handling.

### Selected reader exception path

The already retained [1s2.run](apk/2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
dispatches the selected mapped handler through1rA.A03 at106c..1072. That call's
code-unit offset0836 lies in the exported060f..085c range with an LIL handler
listed at0873 (byte10e6). The selected handler10e6..10f6 calls1rg.A04(node,LIL),
then branches back toward the reader loop if that call returns. At that tracing stage, full nested outer-catch policy, concrete A04 effects,
generated error traffic and native connection recovery were untraced. The
malformed-ACK epoch follow-up below now binds the selected A04/outer-catch exit;
complete listener/recovery and other stanza policies remain open. This proves a parse-error route distinct
from the ordinary ACK branch; it does not justify a new Rust error response,
forced reconnect, callback reset or complete native loop-parity claim. No new
APK export or execution is claimed.

Regressions: new `integer_error_admission_precedes_ordinary_ack_classification`
covers absent, zero/signed/i32 extrema and Unicode valid controls; empty/junk/
whitespace/sign-only/hex/fraction/NUL/overflow inputs and a nontext boundary.
Existing direct in-flight reservation, real-crypto group distribution and
single/batched direct/group receipt challenge controls now also exercise
malformed and out-of-i32 errors. Normal later success/refusal controls remain.

**Limits:** this is root ACK error admission. Sender/address/duplicate-attribute
policy, typed error-code storage, receipt-user state and full native error
handler/recovery are unchanged or incomplete. Low-level callers still own
correlation authority. No new request, receipt, automatic retry, identity reset,
challenge recovery or live account operation.

Validation:1,455 workspace/all-feature tests passed. Strict workspace/all-target/
all-feature Clippy, formatting, CLI reference regeneration, OpenAPI symbol
validation, APK coverage freshness and repository hygiene passed. Offline only.

## Malformed ACK consumes the receive epoch after capture (2026-11-06)

**Confirmed pinned static / Rust correction:**
[new1rg methods, recoverable-tag set/census and selected outer catch](apk/2.26.43.72-stanza-corruption-routing/README.md)
show that malformed root ack is outside the recoverable stanza set. A04 rethrows
LIL; the selected1s2.run outer catch reports1lX.onError(2) and returns. It does
not construct a487 NACK for an ACK, even for class=receipt. Root receipt and ACK
class receipt are different routing inputs. Concrete native listener recovery
remains untraced, so no account-wide logout/ban or automatic retry is inferred.

Rust previously excluded the malformed ACK from ordinary settlement but kept
its receiving epoch alive. Public `messaging::validate_received_ack` now returns
a redacted non-reconnectable local error for malformed ACK numeric admission.
It performs no capture or I/O itself. Normal drive invokes it immediately after
encrypted capture, before clock/state processing; ending drive drops the live
callback registry and its owned link tasks. The CLI lab receive loop validates
after its durable inbound write too.

Captured comparison waits call the same validator after preserving the Node,
before matching or reading another response. Their original unknown send
reservations remain untouched on failure. The legacy uncaptured wait_for also
stops on malformed ACKs under its existing TransportError contract; stateful
callers should use the captured APIs. Valid numeric errors and459 remain
accepted by numeric validation and retain the preceding outcome rules.

Regressions: `malformed_ack_stops_epoch_after_capture_and_keeps_reserved_action`
uses the actual actor with a written pending action. Malformed ACKs across
message/receipt/notification classes end promptly, do not answer a queued ping
or emit later writes, return an error that is not reconnectable, preserve
account/outbox/session/pin/non-capture checkpoints and retain capture/Sent state
across reopen. `captured_wait_stops_on_malformed_ack_before_matching_or_next_read`
checks matching and unrelated malformed ACKs: only one Node is read/captured,
the following valid response remains unread and the unknown intent survives
reopen. Valid signed error and challenge controls still pass numeric validation.

**Limits:** the validator does not mechanically poison a borrowed raw transport;
direct callers must discard it after failure. Normal actor/CLI owned paths stop
and drop their channel. Raw read_frame/read_node remain low-level primitives.
This is numeric-error admission, not complete validation of ACK id/from/class/
participant fields. Full native listener/map cleanup/reconnect policy, generated
NACKs for other root tags, diagnostic/provider semantics and nontext A0M coercion remain open.
Rust's non-reconnectable stateful error is an explicit conservative local
policy, not a claimed mapping of all native onError(2) outcomes. No outgoing
error response, reset, challenge recovery or live account operation is added.

Validation: 1,457 workspace/all-feature tests passed. Strict workspace/all-target/
all-feature Clippy, formatting, regenerated CLI reference, OpenAPI symbol
validation, APK coverage freshness and repository hygiene passed. Offline only.

## Preserve admitted signed ACK error codes (2026-11-06)

**Confirmed pinned static / Rust projection correction:** the retained
[1rH.A03 and 0Tp integer parser](apk/2.26.43.72-ack-auth-challenge/README.md)
admit signed Java i32 errors before ordinary ACK dispatch. Rust already uses
that parser for admission, but outgoing::acknowledge reparsed the same text
as ASCII u32 when creating DeliveryFact. Thus a valid negative or Unicode-digit
refusal became Failed with code=None. This lost diagnostic information even
though the encrypted capture retained the original frame.

The fact writer now reuses messaging::parsed_ack_error. DeliveryFact.code is
Option<i64>: new ACK facts contain only admitted i32 values, while the wider
JSON storage type preserves historical u32 values (including u32::MAX) without
rewriting old facts. This is a public Rust type change from Option<u32>;
consumers with explicitly typed u32 values must convert them. JSON field names
and ordinary positive values remain unchanged. Receipt.code and generic error
codes are separate models and remain unchanged.

The real outbox regression covers ASCII signed/positive errors, Unicode zero
and full-width digits, negative Unicode i32::MIN and i32::MAX. It acknowledges
once, rejects duplicate settlement, reopens the encrypted store, verifies the
exact Failed code and refuses re-reservation. A historical u32::MAX JSON fact
remains deserializable. Existing malformed/error459 guards still run before
fact construction. No new settlement, replay, challenge handling, request or
live account operation is authorized. Full ACK envelope validation and native
listener recovery remain incomplete.

Validation: 1,457 workspace/all-feature tests, strict workspace/all-target/
all-feature Clippy, formatting, regenerated CLI reference, OpenAPI symbol
validation, APK coverage freshness and repository hygiene passed. Offline only.

## Required ACK ID before error and class routing (2026-11-06)

**Confirmed pinned static / Rust correction:** [required-text accessor and
selected handler](apk/2.26.43.72-ack-required-id/README.md) show that absent id
throws LIL before numeric459 diversion or any class routing. The same selected
corruption route rethrows for root ack and exits the reader. Rust previously
kept reading after this malformed envelope when its numeric error was valid.

The shared validate_received_ack now rejects absent id before numeric checks.
Normal actor, captured waits and CLI stateful receive capture first and stop,
retaining reservations; legacy uncaptured wait stops under its existing error
contract. Empty textual id passes this accessor, as in the native null check.
Missing-id error is redacted and non-reconnectable in stateful paths. This is
a conservative local recovery policy, not a native account-wide policy claim.

Regression controls cover missing class, unknown/message/receipt class, error
absence,459 and malformed numeric input, checking ID precedence and empty-id
admission. Actual actor tests include missing-id459 for message/receipt/
notification classes, durable captures, no later ping response, preserved
pending action/account state and reopen. Captured waits include missing-id459,
stop after one read and retain the following valid response unread. No live
traffic, ACK-to-ACK response, replay or challenge recovery.

Limits: complete sender/participant/recipient validation, nontext ID coercion,
duplicate attributes and Efa/Eoh construction remain unverified. Pure outcome
helpers and raw read primitives still do not replace stateful admission.

Validation: 1,458 workspace/all-feature tests passed. Strict workspace/all-target/
all-feature Clippy, formatting, regenerated CLI reference, OpenAPI symbol
validation, APK coverage freshness and repository hygiene passed. Offline only.

## Optional typed ACK addresses are not a blanket fatal gate (2026-11-06)

**Confirmed pinned static / negative finding:** [new A0A and optional JID
parser exports](apk/2.26.43.72-ack-address-accessors/README.md) show that the
ACK handler's from/participant Jid and recipient UserJid accessors differ from
required id. Missing/unparseable text produces null through 0Dd.A02; a
protocol-compliance failure is logged without rejection by that branch; a
failed requested-class cast is logged and returns null. These cases do not
justify adding a blanket fatal-address check to validate_received_ack.

Rust receipts::jid already handles absent/unparseable addresses as None;
outgoing::acknowledge requires a parsed matching from for its supported ordinary
settlement. This is not full equivalence: native message routing first prefers
nonnull recipient, otherwise normalizes from via0Dn.A00, and its no-chat path
still reaches a generic ACK callback. Recipient-based correlation, exact JID
subtype/normalization rules and that generic callback's ownership need further
tracing before widening Rust settlement. Nontext cached representations and
all helper exception paths remain unverified. No runtime claim, code change,
new request/response, replay or stricter connection-stop policy in this audit.

## Select the message ACK chat before outbox correlation (2026-11-06)

**Confirmed pinned static / Rust correction:** [normalization, user factory,
subtype census and success dispatch](apk/2.26.43.72-message-ack-chat/README.md)
resolve the preceding recipient-routing gap for supported numeric PN/LID
addresses. 1rH.A03 selects a typed recipient first, otherwise unwraps device
from to its user, then constructs its message key. Original from remains
separate remote-device metadata. Plain PN/LID bypass special type26/27 mapping.

outgoing::acknowledge now correlates its uniquely reserved wire id against
that selected chat for supported plain numeric PN/LID recipients. It normalizes
PN/LID device from when no user recipient applies. Conflicting supported
recipient cannot settle an entry merely because from matches. Known non-user
or device recipients use from because the native UserJid cast does not accept
them. Other parsed recipient families/forms remain uncertain; this adapter's
1..15 ASCII-digit PN/LID support bound is not a complete native grammar claim.

A real outbox regression covers matching recipient with another from,
conflicting recipient with matching from, device-from normalization, PN/LID
namespace disagreement, non-user/device recipient fallback, and actual LID
outbox entries. Success and ordinary refusal run independently; duplicates
cannot rewrite facts, conflicting rows remain unchanged, store reopen retains
results, and neither accepted nor uncertain reservations are reselected.

Limits: this corrects ordinary saved-outbox settlement, not every distribution
wait or comparison CLI. Full generic callback authority, special/bot feature
policy, hosted/interop normalization and complete JID grammar remain open.
No identity namespace merge, generated request or automatic replay is added.

Validation: 1,459 workspace/all-feature tests passed. Strict workspace/all-target/
all-feature Clippy, formatting, regenerated CLI reference, OpenAPI symbol
validation, APK coverage freshness and repository hygiene passed. Offline only.
