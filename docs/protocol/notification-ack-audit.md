# Notification ACK correctness audit — 2026-11-04

This audit concerns protocol obligations and avoidable receive failures. No live
session, account operation, fabricated provenance or server-policy bypass ran.
It cannot establish why historical accounts were restricted or guarantee that
an unofficial client will never be restricted.

## Confirmed defect and bounded correction

**Confirmed locally:** `notifications::handle` stored supported trusted-contact
tokens but returned no ACK when the same `privacy_token` notification contained
an unknown token kind, an extra child or an empty token list. This extra
post-processing suppression was separate from token validation. The new encrypted-
store regression failed on the original handler at the missing ACK assertion.

**Confirmed static, current APK 2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`:
`EL2.<clinit>` maps `privacy_token` to 234; `10G` registers 234, extends `0v6`,
and its complete `A09` loops only over `token` children and skips token types
other than `trusted_contact`. It does not defer the ACK. `0v6.A01` dispatches
then invokes the base ACK unless a deferred completion exists. Thus ignored
subtypes do not suppress the ACK of this known outer notification type.
[Exact retained evidence and reproduction](rea/2.26.43.72-notification-ack-audit/README.md).

**Implemented/offline:** the unsupported-child post-check was removed. Existing
sender/PN-LID/device, ID, participant/recipient, timestamps, token sizes/list bounds
and store checks remain. Supported token updates commit before ACK construction
returns to the receive loop. Unknown kinds create no token/identity/session state;
unknown *outer notification types* still receive no generic ACK. Duplicate valid
notifications may be ACKed again while the stored strictly-newer timestamp rule
prevents reapplication. No new network operation or public command was added.

## Current Rust ACK inventory and ordering

This table is a source audit of the existing receive path, not a claim of full
current-APK parity for every handler. Most other handler traces are retained
2.26.37.71 evidence; the privacy/base chain above was re-anchored to 2.26.43.72.

| Recognized notification | Existing ACK path | Before ACK / limits |
| --- | --- | --- |
| `privacy_token` | `notifications::handle` | Validate and atomically store supported tokens; ignored subtypes no longer suppress ACK |
| `w:gp2` / `groups_dirty` | `groups::handle_dirty_notification` | Queue refresh durably; malformed wire inputs return no ACK, not receive termination |
| Other `w:gp2` | `groups::notification_ack` | Known-type base ACK after capture; membership/change application remains partial |
| `devices` add/remove | `devices::notification_ack` | Build ACK before queueing; supported remote sync work commits before write; own/unsupported forms are limited |
| `encrypt` identity | `identity_notifications::notification_ack` | Queue-or-skip decision commits before ACK; later key reading/adoption is separate |
| `encrypt` prekey counts | `prekeys::low_count_ack` | Pure parse/ACK checks precede work; wire refusals continue without upload or ACK |
| `mex` | `notifications::mex_ack` | Capture; recognized reach-out lock persists first; other operations remain unapplied |
| `business`, `disappearing_mode` | respective known-type base ACK helpers | Capture retained; feature application remains partial |
| `psa` | `notifications::psa_ack` | Plain base ACK, with 487 only for the traced disabled server PSA children |

The known-type base shape is `ack` with `id`, `to`, `class=notification`,
`type`, and optional `participant`. Retained 37.71 `CmG` notification-descriptor
construction excludes receipt-only `recipient/category/edit` metadata, so the
shared receipt ACK writer's optional fields are not automatically required here.
[Descriptor and wire evidence](apk/2.26.37.71-identity-notification-ack/targets.json).
The privacy path remains narrower and refuses participant/recipient forms.

## Negative findings and open obligations

- The current classic type map has 38 names; the receive code above recognizes
  eight outer types and specific subforms. Enum/map membership alone does not
  establish that another type must be generically ACKed. `contacts`, `account_sync`,
  `server_sync`, `location`, `mediaretry`, `authkey_rotation`, `feature_control_notify`
  and remaining names require handler/deferral/response traces before changes.
- Malformed privacy-token wire content is caught by pure `validate` in
  `connection::run`: capture remains durable, no ACK is sent, and receive
  continues. Stored token corruption/storage failure after validation remains a
  visible error, not silently mistaken for an unsupported network message.
- The suspected bare `groups_dirty` error propagation was inspected and refuted
  for its wire-input path: that helper catches its preparation `InvalidInput`
  internally and returns no ACK. Store faults still propagate intentionally.
- Native `0v6.A01` also catches its corrupt-stanza exception before its normal
  ACK continuation. This client deliberately retains stricter local wire bounds;
  this change does not port every native malformed-stanza ACK behavior.
- Native deferrals and full state application are not reproduced merely by
  returning a matching ACK. This audit establishes one known-type ACK omission;
  it is not a blanket claim that all ACK obligations or account-risk causes are
  resolved. No causal account-ban claim follows from this offline correction.

## Verification

All **nine notifications tests** pass after the correction. The added regression
checks mixed/unknown-only/empty token lists, extra children, exact ACK identifiers,
no state from ignored kinds, malformed-supported-entry atomic refusal, invalid
metadata and unknown outer-type refusal, and stored-token survival after reopening.
Existing concrete-peer and Java timestamp regressions remain passing. No qualifying
server/handset ACK validation was attempted.

### FBNS provider message ACK continuation (2026-11-05)

**Confirmed static, APK43.72:** the selected FBNS worker reaches provider ACK
after normal message handling, empty data or its JSONException catch. Nonempty
sender/notification ID and sender allowlist gate an authenticated Android
broadcast with processor_completed=true. This does not prove payload success,
durable processing or WhatsApp stanza acknowledgement.
[Branch offsets, error continuation and limits](apk/2.26.43.72-push-config-fbns-token-received/README.md#provider-message-ack-after-normal-handling-or-json-failure).
Full downstream processing, provider delivery and durable ACK ownership remain
open. No Rust behavior, provider IPC or live account operation changed.


### Current notification type mapping and picture gap (2026-11-05)

**Confirmed static, APK2.26.43.72:** EL2 maps w:gp2 to209 and picture to189;
1rN.A05 reads IncomingStanza type and passes that code as Message.arg1 via
2Ai.A00. Marker admission remains feature/group/child dependent; picture uses
set/delete. Rust has w:gp2 ACK handling but no picture consumer/ACK or automatic
missing-identification lifecycle. Native deferral and state ordering still need
reconciliation before porting. [Retained map, consumer and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Picture notification base ACK (2026-11-05)

**Confirmed static / locally implemented:** current19k picture handler has only
constructor/A09 and no base A07 deferral calls;8aj selector1 returns processing
to0v6.A01. Rust now emits the shared base ACK for picture after durable capture.
Malformed envelopes log/no-ACK and leave receive running; redelivery is ACKed
again. Photo updates and missing-ID lifecycle remain unported, with no automatic
queries or live validation. Actor regression verifies capture-before-write, exact
participant routing, redelivery, ping after refusal and reopen persistence.
[Source and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Picture ACK wire-field provenance (2026-11-05)

**Confirmed static, APK2.26.43.72:** integer76 dispatches0v0 ->0v2.A02 ->
F3e.A05, which builds the final ACK. F0R's fresh notification key populates
from/id/participant/type/class; recipient/edit/error/offline are not copied.
The existing picture actor regression now supplies those attributes and requires
the same exact base ACK. Native photo exception/CCQ/full address parity remains
unverified. [Retained final builder, key fields and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Notification address diagnostic boundary (2026-11-05)

**Confirmed static, APK2.26.43.72:** DBe.A1A/0Tl preserve Jid objects and raw
strings for ACK attributes.0Tp.A0A logs failed isProtocolCompliant then continues
toward class conversion/return; it does not reject solely on that boolean. A
proposed blanket rejection based on this diagnostic was ruled out. Current
binary-reader zero-AD normalization and subtype writer parity remain unclosed;
no shared Rust address policy changed. [Retained source and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Current-build ACK zero-device normalization (2026-11-05)

**Confirmed static, APK2.26.43.72 / offline regression:**123.A01/A02/A04
reconstruct binary zero-device PN/LID without a device suffix;1r2.A00 chooses
pair for zero compliant getters and AD for positive device/agent. The existing
picture actor test now covers zero/nonzero PN/LID sender and participant forms.
Native domain flags are distinct from Rust's agent field. No runtime policy
change; all subtype/hosted/interop behavior and live acceptance are not claimed.
[Current source and census](apk/2.26.43.72-group-missing-identification/README.md).


### Current notification constructor census (2026-11-05)

**Confirmed static:**34 physical0v6 constructor callers are retained with their
numeric code/base-executor boolean. The type map has38 entries; these counts
are different populations and do not measure completeness. server_sync210 maps
to14l (typed collection/bootstrap work), server235 to15I (log/abprops/push-config
follow-ups). Rust has no dedicated handler/ACK for either. Full state/helper/job
and ACK ordering remain unported; no generic ACK, token fabrication or runtime
change was added. [Census and complete selected handlers](apk/2.26.43.72-notification-handler-census/README.md).


### Server-sync callback ownership reconciliation (2026-11-05)

**Confirmed static:**14l.C6N ->2JI selector6 emits count-minus-one aggregate
metrics and clears its counter map; it is not an ACK completion gate. The
existing lifecycle audit already proves dirty-version assignment and normal
ACK-before-async-result ordering. The constructor census now links that evidence
instead of presenting it as newly unknown. Rust role/recovery/bootstrap and
durable notification scheduling remain unported. No runtime change.
[Callback evidence and limits](apk/2.26.43.72-notification-handler-census/README.md).


### Group notification storage failures stop before ACK (2026-11-05)

**Confirmed local defect/fix:** connection dispatch previously logged every
error from groups::apply_change_notification and then sent the normal w:gp2
ACK. A corrupt saved GroupSnapshot therefore looked acknowledged even though
the membership change could not be committed. The actor now propagates that
helper's store/transaction errors before ACK construction. Its current wire
path filters unsupported/non-group/malformed participant inputs without returning
errors; no new wire-input rejection policy is introduced.

The actor regression corrupt_group_snapshot_is_not_acknowledged_as_applied
failed on the previous path at the emitted-ACK assertion, then passed after
the correction. It delivers a valid group/add notification over a corrupt saved
snapshot, checks no outbound success ACK, observes Error::Storage and verifies
unchanged original bytes plus durable inbound capture before and after reopen.
The capture supports later local investigation; this does not implement automatic
notification replay or state repair. Existing valid membership/ACK behavior
continues through the same functions.

This is a local storage-integrity correction. APK43.72 0v6.A01 separately
rethrows SQLiteFullException/SQLiteDatabaseCorruptException at020c..0210 in the
[retained base handler](apk/2.26.43.72-group-missing-identification/ingress-base.txt);
Rust serialization faults are not claimed to be identical native exceptions.
No live account, automatic query, identity adoption or resend occurred.


### Validate group ACK envelope before membership mutation (2026-11-05)

**Confirmed local defect/fix:** receive dispatch applied a w:gp2 membership
change before validating its id/from/participant ACK envelope. A missing-ID
remove could therefore change saved membership, then be refused without ACK.
The actor now constructs/validates the existing ACK first, applies a supported
change only for that admitted envelope, then writes the ACK after commit.
Existing limits and malformed-envelope logging remain; no new wire rule was
added. groups_dirty retains its separate preceding durable scheduling path.
Store faults still propagate before ACK, preserving the preceding correction.

The actor regression refused_group_ack_envelope_cannot_change_membership failed
on the original missing-ID case with changed saved bytes. The corrected path
checks missing ID, empty ID and unusable participant: each receives no ACK,
passes a server-ping processing barrier and leaves the snapshot byte-for-byte
unchanged. Its valid control removes the participant and verifies the committed
empty membership before ACK write completion and after reopening the store.
No automatic replay, query, identifier change or live operation occurred.

This is a local admission-order correction. Current APK F0R.A02/A00 constructs
the notification key, including id/from/participant, before the base handler
dispatches A09; the [selected retained key path](apk/2.26.43.72-group-missing-identification/notification-key-entry.txt)
and [fields](apk/2.26.43.72-group-missing-identification/notification-key-fields.txt)
provide the comparison boundary. Exact native malformed-envelope/error-ACK
behavior and this client's128-byte ID bound are not claimed identical.


### Public group-change envelope guard (2026-11-05)

**Confirmed local defect/fix:** public groups::apply_change_notification now
validates the same ACK envelope before store access; direct callers cannot
bypass the receive actor's prerequisite. Three invalid-envelope regressions
preserve exact snapshot bytes and journal head. A routing-race fixture gained
a synthetic notification ID without weakening its assertions. The helper sends
nothing. [Details and red/green validation](groups.md#public-group-change-admission-matches-receive-2026-11-05).


### First group event owns the supported change (2026-11-05)

**Confirmed static / locally fixed:** APK43.72 0Tp.A0D/0w9.A09 selects first
child as the event. Rust previously applied every outer sibling; unsupported+
remove could remove membership and add+remove applied two changes. The helper
now selects only the first event, retaining inner participant iteration and
existing ACK/envelope/store guards. Red/green regression preserves unselected
membership and standalone valid updates. Extra metadata siblings are permitted;
full native event/timestamp/alias parity is not claimed.
[Checked selection evidence](apk/2.26.43.72-group-event-selection/README.md).


### Dirty group event with metadata siblings (2026-11-05)

**Confirmed static / locally fixed:** current0w9 selects first-child groups_dirty
without requiring exactly one outer child. Rust previously skipped refresh for
metadata siblings and could generic-ACK without scheduling. Dedicated dirty
handling now selects the first event; generic ACK stays suppressed for it.
Existing durable queue/dedup/envelope/inner-list guards remain. The expanded
queue-before-ACK regression covers metadata sibling, sole ACK ownership and
reopen without another reservation. Missing-identification state remains unported.
[Evidence and limits](apk/2.26.43.72-group-event-selection/README.md).


### Dirty list filters named group entries (2026-11-05)

**Confirmed static / locally fixed:**0w9.A09 calls0Tp.A0N(group), which selects
matching children and skips others. Rust's whole-list rejection on unrelated
children suppressed valid refresh/ACK. It now filters group entries; selected
identity/duplicate/device, empty-work, envelope and256-total-child guards remain.
Red/green queue-before-ACK/reopen regression includes metadata with an unusable
jid beside a valid group. Unknown-only remains unsupported.
[Exact helper and limits](apk/2.26.43.72-group-event-selection/README.md).


### Privacy-token legacy agent qualifier (2026-11-05)

**Confirmed local defect/fix:** supported primary from/sender_lid admission
checked device0 but not agent0; a qualified address could write token state.
Those guards now require both zero, retaining PN/LID namespace handling and
existing invalid-alias policy. Native10G.A09 uses UserJid getters; Rust's LID
domain bit is not its legacy agent field. Red/green typed/text PN/LID and alias
regression preserves checkpoint/pin/session snapshots across reopen. Reach-out
restriction envelope admission was separately rechecked and already guarded.
[Complete current source and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### First named privacy-token container (2026-11-05)

**Confirmed static / locally fixed:**10G.A09 ->0Tp.A0G/A0F selects the first
named tokens container. Rust's uniqueness guard suppressed valid first data/ACK
when another container followed. Selection now matches the APK; later containers
cannot replace state or rescue a malformed first. Existing selected-list/time/
size/address/atomic guards remain. Red/green regression includes leading metadata,
later newer/malformed data and invalid-first/valid-second.
[Complete helper evidence and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### Consume token time only for supported data (2026-11-05)

**Confirmed static / locally fixed:**10G.A09 reads parent t only inside the
trusted_contact branch. Rust's eager timestamp parse suppressed ACK for empty
or ignored-only lists. Parent time is now parsed only for supported entries;
supported-token Java/range/atomic checks remain. Red/green empty/unknown-only
regression covers absent, malformed, negative and overflowing unused t without
changing prior state. [Retained branch proof and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### Ignored privacy tokens do not consume sender alias (2026-11-05)

**Confirmed static / locally fixed:**10G.A09 reads sender_lid only after selecting
trusted_contact. Rust's eager alias resolution suppressed ACK for empty/ignored
lists with invalid unused metadata. After the existing container/list checks it
now returns no updates before alias/time resolution when no supported entries
exist. Supported alias/time/primary/atomic guards remain. Red/green regression
combines ignored lists with malformed or agent-qualified unused alias and time,
preserving prior token state. [Evidence and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### Account privacy first-event admission (2026-11-05)

**Confirmed static / locally fixed:**19N.A09 selects the first event. Rust's
exact-one outer child guard suppressed supported privacy adoption/ACK with later
siblings. It now requires the first event to be supported privacy and ignores
later event siblings; unsupported/malformed first events cannot be rescued or
overridden. New red/green regression preserves prior state for those refusals.
Category/envelope/pending-intent/transaction guards remain.
[Evidence and boundary](account-privacy-notifications.md#first-event-selection-with-later-siblings-2026-11-05).


### Account privacy named-category selection (2026-11-05)

**Confirmed static / locally fixed:**19N.A09 uses0Tp.A0N(category). Rust's
whole-child parsing rejected unrelated metadata beside supported readreceipts.
It now filters named categories before validation, retaining64-total-child,
selected category/value/body, envelope/pending/transaction guards. Metadata-only
remains refused. Red/green regression ensures metadata cannot adopt preference
state and does not suppress the selected valid category.
[Source and limits](account-privacy-notifications.md#named-category-filtering-2026-11-05).


### Count ACK optional source and malformed-source refusal (2026-11-05)

**Confirmed static / locally fixed:** deferred89Y preserves original Eoh; the
common final builder omits absent to. Rust's server fallback fabricated explicit
to for absent source and redirected explicit parse failure. Count ACK now omits
to for absence, retains normal parsed routing and locally refuses explicit
invalid source before ACK/upload. Red/green actor coverage checks source-less
PQ no-upload and malformed-source capture/receive continuation; valid legacy
upload controls remain. Native ACK-after-work timing remains unported.
[Complete deferred chain, wire proof and limits](apk/2.26.43.72-prekey-count-ack-source/README.md).

### Transactional stanza-to-ACK dispatch (2026-11-05)

**Confirmed static, APK2.26.43.72:** Flr selector7 has a conditional outer
transaction that persists selected stanzas and computed pre-ACKs before
Euj.A02 send admission. A thrown outer close propagates before that send call.
The alternate branch persists stanzas without this explicit outer pre-ACK
insertion. Both operate on a filtered return list, not all captured input;
A0I is set after send admission returns, not after server confirmation.
[Five exact methods, offsets and limitations](apk/2.26.43.72-preack-transactional-dispatch/README.md).
This closes the Flr insertion-caller gap, not effective configuration,
count-specific bulk membership, crash replay or upload/socket ordering.
No Rust runtime behavior or live account operation changed.

### Malformed count content still receives its control ACK (2026-11-05)

**Confirmed static and fixed offline:** current15v.A09 throws LIL for a
malformed present PQ count before deferring metadata or queueing upload work.
0v6.A01 catches LIL and dispatches the still-present original ACK context;
SQLite full/corruption failures instead rethrow. Rust now ACKs supported count
envelopes on content refusal without upload. Invalid IDs/sources still refuse
ACK; storage and uncertain write failures still propagate. This supersedes the
earlier local no-ACK policy for malformed PQ content.
[Exact parser/catch paths, incoming-kind census, regression and remaining queue limits](apk/2.26.43.72-count-content-error-ack/README.md).
Valid-count upload/ACK ordering and full queue replay remain open. No live operations.
