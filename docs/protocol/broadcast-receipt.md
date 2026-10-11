# Ordinary broadcast confirmations omit status class

**Confirmed static / implemented offline**, 2026-10-11. WhatsApp 2.26.37.71,
SHA-256 `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The shared receive builder previously gave every broadcast list class=status.
Ordinary list confirmations now omit that attribute, retaining destination,
message ID and sender device. Saved receipt wires are not rewritten.

## Ordinary receive handoff

The [dispatch trace](broadcast-dispatch.md) establishes ordinary lists without
status metadata as Bp2 -> 1UM -> 16p.BFC -> Ahc.run case 19. The new
[consumer DEX](apk/2.26.37.71-broadcast-receipt/ordinary-consumer.txt) retains
the complete switch. Its receive branch preprocesses through Cm3.A06, checks
the queue map and dispatches [A7Z.A00](apk/2.26.37.71-broadcast-receipt/receive-process.txt).
That schedules Agb with discriminator 11. The
[worker](apk/2.26.37.71-broadcast-receipt/receive-worker.txt) calls By0.A09.
[By0](apk/2.26.37.71-broadcast-receipt/decrypt-owner.txt) extends CoU; A09 is
the [inherited entry point](apk/2.26.37.71-broadcast-receipt/decrypt-entry.txt),
not the subclass's retry method A0B.

A09 runs before-decrypt observers and handles their stop/retry actions before
calling [CoU.A04](apk/2.26.37.71-broadcast-receipt/decrypt-flow.txt) under the
sender-device lock when present. A04 creates [CQn](apk/2.26.37.71-broadcast-receipt/decrypt-factory.txt)
and handles results through [A01](apk/2.26.37.71-broadcast-receipt/decryption-result.txt).
The [Java A04 envelope](rea/2.26.37.71-broadcast-receipt/decrypt-flow.json)
contains extensive duplicated blocks and contradictory-looking conditions;
its readable output is a navigation aid, not proof of the complete admission
policy. Full DEX is retained for further branch reconstruction.

[CQn.A00](apk/2.26.37.71-broadcast-receipt/plaintext-handoff.txt) selects
callbacks according to stanza class, payload version and account gates. Its
ordinary D52 callback [BEw](apk/2.26.37.71-broadcast-receipt/plaintext-callback.txt)
rejects empty plaintext, runs the Cnu post-decrypt hook and returns if that hook
produces BlP. Otherwise it invokes DDD.A00.
[DDD](apk/2.26.37.71-broadcast-receipt/content-processor.txt) selects Bp2 ->
1hF.A01 -> ordinary 178 -> 1gz.A01, separately from Bp1 -> status processing.
[1gz.A01](apk/2.26.37.71-broadcast-receipt/message-process.txt) calls
[1gz.A00](apk/2.26.37.71-broadcast-receipt/message-persist.txt), which contains
storage observers and several calls to ordinary receipt policy 0zH.A0K.
This establishes the ordinary message handoff, not every possible storage
observer, callback implementation or successful-message admission branch.

## Receipt policy and writer

[0zH.A0K](apk/2.26.37.71-broadcast-receipt/message-ack.txt) has separate
eligibility, bot-response and read-state policy. Its
[A0F](apk/2.26.37.71-broadcast-receipt/receipt-admission.txt) excludes several
message classes/states. The accepted ordinary branch calls
[A01](apk/2.26.37.71-broadcast-receipt/receipt-message.txt), which creates
Message.what=9 carrying the actual 178. [A07](apk/2.26.37.71-broadcast-receipt/receipt-forward.txt)
forwards to [0h8.A0S](apk/2.26.37.71-broadcast-receipt/send-queue.txt), then
[A09](apk/2.26.37.71-broadcast-receipt/send-enqueue.txt). The complete remaining
queue implementation is outside this trace.

The retained [writer dispatcher](apk/2.26.37.71-identity-ack-writer/writer-dispatch.txt)
maps what=9 to 043e..044c: actual 178 -> 0y2.A00. The
[ordinary writer](apk/2.26.37.71-message-primary-device/message-writer.txt)
calls [Cp2.A04](apk/2.26.37.71-broadcast-receipt/receipt-builder.txt) with mode=0.
A04 passes null extra attributes and signed sts=-1 to Cp2.A00. Thus ordinary
receipts have no class=status, regardless of the ordinary type-selection
branch. This differs from [Cp2.A06](status-receipt.md), which supplies that
class for the separate status holder. Both preserve the
[sender device wire form](message-primary-device.md).

## Rust scope and remaining gaps

received_confirmation changes only the shared broadcast receipt branch. A
plain list with no recognized status metadata now has three attributes:
to, id, participant. Exact meta is_group_status=true or
content_type=status_notify retains the previous status class for non-peer
candidates. [Exact category=peer exclusion](status-dispatch-gate.md) is now
implemented. The effective native configuration gate remains a shortcut,
not full native classification.

The account-aware actor and CLI use this shared base shape; own-account status
selection is a separate [shared account policy](status-ownership.md), now used
by the actor and the CLI's legacy receive loop. This change does not expand explicit read
API broadcast admission, retry support, sender-key eligibility, message-type
admission or own-list sender policy. Normal-message storage observer and
receipt eligibility parity still need work. A broadcast namespace alone is
not proof of identical encryption or full dispatch behavior.

The regression failed before the change on the fourth class attribute. Tests
cover PN/LID primary and LID companion list senders, unchanged incoming nodes,
destination and participant preservation, edit/recipient rejection, exact
metadata matching, and the existing status decryption fixture. All changes
operate on newly generated nodes; keys, identities, request IDs, queued wires
and uncertain outcomes retain their existing policy.

## Reproduction

[Twenty-one DEX topics](apk/recipes/2.26.37.71-broadcast-receipt.json) retain
full signatures, exception tables and register flow. Twenty-one full REA
envelopes retain engine and evidence identity; the old-counter helper is
retained separately to distinguish its negative acknowledgment from ordinary
success. Large methods remain only partly interpreted; retaining their bodies
does not mean every branch is understood or ported.

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-broadcast-receipt.json \
  --output docs/protocol/apk/2.26.37.71-broadcast-receipt --check
cargo test --locked -p whatsapp-api --all-features ordinary_broadcast_list_confirmation_has_no_status_class
```

No fresh live validation was performed; ordinary broadcast receipt shape is
verified offline, not full broadcast-list behavior or ban immunity.
