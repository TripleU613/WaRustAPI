# PSA notification work and acknowledgement

2026-11-06. **Confirmed static**, WhatsApp2.26.43.72 build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Retained handler, constructor, refusal, deferral and message worker](apk/2.26.43.72-psa-ack-lifecycle/README.md).
No live request or ACK-policy change. This is an open correctness obligation.

## Current branch matrix

19n is registered for psa handler104. A09 first reads from as a string and
compares it with literal0@s.whatsapp.net. It selects named direct-child lists,
not merely the first child, with the following precedence:

| Sender/path | Selected work | Native ACK owner/effect |
| --- | --- | --- |
| Exact0 sender, any surfaces | Boolean property3536 (0xdd0) | A07(true) before selection; false ->487; true ->IBq selector20 work |
| Exact0 sender, no surfaces, any psa_wakeup | Boolean property4200 (0x1068) | A07(true); false ->487; true ->Fc2 selector27 work |
| Exact0 sender, neither above, reset_smb_last_qp_prefetch_timestamp | Remove saved smb_last_qp_prefetch_using_graphql_timestamp | No A07 on this branch; normal return reaches ordinary base completion |
| Exact0 sender, other content | A07(true), property1844 (0x734) | False ->487; true with no first child ->487; otherwise Feq30 for revoke list, Feq31 for other content |
| Other/missing/non-exact sender, campaign with revoke list | Read optional campaign/id | A07(true); missing campaign ID ->487; present ->FeS selector34 |
| Other/missing/non-exact sender, other content | Select campaign message children or root message list, duration and campaign ID | A07(true), Fd3 work; normal exit completes plain ACK |

The exact string predicate, property provider and precedence are confirmed.
Effective account property values, MobileConfig precedence and enabled workers'
complete success/error policy are unknown. Rust's lack of implementation does
not prove those native Boolean properties are false. Neither a property ID nor
an empty child list authorizes a guessed feature default or mandatory request.

## Deferral matters

0v6.A01 installs an in-progress token and calls A09. A07(true) transfers that
token out of the atomic slot into deferred ownership, so the normal base tail
cannot send another ordinary ACK for the same token. 89Y.A00 consumes the local
completion owner before dispatch, removes deferred membership and passes optional
attributes/body into the base ACK path. 19n.A03 uses it to attach error=487.
89Y.A01 is the plain completion. This is local ownership, not durable exactly-once
wire delivery or replay authority after an uncertain write.

Fd3 processes selected message bodies and invokes native ingestion before plain
completion at its normal tail. Missing id explicitly throws; no catch/finally in
this method guarantees completion on that path. Full ingestion transaction,
restart/cut handling, scheduler acceptance, the6000 deferral task and other
workers remain separate obligations. Do not equate normal async scheduling or
a returned ingestion call with committed state or server acceptance.

## Rust gap and next implementation evidence

notifications::psa_ack currently preserves the older37.71-derived policy:487
for zero-sender surfaces/psa_wakeup, plain ACK otherwise after capture. It does
not implement the current property1844/revoke/campaign work, reset preference,
exact string-source selection or deferred state owner. The old universal
all-PSA/base-ACK claim is therefore too broad. Historical evidence of a server
ending a session when an ACK was absent does not establish all these current
branches or justify ACKing unprocessed work as if completed.

This trace changes no runtime policy: blindly broadening487, holding every PSA,
or inventing feature values would each choose an unproved native branch. Next
required evidence/port work is effective feature authority, selected worker
success/failure completion, transactional work/recovery ownership and the ACK
writer's captured/live correlation. Existing capture and ACK helpers remain
bounded legacy behavior, not a complete current-build port or zero-ban proof.
