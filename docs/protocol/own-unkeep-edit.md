# Own-message undo-keep edit metadata

2026-11-04. **Confirmed source and offline implementation**, with two separate
APK identities:

- The comparison behavior is traced from **2.26.37.71**, SHA-256
  `1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
- The final edit-field serialization was independently checked in **2.26.43.72**,
  SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.

This is one bounded outgoing metadata correction, not full keep permission,
expiration, cross-device or whole-APK parity. No live message was sent.

## Original ownership controls the field

The retained 37.71 [11x.A01 action](apk/2.26.37.71-group-keep-outbox/keep-action.txt)
first gets the original target's key at `016c..0170`. At `0320..0334` it writes
7 into the keep add-on's `178.A00` only when that original key is `fromMe` and
the new keep flag is false; every other branch writes 0. The same flag is
stored into the add-on and original message, and the failure handler reverses
it. The independent [enum values](apk/2.26.37.71-control-enum/keep-values.txt)
and [receiver restriction](apk/2.26.37.71-own-device-keep/keep-receiver.txt)
support the existing interpretation true=KEEP, false=UNDO. The receiver rejects
nonzero edit metadata on KEEP, not on UNDO. That interpretation remains
**inferred from multiple retained branches**; no new live experiment promotes it.

The resulting four-case field rule is:

| Original target | Action | Root message attribute |
| --- | --- | --- |
| Incoming peer message | keep | absent |
| Incoming peer message | undo keep | absent |
| Our outgoing message | keep | absent |
| Our outgoing message | undo keep | `edit="7"` |

The add-on is sent by this account in every case. Its own sender identity must
not be substituted for the **original target's** `fromMe` predicate. The
direct path's existing `validate_keep` checks only that the supplied target
belongs to the selected chat. Its `MessageRef.direction` is caller-supplied and
already determines the keep protobuf key's `fromMe`; this chunk mirrors that
same value in stanza metadata. It does **not** establish authenticated stored
ownership or native keep permission for direct targets. The group path retains
its stronger saved-reference/original-author resolution and final transactional
recheck. Neither path gains new ownership verification from this metadata fix.

## The stanza writer is no longer an untraced assumption

The older [unknowns](unknowns.md) and [group outbox note](group-keep-outbox.md)
left final `edit` mapping inferred. However, the already retained 37.71
[SendE2EMessageJob.A06](apk/2.26.37.71-groups/group-send-job-hash-check.txt)
explicitly reads `Civ.A02` at `1444`, skips zero, converts nonzero to decimal,
and emits `edit` at `1454..145e`. That is a confirmed attribute writer, not an
inference from the ACK path alone.

The current 43.72 complete checked [A06 disassembly](rea/2.26.43.72-own-unkeep/send-job-send.txt)
closes its own named-field-to-writer path:

1. `0bac..0bb0` loads the job's `editVersion`; `0bdc..0be4` places it in the
   third integer argument of `Ewq.<init>`.
2. The complete REA [Ewq constructor](rea/2.26.43.72-own-unkeep/message-builder-constructor.json)
   writes that third integer (`i3`) into `Ewq.A03`.
3. At `13f6..1410`, A06 reads `Ewq.A03`, skips zero, calls
   `String.valueOf(int)` and appends the `edit` attribute. This is the message
   writer, not the acknowledgement serializer.

REA Evidence for the constructor:
`ev_f8ff3913dff7b5a6a6bce640273cb7c5b1b821809aa79856ddc7a479ee950611`.
The job separately carries the same nonzero decimal value into `Efa.A07`
(`0b5e..0b94`), then through the complete
[stanza-key builder](rea/2.26.43.72-own-unkeep/stanza-key-builder.json) and
[stanza-key constructor](rea/2.26.43.72-own-unkeep/stanza-key-constructor.json).
This corroborates its correlation-key identity; it is not substituted for the
actual message-writer proof above.

**Limit:** the keep action itself was not remapped/retraced in 43.72 by this
chunk. The 37.71 predicate remains the comparison port's source. Confirming
current serialization does not establish that every current keep permission or
producer is identical across releases.

## Rust change and tests

The direct and group builders now share `outgoing::with_edit_attribute`.
Existing sender revoke (`7`) and message edit (`1`) behavior is preserved.
Own-target undo keep adds `7`; normal keep and peer-target undo add nothing.
The keep protobuf, original author, timestamp, encryption, target guards,
request IDs, reservation and replay rules are unchanged.

**Offline verified:** the direct-envelope fixture covers all four supplied
original-direction/action cases, verifies the actual keep protobuf's `fromMe`
and keep type against the same native truth table, and preserves ciphertext,
destination, type and message ID. It intentionally uses no parent store, so the
test cannot be read as proof of authenticated direct-target ownership. The existing real group preparation test now checks the same root attribute
for PN and LID paths, decrypts the paired sender-key distribution and group
payload, verifies native key/fromMe/participant/keep-type/timestamp fields, and
reopens the encrypted store to confirm no second reservation or replay. The
existing reply/reaction cases also continue to require no edit attribute.

Reproduce the focused tests with:

```sh
cargo test --locked -p whatsapp-api --all-features keep_edit_attribute_follows_original_ownership_and_preserves_envelope
cargo test --locked -p whatsapp-api --all-features normal_group_references_decrypt_with_exact_participants_and_never_replay
```

Full workspace and lint results belong to the integration chunk's status.
Handset acceptance/rendering and any native keep permission/expiry behavior
outside the existing port remain unverified.
