# Current notification handler constructor census

**Confirmed static**, APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
DEX exports from xtask apk-callers/apk-smali, not REA Evidence envelopes.
Commands: apk-callers APK --class LX/0v6; --method '<init>'; for each exact
class, apk-smali APK --class CLASS --method '<init>' --signatures --exceptions.
Each bounded constructor prefix ends at the base call and retains the complete
export SHA-256. JSON records code and the base boolean (executor construction),
not proven dynamic registration or ACK deferral policy. Complete14l/15I exports
use apk-smali without --method. Exact class-name string searches for
ServerSyncNotificationHandler/ServerNotificationHandler found no matches;
constructor numeric bindings located the handlers instead.

The existing [complete type map and consumer](../2.26.43.72-group-missing-identification/README.md)
contains38 put calls. This census finds34 physical base-constructor calls.
These are different populations: not every mapped code has a selected class
here, and special dispatcher codes may be absent from the type map. No
percentage-complete metric follows from either number. Interface dispatch,
reflection and dynamic registration remain unverified.

| Class | Registered code | Base executor enabled |
| --- | ---: | --- |
| LX/0v7; | 258 | true |
| LX/0v9; | 228 | false |
| LX/0vQ; | 246 | true |
| LX/0w2; | 279 | true |
| LX/0w9; | 209 | true |
| LX/0zw; | 276 | true |
| LX/10F; | 229 | false |
| LX/10G; | 234 | true |
| LX/10I; | 204 | true |
| LX/10a; | 97 | true |
| LX/111; | 287 | true |
| LX/11A; | 294 | true |
| LX/11I; | 247 | true |
| LX/12z; | 238 | false |
| LX/139; | 261 | true |
| LX/13C; | 256 | false |
| LX/13G; | 270 | true |
| LX/13K; | 285 | true |
| LX/14l; | 210 | true |
| LX/14v; | 265 | true |
| LX/15A; | 252 | true |
| LX/15D; | 290 | true |
| LX/15I; | 235 | true |
| LX/15v; | 241 | false |
| LX/19K; | 251 | true |
| LX/19N; | 203 | false |
| LX/19R; | 202 | true |
| LX/19V; | 25 | true |
| LX/19b; | 240 | false |
| LX/19h; | 248 | true |
| LX/19k; | 189 | true |
| LX/19n; | 104 | true |
| LX/19q; | 236 | true |
| LX/19v; | 274 | true |

## Selected unported obligations

**Confirmed static:** server_sync code210 is14l; server code235 is15I.
Both extend0v6 and construct its executor. Their complete sources are retained.
14l.A09 checks code210, parses notification/from as0mL and type server_sync,
then typed metadata, delete_all_data and0..20 collection entries. The
return of the delete_all_data parser is not assigned here; its parsing alone
is not proof of data deletion. Collection/version metadata is accumulated in
maps; selected companion bootstrap state permits0l8.A07(name,version) and
0aS.A0P calls. The existing [server-sync lifecycle audit](../../server-sync-lifecycle.md)
already establishes transactional dirty-version assignment (not max), bootstrap
and pending-mutation gates, single-flight scheduling and ACK-before-async-result
behavior. These are not newly unknown because this census retains a shorter
trace. C6N schedules2JI selector6 on the base executor, while C6L/C6M return.
The follow-up below identifies that callback as metrics. No generic server_sync
ACK was added.

15I.A09 checks code235 and first child only. Its selected branches handle log,
abprops and push-config. The abprops path checks readiness and calls HBE.A00
with true,false,0 and other configuration follow-ups. The push-config path
accepts missing type or equalsIgnoreCase(gcm), logs unknown other types and
calls RegistrationIntentService.A0F/1uj.A00(context,0Gq) on the selected path.
Logs describe force replacing the GCM token; the actual replacement job and
its persistence/requests are not traced here. Do not manufacture or reuse a
GCM token from these call edges. No branch in this class calls0v6.A07, but
indirect ACK or async behavior is not resolved solely by that negative.

**Rust source comparison:** connection.rs and notifications.rs have no dedicated
server or server_sync consumer/ACK. Capture is durable; it is not native state
application. A broad outer-type ACK would leave collection and configuration
obligations unimplemented. Native helper jobs, supported primary/companion roles,
ACK timing, error policy and current Rust app-state integration must be
reconciled before porting. This research changes no runtime behavior, identity,
server waits, outbound request or live account. Constructor counts do not prove
ban causes or complete notification coverage.

Checksums record every other file's bytes and SHA-256.

## C6N completion callback is metrics, not ACK ownership — 2026-11-05

**Confirmed static:**2JI.run discriminator6 selects1202. It reads14l.A03,
returns via shared001c when empty, otherwise sums each Number.longValue minus1.
The loop exit at123c jumps by0xf66 DEX code units to3108. It obtains0fz,
constructs28R, stores the sum in28R.A00 and submits it to0NX.CSW, then clears
the map and returns. No base notification ACK/deferral or sync dispatch occurs
in these selected blocks. This is a bounded call-body result, not proof about
all metrics-provider initialization effects or all other2JI discriminator arms.

**Research reconciliation:** dirty-version persistence,0aS.A0P/A0Q and0gG.A00
were rechecked against the current APK, but are already retained in the earlier
lifecycle/versioned-fetch audits.0gG.A00's direct body is a provider lookup;
transitive initialization is not known to be a no-op. The earlier full lifecycle
trace proves14l does not defer ordinary ACKs: writes/scheduling precede normal
base return, asynchronous results follow. C6N does not add a new gate to that
ordering. The remaining implementation prerequisite is mapping actual Rust
role/recovery/bootstrap/ready-mutation state and durable notification work,
not waiting for this metrics callback. No dirty metadata should become an
applied collection version or fabricate native preference defaults.

No runtime change or live operation. This follow-up avoids duplicating existing
bulk scheduler/version-store exports and retains only the newly traced callback.

## Concrete BU7 role predicate — 2026-11-05

**Confirmed static:** apk-smali APK --class '' --method BU7 --signatures
--exceptions selects three physical declarations:09F abstract,09G concrete and
09O concrete. Both concrete bodies call09O.BU8(false). This is a physical
method-name census, not complete interface/runtime dispatch proof.
BU8 calls09O.A07 first. With false it reads09T.A01/09U SharedPreferences
registration_device_id with default0 and returns true only for value>0.
The unused true arm consults an AtomicInteger, treating Integer.MIN_VALUE as
uninitialized and CAS-loading the same preference; it still returns value>0.
Negative values do not become true. This closes the selected predicate body,
without establishing all writers or validity rules of that saved integer.

A07 checks09a.A01; when not initialized it schedules1Wk selector1 through
09a.A04 then calls09a.A00. Callable body and waiting/failure semantics remain
untraced. The preference read occurs after this initialization call, so absence
of a Rust preference row is not proof that native initialization would leave0.
Rust AccountRecord and primary_login contain no equivalent persisted native
registration_device_id authority. Its explicit primary payload is useful source
context but not a migration of native role/bootstrap/recovery state. Complete
Rust mapping is [separate](../../server-sync-rust-state-mapping.md).
No new default role value, preferences, query, ACK or runtime policy was added.

## Saved role ID writer/reader census — 2026-11-05

**Confirmed static:** exact registration_device_id anchor scan finds8 physical
references:09O.A01 reset;09O.Auf read; two09O.BU8 reads;0Qq.A0a selected-account
read;107.BLn write;1nW.ABS protobuf-producer read;G0j.A02 reset. This is a
constant-string census, not proof against reflection, dynamic keys, preference
clear/import or indirect writers.

09O.A01 clears Me/providers and writes0 through putInt/apply, then sets the
09T atomic cache to0. Auf returns the saved integer directly with default0.
0Qq.A0a takes5VA, calls5kq.A00(selectedAccount,reader,key) and tests positive;
this is selected-account state, not automatically the current account. Full
5kq directory/key scoping still needs separate verification.

107.BLn's selected registration branch gets v2 from DeviceJid.getDevice on
v13 at13d2 or v14 at140a. Its retained writer block stores that integer in
09T.A01/09U preferences at153e..1550 and updates the atomic cache at1550..155a.
This binds the local value source to a device JID, not arbitrary phone digits.
It does not re-prove every upstream pairing/HMAC/account-signature or owner
binding branch; the [earlier current companion-ingress trace](../../own-companion-ingress-current.md)
retains that broader selected producer. Do not treat any supplied Rust Jid.device
as authorization to write a registered companion index.

G0j.A02's selected cleanup block writes0 with6og.A1M (putInt/apply), separately
writes companion_registration_state0 with commit, then resets the atomic cache.
This is a distinct field/operation: no atomicity across the preference writes,
cache update and broader cleanup is proved. No cleanup/reset was executed.
1nW.ABS reads the preference in its protobuf-building path; that reference is
not another role setter. No full protobuf-field mapping is asserted here.

The1Wk selector1 callable loads selected filesDir/me through0D0.A01, updates
through09O.A00 and completes09a coordination; broader load/invalid-identity
behavior was already traced in [read-self authority](../../read-self-authority-current.md)
and [own identity ingress](../../own-companion-ingress-current.md). The earlier
[role/recovery audit](../../server-sync-role-recovery.md) already establishes
BU7 companion mode and recovery semantics. Recent census/mapping unknowns
are qualified by those existing records; no re-discovery is claimed as a port.

Remaining gaps are exact Rust authority/migration, all positive device-index
writers/clear/import paths and full producer-to-account binding. Rust has no
native preference row to default/reset automatically. No runtime behavior,
ACK, query, identity, account switch or live account operation changed.
