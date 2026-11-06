# Bulk-key job completion and retry boundary, APK 2.26.43.72

Confirmed static for pinned SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`, code264307238. The metadata script verifies APK digest before reading DEX declarations. No execution of the APK, account traffic or REA Evidence envelope claim. xtask static fallback continues the already diagnosed unavailable Android provider; no repeated setup.

## Reproduce

From repo root, set `APK=../android-lab/apks/WhatsApp.1.apk`:

```sh
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/messaging/jobqueue/job/BulkGetPreKeyJob;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/FfK;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/DCZ;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Qlq;' --method run --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'Lorg/whispersystems/jobqueue/Job;' --method A0M --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'Lorg/whispersystems/jobqueue/JobParameters;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'Lorg/whispersystems/jobqueue/JobResult;' --method '<clinit>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0YU;' --method C0f --signatures --exceptions
target/debug/xtask apk-field-refs "$APK" --class 'LX/0Th;' --field A05 --type 'Ljava/util/Map;'
target/debug/xtask apk-callers "$APK" --class 'LX/0YS;' --method C0f
target/debug/xtask apk-callers "$APK" --class 'Lorg/whispersystems/jobqueue/Job;' --method A0N
python3 docs/protocol/apk/2.26.43.72-bulk-key-job-completion/inspect-future-type.py "$APK"
```

All code exports are complete selected methods/classes, not snippets. Only bulk-key execution and the identified runner blocks are interpreted. The metadata script is the prior reader-observer interface inspector with the seed changed to FfK; Java Future declaration is explicitly external/uninspected. Caller/field inventories are physical DEX sites, not reflective/global runtime proof. checksums.json binds every other retained file.

## Constructor and dispatch

Bulk constructor requires upgrade implies fetchPQ; stores requested/identity-changed JID arrays, manager reason, context raw JID, scope ordinal and separate fetch/upgrade flags. Selected parameter builder sets retryCount100, null groupId, ChatConnectionRequirement; regular (upgrade=false) sets persistence true, upgrade=true leaves false. JobParameters constructor confirms field binding. This is an attempt budget, not100 proven wire sends or absence of later queue deduplication. The helper that adds OfflineProcessingCompletedRequirement is not the one this constructor calls. readObject requires a nonempty target array and validates elements of optional identity-changed arrays and individual JIDs; full restart/injection compatibility is not ported here.

A0L generates a new query ID each invocation001a..0026, parses saved arrays and invokes3KB missing-LID work0090..00a0 before request submission. Context-dependent DqO telemetry is separate from IQ body. It constructs Message87 with id/jids/identityJids/fetchPqPrekey/contextJid/sessionScopeOrdinal/isPqSessionUpgrade at01ec..0280. It allocates concrete FfK, stores it in0Th.A05 under ID0294..02b4, dispatches0Sf.A05, calls0ln.A0E(id) and waits on **untimed** FfK.get at02d6..02dc. A0E's prior retained body removes queued IQ registry ownership; it is not timer registration. This job does not establish a32-second full-key deadline. Other watchdog/lifetime mechanisms are separate.

## Concrete future effects and response path

DEX metadata proves FfK implements1ue and Future. FfK holds a CountDownLatch(1). BsT(value) marks success, saves value and counts down; get() returns it. BsR(exception) marks failure, saves exception and counts down; get() wraps it in ExecutionException. The job uses get() without timeout. A timed overload exists but is not this call. cancel() returns false and isCancelled() returns false; the bulk cancel callback only logs. This does not establish socket/query cancellation.

The four physical0Th.A05 references are constructor, disconnect failureA03, message8 consumer0YU.handleMessage and this BulkGetPreKeyJob. Existing [callback manager](../2.26.43.72-retry-refetch-callback-lifetime/callback-manager.txt) A03 invokes BsR then clears map. Existing [reader-response path](../2.26.43.72-iq-reader-admission/README.md#ordinary-registry-miss-has-a-fallback) is1rA result/error ->1lZ.A1E ->1lY.C0g; ordinary registry miss posts message4,1lY calls0YS.C0f(id), retained0YU.C0f posts message8. The existing [complete message handler](../2.26.43.72-iq-shutdown-eligibility/message-handler.txt) removes that ID from0Th.A05 then calls1ue.BsT(null) at0456..0494. The concrete future implementation now closes that previously untraced callback effect for these bulk jobs. Global0YS provider identity/alternate paths and native concurrency remain separate.

In this selected response path, an error response can release the job future just as a result does. Job normal return maps to SUCCESS; it is not proof of usable fetched keys, server success code or identity adoption. Dxh's independent error/bundle callbacks and downstream backoff/admission remain separate. A local write alone is not this response-fallback completion.

## Exception and retry/defer decision

Bulk A0N(Exception) returns !isPqSessionUpgrade. Sole physical base-method caller Qlq.run invokes A0L while attempt count < retryCount. On caught Exception it increments the count. RuntimeException bypasses A0N and is rethrown at01ea (009e+0xa6*2), subject to outer cleanup; false A0N instead reaches FAILURE at01ec (00aa+0xa1*2). For true A0N, inherited Job.A0M checks every Requirement.BWq: when requirements still hold it loops to A0L (budget checked again); unmet requirements produce DEFERRED and a requeue path. SUCCESS/FAILURE/DEFERRED names are verified from JobResult initializer. Full deferred scheduling/persistence/backoff and requirement implementation are outside this audit.

Thus ordinary and conditional PQ-fetch notification jobs (both upgrade=false) have a retry-eligible exception path; upgrade jobs do not. This is not a command to retry a server refusal: the selected ordinary result/error completion can be a normal successful job return, while disconnect/local future failure is exceptional. It also does not authorize reusing Rust's uncertain durable reservation or rotating its request ID. Native job attempts allocate new IDs and require the full native lifecycle to interpret safely.

## Rust boundary

No runtime code or request policy changes. Existing Rust live-callback ownership, stored request/peer bindings, signature/adoption checks and no-replay reservations remain. Future completion is distinguished from server/key acceptance; native exception eligibility, queue requirements and incomplete disconnect/recovery policy are not copied as an unconditional resend loop. Remaining work includes full provider binding, requirement state transitions, deferred scheduling/job dedup, reader exceptions and effective backoff.

**Verified negative comparison:** the freshly exported Bulk A0L body matches the earlier retry-key-query-wire export exactly. New evidence here closes concrete future effects and constructor/runner policy, not a release change or new packet shape.
