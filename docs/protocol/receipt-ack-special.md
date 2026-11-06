# Specialized read, played and server-error ACK messages

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Fourteen-topic recipe](apk/recipes/2.26.37.71-receipt-ack-special.json),
[complete selected methods](apk/2.26.37.71-receipt-ack-special/targets.json).
Offsets are method-relative bytes; catch tables use code units.
This extends [generic ACK dispatch](receipt-ack-dispatch.md), reusing the
[ACK parser](apk/2.26.37.71-message-ack/ack-wire-parser.txt) and
[0yG handler](apk/2.26.37.71-receipt-ack-dispatch/ack-handler.txt).
It adds source coverage, not a Rust state transition or live claim.

## Parser selection and specialized producers

**Confirmed:** 1Qy.A02's receipt type dispatch recognizes read, played,
played-self, read-self and server-error. It builds 1gc with false direction
at 01e0..01ea after selecting/normalizing its remote chat. The read branch
at 0236..025c converts the nullable stanza participant through 0DK.A00 and
calls 1Dz.A11. Played and played-self call A16; server-error calls A17.
Read-self takes the shown readreceipts attribute/227 path rather than this
A11 branch. These paths subsequently reach generic A1E forwarding; specialized
processing is not a substitute for the full-key callback removal.
The [read-setting continuation](read-setting-sync.md#read-self-ack-wire-producer-continuation)
now connects read-self/227 to the preference consumer; full subtype conversion,
broadcast routing and effective dispatch remain unknown. A type named server-error is distinct from the outer
numeric error attribute and is not a guessed registration refusal.

**Confirmed:** 1Dz.A11 calls A0I with participant and key. A0I builds a Bundle
with msgid from key.A01, remote_chat_jid from key.A00 through 0DE.A09 and
participant from the supplied DeviceJid through that same string helper.
It creates Android Message with **what=0, arg1=86 (0x56), arg2=0** and the
Bundle as obj. The constructor argument order is retained; 86 is not what.
Null string conversion semantics remain unknown.

1Dz.A16/A17 each build a one-key array and DDH with null device, null original
user, null stanza key, null metadata, the key's chat, status argument **10**
(played/played-self) or **12** (server-error), and long time argument **0**.
DDH.<init> stores array A07, chat A03, device A02, integer A00, long A01,
stanza A05, metadata A06 and original user A04. A0N packages it as Android
Message **what=0, arg1=262 (0x106), arg2=0**. The zero long is not an observed
peer timestamp; no receipt_user column or row insertion is demonstrated here.
These argument values must not be equated with 5/8/13 live user-timestamp slots.

## Dispatch reads arg1, not what

**Confirmed:** 1UB.A00 returns Message.arg1. 1Dy.CDr takes that selector and
asks 0y8.A0F. True clones the Message and calls A06; false sets what=0 and
posts to its own handler. A06 sets what=1, restores arg1 to the selector and
posts to 0yG. The previously retained 0yG case 1 ultimately invokes 0y8.A00;
full optional instrumentation and exception behavior are not reinterpreted here.

A0F's A0O-enabled branch admits both 86 and 262 explicitly. Otherwise, or for
non-hardcoded selectors, its fallback requires SparseIntArray A02 to resolve a
nonnegative index below handler list A0F.size(). Effective A0O and registration
contents remain unknown. A positive selector is not unconditional admission.

0y8.A00's A0O-enabled branch resolves Mpi.A00 N12 and selects **N12.A0d for
86**, **N12.A0Z for 262**. It resolves the provider, casts to 0jT and calls
BFC(Message,selector). The fallback uses the registered index/list, asserts
index bounds, invokes BFC and throws if the selected handler returns false.
Missing enabled handler selection also throws. Full N12 provider construction,
concrete BFC application and durable message-state effects remain unknown.
This now identifies the next concrete consumers to trace rather than inventing
an update from producer argument names.

In the false CDr branch, 1Dy.handleMessage case 0 calls 0iS.A0r then A0p.
A0p checks selectors 4 and 87 specially; other values, including 86/262, are
cloned and forwarded through 0fY.CDs. Full 0fY implementation and registration
remain unknown. Neither dispatch route proves handler completion or SQL commit.

## Consequences for the Rust port

**Confirmed scoped negative:** these selected producer/dispatch methods do not
insert a receipt_user row, adopt a peer key, decrypt a captured message or issue
a retry. That does not establish absence of downstream effects. Actual N12
A0d/A0Z consumers and fallback 0fY.CDs need tracing before adding message-state
updates. Generic receipt ACK completion remains separate from specialized state
messages and from peer delivery/read/played receipts.

No Rust runtime behavior, private account, key, ID, reservation or wait changed.
No live request ran. Existing durable ACK presence guard remains in place;
callbacks or status arguments do not authorize replay or fabricate user-row
membership needed by primary identity cleanup. Full identity recovery and
receive/messaging parity remain unfinished. Existing offline research tools
expose reproduction; no HTTP route, CLI command/flag or live capability is added.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-ack-special.json \
  --output docs/protocol/apk/2.26.37.71-receipt-ack-special --check
```

Validation: fourteen selected topics/methods, nineteen files / 81,579 bytes
reproduced exactly from the pinned APK. Cached self-diff verified fourteen
unchanged topics and four indexed artifacts. CLI reference regeneration, API
library-symbol verification, formatting and whitespace checks passed. Repository
hygiene checked 1,726 local links and 63 checksum bundles / 12,666,281 bytes.
No fresh runtime or live result is claimed; the prior functional ACK guard
passed 812 workspace/all-feature tests.

## Current auth-challenge diversion precedes receipt settlement (2026-11-06)

**Confirmed static, APK2.26.43.72 / Rust defect fixed offline:**
[1rH.A03 and its shared challenge route](apk/2.26.43.72-ack-auth-challenge/README.md)
parse numeric error459 and return through1lZ.A1F before ACK class dispatch.
Earlier local correction covered message/distribution ACKs, but
receipts::record_ack still marked a matching error-present receipt as refused.
The new regression reproduced that incorrect transition on the old code.

Receipt ACK settlement now uses the existing Java-decimal/i32 challenge
classifier before ordinary mutation. Numeric459 returns false without marking
any reserved receipt refused or acknowledged, including all members correlated
by a batched root. A redacted warning identifies the unimplemented branch. The
normal actor already preserves the inbound Node in its encrypted inbox before
calling this helper; the public helper itself does not claim to perform capture.

Regression `challenged_receipt_acks_preserve_single_and_batched_reservations_across_reopen`
covers direct/group, single/batched read intents and plain/plus/zero-padded/
full-width/Arabic decimal459. Checkpoint bytes and journal head stay unchanged;
reservations remain outcome_unknown and are not selectable again after reopen.
Later ordinary success and403 refusal controls settle the same correlated
reservations once, proving the exclusion did not merely break matching.

CLI direct/group send source already calls shared message_ack_outcome and needs
no separate459 change. Specialized legacy read/played/server-error message
consumers described above remain a different, partial port. **Open:** native
challenge producer/map ownership/recovery, complete sender admission, malformed/
out-of-range numeric ACK error admission and native exception handling. This
exclusion does not implement or bypass a challenge, create another request,
retry uncertain traffic, clear history or claim restriction prevention. No new
APK trace or live account operation.

Validation:1,454 workspace/all-feature tests passed. Strict workspace/all-target/
all-feature Clippy, formatting, CLI reference regeneration, OpenAPI symbol
validation, APK coverage freshness and repository hygiene passed. Offline only.

## Malformed numeric ACK admission follow-up (2026-11-06)

Receipt settlement now also excludes malformed/out-of-i32 root error values
using shared current-build ACK admission. These cannot become an ordinary
terminal refusal; snapshots and uncertain reservations remain held. The existing
459 direct/group/single/batched test now includes malformed cases.
[Parser, selected reader catch path, cross-family tests and limits](message-ack.md#integral-error-admission-before-ordinary-ack-outcomes-2026-11-06).
Native error-handler/recovery and typed error-code projection remain separate.

## Current generic callback key and separate ID maps (2026-11-06)

**Confirmed selected pinned static / negative finding:** [new43.72 generic
bridge, queue and Eoh.equals exports](apk/2.26.43.72-receipt-callback-key/README.md)
connect1rH's generic route through1lZ.A1D/1lY.Big/0ln.A09 to handler what6.
The re-exported complete0m2.handleMessage matches the retained file exactly.
Its0Th.A04 callback lookup removes a full key including nullable participant,
type, recipient and edit alongside class/from/id; extra list/node/string
members are also compared, with full producer meaning still untraced. The
edited-message fallback requires class=message and does not extend to receipts.
A found callback receives BsT(node) even when an error is present. This method
name alone cannot establish business success or convert a refusal to acceptance.

**Important limit:** the same handler subsequently removes timeout and0TV
entries by id independently of full-key callback lookup. A returned Eo7 can
invoke another callback with the id. Full-key mismatch is therefore not proof
that every native record is preserved. Those independent map producers and
receipt-family membership remain untraced. Do not remove strict Rust receipt
correlation based only on their existence, or claim universal native full-key
ownership from0Th alone.

The current Rust receipt adapter already checks class, original/batch id,
logical address, exact type and nullable participant, refusing unsupported
recipient/edit shapes. This bounded audit establishes no confirmed local
mismatch; it retains these rules without claiming complete native map parity.
Callback delivery, local diagnostic refused/server_acknowledged status and
specialized receipt state remain separate. No code change or live account
traffic. APK hash, exact handler re-export, bundle checksums, links and coverage
freshness were verified; Rust tests were not rerun for this evidence-only chunk.

## Bound the independent ID-map producers before changing receipts (2026-11-06)

**Confirmed selected pinned static / negative finding:** [new direct producer/
field censuses, queue insertion and reconnect methods](apk/2.26.43.72-id-ack-owners/README.md)
connect0TV.A05 to0Sf.A04's explicit tracking flag, original Message/callback
and count. A0L enables tracking; A0M disables it. The bounded direct callers
identify VoIP EV7 offers/accepts as a tracked source. Other shown direct A04
producers pass false, while0YB/A08 drain or resubmit retained records. No
ordinary receipt producer was established by this census; runtime/reflection/
aliasing completeness is not claimed.

Native reconnect selects active and challenged ID records and resubmits their
Messages through the tracked wrapper. Its selector includes VoIP tag/config/
attempt policy. This is retained as a separate unported mechanism, not a
reason to replay Rust uncertain receipts or sends. Original full-key callback,
ID-queue completion and specialized receipt state remain distinct. Strict
Rust receipt correlation is unchanged; no new request, challenge recovery,
retry, or runtime behavior. APK hash, evidence checksums, links and coverage
freshness were checked. No Rust tests were rerun for this evidence-only audit.

## Local reservation account authority on settlement (2026-11-06)

**Confirmed Rust correction:** new receipt reservations persist an encrypted
local account-authority digest and record_ack compares it before changing a
matching uncertain row. Account/Signal mutation or missing/corrupt legacy
binding leaves that row uncertain; no received ACK is used to invent missing
ownership. This is separate from native full-key/ID-map semantics above, and
is not proof of a socket write or complete callback parity. [Durability,
compatibility and reopen regression](replay-safety-audit.md#persist-receipt-reservation-authority-through-ack-settlement-2026-11-06).

## Durable original batch callback ownership (2026-11-06)

**Confirmed Rust correction:** new receipt reservations retain encrypted original
member snapshots behind callback_owner. ACK settlement compares every member,
settles only the original keys and consumes that owner atomically. Missing or
replaced members and unbound history stay uncertain. The original partial-write/
reconnect late-ACK behavior is preserved; no receipt is resent. Native full-key
and ID-only callback maps remain separate evidence, not a new Rust recovery
implementation. [Durable ownership, regression and compatibility](replay-safety-audit.md#durable-original-receipt-callback-members-2026-11-06).

## Current receipt registration and future completion (2026-11-06)

[Current pinned static bundle](apk/2.26.43.72-receipt-registration-completion/README.md)
connects played A0L to0Sf.A0B, registration through0Th.A01 before A05 dispatch,
and discriminator0 callback completion through2Gc.BsT -> FfK.BsT/get.
The future completes with null; the job waits on its no-argument get. Current
read/retry/media-error receipt jobs also appear in the direct wrapper caller
census, but their complete policy is not newly traced here.

Callback completion remains distinct from server acceptance and specialized
receipt state. No Rust behavior change is warranted; acknowledged/refused
outcomes, durable ownership and no uncertain replay remain intact. Failure
producers, all dispatch failure paths and native retry policy remain unknown.
This closes the selected current callback path, not older N12 state consumers.
No runtime/account operation or new capability. Bundle hash/checksums and
repository documentation gates verified; no new Rust test result claimed.

## Receipt dispatch drop and failure boundaries (2026-11-06)

**Confirmed selected static, current pinned APK:**
[DEX exports, REA cross-check and direct censuses](apk/2.26.43.72-receipt-dispatch-failure/README.md)
close0Sf.A05's initial absent-channel return/throw, selected played38 preparation,
and later channel reread. The initial drop branch has no explicit completion
of the callback registered before dispatch. No observed runtime race is claimed.
The FfK exception field has one direct writer; its failure method releases the
latch for ExecutionException. Existing size/disconnect exports remain linked,
with normalizedA0T failure treated separately from exact-key0ln.A0B removal.

No confirmed Rust mismatch: local dispatch return, callback completion, socket
write and server acceptance are distinct. Keep durable original ownership and
uncertainty on errors; no retry/reset follows from native callback cleanup.
A05 provider side effects, all failure producers, native job retry and full
shutdown/generation ordering remain open. No runtime code or account operation.
Fresh source exports reproduce byte-for-byte; REA retains complete source and
limitations, with coverage/docs/hygiene checks. No new Rust tests claimed.

## Played job retry eligibility and native parity gap (2026-11-06)

**Confirmed selected static:** [constructor, parameter binding, retry hook,
shared worker comparison and complete REA Evidence](apk/2.26.43.72-played-job-retry/README.md)
connect current played job to persistent per-chat grouping,100 attempt budget
and ChatConnectionRequirement. Its A0N returns true after diagnostic formatting;
shared Qlq worker bypasses it for RuntimeException, checks requirements after
eligible exceptions and either loops within the budget or defers the job.
Fresh shared worker/builder/requirement exports match retained evidence.

This closes hook/budget/requirement eligibility; it does not establish100 wire
sends or replay after server refusal. Native exception retry exists and remains
unported in Rust: uncertain reservations retain their original wire/owner and
are not reselected. This is an explicit lifecycle parity gap, not a claim of
complete APK equivalence. Full source eligibility, played-self policy, callback
replacement, restart deduplication and safe partial-write recovery remain open
before any reconstruction of retry authority. No code or live account change.
Selected exports reproduce exactly; subject/checksums, coverage, OpenAPI/CLI
reference and repository hygiene verified. No new Rust runtime test claimed.
