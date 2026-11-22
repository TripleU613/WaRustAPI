# Key-query `context_jid` (43.72)

**Confirmed static**, APK **2.26.43.72** (SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`), read with
`xtask apk-smali` / `apk-xref` / `apk-field-refs`.

## Writer

`X.0v0.A00` (writer thread, key-fetch case) builds

```xml
<iq id=… xmlns="encrypt" type="get" to=…>
  <key pqsupport="true"? context_jid="…"?>
    <user jid=… [reason="identity"]/> …
  </key>
</iq>
```

and adds the attributes in that order:

- `pqsupport="true"` when the job's `fetchPqPrekey` boolean is set
  ([PQ predicate](kem-prekey-inventory.md#pqsupport-on-the-key-query-implemented-2026-11-22)).
- `context_jid="<raw jid>"` when **property 25904 (`0x6530`) is true**, the
  context JID is present, and its `Jid.getType()` is one of the accepted
  classes: `X.0Dn.A0l` (types 1/2), `A0Q` (type 3) or `A0h` (type 5). Those
  numeric classes are the group, broadcast-list and status namespaces
  ([JID type table](cross-device-sync.md#concrete-jid-kinds-in-group-mode-classification-2026-10-07)),
  so an ordinary user chat contributes no attribute.

`08Y.A0H` registers `0x6530` with `Boolean.TRUE` (`v2 = Boolean.valueOf(true)`),
so the static default is **enabled**; a settled A/B reply can turn it off.

### Producer

The only producer of the `contextJid` bundle extra is `BulkGetPreKeyJob`
(`contextRawJid = context.getRawString()`), created by
`PreKeysRequestManager.A05`/`A00` from a per-request context JID (`X.EX8.A04`,
the chat that requested the directory lookup). `BulkGetPreKeyJob` also carries
`sessionScopeOrdinal` and `isPqSessionUpgrade`; the writer rejects a PQ upgrade
that does not fetch PQ prekeys.

## Rust

`transport::abprops::CONTEXT_JID_PROPERTY` (25904) with
`context_jid_enabled`, defaulting to true when no settled value exists.
`messaging::key_request_with_context` / `key_request_many_with_context` add the
attribute (after `pqsupport`) only for `Server::Group` and `Server::Broadcast`
contexts.

`device_key_batches` freezes the decision with the first page: `Batch.context`
is stored only when the property is enabled at creation, and both the reserved
write snapshot and later pages rebuild the query with that stored context, so
the dequeue payload stays byte-identical. The group send path passes
`entry.intent.chat.jid()`.

Tests: `context_jid_is_attached_for_group_and_broadcast_contexts_only`,
`context_jid_property_defaults_true_and_follows_the_settled_reply`,
`group_context_rides_on_the_key_query_and_survives_rebuild` (including refusal
of a rebuild without the attribute) and
`disabled_context_property_leaves_the_key_query_without_the_attribute`.

## PQ session-upgrade marker (2026-11-22)

**Confirmed static**, APK **2.26.43.72**, `X.0v0.A00` key-fetch case:
`fetchPqPrekey` (16c4..16cc, default false), `sessionScopeOrdinal`
(16e6..16f2, int), `isPqSessionUpgrade` (16f2..16fe, default false).
Before writing, 1706..171e throws `IllegalArgumentException("a PQ session
upgrade must fetch PQ prekeys")` when the upgrade flag is set without the PQ
fetch flag.

Per requested user (178c..1806):

1. member of `identityJids` → `<user jid=… reason="identity"/>`
2. else `isPqSessionUpgrade` → `<user jid=… reason="pq"/>`
3. else → `<user jid=…/>`

Then `pqsupport="true"` when `fetchPqPrekey` (1818..1836), then the
`context_jid` filter above, then the IQ attributes. **The marker is the
`reason="pq"` attribute, not a `<user><pq/></user>` child**; that earlier
guess is refuted.

Producer (`C243214t.A00`, z3 = `isPqSessionUpgrade`): an upgrade bypasses the
in-flight dedup map and never populates the identity list, so an upgrade batch
has no `reason="identity"` users; `A01` drops upgrade batches while PQ backoff
is active. `BulkGetPreKeyJob.A0N` returns `!isPqSessionUpgrade`, so an upgrade
failure is terminal instead of retried.

Rust: `messaging::key_request_many_with_pq_upgrade` adds the user reasons with
identity precedence and refuses the native-invalid combination with the same
diagnostic text. `peer_keys::pq_upgrade_preview` builds the exact stanza
offline and is exposed as `peer-keys --preview-pq-upgrade` (redacted:
namespace, type, reason, `pq_support`, encoded size).

**Why no send path:** a PQ session upgrade requires our KEM prekey to be
published before a PQ bundle can be built; this client does not generate or
upload one ([KEM inventory](kem-prekey-inventory.md)). Selecting the marker
early would request material the upgrade cannot use, so it stays an offline
builder until KEM generation exists. The `pqsupport` predicate
([`pq_support_for`](kem-prekey-inventory.md#pqsupport-on-the-key-query-implemented-2026-11-22))
is unchanged and already requires a sent KEM row.

Tests: `pq_upgrade_marks_each_user_with_reason_pq_after_identity`,
`pq_upgrade_preview_requires_pq_support_and_prints_no_address`; the existing
identity-change tests pin that ordinary producers never emit `reason="pq"`.

## Limits

- Direct user-chat key queries intentionally carry no `context_jid`: the traced
  writer would drop a user-class address, and the APK's context producer is a
  chat lookup.
- `sessionScopeOrdinal` is carried in the native job record (`Dxh`) but is not
  part of this stanza; it is not ported.
- The PQ session-upgrade marker is implemented at the writer and previewed
  offline, but no send path selects it (see below).
- The property is read once when the batch is created; a later A/B change does
  not rewrite an in-flight attempt.
- No live request was sent; server treatment of `context_jid` is unobserved.
