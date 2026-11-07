# Retry session and identity transaction scopes

**Confirmed selected static**, WhatsApp APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Existing xtask DEX fallback; Android REA provider remains unavailable. No new
REA Evidence ID or runtime observation. Instruction PCs are bytes; exception
and branch offsets are code units. Complete methods are retained except the
explicitly bounded FcP.run excerpt and field-reference census.

## Concrete adapter result handling

[0Yj.CrT](session-adapter.txt) validates nonempty Alice base key, serializes Eul
and calls0WJ.A16; the boolean result is discarded. Its selected IOException
validation failures become IllegalArgumentException. [0Yj.Cbo](identity-adapter.txt)
converts EuZ to Ee1 and calls0WJ.A12; that boolean is also discarded. Its ECt
conversion handler throws AssertionError, outside ordinary Exception mapping.

[0WJ.A12](identity-policy.txt) has a companion-identity policy path that logs and
returns false without reaching A0J. Other paths call A0J with null E7b and null
Ejr array. False is not uniformly a persistence failure: A0J's boolean also
reflects identity-change/event decisions. Neither boolean reaches Ecx.A00's caller.
Do not equate adapter return with confirmed identity adoption.

## Separate local transaction scopes, nested behavior unresolved

[0WJ.A16](session-store.txt) resolves aliases and validates serialized Etk before
constructing FcP discriminator26. With A0I enabled it locks FjI and calls
[A0F](session-transaction.txt), which opens0aq/1hk, runs the callback, marks
successful then closes. Otherwise A16 itself opens that transaction around the
callback under an Eoe monitor and marks/closes before returning. Its early Etk
IOException handler logs invalid-session-record and returns false. Database
exceptions are not covered by that early IOException range.

[0aq.A00](handle-begin.txt) calls [A02](handle-constructor.txt), constructing
[1hk](transaction-handle.txt). Constructor has a thread-local outer-manager path
using SQLite beginTransactionWithListener, or0KU.A0E. 1hk.A00 marks successful
through0KU.A0G; close calls0KU.A0F and tracks completion, with exception handling.
[0KU.A0E](database-begin.txt), [A0G](database-success.txt),
[A0F](database-end.txt) bind these calls to SQLite beginTransaction,
setTransactionSuccessful and endTransaction respectively. End corruption is
reported then rethrown. These are real transaction scopes, not just mutexes.
Their outer/nested manager and shared-connection policy are not fully traced.

[0WJ.A0J](identity-store.txt) obtains identity locks and old identity/metadata,
opens another0aq/1hk scope at00a6..00ba, performs selected0YT.A0I identity writes
and metadata updates, marks successful0158 and closes0168..0174. It subsequently
handles optional AEA and identity events outside that local scope. In this Cbo
call E7b is null, so this optional AEA path is skipped; A0K's later AEA update
remains separate. The selected earlier session transaction has closed before
this identity call, but a shared enclosing transaction is not excluded. No
whole-retry atomicity, rollback or crash recovery is proven.

## Session SQL and an additional IOException policy

[FcP constructor](runnable-constructor.txt) binds discriminator26 to bytes,
coordinator, resolved session address and original address from A16. The
[bounded run excerpt](session-runnable-excerpts.txt) retains dispatch, full
selected branch0394..0584, its reachable return22c0, and overlapping exception
records. It updates sessions.record using five dimensions: recipient_account_id,
recipient_account_type, device_id, session_scope and session_type. Zero updates
lead to insertion with record/address/scope/type and timestamp. The branch opens
its own nested1hk scope, marks it successful and closes before updating Eoe cache.
Neither the insert return value nor an exact-one update check is consumed here.
Alias resolution and concrete SQL wrapper semantics remain distinct proof needs.

IOException around the selected database/cache work reaches0544: it logs failure,
then wraps as Dvm only when0WJ.A0T.get() equals Boolean.TRUE. Otherwise it returns
via22c0. This proves a conditional propagation policy, not that any particular
retry swallows a database error. The [exact field census](failure-policy-refs.txt)
finds only clinit write plus reads in FcP.run and Ff8.call. ThreadLocal contents
can be set through those references; the census is not a write-to-contents audit.
Ff8.call is the next policy-provenance trace; its body has not been inspected here.

## Rust comparison and next work

Research only. Rust receipt_keys_unsupported remains unchanged: no key adoption,
new request, signature bypass, attestation generation or live account operation.
Do not port ignored booleans or assume native method zero implies atomic durable
success. Confirm concrete SQL failures, nested manager behavior, Ff8 failure flag,
identity schema/alias ownership and later ExG/DDs updates before integrating the
complete receipt-key workflow. Existing Rust owned transactions and no-replay
rules must be evaluated against that full chain rather than replaced by guesses.

Reproduce complete exports with existing `xtask apk-smali APK --class CLASS
--method METHOD --signatures --exceptions`; transaction-handle retains all1hk
methods. Reproduce field census with apk-field-refs exact0WJ/A0T/ThreadLocal.
FcP.run extraction ranges and full export hash are in the excerpt header; the
full unrelated discriminator branches remain outside Git. Manifest covers each
retained file except itself.

Follow-up: [strict flag owner and failure propagation](../2.26.43.72-strict-signal-store-failure/README.md)
now binds Ff8 discriminator2 to migration bootstrap and retains scoped restore
behavior. Ordinary retry linkage and enclosing atomicity remain unproven.
