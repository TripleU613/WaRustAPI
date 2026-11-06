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
