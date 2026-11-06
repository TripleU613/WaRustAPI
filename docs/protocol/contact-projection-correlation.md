# Single-contact reply projection and original-request correlation

2026-11-05. **Confirmed local source defect**, parent42750790; corrected offline.
This is a Rust single-request provenance invariant, not a claim that the APK's
multi-user parser filters every reply the same way. The current native
[contact result/backoff trace](apk/2.26.43.72-contact-protocol-backoff/README.md)
retains the distinction between protocol metadata and per-user types. Its full
policy remains unported; no live account operation was performed here.

## Defect and reproduced consequence

contacts::queue_check constructs one original phone target and saves its
correlation with the request atomically. The old successful-reply ingestion
instead iterated every usync/list/user and wrote each returned phone's status.
It did not bind projection to the saved original target. A same-ID settled
reply containing another number could overwrite that unrelated number's status.

The new regression first establishes a positive saved status for synthetic B,
then queues a check for A and returns only B/out. Before the correction,
ingest returned1 rather than0 and allowed B's status to be overwritten. After
the correction, both B's original status and A's absence of a result survive.
No claim is made that a real server sent this mismatched result or that this
local defect caused an account restriction.

## Implemented projection rule

In the same write transaction, ingestion reloads the saved request and validates
its IQ identity, single-contact body and original correlation using the existing
reservation guard. A successful outer reply must have one unambiguous usync
and list container. Only one parseable user matching the requested normalized
phone is eligible. Other numbers never receive status writes from this action.
Two matching users are ambiguous, including duplicate identical rows; there is
no arrival-order winner. Missing/malformed/unmatched results preserve contact
status data rather than manufacture an out result.

The selected per-user type/LID parser keeps its existing behavior. Applied means
that its status model was projected, not that the type is necessarily in or that
all protocol metadata succeeded. Missing/unsupported contact types retain their
fixed status error. A correlated outer IQ error remains a refusal attributed to
the original requested number, with contact_type=null; it is not account absence.

Missing historical correlation produces no status mutation and a fixed diagnostic.
Invalid/mismatched stored provenance is a local store fault that rolls back;
it is not silently repaired from the reply or current address book. Already
consumed historical markers are not deleted or replayed, and this is not a
migration of previously misprojected contact statuses.

## Durable inspection and ordering

ActionOutcome.contact_projection is optional and exposed by existing actions
plain/JSON inspection. Its values are applied, server_refused,
missing_request_correlation, malformed_reply, no_matching_user and ambiguous_user.
They contain no raw phone, contact name, response body or timestamp. Legacy,
unrelated and not-yet-projected actions omit the field; absence does not prove
projection succeeded. ActionStatus continues to describe the outer IQ outcome.
A rejected projection can therefore coexist with an Ok outer result honestly.

Contact status data (when applicable), the action's projection diagnostic and
contacts-ingested marker commit atomically. This does not change sent request
bytes, action ID, settlement sequence, reply or callback ownership. A processed
local rejection gets its once-only marker/diagnostic without another query.
If the commit did not land, startup can project the retained settled reply once;
uncertain-fsync recovery accepts either complete state or no projection state.
Sent/settled requests are never returned to Queued or automatically replayed.

## Verification and limits

The unrelated-target regression failed against the parent and passed after the
correction. Tests also cover a matching row plus unrelated rows, duplicate
matching rows, malformed/missing reply structure, missing original correlation,
positive projection, outer-refusal diagnostics, frozen request bytes, disk reopen
and idempotent repeated ingestion. Real full-disk/failed-fsync tests verify that
status, diagnostic and marker are all present or all absent, while the action
remains settled. Local recovery consumes only the saved reply, with no socket
write or fresh contact request.

Run cargo test --locked -p whatsapp-api --all-features contact_projection and
single_contact_reply_never. The broader shared Rust checks cover the normal
receive/startup projection paths. No new command or HTTP route is introduced.

Native factory selection, usync request-specific timeout, protocol integrity/
backoff adoption, per-scope deadline checks and complete partial-result policy
remain unfinished. This local correction neither invents nor bypasses those waits.

## Transaction-bound settlement projection (2026-11-06)

**Confirmed local source gap:** ingest enumerated settled outcomes/replies before opening each write transaction. It reloaded only the request inside that transaction. A saved row changed between enumeration and projection could therefore contribute current request authority but stale reply/status/diagnostics. The earlier atomic status/marker commit did not make those reads atomic.

Enumeration now supplies candidate ID and settlement ordering only. A private ingest_in operation reads current action, request and reply through actions::view_in_write under the same write transaction that stores projection/status/marker. It requires contact-check kind, current settled status and the same settlement sequence as the enumeration. Changed kind/status/order refuses without writes. It also requires a present IQ reply with the original ID and result/error outer type; current Ok requires result. Current Error can retain an IQ result because a scoped contact-protocol refusal may settle an outer result as Error. No generic result-to-success reinterpretation is added.

The existing request/correlation, unique target/container and consumed-marker rules remain. A consumed historical marker remains once-only; it is not deleted to force reprocessing. Fixed local errors identify missing/uncorrelated replies or settlement changes without echoing phone/ID. Existing native parser/backoff evidence remains separate: these are local saved-state integrity requirements, not new server fields or proof of final native lookup parity.

Regression contact_projection_uses_current_transaction_state_after_enumeration freezes an enumeration, then changes saved state before invoking the production transaction operation. A valid replacement out reply is projected instead of the enumerated in reply. Six negative controls change kind, status, settlement sequence, reply ID, reply type or remove the reply; each refuses with unchanged complete checkpoints before and after reopen. The positive control remains consumed after reopen. Existing ordinary result, outer-error and result-with-protocol-refusal tests cover the supported settlement distinctions. No live account operations, deadline changes, replay or extra requests.

## Duplicate field authority is not selected by order (2026-11-06)

**Confirmed local ambiguity correction:** Node::child selects the first matching child. Contact projection therefore previously let a first successful result/lid metadata node hide a conflicting sibling, or select the first per-user LID/contact field. Unique usync/list checks did not cover those nested fields.

A local unique_child guard now requires exactly one result metadata container and one protocol lid before granting LID eligibility, and exactly one per-user lid before storing its value. Ambiguous LID metadata/value drops only LID while preserving otherwise valid contact status. Duplicate per-user contact fields make that user ineligible rather than choosing in/out by order. The existing no-matching-user projection preserves prior contact state. The same parser is used by offline LID repair, which cannot recover a LID from these ambiguous records.

Regression duplicate_contact_fields_never_select_lid_authority_by_child_order exercises result, protocol-lid, user-lid and contact duplicates in both orders. It checks contact preservation versus withheld projection, no LID adoption, repair refusal, once-only ingestion and unchanged checkpoints after reopen. Existing single-field success, typed AD_JID-zero LID, malformed refresh and protocol refusal controls remain.

**Native boundary:** retained APK43.72 [0pD.CK4/A01](apk/2.26.43.72-contact-protocol-backoff/README.md) separates protocol success from user data; the older37.71 LID parsing evidence remains in chat-actions.md. These traces do not establish duplicate-node rejection by Android. This correction is a local ambiguity policy, not an asserted APK mismatch or a new server requirement. Full native query profile, protocol backoff and mapping authority remain incomplete. No network operation, replay, timeout or identity/pin/session mutation.
