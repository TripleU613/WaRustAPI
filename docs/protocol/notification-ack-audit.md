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
