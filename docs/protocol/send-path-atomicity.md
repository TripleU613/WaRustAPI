# Atomic send-path adoption

**Confirmed Rust source defect, corrected offline**, baseline `0fe4e785`.
`transport::send_path::claim` prevents a request ID migrating between comparison
Lab sends and public Outbox sends. Previously its first adoption checked legacy
lab checkpoints and paginated all outbox namespaces in a read transaction, then
installed the claim separately. A historical record inserted before the later
write transaction could be missed. Rechecking only a message-path row did not
close that gap when no competing explicit claim existed.

First adoption now delegates to `claim_in` within the write transaction. That
helper checks an existing explicit claim, the historical lab-send checkpoint,
and all outbox pages using WriteTx::scan_after, then installs the claim. It sees
uncommitted changes in the same transaction and serializes the historical check
with the claim write. An existing explicit same-path claim retains its fast
idempotent return; a different path refuses. Existing policy for simultaneous
historical lab/outbox ownership is unchanged.

The offline regression inserts conflicting historical work inside a write
transaction, then calls the actual transactional claim helper. Lab adoption sees
a just-enqueued outbox entry in another namespace; Outbox adoption sees a
just-written uncertain lab checkpoint. Both conflicts roll back checkpoint rows,
outbox rows and journal head, verified again after reopening the encrypted store.
After rollback, a fresh valid claim succeeds and prevents the alternate path.
Existing historical/restart tests continue to cover committed legacy records.

This is local duplicate-send prevention, not an APK wire field or evidence of
server restriction causality. It does not authorize replay, erase outcomes,
rebind account identity, or prevent an unrelated writer from creating new
historical state after a claim. Each implementation's reservation and uncertain
outcome checks remain necessary. No live network/account operation is performed.

## Historical pagination and entry-point check

**Confirmed current source/test behavior**, baseline `902e0379`; no production
policy change in this follow-up. The comparison CLI's transport entry claims
Lab ownership before inspecting/reusing its message checkpoint or starting media
work. The outbox-send CLI claims Outbox before its upload/preparation workflow.
The direct and group outgoing adapters also claim Outbox during initial send
selection, before key/membership query reservation. These are named call sites,
not proof that every low-level library primitive enforces a claim.

The atomic historical scan uses WriteTx::scan_after with an exclusive physical
cursor and page size 256. Outbox keys are blinded, so creation/insertion order
cannot be used to select a second-page conflict deterministically. The new
regression enqueues 300 synthetic intents across two namespaces, reads their
actual physical ordering and selects the final physical row. It verifies that
row is absent from the first 256, reopens the encrypted store, and invokes the
public claim entry point. Lab adoption refuses without changing outbox rows,
checkpoints or journal head. Outbox adoption then succeeds, and another reopen
still refuses Lab migration. No query or send occurs.

This covers the transactional scanner's multi-page adoption boundary after the
atomicity change. Complete caller coverage, arbitrary external checkpoint writers
and account lifecycle remain distinct from this local historical-path guard.
