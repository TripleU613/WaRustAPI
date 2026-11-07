# Retry post-session chat AEA state storage

**Confirmed selected static**, pinned WhatsApp APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Existing xtask DEX fallback (Android REA provider unavailable), no new REA
Evidence ID. Selected complete methods retain signatures/exceptions. PCs are
bytes; exception/branch offsets use code units. No live/account observation.

## Caller state selection differs from identity AEA

The previously retained [Em2.A02](../2.26.43.72-retry-aea-admission/post-session-state-update.txt)
extracts E7b state and userJid after selected A0K status0. Its helper1Xl.A1Y
returns true ([complete helper](one-helper.txt)), resolving v4=1 in this caller.
With0x6f6f enabled, ExG.A02 maps the user address for chat storage. E7b NOT_SET
returns without update. HYBRID_E2EE/NON_E2EE call ExG.A01 for a chat-state result;
E2EE calls DDs.A0L(mapped,true), converting COEX to DEFAULT_E2EE in this selected
branch. Gate0x7497 controls DDs.A0M(mapped,state,reason02X.A15,boolean), whose last
boolean is true only for UNRESOLVED_NON_E2EE here. No effective flag value is
asserted. These are chat states, not a second write of identity-table E7b.

[ExG selected methods](chat-policy.txt) retain full state selection, two special
state sets and address mapping. Mapping can use0Ws.A0H or fall back to the
original user address. A01 depends on EMO policy, additional peer predicates
and a lazy current-chat-state lookup; it is not a direct enum cast. Full helper
and feature provenance remains open; do not collapse it to E7b or a verified bit.
[Chat enum](chat-states.txt) distinguishes UNKNOWN, UNSET, DEFAULT_E2EE,
NON_E2EE, CAPI, COEX, BOT, BSP_MANAGED, GUEST_E2EE, OPEN_BOT_GROUP,
PRIVATE_BOT_GROUP, PENDING_INVITE_E2EE, UNRESOLVED_NON_E2EE and WORK.
The stored `value` is distinct from Enum.ordinal; retain constructor inputs.
[DDs.A0L](state-read.txt) may recompute state through A02 depending on stored
state and peer metadata. Its bool does not mean merely load a raw saved enum.

## Per-chat lock and a guarded local transaction

[DDs.A0M](update-wrapper.txt) delegates to [A03](chat-update.txt), which obtains
one ReentrantLock per address with ConcurrentHashMap.putIfAbsent and releases
it on normal and exceptional paths. It locates an existing chat row; missing
row logs and returns. A null desired state triggers recomputation and further
policy, while the selected Em2 call supplies a nonnull desired state.

Before SQL, A03 refuses UNRESOLVED_NON_E2EE when old state belongs to the
[resolved set](resolved-states.txt): BOT, COEX, CAPI, WORK, BSP_MANAGED.
An unchanged enum also skips the write path. These are native transition guards,
not evidence that a particular remote attestation should be accepted.

On a change A03 opens0aq/1hk and updates `chat.chat_encryption_state` with
0i7.value using `_id = ?`. Zero affected rows skips success marking and cache/
callback updates. A nonzero result calls DEm.A02 with the same writable handle
and chat row ID. False logs classification-write-failed and closes without
marking successful. [Complete classification method](classification-write.txt)
asserts SQLite inTransaction, has policy/migration readiness branches and SQL
for chat_encryption_classification. Its full classification derivation is not
claimed traced here; retaining its complete method fixes the delegated boundary.

## Optional cooldown joins that transaction

When the final boolean is true, A03 obtains a time value through2LC.A09 and
selects E7O state from supplied E7b or F1d.A02(chat state). It writes
`aea_chat_state` with chat_row_id, cooldown_state and cooldown_started_at via
0KU.A09 conflict argument5. That call's long return is discarded; selected
thrown failures become a failure result, are logged and leave the transaction
without success marking. Do not claim an exact-row check or fully verified SQL
conflict behavior until the wrapper is bound.

Successful paths reach1hk.A00 at0242. A requested cooldown with no known mapped
state logs and also marks successful without that insert. Ordinary no-cooldown
path also marks successful there. Cooldown failure logs rolling-back and skips
that point. This is a local chat-classification/cooldown transaction, **not** a
server ban cooldown, registration wait or proof of network acceptance. Its
clock source/units, duration, expiry and read-side enforcement are still untraced.

After success marking and before closing the transaction handle, A03 mutates
cached0i4.A0g and bot-group flag when that cache object exists. It registers an
8af callback through [0aq.A04](transaction-callback.txt), which asserts an active
SQLite transaction and delegates to0HC.A00. Callback contents, ordering relative
to outer commit/rollback, cache rollback/invalidation and crash recovery remain
unknown. Do not call this atomic durable cache synchronization on static call
ordering alone. Prior transaction-handle evidence binds1hk to SQLite success/end;
no transaction covering earlier session/identity writes has been established.

## Rust boundary and next proof

Research only: receipt_keys_unsupported remains, no runtime key adoption or new
request. Local peer_verification is not this chat classification machine. This
pass identifies concrete missing state/transition/transaction semantics before
receipt-key integration; it does not demonstrate a new defect in supported Rust
traffic. Next bind ExG lazy policy helpers/alias ownership, classification schema,
0KU.A09 behavior, cooldown clock/readers, callback/cache recovery and full owned
retry workflow. No attestation generation, verification bypass, live request or
account operation. Reproduce selected apk-smali methods with signatures and
exceptions for ExG.<clinit>/A01/A02, DDs.<clinit>/A03/A0L/A0M,0i7.<clinit>,
DEm.A02,0aq.A04 and1Xl.A1Y. Manifest covers each retained file except itself.

Follow-up: [cooldown read/write policy](../2.26.43.72-aea-cooldown-policy/README.md)
now binds wall-clock milliseconds, configured-hour expiry, selected identity
reset, classification suppression and ignored SQL failure returns. Full effective
configuration and indirect network policy remain open.
