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
