# Bulk prekey refresh request and callback

APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Confirmed static:** this continues the
[identity-notification scheduler](identity-notifications.md). It traces named
job inputs into wire attributes, not a live identity transition.

## Job and wire request

[BulkGetPreKeyJob constructor](apk/2.26.37.71-bulk-prekey-refresh/bulk-job-constructor.txt)
keeps the normal `jids` list separately from `identityChangedJids`, plus context,
fetchPqPrekey, contextRawJid, sessionScopeOrdinal and isPqSessionUpgrade.
PQ session upgrade requires PQ fetching; these booleans are distinct. The
[run method](apk/2.26.37.71-bulk-prekey-refresh/bulk-job-run.txt) reconstructs
DeviceJid lists, prepares their combined set through `Cio.A02(..., false)`
(`0056..00a0`) and emits handler message **what=0, arg1=87, arg2=0**
(`01da..01e8`). It carries the lists as `jids` and `identityJids`; the reason-8
context is used separately for job telemetry, not copied as a numeric wire
`reason` here. The [processing continuation](bulk-prekey-processing.md) now traces its
conditional owner sync; complete mapping authority remains **unknown**.

The [existing full 0y0.A00 dispatcher](apk/2.26.37.71-read-receipt-job/read-dispatch.txt)
selects 87 -> `16ae`, registers a Bot callback at `1738..175e`, and builds the
request at `1786..193a`. For each requested device, membership in the
identityJids HashSet (`1792..179a`) selects an extra attribute:

```xml
<iq id="fixture-query" xmlns="encrypt" type="get" to="s.whatsapp.net">
  <key>
    <user jid="15555550100@s.whatsapp.net"/>
    <user jid="15555550100:3@s.whatsapp.net" reason="identity"/>
  </key>
</iq>
```

Addresses/selectors above are synthetic. `reason="identity"` belongs on the
selected **user**, not the IQ or key (`17b6..17c8`). It does not replace the
normal key query with the separate `encrypt/identity/user` identity-only read.
The native builder independently adds key-level `pqsupport="true"` for PQ
fetching (`1806..1824`) and guarded `context_jid` (`1824..1886`). Ordinary
identity-notification legacy fetching has null context and PQ false, so neither
attribute is emitted. The separate conditionally queued PQ request remains
unported; no fabricated PQ support is sent by this change.

## Callback and retry boundary

[Bot constructor](apk/2.26.37.71-bulk-prekey-refresh/bulk-callback-constructor.txt)
retains the requested devices, B78 scope, PQ-fetch and upgrade booleans. It does
not receive identityJids as a separate adoption-authority list.
[Bot.A03](apk/2.26.37.71-bulk-prekey-refresh/bulk-callback-result.txt) requires a
list, sorts its children with DOB selector 32, reads DeviceJid/error code per
row (`0072..00ae`) and otherwise parses identity, registration, optional type,
key and signed key, optional device-identity and identity_auth. Type defaults
to 5 only on absence; a present type must contain exactly one byte. Identity_auth
includes version, signature and certs; its presence is not proof of verification.

Parsed candidates are handed to `1Dz.A1O` (`03a0..03d8`); requested-device
none/error paths reach `A15` and `A1T` (`03de..04ce`).
[Bot.A01](apk/2.26.37.71-bulk-prekey-refresh/bulk-callback-error.txt) handles the
outer error across requested addresses, deriving B76 with scope and B77 selected
by PQ mode, then calling A1T (`0056..0064`). The [processing continuation](bulk-prekey-processing.md) follows these
forwarding wrappers into admission and session construction. Complete native
verification, pin/session mutation, sorting authority and partial-set semantics
remain **unknown/unported**. Parsing public keys here does not establish that they can safely
replace an existing authenticated pin.

[Job exception policy](apk/2.26.37.71-bulk-prekey-refresh/bulk-job-exception-policy.txt)
returns `!isPqSessionUpgrade` (`001a..0024`). Native job framework retry behavior
remains untraced. Rust's once-only durable reservations and uncertain/refused
state remain unchanged; this callback does not authorize generic retry/reset.

## Implemented request capability

**Implemented/offline tested:**
`messaging::key_request_many_with_identity_changes(recipients, changed, id)`
adds the user reason only for selected devices. Existing `key_request_many`
passes an empty changed set, preserving ordinary requests. Local bounds remain
1..=32 recipients; changed inputs must fit that bound and belong to the requested
set. Fixed errors omit addresses. These subset/budget refusals are local guards,
not demonstrated server limits.

Matching uses existing Rust exact user/server/agent/device correlation across
Plain/AD_JID encodings. PN, LID and individual companion devices remain distinct.
This is **inferred/local policy**, not a complete native DeviceJid.equals port.
The builder performs no store access, queuing, key verification or adoption.

```sh
cargo run -p whatsapp-cli --features experimental-transport -- \
  peer-keys --chat 15555550100@s.whatsapp.net --preview-identity-change
```

The experimental flag invokes `peer_keys::identity_change_preview` before store
opening. Output contains namespace, IQ type, user reason and encoded byte count
with a fixed offline selector; no address or wire body is printed. It conflicts
with `--show`/`--refresh`. It creates no HTTP route, action or transmission.
An actual fresh-binary check with a nonexistent store and absent key environment
returned the four preview fields (62 encoded bytes for the synthetic PN input)
and created no files. Normal queue/inspection semantics remain unchanged.
Preview success is
not evidence that an identity changed or that a server accepted a refresh.

The [fixture](../../crates/whatsapp-api/fixtures/prekeys/2.26.37.71-identity-query.json)
manually transcribes selected/unselected wire attributes from the dispatcher
using synthetic addresses. Tests compare the complete relevant IQ/key/user
attributes, exercise comparison codec round-trip, companion and namespace
mismatch refusals, encoding equivalence, local bounds, redacted preview and CLI
flag conflicts. It is source-derived structure, not executed APK output.

Automatic identity-notification refresh, Cio preparation, callback authentication,
PQ/context handling and native deferred ACK state remain **unported**. The next
step is A1O/A15/A1T and Cio, followed by durable notification/request correlation
and application of the existing explicit identity-change policy. No real account
was opened and no fresh live request was sent for this work.

## Reproduce retained evidence

The [six-topic recipe](apk/recipes/2.26.37.71-bulk-prekey-refresh.json) retains
11 files / 70,703 bytes: job construction/run/exception policy and callback
construction/error/result.
The full dispatcher is reused rather than copied into another bundle.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-bulk-prekey-refresh.json \
  --output docs/protocol/apk/2.26.37.71-bulk-prekey-refresh --check
cargo test -p whatsapp-api --all-features identity_change_
```

Use `apk-research-diff` on retained release bundles for topic/file changes. A
same-bundle self-diff verifies checksums and retention, not release equivalence.

## Current notification fetch flags and manager suppression (2026-11-06)

**Confirmed static APK43.72:** [17 complete manager/gate/address exports with commands and checksums](apk/2.26.43.72-identity-fetch-manager/README.md), continuing [Fc1 case15](apk/2.26.43.72-identity-ack-dispatch/identity-worker.txt). The first14t.A0A and gated second14t.A07 calls both use null context, default E78.A02 scope, internal reason8 and fromIdentityChange=true. The first has fetchPqPrekey=false; second=true. Both have isPqSessionUpgrade=false. The call flags therefore bypass only the manager's early upgrade-specific backoff branch, not every downstream policy. Internal reason8 is not a new wire attribute.

0Fy.A0W requires a map/cache entry, optional metadata objects/fields and flag0x480b; this was not established as an own-account check. Ex8.A06 additionally gates categories and Ex8.A02/0x5e60; effective runtime values remain unknown. No synthetic capability or conditional PQ request was added to Rust.

**Confirmed suppression scope:** manager14t.A00 removes an in-flight entry only when stored uptime+60000 is strictly less than current uptime, during another invocation. Its non-upgrade duplicate key is EoY; constructor/equality/hash include user/category/device plus request mode and session scope. E7N initializer constructs distinct REGULAR/PQ values; fetchPQ chooses between them. Thus the two modes do not collapse into one key in this map. Later BulkGetPreKeyJob queue deduplication/persistence and actual wire transmission are not proved here.

**Negative result:** this is neither an unconditional second network request nor a 60-second retry authorization. Rust's durable uncertain reservations remain blocked and its conditional PQ metadata path remains unported. Native map lifecycle, completion cleanup, job retry/backoff, guarded address normalization and effective configuration require further evidence before any behavior change. No runtime code change, live account operation or APK execution.

## Bulk job completion is not fetched-key acceptance (2026-11-06)

**Confirmed static APK43.72:** [full bulk job/future/parameter classes, runner and callback inventories](apk/2.26.43.72-bulk-key-job-completion/README.md). The fresh Bulk A0L body exactly matches the earlier retry-key-query-wire export. New evidence connects its concrete FfK instance to the existing1ue completion consumer and traces the surrounding job policy; no packet/release difference is claimed.

The job preserves separate fetchPqPrekey and isPqSessionUpgrade flags, generates a new query ID per A0L invocation and registers FfK in0Th.A05 before dispatching Message87. It then removes queued ordinary callback ownership through0ln.A0E and uses untimed FfK.get; that call is not32-second timeout registration. DEX metadata proves FfK implements1ue/Future. BsT(null) releases its latch as normal completion; BsR(exception) causes get to throw ExecutionException. The selected reader result/error fallback reaches0YU message8 and removes the pending map entry before BsT. Therefore an error response can yield job SUCCESS without usable fetched keys; independent Dxh error/admission/backoff processing still matters. Local socket write alone is not this response-fallback completion.

The selected constructor adds ChatConnectionRequirement, defaults attempt budget100 and null groupId, and sets persistence only when upgrade=false. These are constructor/job facts, not100 wire sends, verified queue dedup behavior or a new Rust retry recipe. Bulk A0N returns !upgrade. Runner Qlq increments failed-attempt count, rethrows RuntimeException at01ea, and for other exceptions consults A0N: false gives FAILURE; true checks current requirements and either loops through the budget gate or produces DEFERRED/requeue. Full deferred scheduling, effective requirement transitions and crash recovery remain incomplete.

**Negative boundaries:** normal job completion is not server/key acceptance; FfK cancel returns false and bulk cancellation only logs, so socket cancellation is not established. Native ordinary/conditional PQ fetch exception eligibility is not authorization to replay Rust's durable uncertain request or rotate its ID. No runtime code, automatic retry, request flag, callback timeout or adoption policy changed. Existing callback ownership tests remain the Rust boundary. No live account operation or APK execution.

## Bulk pre-key missing-LID preflight (2026-11-06)

**Confirmed static, APK43.72:** [complete selected methods](apk/2.26.43.72-prekey-lid-preflight/README.md) extend the retained BulkGetPreKeyJob.A0L call to3KB.A01(set,false), before Message87 key dispatch. A01 passes a null timeout to A00. A00 calls A02 first; an empty filtered set returns1xM(set,1) without submitting this sync request. For a nonempty set the bypass condition is `!0AI.A0O() && input_bool`. Bulk passes false, so this particular bypass does not skip its preflight. The meaning of the readiness predicate is not established here.

A02 excludes0x9.A03-positive entries (instance0xC or the delegated A06 predicate), null JIDs and getType()==7. It reads0YJ.A00 metadata and removes returned entries whose0wj.A0A is false; absence from that returned map is not itself shown as an exclusion. Remaining entries are passed to0YG.A05 with diagnostic labels `requestSyncDevicesAndLidIfMissingLids` and `PreKeyFetch`. That method reads0Ws.A0S and returns its map; A02 removes the returned keys from the candidate set. Concrete subtype semantics, metadata authority and delegated predicate/mapping store behavior remain **unknown**; this is not permission to rewrite PN/LID namespaces from unsigned hints.

For nonempty remaining candidates A00 calls0ob.A0A(0vC.A0M,set). A0A selects0vJ.A0h and0vY.A0N and passes both booleans true to A06. A06 asserts the selected mode predicate and scope==0vG.A03, builds0vW, assigns those flags and adds every candidate user before building0vt. A08 creates E0F, attaches it via0vt.A03 and submits0p4.A04. This proves request-object queue submission, **not** an actual network write or the eventual IQ body.

A00 waits using inherited FfK.get() without a timeout for this null-timeout caller (the separate non-null path uses milliseconds). It casts the returned value to1xM; Bulk ignores the return and proceeds only after the call returns normally. Exceptions can instead enter the already retained job-runner handling. This establishes preflight call/wait ordering; it does not establish successful mapping adoption or that every key fetch emits another network request.

**Unknown / unported:** sync queue coalescing, actual wire selector/body and callback lifetime, response refusal semantics, mapping authentication/adoption and effective per-account filters. No new Rust lookup, automatic retry, timeout or namespace conversion is justified by this partial trace. No live operation or ban-prevention guarantee.

## Missing-LID preset and queue combination (2026-11-06)

**Confirmed static:** [full preset/constructor/queue exports and register mapping](apk/2.26.43.72-prekey-lid-queue/README.md) identify the preflight as LID_REQUEST in MESSAGE_MULTI_PROTOCOL_QUERY context. The selected0vY.A0N initially enables LID only; contact and sidelist flags are false. The helper's diagnostic name is not evidence that it requests both devices and LID. Native0p4/1Wa queue handling can combine target sets and protocol flags before execution, so the original object is not a complete final-IQ oracle. The LID flag reaches the executor's lid_sync_backoff block; full producer-specific reachability/clock authority remain unresolved.

**Unknown / unported:** final post-combination wire, effective gates, callback timer, refusal handling and authenticated mapping adoption. The complete large executor is retained for focused follow-up, not claimed fully reconstructed. No runtime or API capability added; no live account operation.

## Missing-LID multi-query route and nested timing (2026-11-06)

**Confirmed static:** [preset-map binding, complete query/future methods and limits](apk/2.26.43.72-prekey-lid-query-route/README.md) bind MESSAGE_MULTI_PROTOCOL_QUERY to the executor's multi-query handler and0p7.A0O. For its selected nonempty constructed-list dispatch,0pD.A07 receives128000ms and A0W separately waits Future.get(128000ms), requiring non-null response state. The enclosing3KB untimed wait therefore does not prove all nested operations lack deadlines. This is neither a universal usync timeout nor a256-second lifetime; distinct timers need distinct start/ownership evidence.

**Unknown / unported:** effective protocol splitting/configuration, target batching, final per-user/wire profile, complete response/mapping adoption and durable wait/retry policy. Native32-second phone queries are separate factories. No Rust deadline changed without an exact producer binding; no live account operation.

## Missing-LID field mapping and MEX selection (2026-11-06)

**Confirmed static:** [complete final dispatcher/MEX builder and retained field-chain references](apk/2.26.43.72-prekey-lid-mex-selection/README.md) close one LID Boolean/mask mapping:3GQ.A04 ->A0O v26 /mask0x100 ->Hck.A0I ->H6w.A0M. The classic writer can emit query/lid from this field under its explicit conditions. But on the traced Gut.A0A==0/Gut.A04==false branch, an unmerged LID-only request chooses HSo.A01 throughA0F, constructing MEX UsyncQuery with include_lid=true. include_devices comes from a separate field. This refutes treating the classic writer alone as this producer's final wire oracle.

**Unknown / unported:** effective configuration and merges, MEX serializer/query identifier, callback/send timeout, response acceptance and authenticated mapping adoption. The earlier128-second direct classic-send argument remains branch-specific; the shared future wait does not establish the MEX send timer. No Rust lookup, MEX request, retry or deadline changed; no live account operation.

## Missing-LID MEX callback completion and error scope (2026-11-06)

**Confirmed static:** [constructor-bound handlers, base delivery gate and observer effects](apk/2.26.43.72-prekey-lid-mex-callback/README.md) separate request errors, scoped protocol/user errors and exceptional MEX failure. A path-length1 request error calls BLg with converted/capped optional backoff then BsT(null), returning false to suppress data continuation. Scoped575/599 errors with sufficiently long paths populate separate maps and can allow data continuation. Generic failures call BsR(Giw). Success constructs Gzg and calls BLh before BsT; concrete0pA.BLh stores0p7.A00, which the existing future-wait helper requires. Future release alone is not response-state installation or mapping acceptance.

**Unknown / unported:** full success parsing/adoption, map collisions, shared response-field clearing/concurrency, authenticated mapping authority, durable waits and MEX envelope/send timeout. Backoff multiplication is native signed-long arithmetic with an upper cap, not proof of safe local deadline authority. No Rust request/retry/reset or live operation added.

## Missing-LID shared response-state resets (2026-11-06)

**Confirmed static:** [exact field/caller census and full reset/cleanup methods](apk/2.26.43.72-prekey-lid-response-lifecycle/README.md) establish29 physical0p7.A00 references. Nonempty A0G follow-up reads clear response/error/maps before dispatch; empty reads return without clearing. A0U clears response/error/maps at executor drained/exception boundaries. Other identified writes install actual Gzg results. This closes the prior untraced existence of reset/cleanup, not a guarantee of clearing before every batch/request.

**Unknown / unported:** effective serialization/aliases, per-batch lifecycle, late callback cancellation and correlation. No real stale-state race or restriction is established. Rust request-owned callback and uncertain-outcome rules remain; no shared response slot, retry or runtime behavior was added. No live operation.

## Missing-LID MEX operation ownership and nested send timeout (2026-11-06)

**Confirmed static:** [factory/default/handle and transport selection exports](apk/2.26.43.72-prekey-lid-mex-operation/README.md) show HSo creates a fresh0iq with32000ms timeout and defaultA03=false, selecting0j3. Its start-and-returnAQY API discards the operation handle; the selected outer wait does not cancel it.0j3 then branches on transport: non-Pando calls0Sf.A0W with the copied32000ms, while Pando dispatches separately. The outer128000ms future wait and the separate classic usync128000ms send argument do not define this MEX transport's universal lifetime.

**Unknown / unported:** effective Pando choice/deadline/cancellation, IQ callback expiry/removal, late callback suppression and mapping adoption. No native race or restriction is proven; no Rust deadline, callback or retry change was made. Offline research only.

## Missing-LID non-Pando MEX expiry outcome (2026-11-06)

**Confirmed static:** [registry expiry/runnable and concrete MEX failure chain](apk/2.26.43.72-prekey-lid-mex-iq-expiry/README.md) link0mb/0md to0mZ.BL6. The selected registry expiry removes ownership under lock before scheduling BsS. That callback creates typed local transport mex-delivery-failure471; its LIM classification reaches the MEX exceptional FfK.BsR path. This differs from request-refusal BsT(null), and does not establish a server471 response or retry permission. Both IQ outer callback methods share a typed body parser; outer result/error alone does not determine final model success.

**Unknown / unported:** full response constructors/adoption, queued/concurrent callback timing, fallback late-reply effects, Pando expiry/cancellation and reconnect authority. Removal-before-delivery is confirmed; universal late-reply suppression is not. No Rust runtime change or live operation.

## Missing-LID UsyncQuery server ID (2026-11-06)

**Confirmed static APK43.72:** [exact asset, selected entry and constructor/registry/loader chain](apk/2.26.43.72-usync-mex-query-id/README.md) resolve UsyncQuery in whatsapp-android-mex throughCRh to packaged server doc_id38635300486116501. The longer client_doc_id407762649418300062941108847678 is separate. Exact decimal preservation matters because the server ID exceeds2^53. This narrows the prior query-identifier gap for the selected non-Pando resolver; fallback/unregistered config returns no usable server ID.

**Unknown / unported:** effective runtime registry/Pando choice, remote acceptance, complete variables/gates and authenticated mapping adoption. No Rust MEX lookup/preflight or live operation added; this is pinned evidence, not a timeless ID.

## Missing-LID UsyncQuery variables spine (2026-11-06)

**Confirmed static APK43.72:** [target/telemetry/params serialization chain](apk/2.26.43.72-usync-mex-variables/README.md) binds variables.input.query_input[].jid to typed UserJid.getRawString and input.telemetry.context to Locale.US uppercase of the preset context: MESSAGE on this selected route. Requested include_* entries are written true conditionally; skipped branches remain omitted rather than defaulting false. Explicit false is still supported by the general setter. LID and devices retain separate aggregate fields.

**Unknown / unported:** complete per-user cached/profile fields, config/privacy/splitting and merge authority, binary/Pando serializer details, response authentication/adoption and live gates. This is a variable spine, not a full executable request oracle. No Rust lookup/preflight/API or live operation added.

## Missing-LID MEX device cache fields (2026-11-06)

**Confirmed static APK43.72:** [cache getters, field chain and scalar helper](apk/2.26.43.72-usync-mex-device-cache/README.md) show per-user devices cache input requires enabled devices and a nonempty real hash. Positive timestamp and strictly newer expected timestamp are separate gates; supplied values are decimal Strings through String.valueOf(long), not numeric JSON scalars. Hash formatting has a0x415b configuration switch. Initialized null fields are distinct from omitted flags at the intermediate map stage.

**Unknown / unported:** complete hash format/config, cache/alias authority and timestamp units, coherent snapshots, final null/Pando serialization and response adoption. No clock/hash/cache values were invented and no Rust preflight/request or live operation added.

## Missing-LID MEX null serialization platform boundary (2026-11-06)

**Confirmed APK call site and pinned platform source / inferred platform-qualified wire behavior:** [0mF map constructor and unmodified Android16 JSON sources](apk/2.26.43.72-mex-null-serialization/README.md) show JSONObject(Map) recursively preserves stored null entries as explicit JSON nulls. Its separate put(key,null) removal rule must not be applied to this constructor. Thus the selected devices object predicts timestamp:null when cached timestamp<=0, and expected_timestamp:null unless cached>0 and expected>cached; non-null timestamps remain decimal strings. A skipped include_* setter still produces no map entry.

**Unknown / unchanged:** actual handset platform/classloader identity, Pando serialization, full cache/alias authority and response adoption. This is not runtime observation or a universal wire-null requirement. No Rust behavior or live operation changed.
