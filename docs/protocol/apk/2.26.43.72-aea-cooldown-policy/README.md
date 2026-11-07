# Local AEA cooldown read/write policy

**Confirmed selected static**, pinned WhatsApp APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Existing xtask DEX fallback; Android REA provider unavailable. No new REA Evidence
ID or runtime observation. Complete selected methods have signatures/exceptions.
PCs are bytes; branch offsets and exception ranges are code units.

## Write result is weaker than no exception

Previous [DDs.A03](../2.26.43.72-retry-chat-aea-storage/chat-update.txt) invokes
0KU.A09 for aea_chat_state with conflict argument5 and discards its long return.
The [complete wrapper](insert-wrapper.txt) calls Android SQLiteDatabase
insertWithOnConflict. It reports/rethrows SQLiteDatabaseCorruptException but
catches SQLiteConstraintException, logs and returns **-1**. Thus the caller can
continue on a -1 result without reaching its thrown-failure/rollback branch.
A long return from SQLite itself is likewise passed through, not asserted.
Static evidence proves result handling, not that any live cooldown insert failed.
Do not port this ignored-result behavior into Rust or infer durable success.

## Clock and configured interval

[Write clock2LC.A09](write-clock.txt) and [read clock2L7.A01](read-clock.txt)
resolve their provider then call System.currentTimeMillis. cooldown_started_at is
therefore wall-clock milliseconds, not uptime or server time. [DDs.A0P](cooldown-reader.txt)
reads feature0x7a16 through [6oe.A02](duration-getter.txt), which widens00E.A0a's
signed integer to long, then multiplies by60*60*1000. This establishes hours as
the configuration unit; it does not establish the effective value/default.

The selected active test is `now_ms - started_ms < configured_hours * 3600000`.
Equality expires. No clamp of negative elapsed time is visible before this
comparison, so a future stored timestamp can satisfy it; no alternative clock
policy is inferred or implemented. Telemetry later reports remaining seconds
with integer division and clamps that reported remainder to zero. Reporting is
not the admission predicate.

## Read-side admission and clearing

A0P first resolves the chat row ID; missing row returns false. It queries
cooldown_state/cooldown_started_at. Missing row/column or unrecognized E7O integer
leaves no valid cooldown and returns false. [Enum](cooldown-states.txt) binds
stored E7O1=E2EE,2=HYBRID_E2EE,3=NON_E2EE (ordinals0/1/2, not stored values).
[F1d.A02](chat-state-map.txt) maps chat enum ordinals using prior retained0i7
constructor evidence: DEFAULT_E2EE/GUEST_E2EE/PENDING_INVITE_E2EE to E2EE,
COEX/WORK to HYBRID_E2EE, UNKNOWN to null, remaining ordinals to NON_E2EE.

When expired, A0P attempts deletion by chat_row_id in a local transaction. While
active, selected UserJid predicate0Dn.A0k paths also read the primary-device
identity timestamp. [0YT.A09](identity-timestamp.txt) queries identities.timestamp
using recipient/type/device address (zero for absent/unresolved). A positive
identity timestamp multiplied by1000 that is strictly later than the cooldown
start takes the same delete path. This comparison treats the stored identity
value as seconds; its producer and all alias/device policies remain untraced.
Deletion count is discarded. Selected thrown deletion failures are logged and
return false, not a successful clear proof. No row-count or durable clear proof
may be inferred merely from the returned boolean.

For an active cooldown with no selected reset, A0P compares F1d's proposed class
to the cooldown class. Equal returns false; differing classes emit bounded
classification telemetry and return true (suppress proposed state change).
Unknown/null proposed classification differs from any valid stored class here.

## This guards chat classification, not a send rate limit

[Exact A0P callers](reader-callers.txt) are DDs.A03 and ExG.A06 in this build.
A03 checks it for recomputed/null desired state under feature policy. The
[complete ExG.A06](usync-account-type.txt) is explicitly named by native log as
updateChatEncryptionStateFromUsyncAccountType. It parses exact e2ee/hybrid_e2ee/
non_e2ee strings (other/null -> NOT_SET), derives/map chat state through ExG,
and skips the state update when A0P returns true. Otherwise0x7497 permits A0M
with null E7b, reason02X.A1G and false cooldown-request boolean. The full USync
producer and network reply path are not traced in this bundle.

[Anchor census](anchor-refs.txt) is exact literals aea_chat_state and
cooldown_started_at, not all dynamic SQL or every downstream state consumer.
The [schema descriptor](schema-descriptor.txt) retains three column declarations;
its builder field meanings are not fully decoded here. The [literal trigger](chat-delete-trigger.txt)
deletes matching aea_chat_state rows before deleting chat._id. No foreign-key
assumption is needed from descriptor flags.

This proves local classification stabilization with expiry/identity reset; it
establishes no server rate-limit/ban wait, sending authorization, retry deadline
or direct outbound request suppression. Broader indirect effects of chat state,
notification/telemetry delivery and effective config remain unknown.

## Current Rust and integration boundary

Research only. No corresponding AEA cooldown or receipt-key adoption is added
from this partial trace. Existing explicit receipt_keys_unsupported and owned
persistence/no-replay rules remain. Local peer_verification is not a substitute
for these classification states. Next trace full USync account-type consumers,
identity timestamp producer, classification-to-network policy and cache/callback
recovery before integrating state changes. No runtime behavior, attestation,
account operation or live request changed.

Reproduce existing apk-smali with signatures/exceptions for0KU.A09,2LC.A09,
2L7.A01,6oe.A02,DDs.A0P,0YT.A09,F1d.A02,E7O.<clinit>,ExG.A06,0Mp.ATc/ATe;
apk-callers exact DDs.A0P and apk-xref exact retained anchors. Manifest covers
all retained files except itself.
