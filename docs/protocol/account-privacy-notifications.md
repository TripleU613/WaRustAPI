# Account privacy category notifications

2026-11-04. **Confirmed static**, WhatsApp Android **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Rust behavior below is **implemented offline**; no fresh live acceptance is
claimed. The objective still includes complete refresh/configuration/category
processing; this is the first complete supported category branch, not a
redefinition of the remaining work.

## Supported state adoption before ACK

`transport/account_privacy.rs` implements the non-modify `account_sync` privacy
branch when **every supplied category is readreceipts**. Current native
`19N.A09` first-child selection builds a map with last-name-wins replacement,
then `0oe.A0R` validates/adopts values. Its base `0v6.A01` ACKs after dispatch.
Sources are the retained
[incoming handler](rea/2.26.43.72-notification-sync-obligations/README.md) and
[category adoption/8AX validators](rea/2.26.43.72-account-privacy-refresh/README.md).

The admitted values are `all`, `contacts`, `none`, `known`, `off`,
`on_standard`. `8AX.A03` converts only `all` and `on_standard` to true;
other admitted values convert to false. Do not collapse category admission
into an assumption that only all/none exist. Repeated valid readreceipts
categories use the final value, matching the current native HashMap producer.

The receive loop captures the original node first, purely validates the
supported wire subset, then atomically commits the preference and processed-ID
marker before exposing the ACK:

```xml
<ack id="original" to="s.whatsapp.net" class="notification" type="account_sync"/>
```

Store faults propagate; only pure wire refusal is logged and left captured
without ACK while receiving continues. The durable server category is available
through `account_privacy::read_receipts`, returning None when unknown and
`ReadReceiptPreference { value }` when known. `ReadReceiptValue::enabled`
exposes the native Boolean conversion. It does **not** alter explicit local
read/read-self or automatic receipt policy. There is no served HTTP route; offline CLI inspection is described below.

Processed-ID records prevent an old redelivery from overwriting a newer
preference. A matching redelivery can receive another ACK, including after
failed ACK write/reopen. Conflicting values under the same processed ID refuse
with a fixed diagnostic. No ACK outbox or automatic replay on reconnect was
introduced. ACK-write failure leaves both original capture and adoption intact.

## Local supported boundaries

These are conservative Rust admission rules, not claims that the APK rejects
every broader form: server-only `s.whatsapp.net` source; nonempty bounded ID;
nonnegative signed-64-bit decimal timestamp; root attributes from/id/type/t
only; first outer event must be privacy (later siblings are not applied);
1..64 total children in the selected event, with at least one named category;
plain supported
category attributes and empty category bodies. Duplicate/unknown attributes,
nontext metadata, mixed/unsupported categories and malformed values refuse
before any state write. An invalid earlier duplicate is refused even if a later
duplicate would replace it natively. `action=modify` remains unported and
unacknowledged; other text actions follow the native non-modify branch.

Native `0oe.A0R` preserves a contradictory pending desired value. Rust has no
typed account-privacy setter or native pending-category map; searches of the
current transport/model sources establish that boundary, not absence of all
possible generic requests. Public `actions::queue` can still hold a privacy
category setter. The handler therefore checks decoded requests and outcomes in
the same write transaction before adoption. Any queued/sent/successful
`xmlns=privacy,type=set` action with a privacy child blocks fresh adoption/ACK;
a successful generic setter remains unowned until proper reconciliation exists.
Only preserved outer IQ refusals discharge this guard; old misclassified unknown-type errors still block. This is a conservative local guard, not the exact
native pending-map lifecycle. Existing processed redeliveries do not reapply
state and may re-ACK while such an action is pending. Trusted-contact token
issue uses a tokens child and is not mistaken for a category setter.

Remaining privacy categories, siblings, pending setter reconciliation,
refresh scheduling and account-specific gates remain unported. No unsupported
outer `account_sync` envelope receives a generic ACK. No server version,
effective setting or absent property was fabricated.

## Current transport continuation

[New evidence](rea/2.26.43.72-privacy-transport/README.md) closes several
unknowns from [the preceding MEX/config trace](privacy-mex-config.md):

- `00D.A02(66)` -> `A03` -> `1Uw.A00`: high selector `66 >> 14` is zero,
  selecting `1Uw.A01`. Its low-14-bit case66 target09a8 constructs **08Y**.
  Thus this property concretely belongs to the ab-props owner with previously
  traced SDK migration/readiness and compiled legacy default0. It is still
  incorrect to replace hydrated SDK state with that compiled default.
- `0ig.<init>` registers `WHATSAPP_ANDROID_MEX` through `0ik.A01` and
  `1Wy` discriminator22 -> `0ii.A00` -> `0im`. `0ij.<clinit>` supplies exact
  namespace `whatsapp-android-mex` and filename
  `whatsapp-android-mex_client_persist_ids.json`; `0im` loads that asset lazily.
  `CRh(operation)` reads its `doc_id` via JSONObject.getString. This closes
  namespace/asset linkage for the selected GetPrivacySettingsQuery record.
- Privacy's `0iT` has a null secondary model. `0io.A02` tests this before
  Pando allowlist/configuration: it selects `0mO.A0B`,
  REJECTED_MEX_NOT_PANDO_CAPABLE, with `isPando=false`. The normal selected
  `0iq` has defaultfalse alternate transport flag and constructs `0j3`.
  This privacy request therefore takes the ordinary XMPP path; there is no
  need to guess a Pando property for it.
- `0j2.A03` resolves document ID via `0ih` and the registered `0ik` provider;
  `A04` refuses a missing ID before dispatch. `0mF` constructs JSON
  `{queryId: docId, variables: queryParams}`. `0mK` constructs server-only
  IQ/get `xmlns=w:mex`, generated ID, trace/flow_id equal to document ID and
  query/query_id equal to that same ID. `0j3.A06` submits request discriminator
  **341**, timeout literal **32000**. Units remain inferred here.
- `0mb.CK4` delegates to `0md.CK4/A00`, trying success, request-error then
  server-error structural models. The correct `7Ou` success constructor is
  overload1 `(0Tp,0mK)V`; overload0 is unrelated. Success requires IQ result,
  correlated request ID/source and exactly one result child, admitting Argo/JSON.
  `0mZ.BKn` -> `7jy.A00` -> `0is.A03` converts the selected typed payload into
  the `74K` model before product callback. JSON uses the ordinary path when
  no custom converter is supplied; Argo resolves schema by namespace/operation.

**Unknown/unported:** current typed Argo/schema and JSON model coercion,
complete response-error policy, durable hydration of migration/readiness and
property23463, optional STATUSVIEW gates, all-category storage/observers and
refresh retry/dirty completion. Existing generic MEX JSON inspection is not a
correlated privacy refresh consumer. Those dependencies still prevent a faithful
modify-refresh implementation, even though its selected transport is now known.

This remains static/offline work; no account-restriction causation, live success
or guarantee against restrictions is inferred.

Offline inspection: `whatsapp-cli account-privacy-inspect` reads the stored
server category and its Boolean conversion from an existing store. Missing stays
null. It creates no store, refresh, ACK or read receipt and changes no policy.
For historical generic setter rows, only an actual outer IQ error discharges the
conservative pending guard; an old misclassified unknown-type Error stays blocked.

An additional regression keeps legacy unknown-type errors from discharging
the setter guard; only a real outer IQ error permits this local guard to clear.


## First-event selection with later siblings (2026-11-05)

**Confirmed static / locally fixed:** current19N.A09 selects0Tp.A0D at
0038..0040, then dispatches that event. The [retained complete checked handler](rea/2.26.43.72-notification-sync-obligations/account-sync-dex.txt)
and [first-child primitive](apk/2.26.43.72-group-event-selection/first-child-helper.txt)
cover the selection; current APK was rechecked through xtask apk-smali --class
LX/19N; --method A09 --signatures --exceptions. This is existing evidence
corroborated, not a new REA record or a port of all account-sync events.

Rust's exactly-one outer child guard refused supported first privacy events
when later privacy/devices siblings followed. It now selects the first child
and requires that event to be privacy; it does not search later siblings, merge
categories across events, or apply later device changes. Root attribute/source/
ID/time checks, selected privacy action/category bounds/values, pending-intent
protection and transactional adoption/processed-ID markers remain unchanged.
Unsupported first events and malformed selected categories remain unacknowledged.
This supersedes earlier exactly-one/unknown-sibling descriptions only for outer
event selection; mixed unsupported categories within selected privacy remain
unported and refused.

The new first_account_sync_event_owns_supported_privacy_adoption test fails on
the original child-count guard. It verifies first readreceipts=all plus later
privacy=none/devices adopts All and returns ACK; unsupported first and invalid
first category plus later valid privacy preserve the complete prior checkpoint
snapshot. The existing negative sibling fixture now puts devices first to keep
testing unsupported-first admission. Existing redelivery/reopen/pending-intent
and actor capture/adoption/ACK-order regressions remain. No new category, device
operation, query, identity policy or live account activity was introduced.


## Named category filtering (2026-11-05)

**Confirmed static / locally fixed:**19N.A09 uses0Tp.A0N(category) at
09d6..09e2 and09fe..0a06, then builds its category map from that selection.
The [complete handler](rea/2.26.43.72-notification-sync-obligations/account-sync-dex.txt)
and [exact named-child filter](apk/2.26.43.72-group-event-selection/named-child-filter.txt)
establish that non-category metadata is skipped by name, not parsed as a value.
Rust previously rejected any non-category child alongside valid readreceipts.
It now filters before validating category attributes/content/name/value. A
non-category with readreceipts-like attributes cannot change preference state.

The existing64-total-child local bound remains, including metadata. All selected
categories still must be supported readreceipts with valid values and empty body;
unknown actual categories do not gain an ACK. Metadata-only/empty supported work
returns the existing category-count refusal, without reaching state adoption or
panicking. Root/action/envelope, pending setter and transactional processed-ID
guards remain unchanged. Native empty-map ACK behavior is not ported here.

The expanded first-event test fails on the old unsupported-category guard. It
now prepends metadata with invalid readreceipts-like fields and opaque text to
a valid selected category; All is adopted/ACKed while later event siblings remain
unapplied. Metadata-only first privacy is refused with exact checkpoints
unchanged, alongside existing invalid/unsupported first-event controls. Existing
mixed selected-category/refusal/reopen/pending tests remain. No new category,
refresh/device operation, query, identity policy or live account was introduced.
