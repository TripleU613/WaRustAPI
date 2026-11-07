**MobileConfig Java/native completion binding now reaches a stored callable.**

Confirmed selected static MobileConfig shared-handle publication at storage slot+0xe8. It replaces manager+0x20 under mutex+0x34 and stores status+0x30 even after a null/false-probe reason; the selected probe checks only root-pointer presence. Hash handling includes an asymmetric candidate-prefix comparison. Shared-handle replacement is now located; downstream semantic admission and durability remain unresolved. [Handle publication evidence](apk/2.26.43.72-mobileconfig-handle-publication/README.md). No live operation or Rust runtime change.

Confirmed selected static cross-member binding resolves MobileConfig default InlineSerialExecutor dispatch to a four-instruction direct callable invocation in pinned libwhatsappmerged.so. The captured parameter-map task runs during that selected normal dispatch, without a queue in the slot method. Supplied overrides, runtime binding, file success and active configuration installation remain separate. [Default executor evidence](apk/2.26.43.72-mobileconfig-inline-executor/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig upgrade follow-up schedules parameter-map file persistence, not established active-table installation. It supports direct calls or a captured task; missing mcThreadPool logs and appends reason byte3. Default construction imports InlineSerialExecutor symbols, while actual dispatch/completion remains external. Configuration-owner installation is still unresolved. [Parameter-map scheduling evidence](apk/2.26.43.72-mobileconfig-params-scheduling/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig upgrade has direct writes or temporary-file replacement. Default replacement attempts both renames once; flag-controlled ordering uses up to three local rename attempts with 4/8/16ms failure sleeps. Both pair replacements must succeed, while later cleanup failure only logs. This is filesystem policy, not network retry or proof of atomic/durable active state. [Upgrade replacement evidence](apk/2.26.43.72-mobileconfig-upgrade-replacement/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig verifier root/table/vector/string bounds, initialized counter limits and required late scalar. Optional structural fields can be absent; vector end output can be written before the final span test, so outputs alone do not prove validity. Complete nested schema, installation and durable active-state admission remain unresolved. [Verifier bounds evidence](apk/2.26.43.72-mobileconfig-verifier-bounds/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig reader distinguishes raw buffers from framed input and checks framed length/version/declared size/footer/offset before a separate body verifier. Envelope handoff is not table acceptance. The earlier hash transformation is now resolved to the prefix before the first colon; raw-string checks remain separate. Complete verifier/provider and active-state admission remain unresolved. [Reader framing evidence](apk/2.26.43.72-mobileconfig-reader-framing/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig load continuation reaches upgrade/swap and translation-table build paths. The callers use schema-result bit0 versus a low16>=256 test differently; failed swap upgrades invoke cleanup and UpdateFailed diagnostics. Neither returned internal codes nor global attempt bytes prove active configuration. Reader validation, normalization and installation remain unresolved. [Upgrade dispatch evidence](apk/2.26.43.72-mobileconfig-upgrade-dispatch/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig copyFromBackup callback requires a DONE status gate, skips dot/dotdot/DONE entries and stops further copies after a failed copy-helper result. Pre-cleanup precedes marker checking, and selected recovery codes continue into later loading. A true copy latch does not prove complete or durable restored data; startup callers and load/activation admission remain untraced. [Restore evidence](apk/2.26.43.72-mobileconfig-restore/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig flag1 branch prepares a _backup directory, copies three selected files and creates an empty DONE marker before continuing to the same two-file write path. Backup failure codes log but do not stop that selected normal continuation. Copy-helper true does not prove complete bytes: write/close results are unchecked. Restoration, durability and activation remain untraced. [Backup evidence](apk/2.26.43.72-mobileconfig-backup/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig writer framing, per-write stream checks and post-close result gate. The alternate storage path writes two files in order and attempts removal of the first path if the second write fails; this does not establish atomic replacement, complete rollback or crash durability. Full activation and the other storage branch remain untraced. [File writer evidence](apk/2.26.43.72-mobileconfig-file-writer/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig hash-state gates produce internal UnknownQueryHash/UnknownOneQueryHash results 4/5 from presence and empty-string conditions, not hash-equality comparisons. Optional one-hash output omits the later support check. The remaining processing path reaches storage virtual slot +0xb8, bound through the selected vtable to 0x8674c; full write ordering, durability, activation and retries remain untraced. [Hash gate and storage handoff evidence](apk/2.26.43.72-mobileconfig-hash-gates/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig consumer gates now connect parser diagnostics and missing RequestData to internal result byte 10 (Failed to parse delta response). Invalid root kinds set a diagnostic; a separate decoded-flag condition returns byte 13. The remaining path hands ownership to 0x8d670; persistence, activation, retries and account ownership remain unproven. [Consumer gate evidence](apk/2.26.43.72-mobileconfig-consumer-gates/README.md). No live operation or Rust runtime change.

Confirmed selected static storage response preconditions distinguish unavailable RequestData, zero-length JSON and the selector2/context5 gate with internal result bytes2/3/12. Availability can rely on source-pointer presence alone. Later parser-null/wrong-tag paths continue to further processing, so parse result and final rejection are not equated. Validation, persistence and account ownership remain unresolved. [Storage precondition evidence](apk/2.26.43.72-mobileconfig-storage-preconditions/README.md). No live operation or Rust runtime change.

Pinned construction/RTTI binds the selected updater receiver chain through FBMobileConfigManager+0x88 to FBMobileConfigStorageManager. Its vtable+0x10 resolves to 0x82c4c, replacing the earlier unbound manager-slot candidate for this route. Later response processing, replacement, validation, persistence and account ownership remain unresolved. [Storage receiver evidence](apk/2.26.43.72-mobileconfig-storage-receiver/README.md). No live operation or Rust runtime change.

Pinned RTTI identifies the previously traced constructor object as FBMobileConfigRequestData. Its retained records/deferred fields do not identify the updater response receiver. Construction narrows that receiver to an enclosing owner field+0x88, still untyped; a same-slot manager candidate is not a proved call target. Response parsing/adoption and persistence remain unresolved. [RequestData type evidence](apk/2.26.43.72-mobileconfig-request-data-type/README.md). No live operation or Rust runtime change.

Confirmed selected static loader callers retain parse diagnostics while continuing to finalization/accessor handoff. The shared record pair reaches a configuration constructor that retains ownership at state+0/object+0x258 and seeds deferred bytes. Initial policy/cache bytes are canonical on this constructor path; outer admission, later writers, account ownership and persistence remain unresolved. [Loader handoff evidence](apk/2.26.43.72-mobileconfig-loader-handoff/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig record finalization has guarded map-order move-append and group-ordinal existing/new copy-merge paths. The merge compares signed parameter words, retains pending equal words and publishes a shared vector to the earlier accessor destination. Query ordering is conditional on these stages; map comparators, caller admission, ownership and persistence remain unresolved. [Record finalization evidence](apk/2.26.43.72-mobileconfig-record-finalization/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig fallback text loader constructs the 0x50-byte record field layout used by grouping, selection and checksums, including separate group/wire IDs, per-type slots and flags. Lookup keys use accumulated parameter OR config-ID-shift12; temporary strings transfer into map records. Completion byte and diagnostics remain distinct. Real input, final vector conversion, merges and persistence remain unresolved. [Text record evidence](apk/2.26.43.72-mobileconfig-text-records/README.md). No live operation or Rust runtime change.

Confirmed selected static native parameter-map reader bounds header values and requires exact count8*8 body consumption plus EOF/error-state checks. It does not enforce the Java helper's v4-prefix/declared-length equality within this selected native header routine. Pinned assets satisfy qualified byte projections; synthetic variants preserve the separate gates. Whole-loader routing, schema-record creation and persistence remain unresolved. [Native parameter reader evidence](apk/2.26.43.72-mobileconfig-native-param-reader/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig file-content branch uses YAJL through a strlen-based adapter, then passes the root to a distinct existing-record metadata consumer. Null/non-tag4 roots return false, but the selected caller overwrites rather than checks that boolean. Parser success, metadata validity and configuration adoption remain separate; decoder routing, mutation and persistence are unresolved. [JSON adapter evidence](apk/2.26.43.72-mobileconfig-json-adapter/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig deferred source refresh loads from configuration+0x130, supports local/shared-loader cache branches, reaches a file-stream open and publishes a shared record vector. Native filename classification and pinned APK parameter-map names are separately verified; runtime path selection and decoder binding remain unresolved. Shared ownership narrows source-lifetime uncertainty but does not prove replacement or account safety. [Record source evidence](apk/2.26.43.72-mobileconfig-record-source/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig deferred table builder groups adjacent existing records by word+0x34 into encounter-ordered ranges, distinct from wire-token word+0x4c. A separate zero-initialized group-ID map stores sequential table indices; zero alone is not proof of a missing ID. Actual record population, alternate factory, ownership and persistence remain unresolved. [Schema grouping evidence](apk/2.26.43.72-mobileconfig-schema-grouping/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig refresh builds paired schema selections through an ascending filtered table walk. Mode0 leaves side0 empty; modes1/2 partition by a concrete special-label predicate or all-record byte flag. Version/skip/empty gates and selected-record counting are retained with a synthetic matrix. Actual schema construction, field producers, synchronization and persistence remain unresolved. [Schema selection evidence](apk/2.26.43.72-mobileconfig-schema-selection/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig schema fragments retain selected-index order, numeric/name fallback boundaries and literal array wrapping. param_list_checksum uses ordered 32-bit recurrence with multiplier397 and per-record term P+31*C+16337, optional first-record group filtering and conditional second-container XOR. Synthetic projections are analyst-derived only; real schema inputs, other hashes and persistence remain unresolved. [Schema fragment evidence](apk/2.26.43.72-mobileconfig-schema-fragments/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig query producer returns an empty vector for empty deviceId or, on the selected non-1 mode/schema route, absent schema containers. Native metadata has distinct presence rules for global_value_hash, one_query_hash, request_data_query_hash and positive ep_refresh_id; bool_opt_policy combines raw configuration flags. Mode 1 still bypasses the caller empty-vector refusal. Values, remaining algorithms, field producers and persistence remain unresolved. [Query setup evidence](apk/2.26.43.72-mobileconfig-query-setup/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig default updater obtains FBMobileConfigRequester; it copies the queue callback into fetch-interface x4, matching the Java bridge when that fetcher is installed. A non-1 request mode with an empty native query vector completes locally with false/Empty query params and bypasses the fetch call. Runtime fetcher injection, field producers and configuration persistence remain unresolved. [Requester dispatch evidence](apk/2.26.43.72-mobileconfig-requester-dispatch/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig queueRequest callback locks a weak owner before forwarding bool/string to virtual completion slot +0x38; the default-updater slot resolves to 0x7e5ac. Expired-owner processing is conditional, with downstream effects still untraced. The intermediate dispatch linking this candidate to the Java fetch handler remains unresolved. [Callback evidence](apk/2.26.43.72-mobileconfig-updater-callback/README.md). No live operation or Rust runtime change.

Confirmed selected static MobileConfig fetch construction copies a caller-supplied callable into a native handler holder; its target position matches the completion load at holder+0x30. Concrete captures, commit and cache activation remain unresolved. [Construction evidence](apk/2.26.43.72-mobileconfig-handler-construction/README.md). No live operation or Rust runtime change.
Verified member relocations bindonCompletion to59574; argument conversion reaches
an indirect callback at59748. Missing callable throws; persistence remains unproven.
[REA observations, ELF proof and next boundary](apk/2.26.43.72-mobileconfig-native-completion/README.md). No runtime request or account operation.

**MobileConfig list/null setter ambiguity is narrowed.**
queries becomes an empty list; query_hash/query_string retain null. Reused pinned
Android JSON evidence predicts[]/null/null on the selected non-Pando route.
[Source, synthetic shape and platform/Pando limits](apk/2.26.43.72-mobileconfig-null-variables/README.md). No request enabled.

**MobileConfig MEX errors now have a traced failure handoff.**
Selected worker forwards false plus a ranked message; no explicit retry or refresh
write occurs in that Java branch. Summary/critical/exception ranking is retained.
[Exact selectors, excerpts and native-policy limits](apk/2.26.43.72-mobileconfig-fetch-error/README.md). No runtime request enabled.

**MobileConfig fetch is a distinct unported network/SDK pipeline.**
Current APK MEX/WWW routing, selected input setters, persisted IDs and MEX callback
ordering are retained; AB-property inspection is not effective table hydration.
[Network evidence and native/error boundaries](apk/2.26.43.72-mobileconfig-fetch-route/README.md). No request or runtime policy enabled.

**Token parameter-map bodies and selected translation entries are decoded.**
All four packaged pair arrays validate; compiled DEX config keys locate the four
token rows, whose packaged translated slots match their original specifiers.
[Derivation and dynamic-policy limits](apk/2.26.43.72-token-param-body/README.md). No runtime policy change or live operation.

**Token parameter-map v4 headers and schema gating are now decoded.**
Four pinned packaged universes reproduce bounded header/hash metadata; the
lightweight-manager path requires schema compatibility before translation install.
[Methods, artifact identities and undecoded-body limits](apk/2.26.43.72-token-param-header/README.md). No runtime policy change or live operation.

**Token configuration cache refresh and file framing are now separated.**
Retained manager handles, latest/session cache transitions and header validation;
twelve synthetic Java vectors preserve signed-byte and cursor behavior.
[Evidence and remaining effective-policy boundary](apk/2.26.43.72-token-config-refresh/README.md). No runtime policy change or live operation.

**Token SDK value reads now reach concrete Java table handling.**
Nv3/0A3 override the bodyless base; context construction/cache, little-endian
long/metadata reads and source/fallback branches are retained with derived cases.
[Methods and remaining hydration/selection limits](apk/2.26.43.72-token-sdk-value-reader/README.md). No runtime policy change or live operation.

**Token bucket defaults now have two complete static derivations.**
SDK09n switch defaults and final legacy08Y map both resolve865/996=604800,
909/997=4. Override/silent-push/source dispatch remains separate from defaults.
[Retained methods, proofs and remaining boundary](apk/2.26.43.72-token-sdk-defaults/README.md). No runtime policy change or live operation.

**Trusted-contact token schedules now retain account ownership.**
Recent old-account cache state can no longer silently suppress a new scheduling
decision; preparation and successful timestamp updates also check the cache owner.
[Regression evidence and policy/legacy limits](token-schedule-account-ownership.md). No live operation.

**Normal outbox message ACKs now retain reservation account ownership.**
Direct and group ciphertext reservation freeze the original owner; successful
and failed ACKs refuse drift before outbox facts or dependent work can change.
[Encrypted regression evidence and legacy limits](message-ack-account-ownership.md). No live operation.

**Sender-key distribution ACKs now retain reservation account ownership.**
Owner drift refuses before consuming the pending ACK or marking devices delivered;
ordinary successful group ACK settlement shares that transaction with outbox facts.
[Regression evidence and legacy/ordinary-ACK limits](distribution-ack-account-ownership.md). No live operation.

**Normal group preparation now retains its original account.** Timeout/save,
device/key-query creation and final sender-key reservation reject owner drift.
Child key batches receive the original parent authority even on first creation.
[Regression evidence and remaining limits](group-preparation-account-ownership.md). No live operation.

2026-11-06 — **Bound upload result adoption and new cache rows to their owner.**

**New directory key batches retain account ownership.** Page reservation,
settlement and assembled-key reuse now reject changed owners, including after
reopen, without rewriting uncertain outcomes. Legacy owner absence stays unknown.
[Evidence and higher-level group limits](key-batch-account-ownership.md). No live operation.

**Normal group writer account gap closed.** Every queued group step now
checks connection account ownership, including final messages without Pending.
Device-query reservation/deadline admission remains an additional guard.
[Evidence, regressions and staging limits](group-writer-account-ownership.md). No live operation.

**Ordinary direct outbox account gap closed.** Query reservation freezes its
owner; reply/suspension/send transactions and normal queued query/message writes
recheck authority. Refusal preserves uncertain state without replay.
[Evidence, regression and legacy limits](direct-outbox-account-ownership.md). No live operation.

**Identity refresh now defers preparation-account drift.** A same-snapshot
read check prevents stale action staging under a replacement account and avoids
turning the final ownership refusal into a recovery error. Final fences remain.
[Evidence and regression](identity-refresh-account-deferral.md). No live operation.

**Group reply projection ownership gap closed.** Saved action/account authority
and decoded request/reply/settlement are reloaded in the final group transaction.
Stale candidates refuse without membership, marker or journal mutation.
[Evidence and regression](group-projection-ownership.md). No live operation.

**Generic action account ownership closed for new rows.** Preparation now
freezes its owner before builder execution; reservation, normal queued writes,
settlement and transactional projection recheck it. Legacy absence stays unknown.
[Evidence and regressions](action-account-ownership.md). No live operation.

**Retry refetch account handoff gap closed.** Frozen account authority now
guards ACK/query encoding and live refusal/session-adoption transactions.
Stale callbacks remain consumed without changing durable job/answer state.
[Evidence and regression coverage](retry-refetch-account-ownership.md). No live operation.

**Device queries now retain account ownership.** A reproduced stale actor
passed writer admission after saved-account replacement. Reservation now freezes
account/store authority, checked before write and transactional reply adoption.
Refusal preserves pending/query/list state across reopen. [Evidence and tests](device-query-account-ownership.md). No live operation.

**AEA account-type inputs bounded.** Exact consumers point to business-profile
data and notifications; they do not establish a required ordinary contact/device
query field. Native classification capability remains unported. [Evidence](aea-account-type-inputs.md). Research only; no runtime change.

**Local AEA cooldown reader traced.** It stabilizes chat classification using
wall-clock time, configured hours and identity reset. Ignored SQL failure returns
limit success claims; no server restriction meaning proven. [Evidence](retry-answers.md#local-aea-cooldown-reader-and-ignored-insert-result-2026-11-07). Research only.

**Later AEA chat storage traced.** Distinct chat classification, transition
guards and optional local cooldown share a guarded transaction. Cache/callback
recovery and whole-retry atomicity remain open. [Evidence](retry-answers.md#post-session-aea-updates-chat-classification-2026-11-07). Research only.

**Strict store-failure scope located.** The traced flag setter wraps migration
bootstrap decryption and restores prior thread context. Persistence exceptions
remain distinct from content failures; ordinary retry linkage is unproven.
[Evidence](retry-answers.md#strict-store-failure-flag-is-migration-scoped-2026-11-07). Research only; no runtime change.

**Session/identity transaction scopes traced.** Native adapters discard save
booleans; local SQLite scopes and conditional IOException propagation are
retained. Shared outer ownership/crash semantics remain open. [Evidence](retry-answers.md#concrete-session-and-identity-transaction-scopes-2026-11-07). Research only.

**Earlier identity AEA write located.** Session processing writes identity-table
AEA before returning, followed by a separate caller update. Update count is
discarded; exception status does not prove rollback. Concrete transaction/crash
semantics remain open. [Evidence](retry-answers.md#session-processing-contains-an-earlier-aea-write-2026-11-07). Research only.

**Receipt-key AEA input and ordering traced.** Native signature-domain bytes,
state enums, result distinctions and post-session state-update call are retained.
Full verification/persistence port remains open; Rust key-adoption refusal stays
in place. [Evidence](retry-answers.md#receipt-key-aea-admission-and-state-ordering-2026-11-07).

**Prepared retry wire integrity closed.** New saved-account retries freeze a
wire digest at preparation; changed bytes cannot become the first reservation
original. Registration-byte substitution is tested across reopen without row
mutation on refusal. Legacy missing-digest limits remain. [Evidence](outgoing-retry-receipts.md#prepared-retry-wire-integrity-before-reservation-2026-11-07).

**Retry preparation ownership gap closed.** New retries with saved account
state bind registration/identity authority at preparation and recheck it before
reservation. Stale snapshots and intervening changes refuse without rewriting
pending wire, including after reopen. [Evidence](outgoing-retry-receipts.md#retry-preparation-account-binding-2026-11-07).

**Stateless retry configuration inputs narrowed.** Selected compiled0/false
entries and zero initial SDK specifiers are reproducible; effective preferences/
readiness and full precedence remain unproven. No raw-AB shortcut or runtime
policy change added. [Evidence](outgoing-retry-receipts.md#stateless-retry-configuration-authority-2026-11-07).

**Minimal retry count reset closed.** Incoming nonzero/invalid counts can no
longer create a constant count1 request. Aggregate0 retains the existing minimal
request; advanced progression/carried keys remain explicitly unported. Durable
state, reopen and once-only behavior are tested. [Evidence](outgoing-retry-receipts.md#incoming-count-cannot-reset-the-minimal-retry-2026-11-07).

**Retry key publication ownership traced.** Native carried prekeys are marked
directly distributed before retry submission; selected key/skey/pq wire mapping
is now bound. Rust integration still needs durable ownership and upload exclusion;
no key emission or mutation added. [Evidence](outgoing-retry-receipts.md#retry-prekeys-direct-distribution-ownership-2026-11-07).

**Stateless/PQ retry inputs traced.** Incoming enc flags feed native named job
fields and conditional key preparation; Rust policy integration remains unproven.
Native also has a minimal branch, so no speculative gate or key emission was
added. [Evidence](outgoing-retry-receipts.md#statelesspq-retry-preparation-inputs-2026-11-07).

**Overwritten missing-body bypass closed.** All immediate enc children now receive
the native null-body admission check before routing/selection. Exact live-location
exception and null-versus-empty distinction are retained. Real ciphertext verifies
refusal without durable state advancement. [Evidence](media-network-outcomes.md#ciphertext-body-presence-before-descriptor-overwrite-2026-11-07).

**Group ciphertext selection corrected.** Receive now uses the last descriptor
per native parser slot, with matching pairwise type/session routing. A corrupted
last packet cannot fall back to an earlier valid packet and earn ACK authority;
reversing the order decrypts once. [Evidence](media-network-outcomes.md#last-group-ciphertext-descriptor-and-consistent-routing-2026-11-07).

**Mixed encrypted-child type bypass closed.** A missing/unknown or unported enc
can no longer be ignored while a valid sibling decrypts and earns ACK authority.
Both child orders are tested with valid ciphertext and unchanged state on refusal.
Native-valid unported types remain explicit coverage gaps. [Evidence](media-network-outcomes.md#mixed-encrypted-child-type-admission-2026-11-07).

**Encrypted child admission corrected.** Exact outer version2 and native ordered
retry-count checks now precede inbox decryption/ACK authority. Valid-ciphertext
regressions cover refusal without sender-key/journal advancement, then successful
corrected decryption. [Evidence](media-network-outcomes.md#encrypted-child-metadata-before-decryption-2026-11-07).

**Retry state restoration and receipt pass traced.** Native retry state can be
restored from saved metadata; selected first-pass receipt permission depends on
secondary-descriptor absence. Later-pass/durability/classification gaps remain;
no runtime response change. [Evidence](media-network-outcomes.md#retry-metadata-and-receipt-pass-selection-2026-11-06).

**Media error response modes narrowed.** Native sendReceipt differs by callback;
isSendRetry can be set before retry-job enqueue, and typed stop paths clear
action permission. Evidence retained; Rust error491 remains unimplemented until
classification and response ownership are proven. [Trace](media-network-outcomes.md#media-error-response-mode-provenance-2026-11-06).

**Media parser error-ACK boundary traced.** Native Dxp reasons flow conditionally
through a distinct mapper/envelope path: common media reasons map to error491
with optional original failure_reason metadata. This is not receipt-parser487 or
a Signal retry cause. Rust integration remains unproven; no wire change.
[Evidence](media-network-outcomes.md#pinned-media-parser-error-ack-boundary-2026-11-06).

**View-once cached MIME bypass closed.** Shared whole-image MIME admission now
precedes view-once descriptor selection, cached prefetch reporting and one-shot
plaintext consumption. Direct flags and all supported wrappers are regression
tested with valid ciphertext and unchanged state on refusal/reopen.
[Evidence](media-network-outcomes.md#view-once-image-mime-before-cache-and-consumption-2026-11-06).

**Image/video whole-MIME preflight corrected.** A shared supported-format check
now gates descriptors and URLs, preventing parameterized/padded visual metadata
from authorizing cached plaintext or a new GET. Image ASCII case works; MP4 remains
exact. Native WebP context and3GPP support remain separately bounded.
[Evidence](media-network-outcomes.md#pinned-visual-whole-mime-preflight-2026-11-06).

**Sticker MIME preflight corrected.** Pinned Di4/FPx core fields and whole-string
WebP comparison confirm parameters/padding must not authorize the supported path.
ASCII case variants now work; Lottie remains excluded before cache/network use.
The new audio oneof remains explicitly outside confirmed complete handling.
[Evidence](media-network-outcomes.md#pinned-sticker-ingress-and-whole-mime-preflight-2026-11-06).

**Current document ingress traced.** Core tags/hash guards match Rust; native
parser permits zero length and stores MIME without the audio-style allowlist.
Rust's nonempty/size/MIME export bounds remain labelled local; zero-byte transfer
parity remains unverified. No runtime change.
[Evidence](media-network-outcomes.md#pinned-current-document-ingress-2026-11-06).

**Opus voice-note preflight corrected.** Pinned audio schema/converters and exact
MIME selector show contradictory codec metadata must not authorize the supported
Opus path. Only canonical Opus PTT is selectable/cache-reusable; regression covers
valid ciphertext, reopen and no reservation.
[Evidence](media-network-outcomes.md#pinned-audio-ingress-and-opus-mime-preflight-2026-11-06).

**Current video ingress checked.** Core hash/key/length/viewOnce tags match Rust;
selected native converters separate ordinary/view-once ownership before common
projection. Sidecar helper checks key presence, not sidecar presence; descriptor
indexes are independently checked offline. Runtime behavior unchanged.
[Evidence](media-network-outcomes.md#pinned-current-video-ingress-2026-11-06).

**Current image protobuf hash ingress confirmed.** Pinned DiL schema/EwA parser
match Rust's core image tags and trace hashes through message setters into the
download model. Sidecar validation is separate; absence is not a universal image
refusal. Runtime behavior unchanged.
[Evidence](media-network-outcomes.md#pinned-current-image-protobuf-ingress-2026-11-06).

**Message-to-download model traced.** Confirmed stored encrypted/plaintext hash
and length projection, register-correct constructor binding and lazy sidecar
preparation. Transfer category depends on message/provider predicates, not MIME
alone. No runtime or request-policy change.
[Evidence](media-network-outcomes.md#pinned-message-to-download-model-2026-11-06).

**Download factories traced.** Confirmed PUSH/PULL enum-to-handler mapping,
config-dependent mode selection, category eligibility and sidecar Callable
behavior. The selected chat branch fails on missing required sidecar rather than
automatically falling back. No runtime policy change.
[Evidence](media-network-outcomes.md#pinned-download-factory-selection-2026-11-06).

**Download range/result consumers traced.** Pinned MuY/MuZ wrappers lead to
stream/window handlers; MuY packages a chunk-MAC decrypter separately from HTTP
status. Normal finalization checks transfer/plaintext hashes and cancellation.
No universal206/416 success rule or Rust status change is established.
[Evidence](media-network-outcomes.md#pinned-download-range-and-result-consumers-2026-11-06).

**Pinned download HTTP leaves audited.** Confirmed identity and conditional Range
construction, distinct native timeouts,200/206 leaf acceptance and416 special
handling. Rust remains200-only pending operation/caller binding; no guessed
header or live traffic. [Evidence](media-network-outcomes.md#pinned-shared-download-http-leaves-2026-11-06).

**Ordinary media source binding.** Reservation and successful ciphertext adoption
now atomically compare the selected inbox bytes; plaintext handoff rechecks the
eligible source. Refusal observations remain attempt-owned. Offline regression
covers replacement/removal/metadata changes and reopen. Native lifecycle remains
unverified. [Details](media-network-outcomes.md#ordinary-download-source-binding-2026-11-06).
New encrypted phone/generation metadata gates final uploaded promotion and cache
reuse. Revoked/replaced accounts cannot adopt late payloads, while HTTP evidence
remains durable. Tests cover owner/metadata/stale-row faults, every-table/journal
preservation, reopen and public cross-phone cache refusal. Legacy stays unbound.
[Format, tested guarantees and remaining lifecycle scope](media-network-outcomes.md#upload-response-adoption-and-prospective-cache-owner-2026-11-06). No live operation.

2026-11-06 — **Fenced media upload reservation with current route/account state.**
After preparation, expiry/host/auth/registration/phone/generation are rechecked
transactionally. Seed and pending upload commit together; failed checks/builds
leave neither new state nor a POST. Tests cover every-table/journal preservation,
reopen, duplicate refusal and write-local coordinator/corruption visibility.
[Evidence, local policy and remaining in-flight boundaries](media-network-outcomes.md#atomic-upload-routeaccount-fence-at-reservation-2026-11-06). No live operation.

2026-11-06 — **Separated media refresh deadlines from hard auth validity.**
Exact field/method tracing closes120s refresh trigger,60s timer and request-aware
20s wait inputs, with distinct elapsed/uptime clocks and old-auth comparison.
Selected upload paths have additional feature/offline/pre-transfer gates; neither
120s nor route TTL is proven a universal hard upload cutoff. Research-only chunk.
[Complete evidence and unported runtime boundaries](apk/2.26.43.72-media-deadline-consumers/README.md). No live account operation.

2026-11-06 — **Stopped saturated media lifetimes from extending auth eligibility.**
Upload route selection now wraps each seconds-to-ms product before applying its
existing age/minimum guard, matching the pinned constructor operation. Tests cover
negative/zero/short/usable positive wrapped durations for both fields, with all
store tables/journal unchanged across reopen. No automatic refresh or HTTP call.
[Evidence and remaining clock/deadline scope](media-network-outcomes.md#wrapped-lifetime-conversion-before-local-route-eligibility-2026-11-06).

2026-11-06 — **Unified retry number admission and required-field ordering.**
Java-reference BMP counts/times now pass the shared parser and cannot hide a later
missing required field. Independent host Java178-case fixture checks int/long
admission and error order; the saved-session resend proves Unicode/ASCII count1
share one reservation. Non-ASCII failures retain conservative error-ACK limits.
[APK calls, Android contract, host-reference scope and reproduction](retry-parser-admission.md#shared-java-reference-numeric-admission-and-later-error-ordering-2026-11-06). No live account operation.

2026-11-06 — **Shared native-null factory rules with receipt contexts.**
Unknown bare/domain and empty optional participant text now follows the retained
nullable factory rather than blocking retry487/completion ACKs. Typed values and
recognized bare objects remain distinct. Count2 saved-session query/refetch tests
exercise the corrected omission with existing ACK/state/identity safeguards.
[Evidence, cache boundaries and corrected expectations](receipt-optional-addresses.md#shared-definite-null-text-factorycache-boundary-2026-11-06). No live account operation.

2026-11-06 — **Preflighted retry ACK construction/encoding before direct work.**
Eligible direct retries cannot reserve a resend/query or advance a session when
their confirmation is unsupported or unencodable. Both answer entrypoints share
the guard. Known-null optional participants are omitted; valid refetch still
works. Tests preserve every retry-state row on rejected confirmation contexts.
[Evidence and ordering boundaries](retry-ack-dispatch.md#ack-capability-preflight-before-direct-retry-work-2026-11-06). No live account operation.

2026-11-06 — **Selected the correct requesting device for retry receipts.**
Direct retries now use the root sender; group/broadcast use the participant,
matching the native original-chat branch. Known-null text addresses cannot gain
retry authority through generic parsing. Tests cover misleading participants,
real saved-session resend/duplicates and unchanged state for invalid sources.
[Exact branch evidence and remaining limits](apk/2.26.43.72-retry-device-source/README.md). No live account operation.

2026-11-06 — **Blocked group facts from constructor-null incoming addresses.**
Group source/actor selection now preserves native text null before matching saved
rows. A root-source guard also precedes aggregate expansion. Tests seed matching
historical targets/chats and verify no facts or changes to any table/journal,
including reopen/redelivery; stored rows and typed carriers remain unchanged.
[Evidence and remaining boundaries](group-receipts.md#constructor-null-incoming-addresses-cannot-match-saved-rows-2026-11-06). No live account operation.

2026-11-06 — **Kept failed non-user receipt address construction null.**
Receipt success/copied/fresh ACKs now omit invalid group/broadcast/newsletter/
temp/call participants instead of resurrecting them through the generic parser.
Direct facts use the same null result; typed objects and valid writer refusals
remain distinct. Tests cover codec/facts and receive capture/redelivery/no-replay.
[Evidence and limits](receipt-optional-addresses.md#non-user-text-constructor-failures-in-receipts-2026-11-06). No live account operation.

2026-11-06 — **Added copied-context487 for supported unhandled receipt roots.**
Response selection follows the upstream router and preserves original recipient/
edit instead of using fresh parse-error context. Special types and early message_id
routes remain distinct. Receive-loop tests cover capture ordering, explicit
redelivery and write-cut/reopen without replay; broader receipt parity remains open.
[Evidence, field semantics and local limits](apk/2.26.43.72-unhandled-receipt-root/README.md). No live account operation.

2026-11-06 — **Restored proven peer-retry schema/version error responses.**
The peer handler calls the same retry parser before its separate responder.
Shared error classification now permits peer fresh/copied487 responses while
ordinary success ACK, resend and key-refetch remain excluded. Leading aggregate
precedence remains. Full peer success handling is still unported.
[Complete handler, census and corrected assumption](apk/2.26.43.72-peer-retry-handler/README.md). No live account operation.

2026-11-06 — **Stopped key-aggregate validation from blocking valid retry routes.**
A leading participants child without message_id selects key aggregation only for
the native ordinary status set. Retry/special/unknown roots now retain their own
consumer policy. Tests preserve ordinary missing-key errors and exercise a real
saved-session retry through validation, recording and durable response handling.
[Evidence and remaining boundaries](apk/2.26.43.72-retry-upstream-routing/README.md). No live account operation.

2026-11-06 — **Stopped ordinary retry processing on two alternate native routes.**
Leading participants/message_id takes precedence over type=retry; category=peer
selects a separate handler. Shared routing now prevents a false ordinary487,
success ACK, resend or key query on those inputs. Saved-session tests preserve
outbox, checkpoint, ratchet and identity state. Full router parity remains open.
[Evidence, registration and negative findings](apk/2.26.43.72-retry-upstream-routing/README.md). No live account operation.

2026-11-06 — **Located the upstream receipt router and unknown-type context split.**
The inherited generic route dispatches to1rU.A05, whose ordered child/type branches
precede ordinary F10 parsing. An admitted unhandled root uses copied-context487,
not fresh parse-error487; a leading participants/message_id can select ordinary
processing first. No runtime behavior changed; the retained routing matrix now
guides the next response correction.
[Complete adapter/sets, dispatch evidence, negative results and remaining boundaries](apk/2.26.43.72-receipt-router-boundary/README.md). No live account operation.

2026-11-06 — **Added missing aggregate-key error487 with local-ID boundaries.**
Only proved required key absence in the supported key branch gains a native error
cause and fresh ROOT-ID response. Empty/overlong inner IDs remain local refusals,
with tests preventing NACK authority or root-fact fallback. The two aggregate
branches have distinct chat/ID read order, retained with exact evidence.
[Required-ID/constructor evidence, regressions and remaining scope](apk/2.26.43.72-receipt-aggregate-id-boundaries/README.md). No live account operation.

2026-11-06 — **Corrected offline receipt timestamp/status error precedence.**
Required item time is parsed before unknown status conversion, matching the APK.
An unknown kind no longer masks an earlier timestamp487 cause. Valid-time unknown
kinds remain outside487; native queued500 versus legacy rethrow is retained as a
separate unported policy rather than guessed into the wire behavior.
[Read-order/exception evidence, failing-before regression and boundaries](apk/2.26.43.72-receipt-item-status-order/README.md). No live account operation.

2026-11-06 — **Verified lenient ordinary root timestamp handling in the current APK.**
The root parser defaults missing/empty/malformed/overflow time to0 instead of
throwing aggregate-style parse errors. Existing runtime policy is retained;
a new codec/store regression prevents blanket487 NACKs and fabricated supplied
zero times while preserving read facts and duplicate/reopen behavior.
[Current parser evidence, negative hypothesis, regression and limits](apk/2.26.43.72-receipt-root-time-fallback/README.md). No live account operation.

2026-11-06 — **Added evidenced aggregate timestamp error487 responses.**
Missing required item time and invalid supported signed-long text now carry a
native timestamp cause shared by the receive/NACK paths. Key-item absence still
defaults; valid signed/BMP/extreme integers and opaque local values stay distinct.
A delivery-only key-routing hypothesis was refuted by the complete branch join;
existing read/played key semantics remain unchanged.
[Parser/error evidence, regression checks, negative branch result and open scope](apk/2.26.43.72-receipt-timestamp-errors/README.md). No live account operation.

2026-11-06 — **Implemented ordinary required-device parse-error ACKs.**
Proven native required-actor failures now get a fresh root error487 response after
durable capture in the normal actor and selected CLI receive windows. Local limits,
unverified binary failures and storage errors remain separate. Recipient-bearing
offline aggregates no longer bypass this grammar check; no facts or retry work
are authorized by a NACK. Synthetic cut/reopen/redelivery checks retain no-replay.
[Cause/output contract, retained native evidence, verification and open scope](retry-ack-dispatch.md#ordinary-required-device-error-responses-implemented-2026-11-06). No live account operation.

2026-11-06 — **Traced ordinary receipt error487 dispatch and corrected an evidence error.**
Both selected native wrapper paths reach the parsing-error handler for LIL.
A fresh export refuted the old claim that A07(true) returns without sending:
it logs then falls through to the same Message76 path. The prior README and
checksum now explicitly record the correction. Rust runtime remains unchanged;
ordinary error response classification is the next implementation gap.
[Complete handler/wrapper/error evidence, byte comparisons and limits](apk/2.26.43.72-ordinary-receipt-parse-error-dispatch/README.md). No live account operation.

2026-11-06 — **Refused malformed required actors in offline aggregates.**
The message_id branch now distinguishes required DeviceJid failure from nullable
key-aggregate skipping. Known constructor-null, wrong-class and bounded binary
actor failures refuse ordinary processing before any batch facts are written.
[Required accessor/conversion evidence, atomic refusal tests and remaining response-policy gap](apk/2.26.43.72-offline-required-receipt-actor/README.md). No live account operation.

2026-11-06 — **Preserved valid aggregate receipts around native-null actors.**
Key aggregates now skip known constructor-null participant text before parsing
its timestamp, matching the pinned parser branch. A malformed time on such an
ignored item no longer discards valid neighbors. Offline message_id aggregates
and nonnull conversion failures remain separate.
[Branch evidence, mixed-row persistence regression and limits](receipt-canonicalizer.md#constructor-null-aggregate-actors-skip-before-timestamp-parsing-2026-11-06). No live account operation.

2026-11-06 — **Aligned direct receipt fact handling for native-null participants.**
Known constructor-null PN/LID/hosted text now follows the existing supported
sender-device fact fallback rather than becoming a false supplied target.
Valid participants retain original-target checks. Current APK parser/event/worker
traces and encrypted-store regressions bind the distinction and remaining limits.
[Evidence, persistence checks and open branches](apk/2.26.43.72-receipt-null-fact-actor/README.md). No live account operation.

2026-11-06 — **Distinguished null receipt participants from unwritable objects.**
Receipt ACK output now omits known constructor-null text participants rather than
resurrecting them through the broader generic Jid parser. Nonnull unwritable
objects still refuse the ACK. Native context construction accepts nullable fields;
concrete handler admission and this client's supported sender requirement remain distinct.
[Context/constructor proof, regressions and boundaries](apk/2.26.43.72-ack-nullable-context/README.md). No live account operation.

2026-11-06 — **Guarded original device getter range before ACK output.**
The pinned writer rejects AD device getters outside 0..255 despite constructor
byte narrowing. Selected unwritable ACK fields are now refused; valid negative
PN devices use the native pair branch. Recipient omission remains distinct.
[Complete byte-writer evidence, regressions and limits](apk/2.26.43.72-jid-device-byte-range/README.md). No live account operation.

2026-11-06 — **Preserved literal zero-device LID ACK wire class.**
Source-aware ACK output now keeps literal DeviceLid0 as AD-JID while decoded
binary zero-device users still reply with JID_PAIR. Receipt and shared notification
builders have offline kind/byte regressions, including signed and Unicode zero.
Incoming captures and existing persisted selectors remain unchanged.
[Evidence, boundaries and remaining gaps](receipt-primary-ack.md#literal-zero-device-ack-wire-class-2026-11-06). No live account operation.

2026-11-06 — **Guarded selected automatic ACK writer compliance.**
Notification and receipt ACKs no longer copy known noncompliant native address
objects. Omitted recipients and fresh retry contexts remain distinct from copied
fields; valid service-backed devices are not rejected by name.
[Current declaration census, class/writer evidence and codec regressions](apk/2.26.43.72-jid-write-compliance/README.md).
General grammar/recovery remain open; no live account operation.

2026-11-06 — **Bound bare self-user pairs to native ACK and receipt semantics.**
Two native symbolic pair forms now decode with the general server grammar intact.
Self message ACKs remain uncertain; binary bare call sender restrictions match
text. Unsupported self receipt metadata cannot create ordinary facts or ACKs,
and unwritable self notification routes remain refused before work/write.
[Codec, conversion, writer evidence and state/actor regressions](message-ack.md#bare-self-user-pairs-and-carrier-independent-guards-2026-11-06).
Qualified aliases/other classes remain open; no live account operation.

2026-11-06 — **Separated binary ACK construction from optional casts and outcomes.**
Known invalid binary participant/recipient fields can no longer settle through
ordinary refusal or from fallback. Hosted binary participants now retain the
ordinary-success exclusion; valid device/non-user recipient casts and special
participant classes remain distinct from nullable malformed text.
[Current evidence, expected-chat/outbox regressions and limits](message-ack.md#binary-optional-address-construction-and-participant-class-2026-11-06).
No live account operation; other families/decoder paths remain under audit.

2026-11-06 — **Validated binary ACK from before projection and recipient selection.**
Hosted binary sources now project to PN/LID; invalid binary device sources cannot
settle through primary normalization or a valid recipient. Valid source/recipient
precedence and original outcome/journal/no-resend guards remain.
[Current constructor/order evidence and full binary matrix](message-ack.md#binary-from-construction-precedes-chat-projection-2026-11-06).
Other decoder fields/families remain an explicit audit; no live account operation.

2026-11-06 — **Made native long-PN ACK admission carrier-independent.**
A matching native-valid20-digit PN recipient now correlates in binary primary/
hosted pairs and AD0 as it already did in text. Constructor-invalid long users
remain unsettled; LID's separate bound and no-resend/reopen guards remain.
[Current constructor/reader evidence and regression matrix](message-ack.md#long-phone-user-recipient-admission-is-independent-of-carrier-2026-11-06).
No live account operation or outgoing target-builder change.

2026-11-06 — **Preserved binary user-recipient precedence.**
Message ACKs no longer ignore a binary zero-device user recipient and settle
through an unrelated from. Hosted binary recipients project to their underlying
PN/LID user; receipt ACK output now uses that native user address as well.
[Current reader/factory/writer evidence and outbox regressions](receipt-user-recipient.md#hosted-user-output-and-binary-recipient-precedence-2026-11-06).
Success/refusal, mismatch, deduplication and reopen/no-resend controls retained.
No live account operation.

2026-11-06 — **Corrected receipt recipient text-class admission.**
Dot-qualified zero/ignored suffixes and invalid PN/LID users no longer become
primary-user recipient qualifiers through generic parsing. ACKs omit the field;
ordinary matching facts retain original actor/attempt/uncertainty and deduplicate
across reopen. Binary-zero users and strict receipt ACK settlement remain.
[Current class evidence, regressions and limits](receipt-user-recipient.md#current-text-class-admission-including-dot-qualifiers-2026-11-06).
No live account operation.

2026-11-06 — **Closed raw-route gaps in remaining notification ACK builders.**
Device, prekey-count, dirty-group and account-privacy envelope readers now share
empty-user text refusal. Malformed text cannot gain local server settings
admission or queue dirty-group refreshes. Absent-source count ACKs remain valid.
[Evidence, state/receive regressions and remaining limits](notification-ack-audit.md#remaining-ack-builders-preserve-raw-route-refusal-2026-11-06).
Updated the ACK inventory to include existing picture/account-privacy paths.
No live account operation.

2026-11-06 — **Closed separate notification address-reader gaps.**
Identity ACK participant routing now shares malformed empty-user text refusal.
Privacy-token sender/consumed alias readers distinguish explicit .0/:0 text
from primary user authority, preserving binary zero-device handling. Receive
continuation, token/state preservation and original-query recovery are covered.
[Evidence, regressions and native-policy limits](notification-ack-audit.md#separate-identity-and-privacy-token-address-readers-2026-11-06).
No live account operation.

2026-11-06 — **Refused malformed empty-user notification ACK routes.**
Shared ACK handling no longer repairs text such as @s.whatsapp.net into an
outbound server destination; both sender and participant are guarded. Bare
server and typed device routes remain. Native nullable-address behavior and
Rust's stricter local ID/routing limits are explicitly distinguished.
[Current evidence, regression and limits](notification-ack-audit.md#shared-notification-text-routing-guard-2026-11-06).
No live account operation.

2026-11-06 — **Bound PSA watchdog and wakeup completion semantics.**
Linked previously retained diagnostic-only watchdog proof; it does not expire
held ACK ownership. Traced scheduler units/cancellation and wakeup normal versus
Exception completion. Promotion work is not presumed to be a network fetch.
[Evidence and remaining runtime obligations](psa-ack-lifecycle.md#wakeup-completion-and-scheduler-follow-up-2026-11-06).
Runtime policy unchanged; no live operation.

2026-11-06 — **Traced current PSA work-before-ACK obligations.**
Native feature/campaign/revoke branches and one-shot deferred completion exceed
the existing legacy PSA helper. Corrected universal ACK claims; effective flags,
worker outcome and durable recovery remain open. Runtime policy unchanged.
[Branch matrix, retained evidence and next obligations](psa-ack-lifecycle.md).
No live account operation.

2026-11-06 — **Preserved explicit hosted zero-device text qualification.**
Raw hosted/hosted.lid zero qualifiers no longer become plain user metadata;
valid plain/99 and ordinary PN device0 behavior remain. Original text and shared
JID/store/hash behavior are unchanged.
[Native parser evidence and regression](hosted-chatstate-text.md).
No live account operation.

2026-11-06 — **Projected hosted typing addresses through their native user identity.**
Hosted/hosted.lid aliases and device99 now yield PN/LID display metadata while
preserving original addresses. Unsupported explicit device forms stay omitted;
hash/store/outbound identity handling is unchanged.
[Factory evidence, regression and remaining parser gaps](presence.md#hosted-incoming-typing-identities-2026-11-06).
No live account operation.

2026-11-06 — **Corrected incoming PN/LID typing address projection.**
Device-qualified from/participant now unwrap to user metadata; known group and
server-only authors cannot appear as the supplied user. Original frames and
saved identities stay unchanged. Other families remain an explicit open audit.
[Current reader, regression and limits](presence.md#incoming-chatstate-routing-2026-11-06).
No ACK, automatic traffic or live account operation.

2026-11-06 — **Refused unsupported presence/typing targets before construction.**
Empty-user server addresses, agent/device metadata (including explicit AD0/0)
and malformed group contexts no longer pass the supported outbound APIs.
Pinned current writers/type hierarchy and event-driven timer gates retained.
[Evidence and regression](presence.md#current-target-types-and-outbound-guard-2026-11-06).
No traffic, identity normalization or scheduler feature was added.

2026-11-06 — **Corrected report result extensions; resolved native list bounds.**
Valid report results no longer become refusals due to unrelated error extensions.
Current constructors permit empty lists; optional report/id bounds and exact ID
correlation are traced without inventing source or acceptance requirements.
[Evidence, regression and remaining schema gaps](chat-actions.md#report-response-and-list-bounds-2026-11-06).
No live account operation.

2026-11-06 — **Bound individual report callback ownership to native32s.**
Pinned RPC/wrapper evidence supplies32000 to the shared IQ registration path.
Expiry now drops the live claim while preserving uncertain Sent state and no
resend after reopen. Located report/id and current flow-schema limits without
inventing response adoption or optional metadata. No live operation.
[Evidence and boundary regression](chat-actions.md#individual-report-callback-timer-2026-11-06).

2026-11-06 — **Corrected photo lookup result dispatch.**
Unrelated root/nested error extensions no longer turn a matching lookup result
into a refusal. Retained first-named-child and constructor proof; result
payload validation/cache adoption remain unported. No retry or live operation.
[Regression, evidence and limits](chat-actions.md#photo-lookup-result-extensions-2026-11-06).

2026-11-06 — **Corrected photo outer-error selection and retained callback outcome.**
Upload and lookup now select the first direct coded error, matching the pinned
APK, instead of a competing nested error. Optional durable photo_error keeps
signed code and lookup404/410 empty-metadata classification; status remains the
outer IQ Error. No cache mutation, retry or live operation.
[Evidence, regression and limits](chat-actions.md#photo-error-callback-outcomes-2026-11-06).

2026-11-06 — **Held MEX privacy setter intent before notification ACK.**
The pinned mutation serializes as w:mex IQ/get, so the classic-only guard missed
it. Exact direct query_id recognition now preserves the hold across queued,
sent, success and refusal outcomes and store reopen. Getter/body/nested-ID
negatives remain admitted. Native typed ownership remains unported.
[Wire proof and regression](account-privacy-notifications.md#mex-mutation-intent-guard-2026-11-06).
1534 workspace tests passed; no live account operation.

2026-11-06 — **Traced privacy MEX setter input and response admission.**
SetPrivacySettingMutation uses nested enum-valued setting input and a distinct
persisted document. Null-response fallback and enum-translation failure stay
separate; successful maps return to category/pending reconciliation.
[Evidence, SDK limits and Rust boundary](privacy-mex-config.md#privacy-setting-mutation-input-and-response-rules-2026-11-06).
No production request or live account change.

2026-11-06 — **Re-anchored read-self ACK privacy synchronization.**
Current227 handler can disable reads from none, preserves localfalse on all,
and can queue a corrective setter. Producer/message, interface dispatch and
normal enabled provider binding are retained; runtime/recovery gaps stay explicit.
[Current evidence and Rust boundary](read-setting-sync.md#current-build-read-self-ack-setting-synchronization-2026-11-06).
No production or live account change.

2026-11-06 — **Separated optimistic privacy views from confirmation.**
Setter follow-up routes an observer; pending integer overlays and existence
checks are distinct from the stored receipt Boolean. Literal/UI references
are not preference-write proof; listener/writer/recovery gaps remain explicit.
[Evidence and limits](account-privacy-notifications.md#optimistic-privacy-views-are-not-server-confirmation-2026-11-06).
No production or live account change.

2026-11-06 — **Fixed privacy intent release after IQ refusal.**
Native setter failure does not retire its pending category. Rust now preserves
its unowned-setter hold after a real refusal rather than adopting/ACKing a
later notification solely from that error. Saved refusal remains inspectable.
[Evidence, session/reopen regression and remaining ownership](account-privacy-notifications.md#setter-refusal-does-not-retire-pending-category-ownership-2026-11-06).
No action retry, state reset or live account operation.

2026-11-06 — **Fixed privacy duplicate-value validation order.**
Supported readreceipts notifications now select the last raw value before enum
validation, matching the native map producer. An overwritten unsupported text
cannot suppress adoption/ACK; required text/structure and final validation stay.
[Evidence, session ACK-cut test and limits](account-privacy-notifications.md#duplicate-category-values-are-validated-after-selection-2026-11-06).
No live account or automatic refresh operation.

2026-11-06 — **Preserved native signed blocklist error codes.**
Added optional blocklist_error_code to saved outcomes and CLI views; Java
signed32/default0 parsing no longer loses values through the generic u16 field.
Legacy rows remain unknown; request/retry behavior is unchanged.
[Evidence, Unicode/range tests and limits](chat-actions.md#signed-blocklist-error-diagnostics-2026-11-06).
No live account operation.

2026-11-06 — **Fixed blocklist reply dispatch and error attribution.**
Owned outer results cannot be turned into failures by unrelated error
extensions; the separate snapshot validator still applies. Outer errors use
the first direct coded child rather than nested or uncoded errors.
[Regression and native400 reconciliation limits](chat-actions.md#blocklist-outer-result-and-coded-error-settlement-2026-11-06).
No automatic retry, hash reset or live account operation.

2026-11-06 — **Location-disable lifecycle and retry eligibility resolved.**
Persistent job retains its sequence, rechecks sharing on each execution, and
uses ChatConnectionRequirement. Runtime exceptions bypass its retry decision;
other exceptions can retry/defer within the native runner's attempt budget.
[Evidence and remaining recovery limits](chat-actions.md#location-disable-lifecycle-and-retry-boundary-2026-11-06).
No production retry change or live account operation.

2026-11-06 — **Blocklist location-disable wire and stale-job gate traced.**
Existing share state gates cancellation; the queued job rechecks current sharing
before sending notification/location with a saved sequence in disable.id.
Outer request ID, destination mapping and ACK future contract remain distinct.
[Evidence and unported state/recovery](chat-actions.md#blocklist-observer-location-disable-obligation-2026-11-06).
No production protocol or live account change.

2026-11-06 — **Conditional blocklist observer identity query traced.**
DC3's selected job can call the existing missing-LID pipeline before location
state updates; per-chat state and helper filtering gate it. Network-unavailable
sentinel skips the updates; downstream location wire effects remain open.
[Evidence and limits](chat-actions.md#blocklist-location-observer-query-boundary-2026-11-06).
No production protocol change or live account operation.

2026-11-06 — **Offline interface census closes blocklist observer handoff.**
Added apk-virtual-overrides --include-interfaces with checked DEX type lists and
transitive ancestry. Pinned callback routes to0d0.Blg;37 declared bodies retained,
without claiming active registrations or complete runtime dispatch.
[Tool, evidence and remaining jobs](chat-actions.md#blocklist-observer-interface-census-2026-11-06).
No production protocol or live account change.

2026-11-06 — **Blocklist delta storage and side-effect boundaries retained.**
The async worker copies current membership, rewrites its table, then separately
applies hash preferences/metadata/follow-up work. Transaction failures and ignored
row results are distinguished; local system-message handoffs are traced.
[Evidence and concurrency limits](chat-actions.md#blocklist-delta-storage-and-side-effect-boundaries-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist account-sync delta/refresh obligations traced.**
Matching previous hash/admission permits effective deltas; modify/mismatch or
failed admission queues kind4 refresh. Async persistence is separate from the
normal ACK boundary. Rust still captures this unsupported child without ACK.
[Processing and remaining ownership](chat-actions.md#blocklist-account-sync-notification-obligation-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Dirty blocklist fetch role/admission gates resolved.**
Post-adoption dirty reconciliation is primary-role-only;09F.BU7 is companion
mode, not a feature flag. Readiness false bypasses mode checks; otherwise
blocklist mode/global chat-migration authority controls the selected path.
[Exact branches and missing Rust authority](chat-actions.md#blocklist-dirty-fetch-role-and-admission-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Fetch results use the first direct named list.**
Removed the non-native duplicate-list veto; later children cannot stall the
selected valid snapshot. Malformed first matches still refuse without fallback,
and nested-only lists do not replace cached state. Five-case reopen regression.
[Evidence and limits](chat-actions.md#blocklist-fetch-first-named-child-2026-11-06).
No new request or live account operation.

2026-11-06 — **Set-result hash follows requested-target membership change.**
Already-satisfied block/unblock retain cache/hash bytes; newly applied changes
adopt or clear the selected-list hash. Clarified that the native backend Boolean
means execution success, not affected-row change. Twelve-case reopen regression.
[Evidence and separate-state limits](chat-actions.md#blocklist-set-hash-membership-gate-2026-11-06).
No new request or live account operation.

2026-11-06 — **Blocklist outbound hash omission follows distinct APK rules.**
Get omits null/empty; block/unblock also omit Java-blank values. Nonblank hashes
remain byte-for-byte unchanged. Controlled Java oracle checks every UTF16 unit;
synthetic get/block/unblock controls retain request/checkpoint evidence.
[Details and limits](chat-actions.md#blocklist-outbound-hash-omission-2026-11-06).
No live account operation or automatic request.

2026-11-06 — **Full blocklist snapshots clear an absent hash.**
Corrected stale checkpoint retention after an admitted no-hash full snapshot.
No-list/malformed controls preserve prior state; offline get/block construction
controls and reopen regression retain evidence.
[Native/API authority and set-result limits](chat-actions.md#blocklist-full-snapshot-nullable-hash-2026-11-06).
No automatic request or live account operation.

2026-11-06 — **Blocklist set results require the native first list.**
Corrected unsupported empty-result acceptance and root/trailing hash/branch
selection. The selected list is validated before cache/hash/reconciliation
projection; synthetic block/unblock refusal/reopen and first-list controls.
[Evidence and limits](chat-actions.md#blocklist-set-first-list-admission-2026-11-06).
No new request or live account operation.

2026-11-06 — **Blocklist default metadata provider binding resolved.**
Pinned provider/cache/factory instructions connect7to provider0xb43 ->0x430
->0bZ, with delegated12d and metadata helper12g. This establishes packaged
default construction, not current runtime cache or complete backend ownership.
[Exact cases and boundaries](chat-actions.md#blocklist-default-provider-binding-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist metadata concrete writer boundaries retained.**
Traced candidate store declarations, username trim/leading-@ normalization,
database-readiness/delegation and separate write/notification stages. Active
interface binding and atomic backend outcomes remain unproven; REA writer
body was unavailable while DEX instructions were retained.
[Findings and limits](chat-actions.md#blocklist-metadata-concrete-write-boundaries-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist metadata worker and existing-value authority traced.**
Native metadata updates are separately scheduled. Username proposals require
absence in two consulted maps; display names preserve an existing nonblank
value. Associations sort active first, then lexical user, before ordered
submission. [Evidence and backend limits](chat-actions.md#blocklist-metadata-adoption-authority-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist typed fallback and metadata priority traced.**
Pinned helpers show nullable requested-class conversion and type18-only
name -> PN association -> username priority, independent of block membership.
Retained six DEX helpers and complete REA Evidence; current strict Rust identity
admission and unified metadata rows remain identified gaps.
[Details and boundaries](chat-actions.md#blocklist-typed-identity-and-metadata-priority-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist snapshot membership follows the last repeated item.**
Corrected the confirmed APK Map.put mismatch: later non-block actions remove
earlier blocks and duplicate blocks no longer stall projection. Synthetic
ordering/reopen regression and retained pinned parser evidence;
[scope and limits](chat-actions.md#blocklist-snapshot-repeated-identity-membership-2026-11-06).
No automatic request or live account operation.

2026-11-06 — **Migration JSON null/escaping reference executed offline.**

Reused existing shared envelope/AOSP inspection and added a hash-pinned,
controlled JSON-library oracle: four migration-shaped cases verify explicit
nulls, conditional Boolean fields, escaping/UTF8 and empty arrays. It executes
no WhatsApp class or device. [Authority, fixtures and limits](chat-actions.md#migration-json-controlled-reference-2026-11-06).
No Rust sender or live request added; Pando/device behavior remains open.

2026-11-06 — **Migration intermediate values and coroutine lifetime narrowed.**

Direct GraphQL setters retain null through TreeMap export; wrapper omission
rules differ. Selected error policy is literal false and forwards Giw failure
to the waiting coroutine. Fresh operation supplies32s only to its non-Pando
send branch; final wire null encoding/Pando lifetime remain open.
[Evidence and limits](chat-actions.md#migration-values-and-coroutine-completion-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Blocklist migration request and state authority traced.**

Pinned MigrateBlocklistLid MEX operation has a distinct input/hash/list/dirty-ack
builder and response path. Selected worker0 adopts a supplied snapshot; worker1
resolves/merges identities before migration. Persisted addressing mode and the
isRollback guard control state writes; completion log alone proves neither.
[Evidence, identifiers and unported boundaries](chat-actions.md#blocklist-migration-request-and-state-authority-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Malformed blocklist children cannot erase the cache.**

Pinned parser requires every list child to be item. Rust now refuses unknown
children instead of projecting a partial/empty list, preserving every checkpoint,
hash, reconciliation warning and journal across reopen. Valid empty snapshots
remain valid; no request or retry added. [Evidence and regression](chat-actions.md#blocklist-snapshot-child-admission-2026-11-06).
1,517 workspace tests (four threads), strict Clippy and repository gates pass.

2026-11-06 — **Unmatched blocklist results no longer falsely change the cache.**

Pinned set callback diverts list matched=false into reconciliation. Rust now
preserves blocked rows/hash, records a visible durable reconciliation warning,
and marks the result inspected atomically without replaying the set. An admitted
later full fetch repairs the cache; no-list replies cannot clear the warning.
[Evidence, regression and limits](chat-actions.md#unmatched-blocklist-set-results-2026-11-06).
1,516 workspace tests (four threads), strict Clippy and repository gates pass.

2026-11-06 — **Automatic direct played receipts now honor known-disabled privacy.**

Pinned played/played-self selection exposed an unconditional Rust played queue.
New viewer-completion intents retain automatic provenance and the original wire;
known-disabled server preference holds them at queue/reservation and refuses a
late writer handoff. Consumption and uncertain reservations never reset.
Unknown/legacy provenance and group/explicit-read behavior remain unchanged.
[Evidence, regression and limits](read-receipt-policy-current.md#automatic-played-privacy-correction-2026-11-06).
1,515 workspace tests (four threads), strict Clippy and repository gates pass.

2026-11-06 — **Native played-receipt exception retry eligibility established.**

Pinned constructor binds persistence, per-chat group, chat requirement and100
job-attempt budget; exception hook returns true. Shared worker still distinguishes
runtime exceptions, requirement deferral and normal callback completion. Rust's
uncertain-receipt no-replay policy remains an explicit lifecycle compatibility gap;
no unsafe retry loop or live operation added. [Evidence and limits](receipt-ack-special.md#played-job-retry-eligibility-and-native-parity-gap-2026-11-06).

2026-11-06 — **Receipt dispatch drop/exception boundaries confirmed.**

Pinned APK dispatch can return without sending or completing its registered
callback. Selected played preparation preserves the original Message. Failure
censuses separate local future exception, size failure and disconnect cleanup;
none proves non-delivery or safe replay. No Rust behavior change or live action.
[Evidence and open boundaries](receipt-ack-special.md#receipt-dispatch-drop-and-failure-boundaries-2026-11-06).

2026-11-06 — **Pinned receipt callback registration/completion traced.**

Current APK played job registers a full-key callback before dispatch and waits
on a future completed with null by the selected wrapper. This is local completion,
not server acceptance. No Rust behavior change or live operation.
[Selected methods, callers, reproduction and limits](receipt-ack-special.md#current-receipt-registration-and-future-completion-2026-11-06).

2026-11-06 — **Opened-receipt collision and replay protections verified offline.**

New encrypted-store regression covers missing/unready sources, malformed routes,
and existing pending, uncertain, acknowledged or refused receipt intents. Refusals
preserve checkpoint bytes and journal across reopen. No production defect found
or behavior changed; queued intent is not transmission or acceptance.
1,513 workspace tests, strict Clippy and repository hygiene pass.
[Audit and limits](media-network-outcomes.md#opened-receipt-collision-and-replay-audit-2026-11-06).

2026-11-06 — **View-once consumption now verifies the selected source identity.**

A still-decrypted replacement inbox row can no longer be overwritten while the
old image is consumed. Exact source check precedes decrypt/consumed writes;
valid one-time consumption and awaiting-viewer state remain intact.
[Regression, source and limits](media-network-outcomes.md#view-once-consumption-source-identity-2026-11-06).
1,512 workspace tests and strict Clippy pass; no user content opened or live
account operation.

2026-11-06 — **View-once GET settlement now binds its reservation and inbox source.**

Late success/refusal cannot recreate removed state or overwrite changed outcomes;
the pre-network transaction also checks the original source row. Existing
uncertain outcomes remain no-retry. [Regression, evidence and limits](media-network-outcomes.md#view-once-download-result-ownership-2026-11-06).
1,511 workspace tests and strict Clippy pass; no live account operation.

2026-11-06 — **Standard upload category/purpose native transfer confirmed.**

Little-endian serialization and checked native register/member flow connect
category8/purpose0 to WA priority context. The optional property3 suffix remains
unknown. [Reader, instruction checks and scope](media-network-outcomes.md#standard-upload-categorypurpose-native-binding-closed-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Priority property-origin search expanded across Java/native boundaries.**

All24 physical key allocations include a dynamic decoder; request bridging also
uses it. Checked83 native members and direct setter calls; no original key3
producer proven. [Evidence, typed lookup and bounded negatives](media-network-outcomes.md#priority-property-origin-java-and-cross-library-boundary-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Priority service registration and normal graph invocation confirmed.**

Named JNI binding and16-stage reverse walk close two previous boundaries;
priority stage wraps an existing chain, with no separate selection flag found.
Optional r property's producer/first-upload value remains unknown.
[Evidence and bounded negative searches](media-network-outcomes.md#priority-jni-registration-and-normal-graph-activation-resolved-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Downstream native Priority header construction found.**

Prioritization service calls calculator and builds u/i/p plus optional positive
property-driven r. Standard MOu metadata yields category8/purpose0; its WA
calculator prefix is u=3, i. Graph activation/key3 presence remain open before
Rust policy changes. [Complete evidence and boundaries](media-network-outcomes.md#native-priority-header-producer-and-standard-upload-prefix-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **App priority calculator selection and enable-key source traced.**

0eA.A0z lazily constructs the holder from configuration key38178; the default
false constructor is not the selected path. Effective runtime value and wire
priority serialization remain unknown. [Evidence and scoped offset distinction](media-network-outcomes.md#selected-priority-calculator-flag-source-2026-11-06).
No Rust request change or live operation.

2026-11-06 — **Native priority interceptor attaches typed scheduling context.**

Confirmed static registration/factory/vtable/invocation and calculator numeric
branches; no direct header insertion established in this interceptor. No Rust
wire change or live operation. [Evidence and remaining scheduling boundary](media-network-outcomes.md#native-http-priority-context-separated-from-headers-2026-11-06).

2026-11-06 — **New media uploads require an explicit configured User-Agent.**

APK native HTTPS interceptor fills a missing header; Rust now refuses missing
client.user_agent before new upload state or transmission. Saved outcomes remain
unchanged. [Evidence, regression and boundaries](media-network-outcomes.md#new-media-upload-user-agent-preflight-2026-11-06).
1,509 workspace tests and strict Clippy pass; no live operation or ban guarantee.

2026-11-06 — **Native request header member and timeout conversion traced.**

Decoded header map moves to request+0x130; four native timeout inputs pass through
validateTimeout. [Offsets, helper and local-bound distinction](media-network-outcomes.md#native-header-transfer-and-timeout-normalization-2026-11-06).
No Rust timing/header change or live operation; downstream middleware remains open.

2026-11-06 — **Upload provider JNI class/table binding confirmed statically.**

Named registration binds inherited factory and stream/service methods; JNI ABI
slot independently checked. [Complete routine, tuple and source checks](media-network-outcomes.md#upload-jni-class-to-table-registration-closed-statically-2026-11-06).
Runtime registration success unobserved; no Rust behavior change or live traffic.

2026-11-06 — **Selected JNI submission connected to native Java-body adapter.**

Exact tuples/factories/vtables identify a conditional upload-provider chain to previously
traced stream start; effective wrapper gate and downstream headers remain open.
[Static chain and retained scopes](media-network-outcomes.md#selected-jni-upload-provider-adapter-connected-2026-11-06).
No Rust behavior change, replay or live operation.

2026-11-06 — **Selected Java upload header serialization traced to JNI.**

Existing header map is serialized separately from body provider; native internal
-1 candidate is not bound to ordinary uploads. [Full handoff/census and limits](media-network-outcomes.md#java-upload-header-submission-boundary-2026-11-06).
No mandatory-header mismatch, Rust behavior change or live operation established.

2026-11-06 — **Native queue expected size traced to Content-Length.**

Header key verified by relocation; paired store initializes expected/queued
fields. [Exact producer, conversion and search exclusions](media-network-outcomes.md#queue-expected-size-is-initialized-from-content-length-2026-11-06).
No Rust behavior change; preserve actual sealed-body length and uncertain rows.
No live operation or complete wire-parity claim.

2026-11-06 — **Native upload queue bound found; Pando flag scope separated.**

Selected queue rejects bytes above declared bound except-1 and rolls back this
increment. Disable-header getter belongs to Pando configuration. [Evidence and
unresolved expected-size producer](media-network-outcomes.md#native-queue-bound-and-disable-header-namespace-2026-11-06).
No Rust behavior change or live operation; local error never authorizes replay.

2026-11-06 — **Native stored upload length reaches count/total callback.**

Selected consumer pairs declared length with accumulated count; progress semantics
are inferred, exact callback target remains unknown. [Evidence and physical-search
limits](media-network-outcomes.md#stored-length-reaches-a-count-and-total-callback-2026-11-06).
No evidence of framing enforcement here; no Rust behavior change or live traffic.

2026-11-06 — **Conditional native upload length handlers identified.**

RTTI/vtable checks resolve limiting/MNS/HUC slot0x30 candidates. Limiting forwards
unchanged; selected MNS owner stores length under mutex. [Evidence and dynamic
binding limits](media-network-outcomes.md#native-length-handler-candidates-resolved-2026-11-06).
No Rust behavior change or live traffic.

2026-11-06 — **Upload body provider traced to native length forwarding.**

Declared length reporting is separate from callback/EOF bytes; selected provider
compression is disabled. Native JNI wrapper reaches unresolved virtual slot0x30.
[Complete methods, pinned ELF tuples and limits](media-network-outcomes.md#native-upload-length-handoff-2026-11-06).
No inferred truncation, Rust behavior change or live traffic.

2026-11-06 — **Media raw-body and fixed-length gates distinguished.**

Both transport gates derive from0x666; disabling them does not select multipart.
[Exact branches, constructors, complete REA and remaining provider inputs](media-network-outcomes.md#raw-body-and-fixed-length-gates-are-separate-2026-11-06).
No proven mandatory framing mismatch or Rust behavior change; no live operation.

2026-11-06 — **Original ordinary-upload extra-header map traced.**

Ordinary message assembly supplies null; selected factories do likewise and
copies preserve it. Nonempty authorization/metadata map belongs to backup.
[Complete pinned origins and remaining transport/body-mode scope](media-network-outcomes.md#original-media-extra-headers-traced-2026-11-06).
No Rust header change or live operation.

2026-11-06 — **Media header producer scope narrowed.**

Provider aggregation keys only newsletter; descriptor maps override defaults.
Standard voice descriptor copying preserves its prior map, so MetaAI null maps
are not ordinary-upload proof. [Evidence, exact provenance and open producers](media-network-outcomes.md#media-header-producer-scope-2026-11-06).
Static research only; no Rust header change or live operation.

2026-11-05 — **Outgoing retry error domains/eligibility mapped.**

Crypto status, internal failure kind and wire error are distinct; selected
crypto eligibility and guarded refinements recovered. Other producers bypass
the converter. [Evidence and remaining typed-cause work](outgoing-retry-receipts.md#error-domains-and-eligibility-recovered--2026-11-05).
No blanket error default, runtime behavior change or live traffic.

2026-11-05 — **Outgoing retries now preserve original message timestamp.**

Automatic retry.t uses failed message root t instead of local queue time.
Missing/unusable timestamps refuse without queue writes; held retries do not
reset on changed metadata. [Native chain, regressions and limits](outgoing-retry-receipts.md).
Error-code/key/count progression remains incomplete; no live traffic.

2026-11-05 — **Retry refetch callbacks now belong to their live connection.**

Only a locally written query can complete through normal receive; result/error
consumes its callback once. Disconnect/restart never resurrects callbacks from
held jobs. [Native cancellation evidence, tests and limits](retry-answers.md#live-refetch-callback-lifetime-2026-11-05).
No guessed timeout, new request, reset or automatic replay; no live traffic.

2026-11-05 — **Retry key-query manager-to-wire construction audited.**

Classic non-identity/context-disabled IQ baseline matches Rust. Source12 is not
a wire field; identity/PQ/context attrs have separate predicates. Manager entry
expiry is not a proven IQ deadline. [Evidence and remaining gaps](retry-key-query-wire.md).
No production field/backoff/replay change or live traffic.

2026-11-05 — **Durable retry key queries now ACK first.**

Selected post-ACK job/session-requirement fetch entry traced. Normal wrapper
attaches root receipt ACK only after pending job and fetching marker commit;
ACK/query failure stops without replay. [Evidence, tests and limits](retry-ack-dispatch.md#durable-retry-key-query-ack-ordering--2026-11-05).
No key-request field/backoff change or live traffic; recovery remains incomplete.

2026-11-05 — **Reserved direct retries now ACK before resend write.**

Normal receive attaches success ACK only after durable message reservation and
uses root receipt correlation. ACK failure prevents resend submission; any
uncertain work stays held. Queries/refusals/duplicates/group paths remain outside
this ACK authority. [Evidence, tests and limits](retry-ack-dispatch.md#reserved-direct-resend-success-ack--2026-11-05). No live traffic.

2026-11-05 — **Retry refetch settlement ownership strengthened.**

Refusal and resend reservation now recheck the pending job snapshot and exact
fetching/query/count checkpoint before consumption. Changed jobs/terminal or
legacy markers remain held without overwriting peer or retry state.
[Race regressions and limits](retry-answers.md#refetch-completion-ownership-guard-2026-11-05). No new send or success ACK.

2026-11-05 — **Retry schema-error ACK context corrected.**

Confirmed required-field/ASCII numeric errors now receive487 with fresh native
context, omitting recipient/edit. Unsupported-version487 keeps its copied context;
first-error order and local capability refusals remain distinct.
[Evidence, tests and limits](retry-ack-dispatch.md#required-field-parsing-error-responses--2026-11-05). No live traffic.

2026-11-05 — **Unsupported retry versions now receive error487.**

Shared confirmation mirrors supported receipt context for explicit non-1 text
versions after four-byte registration admission. Writer binding and deferred
callback ordering traced. [Evidence, regressions and limits](retry-ack-dispatch.md#unsupported-version-error-response-correction--2026-11-05).
Other retry ACK/error/recovery policy remains incomplete; no live traffic.

2026-11-05 — **Retry ACK dispatch traced; plain-ACK suspicion refuted.**

Current Rust excludes retry ACKs. Selected APK handler separates handled,
unhandled/error487 and parse-exception paths; actual receipt context factory
located. Final Message76 writer binding/async order remain open.
[Evidence and limits](retry-ack-dispatch.md). No runtime policy change or live request.

2026-11-05 — **Retry original-chat selection/correlation strengthened.**

Shared parser separates explicit recipient/normalized chat from asking device.
Normal direct retry checks the selected chat against original peer/known LID
before session work or a query. [Evidence and regressions](retry-original-chat.md).
No receipt-derived alias adoption, live request or full native routing claim.

2026-11-05 — **Retry original-message correlation corrected.**

Shared parser now uses required nested retry.id for original lookup and requires
signed64-bit retry.t. Root receipt ID is independent; malformed nested fields
cannot mutate/query retry state. [Evidence and regressions](retry-parser-admission.md#nested-message-key-and-timestamp-correction-2026-11-05).
No live request or new timestamp/ACK policy.

2026-11-05 — **Comparison group retry enc counts corrected from pinned DEX.**

Received count1/2 now reaches root skmsg and pairwise distribution enc children
unchanged; original first sends still omit it. Ciphertext/ID/addressing remain
preserved. [Binding, wire regression and limits](group-retry-count.md).
No live request or full group retry parity claimed.

2026-11-05 — **Comparison group retry peer state and write settlement fixed.**

Reservation checks identity/session snapshots and holds advanced peer state with
a reserved checkpoint before I/O. Owned settlement records written or uncertain;
legacy/held checkpoints never replay. [Tests and limits](group-retry-admission.md#peer-snapshot-and-write-settlement-correction-2026-11-05).
No live socket, key fetch or resend operation.

2026-11-05 — **Comparison group retry admission strengthened.**

Before reserving a device or querying keys, the window checks group-chat/message
correlation, supported count, carried-key exclusion and original/duplicate device.
[Evidence, tests and remaining issues](group-retry-admission.md). No live request or group policy
completion claimed.

2026-11-05 — **Retry parser registration/version/defaults corrected.**

Shared parser requires four registration bytes, defaults missing v/count to1
and refuses unsupported version/malformed count. Invalid registration no longer
falls through to a key query. [Evidence and tests](retry-parser-admission.md).
No live request or expanded higher-count/key handling.

2026-11-05 — **Unported receipt-key retries no longer fall through.**

Normal direct retry driver records receipt_keys_unsupported and returns no
resend/key query when keys are carried. Supported no-keys retries still work.
[Evidence, local limit and tests](retry-answers.md#receipt-carried-keys-capability-boundary-2026-11-05). No key adoption or live requests.

2026-11-05 — **Retry refetch query reservation now rechecks its inputs.**

Original row/age and identity/session snapshots are checked atomically with
pending query records; stale inputs produce no query job. Duplicate answer
reservation remains once-only. [Correction and regressions](retry-answers.md#refetch-query-reservation-inputs-2026-11-05).
No live requests or expanded retry counts/root behavior.

2026-11-05 — **Saved-session retry reservation race fixed offline.**

Retry reservation now rechecks original outbox snapshot, identity/session and
age after async crypto, preserving newer delivery/failure/attempt/peer state.
Fresh and saved paths share guards. [Correction and tests](retry-answers.md#saved-session-retry-reservation-guard-2026-11-05).
Also retained checked current-APK root-removal gates without guessing their
applicability or changing root attributes. No live retry operation.

2026-11-05 — **Recovered metrics consumer checksum and task admission traced.**

Version3 payload Adler32 is checked after file loading; versions1/2 skip this
consumer comparison. Mismatch increments a counter and ends the remaining
snapshot walk. Passing outgoing entries reach conditional transport handoff.
[Evidence and limits](apk/2.26.43.72-metrics-recovery-consumer/README.md).
Validation:12 native checksum cases and mismatch-counter slices plus pinned
binding/windows. Startup flag/lifetime and actual replay remain unobserved.
No real I/O, Rust runtime change or live traffic.

2026-11-05 — **Metrics buffer loader admission and bulk-delete distinction traced.**

Loader accepts selected prefix/type and WMT versions1/2/3 with positive bounded
payload; restores checksum/state without payload comparison in this body.
Separate directory walk is bulk deletion. [Evidence and limits](apk/2.26.43.72-metrics-buffer-recovery/README.md).
Validation:18 native header cases and pinned loader/helper windows. Remaining
files can pass local gates; automatic restart replay and cleanup policy remain
unproven. No real I/O, Rust runtime change or live traffic.

2026-11-05 — **Metrics completion-to-file-removal chain traced.**

Selected regular table now binds owner callbacks: null-error success queues
buffer removal; terminal local error skips that deletion path despite set
removal. Worker remove failures report errno separately. [Evidence and limits](apk/2.26.43.72-metrics-buffer-deletion/README.md).
Validation:4 controlled full-worker cases and pinned construction/function
windows. Downstream bookkeeping/recovery remains open; no real files removed,
Rust runtime change or live traffic.

2026-11-05 — **Regular metrics recovery timer and provider gates traced.**

Matching provider/status0 with idle low bit starts a drain and interval30 timer;
status1 cancels the existing timer and signals drain error. Timer callbacks
require ready and idle low bits. [Evidence and limits](apk/2.26.43.72-metrics-regular-recovery/README.md).
Validation:72 native status-decision and16 timer-decision fixtures plus pinned
window resolver. Weak resolution, actual timers and durable ownership remain
separate; no Rust runtime or live traffic changed.

2026-11-05 — **Metrics error subcodes and credential effects traced.**

Pinned native paths recognize3067002/3067003/3067004, release the current
metrics credential and promote a spare or request gated renewal. Failure-reason
suffix parsing is distinct from response-body parsing; diagnostic numbers are
not account restriction codes. [Evidence and limits](apk/2.26.43.72-metrics-error-subcodes/README.md).
Validation:24 synthetic body-parser cases,7 worker integer-slice cases, pinned
resolver and bundle checks. Full retry/durable ownership and server meanings
remain open. No Rust runtime or live traffic changed.

2026-11-04 — **Legacy writer/reader reply-map alias traced.**

Normal setup passes one ConcurrentHashMap through both writer and reader
factory context; configured factory membership/lifetime remains open.
[Evidence and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-writerreader-map-alias-2026-11-04).
No production change or live traffic.

2026-11-04 — **Legacy prekey reply removal and error event traced.**

Exact result/error removes the legacy map entry;290 error extracts code/text
and emits arg1=78. Generic callback assumption refuted.
[Evidence and map/listener limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-reply-removal-and-error-event-path-2026-11-04).
No production change or live traffic.

2026-11-04 — **Legacy SET writer and result-event path traced.**

Retained writer case arg1=85 confirms fullSET/op fields and separate reply map;
290/1ru result produces success arg1=77. Prior what85 wording corrected.
[Evidence and lifecycle limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-writer-shape-and-success-event-path-2026-11-04).
No production change or live traffic.

2026-11-04 — **Legacy SET snapshot and message85 handoff traced.**

A0S/A03/Fdv retains a separate legacy path from modern SET/ADD, including
reason-dependent PQ ensure argument and selected-array ownership.
[Evidence and writer/entry limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-set-snapshot-and-message85-handoff-2026-11-04).
No production change or live traffic.

2026-11-04 — **Native notification/daily prekey routing traced.**

43.72 distinguishes migrated ADD, legacy-only ADD and non-migrated SET;
daily missing-migration path reaches PQ-only SET. Worker ACK decision order
retained. [Evidence and remaining scope](apk/2.26.43.72-prekey-upload-replies/README.md#notification-and-daily-pq-entry-routing-2026-11-04).
No production change or live traffic.

2026-11-04 — **Consumed KEM keys can settle later upload ACKs without resurrection.**

Atomic nonsecret inbox marker distinguishes legitimate consumption from missing
material. Stale allocator reuse refuses; rollback/reopen paths tested.
[Prior-guard correction and scope](network-correctness-audit.md#consumed-kem-key-before-upload-ack-2026-11-04).
No live traffic.

2026-11-04 — **KEM acknowledgement cannot silently skip lost key material.**

Settlement requires/validates every reserved KEM row. Missing-second-row
regression proves earlier writes and Batch status roll back across reopen.
[Details and scope](network-correctness-audit.md#kem-acknowledgement-inventory-integrity-2026-11-04).
No live traffic.

2026-11-04 — **Uncertain KEM batches cannot ride a fresh upload again.**

Transactional hold prevents sent=false KEM rows being selected under a new IQ
with newly generated legacy keys. Full-store regression covers reopen and
release only by original settlement. [Details and limits](network-correctness-audit.md#uncertain-kem-batch-replay-admission-2026-11-04).
No live traffic.

2026-11-04 — **New prekey reservations freeze their encoded request.**

Wire payload/mode/dictionary checkpoint commits with keys and Batch. Reopen
regression proves unchanged provenance, no replay and no historical backfill.
[Scope and pending ADD strategy](network-correctness-audit.md#frozen-prekey-request-provenance-2026-11-04).
No live traffic.

2026-11-04 — **SET/ADD manager entry and migration predicates traced.**

Low-count ADD requires21198 plus a sent last-resort PQ row; PQ-only SET has
separate entry. Shared readiness/inflight/backoff guard retained.
[Evidence and upstream limits](apk/2.26.43.72-prekey-upload-replies/README.md#setadd-entry-admission-and-migrated-state-2026-11-04).
No production change or live traffic.

2026-11-04 — **Prekey construction failures roll back upload reservation.**

Identity validation/KEM child signing moved inside the reservation transaction.
PQ-off/on regression proves complete key/checkpoint preservation and reopen,
then valid reservation/settlement. [Details and remaining ADD port](network-correctness-audit.md#prekey-construction-and-reservation-atomicity-2026-11-04).
No live traffic.

2026-11-04 — **PQ generation target and upload batch defaults distinguished.**

43.72 provider21200/default200 governs inventory;21201/default10 governs
batch. Matching REA registry reused; runtime values/lifecycle remain open.
[Evidence and Rust gap](apk/2.26.43.72-prekey-upload-replies/README.md#pq-target-provider-and-declared-defaults-2026-11-04).
No production change or live traffic.

2026-11-04 — **Current-build PQ signing/wire provenance traced.**

ML-KEM-1024 enum and same-public-array signing/upload linkage retained.
Native byte lengths/prefix remain outside the Java wrapper proof.
[Evidence and Rust comparison](apk/2.26.43.72-prekey-upload-replies/README.md#pq-signing-and-wire-byte-provenance-2026-11-04).
No production change or live traffic.

2026-11-04 — **ADD ordinary-PQ selection/generation boundary traced.**

Non-last-resort SQL selection, supplied generation target, native-loader gate
and chunked cursor path retained; target/signing providers remain open.
[Evidence and limits](apk/2.26.43.72-prekey-upload-replies/README.md#add-ordinary-pq-selection-and-generation-dispatch-2026-11-04).
No production change or live traffic.

2026-11-04 — **Native prekey cursor ownership and creation range traced.**

Self-row reader throws when missing; creation independently seeds positive
legacy/PQ cursors. Existing local history must not be reinterpreted or reseeded.
[Evidence and remaining schema/JNI paths](apk/2.26.43.72-prekey-upload-replies/README.md#native-cursor-ownership-and-creation-seed-2026-11-04).
No production change or live traffic.

2026-11-04 — **Current-build legacy generation loop/cursor gap confirmed.**

43.72 fills the 812 deficit in <=50-key chunks; local generation stops at one
chunk and uses different cursor convention. Explicit store compatibility is
required before porting. [Evidence](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-generation-loop-and-cursor-convention-2026-11-04).
No production change or live traffic.

2026-11-04 — **ADD prekey selection and exact child layout traced.**

43.72 SQL filter, key conversion/ID bytes and ADD-only wire shape retained.
Generation/configuration/durable mode remain open before runtime port.
[Evidence and limits](apk/2.26.43.72-prekey-upload-replies/README.md#add-selection-and-exact-child-layout-2026-11-04).
No production change or live traffic.

2026-11-04 — **Prekey success continuation ADD construction gap confirmed.**

43.72 Fcd selects ADD/configured batching; Rust follows older SET/812 layout.
Generation dispatch and callback binding retained; full shape/selection and
persistence compatibility remain open. [Evidence](apk/2.26.43.72-prekey-upload-replies/README.md#successful-continuation-selects-add-2026-11-04).
No production change or live traffic.

2026-11-04 — **Limited prekey retry-mode writer scope traced.**

FDh config allowance is conditional, not a general startup default; visible
exception paths and 14o reset/consumption retained.
[Evidence and unknown lifecycle](apk/2.26.43.72-prekey-upload-replies/README.md#limited-mode-producer-boundary-2026-11-04).
No production change or live traffic.

2026-11-04 — **Prekey retry boundaries and delay inputs traced.**

43.72 13z.A02 separates limited allowance from backoff counter bounds;
406/503 limited mode uses configured delay. Runtime gate/mode producers
remain open. [Evidence and limits](apk/2.26.43.72-prekey-upload-replies/README.md#retry-limits-and-delay-inputs-2026-11-04).
No production change or live traffic.

2026-11-04 — **Prekey storage effects and gated failure branches traced.**

43.72 result worker marks legacy/Kyber sent before completion. EcZ has
configuration-dependent 406/5xx branches; helper bounds and runnable effects
remain open. [Evidence and limits](apk/2.26.43.72-prekey-upload-replies/README.md#store-effects-and-completion-policy-follow-up-2026-11-04).
No production change or live traffic.

2026-11-04 — **Current-build prekey reply callbacks retained.**

43.72 140/141 callbacks confirm optional first list/pq_list count_low parsing;
error extraction and result worker retained with downstream policy gaps explicit.
[Static evidence and limits](apk/2.26.43.72-prekey-upload-replies/README.md).
No production change or live traffic.

2026-11-04 — **Retry refetch rechecks original age before resend.**

Completion and reservation enforce the existing strict 60-day limit. Shared
comparison avoids signed overflow; regression covers aged pending fetch and
boundary/extreme timestamps. [Evidence and approximation limits](network-correctness-audit.md#retry-refetch-age-eligibility-recheck-2026-11-04).
No live traffic.

2026-11-04 — **Delivery received during key refetch suppresses retry resend.**

Completion rechecks acceptance/delivery and atomically guards the original send
row against changes during preparation. Real signed-bundle workflow covers both
delivered and undelivered outcomes. [Evidence and limits](network-correctness-audit.md#retry-refetch-delivery-recheck-2026-11-04).
No live traffic.

2026-11-04 — **Retry refetch refuses to overwrite newer peer state.**

Identity/session are rechecked inside resend reservation after async preparation.
Regression independently changes each and proves newer state and pending fetch
are preserved. [Evidence and remaining concurrency scope](network-correctness-audit.md#retry-refetch-peer-state-reservation-guard-2026-11-04).
No live traffic.

2026-11-04 — **Retry-refetch retirement commits atomically with its outcome.**

Separate early job deletion removed; refusal records or reserved replacement
send/session now share the retirement transaction. Fault regression proves
rollback retains pending work. [Details and native parity limits](network-correctness-audit.md#retry-refetch-atomic-settlement-2026-11-04).
No live traffic.

2026-11-04 — **Media refresh admission and durable route preservation verified.**

Existing type/ID/tag/generation guard reviewed; new regression preserves full
checkpoint state across invalid replies and reopen, then applies a valid reply
idempotently. Production behavior unchanged.
[Evidence and remaining scope](network-correctness-audit.md#media-refresh-settlement-admission-review-2026-11-04).
No live traffic.

2026-11-04 — **Keepalive waits for an actual result/error reply.**

Same-ID non-reply IQs neither terminate receive nor clear the outstanding ping.
Actor regression covers continued processing, later result, actual error and
unanswered timeout. [Evidence and scope](network-correctness-audit.md#keepalive-reply-type-admission-2026-11-04).
No live traffic.

2026-11-04 — **Non-reply IQs no longer terminate activation as refused.**

Exact result/error admission now precedes activation completion/refusal.
Actor regression checks continued ping handling, capture, pending readiness,
unchanged account/journal, then valid activation. Actual error stays terminal.
[Evidence and limits](network-correctness-audit.md#activation-reply-type-admission-2026-11-04).
No live traffic.

2026-11-04 — **Retry refetch preserves pending state for non-reply IQs.**

Exact result/error is admitted before loading/deleting a refetch checkpoint.
Encrypted-store regression covers reopen, unchanged retry/send/session history,
one genuine error and duplicate; the real signed-bundle result workflow also
remains available after non-replies.
[Evidence and scope](network-correctness-audit.md#retry-refetch-reply-type-admission-2026-11-04).
No live traffic.

2026-11-04 — **Direct/group send preparation requires actual IQ reply types.**

Same-ID non-reply IQs no longer fail direct intents or consume group membership/
key waits. APK 43.72 0ln.A01 supports exact result/error callback admission.
Encrypted-store regressions retain reservations and original intent across reopen.
[Evidence, test scope and remaining gaps](network-correctness-audit.md#send-preparation-reply-type-admission-2026-11-04).
No live traffic.

2026-11-04 — **Device-query non-reply IQs preserve uncertain state.**

Exact result/error now required before matching and ingestion, grounded in
43.72 0ln.A01. Regression covers full query preservation, pending users,
reopen and one valid settlement. No live traffic.
[Evidence and limits](network-correctness-audit.md#device-query-reply-type-admission-2026-11-04).

2026-11-04 — **Confirmed source defect fixed: prekey IQ type admission.**
PendingUpload.matches previously checked tag/id only; settle treated every
non-result as refusal and released keys. Pinned 43.72 0ln.A01 dispatches
only result/error to IQ callbacks. [Correction and regression](network-correctness-audit.md#prekey-upload-reply-type-admission-2026-11-04).
No retry, key rotation or live request introduced.

2026-11-04 — **Pinned static: writer callback guard and output boundary.**
0Sf message metadata -> 0v0/0uw -> queued membership and pre-write
0ln.A0C -> 1rj/1r2 output write retained, with I/O requeue qualification.
[Observer binding, exact sequence and uncertainty](rea/2.26.43.72-reachout-state/README.md#writer-callback-admission-before-output-stream-write-2026-11-04).
No sent-state reset, new retry policy or live traffic.

2026-11-04 — **Pinned cleanup and offline durable reopen verification.**
0ln.A0F/handler3 distinguishes callback maps; 0YB/0YU callers and distinct
0Sf.A0J cancellation retained. Existing action regression now reopens while
held and sent, preserving original rows and preventing replay.
[Sources and exact verification boundaries](rea/2.26.43.72-reachout-state/README.md#disconnect-callback-cleanup-and-durable-reopen-regression-2026-11-04). No live traffic.

2026-11-04 — **Pinned static: IQ pending maps, dispatch and timeout.**
0Sf.A0W/A0A -> 0ln registration/map transfer/reply removal and
0m2 case4 -> 0ln.A03 -> 8bj.BsS retained. Delay resolves to ms.
[Sources, in-memory limits and durable port obligations](rea/2.26.43.72-reachout-state/README.md#pending-iq-registration-dispatch-and-timeout-ownership-2026-11-04).
Account/lower-writer teardown remains open; no live traffic.

2026-11-04 — **Pinned static: JSON DI/parser branches and extraction matrix.**
1Uw.A08 key 4417 -> 0iz; SGd.A04-dependent branches, UTF-8, 5ks/BTj
helpers and 1WU/0iv slot binding retained. Wrong-typed data/errors differ
from absence/null. [Sources and failure disposition](rea/2.26.43.72-reachout-state/README.md#pinned-json-decoder-branches-and-extraction-failures-2026-11-04).
No current-build grammar inferred solely from older evidence; no live traffic.

2026-11-04 — **Pinned static/derived: response equality and format fallback.**
8B6.A0J/A0K/A0H plus original 7Ou/8TV bind id/from to original id/to
and explain why ignored JSON format validation does not veto a body.
[Exact constraints and remaining account/decoder boundaries](rea/2.26.43.72-reachout-state/README.md#request-bound-equality-and-json-alternative-error-propagation-2026-11-04).
No universal sender policy or runtime change; no live traffic.

2026-11-04 — **Pinned static: fetch defaults and response-format mapping.**
0iM accessors/74V nested model and 8TV.ABJ -> 7NM -> 7jy selectors
resolved. Prior fetch-default unknown closed; push accessor semantics
remain separate. [Exact methods and remaining decoder work](rea/2.26.43.72-reachout-state/README.md#fetch-field-defaults-and-body-format-selectors-2026-11-04).
No runtime behavior or notification policy change; no live traffic.

2026-11-04 — **Pinned static: refresh response shape and callback admission.**
0mb/0md -> 0mZ/0is -> RsF -> BaseMexCallback/0id retained.
Fetch error callback false suppresses success with partial data; default
empty-data policy declines success. Full wire decoding/correlation open.
[Sources and future durable application obligations](rea/2.26.43.72-reachout-state/README.md#iq-response-conversion-and-refresh-callback-admission-2026-11-04). No live traffic.

2026-11-04 — **Pinned static: normal MEX registry initialization resolved.**
0ig.<init> -> 0ik.A01 -> 1Wy(22) -> 0ii -> 0im -> 0in asset loader;
CRh reads doc_id while AHH separately gates client IDs. Global alternative
registrants retained as unresolved, not sole-owner proof.
[Sources and remaining response/persistence work](rea/2.26.43.72-reachout-state/README.md#executor-registration-and-asset-backed-server-id-lookup-2026-11-04). No live traffic.

2026-11-04 — **Pinned static: refresh spec, transport and runtime-ID boundary.**
6oh/0iT empty-variable request -> 0ig/0iq -> 0j3 XMPP branch retained.
Asset server/client document IDs exported separately. 0ih/0ik fallback
5vM.CRh returns null; runtime map population remains untraced.
[Sources and implementation obligations](rea/2.26.43.72-reachout-state/README.md#refresh-request-construction-and-runtime-document-lookup-2026-11-04). No runtime/live change.

2026-11-04 — **Confirmed source defect: stale queue admission before restriction.**
actions.reserve_limit previously marked an earlier queued group-create sent
without consulting the later active checkpoint. Both bulk/actor and selected
reservations now check it transactionally; blocked intent is unchanged.
[Source chain, native-policy limit and regression](network-correctness-audit.md#restriction-recheck-at-action-reservation-2026-11-04).
No live traffic or universal outreach coverage claim.

2026-11-04 — **Source correction: deadline does not authorize restriction release.**
Native sheet/countdown -> 7uq -> FetchReachoutTimelockQuery -> 8dV success
-> 7vL state application traced. Group create now checks recorded active
state instead of elapsed inspection. Explicit inactive still releases it.
[Evidence, cache limits and missing automatic refresh](rea/2.26.43.72-reachout-state/README.md#deadline-is-a-refresh-trigger-not-an-inactive-update-2026-11-04).
Real-store regression; no live traffic or complete native policy claim.

2026-11-04 — **Pinned static: call gate, sticker refusal and record correction.**
1v7.A02 pre-call branch, ShareMediaViewModel.A01 read-only refusal and
2Q4 wrapper/lookup chain resolve caller semantics. Prior per-message/ent:
metadata interpretation is corrected to JID-bound record with unknown producer.
[Sources and bounded remaining obligations](rea/2.26.43.72-reachout-state/README.md#caller-semantics-call-gate-and-sticker-refusal-2026-11-04). No live traffic.

2026-11-04 — **Pinned static: native outreach predicate and set provenance.**
6po/6pn, 0ht.A0U -> 0hv.A0P trusted-contact rows, helper branches and
2Q4 caller retained with direct caller censuses.
[Decision exits and unresolved native policy](rea/2.26.43.72-reachout-state/README.md#outreach-decision-chain-and-recipient-set-2026-11-04).
No blanket restriction or exemption was inferred; no live traffic.

2026-11-04 — **Pinned static/derived: reach-out enforcement mapping.**
7gL enum names + 02X constants + 7k9 conversion + 7vL saved-type conversion
resolved; 7lA banner reader and discarded 6pn read traced.
[Table, negative search and open consumer policy](rea/2.26.43.72-reachout-state/README.md#enforcement-names-and-stored-type-mapping-2026-11-04).
No Rust runtime change or live traffic.

2026-11-04 — **Pinned static and source correction: reach-out decimal expiry.**
6oe.A03 Long.parseLong now matched using the shared BMP decimal reader.
7jz body adapter, 8AF data/class selection and generated constructor retained;
7k9 enum mapping retained with unresolved meanings.
[Evidence and limits](rea/2.26.43.72-reachout-state/README.md#json-adapter-and-java-decimal-expiry-2026-11-04). Local bounds remain; no live traffic.

2026-11-04 — **Source defect fixed: malformed recognized reach-out body.**
Missing body/bad JSON/missing or nonboolean active no longer silently return
ignored and pass into generic ACK. Real-store regression preserves known lock.
[Scope and native-policy limitation](network-correctness-audit.md#recognized-reach-out-body-refusal-2026-11-04).
Unknown operations unchanged; no live account traffic.

2026-11-04 — **Pinned static/derived: offline device category table.**
2FP.A07,Exz identity,EpS subtypes and0EB conversion retained; categories
are not sender trust labels.
[Table and caller limits](rea/2.26.43.72-reachout-state/README.md#offline-device-classification-table-2026-11-04).
Account predicate/identity authority remains open; no runtime change.

2026-11-04 — **Pinned static: conditional offline preflight/map mechanics.**
1rA notification branch,2FP classification-result gate and0ZJ map factory/
insertion retained. Offline>=11 is conditional, not sender authority proof.
[Evidence and unknowns](rea/2.26.43.72-reachout-state/README.md#high-offline-preflight-and-handler-map-construction-2026-11-04).
No Rust runtime or live-account change.

2026-11-04 — **Pinned static: legacy notification-to-MEX code bound.**
1s2 fallback/1rh layout,1rA/1rN notification branch and EL2 mex->0x102
map retained. Message parser false lead separated from sender authority.
[Evidence/remaining scope](rea/2.26.43.72-reachout-state/README.md#legacy-notification-to-mex-code-binding-2026-11-04).
No runtime or live account changes.

2026-11-04 — **Pinned static: receive gate classified as persistence.**
0ZP.A04/A05 offline-count/readiness and A03 queue scheduling retained;
none establishes sender authority.
[Evidence/negative result](rea/2.26.43.72-reachout-state/README.md#receive-gate-is-persistence-not-origin-validation-2026-11-04).
Origin/fallback trace remains open; no runtime or live traffic change.

2026-11-04 — **Pinned static discovery: alternate MEX receive registry.**
1s2/KWt maps and selected schema/type exports retained.7Ou mex overload
is IQ-result, refuting its use as notification origin evidence.
[Registry boundary and negative results](rea/2.26.43.72-reachout-state/README.md#receive-registry-discovery-and-schema-false-lead-2026-11-04).
No sender-authentication completion or runtime change claimed.

2026-11-04 — **Pinned static: MEX message/worker ordering boundary.**
2.26.43.72 router0x102 registration, wrapper ingress and direct/executor
dispatch retained;7MU reservation is inherited no-op. Upstream sender
authority still open. [Evidence](rea/2.26.43.72-reachout-state/README.md#message-wrapper-and-reach-out-ordering-reservation-2026-11-04).
No runtime policy or live-account change.

2026-11-04 — **Source integrity fixed: reach-out envelope before mutation.**
Existing base_ack validation now runs before lock checkpoint writes; actual
ACK follows persistence. Real-store invalid-envelope regression retained.
[Native base leaves and local scope](rea/2.26.43.72-reachout-state/README.md#base-dispatch-and-local-envelope-before-state-guard-2026-11-04).
Upstream sender authority remains untraced; no live traffic.

2026-11-04 — **Pinned mismatch corrected offline: MEX operation selection.**
2.26.43.72 0v7.A09/8AF.A09 retained; Rust selects first update and supports
ASCII case-insensitive reach-out operation names. Arbitrary tags/later updates
no longer mutate restriction. [Evidence/tests](rea/2.26.43.72-reachout-state/README.md#current-mex-operation-routing-2026-11-04-offline-corrected).
Sender authority untraced; no live account traffic.

2026-11-04 — **Pinned static correction: inactive reach-out ignores expiry.**
2.26.43.72 8AF/7MU/7vL dispatch/state plus6oe parser retained. Explicit
inactive reset does not parse expiry; Rust now accepts it while active bad
expiry preserves lock. [Evidence](rea/2.26.43.72-reachout-state/README.md).
Previous requirement for inactive expiry refuted. Sender authority remains open.

2026-11-04 — **Confirmed source defect fixed offline: reach-out expiry.**
Malformed known expiry previously became0 and could replace an active local
restriction. Parser now refuses before write; receive skips ACK and continues
while storage failures remain fatal.
[Scope and regression](network-correctness-audit.md#malformed-reach-out-expiry-preservation-2026-11-04).
No new native grammar/ban-cause claim or live account traffic.

2026-11-04 — **Confirmed mismatch corrected offline: ordinary path query fields.**
Pinned2.26.43.72 M0g.A00 requires nonnull oe/oh query values; Rust now
requires exact encoded keys with native empty/fragment semantics, preserving
raw reply before refusal. [Evidence/tests](rea/2.26.43.72-media-upload-response/README.md#ordinary-path-query-presence-correction-2026-11-04-offline).
No signature/expiry interpretation, automatic replay or live operation.

2026-11-04 — **Confirmed static: scheduled cross-reference response mode.**
Pinned2.26.43.72 mode writer dataflow, MOu/KmQ consumer and whole-string
regex semantics close opaque UUID meaning; ordinary normalization remains open.
[Evidence](rea/2.26.43.72-media-upload-response/README.md#mode-producers-and-scheduled-cross-reference-consumer-2026-11-04).
No Rust runtime or live-account change.

2026-11-04 — **Confirmed static: upload response mode distinction.**
Pinned2.26.43.72 MtR.CDR/KmR/Lkh retain opaque-handle versus original
direct-path fields conditionally. UUID pattern is not a universal path rule.
[Evidence and untraced producers/consumers](rea/2.26.43.72-media-upload-response/README.md).
No runtime acceptance change or live account traffic.

2026-11-04 — **Confirmed offline: sealed voice checkpoint continuity.**
Real encrypted-store regression covers header receipt/reopen, exact framing
fields, no replay and unchanged completed legacy rows.
[Scope](media-network-outcomes.md#sealed-voice-checkpoint-reopen-regression-2026-11-04).
No new runtime behavior or live traffic.

2026-11-04 — **Confirmed mismatch corrected offline: finished voice seal.**
Pinned2.26.43.72 finished sent/hash seal is appended only to new voice
wire bodies, retaining cipher hash and exact new reservation bytes/version.
Historical reservation handling precedes construction and stays non-replaying.
[Encoding reference, regressions and limits](rea/2.26.43.72-media-upload-http/README.md#finished-voice-seal-integration-2026-11-04-offline-validated).
No live account operations or server acceptance validation.

2026-11-04 — **Confirmed static: standard finished voice seal path.**
Pinned2.26.43.72 MFE/Lg0/Mvc factory chain plus exact interface definitions
close custom Ncy override for standard LKZ. Rust's missing separate seal
now has a bound native finished path; exact serialization/migration pending.
[Evidence](rea/2.26.43.72-media-upload-http/README.md#standard-finished-voice-processor-binding-2026-11-04).
No runtime byte change or live account request.

2026-11-04 — **Confirmed static: standard voice cipher digest boundary.**
Pinned2.26.43.72 MWH/LOh/Mzq/Mtd/LOv chain covers cipher output+10-byte
MAC before external seal. Constructor field mapping resolves digest order.
[Branch-limited evidence](rea/2.26.43.72-media-upload-http/README.md#digest-stream-includes-cipher-mac-excludes-transport-seal-2026-11-04).
Chunk/alternate adapters and operation/override binding remain open.

2026-11-04 — **Confirmed static: finished voice flags and hash formatter.**
Pinned2.26.43.72 plugin LKZ construction/writer census and Mzq/LOh/MAw
format chain retained; Android Base64 flag2 is explicit. Stream composition
and operation override remain untraced.
[Evidence](rea/2.26.43.72-media-upload-http/README.md#finished-voice-construction-and-digest-formatting-2026-11-04).
No runtime byte changes or live account traffic.

2026-11-04 — **Confirmed static: voice seal ownership/encoding boundary.**
Pinned2.26.43.72 MOu default LKZ processor, newsletter-ptt exclusion,
little-endian/UTF-8 helpers and final hash leaf retained. Rust has no
separate voice seal; concrete operation/override/hash encoding still open.
[Evidence and potential omission](rea/2.26.43.72-media-upload-http/README.md#seal-owner-excluded-kind-and-encoding-helpers-2026-11-04).
No runtime change or live request.

2026-11-04 — **Confirmed static: optional upload EOF metadata boundary.**
Pinned2.26.43.72 single-file predicate closed; Kif/MtQ factory/processor
leaves retained. Null processing differs from specialised sent/canceled seals.
[Evidence and operation-binding gaps](rea/2.26.43.72-media-upload-http/README.md#optional-eof-processor-and-single-file-predicate-2026-11-04).
No universal metadata requirement, runtime body change or live traffic.

2026-11-04 — **Confirmed static: route-bound raw upload body branch.**
Pinned2.26.43.72 K2k.A06/A01/A08 and JiH/LzO field mappings retain raw
versus multipart branch, stream offset and optional EOF processing boundary.
[Evidence](rea/2.26.43.72-media-upload-http/README.md#raw-stream-versus-multipart-body-writer-2026-11-04).
Kif/trailer and complete resume producers remain untraced; no runtime change.

2026-11-04 — **Confirmed static: upload HTTP transport/header boundary.**
Pinned2.26.43.72 JAS/JAT selection, route auth pair and JAS conditional
headers/framing retained; map producers/active branch binding remain open.
[Evidence and source-versus-native comparison limits](rea/2.26.43.72-media-upload-http/README.md).
No generic header additions, secret fabrication, runtime change or live traffic.

2026-11-04 — **Confirmed static: upload bytecode mnemonic gaps closed.**
Pinned2.26.43.72 MvO.ARB now decodes static boolean load63 and float
multiplyc8; literal arithmetic bytes verified against classes11.dex.
[Regenerated method, source, hashes and two regressions](rea/2.26.43.72-media-transfer-results/README.md#checked-upload-mnemonic-correction-2026-11-04).
Developer tooling only; no runtime transfer/retry change.

2026-11-04 — **Confirmed static: media result flags and cancellation.**
Pinned2.26.43.72 Ly8 layout, MXe factory matrix, MS4 entry predicates and
MtZ upload producer retained. Auth retry precedes terminal-flag handling.
[Evidence and exact bounded HTTP predicate](rea/2.26.43.72-media-transfer-results/README.md).
Full producer/idempotency reconstruction remains open; no runtime change.

2026-11-04 — **Confirmed static: media readiness and distinct delays.**
Pinned2.26.43.72 CW7 auth budget/threshold branches, Mtv.AXk/1C8 delay,
LIY inherited fixed delay and MvM wrapper construction retained.
[Evidence and untraced branch producers](rea/2.26.43.72-media-http-auth-refresh/README.md#readiness-and-distinct-delay-suppliers-2026-11-04).
No live execution or runtime scheduling change.

2026-11-04 — **Confirmed static: media auth retry loop edge.**
Pinned2.26.43.72 MS4.A03 calls Nd8.BkV and, on true, delays/reloads/rechecks
before NYs.Cb6. LIY forwarding and Mtv constructor A0N write retained.
[Branches, limitations and hashes](rea/2.26.43.72-media-http-auth-refresh/README.md#transfer-loop-continuation-2026-11-04).
No Rust retry policy changed; concrete operation replay remains untraced.

2026-11-04 — **Confirmed static: conditional media HTTP auth refresh.**
Pinned2.26.43.72 checked DEX closes the three direct A0S caller bodies,
changed-auth predicate and wait-budget accounting; flag producers and
actual transfer-loop replay remain unknown. [Evidence](rea/2.26.43.72-media-http-auth-refresh/README.md).
No live traffic, new REA envelope, or runtime change.

2026-11-04 — **Confirmed static: media disconnect and demand triggers.**
Pinned2.26.43.72 checked DEX identifies 1iM.A02 message0 cancellation,
0VM.A0T two-minute demand checks, and A0S401/403 gated refresh triggers.
[Evidence and bounded direct-reference scan](rea/2.26.43.72-media-retry-scheduling/README.md#disconnect-cancellation-and-additional-triggers-2026-11-04).
Caller conditions/account-switch teardown remain unknown. No runtime change.

2026-11-04 — **Confirmed checked DEX/derived:** media Fibonacci seed/cap,
zero initial delay, positive jitter range, Handler selector2/what0 dispatch,
success reset and auth-expiry-minus60s timer are retained with hashes. cb/be
mnemonic omissions are fixed with literal-byte regression tests; existing
retained arithmetic limitations are superseded without rewriting old output.
JADX provider Java was unavailable; these files are not new REA envelopes.
[Evidence and open lifetime/cancellation scope](rea/2.26.43.72-media-retry-scheduling/README.md).

2026-11-04 — **Confirmed source/implemented local policy:** old media route
producer stored matching IQ wire and receipt timestamp. Recovery now recognizes
possible507 bounds under pinned rules without retroactively binding a request or
assigning order. Offline command, correlation/time/status/expiry/reset-ambiguity
tests preserve raw history. Historical build provenance remains unknown; no new
APK or live observation is claimed.
[Contract and limits](media-network-outcomes.md#legacy-refusal-observations-and-offline-command-2026-11-04).

2026-11-04 — **Confirmed Rust/offline:** media throttle recovery now uses
complete validated receipt sequence and original receipt times. Missing waits can
be restored without restarting them; ambiguous saved/unsequenced waits cannot be
silently erased. Four encrypted-store tests cover restoration, ordering, ambiguity
and partial/corrupt history refusal. No new APK or live observation is claimed.
[Implementation and boundaries](media-network-outcomes.md#bounded-historical-wait-recovery-2026-11-04).

2026-11-04 — **Confirmed Rust/offline:** media refresh settlement now
atomically records local receipt order alongside projection/throttle effects.
Mixed-outcome, duplicate/reopen, counter-fault/cut and unchanged legacy-history
cases are tested. The number is local metadata, not a native wire attribute or
proof that historical waits were recovered. No new APK/live evidence is claimed.
[Implementation and limits](media-network-outcomes.md#durable-refresh-receipt-order-2026-11-04).

2026-11-04 — **Confirmed pinned static/implemented offline:** 1xY.Bvj
sets media throttle only for positive507 backoff (cap10800 seconds), resets it
for qualified503 and consumes the first valid nonzero pair. Prospective refresh
now persists/enforces that deadline atomically without automatic retry. Duplicate,
expiry and parser/reset tests are offline. Existing checked arithmetic has an
unsymbolized opcode; the addition remains attributed to recovered Java source.
[Evidence and limits](rea/2.26.43.72-media-route-lifetimes/README.md#explicit-media-refusal-throttle).

2026-11-04 — **Confirmed Rust/offline:** prospective media refresh context
now binds exact request bytes, prior projection, generation and send-time clock.
The existing explicit CLI query uses conditional last_id and atomic raw/effective
reply/current-pointer settlement. Seven real-store tests cover merge, persistence,
CAS/cuts/orphan/generation/digest guards and actual upload route selection. Pinned
0VM.A07/A0A/A0P remain the native reference; no new APK/live evidence is claimed.
[Implementation and remaining lifecycle gaps](media-network-outcomes.md#prospective-refresh-integration-2026-11-04).

2026-11-04 — **Confirmed Rust/offline:** pure media host projection now
implements pinned 0VM.A0A branch selection with exact prior hosts and incoming
metadata, preserving inputs. Cached new-route selection uses the shared helper;
caller claims do not establish durable prior authority. Three tests cover merge,
first/new branches and ambiguity refusal. No new APK/live evidence is claimed.
[Implementation and remaining wrapper](media-network-outcomes.md#pure-host-projection-implemented-2026-11-04).

2026-11-04 — **Confirmed static/support gap guarded:** 0VM.A0A conditionally
retains prior hosts on non-new routing responses; A0P returns the current object
without expiry filtering. Rust previously ignored is_new. Non-new cached replies
now refuse pending frozen prior-host context; native first-route false acceptance
was unsupported at that stage; the prospective wrapper now supports it.
Flag parsing/store-preservation tests are offline.
[Evidence and implementation boundary](media-network-outcomes.md#conditional-prior-host-retention-2026-11-04).

2026-11-04 — **Confirmed static/Rust defect fixed offline:** 0VM.A07 anchors
media deadlines at request handoff; Rust had replaced send reservation time with
response time. Explicit requested_at now survives the CLI reply commit and is
required for upload expiry. Historical unproven clocks refuse explicit-refresh
rather than being reconstructed. Seven retained REA methods plus checked request,
callback and exact field-write census close the base-origin question. No live use.
[Evidence and limits](rea/2.26.43.72-media-route-lifetimes/README.md#request-clock-origin-follow-up).

2026-11-04 — **Confirmed static/corrected:** pinned 2.26.43.72 1xY.CK4 uses
required long readers for both media lifetimes. The local missing-auth fallback
is refuted and removed; Java-compatible parsing replaces Rust decimal parsing.
Six complete REA method envelopes and checked DEX retain parsing, constructor and
conditional cache-restore boundaries. No live observation or full refresh claim.
[Evidence, hashes and unknowns](rea/2.26.43.72-media-route-lifetimes/README.md).

2026-11-04 — **Confirmed Rust defect/fixed offline:** invalid explicit media
auth lifetimes could inherit route TTL. Missing-only fallback now distinguishes
absent from malformed/overflowing values. Encoded-route encrypted-store tests
preserve all checkpoints and cover shorter auth expiry. No new APK/live evidence.
[Details](media-network-outcomes.md#explicit-auth-lifetime-validation-2026-11-04).

2026-11-04 — **Confirmed Rust persistence defect/fixed offline:** upload
HTTP status was discarded when response-body streaming failed before completion.
It now commits before streaming under response_headers_received, retaining the
reservation and blocking replay. Real-store tests cover header-only reopen and
existing-outcome protection. No new APK/live evidence is claimed.
[Transition and tests](media-network-outcomes.md#upload-response-header-persistence-follow-up).

2026-11-04 — **Confirmed Rust defects/fixed offline:** view-once retrieval
lacked ordinary-media account/config preflight and accepted unusable plaintext
hashes before GET. Both now refuse before reservation. Async real-store and
malformed-descriptor tests retain state. No APK or live observation is claimed.
[Details](media-network-outcomes.md#view-once-preflight-2026-11-04).

2026-11-04 — **Confirmed Rust defect/fixed offline:** ordinary media GET
lost definite HTTP refusal status behind outcome_unknown. Numeric refusal
persistence now survives reopen, reports later without a request and protects
other states. Real encrypted-store tests cover 302/403/404/429/503 and missing or
already-settled reservations. No fresh APK or live observation is claimed.
[Local evidence and untraced native boundaries](media-network-outcomes.md).

2026-11-04 — **Confirmed Rust parsing mismatch/fixed offline:** write version
acknowledgements and 409 codes now use the existing Java-compatible signed-decimal
helper. Alternate BMP spellings cannot change prepared-state authority. Real-store
acceptance/refusal tests preserve exact transactional invariants. Native reference
is the retained pinned 2.26.43.72 F2T/0Tp reader, not fresh live traffic.
[Details and tests](app-state-patch-atomicity.md#java-decimal-acknowledgement-and-conflict-spelling-2026-11-04).

2026-11-04 — **Confirmed Rust defect/fixed offline:** inline 409 catch-up
could discard an accompanying snapshot reference. It now runs shared structural
classification/header preflight before held/key reads and refuses unsupported
external retrieval. Complete-checkpoint preservation tests cover snapshots,
duplicate versions and later mixed external data. Native reference remains pinned
2.26.43.72 F2T.A00; no new APK/live evidence is claimed.
[Details](app-state-catchup-atomicity.md#whole-collection-preflight-before-conflict-verification-2026-11-04).

2026-11-04 — **Confirmed Rust defect/fixed offline:** the existing verified
409 chain discarded decrypted actions while advancing integrity state. It now
retains them encrypted in the same transaction, with exact immutable stored
request/reply digests. Independent literal SET/REMOVE tests cover journal reopen
and rollback; actual durable action ingestion checks digest provenance, pending
release and no replay. No new APK evidence or live observation is claimed.
[Correction and bounded contract](app-state-catchup-atomicity.md#authenticated-action-retention-2026-11-04).

2026-11-04 — **Confirmed Rust/offline:** successful held-fetch response
inspection now derives binding from sealed prospective store provenance rather
than caller claims. Exact preserved reply comparison and one-read settlement
checks prevent a fabricated response from borrowing another admission. Reopen,
changed-current-state, no-advance acknowledgement, missing-record and unrequested
collection cases are tested. No new APK claim or live account operation.
[Implementation boundary](app-state-held-fetch-admission-design.md#preserved-response-provenance-inspection).

2026-11-04 — **Confirmed Rust/offline:** prospective held-fetch admission now
seals the exact queued request and complete existing baselines in the queue
transaction, then rechecks them at reservation. Shared refusal backoff covers
both fetch kinds. Safe inspection reads admission and settlement status together.
The encrypted action-journal primitive is staged without a production caller;
it does not establish authenticated response application. No fresh APK evidence
or live account operation is claimed. **1,223 workspace/all-feature tests** and
strict Clippy passed. [Implementation and remaining boundaries](app-state-held-fetch-admission-design.md).

2026-10-13 — **Fixed/offline (PR #4 review, merged as `575d249`):** three
Codex findings on the merged work are fixed, each with a test. A store path
that is a dangling symlink, or appears between the existence check and the
private create, is now refused instead of opened through (which created the
link target with the default mode). The email-answered marker that
`resume-email-challenge` needs is committed in the same transaction as
`RegistrationOutcomeUnknown`, so a crash can no longer strand an attempt the
server definitely answered. CI now routes every `docs/` file a crate compiles
in or reads at runtime (`DOCS_INPUTS`, including `docs/cli.md` and
`docs/api/openapi.json`) to that crate's checks; a planner test scans for
`../docs/` paths so a new one cannot be missed.
[Audit](registration-safety-audit.md), [CI routing](../ci.md).

2026-10-13 — **Corrected/offline (independent review of the descriptor
simulator and edit trace):** no committed decode was wrong, but the simulator
could have printed a wrong pairing as exact in three synthetic cases. It now
refuses instructions rendered without their destination register and unknown
opcodes, follows a helper only on an exact full descriptor (a shorty-only call
is not followed), poisons an array that escapes into another array, requires
the leading `bitFieldN_` slots to match the descriptor's presence words, tracks
headers per method, and reports APK scan failures. BSp (PollEncValue) moved
from exact to literal-order only; the sticker has no retained pairing; the
Bpt/Bps edit values are written, not proven final. A `cargo doc` failure in the
new module (unescaped `<…>`) is fixed. [Notes](apk-analysis.md#field-numbers-from-retained-bundles-retained-proto-fields-2026-10-13).

2026-10-13 — **Changed/offline (research tooling):** `xtask apk-proto-fields`
now uses the same exact objects-array simulator as `retained-proto-fields`,
reading static helper bodies from the APK, instead of pairing names in method
order; `--literal-order` keeps the old pairing, labelled unverified. Not yet
run against an APK (none in this container).
[Notes](apk-analysis.md#reading-protobuf-field-numbers-apk-proto-fields).

2026-10-13 — **Traced/static (sender edit values, retained bundles):** the
send runnable `DNu` keys a scheduled send on the message's own edit field
(`178.A00`), and the retained writers of that field give sender revoke 7,
admin revoke 8, poll edit 1, and undo-keep of one's own message 7 (keep 0).
Text-edit and pin values are not retained. The job constructor mapping and
the stanza writer remain inferred links. This client's `keep --undo` omits
the value (recorded divergence; the one retained receiver check, `D9L`, does
not look at it on undo). No sender
changed. [Trace](unknowns.md#outgoing-revoke-and-edit-2026-10-13).

2026-10-13 — **Implemented/offline (research tooling):** `xtask
retained-proto-fields <file>` reads protobuf-lite field numbers from committed
research bundles, for sessions without the APK. It follows the code that builds
the descriptor's objects array, including retained helpers from the same APK,
and refuses (naming the missing helper) rather than pairing names in method
order. That order is refuted for DocumentMessage: helper `Ayp.A1K` writes
`fileSha256_` at objects[4]. Eleven classes decode exactly (twelve before
the same-day review correction below). They include
Message, Image/Document/Video/Audio and the history Conversation. The project's
media, PreKeySignalMessage, ADV and poll-option encoders agree with them.
`pinInChatMessage` is 63, but pin sending stays blocked on the unretained `BVD`
descriptor. [Evidence](apk-analysis.md#field-numbers-from-retained-bundles-retained-proto-fields-2026-10-13).

2026-10-13 — **Fixed/offline (CLI parity):** `registration status --json` now
prints the saved attempt as redacted JSON, the same as `lab-signup status
--json` and the `GET /v1/signup/status` store route (one shared redaction);
`--json` used to be ignored there.

2026-10-13 — **Implemented/offline (HTTP parity: registration status):** `serve
--store-routes` also answers `GET /v1/signup/status` with the saved
registration attempt exactly as `lab-signup status --json` prints it (phone and
consent secrets redacted), or JSON null, so an agent can follow onboarding
state without the CLI. [API README](../api/README.md#store-routes-opt-in).

2026-10-13 — **Corrected/offline (third registration review):** a review of
the exact-restore change with probe tests found it sound, and one gap: an
unreadable 5xx or 429 email-code answer still allowed a second email request.
Only a definite 4xx (not 408 or 429) now counts as a refusal. The default
(`UnsupportedFlow`) flow now restores a challenge or existing attempt it did
not create instead of blocking over it; the `accounts new` scan is bounded and
refuses past its limit; failed-restore and unsent-attempt refusals say why.
Recorded: records restored before the exact restore keep one extra counted
preparation each. [Audit](registration-safety-audit.md#third-review-2026-10-13).

2026-10-13 — **Implemented/offline (HTTP parity: store inspect):** `serve
--store-routes` now also answers `GET /v1/store` with the `store inspect`
report (table counts, generation, registration status, checkpoint kinds), the
same JSON as the CLI from one shared function. A checkpoint kind is printed by
name only when it is a lowercase record name and not hex-like; any other key
prefix (for example a key without `:` whose text is an address or digest) is
counted as `other`, for the CLI too. [API README](../api/README.md#store-routes-opt-in).

2026-10-13 — **Corrected/offline (second registration review):** a second
read-only review of the registration commits found no request that may have
left being restored as not sent and no latch released after a send, but
fixed: an email-code answer that was unreadable under HTTP 2xx could be
re-requested with no wait (now only an HTTP error status allows the
lost-request resume); a not-sent restore now writes back the exact attempt from
before the reservation, history included, so a request that never left is not
counted in `client_metrics.attempts` or consuming history (and the general
`preparing -> unregistered` edge is gone); resuming `begin` with another method
is refused instead of silently ignored; another profile's flow cannot answer a
saved challenge, and `reset`'s local-block exemption excludes lab attempts and
any that reached a challenge; a store holding another number's material is not
claimed; `accounts new` scans every JSON beside the account directories; a
failed restore reports its cause. [Audit](registration-safety-audit.md#second-review-2026-10-13).

2026-10-13 — **Fixed/offline (registration hardening):** new store files are
created 0600 (records were already encrypted; the file followed the umask);
`registration status`, `reconcile` and `reset` no longer create a missing store;
and the CLI no longer suggests the always-rejected `request-email-code --email`.
Recorded open: one wrong OTP (`mismatch`) leaves the attempt blocked with no way
back to code entry; the APK's handling of `mismatch` and the unit of
`guess_wait` are not traced, so no re-entry command is built on a guess.
Likewise a `/v2/code` `ok`/`attached` answer is blocked and its `login`
discarded: 2.26.37.71 `A09` maps both statuses and reads `login`, but what the
app then does is not traced.
[Audit](registration-safety-audit.md#plausible-not-verified).

2026-10-13 — **Corrected/offline (review of the registration safety fixes):**
a second review found gaps in the fixes below, now corrected. A `/v2/exist`
preflight that had reached the server was restored as "not sent" when the code
request after it failed to connect, so a retry sent another preflight with a
higher attempts counter; both requests are now built before either is sent, and
a failure after a sent preflight records an unknown outcome. A diagnostic or
correction that never left now restores its refusal and the saved material
together and releases its one-shot (before, the corrected token stayed saved,
so the released one-shot refused). `resume-email-challenge` resumes a lost email-code request only when
the server's answer arrived unreadable, never after a timeout or reset.
`lab-signup begin` refuses a store with no attempt unless it was created for
that number (`accounts new` and `--create-store` record the owner), so an empty
store another command created, or a mismatched `--config`, cannot start a fresh
identity. `accounts new` compares numbers canonically and is not blinded by
one malformed profile. A first `begin` that never left now rests
`unregistered`, not as a fake elapsed `cooling_down`, and `registration reset`
clears a local `ProfileIncomplete` block left by an older pinned version. A
refusal without `retry_after` now keeps its longest SMS/voice/email wait
(`ServerRefused.not_before`), which the one-shots, the email-challenge resume
and the `exist` probe enforce; before, it was dropped. Open: not-sent failures
consume bounded history. [Audit](registration-safety-audit.md#review-corrections-2026-10-13).

2026-10-13 — **Fixed/offline (registration: stores and status):** `lab-signup
begin` no longer creates a missing store unless `--create-store` is given;
`accounts new` creates the profile's store and refuses a number another profile
configures. A forgotten `--profile` or a mistyped path can no longer mint a
fresh identity for a number with a real store. `registration begin` no longer
silently replaces another number's or profile's attempt, and `registration
reset` no longer deletes a lab attempt. `accounts status` now shows the real
registration state. The `exist` probe is refused mid-request, once registered
and during a cooldown. Profiles made before this need `begin --create-store`
once. [Audit](registration-safety-audit.md).

2026-10-13 — **Fixed/offline (registration: requests that never left):** a
registration request that could not be built, or whose connection failed
before any byte was written, was recorded as an unknown outcome. The HTTPS flow
cannot reconcile that, so a network outage on `begin` stranded the store,
leaving a fresh identity as the only way on. Such failures now return
`RequestNotSent` and restore the attempt's previous state (a fresh `begin`
resumes later with the same attempt and keys). Failures that may have reached
the server still record an unknown outcome. That connect errors occur before
writing is inferred from the HTTP library. [Audit](registration-safety-audit.md).

2026-10-13 — **Fixed/offline (registration safety audit, waits and email
recovery):** `resume-email-challenge` is now allowed once per recovery kind per
attempt, so an uncertain email-code request can no longer be re-sent in a
loop; the `challenge` refusal's `email_otp_wait` is kept (inferred meaning,
longer value); and a shorter SMS/voice wait no longer replaces a longer
`retry_after`. Open: a `/v2/code` refusal that stays blocked does not keep its
SMS/voice wait (needs a saved-state field). [Audit](registration-safety-audit.md).

2026-10-13 — **Corrected/offline (review of the receive hardening):** the
first undecodable-frame change kept receiving after any undecodable frame, so an
undecodable `stream:error` or `failure` would have been followed by
reconnects, retrying a session the server had ended. The reader now reads such
a frame's name leniently: a terminal stanza stops without reconnecting, and so
does a server close right after a stanza whose name is unreadable. The lab
group-send window no longer answers a retry receipt whose content was refused.
The memory-amplification note is corrected and recorded open (a burst of
crafted server frames could exhaust memory). [Audit](replay-safety-audit.md#network-input-panic-audit-2026-10-13).

2026-10-13 — **Fixed/offline (registration safety audit, first two):** a
read-only audit of the onboarding HTTPS path found that `lab-signup status
--json` printed the consent context's intermediate login and parent consent URL
(now redacted at any depth), and that the code-request diagnostics and
`retry-corrected-token` accepted a `bad_param`/`bad_token` refusal from an OTP
submission or other request, where they would have asked for a new SMS (now
they require the refusal to come from a code request). The remaining findings
are listed and being verified. Nothing was sent. [Audit](registration-safety-audit.md).

2026-10-13 — **Implemented/offline (lab windows):** `lab-transport` receive,
post-send and group-send windows now capture a frame that does not decode and
keep reading, like `listen`, instead of ending the window. Login, handshake and
the bounded reply waits (send ACKs, iq and key replies) still need a decodable
frame. [Audit](replay-safety-audit.md#network-input-panic-audit-2026-10-13).

2026-10-13 — **Implemented/offline (inspection):** `whatsapp-cli undecodable`
(also `GET /v1/inbox/undecodable` under `serve --store-routes`) lists the frames
`listen` captured because it could not decode them: capture digest, error class
then and now, size, epoch. "Now" re-runs the current decoder, so a dictionary
update shows which captures it would read. Read-only and redacted; nothing is
reprocessed or acknowledged. [Audit](replay-safety-audit.md#network-input-panic-audit-2026-10-13).

2026-10-13 — **Fixed/offline (network-input panic audit):** a read-only audit
for panics and aborts reachable from received data found one. Nested JID tags
made the binary decoder recurse until the stack overflowed: a 4 KB compressed
frame from the server would abort the process, uncaptured, at every login.
The decoder now refuses those tags before reading them. Also, a frame the
decoder refuses (for example a newer dictionary token) no longer ends `listen`
uncaptured: it is kept as `inbound-undecodable` with a fixed error class and
skipped. No other reachable panic was found. Only the server can send such
frames. [Audit](replay-safety-audit.md#network-input-panic-audit-2026-10-13).

2026-10-13 — **Checked/offline (live-shape regression sweep):** every stanza
shape the docs record as seen live was compared with the current receive code.
No other regression like the status-receipt one was found. Fixed on the way:
`lab-transport --receive` and the lab group-send window now validate receipt
content like `listen`; a devices or prekey-count notification without `id` no
longer ends `listen`. Open: the 2026-10-11 ACK and confirmation encoding changes
have never been checked live; the next live receive window should confirm no
redelivery after `confirmed=true`. [Sweep](replay-safety-audit.md#live-shape-regression-sweep-2026-10-13).

2026-10-13 — **Corrected/offline (review of the new store routes):** no
security finding. Three protocol docs and the feature-parity exposure note still
said these inspections were not served over HTTP; corrected. The key-query
schema lacked the `unsupported_bundle` reason the library already returns.
A damaged saved group send now answers 422 with its fixed reason, like key
queries and own LID, and the own-LID refusal reasons are now fixed by type.
Tests now cover the 128-byte boundary, 405, the disabled 404, 400-before-401,
non-null answers and the OpenAPI parameter names. [Store routes](../api/README.md#store-routes-opt-in).

2026-10-13 — **Corrected/offline (review of the receive-audit fixes):** the
first version caught every refusal from recording a receipt or token
notification, which would also have hidden faults in our own stored rows (a
corrupt outbox entry) behind silently unacknowledged receipts. Now only pure
wire-content validation, shared with the recording code, is caught; stored-row
faults still end the session. The review also found that status receipts had
been ending `listen` since the 2026-10-10 aggregate branch: the shape observed
live has an empty `participants` list, which that branch refuses. It is
recorded and acknowledged again, and an aggregate with entries is no longer
filed as a status view. Own-LID devices handling now has a test. Not seen live.
[Corrections](replay-safety-audit.md#review-corrections-same-day).

2026-10-13 — **Implemented/offline (HTTP):** `serve --store-routes` now also
serves four counts-only inspections, the same library calls and JSON as the
CLI: `GET /v1/diagnostics/own-lid`, and `/v1/groups/receipts?request_id=`,
`/v1/groups/key-queries?request_id=` and `/v1/groups/refresh-status?id=`.
Each parameterized route accepts exactly that one unencoded, ID-shaped
parameter and refuses anything else with 400 before the store opens; nothing
saved answers `null`; a key-query record that cannot be counted, or own-LID
sources that disagree, answer 422 with the library's fixed reason. The refresh-status JSON is now one shared function, so
CLI and HTTP cannot drift. Five served operations still said "CLI only; no
HTTP route" in OpenAPI; those stale sentences are removed. No route sends,
queues or exposes an address or message content. [Store routes](../api/README.md#store-routes-opt-in).

2026-10-13 — **Corrected/offline (receive-path audit):** a read-only audit of
the receive loop found that a receipt, `privacy_token` notification or
`pq_count` whose content we refuse, or an own-companion message with an
undecodable app-state key share, ended the session before its ACK. The server
then redelivered it at every login, so every later `listen` ended on the same
stanza. These are now logged, captured without ACK, and receive continues.
A viewed group view-once is a duplicate on redelivery, and our own LID no
longer queues a contact device sync. Four lower findings are recorded open.
Not seen live. [Audit](replay-safety-audit.md#receive-side-audit-2026-10-13).

2026-10-13 — **Measured (cross-version coverage):** new `xtask apk-coverage`
reports what the committed research bundles cover. The pinned 2.26.36.75 has no
retained bundle; 2.26.37.70 has one (21 core topics); 2.26.37.71 has 166.
Only those 21 transport/Noise/registration topics have evidence from two
releases. 64 of 142 library files are cited by no bundle topic (many cite APK
classes in comments or REA envelopes instead). [Coverage](apk-coverage.md).

2026-10-13 — **Corrected/offline (security review of HTTP store routes):** a
request with no headers made the connection task panic (an empty header slice);
it is now answered and tested. The token file is guarded before it is written and
also removed on SIGTERM/SIGHUP; 503 is reserved for a store held by another
command and other read failures answer 500 `store_read_failed`; the bearer
scheme is case-insensitive. Separately, `ProtocolStore::open` without `create`
no longer bootstraps a new store (with fresh keys) from an existing empty file
or a database without the store schema; it refuses and leaves the file
untouched. Docs now state the remaining caveats (killed server leaves the token
file, per-request opens can make a starting command fail, `peer_ticket` is a
guessable hash, use `127.0.0.1`). [API README](../api/README.md#store-routes-opt-in).

2026-10-13 — **Implemented/offline (CLI):** `outbox-send` private JSON now
sends direct-chat replies (text, image or voice note) as well as group replies;
a direct quote's author must be the contact or us. The comparison path refuses
any direct quote instead of silently dropping it (`require_comparison_support`).

2026-10-13 — **Implemented/offline (HTTP):** `serve --store-routes --token-file`
serves read-only store inspections (six after the same-day follow-up:
receipts, prekeys, suspended sends, actions, identity conflicts, group
preparations) over loopback with a per-start bearer token kept in a new 0600
file, the store opened per request. Smoke-tested with
the real binary on a temporary store (401 without the token, redacted JSON
with it, token file removed on shutdown, token never printed). Default `serve`
is unchanged; nothing that sends is served. [Design](../feature-parity.md#http-api-parity).

2026-10-13 — **Corrected/offline (review of quoted media sending):** as first
committed, group JSON image/voice-note replies were still refused by the JSON
reader (it required `text` on every reply), so they were reachable only through
the library despite the docs; fixed and tested through the reader. JSON
reactions and `quoted_text` are again refused before the send-path claim.
Direct quotes are now built at selection, so a wrong-chat parent, missing own
identity or over-32-KiB quote fails locally instead of after the key query.
Class names corrected to 2.26.37.71 `BZD`/`BZ0`/`BZL`. [Details](quoted-media.md#sending-2026-10-13).

2026-10-13 — **Implemented/offline:** an uploaded image or voice note can reply
to a message (direct and normal group outbox): the media message carries the
same `BWx` contextInfo a text reply does, at the field the receive path reads.
`outbox-send` group JSON now accepts replies and, after a reader fix, group
image/voice note JSON, which had been refused by the shared reader despite being
documented. Our sent messages never carry `contextInfo.expiration` (recorded
as an unknown). No live quoted media send.
[Sending](quoted-media.md#sending-2026-10-13), [expiration gap](unknowns.md#outgoing-disappearing-context-2026-10-13).

2026-10-13 — **Corrected/offline (second review and replay audit):** a damaged
same-key held session no longer makes identity-refresh application error on
every connection start; it defers as `held_session_inconsistent` (preview
reports the same) and the row is kept. Once-only retry receipts now also probe
the sender's other PN/LID address, so a redelivery under the other address
cannot queue a second retry. The reconnect budget resets only after an
activated connection stayed up for `max_delay_ms`, bounding activate-then-drop
login loops. The hybrid receive helper refuses companion senders itself.
Status-reply quotes (`remoteJid`) remain unread and unverified.
[Held sessions](hybrid-identity-refresh.md#damaged-held-sessions-2026-10-13), [retry aliases](messaging-transport.md#retry-once-only-across-pn-and-lid-2026-10-13), [reconnect](messaging-transport.md#reconnect-budget-stability-2026-10-13), [audit](replay-safety-audit.md).

2026-10-13 — **Corrected/offline (independent review):** hybrid receive now
classifies missing or unusable local keys (including an unparsable KEM row)
as Undecryptable like classic; a group companion's hybrid pairwise child is
skipped without blocking its `skmsg`; fan-out/batches refuse attestation or
malformed-`identity_auth` PQ rows like the direct path (`unsupported_bundle`);
unusable outbox PQ replies suspend the intent with a reason instead of ending
the session. [Inbox](kem-prekey-inventory.md), [fan-out](hybrid-fanout.md), [outbox](hybrid-outbox.md).

2026-10-13 — **Implemented/offline:** incoming media that quotes another
message (BWx stanzaId/participant/quotedMessage in contextInfo 17) now projects
as a media event with `quoted`, attributed by the same rule as text replies,
instead of an opaque placeholder. Stanza-less quotes stay opaque; edits and
sending are unchanged; no live sample. [Behavior and tests](quoted-media.md).

2026-10-13 — **Implemented/offline:** the lab group-send retry answer now
derives a native hybrid session for a device whose fresh directory row carries
a verified `pqkey`, via the new `signal::hybrid::selects_pq_key` selector and
the shared fan-out derivation. Classic rows are unchanged; no live retry answer.
[Lab retry answers](hybrid-fanout.md#lab-retry-answers-follow-up-same-day).

2026-10-13 — **Implemented/offline:** a verified PQ directory reply for a
contact with a held classic session no longer errors the direct send; the held
ratchet continues as on the classic path (no PQ upgrade, which stays unported).
Well-formed log-only `identity_auth` no longer blocks hybrid sends. A coordinator
test that nested `pqkey` inside `skey`, and so never reached the hybrid branch,
is corrected. [Follow-up and correction](hybrid-outbox.md).

2026-10-13 — **Implemented/offline:** device-key batches now answer rows whose
`pqkey` verifies, and group/status fan-out sends those devices native 0x44
hybrid distributions. Previously one PQ row blocked the whole send. Primary,
companion, cached-continuation and signed-replacement cases open with the
device's own keys. Invalid KEM signature/encoding still blocks. `pqsupport` is
not sent; no APK was available; no live test.
[Behavior, evidence and tests](hybrid-fanout.md).

2026-10-13 — **Implemented/offline:** identity-refresh preview and application
now stage a version-4 hybrid session from a verified primary `pqkey` reply:
changed keys derive fresh, admitted same-key refreshes archive held state. The
refreshed record opens a 0x44 exchange the peer authenticates. Invalid PQ defers
as `pq_bundle_invalid`; companion/attestation PQ stays deferred. `key_validation`
now reports ML-KEM-1024 encoding. No APK was available; no live test.
[Behavior, reused evidence and tests](hybrid-identity-refresh.md).

2026-10-13 — **Implemented/offline:** the normal direct and group inboxes now
route primary PN/LID version-4 KEM prekeys through the hybrid receive path.
Matching held sessions decrypt without keys; fresh bootstrap reads a local
ML-KEM-1024 row and consumes non-last-resort rows atomically. Replies decrypt as
0x44 msg. KEM generation/upload stays untraced and absent; own-device/companion
KEM is refused unchanged. No APK was available; no live test.
[Inventory, routing table and encrypted tests](kem-prekey-inventory.md).

2026-10-12 — **Implemented/offline:** staged incoming hybrid prekeys now
bootstrap from exact local EC/KEM keys and authenticate nested ciphertext before
reporting consumption. Matching current/archived sessions decrypt without prekey
lookup; last-resort KEM and ordinary ID 16777215 remain reusable. Encrypted tests
cover pin policy, delayed envelopes, replay, malformed keys and account binding.
Normal inbox/key-inventory integration remains open.
[Native lookup/deletion evidence, API and tests](hybrid-incoming.md).

2026-10-12 — **Implemented/offline:** normal outgoing key replies now explicitly
select verified hybrid material for primary PN/LID peers. Fresh first-use and
policy-approved identity replacement produce native 0x44 candidates through the
existing atomic outbox transaction; held hybrid records continue without reset.
Encrypted fixtures verify request/attempt preservation and no replay after reopen.
Companion/auth/attestation, classic migration and incoming bootstrap remain open.
[Coordinator behavior, evidence and tests](hybrid-outbox.md).

2026-10-12 — **Implemented/offline:** fresh exact-recipient hybrid bundles now
verify both signed EC and original KEM value bytes, validate ML-KEM-1024 encoding,
and stage a native version-4 initial message/session candidate. PN/LID encrypted
fixtures cover optional ordinary prekeys, malformed metadata and pin/account
refusals. Directory authority/adoption and incoming bootstrap remain unfinished;
normal PQ guards are unchanged. [API, evidence and scope](hybrid-bundle.md).

2026-10-12 — **Implemented/offline:** the shared send adapter now prepares
native version-4 ciphertext from held hybrid records after account, registration,
peer-pin and ratchet-pair checks. Text and encoded payload exchange works through
normal APIs; an encrypted-store outbox test commits the same request/wire ID with
DeliveryUnknown and refuses replay after reopen. Fetched signatures remain
required. Fresh PQ directory/receive/bootstrap guards and live gaps remain.
[Shared send integration, ownership guards and coordinator test](held-hybrid-send.md).

2026-10-12 — **Implemented/offline:** staged hybrid Alice/Bob records now
perform real EC/KEM bootstrap, native 0x44 encryption, authenticated reply and
continued ratcheting with/without an ordinary one-time key. Pending KEM metadata
survives canonical reopen and clears after a valid reply; counter leasing and
source-record preservation remain effective. Normal client adoption/send/key
inventory is not connected; PQ guards and live-interoperability gaps remain.
[Session API, native state/chain traces and encrypted exchange test](hybrid-session.md).

2026-10-12 — **Implemented/offline:** low-level Signal wire encoders now
construct native 0x33/0x44 version bytes before HMAC and retain paired outer KEM
fields. Complete bytes match 24 independent nested and eight outer fixtures;
version-3 backend equivalence and version-byte tamper authentication are checked.
Existing application/backend send paths and PQ guards remain unchanged; hybrid
state/coordinator integration and live interoperability remain unfinished.
[Native schemas/MAC, encoder API and independent fixtures](signal-wire.md).

2026-10-12 — **Implemented/offline:** incoming prekey APIs retain original
paired KEM ID/ciphertext metadata and refuse incomplete pairs. A valid classic
MAC no longer makes an envelope with ignored KEM fields consumable. Direct/group
receive defer before conflict observation or prekey lookup; all saved tables and
journal remain unchanged after capture and reopen. Native paired fields and
0x44 version-4 encoding are traced; outbound/hybrid integration remains open.
[Wire evidence, metadata API and preservation tests](prekey-kem-wire.md).

2026-10-12 — **Implemented/offline:** initial hybrid root/chain derivation
now follows the APK's discontinuity prefix, ordered EC agreements, optional
one-time agreement, final KEM secret and exact hybrid HKDF label. Native extract,
counter-one expansion and 64-byte split are retained; Rust outputs are checked
against 16 independent synthetic HMAC fixtures. This is a low-level KDF helper;
PQ admission, pending wire/state integration and live sessions remain unfinished.
[Native construction, independent fixtures and API](hybrid-kdf.md).

2026-10-12 — **Implemented/offline:** experimental `signal::kem` exposes
Rust ML-KEM-1024 generation, encapsulation and expanded-key decapsulation using
pinned RustCrypto with zeroization and redacted secrets. The same 16 APK/NIST
vectors exercise serialized keys, ciphertexts and shared secrets. This is a
primitive API, not PQ session admission; all current bundle/refresh guards remain.
Hybrid sessions and live interoperability remain unfinished.
[Primitive API, dependency limits and tests](rust-kem.md).

2026-10-12 — **Confirmed/offline execution:** the pinned APK's native
ML-KEM-1024 arithmetic/hashing matches 16 independent NIST FIPS203 cases:
key generation, deterministic encapsulation and valid/modified-ciphertext
decapsulation. Only libc memory/allocation and once initialization are modeled.
No JNI, native RNG, account or hybrid session execution is claimed. The selected
primitive vectors pass; Rust integration and live interoperability remain open.
[Native execution, NIST provenance and reproducer](native-kem-vectors.md).

2026-10-12 — **Confirmed/static:** native JNI public-key length dispatch calls
ML-KEM-512/768/1024; the Java CRYSTALS-KYBER-1024 HKDF label stays unchanged.
The Java seed array is length-checked but never consumed; native encapsulation
requests its own 32-byte randomness. Exact JNI widths and import/export thunks
are retained with a hash-bound reproducer. Primitive equivalence and Rust hybrid
sessions remain unverified; no account or network change.
[Native dispatch, sizes and randomness](native-kem.md).

2026-10-12 — **Implemented/offline:** identity refresh diagnostics now expose
redacted PQ signature verification over the original directory value bytes,
using the returned candidate identity. Native KEMPublicKey stores/returns those
bytes unchanged; the hybrid HKDF suite label names CRYSTALS-KYBER-1024, while
JNI algorithm revision/format and hybrid sessions remain unported. A verified
signature is not KEM validity or adoption: preview/application stay deferred,
all saved state is preserved, and the consumable PQ bundle guard remains active.
[Native byte domain and diagnostic/admission tests](pq-signature.md).

2026-10-11 — **Implemented/offline:** selected directory PQ key input now
fails explicitly instead of silently becoming a consumable classic bundle.
Direct preparation and device batches share the boundary; unrelated devices
and classic metadata inspection retain their behavior. Identity preview/application
still defer PQ without derivation or adoption. Saved keys, sessions, IDs, uncertain
sends and waits stay intact. Full native KEM and fresh live interoperability remain
unfinished; this is an unsupported-backend guard, not PQ support or native policy.
[Native bundle path, backend audit and preservation tests](pq-bundle-boundary.md).

2026-10-11 — **Implemented/offline:** own ordinary video/GIF and document
caption edits now preserve original attachments and document filenames. Native
kind equality and the one-way video-to-GIF-parent exception are enforced using
saved synchronized payloads. The deeper decoder trace corrects image removal:
ordinary media edits require a nonempty caption regardless of own ownership.
Empty/missing edits decline without receive-state mutation; Unicode and whitespace
are preserved within the existing byte bound. Peer/special/native media hooks,
modeled document title, large Unicode payloads and fresh live delivery remain open.
[Decoder context, kind and preservation tests](media-caption-edit.md).

2026-10-11 — **Implemented/offline:** authenticated own-device ordinary image
caption edits now emit Edited while retaining the original sealed attachment
reference. Replacement keys/URLs do not redirect the attachment. The later
[ordinary caption decoder correction](media-caption-edit.md) rejects caption removal;
the original permissive claim is superseded. Original parent bytes, send facts, IDs and waits remain unchanged.
Kind, terminal and ordering guards still apply; other media, peer configured
window, special/orphan/expiry lifecycles and fresh live delivery remain open.
[Native media clone and receive tests](image-caption-edit.md).

2026-10-11 — **Implemented/offline:** held authenticated own-device sessions
receive ordinary text edits as MessageUpdated/Edited for loaded own text parents
and attributed outgoing text intents. Parent chat/direction/ID stay intact;
embedded ownership metadata cannot select another owner or chat. Strictly older
edits and edits after an own revoke stay opaque with atomic ratchet/capture
commit; equality is accepted. Existing confirmation, reopen deduplication and
clearing paths remain effective, and uncertain send facts/IDs/waits stay unchanged.
Peer configured-window resolution, media/special/orphan/expiry lifecycles and
fresh live edit interoperability remain unfinished or unverified.
[APK decoder, ordering and receive tests](message-edit.md).

2026-10-11 — **Implemented/offline:** remote status confirmations now omit
sender type; the actor adds it only for participants matching saved own PN/LID,
including own companions. Bp1.A05 is traced through parsed fromMe and native
BO0, with all direct field writers retained. Peer ownership/alias claims do not
select this branch. Invalid own-LID authority refuses without repair. Broader
status admission, broadcast-list class and fresh live acceptance remain open.
[Ownership flow, contextual actor and source/guard tests](status-ownership.md).

2026-10-11 — **Implemented/offline:** message delivery/read/opened confirmation
addresses now match native primary devices: PN zero uses JID_PAIR and LID zero
uses AD. Factory/subtype and direct writer closure refute the old attribute-copy
assumption. Companion devices and raw incoming routes remain intact; durable
LID reads settle plain ACKs after reopen without uncertain replay. Full native
admission, hosted/interop and fresh live acceptance remain open.
[Factory, writer closure and regression/recovery checks](message-primary-device.md).

2026-10-11 — **Implemented/offline / evidence correction:** status delivery
confirmations no longer send sts=65535. Exact Cp2.A06 DEX loads signed -1;
Cp2.A00 emits sts only for a positive long. The old signed-literal explanation
was wrong and is replaced with full descriptors and independent REA evidence.
Original IDs, poster device and sender/class fields remain intact. Bp1.A05
producer policy, primary device wire normalization and live acceptance remain open.
[Exact producer/gate, runtime fix and trace boundaries](status-receipt.md).

2026-10-11 — **Implemented/offline:** identity, device-list, count, group-dirty
and privacy-token ACKs now share the APK-derived zero-device PN/LID user-pair
conversion with receipt ACKs. Literal-wire regressions reproduced identity and
all four other notification failures. Durable control effects and nonzero device
routes remain intact; the identity reply diagnostic no longer falsely says its
ACK is deferred. Full class grammar/native codecs and live acceptance remain open.
[Shared emission, source evidence and regression coverage](receipt-primary-ack.md).

2026-10-11 — **Implemented/offline:** receipt ACK `to`/`participant` now use
JID_PAIR for decoded binary zero-device PN/LID users. Complete writer override
censuses distinguish these users from primary DeviceLid objects, which may still
require AD. Input capture, nonzero device paths and the global encoder remain
intact. Hosted/text-zero/JNI class fidelity and live acceptance remain open.
[Writer census, runtime correction and reproduction](receipt-primary-ack.md).

2026-10-11 — **Implemented/offline:** APK AD JID decoding now uses LID/hosted
flag bits instead of inventing an agent from other flag values. The encoder
refuses nonzero agents in AD fields. The actual Java reader omits a zero device
suffix before JID construction, so zero-component binary AD user recipients
are admitted; literal text :0 remains a device-class mismatch. Native JNI/interop
codec paths and full self-status handling remain open; no live request.
[Decoder/writer evidence, correction and checks](ad-jid-decoding.md).

2026-10-11 — **Implemented/offline:** receipt recipient admission now follows
UserJid rather than generic Jid. Recognized group/status/device recipients are
omitted from ACKs and no longer hide matching direct/group delivery/read/played
facts. Generic participant routing and original chat/device/mapping guards remain
separate. Full special grammar and self-status handling remain unfinished; no
live request or account-state migration.
[Native class evidence, runtime and checks](receipt-user-recipient.md).

2026-10-11 — **Confirmed static / documentation correction:** incoming status-17
self-read submission and common state pools are now separated by exact branch
targets. Failed A09 eligibility reaches common state processing; qualifying
messages reach the separate grouped submission/local status-16 path. Rechecking
the contact policy corrected prior prose: an absent contact is allowed, while
the premium-message query applies to a present smb: contact. Original structural
bytes and complete REA smali agree. Runtime self-read handling remains unported;
no account or network change.
[Worker paths, corrected policy and reproduction](self-read-worker.md).

2026-10-11 — **Confirmed static / implemented analysis tooling:** exact DEX
virtual-override lookup now follows superclass ancestry and encoded virtual
method declarations. The pinned APK contains four A0l()Z declarations across
101,453 physical class definitions: base true, three subtype overrides false.
Shared smali parsing now refuses malformed declarations/body extents instead of
silently omitting them. Complete self-read state/submission effects, native
message-class mapping and effective ordinary-chat preference checks remain open.
No runtime/account/network change.
[Source, override census and update workflow](self-read-eligibility.md).

2026-10-11 — **Implemented/offline:** receipt fact routing now distinguishes
nullable optional addresses from raw attribute presence. Namespace-invalid
participant/recipient fields follow the native null path instead of hiding
matching direct/group observations. Recognized qualifiers and original chat,
message-ID, mapping and addressed-device guards remain effective. Raw capture,
read policy, keys, requests and waits stay intact. Full special JID subtype
rules/self-status handling and fresh live projection remain unported/unverified.
No network request.
[Native accessors, runtime and tests](receipt-optional-addresses.md).

2026-10-11 — **Implemented/offline:** structurally valid companion attestation
metadata now skips primary CPB policy, matching the native early Bjo return.
Known same-key refresh and signed ADV changed-key/returning-device application
retain their exact original-state, prekey, pin, membership and atomic guards.
Inspection preserves failed/null attestation verification; preview remains
read-only and companion admission there remains unported. Native C0F state,
non-E2EE routing and fresh live application remain unfinished/unverified.
No network request.
[Source, runtime guards and verification](companion-attestation-admission.md).

2026-10-11 — **Implemented/offline:** receipt ACKs now share strict receipt
kind and network namespace checks. A present non-text type no longer defaults to
delivery; unknown namespaces cannot become ACK destinations or optional fields.
Bare lid/broadcast aggregate targets skip before time parsing. Unsupported
receipts remain captured and reception continues. Keys, reservations, IDs, read
policy and waits stay intact; specialized JID grammar and fresh live acceptance
remain unported/unverified. No network request.
[Source, runtime behavior and regressions](receipt-ack-namespace.md).

2026-10-10 — **Implemented/offline:** matching queued read receipts now share
native extra-ID lists. Per-message selectors and original saved wire stay intact;
all batch members reserve atomically before writing, settle on the matching root
ACK and never replay unknown writes. Group member devices, destinations and
played/retry receipts remain separate. Public queue counts still count IDs.
Fresh live batch ACK/UI read indicators and full native privacy/read-self policy
remain unverified/unported. No network request.
911 workspace tests, strict Clippy/Rustdoc and CLI checks passed. New group read
attributes also follow the native order; historical wire remains unchanged.
[Native shape, recovery and API scope](read-receipt-batching.md).

2026-10-10 — **Implemented/offline:** primary attestation no longer causes a
blanket identity-refresh refusal. Verified same-key hybrid/non-E2EE metadata
permits guarded session refresh without an override child; structurally valid
changed-key metadata follows the APK E2EE fallback even when verification fails.
Classic signed-prekey, original-state, policy and own-identity guards still apply;
invalid same-key, ambiguous/missing byte fields, PQ and companion metadata defer.
Inspection preserves failed verification and preview remains read-only. Native
attestation-state storage and non-E2EE routing remain unfinished. No live request.
908 workspace tests, strict Clippy/Rustdoc and CLI checks passed.
[Native branches, runtime scope and tests](primary-attestation-admission.md).

2026-10-10 — **Implemented/offline tooling:** whole-APK SHA-256 file indexing
and fast cached diffs now cover every non-directory ZIP entry. The committed
2.26.37.71 baseline lists 14,967 files without retaining file bodies; new releases
can report added/removed/changed paths and hashes without the old APK. Existing
reviewed topic/remapping tools then locate methods and affected Rust files.
[File-diff workflow and validation](apk-file-diff.md).

Actual debug-build measurement: 8.107389 seconds to index the 146,363,876-byte
APK once, 0.211419 seconds to compare cached indexes. Whole APK hash matched the
known source; committed/generated baseline comparison is unchanged. 905 workspace
all-feature tests, strict Clippy, warnings-denied Rustdoc, default CLI/API-symbol
and CLI-reference checks passed. Malformed/duplicate/corrupt inputs refuse.
No account or network operation occurred. Cross-release changes remain synthetic
validation; byte differences do not prove method semantics or compatibility.

2026-10-10 — **Implemented/offline:** direct/group participants-key multi-target
receipts now produce atomic device facts through the existing immutable target
guards. Inner key replaces envelope ID; root status applies to every target.
Missing optional JIDs skip; absent time uses one confirmed/local-clock default,
and present malformed time refuses the batch. Historical inferred stanza times
are excluded from that fallback. Native NTP preference remains unported.
[Multi-target receipt evidence and runtime](offline-aggregated-receipts.md#key-based-multi-target-receipts--2026-10-10).

Validation: 902 workspace/all-feature tests, strict Clippy, warnings-denied
Rustdoc, default CLI/API-symbol tests, CLI-reference check and experimental
CLI build passed. Encrypted-store regressions cover inner key/root status,
clock/Unicode time, malformed rollback, frozen group targets and duplicate-free
reopening. No new live operation; fresh projection and full native privacy/
user-row admission remain unverified. The default build retains the existing
unused clock::observe_login warning. This supersedes historical key-form gaps below.

2026-10-10 — **Implemented/offline:** identity-auth metadata now follows the
APK's NONE/LOG_ONLY admission behavior. Structurally valid selected-user metadata
no longer blocks otherwise admitted classic identity refreshes. Missing/non-byte
signature or absent certs defers application and preview as malformed. Prekey
signature, policy, notification guards, own-account and companion ADV checks
remain required. Certificate/signature trust remains explicitly unported.
Exact enum, verifier, chain/result and public root evidence resolve the earlier
blanket refusal; this follow-up supersedes the historical presence gates below.
[Identity-auth admission and static evidence](identity-auth.md).

Validation: 899 workspace/all-feature tests (758 library), strict all-target
Clippy, warnings-denied Rustdoc, default CLI/API-symbol tests, CLI-reference
check and experimental CLI build passed. Encrypted-store regression tests cover
primary changes, same-key server override, signed companion change, malformed
fields, preserved refusal guards and once-only recovery. No fresh live metadata
application was checked; full certificate/backend verification remains unported.
The default build retains the existing unused clock::observe_login warning.

2026-10-10 — **Implemented/read-only:** identity-refresh inspection now reports
selected identity_auth version, certificate count and byte-body presence with
verification explicitly unported. Classic signed-prekey validity is separate;
metadata never authorizes adoption. Encrypted-store tests cover signed BMP-int
parsing/defaults, malformed/empty bodies, ignored extensions, redaction and
unchanged tables/journal across reopening. Complete REA verifier/primitive
Evidence and exact candidate/parser/field-reference dumps are retained. Native
X.509 roots/chain policy, runtime property 20107, verifier effects and actual
identity-auth signature validation remain unfinished. No new live request.
[Identity-auth evidence and inspection](identity-auth.md).

2026-10-10 — **Implemented/offline:** ordinary receipt list selection now matches
Cmk.A03: only a sole list child expands message IDs. A list beside business
metadata, another list or unrelated children selects only root ID. Previously
Rust could add receipt facts to an extra intent the native branch did not select.
The shared direct/group helper is corrected; encrypted-store tests prove the
failure before the fix and duplicate-free reopening afterward.
[Parser selection and evidence](direct-receipts.md#single-list-branch-correction--2026-10-10).

A single bounded 20-second receive window on the preceding aggregate commit
19ca798 authenticated and stopped cleanly, with no queued sends/actions/receipts,
no automatic reconnect or read marking. Registration summary, generation,
journal head and action/receipt summaries stayed unchanged. Nine checkpoints
were retained. Fresh aggregate/list projection remains unverified.
[Redacted live report](offline-aggregated-receipts-live.json).

# Evidence ledger

2026-10-10 — **Implemented/offline:** direct/group offline device aggregates
now use inner message_id and independent entry actor/status/required time.
The shared handler applies the entire batch atomically through existing original
target guards; malformed input or late saved-map failure leaves no partial facts,
row times or journal events. The unported key-based aggregate stays raw instead
of being misattributed to root ID. REA/JADX now reads the pinned APK, and complete
Evidence plus exact DDF methods are saved. Native primary cleanup and complete
receipt/status parity remain unfinished; fresh live aggregates are unverified.
[Evidence, runtime scope and reproduction](offline-aggregated-receipts.md).


2026-10-10 — **Implemented/offline:** direct participant-qualified delivery,
read and played receipts now retain the actual recipient device when the
saved chat and immutable original send target both match. Authenticated PN/LID
mapping can join namespaces at the same device ID; current membership cannot
fill historical target gaps. Previously this whole direct route was discarded.
Independent facts and server times survive restart without duplicate events;
send progress, attempts, uncertainty and legacy lab summaries are preserved.
No fresh live participant-qualified receipt has validated this extension.
[APK branch, guards and inspection](direct-receipts.md#direct-participant-device-receipts--2026-10-10).

Validation: 888 workspace/all-feature tests passed (747 library, 4 codec,
9 crash, 54 CLI, 1 API-symbol and 73 tooling tests). The new encrypted-store
fixture covers seven authority/target cases, all three fact kinds, supplied
timestamps, legacy isolation, immutable send state and duplicate suppression
across reopening. Strict all-target/all-feature Clippy, warnings-denied
Rustdoc, default CLI tests and CLI-reference regeneration passed. No live
request was made for this chunk. Repository hygiene passed 2,494 local
links and 128 checksum-verified APK bundles totaling 27,307,426 bytes.


2026-10-10 — **Confirmed static:** all seven physical message A0l writers
are classified, including wrapper marking, conditional media copy, cache
replacement and synthetic newsletter substitutions. Three media A0O producers
show asymmetric round-trip behavior: message-to-media copies false, reverse
conversion never clears an already marked message. The later manager filter
is not a universal revoke or timestamp gate. Full lifecycle, user-row admission
and primary identity cleanup remain unfinished; no runtime/account change.
[Exact branches, scans and source](receipt-message-exclusion.md).

Validation: exact regeneration verified 14 topics / 19 files / 509,571 bytes;
cached self-diff found 14 unchanged topics and four metadata artifacts.
CLI-reference regeneration, API/library-symbol validation, formatting,
whitespace and hygiene passed (2,491 local links; 128 bundles / 27,307,426
bytes). No new workspace runtime test or live account operation is claimed;
the prior functional chunk's 887-test result remains separate.


2026-10-09 — **Implemented/offline:** newly recorded direct/group delivery,
read and played facts preserve valid supplied root t as signed/wrapping
milliseconds, separate from observed_at. Missing/invalid receipt time remains
unknown; historical and duplicate facts are unchanged. Shared parsing preserves
the existing earliest-positive device row behavior, and no user-row authority
or identity cleanup is inferred. Fresh live projection remains unverified.
[Evidence and inspection](receipt-fact-time.md).

Live receive validation: one 20-second receive-only window authenticated
and stopped cleanly (33.67 seconds total). Actions 21, receipt rows 9 and
eligible outbox 0 remained unchanged; generation/registration/journal-head
summaries stayed equal. Only unsupported controls appeared; no qualifying
receipt projection or decrypted message was observed. Automatic reads,
outbox sends and reconnects were disabled. Summary/count equality is not
all-record byte equality. [Redacted result](receipt-fact-time-live.json).

Validation: 887 workspace/all-feature tests passed (746 library, 4 codec,
9 crash, 54 CLI, 1 API-symbol and 73 tooling tests). Strict all-target/all-feature
Clippy, warnings-denied Rustdoc, default CLI tests, CLI-reference regeneration,
formatting and whitespace checks passed. Hygiene checked 2,479 local links and
127 APK bundles / 26,797,855 bytes. Existing APK evidence was reused; no new
export or cross-release parity is claimed.


2026-10-09 — **Confirmed static:** returning-companion addition's remote
false-Boolean coordinator path bypasses removal/local-message/own-hosted work.
DNd discriminator 19 clears one user-device-map cache entry; Rust reads its
saved device list directly. Exact CBu scans identify one concrete discriminator
callback: removal work in branch 0, activity UI scheduling in branch 1.
Registration/lifetime, outer callback ordering and complete UI runnable effects
remain unknown. No new Rust/runtime/account operation is claimed.
[Evidence and exact scan boundaries](companion-addition-observers.md).

Validation: exact regeneration verified 16 topics / 21 files / 270,236 bytes;
cached self-diff found 16 unchanged topics and four metadata artifacts.
CLI-reference regeneration, OpenAPI/library-symbol checks, formatting,
whitespace and repository hygiene passed (2,473 local links, 127 bundles /
26,797,855 bytes). No new workspace runtime test or live account operation
is claimed for this static-only chunk.


2026-10-09 — **Confirmed static:** remote companion addition updates existing
participant device maps, recomputes group hashes and replaces exact group/user
device rows. New rows initialize sender-key flags to 0/0 and bucket to NULL;
existing map entries retain their values. False map values/null removal bypass
rotation/removal paths traced here. Rust plans expand current device lists and
distribute to every frozen target; no new runtime behavior or native omission
cache parity is claimed. Hosted/counterpart gates and concrete event consumers
remain unknown. [Evidence and reproducible recipe](companion-addition-group-state.md).

Validation: exact APK export check verified 13 topics / 18 files / 103,661
bytes; cached self-diff reports unchanged retained topics and metadata.
CLI-reference regeneration, OpenAPI/library-symbol validation, formatting and
whitespace checks passed. Repository hygiene checked 2,463 local links and
126 bundles / 26,527,619 bytes. No Rust runtime changed; no fresh workspace
runtime test or live account operation is claimed.


2026-10-09 — **confirmed static / implemented offline**: APK 37.71
11T.A0D skips equal-key shortcut for absent membership, verifies full ADV and
requires newer signed time before add-only remote allocation. 1IW/AzR/B7j
membership/canonical/observer boundaries retained separately. Rust restores
only a guarded previously pinned exact-namespace companion, with both signatures,
current primary/raw-ID/E2EE baseline, atomic membership/provenance/pending sync
and appropriate session/history. No primary floor or existing index replacement,
first-seen pin or canonical namespace inference. Inspection adds the fixed
ready_addition_pending status; live qualifying restoration unknown.
[Evidence and remaining boundaries](companion-directory-addition.md).


2026-10-09 — **confirmed static / implemented offline**: APK 37.71
14J.A04 companion directory candidate calls 11T.A0D discriminator 4 / false.
0jV.A0a/A08 and 08i/08a own predicates close remote login/own branch inputs.
Both ADV signatures bind candidate and independent primary; matched E2EE/raw-ID
baseline then exact device-index/signed-long timestamp admission. Newer ADV
requests sync but existing membership indices are add-only. Rust guarded
changed companion reply application now includes this admission and atomic
history/fresh session/pending sync; shared receive/expected comparison corrected
from unsigned to APK signed long. No first-seen or primary/raw-ID replacement
inferred, live qualifying application unknown.
[Evidence, negatives and recipe](companion-changed-key-refresh.md).


2026-10-09 — **confirmed static / implemented offline**: APK 2.26.37.71
CPB.A00 nonprimary Bjo; 14J.A04 routes to 11T.A0D; nonnull ADV + exact current
membership + equal held key fast-return precedes ADV parsing. B7D/B7E equality
and 0bN.A0B map membership retained. Rust applies only equal-key guarded classic
companion refresh, with stronger local current-primary/list/own/teardown and
atomic authority protections. No ADV verification or changed-key admission is
inferred from fast return. Fresh live application unknown.
[Trace, negative results and recipe](companion-same-key-refresh.md).


Identity notification key reads (2026-10-09, **confirmed static / implemented offline**):
the normal callback target is root from, with device preserved and participant
separate. Authenticated receive now queues reason=identity reads for supported
existing remote PN/LID pins, with durable event/device guards and no replay of
uncertain/refused/invalid reads. Four tests cover routes, guards, signed prior
results and an actor write cut/reopen. Response adoption, full native admission
and identity ACK lifecycle remain unported. [Evidence and limits](identity-notification-key-read.md).


Validation: 852 workspace/all-feature tests passed. Strict Clippy, warnings-
denied Rustdoc, default CLI tests (25 plus API-symbol), CLI-reference regeneration,
formatting and whitespace passed. Exact export check verified 11 topics / 16 files /
102,122 bytes; checksum-verified cached self-diff found 11 unchanged topics and
four metadata artifacts. Hygiene passed (2,294 links / 114 bundles).
One 45-second authenticated receive window observed a
primary-LID read with verified signature and matching pin; no identity transition,
chat send or adoption was validated. [Redacted live evidence](identity-notification-key-read-live.json).


Supplied bundle authentication (2026-10-09, **confirmed static / implemented offline**):
common direct/pairwise encryption verifies supplied signed device keys even with
an existing session. A failing-before generated-key regression proves forged
same-identity refusal; valid/no-bundle session reuse and identity policy remain.
Notification callback construction is retained through CSu.A02; its upstream
routing and native automatic refresh remain unknown/unported. No account or
fresh live operation. [Validation and limits](peer-identity.md#supplied-bundle-validation-with-a-held-session-2026-10-09);
[address evidence](identity-notifications.md#callback-address-construction-continuation-2026-10-09).


Validation: 848 workspace/all-feature tests passed. Strict Clippy, warnings-
denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated CLI reference,
formatting and whitespace checks passed. The four-topic export reproduces exactly
(9 files / 54,997 bytes); checksum-verified cached self-diff shows four unchanged
topics and four metadata artifacts. Hygiene passed (2,264 links / 113 bundles).
No fresh live variant or message send was performed.


Bulk prekey processing (2026-10-09, **confirmed static / implemented offline**):
callback messages separate parsed candidates, no bundle and per-device errors.
Conditional owner sync, admission/ADV gates and session construction are retained.
A failing-before regression fixes missing type: absence defaults to 5; present
malformed/duplicate/unsupported types and forged signatures still refuse. Native
hybrid admission, mapping/primitive authority and automatic refresh remain
unported. No real account or live operation. [Evidence and limits](bulk-prekey-processing.md).

Validation: 847 workspace/all-feature tests passed; strict Clippy, warnings-
denied Rustdoc, default CLI tests (25 plus API-symbol), CLI-reference regeneration,
formatting and whitespace passed. Exact export check: 16 topics / 21 files /
137,128 bytes; cached self-diff: 16 unchanged topics / four metadata artifacts.
Hygiene passed (2,253 local links / 112 APK bundles). No fresh live variant.


Bulk prekey request (2026-10-09, **confirmed static / implemented offline**):
identityChangedJids selects user/@reason=identity in the normal encrypt/key IQ.
The reusable Rust builder and peer-keys --preview-identity-change expose that
request shape without account access or queueing. Device/namespace mismatches
refuse with fixed diagnostics. Job/callback sources are retained; Cio preparation,
response adoption and automatic notification refresh remain unported. No live
operation. [Evidence and limits](bulk-prekey-refresh.md).

Validation: 846 workspace/all-feature tests passed; strict Clippy, warnings-
denied Rustdoc, default CLI tests (25 plus API-symbol), CLI reference, formatting
and whitespace passed. Exact export check: six topics / 11 files / 70,703 bytes;
cached self-diff: six unchanged topics / four metadata artifacts. Hygiene passed
(2,229 local links / 111 APK bundles). Actual CLI preview succeeded without a
store or key environment and created no files. No fresh live variant was sent.


Identity notification source (2026-10-09, **confirmed static**): metadata peer
and callback device follow separate paths; null metadata rejoins device work.
The native worker gates reason-8 prekey refresh on an existing identity, then
uses filtered/in-flight-aware bulk job scheduling. No replacement key is read
from the selected identity child. Ten retained topics close the scheduler path;
wire response/adoption policy and deferred ACK state remain unported. No
runtime, account or live change. [Evidence and limits](identity-notifications.md).

Source validation: exact regeneration verified 15 files / 141,669 bytes;
checksum-verified cached self-diff found ten unchanged topics and four metadata
artifacts. CLI reference, API-symbol, formatting, whitespace and repository
hygiene passed (2,214 local links / 110 APK bundles). No fresh runtime suite
or live validation is claimed for this source/docs-only chunk.


Prekey count dispatch (2026-10-09, **confirmed static / implemented offline**):
first-child selection, signed optional legacy parsing and strict/defaulted PQ
parsing now follow APK 37.71. Legacy dispatch remains true even for an invalid
optional value. Identity-first children cannot trigger count replenishment.
Five retained source topics and 487 executed Java cases support the fix; a
fake-link test preserves malformed capture and state across reopen. Native PQ
strategy, acknowledgement ordering and identity-first handling remain unported.
No real account or fresh live operation. [Evidence and limits](prekey-count-notifications.md).

Validation: 843 workspace/all-feature tests passed, plus default CLI tests
(25 and API-symbol). Strict Clippy, warnings-denied Rustdoc, CLI-reference
regeneration, formatting and whitespace checks passed. Exact bundle regeneration verified ten files / 73,903 bytes; cached self-diff
verified five unchanged topics and four metadata artifacts. Repository hygiene
passed (2,194 local links, 109 APK bundles).


Bounded receive validation (2026-10-09, **observed live**): current experimental
CLI authenticated the saved account, completed 45 receive seconds and stopped
cleanly, with chat sends, automatic reconnects and automatic reads disabled.
Outbox/action/receipt inspections and event head stayed equal; nine inbound
checkpoints and one confirmed login-clock record were added. No supported reply
or peer transition observed. [Redacted evidence and limits](login-identity-updates.md#bounded-live-regression-2026-10-09).

Live-check validation: the experimental CLI build, CLI-reference regeneration,
API/library-symbol checks, formatting, whitespace and repository hygiene passed
(2,186 links/108 APK bundles). One bounded live receive run completed; no fresh
workspace runtime suite was rerun for this evidence-only change. The preceding
840-test implementation validation remains separate.


Partial login identity persistence (2026-10-09, **implemented/offline tested**):
PN-only/LID-only authenticated success updates preserve the other saved namespace.
Malformed replacements/history refuse atomically without changing account state,
clock or uncertain login reservation. A failing-before regression covers the
lost own LID; no real account or live operation. [Evidence and limits](login-identity-updates.md).

Validation: 840 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, formatting and whitespace checks passed. Repository hygiene
checked 2,182 local links and 108 existing APK bundles. No fresh live test.


Incoming recipient-count source (2026-10-09, **confirmed static**): root
`count` defaults only on absence; malformed present input throws from the
signed-int helper. Own group/status-envelope transfer reaches native message.A06
through Cih/CaF/Bp2, with distinct branch guards. This closes the count producer,
not receipt_user membership or primary cleanup. No runtime/live change.
[Evidence and limits](message-recipient-count.md).

Count-source validation: three topics / eight files / 34,489 bytes reproduce
exactly from APK 37.71. Cached self-diff verifies three unchanged topics and
four metadata artifacts; raw count-copy/false-rejoin branch operands and the
field-write opcode agree. CLI reference, API-symbol, formatting, whitespace and
hygiene passed (2,176 links/108 APK bundles). Source/docs only; no fresh Rust
runtime suite, account operation or live test is claimed.


Peer-address guard coverage (2026-10-09, **implemented/offline tested**):
peer-key queue/inspection and primary pin inspection now reject empty users,
extending the existing identity-only prerequisite. A failing-before regression
preserves pins, sessions, events and the valid query's uncertain reservation
across reopen. No live operation. [Evidence and limits](peer-identity.md#empty-peer-guard-on-key-queries-and-pin-inspection-2026-10-09).

Validation: 837 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, formatting and whitespace checks passed. Repository hygiene
checked 2,162 local links and 107 existing APK bundles. No fresh live test.


Attachment expiry continuation (2026-10-09, **implemented/offline tested**):
ordinary media retrieval shares the explicit incoming Java timestamp parser,
fixing event/retrieval disagreement for Unicode-decimal time. Missing-time and
missing-route refusals remain; no live download or account operation.
[Evidence and tests](message-timestamps.md#attachment-expiry-continuation).

Validation: 836 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, formatting and whitespace checks passed. Repository hygiene
checked 2,159 local links and 107 APK bundles. No fresh live test.


Incoming timestamp parsing (2026-10-09, **implemented/offline tested**):
explicit valid root `t` uses APK-derived Java signed-decimal parsing and wrapping
millisecond conversion across incoming event projections. Missing/malformed
historical timestamps remain unknown; expiration and once-only journal guards
remain intact. No account or live operation. [Evidence and limits](message-timestamps.md).

Validation: 835 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, executed Java oracle, formatting and whitespace checks passed.
Repository hygiene checked 2,156 links and 107 existing APK bundles. No live test.


Deferred status observers (2026-10-09, **confirmed static**): corrected the
historical prose selector 5 to 16; raw APK bytes and constructor/switch operands
agree with the already retained source. The immediate hook dispatches message
BuN and chat BfD callbacks, not a direct receipt-user write. An 80-declaration
inventory records 43 return-only, one no-bytecode and 36 body declarations;
registration and transitive effects remain unknown. No runtime/account/live
change. [Evidence and limits](status-observers.md).

Observer evidence validation: exact 14-topic export (15 methods, 19 files,
216,841 bytes), cached self-diff, all 80 declaration/hash records, message/chat
callback prototypes and independent raw selector/payload checks passed. CLI
reference, API-symbol, formatting, whitespace and hygiene checks passed
(2,149 links/107 APK checksum bundles). Research/docs only; no fresh Rust
runtime suite, account operation or live invocation is claimed.

Send-job state path (2026-10-09, **confirmed static**): post-insert DNs
selector 49 falls through to return; independent raw switch bytes confirm it.
The actual send job queues native states 20/21 through 15A.A05/DM8 selector 6,
which looks up existing messages before A07. Conditional ACK/aborted/retry-count
paths are retained without inventing runtime retry or receipt-row authority.
No Rust, saved-account or live behavior change. [Evidence and limits](send-job-state.md).

Send-job evidence validation: exact 14-topic export (19 files/268,818 bytes),
cached self-diff, raw listener switch/branch and state-handoff assertions passed.
CLI reference, API-symbol, formatting, whitespace and hygiene checks passed
(2,138 links/106 APK bundles). Research/docs only; no fresh runtime or live test.

ACK timing (2026-10-09, **implemented/offline tested**): matching success
ACK facts retain parsed server time separately from local observation time,
using the APK's Java default-zero and wrapping semantics. Historical rows remain
unknown; duplicate terminal ACKs cannot rewrite the timestamp. The status API
schemas now describe actual ticket/progress/facts JSON. No new live send or
native receipt admission. [Evidence and limits](ack-timestamps.md).

Validation: 832 workspace/all-feature tests passed, including 480 timestamp
store/reopen cases within one regression. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated CLI
reference, formatting and whitespace checks passed. Repository hygiene checked
2,128 links and 105 existing APK bundles. No qualifying live ACK or account
migration was performed.

ACK history correlation (2026-10-09, **implemented/offline tested**): removed
our 10,000-row refusal using the existing complete paginated wire-ID lookup
inside the ACK transaction. Ambiguous/corrupt history refuses atomically;
recipient/participant guards, terminal facts and replay reservations remain intact.
The native dispatcher prose is corrected to arg1=19, not what=19. No live send,
account migration or native user-row implementation.
[Evidence and limits](ack-correlation-pages.md).

ACK pagination validation: 829 workspace/all-feature tests, strict all-target
Clippy, warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), CLI
reference, API-symbol, formatting and whitespace checks passed. The regression
fails on the old implementation with pagination required. Exact two-topic source
export/self-diff and hygiene (2,119 links/105 APK bundles) passed. No live ACK,
send, account migration or replay ran.

Before-insertion registry (2026-10-09, **confirmed static**): all 13 registered
before entries now resolve to factories and executable CHJ bodies. Together with
the after index this maps 13+54 immediate hooks. Selected delegates distinguish
lookup/origination masks and metadata from native status; no direct receipt_user
writer or native status assignment was found in the retained hook bodies.
Transitive admission, normal-send progression and primary cleanup remain
unknown/unported. No runtime/account/live change.
[Evidence and limits](text-before-hooks.md).

Before-hook validation: exact 21-topic export (26 files/83,350 bytes), cached
self-diff, 13 ordered dispatch/factory/body assertions, CLI reference,
API/library-symbol checks, formatting and whitespace passed. No fresh runtime
suite or live account test is claimed.

Scheduled arithmetic follow-up (2026-10-09, **confirmed static**): DEX static
wide loads/stores now have mnemonics and an APK-byte regression. D7g passes
scheduled_time plus a 30-day millisecond constant to DGf.A03. Complete expiry
semantics and receipt admission remain unknown/unported. No account/live change.
[Evidence and limits](scheduled-deadline.md).

Scheduled-decoder validation: 827 workspace/all-feature tests, strict Clippy,
strict rustdoc, formatting, CLI reference and API/library-symbol checks passed.
Exact three-topic export (eight files/36,973 bytes), cached self-diff and the
single-mnemonic comparison passed. Hygiene checked 2,097 local links and 103 APK
bundles. No live account test ran.

Final after-hook registry (2026-10-09, **confirmed static**): all 54 registered
after entries now resolve to factories and executable CHJ bodies, with an ordered
source index. The final 13 include add-on status setters, scheduled/appointment,
AI/translation, comment, experience and payment paths. Direct exclusions do not
establish transitive receipt preservation. Before hooks, native admission and
primary cleanup remain unfinished. No runtime/account/live change.
[Evidence and limits](text-final-after-hooks.md).

Final-hook validation: exact 25-topic export (30 files/1,106,965 bytes), cached
self-diff (25 topics/four metadata artifacts unchanged), complete 54-entry source
index assertions, CLI reference, API/library-symbol, formatting and whitespace
checks passed. Hygiene checked 2,090 local links and 102 APK bundles. No fresh
runtime suite or live account test is claimed.

Association/sharing after-hooks (2026-10-09, **confirmed static**): five
more registered entries map to parent/child, UI and poll-history storage, incoming
limit-sharing system-message creation and embedded child insertion. AVI sets
status 6 on a new Bmy, not the input; annotation children OR A0A then reach the
insertion wrapper. Current mapping is 41/54 with 13 remaining. Transitive receipt
admission and primary cleanup remain unknown/unported. No runtime/account/live
change. [Evidence and limits](text-association-after-hooks.md).

Association-hook validation: exact 22-topic export and cached self-diff, scoped
source checks, CLI reference, API/library-symbol, formatting, whitespace and
hygiene passed. No fresh runtime suite or live account test is claimed.

Storage/poll/indexing after-hooks (2026-10-09, **confirmed static**): seven
more registered entries and nine selected delegates are attributed. Writes target
ephemeral, media/thumbnail or search tables; thumbnail post-processing clears a
marker, not stored bytes/view-once content. Current mapping is 36/54 with 18
remaining after entries. Transitive receipt admission and primary cleanup remain
unknown/unported. No runtime/account/live change. [Evidence and limits](text-storage-after-hooks.md).

Storage-hook validation: exact 27-topic export and cached self-diff, direct
source checks, CLI reference, API/library-symbol, formatting, whitespace and
hygiene passed. No fresh runtime suite or live account test is claimed.

Additional metadata hooks (2026-10-09, **confirmed static**): eight bucket-one
entries now have registered factory/CHJ boundaries, with inline-video, mention
and question helpers traced. Question SQL failure throws for -1; selected other
negative writes only log or discard IDs. Immediate after coverage is 29/54,
with 25 entries and transitive receipt admission still unknown/unported. Primary
cleanup remains unfinished. No runtime/account/live change.
[Evidence and limits](text-metadata-after-hooks.md).

Metadata-hook validation: exact twelve-topic export and cached self-diff, CLI
reference, API/library-symbol checks, formatting, whitespace and hygiene passed.
No fresh runtime suite or live account test is claimed; see linked evidence.

Bot metadata after-hooks (2026-10-09, **confirmed static**): providers
0xf4b/0xf53 resolve to D7a/D7j. Selected delegated writers target bot_message_info,
bot_plugin_metadata and bot_message_sharing_info, discard SQL return IDs and do
not directly write receipt_user or call the native status setter. This brings
immediate after-hook mapping to 21/54; 33 entries and transitive admission effects
remain unknown/unported. Primary cleanup remains unfinished. No runtime/account/
live change. [Evidence and limits](text-bot-after-hooks.md).

Bot-hook validation: exact nine-topic export and cached self-diff, CLI reference,
API/library-symbol, formatting, whitespace and hygiene passed. No fresh runtime
suite or live account test is claimed; see the linked evidence for counts.

Main-row update (2026-10-09, **confirmed static**): selector 6 reaches
the existing-row message writer; status reads B3b, affected-row count is ignored
and a missing lookup has no inserted-row fallback. Link flag A0n feeds the
computed lookup mask; origination_flags instead reads A0A, with a conditional
bit clear. Hook effects, native user-row admission and primary cleanup remain
unknown/unported. No runtime/account/live change. [Evidence and limits](text-main-update.md).

Common link/span after-hook (2026-10-09, **confirmed static**): Ce8 writes
span indices and ORs A0n flag 0x10000, distinct from A08 status, then queues
reason -1 / Ahi selector 5 / scheduling 22. The update wrapper separates
before/callback/CgJ/after hooks and unsupported-type false returns. Transitive
update effects, user-row admission and primary cleanup remain unknown/unported.
No runtime/account/live change. [Evidence and limits](text-link-after-hook.md).

Selected after-insert hooks (2026-10-09, **confirmed static**): 19 of 54
registered entries now have concrete provider/CHJ boundaries. Selected bodies
write metadata or delegate to helpers; no direct native status setter or
receipt_user SQL is present. Helper effects, 35 remaining entries, user-row
admission and primary cleanup remain unknown/unported. No runtime/account/live
change. [Evidence and limits](text-after-hooks.md).

Text type inserter (2026-10-09, **confirmed static**): type zero resolves
through provider 0x14c1 to 1hl.BJK and optional message_text persistence.
D7J recipient-count preprocessing is now confirmed in the 13-entry before set
at index 8 / provider 0x3c5; 54 after entries are retained. Other hook effects,
full send progression, user-row admission and primary cleanup remain unported.
No runtime/account/live change. [Evidence and limits](text-type-inserter.md).

Main message row (2026-10-09, **confirmed static**): the insertion callback
writes table message before the type-specific inserter. A0h(1) sets storage
type A0f; SQL status reads B3b instead. Main-row ID failures and lazy registry
resolution are retained; concrete type-zero mapping, native user-row admission
and primary cleanup remain unknown/unported. No runtime/account/live change.
[Evidence and limits](text-main-row.md).

Text store insertion (2026-10-09, **confirmed static**): four topics connect
ordinary authored-message insertion to its type registry, transaction wrapper
and before/lambda/inserter/after hook order. The insertion argument 1 is not a
status assignment; void return is not durable-row proof. Concrete type-zero
inserter/provider effects and primary cleanup remain unknown/unported.
No runtime/account/live change. [Evidence and limits](text-store-insertion.md).

Text store submission (2026-10-09, **confirmed static**): nine topics
connect B1l to the deferred 3Pn selector-4 store worker. Reason -1/1/22 and
scheduler 26 are not status assignments. Three setter candidates are separately
attributed to payment, ACK and unsent recovery. Full normal send progression
and primary cleanup remain unported; no runtime/account/live change.
[Evidence and limits](text-store-submission.md).

Receipt count preprocessing (2026-10-09, **confirmed static**): four topics
close participant-map counts and mode-dependent local-user membership; reused
sources attribute copy/builder/ACK writes. The shared group write site is not
always literal two. All ten physical sites have immediate value attribution,
but normal-send and transitive producer authority remain incomplete/unported.
No runtime/account/live change. [Evidence and limits](receipt-count-preprocessing.md).

Recipient-count producers (2026-10-09, **confirmed static**): five topics
separate status-audience size excluding the local user from conditionally written
broadcast clone-entry counts; cursor restoration loads recipient_count directly.
Ordinary send/copy writers and full native admission remain untraced/unported.
No runtime, account or live change.
[Evidence and limits](receipt-recipient-count.md).

Receipt message projection (2026-10-09, **confirmed static**): four topics
close status-only existing-row updates and the conditional view-once state hook.
Multipart status counts each user's highest state against native A06; computed
downgrades log and still write. Threshold producers and complete user admission
remain unknown/unported; no runtime/account/live change.
[Evidence and limits](receipt-projection.md).

Status-array renderer correction (2026-10-09, **confirmed local/static**):
the APK-derived regression exposes the old 64-byte truncation; the bounded
256-byte preview now preserves all 22 receipt-status ranks and reports omitted
bytes on larger arrays. A new pinned snapshot preserves historical exports.
No messaging/account/live change; primary cleanup remains incomplete.
[Evidence and limits](receipt-admission.md#complete-status-array-rendering-2026-10-09).

Blank-receipt failure continuation (2026-10-09, **confirmed static**):
four source topics identify Cnn/0zO as message-send event logging and separate
builder result 3 from native message status 20. No direct status setter or
receipt_user access is established; helper effects and primary cleanup remain
unknown/unported. No runtime, account or live change.
[Evidence and limits](message-status.md#blank-receipt-failure-logging-continuation-2026-10-09).

Offline account diagnostics (2026-10-09, **observed/local**): the original
private key enabled saved-profile receipt inspection: nine ACKed reservations
(eight read, one retry), identical logical results across reopen; no live request.
Account pending-work counts now exclude retained cancelled/failed/in-flight
history and share the pending-list progress predicate. No resend/state reset.
[Count evidence](account-status-counts.md). [Receipt check](receipt-ack-correlation.md#original-saved-account-inspection-restored-2026-10-09).

Key-based receipt lookup (2026-10-09, **confirmed static**): seven topics
close ordinary/add-on overrides and show missing ordinary messages select the
add-on keyed receipt join. Pending membership is distinct from delivered/user
rows; primary cleanup remains unfinished. Offline saved-account receipts check
stopped before store open because its master-key environment input was absent.
No account mutation, runtime change or live request.
[Evidence and limits](receipt-key-lookup.md).

Receipt inspection correction (2026-10-09, **confirmed local**): offline
receipts inspection now diagnoses malformed saved wire/IDs with fixed redacted
errors instead of silently dropping/truncating bytes or returning an empty ID.
Reservations and outcomes remain unchanged across failure/reopen; primary
cleanup and native user-row admission remain unfinished. No live operation.
[Evidence and limits](receipt-ack-correlation.md#saved-receipt-inspection-diagnostics-2026-10-09).

Read persistence release relocation (2026-10-09, **confirmed static/local**):
a separate checked recipe locates six methods; signature plus physical caller
resolves a three-way field-writer ambiguity. Four string-free topics remain
manual and prevent automatic proposal. Cached ten-topic self-diff took
0.014920 seconds; no second release/runtime/live parity is claimed.
[Evidence and limits](read-receipt-persistence.md#release-relocation-2026-10-09).

Deferred blank receipt expansion (2026-10-09, **confirmed static**): eight
source topics close DNt selector 6 as status-target device-set subtraction
and blank-device insertion, not normal receipt_user seeding. Existing-target
property and abstract override boundaries remain unknown; primary cleanup
still requires actual user-row membership. No runtime/live change.
[Evidence and limits](blank-receipt-deferred.md).

Read-receipt persistence (2026-10-09, **confirmed static**): ten source topics
connect local status marking to existing message-row updates and separate
chat row/sort and status-table sent positions. IOException may be logged and
swallowed after the in-memory mark; no ACK wait is established. Complete
queue/store reconciliation remains unknown/unported; no runtime/live change.
[Evidence and limits](read-receipt-persistence.md).

Read-receipt submission (2026-10-09, **confirmed static**): thirteen source
topics separate grouped queueing from later per-message policy, 256-ID batches,
forced read-self and local status-16 marking. Maximum store-time is preserved;
worker/migration authority and queue atomicity remain unknown/unported.
No runtime or live change.
[Evidence and limits](read-receipt-submission.md).

Read-setting observer binding (2026-10-09, **confirmed static**): sixteen
source topics connect generated observer membership to DK9 and distinguish
category/value Bsa from empty list callback C5W. Its read-work scan uses saved
read/receipt positions before played-receipt work; selection is not transmission.
Full sender/other-observer authority remains unknown; no runtime or live change.
[Evidence and limits](read-setting-observers.md).

Read-setting pending lifecycle (2026-10-09, **confirmed static**): eight
source topics bind the shared 20-second main-Handler runnable to clearing all
pending settings and notifying observers, without a preference write or resend
in that branch. UI pending display differs from committed settings. Effective
state/observer authority remain unknown; no runtime or live change.
[Evidence and limits](read-setting-pending.md).

Read-setting fetch and observers (2026-10-09, **confirmed static**): thirteen
source topics distinguish fetch mode 23463 from set mode 23639, retain SMAX
get/callback parsing, and identify played-receipt queueing in a settings observer.
Boolean local-set refusal and local submission versus server acceptance are
explicit. Effective settings/bootstrap/observer authority remain unknown;
no runtime or live change.
[Evidence and limits](read-setting-bootstrap.md).

Read-setting wire producer (2026-10-09, **confirmed static**): a read-self
receipt ACK with a present readreceipts attribute emits arg1 event 227 and
still reaches generic ACK forwarding. Four source topics connect enabled
dispatch to the retained 14c preference consumer. Effective state and fallback
registration remain unknown; no runtime/live change.
[Evidence and limits](read-setting-sync.md#read-self-ack-wire-producer-continuation).

Read-setting synchronization (2026-10-09, **confirmed static**): ten source
topics distinguish local-off-preserving discriminator 227 from validated
category-map adoption and pending-value mismatch guards. A0Q dispatches observers,
not a direct query. Effective account setting, bootstrap and complete lifecycle
remain unknown/unported; no runtime or live change.
[Evidence and limits](read-setting-sync.md).

Device-receipt timestamp correction (2026-10-09, **confirmed local / APK-backed**):
Java BMP receipt times now reach earliest-positive mutable device rows rather
than remaining falsely pending. A failing-before test checks received-row
preservation through companion cleanup, rollback, immutable provenance and reopen;
normal-outbox receipt attribution is also exercised. Primary cleanup remains
unported; no real account/live change.
[Evidence and limits](device-receipt-rows.md#java-receipt-time-and-companion-cleanup-correction-2026-10-09).

Ordinary text status source (2026-10-09, **confirmed static / bounded inference**):
user-input text construction uses 1Ze/1he and the NTP-preferred clock; its
plain construction boundary differs from media's explicit status 1. Seventeen
source topics retain callbacks and exclude unrelated setter workflows.
Full preparation/user-row admission and identity receipt cleanup remain unported.
[Evidence and limits](message-status.md#ordinary-text-construction-and-excluded-setter-candidates-2026-10-09).

Trusted-contact timestamp parser (2026-10-09, **confirmed local / static-backed**):
parent/token times now use Java-compatible decimal-long parsing within the
existing nonnegative storage range. A failing-before regression covers BMP
digits, override/fallback, malformed-token rollback and reopening. No real
account/live change; complete native token admission remains unfinished.
[Evidence and limits](token-notification-peer.md#timestamp-parser-correction-2026-10-09).

NTP clock preference (2026-10-09, **confirmed static**): nine source topics
establish separate NTP/WhatsApp baselines, offset sign, acquisition correlation,
cache/recent-sync checks and feature-controlled rejection. NTP fetching,
effective property 2225 and upstream scheduling remain unported/unknown.
No runtime or real-account/live change.
[Evidence and limits](token-clock-producers.md#ntp-preference-and-acquisition-2026-10-09-continuation).

Confirmed clock runtime (2026-10-09, **confirmed local / APK-backed**):
authenticated login and server-ping time now save source/offset separately from
inferred stanza samples. Linux/Android use suspend-aware elapsed time during the
same process; restart uses the saved offset. Client mute resolution prefers this
confirmed source. Offline `clock-inspect` reports redacted provenance. NTP clock,
token scheduling and live parity remain unfinished; no real-account/live change.
[Evidence and limits](token-clock-producers.md#confirmed-runtime-clock-inputs-2026-10-09).

Server-ping clock input (2026-10-09, **confirmed static/local audit**):
`iq type=get xmlns=urn:xmpp:ping` supplies envelope t through signed-long/fallback
parsing to Android arg1 selector 4. Result IQs do not use this clock branch.
Six source topics close wire provenance; downstream reply writing and Rust
clock parity remain unported. No runtime or real-account/live change.
[Evidence and limits](token-clock-producers.md#server-ping-wire-producer-2026-10-09-continuation).

Token clock producers (2026-10-09, **confirmed static/local audit**):
APK `08B.A07` takes login `success@t` through `1Rd` or selector-4 Bundle time,
sets an elapsed baseline and persists client/server difference. Seven source
topics close those inputs; selector-4 wire provenance and full Rust clock parity
remain unknown/unported. Current stanza clock sampling is still an inference;
no real account or live change.
[Evidence and limits](token-clock-producers.md).

Trusted-contact peer guard (2026-10-09, **confirmed local / static-backed**):
privacy-token receive now rejects empty PN/LID users before storage or ACK;
supplied empty sender_lid keeps a fixed diagnostic. Generic server-only JIDs
previously passed our namespace-only guard. A failing-before regression checks
text/Plain/AD inputs, unchanged tokens/pins/sessions and reopening; valid token
handling remains covered. No real account/live change.
[Evidence and limits](token-notification-peer.md).

Token mapping consumers (2026-10-09, **confirmed static/local**):
selected table readers feed a separate specifier cache and native stable-ID list.
`00D.A0C` compares/logs values; its caller separately chooses native or fallback
return. Readiness and initial preference inputs are traced, effective account
settings remain unknown. Renderer now decodes APK shift opcodes b8/c5 with a
failing-before regression; seven source topics preserve the boundary.
[Evidence and limits](token-feature-inputs.md#mapping-consumers-and-shadow-comparison-2026-10-09).

Token MobileConfig mapping (2026-10-09, **confirmed static**): provider
233 resolves to `09N`; its generated array has nonzero initial specifiers for
15491 (`0x811d83000016e2`) and 14303 (`0x811af9000014f5`). Eight source topics
preserve the binding and indexed lookup. Effective native values/readiness and
array consumers remain unknown; no runtime/live change.
[Evidence and limits](token-feature-inputs.md#property-to-mobileconfig-mapping-2026-10-09).

Token Boolean provider bindings (2026-10-09, **confirmed static**):
`00D` binds `1Y4(4)` and `1Y5(0)` to its Boolean providers. Both resolve IDs
through `09M.B2y`; zero mapping returns false, nonzero calls MobileConfig
`AVn`. Six source topics preserve bindings/selectors/branches; actual mappings,
native values and effective precedence remain unknown. No runtime/live change.
[Evidence and limits](token-feature-inputs.md#boolean-provider-bindings-2026-10-09-continuation).

Token feature inputs (2026-10-09, **confirmed static/local**): APK 37.71
registers properties 15491/14303 as false Booleans; its getter can use caches,
preferences or MobileConfig, so effective values remain unknown. Offline
`abprops-query --show` now reports selected returned token values and invalid
last-value flags, without default fallback or runtime activation. Twelve source
topics and regression/reopen tests preserve the evidence. No real account or
live change. [Evidence and limits](token-feature-inputs.md).

Token canonicalization gates (2026-10-09, **confirmed static**): APK
37.71 normalizes type-zero phone users independently of property 15491, which
suppresses only counterpart lookup/cleanup. Separate property 14303 selects
pending-job namespace. The six-topic snapshot records incoming-byte versus
outbound timestamp getters and four physical counterpart-helper callers. Effective
flags, mapping authority and refresh runtime remain unknown/unported. No account
state or live request changed.
[Evidence and limits](token-canonicalization.md).

Identity-only peer prerequisite (2026-10-09, **confirmed local**): public
query/inspection now refuses an empty peer user before reading or saving action
state; shared primary-address matching also refuses server-only values. The
regression failed before the fix and covers PN/LID, Plain/AD encodings, unchanged
pins/sessions/checkpoints and reopening without replay of an uncertain query.
This is our input guard, not a WhatsApp refusal; no real account or live change.
[Evidence and limits](peer-identity.md#empty-peer-user-refusal-2026-10-09).

Receipt ACK state boundary (2026-10-09, **confirmed static/local**): APK
37.71 DDH preserves ACK status 10/12 and zero time; selected ordinary items
can reach 15A.A07 with a null callback through Cmn. Permanent-failure status
21 skips that update. Deferred jobs preserve key direction/status/time and use
a separate processing budget. The six-topic snapshot closes model/deferred-job
inputs, not complete persistence or primary user-row authority. Rust ACKs still
settle receipt reservations separately from peer facts; no runtime or live change.
[Evidence and limits](receipt-ack-state.md).

Receipt ACK handler bindings (2026-10-09, **confirmed static/local**): APK
37.71 N12 providers resolve read selector 86 to 14c and state selector 262
to 17J. Read processing checks chat type 5; state processing enqueues Cmn
with arguments 0/5000/0. The offline switch renderer now retains every declared
case and rejects malformed tails, closing its former 64-case visibility gap.
The eighteen-topic snapshot preserves concrete worker boundaries; full persistence
and live parity remain unknown. No account state or live request changed.
[Evidence and limits](receipt-ack-handlers.md).

Specialized receipt ACK producers (2026-10-09, **confirmed static**): APK
37.71 read ACKs produce selector 86 bundles; played/server-error produce selector
262 models with arguments 10/12 and zero time. The fourteen-topic snapshot
traces arg1 dispatch to N12 A0d/A0Z or registered/fallback handlers. Concrete
application and persistence remain unknown; these are not peer timestamps or
receipt_user membership. No runtime, account state or live request changed.
[Evidence and limits](receipt-ack-special.md).

Receipt ACK dispatch (2026-10-09, **confirmed static**): APK 37.71 generic
ACKs remove full Cb1 callback keys under a map lock, then separately handle ID
queues. Read-job registration normalizes only device-zero address forms; duplicate
registration replaces callbacks after logging. The ten-topic snapshot retains
null versus original-node callback payloads. Full queue ownership, timeout/error
lifecycle and receive effects remain unknown. No runtime, account state or live
request changed.
[Evidence and limits](receipt-ack-dispatch.md).

Receipt ACK participant guard (2026-10-09, **confirmed static/local**): APK
37.71 Cb1 compares nullable participants. Rust now rejects a participant-less
ACK for a group read/retry reservation; incomplete error ACKs also leave the
saved outcome unknown. Regression coverage checks restart/no replay and complete
matching ACKs. The former CZH attribution is corrected and retained as a negative.
No private state or live request changed; full primary cleanup remains unfinished.
[Evidence and limits](receipt-ack-correlation.md).

Linked metadata observers (2026-10-09, **confirmed static**): APK 37.71
case 26 invokes a no-payload Bjk callback. A physical scan finds thirteen empty
concrete bodies and two state-update consumers; transitive effects/registration
remain unknown. The five-topic snapshot also names the browser/desktop flag
and distinguishes view lookup from raw-cache lookup. No runtime, account state
or live request changed; this is not identity-verification or receive-replay proof.
[Evidence and limits](linked-metadata-observers.md).

Linked-device runnable writes (2026-10-09, **confirmed static**): APK 37.71
DLo case 23, DM7 case 2 and DNh case 20 write history access, last-active time
and nickname. A three-topic anchored snapshot completes immediate classification
of the twenty physical cache stores; transitive authority remains unfinished.
Last-active mutates a record before SQL, and only its separate companion
preference update has a greater-than check. No runtime, account state or live
request changed.
[Evidence and limits](linked-runnable-writes.md).

Linked-device metadata caches (2026-10-09, **confirmed static**): APK 37.71
place/instrumentation writers update SQL, null only the view map and mutate
one existing raw-map record. The five-topic snapshot distinguishes that behavior
from GEu's broader store-reset context. Location requirements, metadata authority,
remaining runnable writers and complete linked-device trust remain unknown.
No runtime, private account state or live request changed.
[Evidence and limits](linked-metadata-cache.md).

Linked-device cache writes (2026-10-09, **confirmed static**): APK 37.71
has twenty physical raw/view cache writes across fourteen methods. The six-topic
snapshot traces metadata writes, logout marking versus row deletion, own-alias
selectors and bounded PN/LID migration resets. Full remaining writer/caller
authority and commit/rollback semantics remain unknown; no cache reset implies
verified identity or removal permission. No runtime, account state or live
request changed; linked-device receipt handling remains unfinished.
[Evidence and limits](linked-cache-writes.md).

Read-job saved linked set (2026-10-09, **confirmed static**): APK 37.71
requires loaded Me and derives its set from saved devices, excluding positive
logout times. The nine-topic snapshot traces SQL loading and bounded own
LID-to-PN device conversion with existing-PN collision preference. Set membership
is not key verification; full ingestion/deletion authority, cache invalidation
and read-self fanout remain unknown. No runtime, account state or live request
changed; primary cleanup remains unfinished.
[Evidence and limits](read-linked-set.md).

Outgoing read-job trace (2026-10-09, **confirmed static**): APK 37.71
SendReadReceiptJob uses Cp2's shared receipt builder and extra-ID list construction.
Its outgoing read/read-self selection differs from the incoming factory policy;
job admission also checks IDs, status age and a saved set. The eleven-topic
snapshot supersedes wire-format inference. Rust still sends one explicit ID
per receipt; complete policy, linked-device scope and scheduling remain unfinished.
No runtime behavior, account state or live request changed.
[Evidence and limits](read-receipt-job.md).

Receipt read-policy classifiers (2026-10-09, **confirmed static**): APK 37.71
uses a configured JID-user list, chat codes -1/-3 and contact-addon presence.
The twelve-topic snapshot retains dependent-account age-state checks and a
predicate that can schedule a refresh before returning true. Effective state,
classifier writers and refresh completion remain unknown; full user-row
admission and primary cleanup remain unfinished. No runtime, account state
or live request changed.
[Evidence and limits](receipt-read-classifiers.md).

Receipt factory read policy (2026-10-09, **confirmed static**): APK 37.71
uses read_receipts_enabled default true plus chat, contact and account predicates.
The eleven-topic snapshot retains literal smb:/ent: prefix checks, premium-message
existence queries, dependent-account preference and the entry thread guard.
Effective preferences/classifiers and settings synchronization remain unknown;
full user-row admission and primary cleanup remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](receipt-read-policy.md).

Receipt state-only suppression (2026-10-09, **confirmed static**): APK 37.71
C8L selects incoming keys and mapped state arguments 13/16/17. Its state-only
caller skips A07 when the chat last-read message row is at least message A0j;
the missing-chat-model getter returns 1. A two-topic snapshot and eleven-write
inventory retain the immediate boundary. Full chat read-state lifecycle,
self/user-row admission and primary cleanup remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](receipt-state-suppression.md).

Receipt state-only admission (2026-10-09, **confirmed static**): APK 37.71
keeps message-key direction separate from receipt status: read-self forces false,
while played-self retains the recipient-presence branch. The ten-topic snapshot
traces A07 advancement, played eligibility and core persistence. A07 true does
not prove every downstream database update succeeded; the base A0V helper tests
chat kind, not a view-once flag. Full self/user-row admission and primary cleanup
remain unfinished. No runtime, account state or live request changed.
[Evidence and limits](receipt-state-only.md).

History mode provenance (2026-10-09, **confirmed static**): APK 37.71
BMv reads positive registration_device_id from preferences; BNm tests loaded
Me presence. These are separate from own-user equality, registration state and
linked-device authentication. An eight-topic snapshot retains mode/getter/setter
and the bounded completion-handler source. Full pairing/crypto and receipt_user
admission remain unfinished. No runtime, account state or live request changed.
[Evidence and limits](history-mode.md).

History queue admission (2026-10-09, **confirmed static**): APK 37.71
stages history chunks in the peer handler after own-user and companion-mode
checks. SQL insertion-constraint recovery updates the original message ID;
queue selection and null/empty binding differences are retained. The four-topic
snapshot preserves full staging/store methods. Upstream crypto, linked-device
trust, media authentication and receipt_user creation remain unfinished.
No runtime, account state or live request changed.
[Evidence and limits](history-queue.md).

Mapping import entry points (2026-10-09, **confirmed static**): APK 37.71's
remaining conversation-mapping callers are chat-transfer import and the initial
history-bootstrap branch. The immediate history inputs are bounded file reads
or inflated inline chunks; nonce presence/storage is not demonstrated nonce
verification. A four-topic snapshot preserves the callers and full worker.
Upstream chunk/transfer authentication and receipt_user admission remain unknown.
No runtime, account state or live request changed.
[Evidence and limits](mapping-admission.md).

Chat counterpart provenance (2026-10-09, **confirmed static**): APK 37.71
uses a HashBiMap populated from distinct positive original chat-JID rows and
from parsed conversation account-LID/phone pairs. Reverse lookup, chat deletion,
cache clearing and a mapping-sync consumer are retained in a twelve-topic snapshot.
The backup restore caller is distinct from direct wire admission; upstream trust,
full sync receiver and receipt_user membership remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](chat-counterparts.md).

Receipt counterpart selection (2026-10-09, **confirmed static**): APK 37.71
8G8.A02 expands selected JID rows only through the migration gate and the
0Yo.A0B(input, false) mapping path; its comparison uses object identity.
JID-row materialization does not establish receipt_user membership. The eleven-topic
snapshot retains the mapping/gate and full untraced chat-cache loader. Mapping
producers, full user-row admission and primary cleanup remain unported.
No runtime, account state or live request changed.
[Evidence and limits](receipt-counterparts.md).

Receipt metadata provenance (2026-10-09, **confirmed static / inferred lifecycle**):
APK 37.71 has four physical route-flag writers and hydrates 1ic mentions from
message_mentions, separately from receipt_user. Clearing its list preserves
lookup bit 1 and the route flag; BotGroupMetadata attachment explicitly sets or
clears a separate bit. The fourteen-topic snapshot preserves these sources and
remaining constructor inventories. Complete user-row admission and primary
cleanup remain unported. No Rust runtime or account state changed.
[Evidence and limits](receipt-metadata.md).

Receipt metadata route (2026-10-09, **confirmed static**): APK 37.71 special
classification requires chat exclusion, flag/first qualifying metadata user,
message A02 and descriptor absence. It does not count recipients; A0z bypasses
metadata selection only. The fourteen-topic snapshot retains the helper chain.
Disassembler opcode-c2 naming is corrected with a literal-byte regression.
Metadata producers, full user-row admission and primary cleanup remain unported.
No messaging runtime, saved identity or reservation changed.
[Evidence and limits](receipt-route.md).

Ordinary cursor status (2026-10-09, **confirmed static**): APK 37.71 0VA.A00
reads status into the virtual message setter and loads server receipt time
without enum conversion or minimum selection. Two physical cursor entry points,
the type factory and one status-1 media preparation source are retained in a
five-topic snapshot. Full outer SQL/cache and user-row creation remain unknown;
no Rust runtime, reservation or account state changed.
[Evidence and limits](message-loader.md).

FStatus conversion (2026-10-09, **confirmed static**): APK 37.71 7z7.A00
maps named 1cE enum ordinals with media/subtype conditions; 7zY.A01 keeps
native 178 integers separate. The ten-topic snapshot retains the transition
matrix and all six direct enum writes. This narrows the earlier restoration
trace: ordinary SQL status loading and universal outgoing initial status remain
unknown. No Rust runtime or account state changed.
[Evidence and limits](status-enum.md).

Outgoing success ACK participant trace (2026-10-09, **confirmed static / tested offline**): APK 37.71
wire parser → device conversion → event 19 → handler proves that a nonnull
converted participant suppresses success acceptance. Rust now guards supported
PN/LID participants, including primary forms, while preserving error ACKs and
original uncertain reservations across restart. The twenty-topic snapshot also
retains native accepted-message SQL, status branches, failure helpers and scoped
poll-path negatives. Native extra user subtypes, complete status/user-row parity
and a qualifying live ACK remain unverified. No sends or account resets ran.
[Evidence and limits](message-ack.md).

### Polls — 2026-09-20

`LX/BYa`, `LX/BWH`, `LX/BOq`, `LX/BQ2`, `LX/BSo` descriptors give the poll
protobufs; a live poll used top-level fields 35 and 64 with `meta
polltype=creation`. `LX/0ZG.A01` shows the vote AAD (`pollId || 0 || voterJid`)
and the inputs handed to native code for key derivation; the derivation itself
is not in DEX. Our vote and our poll were server-accepted; the poll was
delivered; incoming votes decrypted with the documented recipe.

### Quoted replies — 2026-09-20

Comparison 2.26.37.71 descriptors `LX/BVL` (Message: conversation=1,
extendedTextMessage=6, messageContextInfo=35), `LX/BVK` (stanzaId=1,
participant=2, quotedMessage=3) and `LX/BWx` (ContextInfo expiration=25). A live
handset reply to our image arrived as `type=text`, payload fields 6+35, with
`contextInfo.participant` = our LID and `quotedMessage` fields `[3]`. Our quoted
reply (same shape, participant = the contact's LID, quoted = the stored
authenticated payload) was server-accepted, delivered and read.

### Reactions — 2026-09-19

`LX/BWI`/`LX/BZa` descriptors give ReactionMessage (key=1, text=2,
groupingKey=3, senderTimestampMs=4) and MessageKey (remoteJid=1, fromMe=2,
id=3, participant=4); `LX/0zO.A01` lists the `reaction` stanza kind. A live
incoming reaction (`message type=reaction`, payload field 46) decrypted and
projected as a message update; a reaction sent back was server-accepted.

### Live confirmation of 2.26.37.71-derived requests — 2026-09-19

On the saved comparison account: the `iq/encrypt/set` prekey upload built from
2.26.37.71 evidence was accepted (`iq type=result`, 50 keys, no `count_low`); the
`usync devices version=2` query was answered with a `device-list` of one primary
device and no `key-index-list`, consistent with a single-device contact; an
explicit `receipt type=read` received a matching `ack class=receipt type=read`.
These are server-reported outcomes, not handset observations.

### Comparison 2.26.37.71 contact device-list sync — 2026-09-18

Same build. `LX/0u7.A06` builds the `usync` query (`sid`, `index`, `last`,
`mode`, `context`, `query/devices version=2`, `list/user`), `LX/0u7.C8f` parses
`device-list/device[id,key-index,is_hosted]` and `key-index-list[ts,
expected_ts]`, `LX/11T.A07` verifies the signed list with the pinned primary
identity (or an ADV-provided key, not implemented), `LX/11Q.A09` parses
`notification type=devices` and `LX/101.A02` records the user for a sync.
Implemented for contacts with tests; own-account sync and send fan-out remain
open. No live query has run.

### Comparison 2.26.37.71 prekey replenishment — 2026-09-18

A second comparison build, 2.26.37.71 (APK SHA-256 `1755111a…7d5c`, same
signing certificate as 2.26.37.70, different obfuscated class names), provides
the one-time prekey path: generation of `min(812 - unsent, 50)` keys with ids
`((next + i) % 0xFFFFFE) + 1` (`LX/0ko.A06`), the 812-key unsent batch
(`LX/0kf.A01`), the `iq/encrypt/set` upload layout with 3-byte key ids and a
4-byte registration id (`LX/0y0.A00` case 85, `LX/1Lz.A03/A04`), the
`notification type=encrypt count/@value` trigger and its acknowledgement
(`LX/14z.A09/A02`), startup upload of unsent keys (`LX/DNq.run`) and the
`count_low` follow-up in the result (`LX/Bor.A03`). Kyber (`pq_*`) children are
optional in the APK and are not sent. Offline tests reproduce the layout; no
live upload has run. Details: docs/protocol/messaging-transport.md.

### Comparison receive activation and view-once prefetch — 2026-09-18

Derived from 2.26.37.70: `LX/Avw.A00` 0x0008–0068 builds
`iq/id/xmlns=passive/type=set/to=server` with `active` for true;
primary ConnectionThread `LX/0iS.A0e` 0x0014–004a generates the ID and
calls that true branch. `LX/Cns.A00` 0x007c–0108 constructs delivery
receipts with optional type/participant/recipient; `A05` 0x0000–0074
constructs receipt acknowledgements, omitting type for delivery. Our bounded
implementation supports primary direct routes only; read/opened receipts are
emitted only on explicit request (viewer completion or `send_receipt`), never
automatically. Messages must already be authenticated and committed before delivery
confirmation; all incoming frames are encrypted at rest before acknowledgement.

Observed on our comparison lab client: activation result accepted. Activation
alone yielded the five old messages. Adding delivery/receipt confirmations was
followed by four additional authenticated payloads; one had direct ImageMessage
flag 25. A single HTTPS prefetch downloaded 91,562 ciphertext bytes and passed
size/hash/MAC verification. Encrypted state is `downloaded_unopened`, with no
plaintext export or opened receipt. Queue progress followed the confirmations;
the exact internal server queue policy remains unknown. This is not a capture
from an official Android device or evidence for the pinned build.

### Comparison image upload/send — 2026-09-18

Comparison 2.26.37.70: media route query returned nine hosts and auth. The first
route expired before use and was explicitly refreshed; no upload used expired
auth. One encrypted PNG POST returned HTTP 200 with URL/direct_path, followed
by authenticated login and a matching encrypted-image message acknowledgement.
Reusing the private request returned locally without uploading or sending again.
The operator subsequently confirmed receiving the image. APK mappings, limits and commands are in
[messaging transport](messaging-transport.md#image-sending-comparison-lab).
No route credentials, media keys, phone numbers or returned private URLs are
included here. This observation is comparison-only, not the pinned APK profile.

### Comparison inbound messages — 2026-09-18

One bounded authenticated connection captured two `enc/v=2/type=msg` messages.
Both used a primary LID `from` and explicit `sender_pn`; comparison APK
`LX/CmF.A01` 0x0096–00c6 reads `sender_pn` as `PhoneUserJid`. Selecting the saved
pinned PN session allowed authenticated offline Signal decryption of both.
Plaintext/session were committed atomically and a repeated offline run skipped
both. This verifies this mapped established-session case, not general LID or
prekey receiving. No message content, sender values or credentials are recorded.

Subsequent offline protobuf inspection established one extended-text payload
(Message fields 6/35) and one protocol payload (field 12, type 4), not two user
replies. Comparison `LX/BXF.dynamicMethod` 0x0578 and `LX/BXE.dynamicMethod`
0x01c4 supply the field schemas; `LX/C1B.<clinit>` 0x0028–003c labels type 4
`EPHEMERAL_SYNC_RESPONSE`. No new connection or Signal ratchet mutation was needed.

Labels (docs/design/README.md 6.1): `Observed` · `Derived` · `ReferenceOnly` · `Unknown`.
Every important wire constant used by the code points here through a
`ledger_ref` string. **No `Observed` transport fact exists yet**: nothing has
been captured from the pinned Android client or a live server.

Reference client: `com.whatsapp` 2.26.36.75 (code 263607533), APK sha256
`ce746890…922e9`, signing cert sha256 `39:87:D0:43:…:5D:44`. Full record:
`crates/whatsapp-api/fixtures/reference-manifest.json`.

Reference implementations (web multi-device profile, **not** Android primary):
`tulir/whatsmeow` @ 2375e175 (MPL-2.0); `oxidezap/whatsapp-rust` @ 4a0721b (MIT
headline; see ADR 0005 for AGPL files).

| Ledger ref | Fact | Value in code | Label | Source / notes |
| --- | --- | --- | --- | --- |
| `framing.length_prefix` | Frame length prefix width | 3 bytes big-endian | ReferenceOnly | whatsmeow `socket/constants.go` `FrameLengthSize`; whatsapp-rust `framing.rs`. Web uses WebSocket; Android transport (direct TCP vs WS) is `Unknown` (`transport.endpoint`). |
| `framing.max_frame_len` | Maximum frame body | `1 << 24` | ReferenceOnly | whatsmeow `FrameMaxSize`. Local product limit defaults to 1 MiB (`Limits`). |
| `framing.connection_header` | One-time header before first frame | `WA 0x06 <dict_version>` | ReferenceOnly | whatsmeow `WAConnHeader`; whatsapp-rust `WA_CONN_HEADER`. |
| `binary.frame_flags` | Payload flag byte; bit 1 = zlib | `0x02` | ReferenceOnly | whatsmeow `binary/unpack.go`. |
| `binary.tags` | Structural tag bytes | LIST_EMPTY=0, DICT_0..3=236..239, INTEROP_JID=245, FB_JID=246, AD_JID=247, LIST_8=248, LIST_16=249, JID_PAIR=250, HEX_8=251, BINARY_8/20/32=252/253/254, NIBBLE_8=255 | ReferenceOnly | whatsmeow `binary/token/token.go`; whatsapp-rust `token.rs`. |
| `binary.tokens` | Token dictionary content and order | 235 single + 4×256 double | **Observed (client, static)** | Recovered in order from the `const-string` initialiser in `classes2.dex @ 0x126a96` of APK 2.26.36.75 by `cargo run -p xtask -- apk-tokens <apk> --check fixtures/tokens/web-md-v3.json` (2026-09-17); identical to the web v3 list from whatsapp-rust/whatsmeow. |
| `binary.tokens_version` | Dictionary version byte sent in header/prologue | 3 | ReferenceOnly | Not established from the APK (an integer constant, not a string). |
| `jid.servers` | Server names | `s.whatsapp.net`, `g.us`, `lid`, `broadcast`, `newsletter`, `c.us`, `hosted`, `hosted.lid`, `msgr`, `interop` | ReferenceOnly | whatsmeow `types/jid.go`. |
| `jid.ad_domain_types` | AD_JID first byte → server | bit 1 selects LID; bit 128 selects hosted; other bits are not agents | ConfirmedStatic / Implemented | APK 37.71 124.A02 and 1Qe.A00; [source and checks](ad-jid-decoding.md). |
| `noise.pattern` | Handshake pattern | `Noise_XX_25519_AESGCM_SHA256` (IK resume also used by web) | ReferenceOnly | whatsmeow `NoiseStartPattern`; whatsapp-rust `consts.rs`. docs/design/README.md 8.3: hypothesis until confirmed. **APK 2.26.36.75 also contains twelve pattern names including ML-KEM512 hybrids** (`noise.client_patterns`). |
| `noise.client_patterns` | Pattern names compiled into the client | 12 names: XX, IK, XXfallback and nine `*kem*MLKEM512*` hybrids | Observed (client, static) | `xtask apk-strings`, 2026-09-17. Which one the server negotiates/accepts is `Unknown`. `snow` has no MLKEM512 hybrid patterns (ADR 0004). |
| `noise.prologue` | Prologue | `WA 0x06 <dict_version>` | ReferenceOnly | Same bytes as the connection header in the references. |
| `noise.trust_root` | Root public key for cert chain | `1423 7557 4d0a … a8ee 6b` | ReferenceOnly | whatsmeow `handshake.go` `WACertPubKey`; whatsapp-rust `WA_CERT_PUB_KEY`. Signature scheme XEdDSA. |
| `noise.issuer_serial` | Intermediate issuer serial | 0 | ReferenceOnly | whatsmeow `WACertIssuerSerial`. |
| `noise.cert_chain` | Chain structure and check order | `CertChain{intermediate,leaf}`; details/signature(64) | ReferenceOnly | whatsmeow `verifyServerCert`; schema `WACert.proto`. |
| `noise.record_limit` | Max Noise record size on the wire | — | Unknown | Needed to accept `snow` (65,535-byte limit); ADR 0004. |
| `schemas.*` | Protobuf schemas | `WACert`, `WAWebProtobufsWa6`, `WACommon`, `WAAdv` | ReferenceOnly | `proto/whatsmeow/` @ 2375e175. |
| `transport.endpoint` | Chat endpoint (port, TCP vs WebSocket, TLS layer) | — | Unknown | Active pinned profile remains unresolved. Comparison APK 2.26.37.70 uses Java sockets: built-in primary/fallback gateway branches are raw TCP on 443/5222; numbered edges use TLS on 443 and raw TCP on 5222; port 80 uses raw TCP wrapped in bounded HTTP/1.1 chunked streams (`POST /chat`, host `c.whatsapp.net`). |
| `transport.hosts` | Chat gateway hostnames in the client | `g.whatsapp.net`, `g-fallback.whatsapp.net`, `e1`–`e16.whatsapp.net` | Observed (client, static) | `xtask apk-strings`, 2026-09-17. |
| `registration.host` | Registration hostname in the client | `v.whatsapp.net` | Observed (client, static) | `xtask apk-strings`, 2026-09-17. |
| `registration.vocabulary` | Path fragments and parameter names in the client | `v2`, `code`, `exist`, `register`, `security`, `verify`; params `cc`, `in`, `lg`, `lc`, `mcc`, `mnc`, `method`, `token`, `id`, `e_regid`, `e_keytype`, `e_ident`, `e_skey_id`, `e_skey_val`, `e_skey_sig`, `fdid`, `expid`, `backup_token`, `authkey`, … | Observed (client, static) | Presence only; assembly, encoding, order and required subset are `Unknown` (`registration.payload_encoding`). |
| `login.client_payload_fields` | Protobuf field constants matching web `ClientPayload`/`HandshakeMessage` | `CLIENT_HELLO`, `CLIENT_FINISH`, `HANDSHAKE_MATERIAL`, `CONNECT_TYPE`, `CONNECT_REASON`, `DEVICE_PROPS`, `LID_DB_MIGRATED`, … plus new `HN_EPOCH1_EPH_PUB`, `EPOCH1_TRANSCRIPT_HASH` | Observed (client, static) | Corroborates the vendored `WAWebProtobufsWa6` schema as a starting point; primary-device field values remain `Unknown`. |
| `login.client_payload` | `ClientPayload` fields for a primary Android device | — | Unknown | whatsmeow builds a WEB/companion payload; primary markers, version fields and startup requests are not observed. |
| `registration.endpoints` | HTTPS registration endpoints | — | Unknown | Not observed. |
| `registration.payload_encoding` | Request/response encoding | — | Unknown | Not observed. |
| `registration.prerequisites` | Keys/ids needed before first request | — | Unknown | Section 7 requires generating them before the first request that needs them. |
| `registration.verification_steps` | OTP / PIN / other steps and cooldowns | — | Unknown | Not observed. |
| `messaging.id_format` | Wire message-ID format | — | Unknown | Web references use uppercase hex of random bytes; tickets carry `wire_id: None` until observed. |
| `e2ee.*` | Session/message versions, prekey upload, padding, retry | — | Unknown | Gate ADR 0005. |
| `devices.*` | Device list discovery/invalidation | — | Unknown | |
| `media.*` | Media construction | — | Unknown | |
| `store.*` | Local store facts (not wire) | see ADR 0002 | Observed | Tested offline in this repository. |

## Dated client observations (static)

2026-09-17 — APK `com.whatsapp` 2.26.36.75, string tables of `classes*.dex`
extracted with `cargo run -p xtask -- apk-strings <apk>` (449,216 unique
strings). Findings are in `crates/whatsapp-api/fixtures/reference-manifest.json`
under `client_observations`. Presence proves the client contains a constant,
not how it is used; nothing here changes a wire fact from hypothesis to fact
without a capture. Native `libs.so` carries no protocol strings.

2026-09-17 — a separately hashed, WhatsApp-signed newer build
(`com.whatsapp` 2.26.37.70, code 263707033, sha256 `b2090b…ee732`) was used as
a comparison because the official `/current/` URL had advanced and the pinned
APK was unavailable. Structural bytecode analysis observed `WA 06 03`, the
`https://v.whatsapp.net` base plus 17 `/v2/*` paths, UTF-8 URL-encoded form
POST bodies, construction of the `authkey`/`e_*` key bundle, the same
`WhatsAppLongTerm1` root and issuer serial used by the reference code, and
chat connection candidates on ports 443/5222 (with a port-80 fallback). Its
token dictionary differs from the pinned fixture. Its registration bridge
also parses the JSON response status/reason and cooldown vocabulary recorded
in the comparison fixture. The key bundle uses the client Noise public key,
key type `05`, big-endian 4-byte/3-byte identifiers, and unpadded URL-safe
Base64. The `/v2/code` token is derived locally with PBKDF2-HMAC-SHA1 over the
package/logo material and HMAC-SHA1 over the signing certificate, primary-DEX
MD5, and national number. Fresh fallback prerequisites are persisted UUIDv4
values for `fdid`/`expid`/`access_session_id` and 20 random bytes apiece for
the recovery and backup tokens; the exact recipes are in the comparison fixture.
The built-in socket path does not construct a WebSocket: it supplies a Java
`SSLSocketFactory` for numbered-edge port 443, but uses raw TCP for primary or
fallback gateways even on port 443 and for all built-in port-5222 branches.
Dedicated port-80 states wrap those raw sockets in a pseudo-HTTP/1.1 chunked
stream: `POST /chat`, host `c.whatsapp.net`, an 8,192-byte output chunk buffer,
and 1,024-byte response/chunk-line bounds.
These are version-scoped comparative observations only: the
pinned-profile labels above remain unchanged. Full details and hashes are in
`fixtures/comparisons/whatsapp-2.26.37.70.json` and `decompilation-log.md`.

Additional comparison framing observation (2026-09-17): `LX/1NX;->write(VLII)`
rejects lengths at or above `0x1000000`; `LX/1Lz;->A04(LI)` encodes the
accepted length as three big-endian bytes. This establishes a static writer
bound of 16,777,215 body bytes for 2.26.37.70 only, not an observed server
Noise record limit. See the continuation entry in `decompilation-log.md`.

## Dated server observations

The active pinned reference manifest still has `server_observation: null`.

2026-09-18 UTC, comparison messaging lab: the standard snow XX message adapter
received a server hello but failed decryption. APK sequencing showed an extra
empty-payload transcript hash in the adapter, absent from `LX/1NZ.A06`. After
correcting that sequencing over existing cryptographic primitives, the client
verified the WhatsApp root/intermediate/leaf chain and its binding to the Noise
remote static (intermediate serial 3, leaf serial 361). A 34-byte minimal primary
login using saved registration credentials returned encrypted `success`.

One recipient-key query returned identity, signed and one-time keys. The client
verified the signed key, established Signal encryption, atomically committed
session/send intent and transmitted one direct text. An encrypted matching
`ack`, `class=message`, without `error` established server acceptance. A local
repeat of the same request read the persisted acknowledgement and sent nothing.
Handset delivery/read is unconfirmed. No phone, plaintext, key or private stanza
is recorded here. These observations apply to the comparison lab only, not the
pinned default profile. [Implementation and static sources](messaging-transport.md).

2026-09-18 UTC, comparison lab only: one operator-authorized diagnostic
`POST https://v.whatsapp.net/v2/code` reused an existing legacy attempt's
configuration and keys. The server returned HTTP 200 and parsed JSON
`status=fail`, `reason=bad_param`; the bounded response body was 58 bytes.
Parsed retry/SMS/voice waits were absent. No successful code delivery,
registration, or specific offending parameter was established. The new
HTTP status and recognized reason were verified in the encrypted saved state.
No further request followed, and the durable one-shot diagnostic latch remains.
Phone, tokens, keys and raw request/response bodies are intentionally not published.

2026-09-18 UTC, follow-up comparison: an explicitly authorized diagnostic
repeat on the separately supplied comparison number reused its saved keys and
unchanged 612-byte form. HTTP 200 / `fail` / `bad_param` was unchanged; the
response was again 58 bytes. The updated parser retained no recognized `param`
hint (`invalid_parameter: null`). This does not prove that no unknown hint was
present. No new optional fields were populated and no successful signup or OTP
delivery was established. The dedicated bad-parameter diagnostic latch is used;
no automatic retry or identity rotation occurred.

2026-09-18 UTC, APK-only controlled continuation: empty-H, seven fallback
fields, and post-failure reason/metrics experiments each still returned
HTTP 200 / fail / bad_param. The next experiment used the APK's
unknown-build-properties User-Agent and advanced the checked preparation count
to 7. The 1167-byte ENC/H request returned HTTP 200 / fail / format_wrong
(64-byte response, 189 ms). The new refusal is persisted on the same attempt
and account keys. This is changed behavior, not successful registration.
No recognized offending-field hint or wait was supplied. Detailed inputs and
static offsets are recorded in [the HTTP analysis](http-post-analysis.md).

Subsequent APK-only analysis resolved `format_wrong` through response enum 29,
`VerifyPhoneNumber.A19`, and dialog 53 to resource `0x7f123741`: the official
client says the mobile number is invalid for the country. This identifies the
client's interpretation, not the server's internal rule. No additional live
attempt, identity change, or successful signup occurred during this analysis.

Next operator-requested test used a different supplied number and its own
private JSON/store, leaving the previous attempt untouched. The JSON selected
`enc_empty_h`, the APK unknown-build-properties User-Agent, the seven documented
fallback fields, and `client_metrics` preparation count 1; no prior-failure
reason was copied. A phone-bound token was derived from the same comparison
APK. One 1126-byte POST returned HTTP 200 / fail / bad_token (61 bytes, 559 ms),
with no recognized parameter hint or wait. This is a different refusal, not
accepted registration, proven protocol progress, or confirmed OTP delivery.
No retry followed. Because number, fresh identity, and request context differ,
this is not a controlled single-variable comparison with the previous attempt.

Token audit subsequently found a single-byte transcription error in the
PBKDF2 salt: offset 62 was `0xfc`, not the APK's `0xf8`. After correcting it,
the private profile changed only its token. One explicitly authorized retry
preserved the same attempt/account material and all other inner fields
(including the previous metrics value for this token-only comparison).
The fresh encrypted envelope was 1123 bytes. The server returned HTTP 200 /
sent (347 bytes, 3219 ms), retry/SMS wait 125 seconds and voice wait 65 seconds.
The saved state is now an SMS challenge. This is server-reported dispatch;
receipt, OTP verification, completed registration and messaging are not yet
confirmed. No further request or OTP submission followed.

The operator then selected the original historical number again. Its saved
opaque refusal was migrated once through corrected JSON: fixed phone-bound
token, APK unknown-build-properties header, ENC/empty-H encoding, documented
fallback fields and history-checked preparation count 2. Original account keys,
attempt, country/locale and registration IDs were retained. The 1126-byte POST
returned HTTP 200 / fail / too_many_all_methods (329 bytes, 1017 ms), with
32417 seconds for retry, SMS and voice. The service persisted a cooldown;
no further request or delivery-method switch followed. No SMS dispatch or
successful registration was established for this number.

## 2026-10-07: comparison 37.71 sticker descriptor and key domain

**Confirmed/static:** `BZC.dynamicMethod` (`00de..00ec`), with field-name
array helpers `6al.A1W` and `Ayq.A11`, establishes all 22 StickerMessage
fields, including skipped wire tags and integer `premium`.
`1qB.<clinit>` (`0028..0034`, `02a0..02be`) passes the image-key label to
the sticker type. **Negative/tooling:** automatic descriptor extraction refuses
the incomplete directly visible name array; exact-string sticker-key-domain
xref has no references. **Offline:** ordinary URL-bearing WebP receive media
uses the existing bounded authenticated retrieval and sealed references.
**Unknown:** live sticker receive/display, Lottie and directPath-only retrieval.
Full evidence and reasoning: [stickers](stickers.md).

## 2026-10-07: video and PTV receive media

**Confirmed/static, 37.71:** Message fields 9 and 66 reference BZE; its
`dynamicMethod` descriptor (`01a4..01b2`) plus `Ayq.A12` establishes the
30-field shared schema, including caption=7 and viewOnce=20. `1qB.<clinit>`
(`0054..006a`, `02e4..031a`) passes the video-key label to both types.
**Negative/tooling:** automatic field extraction refuses helper-filled names.
**Offline:** ordinary URL-bearing MP4 retrieval and event/reference projection
use the distinct domain and existing bounded downloader. An independent
Python/OpenSSL fixture checks domain separation and journal/retrieval behavior.
**Unknown:** live incoming/playback, directPath-only retrieval, HD variants,
circular rendering, sending and view-once video consumption.
[Full video evidence](video-media.md).

**Observed/negative live check:** a rebuilt saved-account 60-second receive-only
window authenticated/activated and ended cleanly. No new event or receipt was
recorded, so it does not verify incoming video/PTV or automatic reads. The
complete 20-event journal and nine acknowledged receipts were unchanged; the
existing uncertain action was preserved. Private evidence paths are in the
video document.

## 2026-10-07: safety-number schemes and packed native boundary

**Confirmed/static, 37.71:** Java `0jc.A0V` / `CjU` uses sorted serialized
identity-key sets, normalized UTF-8 identifiers, 5200 SHA-512 iterations and
six five-digit decimal groups per side. Its legacy QR uses version zero and
raw concatenated public identities. **Refuted:** A0V is not a native wrapper;
one diagnostic SHA-256 pin is not its safety-number input. **Unknown:** native
A0T/A0U algorithms, active UI scheme, inventory producers, modern QR comparison
and human verification. The native ELF is an outer superpack archive container;
ordinary strings/export inspection cannot establish absence of fingerprint code.
The disassembler now names the seven previously opaque opcodes used in these
traces, with literal DEX fixtures. [Full evidence](safety-number.md).

## 2026-10-07: native decoder and loader baseline

**Confirmed/static, 37.71:** the APK includes separate ARM64/x86_64 Superpack
decoders; the earlier scoped name search missed them. One bounded XZ stream
reveals V3/V4 fingerprint creation/comparison anchors, but does not reconstruct
an ELF (all four ELF-magic matches have invalid header bytes). x86_64 JNI
relocations and entry-point disassembly establish the `spo` branch and archive
start/end resolution; reconstruction internals remain unknown.
**Observed/local:** a pinned four-file public Qiling rootfs and Python 3.12
environment run hello to `exit_group(0)` with a narrow proc-exe stat adapter.
Python 3.14, single-threaded futex and proc-exe stat failures are preserved,
along with remaining loader warnings. No APK decoder or live account request
was executed. [Hashes, offsets, reproducer and limits](native-unpacking.md).

**Follow-up, confirmed/static:** ARM64 JNI relocations identify range entry
`0x9a50` and simple entry `0x9ec8`; wrapper `0xa964` has an exclusive upper
range bound. **Observed/local:** own-source, fail-closed asset/JNI adapters
allow the exact decoder to load and register 19 methods, returning `0x10006`
and guest exit zero. First-entry reconstruction reaches the work queue but
returns native `9/2` / `AssetDecompressionException`, exit 7, no file. Real
host meminfo removes one warning but not this failure. The cause is unknown;
neither equal printed byte counts nor initialization prove extraction.
**Refuted/fixed:** Qiling's threaded runner ignores its ordinary timeout/count
arguments. Parent-process wall limits and shared block charging now enforce
bounds, tested with actual hello/infinite ARM64 guests. An initial C shadowed
JNI pointer and proc-mapper API owner error were our harness bugs, corrected.
No live account request occurred. [Full follow-up evidence](native-unpacking.md).

**Observed/confirmed, worker follow-up:** stream 6's produced/declared sizes
are 125448, but requested bytes are zero. It reaches the oversized-file branch
`0x1efa4`, not a traced checksum mismatch. Producer `0x1e544..0x1e564` and
worker `0x1ef00..0x1f078` distinguish the requested and declared fields; the
diagnostic prominently prints the latter. Partial-range activation of the
indexed requested-size selection remains inferred. Full `decompress_from_so`
checks first timed out with no output/refusal; this refutes generalizing the
range's `9/2` result to a demonstrated full-extraction failure. Read-only
hash-pinned tracing and a normal full-entry probe are now reproducible, with
tests for preserving partial files and rejecting unknown decoder layouts.

**Observed/local positive:** QEMU 11.1.1 completes ordinary full extraction
through the same unmodified decoder and own-source JNI probe. Executable-mode
and read-only mount-point setup failures are corrected reproducibly. The
isolated checked-in wrapper returns process zero and reproduces all 79 unique
ELF hashes (80 names, duplicate libsodium.so), totaling 62743576 bytes. All
parse as little-endian ARM64 ET_DYN with bounded file segment extents.
**Confirmed/static:** reconstructed libwhatsapp.so contains the four V3/V4
fingerprint anchors and identifiable ADRP/ADD references/unwind ranges.
Dynamic exports/direct relocation references are absent for those names;
input/crypto algorithms and dispatch mapping remain untraced. The redundant
Qiling long run was explicitly terminated after QEMU success. No account
state was opened or live request sent. [Complete evidence](native-unpacking.md).


**Confirmed/static, reconstructed 37.71 libwhatsapp.so:** shared V3/V4 hash
helper `0x713938..0x7139e4` finishes/restarts SHA-512 once plus 5199 further
rounds, hashing previous output plus its second data argument. Both create
paths concatenate sorted key data and call H(K,K); their display routines
construct a separate identifier-bound seed with two zero bytes. V3/V4 params
place PN/LID inputs differently; JNI-backed normalization remains untraced.
Sort slot `0xd1c2d0` resolves to `0x712ef0`, which requires equal-length
representations and uses memcmp. Modern QR field mapping/current scheme and
human verification remain unfinished. **Observed/local:** two bounded native
control-flow slices with host-modeled SHA/MCF calls agree on 5200 finishes and
10400 updates. This does not execute native crypto or verify an account.
[Full source addresses and model limitations](safety-number.md).


**Confirmed/static, same 37.71 reconstructed ELF:** stable PN/LID adapters
`0x6b7a04` / `0x6b7a94` call Java `JniBridge.jnidispatchOO` selectors 1/2,
resolved through initializer method slot `0xd36ae8`. The PN/type-zero branch
uses HNy shortening; type-18 LID branches return the parsed JID user and native
conversion rejects non-ASCII output. Key constructors resolve raw 32-byte sort
data versus full 33-byte serialized hashing data (raw adapter prefix 0x05).
Java V4 creation selector 2 now connects through the registered dispatch entry
and context wrapper to the V4 native create routines. Its result selector 4
passes QR serialization version zero; alternate selector 5 passes one.
This refutes inferring QR version 4 from the V4 algorithm name. Current UI
scheme/input inventory and individual QR fields/comparison semantics remain
unfinished. Exact-class getType searches missed the obfuscated LID class;
0ZY.getType confirms type 18. No live request or account verification occurred.
[Complete callback/relocation/DEX evidence](safety-number.md#native-to-java-identifier-callback-and-dispatch-mapping).


**Confirmed/static, 37.71 native QR:** libwhatsappmerged.so initializer and
unpack wrappers resolve two protobuf-C message descriptors and HostedState.
FingerprintData has six optional fields: public_key, PN, LID, username,
hosted_state and hashed_public_key. The compact committed schema artifact
records actual tags/types/offsets/addresses; upstream descriptor ABI matching
is not proof of the APK's protobuf-C version. Serializer version zero carries
concatenated serialized keys, version one H(K,K), not the display digest.
V4 can deliberately omit local PN when a username exists. Comparison resolves
orientation by LID and requires selected key equality plus identifier conditions;
matching LID/key alone is insufficient. Hosted enum is E2EE=0/HOSTED=1; unknown
is params sentinel -1, not an invented enum. Username comparison flag semantics,
complete error remapping and verified-state action remain unfinished. Fourteen
local tooling checks pass; no live scan or account verification occurred.
[Schema and comparison evidence](safety-number.md#native-qr-schema-serialization-and-v4-comparison).


**Confirmed/static, 37.71 fingerprint comparison caller:** KXj.A01 invokes
V4 selector 4 with a native holder and feature 0x1d2d; 1xm.A1P accepts exactly
zero. NFC/camera/async callers and success callbacks reach presentation through
A18. No persisted verification action is established; delayed callbacks and
writer chain remain unknown. Other same-dispatch operations are not fingerprint
evidence. [Selection and action trace](safety-number.md#java-comparison-selection-and-scan-result-action).


**Confirmed/static, 37.71 verification writers:** Cg3.A00 checks rendered
identity then rechecks serialized bytes under the address lock before updating
verification metadata and sequence. Companion peer actions require a LID,
32-byte key and fresh positive sequence; primary inbound actions drop.
DNu history applies a distinct BT5 path. BWE descriptor tags 1..4 and
ProtocolMessage field 32 are recovered. Delayed scan callbacks are presentation.
No live verification/metadata update occurred. [Writer evidence](peer-identity.md#verification-metadata-writers-and-rotation-guards-3771).


**Confirmed/static, 37.71 local verification control:** DUm selector 15
binds primary LID, rendered key and saved verified bit; 6Ac installs 5XZ.
Its marked branch schedules unmark, while unmarked opens confirmation whose
positive callback schedules mark. Runtime feature/exclusion predicates and
complete fingerprint inventory remain unknown. Two missing DEX field-write
mnemonics are fixed with literal APK bytes and actual APK re-rendering.
530 all-feature tests and strict Clippy pass; no account state changed.
[Control and decoder evidence](peer-identity.md#local-verification-control-and-captured-identity-3771).

### comparison.binary.tokens.37_71 — confirmed static

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
`1Rl.<clinit>`: typed indexed stores publish 236 single and 4×256 double tokens.
Four historical comparison page/index-1 values are refuted by actual stores;
this establishes an extraction defect, not an upstream dictionary change.
[Token extraction](token-extraction.md) carries offsets, negative results and
runtime fixture provenance. All 1,260 entries equal the separately held web-v3
fixture. Live acceptance of the four corrected slots remains unknown.
[Core release review](core-release-review.md) records 22-topic coverage and seven
manual relocation decisions; text differences do not establish behavior changes.

### comparison.group.subject.typed_37_71 — confirmed static

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
0hs.A09 calls signed Long.parseLong; A08 is the separate optional-default path.
0Fs.A0A reads cached 0Ze.A04; A02 ties it to database group_type at 047c/0aa4.
0yv.A09 branches on types 1/3 at 477c..478c; typed message constructions and
Bo3/BoJ getters/setters are preserved in [source evidence](group-subject-types-evidence.txt).
1bh.<init> stores the supplied system kind in A00. Full-signature constructor
resolution, message storage and receive parity remain unknown/unimplemented;
see [interpretation](groups.md#subject-typed-branches-and-signed-timestamp-contract-2026-10-07).
The long parser diagnostic also matches integer/reference-valued methods;
method_shorty JLL distinguishes them without class-name fallback. VLL yields
no_matching_shorty and no proposal. Two string-free helpers remain unanchored.
0Xx.A02+00ba raw 16 02 ff ff 31 01 09 02 occurs once in classes.dex at 0x5cbc2e:
-1 insert-result comparison, not 65535. Corrected display retains hex bits and
signed meaning. No server request or saved-account access occurred in this work.

### comparison.group.info.recovery_37_71 — confirmed static

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
0Xm.A0H constructs false/false A9r arguments for normal mode 2; 5/6 alone use
its in-flight-set branch. A0F consults 8km.A02's 60,000-ms local monotonic guard,
then normal-JID dispatch submits 0Xo.A02's MEX QueryGroupInfo. That context
builder maps ten explicit reasons, unknown reasons to UNKNOWN, and a private
set membership override to MISSING_PARTICIPANT_IDENTIFICATION. Our working
classic IQ info path is not this recovered APK path. 9Np selector-0 callbacks
and numeric error 429 handling are partial; no retry duration was established.
[Evidence](group-info-recovery-evidence.txt) and [interpretation](groups.md#subject-recovery-mex-dispatch-and-local-guard-2026-10-07)
carry offsets, reasoning and remaining unknowns. No client MEX route, handler,
server wait bypass, identity reset or live request was added by this research.

### comparison.mex.group_query_37_71 — confirmed static, offline Rust port

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
QueryGroupInfo server ID 37759635623635554 is selected from the namespace's
persisted-ID asset (SHA-256 a528a786209342b3e1d8cf8a78dc046599731c3842ea4bc3b316c64647bdfec0).
164.CFb reads doc_id; the separate AFf client_doc_id feature path is not this
JSON/XMPP ID. Lae/15t/161 resolve it; Lai constructs queryId/variables and passes
it to BpQ selector 3 and 9QZ(VLLLL). The same ID appears in trace/flow_id,
query_id and JSON. 1S7.getServer, 8kY.A1H, 8kX.A0T and 8ka.A0a close the
server-only destination/type/id helper trace. GraphQlCallInput.A00/A01 confirms
recursive input objects and 1xg.A1P JSONObject string serialization. Missing
registration returns null, not a guessed identifier. Source assertion semantics,
actual namespace-registration filename producer, Pando/Argo feature selection,
timeout units and complete response/error conversion remain unknown.
[Interpretation and release/CLI workflow](mex.md),
[request evidence](mex-request-evidence.txt) and
[partial response evidence](mex-response-evidence.txt) preserve reasoning and
negative results. LaT chooses result/update; LaZ distinguishes data/errors
parse failures; 2FU itself has no schema methods. Initial 0Xo.A03 reads
xwa2_group_query_by_id through 2FT/90x/ADv, not a proven complete projection.
Rust's offline builder and selected-ID exporter are implemented and invokable;
no durable MEX action, response application, HTTP route or live verification
is claimed. Existing account identities, reservations and snapshots untouched.

### comparison.mex.json_response_37_71 — confirmed static, offline response-parts port

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
00S/1UE provider 0x1763 resolves through the checked packed-switch entry
(classes.dex 0x909e70 fd010000, target A0E+03fe) to 0yN.A00/O4r selector 0.
Lab/LaZ constructors resolve selectors 2/1/4 for nullable object data, nullable
array errors, and malformed-errors exception conversion. Independent failures
preserve the other field. LZw stores data/errors; LZv's nonempty flags do not
establish group success. 90x.A0J confirms subject/8yq rather than the earlier
inferred getter interpretation. Complete callback, per-error model, Argo/Pando,
Android lexical JSON acceptance and group projection remain unverified.
[Interpretation](mex.md#json-response-field-port-2026-10-07),
[source evidence](mex-json-parser-evidence.txt) and
[raw switch entry](mex-json-provider-switch.json) preserve offsets and reasoning.
Rust response-parts parsing and a counts-only CLI inspector are implemented,
including explicit invalid states, preserved counterparts and redacted Debug.
Sixteen synthetic getter-derived combinations exercise result/update; no
server response, live request or durable group-state update is claimed.

### comparison.mex.group_callback_37_71 — confirmed static, offline gate port

APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
Lac/LaT typed model conversion dispatches Okj.C2c; 9Np directly extends
BaseMexCallback, whose A03 consults A06 before nonempty-data A04. Group
selector-0 A06 always returns false, including 429 handling: partial data with
errors cannot reach success callback. I1H requires every error entry to be a
JSONObject. 2FU requires nonnull data; A04's 0yw.A0x gate can skip conversion.
[Source/interpretation](mex.md#group-callback-gate-and-inherited-dispatch-2026-10-07)
include IWF extensions/root code precedence, backoff/severity/path/message,
selected-error ranking boundary and partial ADv/LbE getter findings. Full
transport classification, exception/coercion/ranking, native/Pando/Argo,
converter and durable application remain unfinished. Rust group disposition
and the optional offline --group-info inspector are implemented; no live
request or account/state change is claimed. apk-smali's checked superclass
header exposes inherited ownership rather than guessing absent functionality.

## comparison.mex.callback_source_coverage_37_71 — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [eighteen-topic recipe](apk/recipes/2.26.37.71-mex.json) now reproduces
Lac.BEE, LaT.A00, BaseMexCallback.A03, 9Np.A06, IWF.AYS and I1H.A06 as
separate versioned methods with Rust ownership, on top of twelve unchanged
request/parser topics. Exact export/check: 23 files/76,908 bytes.
[Coverage diff](apk/mex-callback-coverage-diff.json) compares old Git cdd4056
against this change, using the same APK hash: six added, twelve unchanged;
targets metadata changed, other three artifacts unchanged, 0.0069 seconds.
This is cached source coverage, not a new release or six protocol changes.

[Actual remap](apk/remap-evidence/2.26.37.71-mex.json): eight unique, seven
unanchored, three ambiguous. IWF.AYS code/error_code plus shorty I is unique;
is_retryable plus shorty Z still matches I1H.A06 and 916.BGC/BQ2. Unknown
new-release mapping remains explicitly unresolved; no full recipe proposal.
[Interpretation](mex.md#versioned-parserrequest-files) distinguishes source-only
getters and callback admission from unfinished converter/live application.

## comparison.mex.group_converter_branches_37_71 — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Selected source](mex-group-converter-evidence.txt) preserves full 0Xo.A03,
90x getter fields, required properties getters, 8kY.A0B/0Gv.A01, group id parser,
creator-id getter, feature-gated 0Xo.A01/A00 and required LbE.A05. Four
__typename hash branches construct A4m with distinct integer/flag inputs;
unknown hashes/null group object throw, subject is nullable, participants and
properties are required. Exact wire type names and A4m slot mapping remain
unknown. Searching 453,954 escaped DEX strings found none of the four hashes;
this does not exhaust runtime assembly/unescaped UTF-16 names.

Group/subject times multiply parsed long by 1000; announcement_version does
not. Raw classes6.dex 0x26374c bytes `16 00 e8 03 bd 02 10 02` independently
prove 8kY.A0B's opcode/operands; xtask now names bd mul-long/2addr with an
independent regression. Parser catch types/coercion/storage units remain
unknown. Creator is creator.id, with feature 0x36fe controlling additional
LID/country maps. [Interpretation and boundaries](mex.md#group-converter-branches-and-time-arithmetic-2026-10-07)
record source facts without claiming a live or completed Rust converter.

## comparison.mex.constructor_and_catches_37_71 — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[A4m/catch source](mex-group-constructor-catch-evidence.txt) and
[literal code_item](mex-long-parser-catch-fixture.json) resolve group type
A03, distinct creation/subject/version/appeal long slots and ordered map/flag
inputs. Wide arguments occupy two words; interpreting every argument as one
register would mislabel later fields. [Full mapping/boundaries](mex.md#result-constructor-and-explicit-exception-inspection-2026-10-07)
retain unknown enum/getter meanings and unfinished state application.

0Gv.A01 catches NumberFormatException over code units 7..12, handler 12 returns
the supplied default (zero at traced callers). This closes the previous
unknown parse-failure path, not all Java numeric lexing/coercion. LbE.A04
catches four reflection exceptions at one null-return handler; it has no
arbitrary Throwable catch. 0Xo.A03 has no local try range; caller handling is
not refuted. Reusable apk-smali --exceptions adds checked table inspection,
synthetic malformed/overflow/catch-all tests and independent APK regression.
Source format AOSP DEX code_item/try_item/encoded_catch_handler_list;
no live account/server validation or full group converter claimed.

Inspection failure correction: apk_smali previously warned on a later DEX
parse error and returned earlier methods. It now propagates the named file
error before rendering any method result; two-DEX tests cover malformed bytes
and invalid catch tables following valid methods. This changes failure
reporting, not valid default output or protocol state.

Core rendering correction: the full 37.71 check found noise-handshake
+045e..0460 op=bd had retained ?? after the preceding multiplication-name fix.
Searching for CLI-style ??(0xbd) missed bundle op=bd syntax. Exact replacement
shows no other method-content change; same operands/offset/opcode. Only that
file/checksum refreshed, core 27 files/731,494 bytes. [Cached diff](apk/core-mul-long-rendering-diff.json)
against Git 34b5ab5: 0.0328 seconds, one changed/21 unchanged topics, all four
artifacts unchanged, same APK. This is presentation, not a protocol delta.

## comparison.mex.numeric_projection_37_71 — 2026-10-07

**Confirmed static APK calls**, build 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
ADv.A04/A05 -> 8kY.A0B -> 0Gv.A01 typed NumberFormatException/default zero,
then signed multiplication 1000; ADv.A03 unscaled; A4m A0A/A0B/A08 slots.
Rust GroupNumericMetadata now projects the three numbers after callback gate
and required group-object checks; it does not validate/apply a complete group.
CLI --group-numbers (requires --group-info) explicitly prints private numeric
values as signed decimal strings. Default summaries/Debug remain redacted;
blocked callbacks/wrong group shape refuse without partial output.

**Executed host reference / inferred platform compatibility:** independent
[Java oracle](MexJavaNumericOracle.java), Temurin 21.0.12.1+1 LTS, produces
[479 cases](../../crates/whatsapp-api/fixtures/mex/2.26.37.71-numeric-reference.json)
and the complete UTF-16 digit-domain enumeration (370 digits/37 BMP blocks).
Tests compare parsing/default/wrapping and all 65,536 code units. AOSP
JSONObject/JSON String.valueOf supports inferred numeric coercions; strict
JSON lexical acceptance and live Android SDK/Unicode behavior remain unverified.
[Interpretation/reference links](mex.md#numeric-group-projection-into-rust-2026-10-07)
separate APK evidence, host reference, inferred coercion and unknown runtime.
No account opened, saved identity/state mutated or live request sent.

## comparison.mex.participant_roles_37_71 — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
2hu enum ordinals, LbE.A0C -> 4R7.A00 Locale.US/no-trim/typed fallback,
3c5.A0C ordinal and both ADv.A0D stored-role branches. Rust nullable-string
role helper now preserves member versus unrecognized; CLI mex-role-inspect
prints only recognized output. **Executed independent host reference:** 91
Java cases, Temurin 21.0.12.1+1 LTS; complete Android Unicode parity unknown.
[Findings and boundaries](mex.md#participant-role-conversion-and-member-map-boundaries-2026-10-07),
[attributable selected bytecode](mex-participant-role-evidence.txt).

Additional **confirmed static prerequisites**, unported: matching phash returns
null (retain intent inferred; downstream application untraced); Ayj.A1R true only for integer 1; BotJid branch/domain bot/type26,
1..20 ASCII digits/no leading zero; separate strict UserJid branch and optional
aliases; 32q alias type filtering. Unknown wire typename hash preimages are
preserved as unknown; complete list/coercion/device/result-field mapping and
member application are unfinished. No saved account or live server touched.

## comparison.mex.exception_aware_release_coverage — 2026-10-07

**Confirmed static/tooling**, same APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
recipe expands to 38 methods / 43 files / 162,435 bytes and opts into explicit
catch tables via include_exception_tables=true. Default false preserves old
formats. [Actual cached diff](apk/mex-participant-coverage-diff.json), Git
78a7218 old paths: 0.023364 seconds, twenty added/eighteen annotations-only
changed; prior instruction content identical, inventory/manifest/tokens same.
Fixture-based catch-only mutation regression proves the diff blind spot fixed.

**Negative relocation results:** [report](apk/remap-evidence/2.26.37.71-mex.json)
14 unique / 17 unanchored / 7 ambiguous; creation_time/JL matches both times,
invalid-bot/LL both parsers, bot/L four methods. No automatic full proposal.
**New static catch facts:** 0hj.A03 local catch-all 0009..000e -> 000e, versus
0D1.A00 NumberFormatException 0106..010e -> 0191, code units. Complete members,
device/alias/feature/state conversion remains unknown/unported; account state
unchanged and no live request. [Reproduce/interpret](mex.md#exception-aware-numeric-and-participant-release-coverage-2026-10-07).

## comparison.mex.participant_model_and_edges — 2026-10-07

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
LbE.A0A/A09 object-array filter and empty fallback, 8yl constructor, 31c
field slots, 1xf.A17 int-to-long join boxing, 8kW.A0q nullable-history value
boxing, 3c9.A0E optString typename/hash. A0F is not a required field reader.
**Independent literal class_def evidence** proves 0hh -> abstract 0ZX ->
UserJid; 0ZX has no class data, not no class. Strict UserJid is not PN-only or
disjoint from bot. [Full findings/sources](mex.md#participant-model-fields-and-object-edge-projection-2026-10-07).

Rust group_participant_edge_roles and explicit offline --edge-roles now expose
ordered recognized roles of object edges, before phash/model/JID validation.
Eighteen synthetic cases cover both payload children; private diagnostics and
default summary remain redacted. This is not a member map/state application.
Constructor A08 extra string, final join-time unit, device hierarchy and full
member/alias/boolean/feature application remain unknown/unported. No saved
account access, identity reclassification or live request.

## comparison.mex.participant_scalars — 2026-10-07

**Confirmed static** APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
LbE.A0G/Ayj.A1R optInt==1 then optBoolean; ADv.A0D/1xf.A17 positive join int
boxed long, isNull/8kW.A0q nullable history. **Inferred platform / executed
reference:** six hash-pinned AOSP org.json sources commit de876a01b29230b877c9f408348c38a90ee724a5,
Temurin21.0.12.1+1 with compile-only annotation stubs, 408 independent cases.
**Observed local defect fixed:** default serde float parser rounded
0.99999999999999994 to1, falsely setting phash true; float_roundtrip restores
Java-reference below1 result. The exact literal remains a regression.
[Sources/oracle/semantics/unknowns](mex.md#participant-scalar-coercion-and-float-rounding-2026-10-07).

Rust scalar projection/explicit --participant-scalars now exposes private
phash/history/join, with redacted Debug/default output; integer Number wrapping
versus string/floating saturation and decimal/hex/suffix/NaN/Infinity supported.
This is not a member map or state application. Full JSONTokener/Android runtime
parity and remaining member/JID/alias/feature assembly stay unverified/unported.
Four topic owners add the coercion Rust module; 38 unchanged topic texts,
owner-only cached diff0.016635s, 43 files/162671bytes. No account/live action.


## comparison.mex.participant_addresses — 2026-10-07

**Confirmed static** APK 2.26.37.71 / SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
0D2.A02/A04, strict 0ZZ.A00/0D9.A00, PhoneUserJid/0ZY/9Lm constructors,
0D9.A02, 3c8.A1a, 0CM.A0J, DeviceJid and device-subclass constructors.
Thirteen literal class definitions prove DeviceJid is outside UserJid:
strict participant/alias checks reject device wrappers; plain bot/interop qualify.
Phone prefix/length/ASCII rules and plain hosted-domain normalization are traced.
Interop uses last hyphen and signed Java parseInt; no unsigned-only rule inferred.
[Source, reasoning and negative results](mex.md#participant-address-validation-and-strict-type-boundary-2026-10-07).
Remaining unknowns: singleton types, platform parseInt parity, Ble final assertion,
full member assembly and phash downstream application. Existing send policy,
API/CLI capabilities, saved account state and live transport remain unchanged.


## comparison.mex.strict_participant_address_port — 2026-10-07

**Confirmed static** APK 2.26.37.71 / SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
0D2.A00/A02/A03, seven scoped singleton/factory classes, nineteen literal domain
class definitions settle strict UserJid special cases and non-user domains.
Server/0 and bare Me placeholders qualify; gdpr/bare-server/group/broadcast/
newsletter/call/temp/interop.g.us do not. **Implemented** ParticipantAddress
strict primary parsing and redacted offline inspector, separate from send policy.
283 executed host Java Integer.parseInt cases validate platform conversion;
Android Unicode/cache parity and member/alias/durable application remain unknown.
Observed negative-prefix CLI flag bug fixed with regression and actual CLI checks.
[Sources, reasoning and reproduction](mex.md#strict-member-primary-address-parser-in-rust-2026-10-07).
Release recipe now 50 methods / 55 files / 191422 bytes, twelve additions,
38 prior texts unchanged; cached diff0.051814s. No saved account/live request.


## comparison.mex.nullable_ordered_members — 2026-10-07

**Confirmed static** APK 2.26.37.71 / SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
LbE.A0E/A0F nullable/default optString, 8kU.A1A id, 32q.A00 filters and inherited
Jid.equals/hashCode/raw formatting (08p.A00 nullable Object.equals).
**Implemented** ADv.A0D member map with distinct phash-null variant, object-edge
filter, Java typename-hash dispatch, bot skip versus strict UserJid/alias failure,
ordered duplicate replacement and mapped role/join/history/display fields.
**Independent host execution:** eight Java String.hashCode cases; 41 synthetic
APK-branch cases through both children. Wire typename names remain unknown;
fixture preimages are not claimed server spellings and were never sent.
[Source/reasoning/limitations](mex.md#nullable-ordered-participant-map-port-2026-10-07).
Explicit local refusal for unported Java floating/structured optString forms;
this is not an APK field-type requirement. Downstream phash handling, complete
group conversion/Android/cache parity and live/durable application remain open.
New private --participant-members inspection composes with existing projections,
redacts Debug/default output and never emits a partial failed map. No account/live
request, sending-policy change or identity reclassification.

0DE.A0Z/A0e/A0X are numeric type predicates (18/19/25; 0; 22), not
instanceof checks. Strict caller alias parsers prevent device aliases from entering
this port; broader 32q callers remain untraced. Phone/LID types 0/18 confirmed.
59 methods/64 files/204600bytes; nine added topics, seven owners updated,
50 prior method texts unchanged; checksum-verified cached diff0.019005s.

8yk/8yg/8yj constructors confirmed as nonnull JSONObject wrappers only;
no additional field validation found. Refusals identify the offending attribute
without its value; diagnostic fixture checks cover id/lid/pn/display_name/__typename.


### Participant string metadata (2026-10-07)

**Confirmed static (APK 2.26.37.71):** LbE.A0E/A0F delegate nullable/default
strings to JSONObject; ADv.A0D consumes them for display metadata/model dispatch.
**Observed reference:** pinned AOSP org.json on host Temurin 21.0.12.1+1 agrees
with the scoped ordered Rust conversion across 424 raw JSON and 2242 IEEE
references. Original source/token distinctions, duplicate ordering, escaping,
container integral reduction and negative zero are preserved. An independent
exact-rational Java 21 formatter implements the specification; no JDK source
was copied. **Observed corrections:** serde's integer `-0` classification needed
scoped normalization; Python splitlines broke literal Unicode separators in
oracle rows and was replaced with physical-LF splitting. Both negative results
and reproduction evidence are in
[the string conversion trace](mex.md#participant-metadata-string-conversion-2026-10-07).
The existing CLI exposes these conversions privately and keeps default summaries
redacted; API metadata remains CLI-only/x-live false. **Unknown:** Android
runtime-version and full lexical parity, actual typename spellings, full group
application/live MEX settlement. Owner-only re-export: 59 unchanged method texts,
64 files/205234 bytes, cached diff 0.038028 s; baseline Git a737c69. No account
state access/mutation or live network request.


### MEX downstream participant storage (2026-10-07)

**Confirmed static, APK 2.26.37.71:** exact A4m.A0O field search finds one
behavioral reader, 0yw.A0X; it forwards the nullable map to A0U. Null skips
A00 membership reconciliation while other group-field updates continue; empty
nonnull maps still reconcile. A00 scales positive 31c.A05 by 1000, otherwise
preserves positive previous 1uU.A01 or uses 08B.A00. 0bA.A00 maps nullable history
to prior integer state: true minimum 2, false null/zero→ 1 otherwise unchanged,
null preserves. **Inferred:** join storage milliseconds/raw seconds from the
multiplication and clock calls. **Observed reference:** pure Rust metadata helper
and CLI match 432 host-Java transcribed branch cases; not Android/APK execution.
**Unknown:** clock calibration, higher history-state meanings, inner storage
atomicity, full alias/device/own-account reconciliation and live MEX durable
application. Those are not inferred from A0i delegation order. Seven methods
added to the versioned bundle; 59 existing texts unchanged,66 methods / 71 files /
285828 bytes, cached diff 0.035567s, baseline 8295e18.
[Trace, negative boundaries, evidence and reproduction](mex.md#downstream-member-storage-and-null-map-2026-10-07).
No private account state or live operation changed.


**Observed research tooling correction (same chunk):** CI hygiene's checksum
pass covered only the hardcoded 37.70 snapshot. It now verifies every discovered
checksum snapshot separately; four bundles/2029764 bytes and 621 local links
passed. A standalone corruption/size/extra-file regression is wired into CI.
Earlier 564315-byte hygiene output did not establish newer snapshot coverage;
MEX exports were separately checked by apk-research-diff. See
[checksum correction](apk-porting.md#checksum-coverage-correction-2026-10-07).
No new protocol/API capability is inferred from this tooling change.


### Member cache and versioned participant hashes (2026-10-07)

**Confirmed static, APK 2.26.37.71:** `1jL.A04/A0T/A00/A0G/A01/A0U/A0R/A05`
trace cache changes, preserve existing devices and nullable history; separate
locks do not prove a database transaction. `290.A02` plus `28z.A00/2l0.A00`
synchronize alias ordinals. A0R/A02 hash user keys; A0N/A0O gather stored device
values, refuting the earlier device-keyed-map uncertainty. `AzR.A03` selects
V2 literal agent zero or feature-0x415b V3 no-agent rendering; both SHA-256 via
Ayj.A19, first six bytes, NO_WRAP via A18, Java UTF-16 sorting, no rendered dedup.
**Observed host reference:** 114 synthetic cases/684 CLI modes match executed
Java; old Rust rules disagree on 2 V1 and 89 V2-set probes. Six redacted refusal
probes pass. Pure offline module fixes default xtask's feature-gating build
failure; no dependency changes. **Unknown:** active feature, Android charset/
subtype equality, complete own/companion inventory and durable/live application.
Automatic group sends still V2 primary-member hashing before device resolution.
[Sources, reasoning, reproduction and negative boundaries](groups.md#participant-cache-and-versioned-hashes-2026-10-07).
Snapshot: 86 methods/91 files/337088 bytes; all 66 older texts unchanged, cached
diff 0.052231 s against 875ed8f. No account access, new numbers or live sends.


### Saved group device hash inputs and send-job consumers (2026-10-07)

**Confirmed static, APK 37.71:** 1jL.A0I/J/K/L and 0bA.A0H/I select cached
PN/non-PN device/user hashes; SendE2EMessageJob.A06 compares separate hashes
in its gated mismatch resolver and emits groupParticipantHash as phash.
DNu/Ceo receipt/message-specific source branches are distinct from fresh group
inventory; 0Z5.A0O is abstract here, concrete behavior remains unknown. A0W
PN-map replacement skips/logs non-PN entries, does not itself refresh hashes.
**Observed local correction:** the planner hashes saved recipient companion
devices plus known own primary, instead of ignoring addressed companions;
independent Python vector 2:Fhoy7LxL versus primary-only 2:79jEAyGD. Foreign
saved devices refuse. CLI group fanout-inspect exposes counts/own unknown
without addresses or implicit hash; --hash reveals only the requested candidate.
Device preparation uses message context. **Unknown:** full own-device/account
sync, fresh/retry selection, active feature and live companion send parity.
34 group methods/39 files/773068 bytes, 20 prior texts unchanged; cached diff
0.045388 s, baseline 0e216b2. Actual CLI checks used only an isolated encrypted synthetic store: default
redaction, explicit hash, repeated counts and content-free refusal passed. No
canonical account opened or live operation performed. 639 workspace tests and
24 default CLI tests, strict Clippy/Rustdoc/reference/hygiene passed.
[Reasoning, sources, negative results and reproduction](groups.md#saved-device-hash-planning-2026-10-07).


### Scoped send-preparation device queries (2026-10-07)

**Observed local correction:** lab group/status fetch_devices formerly queried
only one global 64-user batch and returned success even on refused/partial
refreshes; cached lists could then permit preparation. Selected pending batches
now reserve atomically, process a fixed recipient set once, and require accepted
complete per-user replies before advancing. Distinct-user accounting prevents
duplicate rows hiding omissions. QueryRecord's new omitted_users is nullable:
historical/unsettled unknown stays null, never inferred from old counts. CLI
device-query exposes saved status/counts and numeric codes only, no HTTP/reset.
**Confirmed substrate:** APK 37.71 0u7.A06/C8f, SyncDevicesJob presets and
11T.A07/A08 previously selected request/parsing/verification traces. The 64-user
bound and completeness guard are local policy, not newly proven APK requirements.
**Observed offline:** 130 targets settle 64/64/2, unrelated pending remains;
503 with old lists and duplicate-row omission block; explicit later 404 keeps
prior state, refused record stays refused; reopen/null/redaction cases pass.
**Unknown:** live large-group batches, own account sync, complete ADV-key/fresh
source selection and public-outbox group parity. No APK snapshot change claimed.
[Reasoning, negative cases and reproduction](messaging-transport.md#scoped-device-batches-and-completeness-2026-10-07).


**Observed live follow-up, same chunk:** a 20-second receive-only session after
zero-send/action/receipt/device-pending preflight completed one existing-peer
device read. Saved query: server_acknowledged, one requested/ingested user,
zero partial errors/omissions, no code; pending users returned to zero. Retained
one primary/no signed index list, companions unverified. Nine ACKed receipts
and action counts (1 error/15 ok/1 sent) stayed unchanged; no chat send occurred.
Default incoming-only logging hid the INFO counters and caused a harness-only
assertion failure; the same completed session was inspected, never restarted.
This proves shared one-user accounting, not live selected multi-batch/group send.
644 workspace tests, default24, strict Clippy/Rustdoc/reference passed; actual
synthetic/query CLI redaction and canonical saved-query inspection passed.


**2026-10-07 — background device revision/continuation safeguards.** Confirmed
substrate: APK 37.71 0u7.A06/C8f, SyncDevicesJob presets, 11Q.A09,
11T.A07/A08 and Cp2.A05. No new upstream trace or APK bundle change. Observed
local lost-work races (old reply/new notification and ACK write cut) and single
startup-batch limit corrected using durable local generations and a per-connection
cursor. Exact failed/omitted/unknown users remain held; successful newer work and
other batches advance. Offline actor/reopen tests establish local behavior;
live races, full own-account sync and notification-ID dedup remain unknown.
[Reasoning and regression scope](messaging-transport.md#background-device-revisions-and-continuation-2026-10-07).

Live follow-up 21:57:48 UTC: one existing-peer device read in a bounded
20-second receive-only session saved server_acknowledged, one requested/ingested,
zero errors/omissions/superseded, code null. Device pending returned to zero;
outbox/actions/nine ACKed receipts unchanged, no chat sends. Primary only,
companions still unverified. Live races/large batches/socket-cut remain unknown.
Validation: 651 workspace/all-feature tests, default CLI24, strict Clippy,
warning-free Rustdoc, CLI reference and formatting passed. Six actual isolated
CLI probes plus live saved-query inspection passed; hygiene645 local links,
2405965 bytes/four unchanged APK bundles.


Contact ADV caller trace (2026-10-07, APK 37.71): eleven exact selected
methods now preserve supplied-key verification, stale/invalid caller branches,
notification addition and primary identity/session application. Saved identity
is not always the APK's selected key. Invalid/stale signed candidates do not
apply through the success path; raw-ID mismatch has separate reset branches.
Our current fallback actually drops companions and keeps only raw-ID/time floor;
prior preservation comments were wrong and are corrected. Runtime retention,
verified device authority after identity change, feature 0x3285 selection and
full supplied-key integration remain unfinished; no account/live request changed.
[Evidence and unknowns](cross-device-sync.md#contact-adv-verification-and-retention-2026-10-07).
Core snapshot: 33 topics/161 methods, 38 files/967529 bytes. Cached comparison
against b058f61 took 0.028928 seconds: eleven added, all twenty-two old topics
unchanged; inventory/manifest/tokens unchanged, targets metadata expanded.

Validation: exact APK re-export/check verified all 38 files/967529 bytes;
checksum-verified cached diff preserved all 22 older topic texts. CLI reference
regeneration, four CLI OpenAPI coverage checks, API symbol check, formatting and
staged whitespace passed. Hygiene verified 659 local links and 2642000 bytes
across all four independent APK bundles. This source/comment-only chunk makes
no runtime behavior change; no new workspace-test or live-interop claim.


Device candidate retention implemented (2026-10-07): stale/invalid supplied
signed candidates and unsigned non-hosted companions now preserve cached lists,
pins/sessions and pending work. Fixed local verification_failures counts remain
separate from server errors; device-query exposes counts/null history and send
preparation blocks rejected reads. The receive cursor defers rejected users while
other users advance. Raw-ID replacement/supplied-key application remain unsupported;
no resets occur. Regression covers cache/pin/session preservation, duplicate rows,
encrypted reopen, fresh valid settlement and mixed missing-pin/other-user success.
[Evidence and limits](cross-device-sync.md#rejected-device-candidate-retention-2026-10-07).

Conflicting duplicate-row correction: a valid first row could previously clear
pending before a later refused row. Selection now preflights all requested-user
rows before mutation; exact duplicates collapse, differing rows refuse with fixed
local conflicting_user_rows, preserving cached/pending state in both orders and
across reopen. This is local ambiguity policy; APK duplicate dispatch is unknown.
The fixed local reason set above additionally includes conflicting_user_rows.


Observed live 2026-10-07 22:13:51 UTC: zero outbox/queued-action/pending-receipt/
pending-device preflight, one existing-peer read and a 20-second authenticated
receive-only session completed cleanly. Saved CLI query: server_acknowledged,
requested_users=1, ingested=1, partial_errors=0, omitted_users=0,
superseded_users=0, verification_errors=0, verification_failures={}, code=null.
Pending users returned to zero; outbox, 17 actions (error1/ok15/sent1) and all
nine ACKed receipts unchanged. No chat sends or resets. This proves the supported
unsigned-primary live path and new diagnostics schema, not live rejected-list or
supplied-key behavior; the primary-only list still has no signed index evidence.

Validation: 654 workspace/all-feature tests and 24 default CLI tests passed;
strict workspace/all-target Clippy, warning-free Rustdoc, CLI reference/API
coverage and formatting passed. Fifteen actual isolated CLI probes covered all
nine local reasons, zero success, historical/unsettled null, absent, unchanged
repeat and invalid-ID redaction. Conflicting duplicates preserve cache/pending
in either order and after encrypted reopen. Hygiene663 local links and 2642000
bytes across four unchanged APK checksum bundles passed. No live malformed,
signature-refusal or raw-ID replacement claim is made.


Signed-device authority binding and authenticated refresh (2026-10-07): successful
signed lists privately bind their actual verifier key; historical rows remain
unknown. devices/peer-identity expose current authority flags without the binding
fingerprint. Group/status plans refuse unbound/changed cached companions while
preserving history. Already-authenticated remote primary changes queue refresh
inside their pin/session/archive/event transaction; own PN/LID and companions
are excluded. Receive heartbeat picks up eligible post-startup work without a
notification/reconnect and keeps existing refusal/unknown deferral. Offline
rollback/reopen/rebinding and queued-during-receive checks cover the local path;
live primary replacement/own inventory/supplied ADV-key adoption remain unknown.
[Evidence and limits](cross-device-sync.md#signed-device-authority-and-authenticated-refresh-2026-10-07).


Observed live 2026-10-07 22:33:56 UTC: a zero-send/action/receipt/device-pending preflight,
one existing-peer read and a 20-second authenticated receive-only session
completed cleanly. Saved query: server_acknowledged, requested/ingested=1,
partial_errors/omitted/superseded/verification_errors=0, verification_failures={},
code=null. Pending devices returned to zero; outbox, all 17 actions
(error1/ok15/sent1) and nine ACKed receipts unchanged, no chat sends. Authority:
binding_known=false, primary_pin_present=true, primary_pin_matches=null,
companions_current=false. This accurately reports the unsigned primary-only
reply with no signed index list; it does not prove live companion binding,
identity replacement, own inventory or a supplied-key branch.

Validation: 658 workspace/all-feature tests and 24 default CLI tests passed.
Strict workspace/all-target Clippy, warning-free Rustdoc, CLI reference/API
coverage, formatting and staged whitespace passed. Thirteen actual isolated
CLI JSON/human/request probes covered bound/current, changed, missing pin,
historical unknown, unsigned primary and absent list; no private verifier
fingerprint was exported and --json --request parsed as one object. Fixture
metadata probes test inspection, not cryptographic signing; genuine signed
baseline/replacement tests cover authority separately. The own-primary hook
additionally passed explicit PN/LID/companion exclusions. Hygiene668 local
links and 2642000 bytes across four unchanged APK checksum bundles passed.

ADV identity follow-up trace (2026-10-07, **confirmed static**, APK 37.71):
DNy case 40 sends an encrypt/identity read after verifier failure; DF6/DNk
separate primary identity saves from companion ADV validation. Feature 0x3285
enabled skips immediate application; its raw-ID callback does not establish
ordinary deferred success. Raw-ID cleanup removes remote companions while
explicitly preserving the primary. Remaining key-type/companion verifier,
continuation and cleanup internals are unknown; current full peer-keys queries
are not identity-only recovery. No account or live request changed.
[Source and limits](cross-device-sync.md#adv-identity-follow-up-and-raw-id-cleanup-2026-10-07).

Primary identity-only read staging (2026-10-07): peer-identity-query queues the
APK 37.71 encrypt/identity read once; --show classifies saved primary PN/LID
public-key evidence with exact namespace pin match and redacted failures.
B7H type 5 / first 32 key bytes and companion dual-signature checks are traced.
Four offline regressions preserve state and uncertain/refused IDs. Automatic
ADV-failure scheduling, key adoption, companion policy and downstream success
event/session recovery remain unfinished; presence is not verification.
[Evidence and usage](cross-device-sync.md#primary-identity-only-query-2026-10-07).

Identity-only read live validation (2026-10-07): one bounded 20-second session
settled the existing-peer query ok / primary_identity_received, null failure/code,
PN pin match false and no device-identity node. Returned public identity matched
an already saved verified full bundle in an offline cross-check. No pin/session,
outbox/receipt/prior-action change or chat send; repeat reuses the original ID.
This validates the read, not adoption or automatic ADV-failure recovery.
662 workspace/all-feature tests, 24 default CLI tests, strict Clippy/Rustdoc and
CLI/API coverage passed; core cached diff is 0.053777 seconds.

Sender-key device separation (2026-10-07, offline): APK Cnr retains recipient
and device fields; Rust previously flattened companions into the primary's
record. Storage/libsignal names now preserve the actual sender device and keep
primary history in place. Real-crypto PN/LID three-device store/reopen coverage
passes; companions never borrow a historical primary record. Primary save
callbacks additionally trace remote companion cleanup and sender-key rotation;
full actor/category/alias variants and identity-triggered rotation remain
unfinished. Identity-only queries still do not adopt keys.
[Evidence and limits](cross-device-sync.md#primary-save-callbacks-and-device-sender-key-storage-2026-10-07).

Validation of this correction: 663 workspace/all-feature tests, 24 default CLI
tests, strict workspace/all-target Clippy, warning-free Rustdoc and regenerated
CLI/API coverage passed. The 13 added source topics expanded the core to 60
topics / 197 methods, 65 files / 1,528,287 bytes. Cached comparison took 0.053107
seconds: all 47 prior topics unchanged; manifest/inventory/tokens unchanged;
checksums verified. A rebuilt-client 20-second receive-only session ended
cleanly with peer pins, device inspection, actions, outbox and receipts unchanged,
no added query or chat sends. Private evidence index:
`lab/releases/sender-key-live/active.json`. Live companion decryption and
identity-triggered rotation remain unverified; no primary key was adopted.


Sender-key cleanup trace (2026-10-07, **confirmed static** APK 2.26.37.71):
0jc.A19 deletes one translated chat/account-type/device/bucket tuple;
0kX.A00 and 0kT.A02 preserve chat, bucket and sender device during translation.
This does not establish blanket remote incoming-key deletion. Account-provider
selection is partly traced; feature/alias authority and distribution-history
parity remain unknown/unimplemented. The comparison CLI distributes an existing
SKDM to every target but has no sent-sender-key history; redistributing is not
identity-triggered rotation. No live request or identity adoption occurred.
See [evidence and next implementation](cross-device-sync.md#scoped-sender-key-removal-and-recipient-translation-2026-10-07).


Own sender-key exposure/rotation (2026-10-07): the comparison sender reserves
exact recipient-device exposure with key/session/send state. An authenticated
remote primary change marks indexed own keys pending in the same archive/event
transaction; the next new authorized group/status intent rotates with OCC and
private old-key archival. Incoming chains and uncertain intents are retained.
`peer-identity` exposes redacted pending_sender_key_rotations counts; zero does
not establish historical coverage. APK 37.71 D4x.A00/0jc.A19 motivate this local
subset; full bucket/alias/companion and normal actor group parity remain
unimplemented, and live rotation is unverified.
See [evidence and limits](cross-device-sync.md#durable-own-sender-key-exposure-and-rotation-2026-10-07).


Directory bundle selection (2026-10-07): shared parser now compares user/server/
agent/device without conflating wire-encoding kinds, refuses duplicate requested
user rows and optional one-time keys, and refuses contradictory outer errors.
These local ambiguity guards precede identity/session preparation; other batch
devices remain valid. Existing peer-keys inspection reports invalid_reply and
blocks refresh while preserving its selector and pins. APK 37.71 B7H.A06/CQc.A00
provide address/signature context; exact duplicate policy is not attributed to
the APK. [Evidence and tests](peer-identity.md#exact-directory-bundle-selection-2026-10-07).


Missing group sender-key correction (2026-10-07): supported nonempty skmsg with
no sender key is now Undecryptable, reaching the existing once-only retry queue;
it is no longer silently Unsupported. Empty/structured skmsg with a stored key
cannot be reported Decrypted or delivery-confirmed. Byte/Text ciphertext views
are consistent across direct/group receive. Real-crypto late-key recovery and
direct/group loop write-cut fixtures preserve captures, ratchets and retry
reservations. Handset retry shape is observed; APK 37.71 Cp2.A05 is ACK evidence,
not a confirmed missing-key retry count policy. Fresh live group recovery remains
unverified. [Evidence and limits](messaging-transport.md#missing-group-sender-key-recovery-2026-10-07).


SKDM-only group control deduplication (2026-10-07): real Signal regression
reproduced Undecryptable after a previously accepted control was redelivered
across reopen. Newly authenticated controls now commit sender_key_distributed
markers with key/session state and return Duplicate without another decrypt.
They carry no user protobuf/event/read receipt; lab-inbox --inspect labels them
redacted. APK Message field 2 is existing schema evidence; this private marker
is a local atomicity fix, and APK internal duplicate policy is unknown. No
historical marker synthesis or ratchet rewind; fresh live control redelivery
remains unverified. [Evidence and regression](messaging-transport.md#durable-sender-key-only-group-controls-2026-10-07).


Retry/ACK encoding identity correction (2026-10-07): a reproduced second-retry
reservation caused only by JidKind differences is fixed. Canonical logical
selectors preserve user/server/agent/device and probe all historical kind keys
in the same transaction, retaining pending/unknown/refused/acknowledged records
and original wire. ACKs ignore encoding kind but keep exact logical routes.
Independent historical-hash, reopen/state-preservation, foreign-route ACK and
actual receive-loop cut/reconnect tests cover the correction. APK 37.71 Cp2.A05
is wire ACK context; APK duplicate/retry-limit policy remains unknown. Historical
duplicate rows are not consolidated; live encoding changes remain unverified.
[Evidence](messaging-transport.md#retry-reservation-identity-across-jid-encodings-2026-10-07).

Primary-change encoding correction (2026-10-07, confirmed offline): reproduced
missed own group-key rotation for a remote AgentDevice/0 primary and accidental
own-PN device refresh. Encoding is normalized only after primary agent/device
checks; PN/LID namespaces and companion boundaries remain exact. Redacted rotation
inspection now ignores kind without folding devices. PN incoming and LID signed
bundle archive/reopen, own exclusion, rollback and stale-reservation tests cover
the local correction. APK 37.71 D4x.A00/B7H.A06 remain source context; live alternate
primary encoding and subsequent group rotation remain unverified.
[Evidence](cross-device-sync.md#encoding-independent-primary-change-follow-ups-2026-10-07).

Own sender account selection (2026-10-07, confirmed static APK 37.71): 08i
initializes Apw from self_lid and Apz from saved Me.jabber_id; both retain the
persisted registration_device_id (default 0). D4x.A00 selects LID versus PN using
0bA.A0k -> 1jL.A0Y (group row.A00 == 2), then wraps the own device via B7H.A04.
Me.lid consistency diagnostics are not an established immediate self_lid fallback;
1XY is a call callback, not run. Group-row mode production, async reconciliation,
complete bucket/actor semantics and Rust own-address parity remain unfinished.
No account state, keys, identity policy or live traffic changed.
[Sources and boundaries](cross-device-sync.md#own-pn-lid-sender-selection-and-persisted-device-index-2026-10-07).

Group-mode producer/storage trace (2026-10-07, confirmed static APK 37.71): five
1jL.A00 writes and its constructor input are retained. Cache-miss mode comes
from B7i.A00 chat predicates, not membership suffixes alone; contact mode is saved
through 0Y9.A0s -> 0gt.A0V -> 9R1.A0J in wa_group_admin_settings. Reload 0lO.A01
normalizes case-insensitive pn to pn, other returned values to lid. B7i lookup
and predicate have different absent-contact defaults; broadcasts have additional
diagnostic behavior. Full mode provenance, category predicates and Rust mode/
own-PN fallback integration remain incomplete. No live/account state changed.
[Evidence and negative results](cross-device-sync.md#group-mode-producers-storage-and-cache-classification-2026-10-07).

Group routing before device sync (2026-10-07, confirmed offline): the comparison
sender's catch-all plan fallback returned device-query targets even after route
refusals. The extracted old logic failed a mixed-membership regression. Shared
recipients_for_group now validates namespace/own membership before sync, allows
initial missing lists and excludes/deduplicates own/remote targets. Final plan
still validates device authority before reservation. APK mode/category state
integration and automatic own-PN selection remain incomplete; no new live send.
[Routing correction](groups.md#group-recipient-validation-before-device-sync-2026-10-07).

Concrete group-mode kinds (2026-10-07, confirmed APK 37.71): 0DE predicates
resolve broadcast-list type 3, interop.g.us type 28, status@broadcast type 5,
and cache-group types 1=g.us/2=temp. A0n additionally includes type 28.
Temporary groups are not established as old PN g.us groups. Existing 0Fs cache
and ADs reply classifications are linked; numeric group-type agreement alone
does not establish complete callback/history provenance. Runtime mode/contact
synchronization and unsupported namespaces remain unported; no account request.
[Kinds and remaining gaps](cross-device-sync.md#concrete-jid-kinds-in-group-mode-classification-2026-10-07).

Group-type persistence continuation (2026-10-07, **confirmed static APK 37.71**):
existing-chat updates write group_type through transactional database helpers;
missing chats are left absent. Selected subgroup/zero-update callers and secondary
type-4 model writes are retained without inventing complete-reply enum values.
A0Y's outer A0n guard is narrowed again by A0A/A0m, excluding interop type 28
from the type-3 branch. Runtime contact/type synchronization remains incomplete;
no account/network request. [Evidence and unknowns](cross-device-sync.md#persisted-group-type-updates-feeding-sender-mode-2026-10-07).

Addressing-mode producer trace (2026-10-07, **confirmed static APK 37.71**):
complete binary group replies do not directly read the mode attribute. Generated
validators use exact pn/lid membership; incoming group-message parsing has a
separate case-insensitive PN check. Five contact-mode setter callers now have
selected paths retained, including gated info-error and PN-conversion branches;
triggers, full callback provenance and runtime integration remain unfinished.
No live request or account change. [Evidence and boundaries](groups.md#addressing-mode-producers-and-separate-parser-rules-2026-10-07).

Direct LID representation correction (2026-10-07, **confirmed offline**):
textual and typed sender JIDs now select the same held LID session. A real
Signal regression failed Undecryptable before the fix; both representations
now decrypt and cross-representation redelivery remains Duplicate after reopen,
with no PN session or identity rewrite. APK DEw.CF0 parsing is linked, while
full direct-session producer provenance remains incomplete.
[Evidence and boundaries](messaging-transport.md#direct-lid-sessions-across-attribute-representations-2026-10-07).

Group pairwise companion correction (2026-10-07, **confirmed offline**):
held and pinned companion msg sessions retain their actual participant device;
primary/alias fallback, unpinned and prekey-bootstrap paths are excluded from
this local subset. Real Signal PN/LID primary/device-3 control fixtures now
install sender keys and retain duplicate markers across reopen. B7H forward/
reverse device construction is linked; CPQ.A00 is refuted as a 37.71 session
selector. Full companion bootstrap/identity policy and live device coverage
remain unfinished. [Evidence and scope](groups.md#group-pairwise-session-device-correction-2026-10-07).


### Ordinary companion ADV and pending refresh — 2026-10-07

**Confirmed static, APK 2.26.37.71:** DEs.CF2 writes the `device-identity`
body to Cih.A0V; Cih.A01 passes constructor argument 26 to Bp2.A0L.
11T.A0D's matched-primary/raw-ID/type branch refuses old/equal timestamp
candidates with absent/different device index. 0bN.A05 preserves LbQ.A05
(list timestamp) while advancing A02 (expected timestamp); 0bp.A00/A05 and
LbQ.<init> independently identify the SQL mappings. Treating the newer message
ADV as a verified list timestamp was refuted. Callback selection, cache callback,
full alias fan-out and atomic runtime admission remain unknown/unimplemented.
[Trace and reproducible sources](peer-identity.md#ordinary-companion-adv-producer-and-baseline-timestamps-2026-10-07).


### Known companion prekey bootstrap — 2026-10-07

**Confirmed static / implemented offline:** APK 37.71 11T.A0D matched-held-
primary/raw-ID/type branch (`04c8..05fa`) accepts an equal actual device index
with a non-newer candidate timestamp. Group/status pkmsg now implements that
branch using DEs.CF2's ordinary device-identity body, both ADV signatures and
nested Signal MAC authentication. Exact companion state, one-time prekey,
SKDM/control/message are committed atomically with current authority rechecks.
Own-device/unknown-own guards are local limitations; newer timestamps require
separate pending refresh and are deferred. Five real-crypto tests cover success,
replacement/refusal, authority rollback, reopen dedupe and sender-key fallback.
Direct companion prekeys and wider admission/reconciliation remain unimplemented;
no fresh companion live sample is established.
[Implementation and remaining gaps](peer-identity.md#known-companion-prekey-bootstrap-2026-10-07).


### Direct companion receive/receipt route — 2026-10-08

**Implemented/offline:** same known-companion ADV branch now handles direct
prekeys; actual-device ratchet/event/SKDM routing is separate from primary chat
identity. Alias disappearance dedupes. **Confirmed static 37.71:** Cp2.A00 ->
CmC.A06 retains supplied DeviceJid as direct receipt destination for the traced
nongroup/non-A0Q branch; A0Q types 3/5/6 are not a companion predicate. The
initial Cp2.A06 ordinary-receipt hypothesis was refuted (Bp1/status only).
A0p/CM8 upstream producers and full direct-control parity remain unknown.
[Trace, tests and remaining work](peer-identity.md#direct-companion-receive-and-receipts-2026-10-08).


### Known companion newer ADV — 2026-10-08

**Confirmed static 37.71:** `101.A03`/`AzX`/`DLx.run` case 14 schedule deduplicated SyncDevicesJob; `Lcr.A00` preserves an unresolved expected timestamp on a lower normal response. `0bN.A0H` persists, rather than constructs, that baseline. **Verified offline:** known-index direct/group admission records the expectation and queues refresh only with authenticated atomic receive; signed responses below/reaching it retain/clear pending work. Boolean reset semantics, new-device/own-account application and live coverage remain unknown. See [reasoning and reproduction](peer-identity.md#known-companion-newer-adv-refresh-2026-10-08).


Companion new-device membership trace (2026-10-08, **confirmed static 37.71**): `11T.A0D` removes already present addresses before add-only application; a newer different index does not itself overwrite existing membership. `0bN.A0G` filters ownership; `1IW.A02/A09` selects gated canonical users; `AzR.A00` rebinds device numbers; `B7j.A03` writes user_device rows separately from the verified LbQ timestamp. Added-member participant/observer calls and caller teardown Boolean are traced only to their boundaries. Runtime gate values, full side effects and Rust new-device application remain incomplete. Full raw methods and reasoning: [new-device application](peer-identity.md#companion-new-device-membership-application-2026-10-08). No live request or saved account mutation occurred.


Companion teardown producer (2026-10-08, **confirmed static 37.71**): ordinary CoU.A04 combines 0bq.A04, the local dead_devices registry and a nonnull parsed offline Integer for its final admission Boolean. Offline metadata is separate from retry count/dhash. 1Uf.A06 persists removal markers; A02 rechecks current membership before cleanup, dropping returned-device markers without key teardown. Removal callbacks, transient A09 set and local worker bounds are distinct. Rust registry/reap and new-device application remain unported; no account mutation occurred. Disassembler b2 multiplication now has its proper label and independent literal regression. [Evidence and limits](peer-identity.md#companion-teardown-producer-and-registry-2026-10-08).


Remote newer-ADV application (2026-10-08, APK 37.71 static/offline): absent non-offline companions now enter authenticated direct/group receive and are added atomically with provenance/refresh, preserving verified timestamp. Existing different indices are retained until sync. devices exposes adv_added_devices and human expected_ts/count. Signed reconciliation retains newer allocations and clears provenance only on confirmation/removal. Absent offline candidates remain locally deferred pending teardown lifecycle; alias/participant/own reconciliation and fresh live validation remain incomplete. [Implementation and limits](peer-identity.md#remote-newer-adv-device-application-in-rust-2026-10-08).


Pending removal registry (2026-10-08, **static/offline**): signed device reconciliation records disappeared companions and clears authenticated returned addresses in nullable pending_removed_devices. Legacy null stays unknown until signed reconciliation; unsigned responses do not initialize/clear it. Offline absent candidates proceed only with known no-marker state; pending_device_teardown refuses a marked address. Complete-list CAS includes removal state. Inspection exposes unknown/known-empty/count. No cleanup, identity/key deletion, reset, alias fan-out or worker parity is implemented. [Evidence and limits](peer-identity.md#namespace-local-pending-removal-registry-2026-10-08).


## Contact LID result parsing (2026-10-08)

**Confirmed**, APK 2.26.37.71 (`1755111a…7d5c`): `0u7.C8f` reads
`user/lid@val` as `0ZY` after `usync/result/lid` protocol success;
`0u7.A01`, `A3p.<init>` and `0hs.A0A` close the success-flag/typed-attribute chain.
The earlier Rust `lid@jid` assumption is **refuted**. Strict plain-address
validation and the result gate are now implemented and offline-tested.
Fresh live replies, complete contact state handling and refresh scheduling remain
**unknown/unverified**. The initial four-topic (now seven-topic)
[recipe](apk/recipes/2.26.37.71-contact-lid.json) preserves a compact deterministic
[build-specific bundle](apk/2.26.37.71-contact-lid/manifest.json); exact APK
re-export with `--check` matches. See [reasoning and boundaries](chat-actions.md#contact-lid-response-correction-2026-10-08).


Contact-type follow-up (2026-10-08, **confirmed APK 37.71**): `0u3.A02` writes
its positive account flag only for `N0z.A04=1` (`in`), while `0u5.BF6` handles
`invalid` separately from `in`/`out`. The prior JID-presence existence rule is
**refuted**. Rust preserves nullable recognized type and historical unknowns;
`contacts status` inspects offline without queuing. Full state-machine parity
and fresh live contact replies remain **unknown**. See
[consumer chain](chat-actions.md#contact-type-and-offline-status-2026-10-08).


## Live contact projection and primary wire LID (2026-10-08)

**Observed live:** one original existing-peer query returned `in` with `lid@val`
and successful LID protocol metadata during a clean 20-second receive-only
session. **Confirmed local bug:** no receive projection, plus rejection of
zero-component AD_JID encoding as if JidKind were an APK Java subtype. Both are
corrected; receive/startup projection is real-loop tested. Guarded offline repair
of the same reply recovered LID once with retained prior status and no action,
receipt, timestamp or outbox changes. No request repeated. **Unknown:** native
wire-to-Java factory, fresh post-fix replies and complete contact sync. The
[factory/caller negative trace](contact-primary-wire-evidence.txt) is scoped,
not native recovery. [Full reasoning and operator corrections](chat-actions.md#live-contact-projection-and-primary-wire-lid-2026-10-08).


Primary wire correlation audit (2026-10-08): **confirmed local defects** in
identity-only full-Jid comparison and key-directory per-user refusal comparison.
**Implemented/offline-tested** exact component matching across Plain/AD encodings,
with foreign namespace/device/agent and duplicate identity-only guards retained.
DF6.C8f typed dispatch, DNk.run primary branching and B7H.A06 address construction
are APK 37.71 evidence;
native wire object construction and fresh identity-only AD replies remain
**unknown/unverified**. Devices/successful bundle paths already have component
behavior; group/manual-verification paths are not blindly normalized. Existing
private PN inspection was already received; LID evidence absent. See
[reasoning and negative audit](peer-identity.md#primary-wire-address-correlation-2026-10-08).

Directory refusal attribution (2026-10-08, offline): `peer-keys --show` now
reports fixed local invalid-reply diagnoses and binds numeric refusals to one
unambiguous selected-user/root context. Foreign first-error codes and duplicate
rows cannot masquerade as selected-peer refusals. Original action history and
retry reservations remain intact. APK 37.71 DF6 identity-only error handling
supports contextual attribution; duplicate guards are local policy and fresh
live refusal validation remains unverified.
[Evidence and limits](peer-identity.md#directory-refusal-attribution-2026-10-08).

Identity-success wait consumer (2026-10-09, **confirmed static APK 37.71**):
the sole DEX Fco constructor caller is a feature-gated extensions/Flows path.
Its 30-second wait wakes callbacks after success or Exception timeout; those
callbacks reverify held keys. This does not establish incoming replay/session
repair. Eight new reproducible identity-wait topics preserve the caller, observer,
timed worker and verification chain; all other event consumers and complete
identity-only application remain unfinished. No account or network changed.
[Evidence and scope](cross-device-sync.md#identity-wait-caller-and-extensions-verification-2026-10-09).

Identity-success consumers (2026-10-09, **confirmed static APK 37.71**): the
full DEX scan preserves 22 BdO/BdR declarations/bodies. Only Fco and AL2 have
nonempty BdO; the second waiter is a 30-second business-profile UI task.
No scanned callback directly decrypts or retries messages; transitive helpers
remain unknown. Primary LID receive's sender_pn prerequisite is a local guard,
not a proven APK requirement, and remains the next receive-routing trace.
No account or network changed. [Evidence and limits](cross-device-sync.md#identity-success-consumer-inventory-2026-10-09).

Primary LID routing trace (2026-10-09, **confirmed static APK 37.71**): missing
sender_pn/username logs join nullable metadata construction rather than throwing.
The parsed username attribute is username; DFc's 13 field bindings are recovered.
Ordinary LID's type 18 selects the message's A05 route for sender-device lookup,
without consulting alias metadata. Rust's unconditional primary alias guard is
a local limitation; historical PN-marker stability and the raw stanza producer
remain next boundaries before a receive change. Six unrelated/special builder
write paths are recorded to avoid repeating them. No account or network changed.
[Evidence and limits](peer-identity.md#primary-lid-route-and-nullable-aliases-2026-10-09).

Primary LID receive (2026-10-09, implemented/offline): new primary messages use
actual LID without requiring a phone alias. Original authenticated capture proofs
retain historical PN rows for duplicate/event/read lookup; no row rename, ratchet
replay or alias adoption from redelivery. Real-crypto msg/pkmsg, reopen/MAC failure,
legacy capture ambiguity and poll-target fixtures cover the correction. Fresh
live alias-less interoperability and full APK storage parity remain unverified.
[Evidence and scope](peer-identity.md#primary-lid-receive-and-legacy-record-stability-2026-10-09).

- Primary LID legacy deduplication also retains consumed view-once markers using
  original route proof; alias-less redelivery cannot recreate plaintext or an event.
- Bounded receive smoke check authenticated and stopped cleanly after 45 seconds
  with reconnect/chat sends disabled; 15 decrypted records remained 15, with no
  fresh DM or alias-less live proof. Original action/receipt/outbox/event summaries
  were unchanged. The later consumed-marker fix has offline validation only.

Reaction receive correction (2026-10-09, implemented/offline): APK 37.71 C7G
requires target ID plus parseable remoteJid; malformed keys now remain opaque
with redacted key_validation diagnostics. Primary direct reactions retain proven
historical PN target keys while the reactor keeps its actual LID; consumed
markers and original records are preserved. Status likes now require the actual
status@broadcast singleton. Own/outgoing/group/companion reconciliation and fresh
live reaction validation remain incomplete. A 21-topic/26-method checksum bundle
preserves the trace and cached diff (0.028679 seconds).
[Evidence, negative findings and limits](reactions.md).
Validation: 727 workspace/all-feature tests, strict Clippy, warning-free Rustdoc,
24 default CLI checks and CLI/API reference checks passed. Retained-account
offline reprojection upgraded zero events; action/receipt/outbox/event and
peer-key summaries stayed unchanged, with two valid saved reaction keys. Capture
inspection now uses one timestamp snapshot and handles overflowing ages; a
subprocess regression proves malformed-key diagnostics remain visible. No new
chat/live request or identity/reset/retry authority was added.

Outgoing reaction attribution (2026-10-09, implemented/offline): a unique normal-
outbox wire reservation plus exact address or existing authenticated PN/LID pair
retains the original sent-message key. Reactor identity remains actual; attempts,
facts, progress and uncertainty are unchanged. Lookup now checks all pages and
uncommitted rows for duplicate wire IDs. Outgoing quote bodies require the actual
saved intent's chat, rather than just the caller's reference. Unknown targets,
legacy lab-only sends, full native alias/companion/group reconciliation and fresh
live reactions remain unfinished.
[Sources, negative findings and fixtures](reactions.md#outgoing-intent-attribution-and-quote-scope-2026-10-09).
Validation: 730 workspace/all-feature tests, strict Clippy, warning-free Rustdoc,
24 default CLI checks and regenerated CLI/API reference checks passed. Offline
reprojection upgraded zero events and kept retained summaries unchanged. A
45-second receive-only live smoke check authenticated and stopped cleanly, with
15 decrypted records remaining 15; no fresh reaction/DM established new target
interoperability. Original uncertain actions, outbox and receipts were preserved.

### 2026-10-09: poll parent-key validation and mapper

**Confirmed**, APK 2.26.37.71 Ban.A01 -> C7G.A00 and Clc.A02 -> BoQ.A00;
BWH/BSp/BZa descriptors; six exact parent-key field references in Ban.A01/AD7.
Seven selected source topics/methods retain operands/exceptions in a checksummed
69,090-byte bundle. Rust checks common supported structure and CLI reports
fixed redacted refusals independently of decoding. Native JID factories,
effective decryption property, AD7 writer, transitive parent/expiry/edit/quiz
policy remain unknown/unported. Own poll secret chat binding is a confirmed
local implementation gap and remains next work. No fresh live poll test.
[Detailed evidence and boundaries](poll-votes.md).

### 2026-10-09: own-poll chat provenance in projection/preparation

**Confirmed, implementation**, existing normal reservation chat now binds own
poll secret lookup and outgoing vote preparation. Primary PN/LID association
uses only exact components or saved authenticated mapping, shared with reaction
attribution. Actual AEAD fixtures verify original target/voter identity and
durable dedupe; missing/foreign/alias-only/companion and corrupt-state boundaries
remain explicit. Read-only binding inspection is exposed by lab-inbox.
APK context remains the prior 37.71 Ban/Clc/BoQ source bundle, not a newly
traced native parent store; full native parity and fresh live poll coverage are
unproven. [Details](poll-votes.md#saved-chat-attribution-2026-10-09).

**Observed**, bounded receive-only 45-second login/active/clean-stop; retained
peer/action/receipt/outbox/event summaries unchanged, fifteen decrypted records
remained fifteen. No fresh poll/vote attribution, replay or identity adoption.
All 734 workspace tests and strict/local feature checks passed.

### 2026-10-09: identity receipt selection, primary version and token refresh

**Confirmed**, APK 2.26.37.71 D4x/0Z5/8G8 row predicates, DNd case-9 cache
operation, 0Z6/0ZE concrete columns, 0Z4 scoped companion/status/coexistence
cleanup, 0Z7 persistent per-user version update and HxT conditional existing-token
refresh. The 23-topic/53-declaration checksummed source bundle is distinct from
the older sender-key/key-removal scope. Runtime counterpart/migration state,
0Z8/A06 cache internals, callback closure and privacy job lifecycle remain unknown.
**Confirmed local gap:** receipts::observe still uses full Jid equality for
normal-outbox fact attribution; the actual incoming receipt authority path must
be traced before fixing PN/LID namespace matching. No live mutation, deletion,
replay or native-parity claim. [Details](identity-receipts.md).

Direct receipt attribution (2026-10-09, **confirmed static / local correction**):
APK 37.71 `CmG`/`Cmk`/`DDH` keep message chat and receipt actor separate;
stale `ClU` attribution is corrected. Primary normal-outbox receipts now bind
the unique saved wire intent to the actual actor through exact PN/LID or an
existing authenticated association in either direction, within one transaction.
Actual actor, original IDs, progress, uncertainty and receipt-send reservations
are preserved. Legacy PN records page history; ambiguous/corrupt matches roll
back. Native canonicalizer gates, identity cleanup and fresh live PN/LID
receipt attribution remain unverified. [Evidence and boundaries](direct-receipts.md).

Receipt correction validation: 739 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and repository hygiene
pass. The 12-topic APK bundle reproduces exactly; cached self-diff is 0.017892
seconds. A 45-second receive-only session authenticated and stopped cleanly;
no fresh user message or qualifying receipt arrived. Saved peer keys, actions,
receipt ACKs, outbox and events remained unchanged. No chat was sent, no
uncertain action replayed. Fresh live namespace attribution remains pending.

Receipt canonicalizer (2026-10-09, **confirmed static / local correction**):
APK 37.71 `0Yo.A07` selects PN/LID by global migration state and exempts
PNH_CTWA chats; `0Xw.A0I` has explicit subtype filters. Bot conversion uses
a separate gated static dictionary. Effective state and complete application
remain unknown/unported. `0hs.A0C/A0M` defaults only for absent attributes;
Rust now refuses present nontext/empty/unknown receipt types without false
delivery mutation, covered through actual wire encode/decode and store reopen.
No account identity, map or send state is reset. [Evidence and limits](receipt-canonicalizer.md).

Canonicalizer validation: 740 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, CLI/API/reference checks and hygiene pass. The 24-method
source bundle reproduces exactly and cached self-diff takes 0.007559 seconds.
Offline projection upgraded zero events; saved keys, actions, receipt ACKs,
outbox, events and retained inspection stayed identical. No new live request
or saved-account mutation; malformed types and native runtime gates remain
unverified live.

Group participant receipts (2026-10-09, **confirmed static / local implementation**):
APK 37.71 `Cmk`/`DDH`/`ReceiptProcessingJob`/`Cmn` retain separate message key,
group chat and participant device through deferred receipt processing. Rust now
tracks ordinary group delivery/read/played against the original addressed-device
set, retaining actual PN/LID actors and companion indices. Frozen targets are a
local guard; historical out-of-band targets remain unknown. Comparison sends
preserve new targets before writing; `group receipts --request-id` inspects only
redacted counts offline. Normal group send preparation, native orphaned receipt
application/aggregation and fresh live participant attribution remain unfinished.
[Evidence and boundaries](group-receipts.md).

Group receipt validation: 749 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and formatting pass.
The 21-method / 15-file / 124,482-byte source bundle reproduces exactly; cached
self-comparison takes 0.019472 seconds. Offline inspection found seven saved
group reservations across twelve request files, with zero tracked participant
receipt devices. Projection upgraded zero events; saved peer keys, action
outcomes, receipt ACKs, outbox, events and normalized inspection were unchanged.
No historical receipt capture was replayed or new group send prepared.

**Observed live regression:** a 45-second receive-only session authenticated,
activated and stopped cleanly with chat sending and reconnects disabled.
Peer keys, action outcomes, receipt ACKs, empty outbox, events and retained
noncapture inspection stayed unchanged. Transport capture rows increased from
442 to 446; these are not four user messages. All seven saved group summaries
stayed unchanged with zero tracked participant receipt devices. No fresh group
receipt established live attribution, and no uncertain send was replayed.

Shared group fanout (2026-10-09, **confirmed local correction**): the comparison
CLI now uses library Snapshot/Prepared staging and atomic crypto reservation.
Fresh mode retains the original ratchet for concurrency checks instead of
allowing a late same-identity advance to be overwritten. Pins remain enforced;
identity-triggered own-key rotation archives the original generation and leaves
uncertain sends intact. Existing fanout-inspect adds nullable missing-bundle and
cached-session counts offline. Native query batching and normal group outbox
integration remain unfinished; no new group/status send is claimed.
[Evidence and port boundaries](group-fanout.md).

Shared fanout validation: 753 workspace/all-feature tests passed, plus the final
CLI inspection regression, strict Clippy, warning-free Rustdoc, default CLI,
API symbol/reference, formatting and hygiene checks (1,117 local links; 15 APK
bundles). Offline projection upgraded zero events. Saved peer keys, action
outcomes, receipt ACKs, outbox, events and normalized retained inspection were
unchanged. No server connection, group/status send, request replay or saved
account crypto mutation occurred in this chunk. Shared staging/rotation is
verified offline; live group/status validation and normal actor wiring remain.

Directory-key batches (2026-10-09, **confirmed static / local implementation**):
APK 37.71 `0y0.A00` confirms batched encrypt/key/user construction and optional
identity-reason/PQ/context branches; effective native producer flags and limits
remain unknown. Group/status lookup now covers all frozen bundle-required
devices in local 32-device pages, durably reserved before write, with complete
signed responses reused only for the same plan. Unknown/refused/invalid/legacy
unattributed outcomes block without resets. New group key-queries inspection
prints redacted progress offline. Normal group actor integration, optional native
fields, own inventory and fresh live large-group coverage remain unfinished.
[Evidence and boundaries](device-key-batches.md).

Directory-batch validation: 758 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI/API/reference checks and formatting pass.
Offline projection upgraded zero events; saved peer keys, action outcomes,
receipt ACKs, outbox, events and normalized retained inspection stayed unchanged.
Twelve existing request files were inspected offline: no new batch records,
eleven with neither batch nor historical key outcome, one with a preserved
legacy outcome and unknown frozen counts. No server request or message was sent.

The existing groups APK recipe now anchors get-pre-key-batch and maps its writer
to messaging.rs and device_key_batches.rs for future release diffs. Exact
re-export changed only targets/checksums metadata, not native method bodies;
all 44 files / 928,943 bytes / 43 selected methods reproduce. Cached self-diff
is 0.047877 seconds, not a claim of cross-release equivalence. Large-group live
lookup, native optional-field gates and normal group actor wiring remain pending.

Group routing snapshot (2026-10-09, **confirmed local correction**): planning
now reads membership/lists/primary authority together; shared fanout revalidates
the supplied plan and refuses changed routing rows inside the original send
transaction. Fresh/cached crypto and caller reservation roll back together,
with newer rows preserved. The native TEE node-token equality is not evidence
of a membership transaction. Normal group outbox actor wiring, own inventory
and fresh live group fanout remain unfinished.
[Evidence and limits](group-fanout.md#routing-snapshot-correction-2026-10-09).

Routing-guard validation: 760 workspace/all-feature tests passed, alongside
strict Clippy, warning-free Rustdoc, default CLI, API symbols/CLI reference,
formatting and repository hygiene (1,129 local links; 15 APK bundles). Offline
projection upgraded zero events. Saved peer keys, action outcomes, receipt ACKs,
outbox, events and normalized retained inspection stayed unchanged. No server
request, message transmission or saved-account crypto mutation occurred.

Normal group outbox (2026-10-09, **confirmed local implementation**): the actor
now resolves fresh membership, selected message-context device pages and all
missing signed key pages, then reserves shared fanout crypto and the original
normal intent together before write. Existing private JSON group/text/image/voice
uses outbox-send; group uploads bind the chat. Redacted outbox-group-preparations
shows original phase, fixed blockers, counts and scoped batch outcomes offline.
Cold PN/LID real-crypto tests, 130-user batches and injected four-stage write cuts
cover preparation and once-only reservations; simulated late ACK/participant
delivery settles without another attempt. Normal group live acceptance, broader
message kinds, answered-preparation resume, group peer replacement and full own
inventory remain unfinished.
[Evidence, invocation and boundaries](normal-group-outbox.md).

Validation for this chunk: 767 workspace/all-feature tests and strict Clippy,
Rustdoc, default CLI/API/reference/format checks passed. The upgraded saved
account authenticated and stopped cleanly after a 45-second receive-only window,
with chat sends and reconnect disabled. No live outgoing group was tested.

Signed group peer replacement (2026-10-09, **confirmed local implementation**):
normal group fanout now honors the explicit outgoing APK-authenticated policy for
verified signed remote bundles, archives old pins/ratchets and emits identity
events in the original send transaction, and rotates an exposed current chain
while leaving other exposed groups pending. Own identities remain pinned;
changed-primary fanout with old companion authority reports
companion_authority_changed. Interrupted/blocked preparations are not reset or
replayed. Cached-session refresh, companion reconciliation, preparation resume
and live group identity transition remain unfinished.

Signed group policy validation: 770 workspace/all-feature tests and strict
Clippy, Rustdoc, default CLI/API/reference/format and hygiene checks passed.
A 45-second receive-only check authenticated and stopped cleanly; peer keys,
actions, receipts, outbox and events stayed unchanged. Five transport captures
were added. No live outgoing group identity transition was tested.

Typed group quotes/reactions (2026-10-09, **confirmed local implementation**):
reply/react now accept an original --participant for incoming group targets and
--ours derives frozen own identity. Normal fanout carries the existing APK
quote/reaction encodings, checks retained parent attribution, and blocks late
reference changes without a wire attempt. Existing default IDs/queued records
are preserved. Private JSON quote, media quotes, incoming group/id collisions
and fresh live group rendering remain unfinished.

Typed group reference validation: 773 workspace/all-feature tests and strict
Clippy, Rustdoc, default CLI/API/reference/format and hygiene checks passed.
A 45-second receive-only check authenticated and stopped cleanly with sends and
reconnect disabled; peer keys, actions, receipts, outbox and events stayed
unchanged. Five transport captures were added. Group quote/reaction handset
rendering and the participant-aware incoming storage port remain unverified.

Participant-scoped group inbox (2026-10-09, **confirmed local implementation**):
new records distinguish original participant namespace/agent and wire ID, retain
original device routes, and preserve legacy records/event markers. Quotes use the
selected participant's body. `mark-read --participant` selects a group member;
ambiguous unqualified group/ID reads refuse atomically. Automatic reads remain
opt-in and refuse unsupported agent selection. Native SQLite-key parity and live
same-ID group collision behavior remain unknown. See [group inbox](group-inbox.md).

Group inbox validation: 775 workspace/all-feature tests passed, together with
strict Clippy, Rustdoc, default CLI, library-symbol, regenerated CLI reference
and formatting checks. Repository hygiene passed (15 APK research bundles,
5,815,284 bytes). A 45-second receive-only check authenticated and stopped
cleanly with sends/reconnect disabled; peer keys, actions, receipts, outbox and
events were unchanged. Original capture shapes were preserved; captures grew
from 461 to 466. No fresh group send, explicit read request or live same-ID
collision was performed. The prior incoming storage gap is now corrected
locally; native storage parity and fresh group handset rendering remain unknown.

Incoming group reaction authors (2026-10-09, **confirmed local correction**):
BZa participant now selects the original author; a third member's message no
longer becomes an outgoing target merely because fromMe is false. Matching
legacy/new parent routes retain actual devices; consumed bodies stay consumed.
Known own PN/LID selects outgoing targets; unsupported agent/malformed/foreign
group keys remain opaque. Inspection adds fixed target_validation labels. Parent
ambiguities refuse atomically; old event markers are preserved. Fresh live group
reactions and native own/hosted normalization remain unknown. See [reactions](reactions.md#incoming-group-original-author-2026-10-09).

Group reaction validation: 778 workspace/all-feature tests and strict Clippy,
Rustdoc, default CLI, API/library symbols, CLI-reference regeneration and format
checks passed. Hygiene checked the unchanged 15 APK evidence checksum bundles
(5,815,284 bytes). Two retained group keys report target_validation=valid; no
historical events were rewritten. A 45-second receive-only session authenticated
and stopped cleanly with sends/reconnect disabled. Peer keys, actions, receipts,
outbox and events stayed unchanged; original capture shapes were retained and
captures grew from 466 to 471. Fresh live group reaction targeting is unverified.
BoQ.A00's BO0 predicate, effective caller gates and contradictory fromMe/author
normalization remain unported; exact known own PN/LID matching is a local
projection contract. One temporary-store ownership test refusal passed isolated
and on full rerun without changing ownership guards; its cause remains unknown.

Native group reaction predicate/branch (2026-10-09, **confirmed static/local**):
the new 18-topic APK 37.71 bundle closes BO0's own PN/LID comparison and
user/server/type equality. Ordinary groups consult it only for false/default
fromMe; true/present-author stays incoming, correcting our prior unconditional
own classification. Nonzero device authors no longer fold into UserJid equality.
Nullable-author fallback, actor producer authority, zero-device text and
authenticated own-envelope context remain unported. No identity/alias adoption
or historical-marker rewrite is introduced. See [group mapper](group-reaction-mapper.md).

Ordinary remote-key correction (**confirmed APK 37.71 BoQ.A00**, `00fe..0106`):
plain PN/LID/group remote keys may differ from the envelope group. The update
and parent selection now remain in that authenticated envelope group. The
previous exact-match refusal was our local limitation; status/device/unported
remote kinds stay separate. Native nullable-author and own-envelope source
context remain unported; no fresh live group-targeting result is claimed.

**Observed receive validation (2026-10-09):** two bounded 45-second windows
authenticated and stopped cleanly; the second ran the final ordinary-remote
correction. Both disabled chat sends and automatic reconnect. Original captured
records were retained; capture rows grew from 471 to 481. Peer keys, actions,
receipts, outbox and durable events were unchanged. Two retained group reaction
keys still inspected as target_validation=valid. These are retained-key and
receive-runtime checks, not fresh live group reaction/own-envelope proof.

Native mapper verification: 778 workspace/all-feature tests, strict Clippy,
Rustdoc, default CLI, API/library-symbol checks, CLI-reference regeneration and
formatting passed. Final API edits passed five OpenAPI tests and rebuilt the CLI.
The 18-topic APK export/check reproduces 23 files / 86,053 bytes; hygiene validates
16 checksum bundles / 5,901,337 bytes. Cached verified self-diff took 0.008 seconds;
new-release extraction and re-anchoring speed remains unmeasured.

Group envelope direction (2026-10-09, **confirmed static/local**): the APK 37.71
CnN/CgC/Cih/Bp2/Cna producer chain now establishes the ordinary network group's
selected sender and own-envelope flag. Sender-aware reactions preserve payload
direction for a known own sender; remote true/absent-or-empty-author references
fall back to the authenticated sender user. Inspection verifies the saved original
route against its selector. Existing projection markers, consumed bodies, parent
routes, outbox records and trust admission remain preserved. This supersedes the
ordinary own-envelope/source-producer gap above; full own-device synchronization,
hosted/status references, special chat normalization and fresh live own-device
reaction behavior remain unverified/unported. See [envelope direction](group-envelope-own.md).

**Confirmed verification:** 779 workspace/all-feature tests passed (652 API,
4 property, 9 crash, 50 CLI, 1 API-symbol and 63 xtask tests), along with strict
all-target/all-feature Clippy, Rustdoc with warnings denied, default CLI tests,
API/library checks, CLI-reference regeneration and formatting. The final embedded
API/build check also passed. The 24-topic export and revalidation agree on
29 files / 127,370 bytes. Hygiene validates 17 APK checksum bundles totaling
6,028,707 bytes.

**Observed receive check (2026-10-09):** one 45-second window authenticated and
stopped cleanly with chat sends and automatic reconnect disabled. Original
captures were retained; capture rows grew from 481 to 487. Peer keys, actions,
receipts, outbox and durable events stayed unchanged. Both retained group keys
still inspected as target_validation=valid using their original sender routes.
No historical markers were repaired or removed. This checks the receive runtime
and retained references; it does not establish fresh live own-device/group
reaction handling or human handset rendering.

Group keep/unkeep receive attribution (2026-10-09, **confirmed static/local**):
APK 37.71 D9L.A01 maps BV3.key through BoQ, just like ordinary group reactions.
Reception now resolves the original author independently of the keeper, uses
authenticated own-envelope context and retains matching parent devices. Incoming
updates preserve consumed bodies, outbox facts and existing event markers.
`lab-inbox --inspect` exposes fixed group keep target_validation labels using
original saved routes. Direct keep projection retains its existing behavior.
Normal group keep sending, native downstream authorization/expiry enforcement,
and fresh live group keep/handset rendering remain unimplemented or unverified.
See [group keep](group-keep.md).

**Confirmed verification:** 780 workspace/all-feature tests passed (653 API,
4 property, 9 crash, 50 CLI, 1 API-symbol and 63 xtask tests), together with
strict all-target/all-feature Clippy, Rustdoc with warnings denied, default CLI,
API/library-symbol checks, CLI-reference regeneration and formatting. Export
and checksum revalidation reproduce six topics / 11 files / 84,246 bytes.
Repository hygiene validates 18 APK bundles totaling 6,112,953 bytes.

**Observed receive validation (2026-10-09):** one bounded 45-second window
authenticated and stopped cleanly with chat sends and automatic reconnect
disabled. Original captures were retained; capture rows grew from 487 to 492.
Peer keys, actions, receipts, outbox and durable events stayed unchanged. Two
retained group reaction targets remain valid; no retained group keep payload
was available to validate live attribution. No fresh group keep/unkeep, group
send or human handset rendering is claimed. Native keep timestamp ordering,
authorization, expiry enforcement and outgoing group keep remain open gaps.

Outgoing keep key research (2026-10-09, **confirmed static**): APK 37.71
Bai.AD7 serializes the original parent key through CSZ.A01. The new 22-topic
bundle traces explicit id/fromMe/participant setters, gated parent/edit/partial-ID
lookup and retained author-device selection. A supplied author may be omitted
when the gated lookup misses; remote namespace mapping and effective gate values
remain unknown. No Rust send behavior changed. Normal group keep sending,
native permission/expiry checks and fresh live keep/unkeep remain unfinished.
[Evidence and implementation boundaries](group-keep-send-key.md).

**Confirmed research validation:** exact re-export/check reproduces 22 topics /
27 files / 75,556 bytes; checksum-verified self-diff is unchanged. All 63 xtask,
50 experimental CLI and one API-library symbol tests passed, including CLI
reference regeneration. Hygiene validates 19 APK bundles / 6,188,509 bytes.
No account connection, group draft, message send or private state change was
performed. This verifies the research export, not outgoing keep implementation.

Normal group keep/unkeep (2026-10-09, **confirmed static/local implementation**):
APK 37.71 original-key, namespace, creation-clock and stanza traces now ground
a separate group codec and typed keep --participant/--ours/--undo/--dry-run.
Normal fanout dispatch preserves the original accepted timestamp and original
intent reservations. Shared quote/reaction/keep parent selection now reads one
snapshot and revalidates inside the crypto/outbox reservation transaction;
changed references roll back with reference_changed. Consumed bodies stay
consumed and keeps/reactions do not parse parent text previews. Native
permission/expiry enforcement, adjusted-clock synchronization, complete parent
normalization, full own-device sync and fresh group ACK/rendering remain open.
[Evidence and limits](group-keep-outbox.md).

**Confirmed validation:** 784 workspace/all-feature tests (656 API, 4 property,
9 crash, 51 CLI, one API-library symbol, 63 xtask), strict Clippy, warning-free
Rustdoc, default CLI/API checks, regenerated CLI reference, formatting and build
passed. APK re-export/check reproduces 33 topics / 38 files / 259,783 bytes;
cached checksum-verified self-diff is unchanged in 0.017053 seconds, not a
new-release extraction benchmark. Hygiene checks 1,237 local links and 20 APK
bundles / 6,448,292 bytes. Offline keep dry-runs enqueue nothing; missing group
participant refuses. Saved account facts and 492 retained captures were unchanged
before live validation.

**Observed receive validation:** a single completed 45-second window
authenticated and stopped cleanly with sends and automatic reconnect disabled.
Peer keys, actions, receipts, outbox, events and retained original capture shapes
stayed unchanged; captures grew from 492 to 497. Two retained group reaction
references remain valid; no group keep payload was available. No new group draft,
keep/unkeep transmission, group ACK or handset rendering was tested. An earlier
local ownership refusal resulted from overlapping offline inspection and receive
orchestration; its logs were preserved and inspection completion verified before
this window. No server retry, account reset or ownership bypass was added.

2026-10-09 — **Confirmed local/static:** incoming keep events now preserve
optional payload milliseconds and their own envelope ID, independently of the
original target and stanza time. Literal legacy-journal records remain readable
and retain their shape; reopening preserves group update metadata without
republication. APK 37.71 native ordering prefers newer envelope time, then newer
payload time, then the smaller action ID. Permission/expiry enforcement and
fresh live keep reception remain unverified. [Evidence and limits](keep-timestamps.md).

Keep metadata verification: 786 workspace/all-feature tests, strict Clippy,
Rustdoc, default CLI, API-symbol checks, CLI reference and formatting passed.
One 45-second receive-only window authenticated and stopped cleanly with sends
and reconnect disabled. Captures grew 497→502 with originals retained; keys,
actions, receipts, outbox and event summaries were unchanged. No group keep
payload was available, so fresh keep semantics remain unverified.

2026-10-09 — **Confirmed static:** identity-change token refresh now has traced
outbound issue/send storage, bucket expiry formula, request destination/time,
callbacks and explicit completion/cancellation/retry branches. Incoming token
bytes are not its outbound state. Native 4xx IQ refusal suppresses job retry;
base terminal dispatch, active properties and canonical counterpart gates remain
unknown. Rust refresh sending/storage is still unported; account state was
untouched. [Evidence and port boundary](identity-token-refresh.md).

Token-refresh research validation: 26 topics / 31 files / 104,087 bytes reproduce
from the comparison APK; checksum/self-diff, 63 xtask tests, API symbols, CLI
reference, formatting and repository hygiene passed. Source changes are a
comment only. No live refresh or identity replacement was attempted.

2026-10-09 — **Confirmed static/local:** traced the normal server-acceptance
producer of outbound token scheduled timestamps, its raw message exclusions,
namespace transfer and separate call producers. Scheduled timestamp state is
not proof of token server acceptance. New apk-smali --signatures and recipe
include_method_signatures expose complete DEX descriptors, distinguishing two
DNf constructors with identical shorties while preserving legacy defaults.
Token scheduling/refresh runtime remains unported; account state was untouched.
[Evidence, invocation and boundaries](token-issue.md).

Token-issue/full-signature validation: 788 workspace/all-feature tests, strict
Clippy, Rustdoc, default CLI/API/reference/format and build checks passed. New
14-topic evidence and the prior default-mode bundle reproduce; self-diff took
0.022072 seconds. No account operation or live token/identity request was made.

2026-10-09 — **Confirmed local:** manual peer verification now refuses a saved
own LID with no user before writing metadata. A failing-before regression and
reopen checks cover mark/unmark refusals, unchanged pins/sessions/events and
valid remote decisions. The initial zero-device bypass hypothesis was refuted:
valid textual zero forms already normalize; no parser change was needed.
This is our local prerequisite, not a server refusal. Native malformed-login
parity and fresh live identity transitions remain unknown. No real account
verification decision or network request was made.
[Evidence and reproduction](own-verification.md).

Own-prerequisite verification: 790 workspace/all-feature tests (660 API, four
property, nine crash, 51 CLI, one API symbol and 65 xtask), strict Clippy,
warning-free Rustdoc, default CLI/API checks, regenerated CLI reference,
formatting and build passed. Hygiene checked 1,283 local links and 23 APK
bundles / 6,988,623 bytes. Validation used synthetic stores only.

2026-10-09 — **Confirmed static:** the two remaining outbound token timestamp
writers restore imported chat-transfer/companion-history sender state, separately
from received token bytes. The companion key-parse failure can still reach token
restoration; no valid-key admission shortcut is inferred. The hidden descriptor
id-array helper explains the field decoder's honest 59/58 refusal. Manual tag and
presence mapping is preserved with the helper evidence. History source authority,
active gates and Rust import/refresh remain unported. Account state was untouched.
[Evidence and limits](token-import.md).

Token-import research validation: seven topics / 12 files / 412,652 bytes
reproduce from the APK. Cached self-diff reports seven topics and four metadata
artifacts unchanged in 0.021659 seconds; this is not a cross-release benchmark.
All 65 xtask tests, CLI-reference regeneration, API/library-symbol check,
formatting and diff hygiene passed. Repository hygiene checked 1,287 local links
and 24 APK bundles / 7,401,275 bytes. Runtime source changes are comments only;
no account operation or live token/history request was made.

2026-10-09 — **Confirmed local/static:** authenticated primary replacements
now persist the APK-style per-user device version atomically with their archive
and follow-ups, exposed in PeerPinState through existing offline inspection.
Missing old state stays null; first tracked change starts at 1. PN/LID namespaces
stay separate, own/companion exclusions remain, and receipt reservations are
preserved. Native receipt version use and fresh live increments remain unverified.
[Evidence and limits](primary-device-version.md).

Primary-version verification: 792 workspace/all-feature tests (662 API, four
property, nine crash, 51 CLI, one API symbol and 65 xtask), strict Clippy,
warning-free Rustdoc, default CLI/API checks, regenerated CLI reference,
formatting and build passed. Hygiene checked 1,296 local links and 24 APK
bundles / 7,401,275 bytes. Real encrypted direct/group transition fixtures
exercise the receive integration; isolated tests cover both archive sources,
rollback, reopening, namespace separation, signed-long overflow and legacy JSON.

**Observed compatibility:** one completed 45-second receive-only window
authenticated and stopped cleanly, with chat sends and automatic reconnect
disabled. Saved peer facts (apart from the new absent-version null field),
actions, receipts, outbox and event summaries were unchanged. Captures grew
502→508 with all original shapes retained; two retained group reaction targets
remain valid. No fresh identity transition occurred, so live version increments
remain unverified. The original 23-topic / 28-file / 323,094-byte APK receipt
bundle reproduced byte-for-byte.

**Observed inspection clarification:** direct `peer-identity` checks with the
previous and rebuilt binaries match after omitting only the new null counter.
Two inspected primary namespaces have absent historical versions. `peer-keys
--show` exposes directory match diagnostics, not PeerPinState; its original
compatibility comparison remains valid but cannot prove counter output. The API
and invocation description were corrected to name `peer-identity` only.

2026-10-09 — **Confirmed static:** native pending device-receipt rows capture
nullable send-time primary versions; existing-row timestamp updates retain them.
The ordinary sync/resend path compares prior/current values with nullable equality,
plus a gated absent-to-1 exception. The raw hosted branch has additional gates;
0x6ebd does not surround all ordinary comparisons. Historical Rust sends lack
this snapshot and remain unknown. Rust resend/snapshot runtime remains unported;
no saved reservation or account was changed. [Evidence and next input](receipt-version.md).

Receipt-version research validation: 14 topics / 19 files / 135,279 bytes
reproduce exactly from the comparison APK. Cached self-diff reports all topics
and four metadata artifacts unchanged in 0.022983 seconds, not cross-release
equivalence. All 65 xtask tests, CLI-reference regeneration, API/library-symbol
check, formatting and diff hygiene passed. Repository hygiene checked 1,304
local links and 25 APK bundles / 7,536,554 bytes. Runtime changes are comments
only; no account operation, live resend or receipt reservation change occurred.

2026-10-09 — **Confirmed local/static:** normal direct/group original-send
reservations now capture nullable per-user primary versions after authenticated
replacement hooks, atomically with their original state. All group plan devices
are included. Offline outbox-versions exposes hashes/counts and prior/current
values; absent historical snapshots remain unknown. Missing JSON version members
and malformed state refuse without private echoes. Native resend/receipt-cache
runtime remains unported; no new live send was authorized or performed.
[Evidence and limits](send-version-snapshot.md).

Send-version validation: all 795 workspace/all-feature tests, strict Clippy,
warning-denying Rustdoc, 26 default CLI checks, API/library-symbol check,
CLI-reference regeneration, formatting and experimental CLI build passed.
Hygiene checked 1,313 local links and 25 APK bundles / 7,536,554 bytes.
Offline retained-profile CLI checks left store inspection and queued intents
unchanged; invalid IDs and unknown intents refused without input echoes.
There were no retained group preparations to inspect; historical missing-state
coverage is from fixtures. New send capture has no live validation in this chunk.

2026-10-09 — **Confirmed static/local audit:** native sync/resend now has a
22-topic trace of its distinct encrypt/key_fetch request, existing remote
registration encoding, reply/worker completion, original receipt-device
subtraction and post-version target mutation. An empty/errored-device result
can complete true; callback completion is not per-device usability. Expiry
is strictly past, not equal-time rejection. Rust lacks key_fetch and original
per-device receipt membership; user counts cannot reproduce removeAll.
Current nullable user snapshots are preserved; no live worker/resend, account
operation or retry-guard relaxation was added. [Evidence and next port inputs](sync-resend.md).

Sync/resend research validation: all 22 source topics / 27 files / 272,318
bytes reproduced from the exact comparison APK. Cached self-diff verified
22 unchanged topics and four unchanged metadata artifacts in 0.031055 seconds;
this is plumbing verification, not cross-release equivalence. All 65 xtask
tests, CLI-reference regeneration, API/library-symbol check and formatting
passed. Repository hygiene checked 1,324 local links and 26 APK bundles /
7,808,872 bytes. Runtime changes are comments only; no account operation,
key-fetch query or live resend was performed.

2026-10-09 — **Confirmed local/static:** original send snapshots now preserve
exact per-user device components; outbox-versions exposes only component/hash
rows. Historical count-only rows remain unknown and untouched. Group receipts
use the full original plan, validate envelope distributions as a subset and
refuse contradictions atomically. Current fanout sends SKDMs to every target;
omitted-distribution coverage is fixture/native compatibility, not a reproduced
current live failure. Native mutable receipt lifecycle/resend remains unfinished.
[Evidence and limits](send-version-snapshot.md).

Exact-device validation: all 798 workspace/all-feature tests passed (668 API,
4 property, 9 crash, 51 CLI, 1 API-symbol and 65 xtask), alongside strict
Clippy, warning-denying Rustdoc, 26 default CLI checks, CLI-reference/API checks,
formatting and experimental CLI build. Hygiene checked 1,329 local links and
26 APK bundles / 7,808,872 bytes. Native-shaped subset/contradiction fixtures
cover planned companions, unplanned devices, immutable send state, reopening
and legacy wire-only fallback. Old count-only snapshots remain readable;
malformed device IDs refuse with redacted diagnostics.
Offline retained-profile CLI checks left store inspection and queued intents
unchanged. No retained group preparations existed to inspect; no new live
message or native pending receipt cache is claimed from these fixtures.

2026-10-09 — **Confirmed static:** receipt application now has an eight-topic
trace through 14e/Cmn and the DDH device/time/status/key getters. Cmn.A00
selects CNn.A00 for a status predicate and separately 15A.A07 for other
statuses. CNn/CYV row policy and complete clocks/mapping remain untraced;
no mutable receipt ledger or runtime status promotion is claimed.
[Writer boundary and next inputs](receipt-apply.md).

Receipt-application validation: eight source topics / 13 files / 112,789 bytes
reproduced exactly from the pinned comparison APK. Cached self-diff verified
eight unchanged topics and four unchanged metadata artifacts. Repository
hygiene checked 1,336 local links and 27 APK bundles / 7,921,661 bytes; diff
hygiene passed. This follow-up changes research/docs only; the preceding
cdca7ad runtime chunk passed all 798 workspace tests and strict lint/doc/CLI
checks. No new runtime implementation or live receipt application is claimed.

2026-10-09 — **Confirmed static:** CNn is a receipt value factory, not a row
writer. Its settings branch downgrades played/read (8/13) to delivery (5);
1hg.A03 admits six statuses. The downstream 0Z4 dispatch separates message
classes and reaches 0Z5 timestamp storage. Complete 15A admission, clock and
namespace/cleanup policy remain unknown; no runtime or live operation changed.
[Evidence and reproducible recipe](receipt-factory.md).

Receipt-factory validation: seven source topics / 12 files / 137,766 bytes
reproduce exactly from APK 2.26.37.71. Cached self-diff reports seven unchanged
topics and four unchanged metadata artifacts; this is reproducibility, not
cross-release equivalence. Repository hygiene checked 1,340 local links and
28 APK bundles / 8,059,427 bytes; diff hygiene passed. Research/docs only; no
fresh Rust runtime test or live account operation is claimed for this chunk.

2026-10-09 — **Confirmed static:** receipt timestamp branches now distinguish
direct zero defaults, strict aggregated participant parsing and adjusted-clock
defaults for successful-message participant receipts. Wire seconds become
milliseconds. The cache retains earliest positive time without rewriting the
nullable primary version; ordinary concrete device-receipt columns and conditional
device-preserving namespace mapping are traced. Clock calibration, complete
message admission and mutable identity cleanup remain unknown/unported.
[Evidence and comparison recipe](receipt-clock.md); no runtime/account operation changed.

Receipt-clock validation: 18 source topics / 23 files / 82,330 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified 18 unchanged
topics and four unchanged metadata artifacts; this does not establish parity
between different releases. Repository hygiene checked 1,350 local links and
29 APK bundles / 8,141,757 bytes. Diff hygiene passed. Changes are research/docs
only; no new runtime tests, live receipt or account changes are claimed.

2026-10-09 — **Confirmed local/static:** signed const/4 annotations now show
negative decimal values; 0Z8's trimToSize input is -1 (clear), not 255.
Primary selected-row invalidation and companion broad cache clear are traced,
including optional map/callback handling. Incoming SQL fallback selects timestamps
without primary versions; general load/eviction and complete mutable receipt
runtime remain unported. [Evidence and recipe](receipt-cache.md).

Receipt-cache validation: all 66 xtask tests, strict xtask Clippy, rebuilt
research CLI, CLI-reference regeneration, OpenAPI/library-symbol check, formatting
and diff hygiene passed. Eight source topics / 13 files / 57,611 bytes reproduce
exactly with the corrected renderer; cached self-diff verifies eight unchanged
topics and four metadata artifacts. Hygiene checked 1,359 local links and
30 APK bundles / 8,199,368 bytes. No transport code changed; previous 798-test
runtime validation is not claimed as a fresh run. No live account operation ran.

2026-10-09 — **Confirmed static:** the general receipt loader retains device
row membership without a positive timestamp filter; it does not load primary
versions. Cache install rechecks rather than overwrites, and general row mapping
uses 0Yo.A03 rather than the incoming update's A07. Pending row membership,
delivery facts and immutable original targets remain distinct. Mutable runtime
receipt_user/status admission and alias authority are still unported.
[Evidence and next inputs](receipt-load.md); no account or live operation changed.

Receipt-load validation: seven source topics / 12 files / 58,000 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified seven unchanged
topics and four metadata artifacts; this does not prove cross-release parity.
Repository hygiene checked 1,368 local links and 31 APK bundles / 8,257,368
bytes; diff hygiene passed. Research/docs only; no fresh Rust or live tests
are claimed beyond the preceding 66-test tooling chunk.

2026-10-09 — **Confirmed static:** per-user receipt timestamps retain earliest
positive delivery/read/played independently. User rows can be created by read
or played without a delivery timestamp; identity cleanup's existence query
is not equivalent to a Delivered fact check. Writer rejects other statuses,
while complete upstream 15A admission and namespace authority remain unknown.
[Evidence and reproducible recipe](receipt-user.md); mutable Rust receipt runtime remains
unported, with no account or live operation in this chunk.

Receipt-user validation: eight source topics / 13 files / 128,135 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified eight unchanged
topics and four metadata artifacts, not cross-release equivalence. Hygiene
checked 1,381 local links and 32 APK bundles / 8,385,503 bytes; diff hygiene
passed. Research/docs only; no new runtime tests or live operation is claimed.

2026-10-09 — **Confirmed static:** 15A admission rejects nonpositive times
and splits message/user advancement from device timestamp handling. Native
nonadvancing comparison includes equal ranks; its 22-entry ordering was recovered
from the exact APK payload. Lower/equal events can still reach device storage.
State-only status translation, manager exclusions and conditional message
classification are traced with explicit remaining authority/lifecycle unknowns.
[Evidence and next port inputs](receipt-admission.md); no runtime/live changes in this chunk.

Receipt-admission validation: 13 source topics / 18 files / 121,518 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified 13 unchanged
topics and four metadata artifacts; raw 96-byte/22-entry status-order evidence
independently matches classes2.dex at its recorded offset. Repository hygiene
checked 1,393 local links and 33 APK bundles / 8,507,021 bytes; diff hygiene
passed. Research/docs only; no fresh runtime tests or live account operation
is claimed.

2026-10-09 — **Implemented/offline:** APK 37.71 0Z5.A0D/DPo/0Z6.A0L pending
nullable-version rows and Cmk.A03/0Gv.A01/7hf.A01 earliest-positive device times
now have bounded Rust storage/receipt/inspection paths. Duplicate fact admission
no longer suppresses an earlier supported root timestamp. Exact original-device
provenance and atomic rollback are required; historical rows are never backfilled.
[Implementation, sources and remaining unknowns](device-receipt-rows.md).
No new raw APK bundle or live account operation in this chunk.

Receipt-row runtime validation: 802 workspace/all-feature tests passed, plus
strict Clippy, warning-denying Rustdoc, formatting, default CLI/API symbols,
CLI-reference regeneration, experimental CLI build and hygiene (1,405 local
links, 33 bundles / 8,507,021 bytes). Retained-state offline inspection was
unchanged; zero network commands and zero retained group preparation samples.
A separate local owner-drop shared-descriptor regression confirms teardown
release without widening active-owner admission. Historical refusal holder
attribution remains unknown; see [verification](../design/verification.md#temporary-store-ownership-refusal-observed-2026-10-09).

2026-10-09 — **Confirmed static:** APK 37.71 13v constructor/A02 and 8UM44
close the A03 revoke exclusion producer; 15A's earlier device-only pool remains
separate. 0V9.A00 and Cjb.A02 establish deletion-path marking/copying of A0l,
not its full persistence/producer semantics. Raw 1gB.close decrement bytes at
classes2.dex 0x12a6da independently confirm signed -1; 22b/22s renderer output
is corrected with boundary tests and preserved predecessor generations.
[Evidence and unknowns](receipt-manager.md). No Rust revoke/cleanup or live claim.

Receipt-manager validation: 67 xtask tests and strict xtask Clippy passed,
plus rebuilt CLI, CLI reference/API symbols, formatting and hygiene. Eight source
topics / 13 files / 301,312 bytes reproduce exactly; self-diff verifies eight
unchanged topics/four artifacts. Independent raw DEX read matches signed decrement
at 0x12a6da. Hygiene: 1,416 links, 34 bundles / 8,808,333 bytes. Tooling/research
only; no new account/shared-runtime/live test is claimed.

2026-10-09 — **Confirmed static / implemented offline:** 08a delegates BO0/BO1
to 08i; BO0 compares stored A0I PN/A0H LID, BO1 extracts the device user first.
A01/A02 field producers and nullable self_lid initialization are retained. Rust
primary-change hooks now share authenticated login/retained own-LID lookup;
conflict refusal is explicit local policy, not a native/server requirement.
[Evidence, inspector and remaining unknowns](own-identity.md).

Own-LID validation: all 807 workspace/all-feature tests, strict Clippy,
warning-denying Rustdoc, default CLI, reference/API-symbol and formatting checks
passed. Exact bundle reproduction and cached self-diff passed. One 45-second
receive-only session authenticated and stopped cleanly, with sends/reconnects
disabled; peer/actions/receipts/outbox/events remained unchanged. Eight additional
inbound checkpoints remain unclassified; no live own-primary transition is
claimed. See the linked own-identity evidence for counts and limits.

2026-10-09 — **Confirmed static:** revoke receipt callbacks are now traced:
8SJ discriminator 6 removes the selected message-row user cache entry under a
monitor; DLn discriminator 6 delegates selected-row device-cache invalidation.
The user cache constructor uses plain 0D3; base entryRemoved is empty. Existing
transaction-close/rollback and optional device-cache callback evidence remains
separate. Mutable cleanup, revoke projection and resend are unported.
[Worker evidence and boundaries](receipt-callbacks.md).

Callback validation: the five-topic / 10-file / 70,998-byte bundle reproduces
exactly; cached self-diff verifies five unchanged topics and four metadata
artifacts, not cross-release protocol equivalence. CLI-reference regeneration,
OpenAPI/library-symbol checks, formatting and diff hygiene passed. Repository
hygiene checked 1,436 local links and 36 bundles / 8,929,026 bytes. This final
chunk changes static research/docs only: it does not claim a new workspace
runtime or live cleanup test. The earlier own-LID runtime validation is separate.
No retained account was read or changed during this callback research.

2026-10-09 — Companion receipt lifecycle implementation (offline verified):
authenticated changed companion keys now clear only pending exact-device mutable
membership for ordinary direct/group original intents. A later positive supported
receipt reinserts with current nullable user version; original at_send/targets
and held send/receipt reservations are preserved. Primary/counterpart/status/
coexistence cleanup and final resend remain unfinished.
[APK evidence, diagnostics and boundary](companion-receipt-cleanup.md).

Companion lifecycle validation: all 810 workspace/all-feature tests, strict
Clippy, warning-denying Rustdoc, default CLI, CLI reference/API symbols,
formatting and experimental CLI build passed. Hygiene checked 1,447 links and
36 bundles / 8,929,026 bytes. One receive-only session authenticated, then ended
early with unknown cause and no retry. Saved peer/actions/receipts/outbox/events
were unchanged; six new inbound checkpoints remain controls/unknowns rather
than user replies. No qualifying live companion transition occurred. listen now
preserves terminal state before shutdown and prints fixed redacted cause labels;
this diagnostic fix is locally tested, not yet live observed. Primary cleanup,
namespace migration and resend remain unfinished.

2026-10-09 — **Confirmed static:** bulk user receipt restoration uses a distinct
8G8.A05 writer from chat transfer/history sync. Any cache-slot improvement enables
SQL with every positive supplied timestamp; this is not the one-column live
writer or a universal SQL minimum policy. Descriptor field 40 and nested fields
1..6, actual caller branches and unique user-row index are retained. Exact table
xref negatives do not prove absence of generic normal-send seeding. Primary
cleanup still needs actual user-row existence/admission, not synthesized facts.
[Evidence and remaining port inputs](user-receipt-import.md).

Validation: nine source topics / 14 files / 204,931 bytes reproduce exactly from
the pinned APK. Cached self-diff verifies nine unchanged topics and four metadata
artifacts, not another release's behavior. Self-remap selects the correct original
class/method for seven topics; one is ambiguous and one unanchored, so no complete
proposed recipe is emitted. The committed compact report keeps actual candidate
selections and the full report digest; unrelated per-anchor hit lists stay local.
CLI-reference regeneration, API/library-symbol validation, formatting and diff
hygiene passed. Repository hygiene checked 1,463 local links and 37 bundles /
9,133,957 bytes. No Rust runtime changed, no fresh workspace/runtime test is
claimed, and no account or live operation ran. The prior cfda410 functional
chunk's 810-test validation remains separate.

2026-10-09 — **Confirmed static:** the four remaining exact receipt_user
reference roles are classified: backup_changes trigger construction, migration
error classification, whole-table deletion and receipt table metadata. The
trigger INSERT destinations are backup_changes, not receipt_user. Generic
consumers/normal-send user-row seeding remain unknown; primary cleanup still
requires actual user-row existence. No runtime or account behavior changed.
[Complete methods and reasoning](user-receipt-reference-roles.md).

Validation: ten source topics / 15 files / 138,158 bytes reproduce exactly from
the pinned 37.71 APK; cached self-diff verifies ten unchanged topics and four
metadata artifacts. CLI-reference regeneration, API/library-symbol validation,
formatting and whitespace checks passed. No fresh workspace/runtime test or
live operation is claimed; the prior functional validation remains separate.
Repository hygiene checked 1,471 local links and 38 bundles / 9,272,115 bytes.

2026-10-09 — **Confirmed static:** explicit receipt table-descriptor readers
are incremental backup export/restore and JSON metadata serialization. Descriptor
fields now have actual serializer labels, including optional lookup_field.
Restore contains a dynamic table INSERT; its specific receipt input continuity
and complete validation remain untraced. Normal-send user-row seeding and
primary cleanup are still unfinished. Exact scans, scoped source, signatures,
catch tables and complete-dump digests are retained outside the checksum bundle.
[Consumer evidence and limits](user-receipt-reference-roles.md#descriptor-consumers-follow-up).
No Rust/account/network behavior changed.

Consumer follow-up validation: CLI-reference regeneration, OpenAPI/library-symbol
checks, formatting and whitespace checks passed. Hygiene checked 1,475 local
links and the unchanged 38 checksum bundles / 9,272,115 bytes. The new excerpts
are separately attributed, not counted as checksum-bundle coverage. No fresh
workspace/runtime or live test is claimed.

2026-10-09 — **Confirmed static:** receipt ownership now has an exact caller
boundary: true message-key from-me inputs short circuit to ordinary admission
before the A0y helper/foreign-input property. False-key helper logic is
(type 64 + stored-own Bps.A00 + clear flag); later pool gates still apply.
Property 35516 has true/true baked defaults, not a known effective account value.
Ten explicit flag writers are retained with scoped source and full-dump digests;
constructors/receive/copy contexts do not prove universal origin semantics.
[Source, branch arithmetic and remaining prerequisites](receipt-ownership.md).
No Rust user-row projection, primary cleanup, account or network change is claimed.

Ownership validation: eight topics / 13 files / 59,820 bytes reproduce exactly
from APK 37.71. Cached self-diff verifies eight unchanged topics and four metadata
artifacts, not cross-release equivalence. CLI-reference regeneration,
OpenAPI/library-symbol checks, formatting, whitespace and hygiene passed.
Hygiene checked 1,486 local links and 39 bundles / 9,331,935 bytes. Flag-writer
excerpts are separately attributed outside that bundle. No fresh workspace
runtime test or live account operation is claimed.

2026-10-09 — **Confirmed static:** native message status is A08, with direct
getter/reset and a downgrade-logging setter that still assigns. Two subtype
setters always select 6. Five direct writer methods and 111 physical superclass
setter calls are retained. The blank receipt message entry connects to device
pending insertion; failure writes local state 20, not a recognized server reason.
Conditional coexistence rows are separate from receipt_user membership.
[Exact methods, provenance and remaining initialization inputs](message-status.md).
Ordinary outgoing initialization, full admission and primary cleanup remain
unfinished. No Rust runtime, account or network behavior changed.

Message-status validation: eleven topics / 16 files / 79,539 bytes reproduce
exactly from APK 37.71. Cached self-diff verifies eleven unchanged topics and
four metadata artifacts, not cross-release semantic parity. CLI-reference
regeneration, OpenAPI/library-symbol checks, formatting, whitespace and hygiene
passed. Hygiene checked 1,499 local links and 40 bundles / 9,411,474 bytes.
Caller/writer inventories and the no-matching ordinary override body result
remain separately attributed. No fresh workspace/runtime or live test is claimed.

## 2026-10-09 — Identity ACK dispatch correction (37.71)

**Confirmed static / implemented offline:** identity dispatch does not call the
ACK deferral used by count/PQ-count. Original descriptor is ACKed after dispatch,
independently of worker success/adoption. Deferred watchdog logs and clears;
no retry. [Source, reasoning and unknowns](identity-notification-ack.md).

## 2026-10-09 — Identity ACK writer and bounded live validation (37.71)

**Confirmed static:** arg1=76 legacy case reaches 0y2.A02, Cp2.A05 and
1Rd.A04. Separate send-message-ACK call was insufficient evidence for this
route; native in-memory offline queue is not durable process-restart replay.
**Observed:** receive-only authenticated and stopped; eight inbound checkpoints,
unchanged actions/receipts/pending/head, no qualifying identity control. Live
ACK acceptance **unknown**. [Trace and redacted record](identity-notification-ack.md).

## 2026-10-09 — Same-key classic refresh with authenticated server child (37.71)

**Confirmed static / implemented offline:** Bot.A03 → 1xi.A1U → Cai.A08
uses selected-user force_override_e2ee_state child presence. CPB admits the
same-key absent-CWS subset under either property value and every decoded old
state when present. Held ratchet is archived by CQc, not discarded or reused
as current. No-child unknown sticky state remains deferred; no metadata bypass,
native state fabrication or transmitted override. [Evidence and tests](same-key-refresh.md).

Same-key live validation: rebuilt b771786 actor authenticated/activated/stopped
in one bounded 45-second receive-only window. Ten inbound checkpoints; actions,
receipts, pending outbox, generation, registration and journal head unchanged.
Historical verified matching reply has server child absent, original guard
untracked and applied=false. No fresh qualifying reply or live application
claimed. [Redacted record](same-key-refresh-live.json).

## 2026-10-09 — Companion ADV details parse/serialize (37.71)

**Confirmed static / implemented offline:** 11T.A0D uses GsB.toByteArray at
024c–0254 for both signatures. Original-byte signing assumption refuted;
known normalization and lite unknown fields retained. Azc enum includes
NON_E2EE=2; local reference schema extension is annotated and generated.
**Observed reference:** 637 descriptor-reconstructed Java serialization vectors
reproduce; exact APK runtime version/depth remains unknown. No membership/MAC,
pin/state/teardown guard is bypassed. [Trace, oracle and receiving tests](adv-details-serialization.md).

ADV serialization live validation: one rebuilt saved-account 45-second receive-only
window authenticated/activated/stopped with unchanged actions/receipts/outbox,
generation/registration/head and 8 new inbound checkpoints. No qualifying
companion ADV or decrypted message; live branch parity unknown. Historical
untracked identity action stays unapplied. [Redacted record](adv-details-serialization-live.json).

## 2026-11-04 — Legacy IQ registration boundary

**Confirmed static (43.72):** inherited 1rA.A02 maps 1rD to `iq`;
0ZJ.A19 rejects duplicate tag registration. Exact 1rC constructor search
finds no direct invoke callers, which does not establish factory absence.
Configured factory membership and actual dispatch remain unknown.
[Exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-iq-handler-registration-boundary-2026-11-04). No production change or live traffic.

## 2026-11-04 — Concrete legacy listener handoff

**Confirmed static (43.72):** normal setup binds1lZ.A01 to1lY;
CPm uses arg1 and conditionally copies events to0ln/0m2 or queues on1lY.
A0Q-enabled77/78 use0ln; subsequent14o manager binding remains unknown.
[Exports, branch boundaries and interface-search limits](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-concrete-listener-handoff-2026-11-04).
No production change or live traffic.

## 2026-11-04 — Legacy enabled-route manager binding

**Confirmed static, conditional (43.72):**0ln.A00 routes77/78 through
SQ0.A0s provider6735, whose application-scope factory constructs14o.
This closes the A0Q-enabled route to retained14o.BLn upload branches.
Alternate configuration/factory membership remain unproven.
[Exports and conditions](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-enabled-route-manager-binding-2026-11-04). No production change or live traffic.

## 2026-11-04 — Legacy fallback event route

**Confirmed static, conditional (43.72):** selected cold-start fallback
0ZJ.A0q ->0YU.CPn/what5 returns without handling77/78. This does not
prove these events take fallback in normal configuration. Enabled route
remains bound to14o; lifecycle guards remain unverified.
[Exports and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-fallback-event-route-2026-11-04). No production change or live traffic.

## 2026-11-04 — Configured legacy IQ factory membership

**Confirmed static (43.72):** A1H set7466 includes binding126, which
allocates1rC through08o.A0Q with Object constructor inlining. Membership
is now proven; earlier zero direct constructor callers did not cover it.
A0Q comes from property0x4c7a at construction; runtime value unknown.
[Exports and remaining lifecycle scope](apk/2.26.43.72-prekey-upload-replies/README.md#configured-legacy-iq-factory-membership-2026-11-04). No production change or live traffic.

## 2026-11-04 — Disabled-flag registry route

**Confirmed static after initialization (43.72):** set7469 includes14o
(binding6735), Amf declares77/78, and0ZJ.onLooperPrepared calls0ln.A05
to register them. Disabled flag does not itself imply callback loss.
Fallback disposition remains true if reached; early-event/lifecycle unknowns
remain. [Exports and conditions](apk/2.26.43.72-prekey-upload-replies/README.md#disabled-flag-registry-route-2026-11-04). No production change or live traffic.

## 2026-11-04 — Upload reply fields separated

**Implemented offline,43.72 callback evidence:** successful prekey replies
ignore error metadata; refusals ignore count_low flags. Synthetic mixed-body
reply regression verifies outcomes and persisted batches after reopen.
[Evidence, test and remaining gaps](apk/2.26.43.72-prekey-upload-replies/README.md#rust-reply-field-separation-2026-11-04). No live traffic.

## 2026-11-04 — First coded upload error and numeric conversion

**Confirmed static / implemented offline (43.72):**1ud.A01/0HF.A00
select first present code, parse signed Java integer, default0. Rust now
persists canonical decimal without eight-character truncation; historical
batches retained. Reopen regression covers malformed-first and limits.
[Evidence and remaining scope](apk/2.26.43.72-prekey-upload-replies/README.md#rust-coded-error-selection-and-numeric-conversion-2026-11-04). No automatic retry or live traffic.

## 2026-11-04 — Upload settlement reservation ownership

**Implemented offline, local invariant:** correlated upload settlement now
rechecks ordered durable batch IDs and each key metadata status/owner inside
the transaction. Mismatches refuse without partial writes; private-row
absence does not imply lost ownership. Fault/reopen regression retained.
[Details and limits](prekey-count-notifications.md#upload-settlement-reservation-ownership-2026-11-04). No new retry or live traffic.

## 2026-11-04 — KEM upload reservation binding

**Implemented offline, local invariant:**PendingUpload retains the KEM IDs
from its reservation transaction and compares the durable list at settlement.
Empty/reordered/substituted lists refuse before any writes, on result/error.
Existing consumed-key late-ACK regression expanded.
[Details and limits](prekey-count-notifications.md#kem-upload-reservation-binding-2026-11-04). No new retry or live traffic.

## 2026-11-04 — Independent PQ count-low retention

**Implemented offline,43.72 callback evidence:**persist independent PQ
flag, retain historicalunknowns and expose redacted acknowledged-batch count
in existing CLI/HTTP prekey status. No PQ-to-legacy trigger conflation or
automatic PQ replenishment added.
[Evidence and regression](prekey-count-notifications.md#independent-pq-count-low-retention-2026-11-04). No live traffic.

## 2026-11-04 — Durable refusal upload hold

**Implemented offline, local incomplete-policy hold:** source audit found
refused unsent keys could be reserved again on notification/reconnect.
Any saved refusal now holds new reservations transactionally and is visible
in prekey status. Native retry strategy is still required, not declared
complete. [Evidence, regression and correction](prekey-count-notifications.md#durable-refusal-upload-hold-2026-11-04). No live traffic.

## 2026-11-04 — Legacy refusal workers/reset boundary

**Confirmed static (43.72):**legacy406 Fea9 callsA05/A0m(reason9)
thenA0S(reason7), distinct from modernFea13/A06/A0R. A0m records identity
reset reason and enters cleanup whose full effects remain unknown.
Legacy5xx Fea10/11 ->Fea8 rechecks backoff/admission beforeA03.
[Exports and incomplete port scope](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-refusal-retry-workers-and-reset-boundary-2026-11-04). No production change or live traffic.

## 2026-11-04 — SQLite reset base/helper selection

**Confirmed static (43.72):**base helper attempts database deletion and
checks sidecar absence;Eoe clears three collections. Helper flag0x646a
selects0Xv(axolotl.db) or28o path. Full virtual/native cleanup and rebuild
semantics remain unproven. [Exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#sqlite-reset-base-implementation-and-helper-selection-2026-11-04).
No production change, account reset or live traffic.

## 2026-11-04 — Last-resort producer and timestamp

**Confirmed static (43.72):**A0e reuses newest unsentLR or generates at
rawKEM cursor and advancespositive cursor. Storage writes upload_timestamp
while sent=false, so timestamp is not migration/ACK proof. Current Rust
SET helper differs; entry-driven ADD/rotation port remains incomplete.
[Exports and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#last-resort-producer-and-pre-upload-timestamp-2026-11-04). No production change or live traffic.

## 2026-11-04 — Refusal hold actor verification

**Verified synthetic actor:**503 settlement, repeated count notification
ACKs, clean receive and exact-store reopen produce no replacement upload
or changed private prekey rows. Only the original refused batch remains.
[Regression scope and limits](prekey-count-notifications.md#refusal-hold-actor-verification-2026-11-04). No live traffic or new retry capability.

## 2026-11-04 — Legacy success reaches migrated ADD admission

**Confirmed static (43.72):**14o77 ->Fcj3 marks both families, clears
backoff, then either demandflag ->Fc8(selector7) ->A0V. Thus existing
migration/admission/ADD evidence applies to legacy SET success too.
Rust unconditional legacy SET followup remains a confirmed gap.
[Exports and port requirements](apk/2.26.43.72-prekey-upload-replies/README.md#legacy-success-to-migrated-add-admission-2026-11-04). No production change or live traffic.

## 2026-11-04 — Migrated success followup ADD

**Implemented offline,43.72 evidence:**independent success demand enters
gated uploaded-LR ADD reservation; family-specific inventory/batch defaults
and exact ADD children frozen transactionally. Existing cursors/SET wire
and uncertainty/refusal holds retained. Actor/PQ-only and rollback/reopen
regressions added. [Scope and remaining routing/config gaps](prekey-count-notifications.md#migrated-success-followup-add-2026-11-04).
No live traffic or full replenishment parity claimed.

## 2026-11-04 — Count notification SET/ADD routing

**Implemented offline,43.72 declared-default branch:**migrated legacy
notifications use ADD;non-migrated use SET;PQ-only remainsACK-only under
EMv0x70d7 defaultfalse. Explicit actor trigger preserves entry semantics.
Reopen/uncertain-KEM and synthetic actor tests expanded.
[Evidence and config/ACK/startup limits](prekey-count-notifications.md#count-notification-setadd-routing-2026-11-04). No live traffic.

## 2026-11-04 — Startup new-store/digest distinction

**Confirmed static (43.72):**new-store flag drivesFea6, which sends
existingunsent keys or onlygenerates;non-new-store digestflag drives
A0M/readiness/elapsed>60000ms thenarg1=88. A0Q mark-unsent entry is not
proven genericstartup. Rust startup parity remains incomplete.
[Exports and port requirements](apk/2.26.43.72-prekey-upload-replies/README.md#new-store-startup-and-digest-entry-distinction-2026-11-04). No production change or live traffic.

## 2026-11-04 — Digest request/reply wire grammar

**Confirmed static (43.72):**arg1=88 GETencrypt/digest with291selector1;
result fixed-sizefields/id lists and hash20/32 dependingonLR presence.
Success/error events82/83/84 distinguished; numeric parser differs from
upload290. [Exports and verification/port limits](apk/2.26.43.72-prekey-upload-replies/README.md#digest-request-and-wire-callback-grammar-2026-11-04).
No production change or live traffic.

## 2026-11-04 — Digest local verifier/resynchronization

**Confirmed static (43.72):**orderedlocalkey-material digest with
SHA1/SHA256 chosenbyPQgate+LRpresence;IDs arenot hashed. Registration,
key availability/ID andhashmismatches have separateA0Q reasons.
Attempttime andpersisted digestflag clears distinguished.
[Exports, preimage and recovery limits](apk/2.26.43.72-prekey-upload-replies/README.md#digest-local-material-verification-and-resynchronization-2026-11-04). No production change or live traffic.

## 2026-11-04 — Digest error/resync metadata scope

**Confirmed static (43.72):**404 worker callsA0Q4 thenclearsdigestflag;
503 onlyclearsattempttime. Resync updates formerlysent legacy/KEM metadata
and clears key-type uploadhistory, preserving recordfields inselectedSQL.
Uncertain Rustreservations remain distinct; correlation/port stillopen.
[Exports and statechange boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#digest-not-foundserver-error-and-metadata-resync-scope-2026-11-04). No production change or live traffic.

## 2026-11-04 — Offline digest inspection

**Implemented/verified offline:**existingcapturereply grammar andlocal
materialcomparison exposedthroughprekeys --digest-capture. Redactedresult
only; no statechange. FrozenACKedLRsignature used without randomized
resigning; unavailable/ambiguous evidence stays explicit. IndependentSHA1
andread-only/PQregressions retained. [Limits and usage](prekey-count-notifications.md#offline-digest-inspection-2026-11-04). No live traffic.

### Exact KEM publication signatures (2026-11-04)

Implemented offline: KEM upload reservation now atomically persists the exact
identity signature sent for each selected row. Later construction reuses and
verifies saved signatures; digest inspection prefers the saved last-resort
signature, with acknowledged frozen SET recovery for historical unsigned rows.
Missing historical signatures remain unavailable; inspection never re-signs or
writes. Invalid saved signatures fail locally. Reservation rollback and consumed
row handling remain unchanged.

Confirmed static source remains pinned APK 2.26.43.72 EqE/Dfd signature storage
and Fdf digest preimage (see the prekey-upload-replies evidence bundle). This is
partial storage parity: Rust signs at reservation, whereas native generation
stores the signature earlier. No live verification, recovery, retry-policy or
ban-prevention guarantee is implied.

### PQ gate and key transaction consistency (2026-11-04)

**Implemented, offline:** prekey SET/ADD reservation reads the saved property
21198 through the same WriteTx that selects/generates key rows and freezes the
publication request. Digest inspection reads the saved gate, account, registered
signed key and local prekeys through one ReadTx. A prior separate store read
could combine a previously read gate with later key state. This is a local
transaction consistency fix, not new APK timing evidence or a MobileConfig port.

Both paths reuse the existing correlated AB-property parser and its absent/
invalid false default. The reservation path also validates decoding of the
saved original query through the existing action write-view helper. A missing
referenced action or malformed stored action remains a local error, never an
inferred enabled gate. Static gate provenance remains APK 2.26.37.71 D1G.Bat /
07t.A0L and 2.26.43.72 13z migration predicates in the retained prekey bundles.

Regression holds an immutable snapshot across a committed config/key change,
checks uncommitted gate deletion in a write transaction and rollback, and
reopens the resulting synthetic store. Request/reply origin, current actor
account binding, effective MobileConfig, startup recovery and safe resync remain
separate unfinished work. No live operation or automatic recovery was added.

### Prekey publication account authority (2026-11-04)

**Confirmed local defect, fixed offline:** begin_upload_selected accepted an
AccountRecord supplied before reservation without rereading the durable account.
It also serialized the supplied signed-prekey signature without verification on
the PQ-disabled path. A stale caller could therefore freeze a SET under obsolete
registration/identity material, or submit a signed key from another identity.

SET and ADD admission now require registered caller and stored accounts, equal
transaction/stored/caller generation, registration ID, own JID, phone and Signal
public/private identity. These checks run in the existing reservation WriteTx
before key selection, generation, signature persistence or request freezing.
Missing account and changed authority have fixed local errors. Signed-prekey
signature is verified over the type-prefixed serialized public key under the
publication identity even when PQ is off. Authentication timestamps/credentials
are not publication identity and do not invalidate an otherwise matching caller.

This is a local consistency invariant. APK 2.26.43.72 13z.A0B/A0S request layout
and stored key sourcing remain documented in the prekey-upload-replies bundle;
these Rust error messages and checks are not claimed as native callback policy.
Regression fixtures now persist their synthetic account before reserving uploads.
Nine stale/missing/revoked account cases leave Prekeys and Checkpoints unchanged
across reopen; a foreign-identity signed key fails with PQ off, while an unrelated
authentication timestamp update permits the valid request.

**Remaining:** same-identity signed-key rotation ownership, account changes after
reservation/before write, correlated digest recovery and native retry/reset
policy are not closed by this guard. No live operations or automatic resets.

### Prekey wire ID integrity (2026-11-04)

**Confirmed local construction defects, fixed offline:** signed-prekey IDs wider
than three bytes were truncated by be3. Legacy publication trusted the numeric
store-key suffix without checking the deserialized PreKeyRecord ID, so a row
could publish its public key under a different ID. Offline digest comparison
likewise accepted those inconsistent rows and could match an aliased signed ID.

Signed IDs are now checked before publication or digest comparison. Legacy rows
share a checked loader that requires a lossless three-byte requested ID and an
embedded record ID equal to its store key. Publication failures roll back the
reservation transaction; read-only digest failures leave records untouched.
KEM publication already checked the three-byte width and row ID binding.

**Confirmed static boundary:** pinned APK 2.26.43.72 1nr.A04 writes low24 bits
(shift16/8/0), retained under ADD selection and exact child layout in the
prekey-upload-replies bundle. Its truncation is native encoder behavior; rejecting
wide local values is our consistency guard, not evidence that native rejects
those values. Native legacy row parsing can skip caught IOExceptions; Rust's
stricter invalid-row refusal remains explicitly different. No cursor, sorting,
generation, retry, reset or acknowledgement policy changes are inferred.

Regressions cover mismatched legacy store/embedded IDs, wide legacy IDs, signed
IDs 0x1000000/u32::MAX, and exact 0/0xffffff field boundaries. An independent
SHA1 digest fixture is reused with unchanged public bytes but a mismatched row
ID, and with an aliased wide signed ID: both now fail instead of falsely matching.
These field-boundary tests do not establish native generation of ID zero.

### Count ACK metadata and queue boundaries (2026-11-04)

**Confirmed static, APK 2.26.43.72:** count dispatch requests deferred metadata;
89Y.A01 consumes a one-use7p2 token viaA00. The downstream0v6.A00 branches to
bulk1ic.A0G or0Sf.A0O readiness/pre-ACK queue admission. The latter creates
message76 and can convert to queued pre-ACK message43; it is not a synchronous
socket-completion callback. [Exact methods, offsets, limitations and 18 checked
exports](apk/2.26.43.72-prekey-upload-replies/README.md#count-notification-ack-token-and-queue-stages-2026-11-04).

This closes the meaning of the worker's ACK call, not native wire-order parity.
The same metadata object dispatches at most once, but durable/redelivery-wide
ACK deduplication is not established. A six-second handler timer and pre-ACK
threshold200 were observed statically; neither is a new Rust timer/default.
Concrete live branch selection, queue durability,43 pre-ACK conversion/flush and
failure/reconnect drain behavior remain unknown. Rust ACK-before-upload-decision
ordering remains unchanged pending those boundaries; no speculative socket
reorder, pre-ACK outcome_unknown reservation, retry or duplicate suppression.
No production source change or live account operation in this evidence chunk.

### Durable pre-ACK storage and confirmation watermark (2026-11-04)

**Confirmed static, APK2.26.43.72:** the ACK conversion helper accepts message76
and uses the same generic ACK builder as its direct writer. Message43 writes the
already-built node. Native1iV.A0F serializes queued nodes into SQLite preacks.ptn;
1iY loads by ascending row ID and deletes cumulatively through a confirmed row
watermark. This closes the prior unknown about existence of durable pre-ACK
storage, but not whether every live count notification selects that branch.

The ping manager snapshots a queue marker and its successful IQ callback requests
retirement; socket write alone is not proven retirement authority. Replay-deferred
and passive-state holds, a reconnect-epoch check, capacity-driven pending drops,
periodic flush scheduling and persisted/pending selection are retained. [Exact
methods, boundaries and 24 new checked exports](apk/2.26.43.72-prekey-upload-replies/README.md#pre-ack-conversion-sqlite-persistence-and-retirement-2026-11-04).

**Still open:** effective configuration, complete initial load and replay-send
ownership, bulkCCQ selection, ping correlation/error timing, partial/ambiguous
commit handling, and all retirement entry points. Rust has durable raw inbound
captures, not this native pre-ACK queue. No ACK replay/deduplication, new ping,
automatic deletion or socket-order change was implemented from partial evidence.
No live account operation; this is a documentation/evidence chunk.

### Independent keepalive response deadline (2026-11-04)

**Fixed, offline:** heartbeat maintenance writes could delay an outbound ping,
then the next heartbeat expired it after only the remaining fraction of the
interval. A paused-clock regression reproduces a19-second maintenance delay
followed by a ping that previously timed out1second later. Each ping now owns
an admission-time deadline and a separate timeout selection; silence still
produces the existing recoverable Timeout after the configured full window.
Heartbeats cannot send a second ping while one is pending. Existing response
correlation, duration policy, request shape and reconnect classification remain.

**Confirmed static, APK43.72:** the pre-ACK confirmation ping is an empty
IQ/get/w:p to s.whatsapp.net. 0Sf.A06 registers its ID callback in0ln.A08 before
send admission, with an independent Handler delay (32000ms for this native
purpose). [Request, timer methods and regression limits](apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
The native pre-ACK delay does not replace Rust's local keepalive setting. This
does not port durable ACK replay/retirement, or guarantee timely observation
while the actor is blocked in a write. No live operation was performed.

### Pre-ACK reload and replay-generation ownership (2026-11-04)

**Confirmed static,43.72:** initial load has unbounded and oldest-window modes;
window mode tracks total persisted count separately and can skip up to eight
empty decoded pages. Reload preserves SQLite IDs with loggableID=-1. The store
compares dictionary version3 and deletes incompatible nonzero-version pre-ACK
rows; this is not identity/key reset authority. Declared descriptor values are
retained, while effective configuration remains unknown.

Replay generation initialization clears the sent watermark and increments an
in-memory connection counter. Mark-sent, clear-deferred and delayed retirement
check that generation; a watermark is not an account-generation or stanza ID.
Capacity SQL prunes newest excess rows in batches500, preserving the oldest
prefix. An independent synthetic SQLite oracle verifies strict-after paging,
cumulative retirement and that pruning direction. [Exact boundaries,11 new
exports and oracle](apk/2.26.43.72-prekey-upload-replies/README.md#pre-ack-reload-capacity-and-connection-generation-ownership-2026-11-04).

A supplied-transaction insertion path also exists, but its callback commit/
rollback semantics and full replay-batch ownership remain unfinished. No Rust
send/replay/deletion, socket order, timer or account behavior changed here.

### ACK transaction success and rollback bridge (2026-11-04)

**Confirmed static,43.72:** ACK insertion's0aq.A04 registers MZH selector1.
Its Runnable executes in BjU after successful outer transaction completion,
not directly in SQLite.onCommit. Separate selector2/BjQ handles rollback.
1hk.close ends SQLite first and only selects success release when no captured
end error, its success flag is set, and no rollback marker is present. Inner
transaction closure does not release the outer callback map.

Bulk1ic.A0G ->8b6(selector1)->1ic.A0A can insert pre-ACKs with the same0aq
transaction used by its queue helpers, then register success/rollback Runnables.
After successful completion, TB5(11) promotes persisted entries directly or
schedules further projection according to load/window state. This closes the
normal callback boundary, not crash/disk-error semantics or complete provenance
of every count notification's ACK list. [Methods,19 new exports and limits](apk/2.26.43.72-prekey-upload-replies/README.md#ack-transaction-completion-and-rollback-bridge-2026-11-04).

**Remaining:** complete count-specific bulk/config selection, Flr insertion
coroutine, TB8 projection, exact window comparator, ambiguous persistence and
full replay ownership. No Rust network/recovery behavior or live account state
changed. Future port must distinguish durable insertion from post-commit memory
promotion and confirmation-based retirement.

### ACK projection serial queue and row-order helpers (2026-11-04)

**Confirmed static,43.72:** TB8 selector8 directly calls A07; it does not add
another latch wait. The queue owns a098 serial executor: its load/projection/
periodic jobs use a FIFO wrapper that attempts next-task dispatch on normal or
exceptional completion. This closes same-instance queued-job ordering, not all
cross-thread epoch mutation, dispatch failures or database snapshot isolation.

Fey14 orders persisted row IDs naturally, substituting Long.MAX_VALUE for null;
no loggable-ID tie-break is used. Out-of-order window merge concatenates, sorts
and takes a prefix without deduplication. Refill keeps a null/backend failure
separate from a completed empty read, and adjusts only memory/count state on
successful short reads. [Methods,17 new exports and boundaries](apk/2.26.43.72-prekey-upload-replies/README.md#ack-projection-serialization-merge-order-and-refill-2026-11-04).

These are additional recovery-port inputs, not implemented Rust ACK replay or
proof every callback schedules after loading. Complete bulk ACK-list provenance,
Flr insertion, replay ownership and ambiguous persistence failures remain open.
No production Rust source or live account/network behavior changed.

### Actor IQ waits expire independently (2026-11-04)

**Confirmed local defects, fixed offline:** prekey upload and device-query waits
had30-second monotonic deadlines, but only a20-second heartbeat checked them.
A reply at31seconds could settle either request before the40-second tick. In
paused-clock regressions, the old code promoted50 prekeys to uploaded and
converted a reserved device query to server-refused from such late replies.

The actor now selects each wait's own deadline. Expired uploads drop only their
session-local waiter and keep durable outcome_unknown/key reservations. Expired
device queries are deferred for this session while pending users and the durable
query row remain unchanged. This timeout does not authorize a resend, key reuse,
repeating those deferred users, cooldown bypass or clearing uncertain history. Existing admission
triggers and the selected30-second local durations are retained.

Prekey, device-query and keepalive reply admission also checks that its waiter
has not expired, so a simultaneously ready incoming branch cannot settle an
expired waiter merely by winning select. Original inbound capture still commits
before reply handling. Session monotonic deadlines are not reconstructed from
wall-clock/server timestamps after restart; offline library settlement/inspection
and other request families remain separate authority boundaries.

**Confirmed static timing architecture, APK43.72:** 0Sf.A06 registers a pending
ID callback through0ln.A08 before send dispatch; positive delays use an
ID-specific Handler message. [Retained methods](apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
That supports independent request timers, not proof that30seconds is this APK's
native timeout for both families or that Rust now ports its entire callback map.
The earlier native pre-ACK confirmation ping's32-second delay remains distinct.

Regressions: late_prekey_reply_cannot_settle_between_deadline_and_heartbeat and
late_device_reply_cannot_settle_between_deadline_and_heartbeat. They preserve
late captures, check uncertain/query/pending-user state, service an inbound ping
and reopen without reinstalling account material. The former verifies50 keys
remain uncertain and none uploaded; the latter verifies the query remains
identical instead of storing a late503 refusal.

**Limits:** awaited writes may still delay observing expiry. Device admission
and unrelated maintenance remain on their existing cadence. Outgoing/group
waits and other request families are not converted by this chunk; no full
network/timing or ban-prevention guarantee is claimed. No live operations.

### Outgoing preparation owns its deadline (2026-11-04)

**Confirmed local defect, fixed offline:** direct key-query and group preparation
expiry was checked only on the one-second outbox tick. A group device reply
at30.75s, after its30.5s deadline but before the31s tick, could start an encrypt
query. The paused-clock regression fails on the old actor and passes with
independent timers and deadline checks before direct/group reply advancement.
The same admission check covers the group's distribution-ACK waiter.

Existing30-second local durations and failure policy remain: direct key-query
expiry ends the actor with its reserved/no-retry error; group expiry persists
blocked/outcome_unknown and defers an outstanding device query. No automatic
resend, identity change, reservation release or retry was added. Nodes received
by the actor still commit their raw capture before reply handling; actor exit
does not promise reception of later traffic. Generic durable reply/ACK
processing remains unchanged.
Awaited writes can still delay timer observation; no family-specific native
timeout duration or complete callback-map parity is claimed.

**Confirmed static architectural support, APK2.26.43.72:**0Sf.A06 registers
callbacks through0ln.A08 before dispatch, with an ID-specific delayed Handler
message; see the retained [timer methods](apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
This supports independent ownership, not copying the separate native32s
pre-ACK confirmation timeout into outgoing preparation.

Regression: group_preparation_deadline_cannot_advance_on_a_late_device_result
services the30s tick before advancing past the30.5s deadline. It verifies no
new key query, a retained late capture, continued ping service and the same
durable blocked summary after reopening without replacing account material.

**Unknown / next trace:** outgoing_group::Pending::matches accepts a distribution
ACK by description and ID; settle_with_policy inspects its error attribute.
The separate outgoing::acknowledge route has stronger class/from/ledger checks.
Native distribution ACK sender/class/callback ownership and whether every
out-of-band distribution is represented in that generic ledger remain untraced.
This is a local audit lead, not a confirmed APK mismatch; do not guess sender
address requirements or claim generic ACK persistence covers every distribution.
No live account or network operations were used.

### Distribution ACK class admission (2026-11-04)

**Confirmed static / fixed offline:** APK43.72 1rH.A03 dispatches ordinary
message ACKs by exact class; Efa/Eoh field propagation and Eoh.equals confirm
class is part of callback identity. Rust's distribution waiter previously
accepted any ACK family with the same ID. It now requires class=message for
both success and error settlement. Missing/receipt/notification/call/case-mismatch
regressions fail before the fix and pass afterward, with valid reaction
distributions still completing the existing crypto and reopen/no-replay test.
[Retained methods, error459 qualification and unknowns](apk/2.26.43.72-distribution-ack-admission/README.md).
Native distribution callback registration, address normalization and complete
durable confirmation remain open; no raw sender-equality rule or retry was added.
No live operations or complete ban-prevention claim.

### Sender-key ACK marking, one-time reset and callback ownership (2026-11-04)

**Confirmed static, APK43.72:** SendE2EMessageJob.A06 registers FfK before
dispatch and waits for its matched callback; normal successful ACK processing
checks key existence and marks selected devices. The one-time-key branch instead
reaches0i9.A0X ->0j9.A01, which resets sent_sender_key=0, not1. Blanket
interpretations of every post-ACK helper as marking reusable key possession
are refuted. This is not proof that reactions use the one-time branch.
0Th.A02 also connects2Gc selector1's pre-ACK marker origin: a matched callback
invokes the wrapped result before scheduling marker retirement, so those effects
are not one atomic transaction.
[Methods, offsets, local persistence gap and next port boundary](apk/2.26.43.72-sender-key-ack-confirmation/README.md).
Native out-of-band distribution address/registration ownership remains open.
No production change, new replay authority or live operation.

### ACK callback normalization and inbound bridge (2026-11-04)

**Confirmed static, APK43.72:** Ewq/Eng preserve the job's Eoh callback key.
0Sf.A02 canonicalizes supported device-zero from/participant fields to user
JIDs and retains nonzero companions. This helper does not establish PN/LID
alias equivalence. The generic1lZ.A1D ->1lY.Big ->0ln.A09 bridge is now
connected; the selected inbound bridge does not call that outbound normalizer.
A0T is exact normalized-key failure removal, not success handling.
[Field propagation, type predicates, callers and port limits](apk/2.26.43.72-ack-key-normalization/README.md).
Distribution producer inputs and parser/type mapping still need tracing before
changing expected-address admission. No production or live behavior changed.

### ACK API result variants and distribution-shape qualification (2026-11-04)

**Confirmed static, APK43.72:**0Sf.A0B/A0C register through0Th.A01/2Gc0,
which discards the returned node; A0D and ordinary SendE2EMessageJob use
A02/2Gc1 and retain it. Null get() results therefore do not universally mean
a missing ACK. Failure forwarding does not retire the held pre-ACK marker.
SendLiveLocationKeyJob builds notification/type=location with participants,
not ordinary direct messages. Neither that job nor includeSenderKeysInMessage
proves the native ordinary reaction distribution shape or callback address tuple.
[Methods, caller inventories and remaining gaps](apk/2.26.43.72-ack-api-variants/README.md).
No production behavior, speculative address restriction or live operation changed.

### Auth-challenge ACKs preserve uncertainty (2026-11-04)

**Confirmed static / fixed offline:** APK43.72 1rH.A03 diverts numeric459
to onAuthChallengeReceived before ordinary ACK class processing. The traced
manager transfers an existing request between maps; it is not an ordinary
terminal message failure or proof of a ban. Rust direct/group ACK settlement
now excludes459, preserving original reservations and uncertainty. Java decimal
spellings are covered; no challenge bypass, retry or account-wide policy added.
[Methods, before/after regression and unported boundaries](apk/2.26.43.72-ack-auth-challenge/README.md).
No live operations. Full challenge-controller parity remains incomplete.

### Comparison sends share message ACK classification (2026-11-04)

**Confirmed local defects / fixed offline:** CLI comparison direct and final
group sends stored every nonaccepted ACK as server_refused, including numeric459.
The out-of-band group distribution loop accepted any ACK family without an
error and did not preserve the matched ACK itself with its other captures.

All three now use messaging::message_ack_outcome: ordinary message-class ACKs
classify as accepted/refused; numeric459 is AuthChallenge before class dispatch;
other ACK families are Unrelated. Challenge/unrelated map to outcome_unknown
and return an explicit no-retry error after saving the response. Distribution
ACKs are preserved through the encrypted inbound capture path, and unresolved
ones stop the loop before any later distribution/group write. They do not
confirm sender-key possession. Original intents remain nonreplayable.

**Confirmed static support, APK43.72:**1rH.A03's class dispatch and special459
diversion, Integer.parseInt grammar and callback-map distinction are retained
in [auth-challenge evidence](apk/2.26.43.72-ack-auth-challenge/README.md) and
[ACK admission evidence](apk/2.26.43.72-distribution-ack-admission/README.md).
The shared helper classifies a node only; ID/address binding remains the caller's
duty. Existing wait_for matches tag/ID. Full native expected-address tuples,
participant/type/edit rules and challenge recovery remain incomplete.

The bounded CLI exits with uncertainty rather than waiting for a later matching
family; this is local no-replay policy, not a port of the native callback
scheduler. Existing ordinary refusal statuses remain unchanged. Historical
saved server_refused rows are not rewritten. This chunk adds no retry, reset,
new request shape or live operation.

Validation: protocol table covers missing/wrong classes, numeric459 decimal
variants including full-width digits, ordinary500, accepted ACKs and non-ACK
nodes. Existing durable actor reservation regressions still exercise the same
shared challenge parser. Full socket-level CLI sequence coverage is not claimed.

### Comparison IQ waiter reply admission (2026-11-04)

**Confirmed local defect / fixed offline:** messaging::wait_for matched only
node tag and ID. A same-ID IQ get/set, absent type, empty type or differently
cased Result could return as a request response, ending the wait and reaching
a caller's success/refusal or parsing path. The shared comparison waiter now
requires exact result/error for IQs. Other stanza families keep tag/ID matching;
message ACK classification remains separate.

**Confirmed static, APK43.72**, SHA-256887a6f94…0c2e:
0ln.A01 reads type and invokes CK4 only for result and Bvj only for error;
unexpected types reach unexpected_recv_iq_type diagnostics/assertion.
[Retained REA method](rea/2.26.43.72-authkey-outcomes/incoming-iq-callback.json),
Evidence ev_445bcabed23b1091f4b30e63e2bd073f980299ad5b82df8787e84b425d8bf9c2.
This supports dispatch grammar, not identical native pending-map lifetime or
assertion/error policy. Rust retains unmatched IQs in its existing bounded
pending buffer and continues awaiting a valid reply. No new resend.

The existing30-second whole-operation timeout and32-node count bound remain
local policy. Regression comparison_iq_wait_only_admits_native_result_error_replies
fails with the old extracted predicate and passes after the guard; it covers
request/absent/empty/case-mismatched types, both response types, wrong IDs/tags
and unchanged ACK tag/ID admission. This is a predicate regression, not a new
encrypted-socket integration proof or complete native callback-map parity.

**Remaining capture gap:** wait_for returns buffered unmatched nodes only on
success. Timeout, terminal stanza, read error and count exhaustion currently
discard that local buffer; the helper has no store parameter. That loss is not
fixed by this admission guard and must be addressed separately without turning
a failed wait into permission to resend reserved requests. Generic ACK sender
normalization and challenge-controller integration also remain incomplete.
No live traffic or account operation.

### Comparison wait captures survive failed exits (2026-11-04)

**Confirmed local defect / fixed for repository CLI callers:** wait_for kept
unmatched decoded nodes only in its pending Vec, returned only on success.
Timeout, transport error, terminal stanza and count exhaustion discarded that
buffer before callers could persist it. This impaired diagnosis of reserved
requests; it was not evidence those requests had never reached the server.

New messaging::wait_for_captured uses the same bounded read/match loop with an
observation callback that commits each decoded node through inbox::preserve
before matching, checking a terminal tag or advancing the count. It captures
the matched reply too. Capture failure propagates and stops the operation; it
does not authorize settlement or another write. Every current repository CLI
comparison IQ/ACK wait now supplies its existing encrypted store, including
media routes, direct/group key queries, distributions and group retry refetch.
Older successful-path captures remain for compatibility and content-hash writes
remain idempotent; these are diagnostics, not ACKs or processing confirmations.

The existing public wait_for signature is retained for callers without a store;
it still has no durable capture guarantee. Both wrappers share the same IQ
result/error admission,30-second whole-operation deadline and32-node bound.
An explicit post-observation monotonic check prevents capture latency from
allowing reply settlement after the deadline. Blocking storage can delay
observing expiry; no hard real-time responsiveness guarantee is claimed.

**Evidence scope:** this is local diagnostic/persistence hardening, not a claim
that the APK uses our inbox table or content-hash keys. The pinned43.72
[IQ callback grammar](rea/2.26.43.72-authkey-outcomes/incoming-iq-callback.json)
remains the admission reference; native callback-map/challenge parity and
per-request durable recovery are separate work.

Regression decoded_wait_captures_survive_each_exit_and_reopen_without_settlement
uses the actual shared wait loop and encrypted ProtocolStore. Synthetic readers
exercise valid response, timeout, read error, terminal stanza, count exhaustion
and injected capture failure. After reopen, every committed decoded-node
capture exists and a saved uncertain intent remains unchanged.
Capture failure prevents reading the queued matching response. The test is
not an encrypted-socket/Noise interoperability proof.

**Limits:** undecodable or partially read frames do not produce a Node and are
not captured by this helper. Nodes not yet read at timeout cannot be preserved.
No existing historical loss is reconstructed, uncertain send is replayed,
identity is changed, or server wait is overridden. Raw-node capture alone does
not perform protocol ACK/business processing for unrelated notifications.
No live account or network operations were used.

### Undecodable comparison wait frames retained (2026-11-04)

**Confirmed local gap / fixed offline:** wait_for_captured persisted decoded
nodes but still used read_node, which discarded a decrypted frame on stanza
inflation or binary decode failure. It now reads FrameRead and preserves an
Undecodable frame through the existing encrypted inbox capture mechanism
before returning a fixed uncertainty/no-retry error. Already captured decoded
nodes remain. The loop never classifies the malformed frame as a response or
ACKs it, and never reads another node after that failure.

The stored record retains the exact decrypted, pre-inflation frame, fixed codec
error class, first observation time and epoch=None (comparison/lab window),
using the existing content-hash identity. The existing default `undecodable`
command lists these captures redacted and rechecks them with the current
decoder; it does not send traffic or automatically settle them. Capture-store
errors propagate and cannot become successful request settlement.

The shared wait-loop regression now includes malformed binary and invalid
compressed payload cases after two decoded unmatched nodes. It verifies the
error, all prior captures, exact saved frame bytes/class/size, absent actor
epoch and unchanged uncertain send intent after reopening the encrypted store.
Reader injection tests the shared control flow and capture storage; it is not
a Noise-handshake or live malformed-server test.

**Evidence/authority:** this is local diagnostic hardening, not a reconstructed
APK decoder-failure policy. Existing native result/error dispatch evidence
remains the admission reference; whether the APK continues or closes after each
decode failure remains untraced. The comparison waiter exits conservatively.
The ordinary actor's existing continue/capture policy is unchanged.

**Limits:** ciphertext that fails authentication, incomplete frames and data not
yet read do not yield a decrypted frame for this path. The legacy no-store
wait_for remains uncaptured. The earlier decoded-only comparison limitation is
closed for successfully decrypted records; no old missing evidence is restored.
This adds no replay, account reset, fabricated response, new route or live operation.

### Activation reply grammar and deadline ownership (2026-11-04)

**Confirmed local defects / fixed offline:** comparison receive classified any
same-ID IQ as activation result/refusal, including request-shaped IQs. Actor
activation already checked result/error but could admit a late result when the
incoming select branch won against its elapsed timer. Both also started their
30-second response window only after the activation write completed.

The comparison path now uses shared messaging::iq_reply_matches, admitting only
exact IQ/id/result-or-error, and checks its deadline after capture. The actor
checks the same grammar and monotonic deadline before activation-triggered
availability, readiness and startup work. Both create the existing30-second
deadline before sending the request. An awaited write can still delay timer
observation; completion does not restart the response window or authorize a
late result. No activation resend or identity reset was added.

**Confirmed static architectural support, APK43.72:**0ln.A01 dispatches exact
result/error, and0Sf.A0A registers through0ln.A08 before request dispatch.
A08 schedules an ID-specific delayed Handler message for a positive duration.
[Retained callback grammar](rea/2.26.43.72-authkey-outcomes/incoming-iq-callback.json),
[registration](rea/2.26.43.72-authkey-outcomes/pending-registration.json) and
[send order](rea/2.26.43.72-authkey-outcomes/send-iq-internal.json).
This supports grammar and timer ownership, not proof of a native30-second
activation duration or complete callback-map/activation scheduler parity.

Regressions: activation_admission_rejects_the_deadline_boundary_and_later_replies
checks result/error before, at and after expiry using the production admission
helper. slow_activation_write_does_not_restart_the_response_deadline holds the
fake activation write for31seconds, then verifies prompt terminal uncertainty,
no readiness and no availability/startup write. Existing non-reply IQ actor
coverage and native server ping/activation cut tests remain. The comparison
loop's encrypted-socket behavior is not independently newly exercised here.

Captures precede reply admission. A comparison request-shaped activation IQ
stays captured rather than becoming a refusal; if no timely valid
response arrives, activation remains unconfirmed. Existing account/key/send
reservations are not cleared by expiry. Other reply families, awaited-write
responsiveness and full native request-address policy remain separate audits.
No live operations or complete ban-prevention guarantee.

### Comparison request deadlines start before write (2026-11-04)

**Confirmed local timing gap / fixed offline:** comparison IQ/message callers
performed write_node and only then started wait_for_captured's30-second window.
A slow request write could therefore extend request ownership by a fresh30s.
Activation was already fixed separately; the remaining9 repository comparison
write/wait pairs now use messaging::write_and_wait_captured.

The helper creates the existing30-second monotonic deadline before exactly one
write and passes that same deadline into the capture-aware wait. An expired
deadline immediately returns uncertainty without reading a queued late response.
The existing post-capture expiry guard also remains. Request envelopes, IDs,
query batching, encryption and saved send reservations are unchanged. No send
is reserved or retried by the helper itself; the caller retains that duty.
Membership/read-only and retry-refetch request durability is not newly claimed.

Covered pairs: group membership, device batches, directory batches, direct
distributions, final group send, group retry key refetch, direct message send,
media-route query and direct-message key query. Other unpaired writes and
receive loops retain their existing policies. Read-only wait_for and
wait_for_captured APIs still start a window when called; request senders should
use the new combined helper to avoid splitting deadline ownership.

**Confirmed static architectural support, APK43.72:**0Sf.A0A calls0ln.A08
registration before A05 dispatch; A08 schedules its positive-duration callback
timer by ID. [Retained send order](rea/2.26.43.72-authkey-outcomes/send-iq-internal.json)
and [registration](rea/2.26.43.72-authkey-outcomes/pending-registration.json).
The separate [ACK APIs](apk/2.26.43.72-ack-api-variants/README.md) also register
with0Th before dispatch; their full timer policy remains distinct.
This supports registration-time ownership, not a claim that30s is the native
duration for every covered request or that all native scheduling is ported.

Regression expired_request_deadline_is_not_restarted_when_reply_wait_begins
advances the paused clock past a preexisting deadline before entering the actual
shared wait loop, then verifies Deadline and zero reader calls despite a queued
matching result. This models slow-write latency; it is not a new encrypted
socket integration test. Existing decoded/malformed capture and uncertainty
reopen regressions exercise the same deadline-aware core.

**Limits:** a pending write is still governed by its existing transport I/O
policy and can delay observation of the response deadline; it is not silently
cancelled by this helper. A failed write returns its existing error. Unread
late records are not captured, and expiry never proves the server did not
receive the write. Original uncertain intents remain nonreplayable. No native
request-address/challenge-controller parity, new route or live operation.

### Challenged-request selection and re-dispatch ownership (2026-11-04)

**Confirmed static, APK43.72:**0TV.A03 selects eligible held requests without
removing those source entries; over-budget entries are removed.0Sf.A08/A0L
re-dispatch through A04 with the original callback retained when absent from
the new invocation. A token-result callback and the selected handleConnected
block reach selection. The token callback calls key rotation before dispatch,
but does not prove rotation completion. Client-side selection counters are not
a server retry allowance or authority to replay Rust reservations.
[Exact methods, map inventories, count conditions and remaining gaps](apk/2.26.43.72-challenged-request-ownership/README.md).
The post-token helper is now connected; token construction/converters, request
family/callback ownership and durable restart recovery remain incomplete.
No production change, new retry authority or live operation.

### Client-auth-token writer grammar and validation (2026-11-04)

**Confirmed static, APK43.72:** exact Dzb(String,String,int) selector28 builds
a server IQ GET in urn:xmpp:whatsapp:push with a pn child containing UTF-8
manager input. That label is not proof of phone-number input; the branch does
not create an auth_token element. The ID helper checks String.length, not a
53-bit numeric value. Nonnull length violations log via00L.A0C and return true
in the selected validator;300 is not a proven server or hard constructor limit.
[Selected overload, helper chain, checksums and unknowns](apk/2.26.43.72-client-auth-token-writer/README.md).
Writer construction is now connected; input eligibility/authenticity, reply
converters and durable challenge recovery remain unported. No production
change, request sender, new retry authority or live traffic.

### Client-auth-token reply binding and byte grammar (2026-11-04)

**Confirmed static, APK43.72:** FUT's selected result pipeline first requires
iq, matching original request ID, parsed from equal to original destination
and type=result. It then reads opaque cat bytes with strict1..1024 bounds.
The value converter alone does not perform request correlation. Native child
lookup picks the first cat, without duplicate counting in this selected path.
Invalid header throws schema error; null token conversion after a valid header
returns without token persistence or held-request dispatch.
[Converters, helper chain and remaining eligibility/recovery limits](apk/2.26.43.72-client-auth-token-reply/README.md).
This closes the selected reply converter gap, not native JID parser, input
authenticity, failure/backoff or durable recovery parity. No production change
or live operation; Rust uncertain holds remain.

### Client-auth-token trigger boundary (2026-11-04)

**Confirmed static, APK43.72:** GCM map key pn maps to the push-consumer
argument labelled pushNonce. The manager call is guarded by its nonnull check;
full v31 control-flow preservation and trusted delivery provenance remain open.
0AI.A0O is exactly state==2; traced transitions write3 (with server-disconnected
diagnostic) and4. The state2 producer and complete eligibility remain unknown.
[Checked exports, caller inventory and limits](apk/2.26.43.72-client-auth-token-trigger/README.md).
No sender, retry authority, production behavior change or live traffic.

### Client-auth-token readiness producer (2026-11-04)

**Confirmed static, APK43.72:** 0AI.A0M writes state2, opens the condition
variable and invokes the selector2 observer notifier. Both exact callers are
in the connected handler, on distinct branches; state2 does not prove every
connected-path side effect completed. The notifier invokes BzL; the token
manager implementation drains one replaceable in-memory pending slot and
rechecks readiness. Registration, scheduling, instance identity and concurrent
lifecycle ordering remain untraced.
[Full selected bodies and limits](apk/2.26.43.72-client-auth-token-readiness/README.md).
No runtime sender, durable recovery claim, new replay authority or live traffic.

### Client-auth-token observer topology (2026-11-04)

**Confirmed static, APK43.72:** readiness dispatcher separates inline ordinary
observers from marker-dependent inline/scheduled paths. The state constructor
uses provider-backed combined observer sets; explicit registration search alone
cannot establish membership. A separate connection-thread pending-slot drain
precedes a conditional rotation invocation in its selected block. Membership,
dispatch category, provider identity and cross-thread ordering remain open.
[Selected bodies, excerpt, searches and limits](apk/2.26.43.72-client-auth-token-observer/README.md).
No Rust behavior change, sender, new retry authority or live traffic.

### Client-auth-token observer membership and delivery (2026-11-04)

**Confirmed static, APK43.72:** readiness set binding0x1d2c includes0x83,
which resolves to the token-manager factory through the global cached resolver.
The combined set preserves per-member resolvers into observer iteration. DEX
interface declarations place1jA in07v, so the null-hint readiness notification
enters scheduled dispatch. The task re-enumerates observers before invoking
the notifier; token-manager delivery still rechecks readiness.
[Bindings, declarations, dispatch and limits](apk/2.26.43.72-client-auth-token-membership/README.md).
Membership/category gaps from the prior trace are closed for this selected
path. Executor completion/order, cache resets, generation binding, trusted
push origin and failure/backoff remain open. No runtime or live change.

### Client-auth-token nonce register preservation (2026-11-04)

**Confirmed static, APK43.72:** a digest-bound conservative CFG checker
finds only the original v69-to-v31 copy reaches the manager argument move.
The later v31 reuses cannot reach it without another definition; the sole
call chain is guarded by the original nonce's nonnull check. The previously
retained method was regenerated from the pinned APK and matched byte-for-byte.
[Reproducible checker, results and limits](apk/2.26.43.72-client-auth-token-nonce-flow/README.md).
This closes the register-preservation gap, not trusted push origin, freshness,
complete eligibility, failure/backoff or durable recovery. No runtime change,
sender, new retry authority or live traffic.

### Client-auth-token delivery failure versus IQ error (2026-11-04)

**Confirmed static, APK43.72:** FUT's token selector restores the original
nonce on BsS only via compareAndSet(null,nonce), preserving newer pending
input. Bvj only logs in that selector; client-config selector0 behavior does
not apply. Neither branch immediately sends or dispatches held requests.
Submit-time A0U requires ready flag A06 plus state2, beyond the manager's first
state2 check; failure calls BsS directly, while success registers before dispatch.
[Callbacks, helper, submit chain and outcome limits](apk/2.26.43.72-client-auth-token-failure/README.md).
Timer expiry shares BsS restoration, not proof of rejection or safe replay.
Global backoff, shutdown/callback races and durable recovery remain open.
No Rust sender, runtime change, new retry authority or live traffic.

### IQ pre-write promotion and shutdown boundaries (2026-11-04)

**Confirmed static, APK43.72:** writer C0e promotes callbacks fromA0I toA0H
before its node-write entry point. The concrete0uw binding is traced through
writer initialization. A0H membership is not proof of wire delivery. Callback
presence check and promotion are separate, leaving a possible timeout
interleaving; removal alone cannot establish definite non-delivery. Local
drops post a failure path; shutdown case3 snapshots/clearsA0H and schedules
BsS, without clearingA0I in its selected IQ block.
[Complete selected bodies, ordering and limits](apk/2.26.43.72-iq-promotion-shutdown/README.md).
Full socket/reconnect behavior, shutdown eligibility, concurrent and durable
ownership remain open. No Rust behavior change, new replay authority or live traffic.

### Partial encrypted comparison write regression (2026-11-04)

**Observed offline local fixture:** the verified classic-Noise handshake test
now exercises `messaging::write_and_wait_captured` with a real encoded IQ,
encrypted record and framed AsyncWrite over a local Tokio duplex stream. The
peer reads the three-byte record header and64 ciphertext bytes, then drops
the stream. The advertised record exceeds the8192-byte duplex capacity, so
the whole record cannot be buffered before disconnect. The helper returns
BrokenPipe; a second attempt on that consumed channel returns Closed. A
synthetic outcome_unknown checkpoint is unchanged after encrypted-store reopen.

Test: `transport::handshake::tests::partial_encrypted_comparison_write_keeps_durable_uncertainty_and_consumes_channel`.
The fixture synchronizes after handshake-finish consumption to avoid hiding
record bytes inside the peer's frame-reader buffer. It uses a valid local
certificate chain, not disabled certificate verification.

This adds a partial-wire cut to the existing callback-level failure and raw
framing/cancellation checks. It supports the [native pre-write promotion
boundary](apk/2.26.43.72-iq-promotion-shutdown/README.md): registry admission
and an I/O error cannot establish definite non-delivery. The checkpoint is
a test fixture, not a production outbox reservation or complete reconnect
policy; this is local duplex I/O, not a TCP/live-server observation. No runtime
behavior, sender, retry authority or live account operation changes.

### IQ shutdown eligibility and cancellation-map ownership (2026-11-04)

**Confirmed static, APK43.72:** ordinary disconnect passes the original reason
and channel.BNJ (false when absent) into the registry flush. The selected
login-back failure branch instead gates the call on BNJ and passes(true,-1).
Concrete1A3.BNJ is queued-handler-message4 OR delegate.BNJ, not a connection
state alias. Adjacent0Sf.A0J invokes cancellation callbacks and clears a
separate map; it does not directly clear IQ maps or the pending token slot.
[Complete selected bodies, channel path and limits](apk/2.26.43.72-iq-shutdown-eligibility/README.md).
Delegate semantics, cancellation callback effects, full reconnect ordering and
durable ownership remain open. No Rust behavior change or live traffic.

### Concrete channel queued-work predicate and deferred drain (2026-11-04)

**Confirmed static, APK43.72:** selected writer readiness binds0vI into
0ZJ.A04. The channel predicate combines connection-handler message4, writer
message2 and a nonempty deferred deque. Those are queued-work checks, not
wire-delivery or executing-write completion proofs. Connection case0 clears
the writer flag and drains head Messages throughA00's existing gates; case1
sets the flag. Dequeue/drain does not prove retransmission or safe replay.
[Concrete binding, submit chain, drain and limits](apk/2.26.43.72-channel-queue-predicate/README.md).
Full reconnect ordering, races, replacement/generation ownership and durable
recovery remain open. No Rust behavior change or live account operation.

### IQ response ownership precedes token parsing (2026-11-04)

**Confirmed static, APK43.72:** response admission cancels the tracked timer
and looks upA0H; the ordinary token callback is later claimed by removal in
A02 before executor parsing. Failure/shutdown can win between those stages;
lookup or timer removal is not successful token installation. The selected
LV5 catch handles LIL's recoverable flag without reinsertion, BsS restoration
or held-request dispatch. Recoverable does not authorize network replay.
[Dispatch chain, declarations, exception routing and limits](apk/2.26.43.72-iq-response-ownership/README.md).
Full upstream admission, hooks, executor ordering and durable/generation
ownership remain open. No Rust behavior change or live account operation.

### IQ reader admission and unmatched fallback (2026-11-04)

**Confirmed static, APK43.72:** the selected1rD IQ processor reaches the
ordinary response bridge only for result/error with nonnull ID. get/set have
separate branches. A lower-level1ru map can process a result first; an
ordinary IQ-registry miss posts a fallback which can remove a separate
message callback and invokeBsT(null). A registry miss is not proof of global
nonprocessing. Declared iq tag mapping and processor-map registration are
traced; full injected/alternate bindings and callback effects remain open.
[Selected bodies, caller inventories and limits](apk/2.26.43.72-iq-reader-admission/README.md).
No Rust behavior change, new replay authority or live account operation.

### Shared lower-level IQ ownership and push-config producer (2026-11-04)

**Confirmed static, APK43.72:** the selected setup passes one new map to
reader1rD.A00 and writer1rj.A03. Push-config GET registers1rv before writing;
its result builds a configuration object and forwards it separately from the
ordinary IQ registry. The generic token node path has no explicit low-level
record insertion in the selected writer bodies. Alias escapes, ID collisions
and full Cyu behavior remain open; this is not a global absence claim.
[Shared-map setup, producer, result bridge and limits](apk/2.26.43.72-lower-level-iq-ownership/README.md).
Full configuration grammar/application, other families and durable/generation
ownership remain untraced. No Rust behavior change or live account operation.

### Push-config projection and selected consumer (2026-11-04)

**Confirmed static, APK43.72:** parsed config projects eight internal Bundle
keys: push tokens, mute state, account count, pkey and VOIP payload type.
Long-to-String and per-field holder overwrite behavior are traced; the keys
are not proven wire attribute names. Selected type6 consumer forwards seven
Strings to1uj.A04 and handles FBNS separately via conditional1uo.A00.
[Projection, dispatch/consumer bodies and limits](apk/2.26.43.72-push-config-projection/README.md).
Full wire grammar, injected handler binding, downstream effects and durable
reconciliation remain open. No Rust behavior change or live account operation.

### Push-config verification work and numeric defaults (2026-11-04)

**Confirmed static, APK43.72:**1uj.A04 builds VERIFY work and omits null/empty
String extras. It puts account count/VOIP ints with parse fallbacks1/0; the
helper uses Java Integer.parseInt without explicit trim/clamp. Enqueue failure
can be logged with normal return, so parsing/consumer return does not establish
work completion or corrective send. ServiceA05 separately compares state and
applies additional numeric/helper gates before invokingEbv.A00.
[Projection, callers, selected gates and limits](apk/2.26.43.72-push-config-verification-work/README.md).
Complete token/provider lifecycle, comparisons, sender outcomes and durable
ownership remain open. No Rust behavior change or live account operation.

### Corrective push-config construction and local admission (2026-11-05)

**Confirmed static, APK43.72:** Ebv emits server iq/set with one config child;
builderA04 merges same-tag fragments instead of appending another wrapper.
Provider-token config ID and generated IQ ID are distinct. A0W/A0A requires
ready flag/channel presence, logs connection state without gating on it,
registers before dispatch and can permit deferred offline writer queueing.
Its boolean is local admission. Config FUT selector0 differs from token
persistence/recovery and its immediate-failure path.
[Sender, helper composition, validators and limits](apk/2.26.43.72-push-config-corrective-submit/README.md).
Full attribute/trigger policy, helper effects, races and durable/generation
ownership remain open. No Rust behavior change or live account operation.

### Push-config reply collection and variant choice (2026-11-05)

**Confirmed static, APK43.72:**7Ow bounds successfully converted config wrappers
at0..2, rather than counting every raw config child.8B6.A0K returns the first
nonnull variant or null after all candidates fail; the constructor skips null
conversions. Converter exceptions are not caught by the selected helpers.
[Complete selected methods and limits](apk/2.26.43.72-push-config-reply-collection/README.md).
Individual variant grammar, thrown failures, duplicate semantics and durable
reconciliation remain open. No Rust behavior change or live account operation.

### Android push-config wire fields and null gates (2026-11-05)

**Confirmed static, APK43.72:**8TD requires platform adm/fbns/gcm and token
id String length1..1024 to construct its Android variant. Optional mute/count/
pkey/VOIP results may be null; appid/settings parse results are discarded.
Item/hash parsing bounds converted entries at0..1 and nullable item failure
does not itself reject the Android variant.
[Field mapping, exact helpers and limits](apk/2.26.43.72-push-config-android-grammar/README.md).
Other variants, header/Long lexical details, provenance and durable reconciliation
remain open. No Rust behavior change or live account operation.

### Push-config alternate variants and Long syntax (2026-11-05)

**Confirmed static, APK43.72:** the five non-Android converter types have
selected admission gates and still count toward the reply collection bound,
even though their visitors do not update Android projection holders. Long
conversion handles optional sign, leading zeros and overflow without trimming;
Character.digit delegates digit acceptance to the Android runtime.
[Selected converter gates, complete exports and limits](apk/2.26.43.72-push-config-variant-admission/README.md).
Full optional/Apple item semantics, Unicode tables, header validation, provenance
and durable reconciliation remain open. No Rust behavior or live traffic changed.

### Push-config GET startup ordering and404 work (2026-11-05)

**Confirmed static, APK43.72:** the sole direct GET producer caller is in the
selected startup method after token drain/conditional rotation. Lower-level
error ownership is removed before code parsing and can emit more than one
callback for multiple error children. Its type27 bridge reaches selected404
handling that queues VERIFY with seven null String inputs, not a direct GET
retry. [Caller, excerpt, error bodies and limits](apk/2.26.43.72-push-config-trigger-error/README.md).
Full startup eligibility, provider work, timeouts and durable recovery remain
open. No Rust behavior or live account operation changed.

### FBNS local/server comparison and provider entry (2026-11-05)

**Confirmed static, APK43.72:**1uo suppresses refresh only for a nonempty matching
server/local token and matching saved app version, behind its eligibility gate.
It updates last_server_fbns_token before refresh; null removes that server view,
not the local token in this body. Refresh selects an accepted Android provider
and requests registration through explicit service IPC, not a direct IQ resend.
[Complete methods and limits](apk/2.26.43.72-push-config-fbns-reconciliation/README.md).
Provider callback/authenticity, token persistence, later SET and durable recovery
remain open. No Rust behavior, provider IPC or live account operation changed.

### FBNS token receipt and optimistic server marker (2026-11-05)

**Confirmed static, APK43.72:** registered-token worker validates the provider
PendingIntent creator, eligibility and nonempty token before its state path.
When the token differs from last_server_fbns_token it invokes config SET,
discards the admission boolean and writes that marker on normal return before
local-token comparison/save. The marker is not exclusively server-observed state.
[Receiver, explicit excerpts, helpers and limits](apk/2.26.43.72-push-config-fbns-token-received/README.md).
Full provider/generation authority, ACKs and durable recovery remain open.
No Rust behavior, provider IPC or live account operation changed.

### FBNS provider message ACK continuation (2026-11-05)

**Confirmed static, APK43.72:** the selected FBNS worker reaches provider ACK
after normal message handling, empty data or its JSONException catch. Nonempty
sender/notification ID and sender allowlist gate an authenticated Android
broadcast with processor_completed=true. This does not prove payload success,
durable processing or WhatsApp stanza acknowledgement.
[Branch offsets, error continuation and limits](apk/2.26.43.72-push-config-fbns-token-received/README.md#provider-message-ack-after-normal-handling-or-json-failure).
Full downstream processing, provider delivery and durable ACK ownership remain
open. No Rust behavior, provider IPC or live account operation changed.

### Unified-session startup node and cached ID (2026-11-05)

**Confirmed static, APK43.72:** the node immediately after config GET in the
selected startup block is ib/unified_session, not another IQ. Its String ID is
cached/generated from an offset-aware clock modulo a week, with separate refresh
and identity-observer producers. Refresh submission uses family447 and its local
boolean does not establish delivery.
[Exact methods, callers and limits](apk/2.26.43.72-unified-session-startup/README.md).
Full eligibility, clock authority, cadence, writer/ACK and durable recovery remain
open. No Rust behavior or live account operation changed.

### Unified-session eligibility is Me-record presence (2026-11-05)

**Confirmed static, APK43.72:**0Sb's global provider binds09G, whose BUx delegates
to09O: lazy-load then test nonnull Me record. That predicate is not socket
readiness or current authenticated-session proof. The cached unified ID fast path
can return before rechecking it; explicit refresh and identity observer are separate.
[Provider route, metadata, exact predicate and limits](apk/2.26.43.72-unified-session-eligibility/README.md).
Full provider/load authority, generation fencing, writer/ACK policy and durable
recovery remain open. No Rust behavior or live account operation changed.

### Unified-session envelope and writer queue (2026-11-05)

**Confirmed static, APK43.72:** logical kind447 travels in arg2 of generic
arg1=6; normalization returns no replacement for this envelope. The selected
writer skips IQ promotion, can defer offline and calls the generic node writer.
IOException requeues locally; neither queueing nor local return proves delivery
or durable replay authority.
[Helpers, exact branch offsets and limits](apk/2.26.43.72-unified-session-writer/README.md).
Full Cyu/reconnect, server ACK policy and generation recovery remain open.
No Rust behavior or live account operation changed.

### Conditional startup availability and bounded inbound search (2026-11-05)

**Confirmed static, APK43.72:** after config/unified-session, selected startup
conditionally calls the bare presence/type=available writer. Rust shape agrees;
full availability/mode/control-flow policy remains unproven. Exact unified_session
literal search finds only its two outbound builders, not proof of absent ACKs.
[Complete writer, selected gates, query and limits](apk/2.26.43.72-startup-availability/README.md).
Full caller/companion/passive semantics, response policy and generation recovery
remain open. No Rust behavior or live account operation changed.

### Post-availability session hooks and metrics handoff (2026-11-05)

**Confirmed static, APK43.72:** selected startup records session ID, starts
conditional offline-resume metrics and records uptime. Persisted metric bytes
are removed before parsing/handoff; prior incomplete metric entries can be
removed before sink invocation. This is metrics state, not proof of application
completion, mandatory telemetry or request replay authority.
[Complete methods, explicit worker excerpts and limits](apk/2.26.43.72-startup-session-hooks/README.md).
Full sink transport/sampling, offline-complete semantics and durable/generation
recovery remain open. No Rust behavior or live account operation changed.

### Startup metrics sink selection and sampling (2026-11-05)

**Confirmed static, APK43.72:** metrics provider959 resolves to a factory selecting
0NY,KJu orKJv by configuration. PrimaryCSJ can drop via nullable sampling result;
accepted events reach another conditional local queue. Routing may target primary,
alternate or both, not a universal per-connect network write.
[Factory route, complete entries, metadata and limits](apk/2.26.43.72-startup-metrics-sink/README.md).
Full predicates, event authority, queue/transport, responses and durable generation
recovery remain open. No Rust behavior or live account operation changed.

### Primary metrics worker reaches a native boundary (2026-11-05)

**Confirmed static, APK43.72:** selected worker14 can enqueue validation diagnostics
and still continue the original event. It awaits a latch (interruption returns
normally after reinterrupt), maps channels and calls JniBridge with event fields.
The return is discarded; native/network delivery remains unproven.
[Constructor, selected worker, wait and limits](apk/2.26.43.72-startup-metrics-native-handoff/README.md).
Native serialization/transport, alternate sink, durability and generation ownership
remain open. No Rust behavior or live account operation changed.

### Metrics latch release and activation scheduling (2026-11-05)

**Confirmed static, APK43.72:** static CountDownLatch(1) status is opened by a
separately queuedI9Q selector3, after initialization submissions inALK. The
activation flag is per-instance; initialization native return is discarded.
Count zero is not by itself proof every instance's native setup succeeded.
[Selected activation, release, status and native boundary](apk/2.26.43.72-metrics-readiness-latch/README.md).
Executor failure/order, activation/source authority, native transport and durable
generation recovery remain open. No Rust behavior or live account operation changed.

### Metrics FIFO advances after exceptional completion (2026-11-05)

**Confirmed static, APK43.72:** reverified098/1VR FIFO dispatches the next task
after either task return or catch-all cleanup. Initialization47 precedes release3
on the same queue, but exceptional completion can still advance toward latch
release. An open latch therefore remains distinct from native success.
[Full executor links and scoped failure analysis](apk/2.26.43.72-metrics-readiness-latch/README.md#fifo-completion-advances-even-after-an-initialization-exception).
Underlying dispatch failure, middle task, activation authority and native transport
remain open. No Rust behavior or live account operation changed.

### Metrics native carrier and provider boundary (2026-11-05)

**Confirmed static, APK43.72:** exact event/setup Java methods are native. The
selected ARM64 libs.so exposes superpack archive coordinates, not named JNI
dispatchers in its six-entry dynamic-symbol table. Carrier digest and differing
file/virtual offsets are retained; inner code is not yet decoded. Current REA
native providers are unavailable under their configured paths.
[Declarations, carrier metadata, readelf and scope](apk/2.26.43.72-metrics-native-carrier/README.md).
Unpacking, JNI registration, native transport and durable recovery remain open.
No Rust behavior or live account operation changed.

### Superpack loader and bounded XZ reconstruction probe (2026-11-05)

**Confirmed static, APK43.72:** loader/API bodies connect to the packaged
superpack decoder. A digest-bound probe finds140 XZ magic candidates and
reproduces successful bounded decoding of the first12. None starts with ELF;
these outputs are partial payloads, not recovered native libraries.
[Complete loader/API, reproducible probe and limits](apk/2.26.43.72-superpack-unpack-boundary/README.md).
Archive transforms, member identity, JNI registration and native transport/recovery
remain open. No Rust behavior or live account operation changed.

### Superpack decoder usability is a narrow JNI probe (2026-11-05)

**Confirmed static, APK43.72:** decoder member/export bytes pinned. The56-byte
usability export constructs47 77 53 and passes them through one JNI-table call;
it does not exercise archive decompression. Host JNI layout supports setter
identity as an inference, not observed Android execution. Openbox Java wrapper
exposes file-range/reader-pointer boundaries, not a restored container schema.
[Exact assembly, symbols, API and limits](apk/2.26.43.72-superpack-decoder-entry/README.md).
Native registration/transforms, complete inner libraries and metrics transport
remain open. No Rust behavior or live account operation changed.

### Superpack relocated JNI entries mapped (2026-11-05)

**Confirmed static / derived, APK43.72:** selected registration call sites and
RELATIVE-relocated tables connect8 AssetDecompressor and5 Openbox methods to
native image addresses. All13 full signatures match retained DEX declarations;
host JNI layout checks support the registration interpretation. This is not
executed registration or recovered inner native code.
[Reproducible table resolver, relocations and bounded assembly](apk/2.26.43.72-superpack-jni-registration/README.md).
Container transforms, complete libraries and metrics transport/recovery remain
open. No Rust behavior or live account operation changed.

### Superpack carrier loading and decode-core handoff (2026-11-05)

**Confirmed static / derived, APK43.72:** mapped from-so wrapper supplies default
ranges, resolves exact carrier archive symbols and passes end-start to0xa964
under the spo storage branch. Missing symbols/error and cached-buffer paths
remain distinct; the core format and complete recovery are not yet reconstructed.
[Bounded assembly, byte digests, exact Strings and limits](apk/2.26.43.72-superpack-from-so/README.md).
Full transforms, inner libraries and metrics network/durable behavior remain
open. No Rust behavior or live account operation changed.

### Superpack member selection and disk-output boundary (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** core 0xa964 iterates members,
selects ordinal start/exclusive-end (end -1 means exhaustion), writes selected
members and returns their names. The selected writer checks fwrite length but
ignores fclose result; this body has no fsync/rename or prior-file rollback.
[Complete selected core/writer windows, ABI probe and limits](apk/2.26.43.72-superpack-member-output/README.md).
Parser transforms, usable complete libraries and native metrics transport remain
unverified. No Rust behavior or live account operation changed.

### Superpack structured native-member inventory (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** traced header and metadata parser
yields 83 records, including libwhatsapp.so (ordinal 63) and whatsappmerged
(64). Raw/effective type, declared size, opaque second word and offsets retained;
metadata ends at archive offset 0xc73. This is not decoded ELF recovery.
[Digest-bound inspector, full inventory and parser assembly](apk/2.26.43.72-superpack-metadata/README.md).
The spo transform, complete members and native metrics network behavior remain
unverified. No runtime behavior or live account operation changed.

### Superpack spo transform dispatch and complete block framing (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** relocated spo descriptor and
42-byte block headers yield 213 framed records ending exactly at archive length.
140 kind-1 blocks start with XZ magic; 73 kind-42 blocks call a separate decoder
0x218bc. The old XZ-only probe did not reconstruct complete native libraries.
[Digest-bound descriptor/block inspector and full reader/worker windows](apk/2.26.43.72-superpack-transform-blocks/README.md).
Decoded stitching, complete ELF recovery and metrics network behavior remain
unverified. No runtime behavior or live account operation changed.

### Type-42 native decoder bounded emulation (2026-11-05)

**Observed in Unicorn emulator / static entry mapping, APK 2.26.43.72:** selected
type-42 blocks 0, 168 and 201 return declared output lengths; ordinal 0 reproduced
byte-for-byte and 201 agreed across two memory-helper implementations. This
executes selected target instructions in memory, not an Android app or account.
[Digest-bound harness, results and execution limits](apk/2.26.43.72-superpack-kind42-emulation/README.md).
Candidates are not ELF images; member stitching/checksums, other type-42 blocks
and native metrics network behavior remain open. No Rust behavior/live traffic changed.

### Full Superpack block decode and so-driver boundary (2026-11-05)

**Observed offline / emulated plus static mapping, APK 2.26.43.72:** all 213
framed blocks produce declared lengths (140 XZ EOF-complete, 73 bounded ARM64
emulations), totaling 81747344 bytes. None starts with ELF magic; complete native
libraries remain unreconstructed. The registered so-driver resolves to 0x1af20,
then helper 0x1adb8 for the additional member-output stage.
[Full output manifest, 70 new emulation logs, driver assembly and limits](apk/2.26.43.72-superpack-all-blocks/README.md).
Native metrics network/persistence behavior remains open. No Rust/live change.

### All ARM64 packed native members reconstructed and metadata-checked (2026-11-05)

**Observed bounded emulation / independent offline checks, APK 2.26.43.72:** all
83 members reconstructed, 65766648 bytes. Every size and XXH64(seed=0) matches
pinned metadata; readelf layout checks pass. First 37 also match the alternative
sort-helper run. Recovered whatsapp/whatsappmerged are available for static JNI
and metrics analysis, not proven Android execution or wire correctness.
[Harness, full digest manifest, verification and limits](apk/2.26.43.72-superpack-reconstructed-members/README.md).
Native metrics behavior and broader network audit remain open. No Rust/live change.

### Recovered native metrics JNI event routing (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** exact relocated JNI tuples map
event/setup dispatch into recovered libwhatsapp.so. Event entry narrows longs,
keeps channel-dependent psId handling, visits typed fields, conditionally hands
to a state-based local manager and returns literal zero. Local append/scheduling
is not delivery or native persistence proof.
[Tuple resolver, complete selected native bodies, outlined flow and limits](apk/2.26.43.72-metrics-native-event-routing/README.md).
Setup initialization, flush/storage/transport and replies remain open. No Rust/live change.

### Native metrics pending drain, time enrichment and selected persistence (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** queued thunk reaches native pending
worker, which removes array index zero before processing. Channels 2/3 bucket a
clock-derived field 47 by 3600. Selected snapshot/file helpers reach fwrite and
fflush; flush return is ignored by the selected snapshot caller. Guard/caller
reachability, full durable format and actual transport remain unverified.
[Complete selected bodies, clock/thunk resolver and scoped limits](apk/2.26.43.72-metrics-native-drain-persistence/README.md).
No queue removal/stdio flush is promoted to delivery or crash durability. No Rust/live change.

### Native metrics metadata path, modes and recovery envelope (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** selected metadata path appends
/wam-metadata, uses rb recovery/wb writing, and rewinds before snapshot writes.
Reader checks 1692-byte MTD/version-4 envelope, matching writer header bytes;
a normal worker save passes the snapshot's zero guard. Termination and complete
validation/durability/transport semantics remain open.
[Mode/branch resolver, paired read/write evidence and limits](apk/2.26.43.72-metrics-native-metadata-file/README.md).
Local metrics metadata is not account/message replay authority. No Rust/live change.

### Native metrics snapshot range validation and replacement (2026-11-05)

**Confirmed static / observed synthetic emulation, APK 2.26.43.72:** local
validator checks four groups, with A/B nonnegative and C/D in 1..65535. Later
checks win within a group; the first failing group stops validation. All 43
bounded native fixtures pass. Recovery replaces rejected local snapshot state;
this is not server/account/message replay authority.
[Native bodies, exact ranges, synthetic harness/results and limits](apk/2.26.43.72-metrics-native-snapshot-validation/README.md).
Error objects, full manager lifecycle and event transport remain open. No Rust/live change.

### Native metrics integer payload encoding and selected field framing (2026-11-05)

**Confirmed static / observed synthetic emulation, APK 2.26.43.72:** native
integer helpers choose signed 1/2/4/8-byte little-endian payloads; scalar 0/1 use
special tags without bytes. All 42 boundary fixtures match independent expected
bytes/tags. Worker-to-field-encoder calls and selected header packing retained;
full field/event framing and send/reply behavior remain incomplete.
[Native bodies, digest-bound fixture harness and scope](apk/2.26.43.72-metrics-native-integer-encoding/README.md).
No new Rust telemetry or inferred mandatory request. No runtime/live change.

### Native metrics HTTP response ordering and task outcomes (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** native buffer sender constructs
HTTP-body request objects. Its response worker reads URL status and accepts the
ordinary 200/no-error branch; error/endpoint fallback precedes distinct 401 and
other failures. Regular task labels explicitly distinguish success, retryable,
non-retryable and failure; callback ownership/deletion remains incomplete.
[Complete selected transport bodies, labels/thunk resolver and scoped rules](apk/2.26.43.72-metrics-native-transport-responses/README.md).
These rules are not universal account/login retry authority. No Rust/live change.

### Native metrics request framing (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** imported POST method, five
header sources, four multipart sections and exact boundary/header bytes retained.
Host comes from selector 0x1a; access_token from selector 0x17. Endpoint index
zero is removed before construction. Credential/provider decoding, metadata
semantics and complete transport ownership remain open.
[Digest-bound literal/relocation resolver, helper bodies and limits](apk/2.26.43.72-metrics-native-request-builder/README.md).
No Rust runtime or live traffic changed; no mandatory telemetry or ban guarantee.

### Native metrics credential lifecycle (2026-11-05)

**Confirmed static / observed synthetic emulation, APK 2.26.43.72:** credential
admission checks use count/clock, consumes a use before construction, and can
promote a spare. Constructor raw lifetime increment172800/default count64,
nonnegative setter and refresh guards have17 bounded native fixture cases.
Mode-zero count180 override, transient cleanup and raw result switch retained
statically. These are metrics-local counters/results, not server/account limits.
[Native bodies, pinned resolver, fixtures and unresolved provider/crypto scope](apk/2.26.43.72-metrics-native-credential-lifecycle/README.md).
No Rust runtime or live operations changed; selector7 and completion binding
remain open.

### Metrics credential IQ producer and native callback mapping (2026-11-05)

**Confirmed static / derived, APK 2.26.43.72:** Java selector7 constructs
privatestats/sign_credential version1 with diagnostic32-byte blinded_credential bounds;
callback registration and32000ms delay precede dispatch. Selected not-ready
wrapper returns false without Java BsS. Hyg parses required integral t and
signed/public-key data plus optional config_id. Both selected failure methods
map through JNI selector4 to native raw1, not the Java diagnostic value3.
[Producer, parsing/queue evidence, native bridges and binding limits](apk/2.26.43.72-metrics-credential-iq/README.md).
Native method-cache initialization, crypto, full error/timeout dispatch and
operational Rust support remain open. No Rust runtime or live request added.

### Credential IQ timeout and malformed-response ownership (2026-11-05)

**Confirmed static / derived, APK43.72:** Hyg implements only0lG, so incoming
responses take ordinary handler2/claim-before-parse routing. Timeout handler4
removes queued/sent callback ownership and schedules BsS via8bj21, reaching
null-data/native result1. Required-field parse errors can consume callback
ownership without invoking that native completion; broader recovery is unknown.
[Callback-specific interface proof, shared dispatcher trace and scoped risk](apk/2.26.43.72-metrics-credential-iq/README.md#credential-callback-timeout-ownership-and-schema-failure-2026-11-05).
No Rust/live change or automatic retry authority; full generation/recovery
behavior remains unproven.

### Outbound metrics JNI cache binding closed statically (2026-11-05)

**Confirmed static / observed bounded synthetic emulation, APK43.72:**
JNI_OnLoad resolves JniBridge.jnidispatchIO(ILjava/lang/Object;)J at index0
into cache0xc467f0, consumed by the outbound static-long bridge. Five native-loop
fixtures verify success and early lookup failures. Earlier slots may remain
nonnull while the all-lookup flag is false; runtime initialization is unobserved.
[ELF symbol/tables, indexed-store proof, native fixtures and limits](apk/2.26.43.72-metrics-jni-method-cache/README.md).
This supersedes the prior static cache-binding unknown. Whole-loader recovery,
crypto and runtime availability remain open. No Rust/live change.

### Native metrics credential persistence (2026-11-05)

**Confirmed static / observed synthetic admission, APK43.72:** paired mode-
suffixed private credential file uses version2/25-byte header and three data
segments.23 bounded native header cases match independent parsing. Loader
overrides constructor defaults with saved expiry/count; header admission permits
zero lengths/trailing bytes and is not cryptographic validation. Save checks
write/close/count but uses wb; atomic durability remains unproven.
[Paired native bodies, raw schema, fixtures and failure-branch limits](apk/2.26.43.72-metrics-native-credential-file/README.md).
Observer callback is persistence, not endpoint production. Endpoints, full load
scheduling, crypto and runtime behavior remain open. No Rust/live change.

### Native metrics endpoint production and finite-list fallback (2026-11-05)

**Confirmed static / observed bounded synthetic append/store, APK43.72:**
scheduler replaces one task's URL list with accepted provider entries in order
then native selector22 fallback. JNI index26 binds Java selector0 provider; its
selected prefix/suffix decode to https:// and /deidentified_telemetry. Seven
native cases verify append/store ordering with controlled URL results. Sender
consumes index0 before construction; selected response fallback reuses the list.
[Producer conditions, native bodies, fixtures and remaining scope](apk/2.26.43.72-metrics-native-endpoints/README.md).
Actual dynamic/default endpoint values, generic URL policy, full retry timing
and generation fencing remain open. No Rust runtime or live operation added.

### Native request analytics tags and metadata sources (2026-11-05)

**Confirmed static / observed synthetic branch selection, APK43.72:** selected
sender purpose1 maps to prefetch and category4 to analytics; empty surface/
application fields omit.24 bounded cases verify native enum/pair selection
with append/zero-format adapters, not full JSON serialization. Separate
meta_data rt depends on channel3; optional config_id derives from credential
data, not Host. Raw clock/mode semantics and actual product remain unknown.
[Descriptor, tables, formatter scope, metadata provenance and fixtures](apk/2.26.43.72-metrics-native-request-tags/README.md).
No Rust runtime, invented telemetry or live requests added. General formatting,
selector plaintext and crypto remain open.

### Complete selected native analytics serialization (2026-11-05)

**Observed bounded synthetic emulation, APK43.72:**19 additional cases run the
complete native tag builder/pair/retry code with explicit allocation/strlen and
host snprintf adapters reading native va_list. Exact bytes match independent
expected JSON or an intentional invalid-quote fixture. Retry text truncates to
four characters; raw object insertion and no quote escaping are corroborated.
[Full-builder harness, exact outputs, ABI windows and limits](apk/2.26.43.72-metrics-native-request-tags/README.md#complete-selected-serialization-fixtures-2026-11-05).
This extends the earlier24 branch-only cases. No Android libc, real product,
allocator-failure behavior, live traffic or Rust telemetry implementation proved.

### Native metrics connection-status repeating timer (2026-11-05)

**Confirmed static / observed bounded synthetic execution, APK43.72:** body
72ab10 is connection-status handling, correcting a provisional load-completion
label. Ready0 schedules tasks and starts interval30 timer; unavailable1 cancels
then signals error. Fifteen native arm/event/cancel/callback fixtures verify
repeating/cancellation and readiness gates with explicit queue/mutex adapters.
Not-ready callback does not itself stop generic timer repetition.
[Native bodies, corrected scope, fixtures and concurrency limits](apk/2.26.43.72-metrics-native-recovery-timer/README.md).
Rust IQ review did not establish a universal from rule; no guessed guard/retry
was added. Status producers/generation/concurrency and live behavior remain open.

### Native metrics task cancellation and destructor ordering (2026-11-05)

**Confirmed static / observed bounded synthetic execution, APK43.72:**
cancel helper issues handle cancels and optional table+0x28 callbacks before
clearing active tasks/provider/network. Endpoint map is separate; absent-network
path does not clear busy byte. Destructor clears callback table first, so helper
does not enqueue those callbacks during teardown.12 native cases verify exact
synthetic cancel arguments, queued task keys and cleanup/invariant branches.
[Native bodies, direct callers, fixtures and delivery/ownership limits](apk/2.26.43.72-metrics-native-task-cancellation/README.md).
Real cancellation, queued callback completion and durable replay remain open.
No Rust runtime or live operations changed.

### Native regular drain status versus task outcome (2026-11-05)

**Confirmed static / observed bounded synthetic execution, APK43.72:** helper
7273a8 ends active drain once, optionally creating local error3; it is not
backoff or persistent buffer mutation. Fourteen native helper cases verify
flag/argument/optional-table/repeated-call behavior. Task outcomes separately
retain retryable/failure entries or remove success/non-retryable entries.
[Start/end bodies, helper fixtures and ownership distinctions](apk/2.26.43.72-metrics-native-drain-outcomes/README.md).
Upper callback/network job binding, backoff, generation and durable ownership
remain open. No Rust runtime or live operation changed.

### Regular metrics w:stats IQ and diagnostic-bound correction (2026-11-05)

**Confirmed static, APK43.72:** regular native job calls Java selector6, builds
iq/set/xmlns=w:stats/add binary payload, registers32000ms callback before send
and waits through FfK/T54. Exact result returns true; inhibition/error/wait
failures return false, mapped to native success0/failure3. Callback-backed wait
is distinct from hard end-to-end deadline and durable delivery.
**Correction:**00L.A0C logs AssertionError and returns. A04 bounds are diagnostic,
not hard writer rejection; prior credential exact32-byte guard wording corrected.
[Native/Java contract, callback interfaces and diagnostic source](apk/2.26.43.72-metrics-regular-iq/README.md).
No Rust runtime or live request added. Full buffer/retry/durable ownership
remains open.

## Old counters no longer request retries — 2026-11-05

**Confirmed static:** [ED4 old-counter producers and -1001 mapping](apk/2.26.43.72-old-counter-no-retry/README.md)
close one typed cause: selected native retry eligibility excludes this status.
**Inferred compatibility mapping / offline implementation:** libsignal's
DuplicatedMessage now survives as Error::SignalOldMessageCounter. Direct and
pairwise SKDM inbox handling preserves capture and continues as Unsupported,
without retry or delivery ACK. Durable authenticated same-ID duplicates keep
the existing Duplicate result. No counter/address/backend data enters the error.

Other retry error codes, native old-counter confirmation policy, sender-key
primitive classification and full route/key/count policy remain open. Existing
group crypto error formatting is also still an uncorrected redaction gap; no
primitive equivalence is inferred from direct cipher evidence. No live request.

## Group counter and producer-specific errors — 2026-11-05

**Confirmed static:** [sender-key ED4/-1001 and key-error producer paths](apk/2.26.43.72-group-counter-error-paths/README.md).
Sender-key old counters use the same excluded retry status as direct messages.
**Inferred mapping, implemented offline:** group DuplicatedMessage now retains
SignalOldMessageCounter, preserving capture without retry, delivery ACK or state
commit. Group provider diagnostics now use fixed variant-only messages. Durable
authenticated duplicates retain their prior confirmation behavior.

EDR identifies missing/invalid local prekeys in F22.A04, but group Eyj.A01 wraps
EDR as EDY before result classification. Session builder signature failures
also use a different exception from the exact “Invalid signature!” refinement.
Global exception/error-name -> wire-code mappings would lose these distinctions.
No other retry wire-code policy changed. Full native ACK policy and remaining
producer/primitive/key/count/routing parity remain open; no live request.

## Direct coordinator attribution and cause aggregation — 2026-11-05

**Correction / confirmed static:**0WJ.A0P is fast-ratchet group decrypt, not
ordinary direct decrypt. Ordinary direct0WJ.A0N -> F22.A05/A01/A02 independently
confirms ED4 -> -1001, supporting the already implemented no-retry behavior.
[Selected methods, missing-session chain and signature producer](apk/2.26.43.72-direct-retry-cause-paths/README.md).

F22.A05 missing session throws EDS -> A0N -1008 -> wire1. F22.A01 aggregates
EDY failures into EDY(List), whose outer No valid sessions. message refines to
wire8; raw BadMac/future-counter mappings cannot ignore this wrapper. Sender-key
Fj2.A00 exact Invalid signature! directly throws EDY, supporting selected group
wire5; bundle key signatures use a separate producer. No new wire code/runtime
behavior added. Remaining cause propagation, provider equivalence and full
retry/ACK policy are open. No live account operation.

## Missing-session retry error preserved — 2026-11-05

**Confirmed static:** [ordinary direct parser/session order and -1008 -> wire1](apk/2.26.43.72-direct-retry-cause-paths/README.md),
and [group absent sender-key EDS -> -1008](apk/2.26.43.72-group-counter-error-paths/README.md).
A0N constructs Fj3 beforeF22.A05 tests session presence. Eyj.A01 checks empty
sender-key record before parsing its message. These orderings differ.

**Implemented / offline tested:** IncomingResult::MissingSession preserves the
specific absence through normal receive. Direct requires no saved session and
successful SignalMessage syntax admission; a present session with missing
identity remains generic Undecryptable. No sender-key record yields MissingSession
on the group path. Normal receive queues retry with error=1 using the existing
count1/original timestamp and durable selector/reservation. Malformed direct
input does not inherit this cause. Syntax admission does not authenticate MAC
or plaintext without a session.

The existing public low-level queue_retry_for_incoming and retry_receipt helpers
retain their unclassified behavior; callers without cause evidence do not gain
a guessed default. Private construction accepts the confirmed code from normal
receive. Existing reservation lookup keeps old held/uncertain/acknowledged wire
unchanged; introducing a cause never resets retry count or creates a second
attempt. Offline CLI replay recognizes the new outcome and sends nothing.

Production-loop regression uses a structurally valid synthetic Signal message
for the direct absent-session case and absent group key for skmsg; asserts
error1/original timestamp, pending reservation before failed write, and no repeat
or delivery ACK after reconnect/redelivery. This does not prove server acceptance.
Generic failures still lack full typed eligibility/error classification, and
native key/count/routing/ACK parity remains unfinished. No live account traffic.

## Version gates and no-retry limits — 2026-11-05

**Confirmed static:** [selected native version gates and backend comparison](apk/2.26.43.72-message-version-error-paths/README.md).
ED6 is used for legacy versions by four selected message constructors; it does
not establish identity-refusal semantics. Direct Fj3 admits version nibble2..4,
while the current backend admits3..4. Full version2 crypto support is not proven.
Prekey gate3..4 and sender-key gates exactly3 have different high-version exception
paths. Ordinary direct/group ED6 -> -1007 is excluded, while EDY -> -1005 is eligible.
Prekey outer construction has a separate catch region; later ED6 handlers cannot
be applied indiscriminately to it. These differences rule out a blanket backend
unsupported-version -> no-retry mapping. Identity policy, full range/primitive
parity and producer-specific retry causes remain open. Research only; no new
wire code, runtime behavior or live request.

## Legacy sender-key message retry exclusion — 2026-11-05

**Confirmed static:** [Fj2 legacy version gate -> ED6 and selected -1007 exclusion](apk/2.26.43.72-message-version-error-paths/README.md),
with [0Yl.A00 group coordinator catch table](apk/2.26.43.72-group-counter-error-paths/README.md).
**Inferred provider mapping / implemented offline:** only receive_group's skmsg
decryption maps backend LegacyCiphertextVersion to the redacted typed error
SignalLegacySenderKeyMessage. Group inbox preserves capture and continues as
Unsupported, with no automatic retry or delivery ACK and no ratchet/event commit.
This neither authenticates the input nor creates a durable duplicate marker.

The classification stays scoped to sender-key message decrypt. SKDM ingestion,
ordinary/prekey direct parsing, newer unknown versions and short ciphertext
retain their existing policy; their full native result semantics are not inferred
from the legacy skmsg branch. Missing sender-key record is still classified
before message parsing and uses the existing MissingSession/error1 path.

Offline inbox regression mutates real synthetic sender-key ciphertext to version
nibbles0/1/2 under fresh IDs and verifies Unsupported with unchanged journal and
sender keys. Version4 and a64-byte version2 payload remain Undecryptable, showing
that newer-version and length admission are not swept into this exclusion.
Authenticated durable duplicates, including consumed content, remain Duplicate.
Other producer-specific retry code/key/count/routing/ACK parity remains unfinished.
No live account operation or server acceptance claimed.

## Sender-key signature retry error preserved — 2026-11-05

**Confirmed static / inferred provider equivalence:** [message signature verifier,
status refinement, wire5 and backend source identity](apk/2.26.43.72-sender-key-signature-retry/README.md).
Only skmsg decryption maps SignatureValidationFailed to the fixed redacted
Error::SignalInvalidSenderKeySignature. Inbox returns InvalidSenderKeySignature;
normal receive queues retry/error5. The private RetryCause enum now distinguishes
missing session (wire1) from sender-key message signature (wire5), with no guessed
generic fallback. Other signature producers are unchanged.

Production-loop regression tampers a real synthetic group ciphertext signature,
checks error5, original timestamp and reservation before failed write; reconnect
redelivery emits neither another retry nor delivery ACK. Sender-key rows remain
unchanged. Existing direct/group missing-session cases retain error1. Offline CLI
replay sends nothing. Count/key progression, other typed causes, full routing/ACK
policy and server acceptance remain unverified. No live account operation.

## Future-counter typed mapping prerequisites — 2026-11-05

**Confirmed static:** [sender-key signature-before-counter order, strict2000
boundary and backend limits](apk/2.26.43.72-sender-key-future-counter/README.md).
Native exact future-counter message refines to wire6. Backend also uses2000,
but returns generic InvalidMessage shared with CBC failures. Its selected-chain
accessor is private, front state may be wrong, and into_components advances the
front chain to its lease ceiling rather than passively exposing current position.
No provider-string matching or partial front-state mapping was added.

Native signed-int versus backend unsigned-counter comparison also differs at
high-bit inputs; this arithmetic inference prevents claiming universal cause
parity from a matching normal-range constant. Full typed cause/state access,
integer-domain compatibility, precedence and remaining retry/ACK policy are open.
Research only; no runtime behavior or live request changed.

## Sender-key future-counter retry error preserved — 2026-11-05

**Confirmed source / inferred provider equivalence / implemented offline:**
[effective matching-state metadata view and typed future-counter retry](apk/2.26.43.72-sender-key-future-retry/README.md).
Loaded-record serialization provides a passive view after reservation recovery,
without component-export advancement or front-state guessing. After backend
InvalidMessage(SenderKey), the adapter classifies only a matching-chain jump
strictly above2000 with both counters in native nonnegative signed-int range.
It uses typed fields, not provider strings, and zeroizes temporary record bytes.

The fixed typed error/outcome reaches normal retry/error6; signature precedence,
missing-session error1, signature error5, original timestamp and once-only durable
reservation remain intact. Actual backend tests cover2000/2001, retained older
chain, reservation ceiling, bad signature and high-bit nonclassification. The
production loop checks error6 and no replay/state change after uncertain write.
Offline CLI replay sends nothing. High-bit/native arithmetic equivalence, other
primitives and full retry key/count/routing/ACK parity remain open. No live traffic.

## Empty sender-key records and missing chains — 2026-11-05

**Confirmed static + platform contract / inferred equivalence:** [native empty
record versus wrapped missing-chain paths](apk/2.26.43.72-sender-key-chain-retry/README.md).
Empty record is checked before message parsing and selects error1. Unknown chain
inside a nonempty record constructs EDR then EDY(Throwable); Android's documented
class-prefixed cause-message behavior prevents the bare No keys prefix refinement,
retaining -1005 -> wire4. Full classes, platform references and a local OpenJDK
corroboration fixture are retained; no APK runtime observation is claimed.

receive_group now preserves empty-record precedence. Nonempty-record typed
NoSenderKeyState becomes SignalMissingSenderKeyChain / MissingSenderKeyChain and
normal retry/error4. Diagnostics are fixed/redacted. Production-loop fixtures
check error1 for malformed input with an empty record, error4 for a parsed unknown
chain, and unchanged keys/no repeated retry or delivery ACK after uncertain write.
Existing signature/future/missing-session cases retain5/6/1. Offline CLI replay
sends nothing. Other causes, count/key/routing/ACK parity and live acceptance
remain incomplete. No live account operation.

## Classic incoming local-prekey retry error and lazy lookup — 2026-11-05

**Confirmed static / inferred provider equivalence / implemented offline:**
[matching-session reuse, local EDR -> wire3, adapter ordering and tests](apk/2.26.43.72-incoming-local-prekey-retry/README.md).
Classic receive now decodes local prekey records only on backend key-store lookup,
so corrupt unused records do not block a matching-session follow-up. Actual
selected local key failures retain SignalIncomingLocalPrekey / IncomingLocalPrekey
through normal retry/error3, including classic pairwise group processing.

Real-crypto tests cover required missing/corrupt records and successful held-session
reuse with corrupt unused records. Normal-loop cases assert error3 and unchanged
prekeys/no repeated retry or delivery ACK after uncertain write. Three claimed
MAC fixtures were corrected to modify the nested MAC, preserving outer selectors.
Factory signed-material gates, storage failures, hybrid/KEM and later key-pair
integrity classification remain separate open work. Other typed causes/count/key/
routing/ACK parity remain incomplete; no live traffic.

## Newer unknown sender-key versions retain retry error4 — 2026-11-05

**Confirmed static / inferred provider equivalence / implemented offline:**
[version gate, cause refinement and backend identity](apk/2.26.43.72-sender-key-unknown-version/README.md).
Only skmsg version>3 typed rejection becomes SignalUnknownSenderKeyVersion /
UnknownSenderKeyVersion and normal retry/error4. Native Unknown version is not
EtY's Unknown type refinement. Lower legacy versions keep no-retry handling;
empty-record and short-input precedence remain intact. Other formats are unchanged.

Inbox tests cover high nibbles4/15, legacy0/1/2 and short2/4, with unchanged keys/
journal. The production-loop fixture checks error4 before signature verification,
original timestamp, durable uncertain reservation and no second retry/delivery ACK
on reconnect/redelivery. Offline CLI replay sends nothing. Native short-input,
other formats and full key/count/routing/ACK parity remain unfinished. No live traffic.

## Application-padding count and retry admission corrected — 2026-11-05

**Confirmed static / implemented offline:** [EHq unsigned count rule, retry
exclusion and separate CBC failure exploration](apk/2.26.43.72-application-padding-retry/README.md).
Receive now shares the native count-only rule0<count<plaintext length. It no longer
requires count<=16 or repeated padding bytes, and it rejects entire-buffer padding
rather than returning empty plaintext. Existing crypto authentication/block padding
and sender-side padding are unchanged.

SignalInvalidApplicationPadding covers empty/invalid decrypted application framing;
direct/group inbox continues as Unsupported without automatic retry, delivery ACK
or Rust atomic inbox/ratchet commit. Native Signal-store call order before EHq
trimming is retained as a separate durability/transaction tracing gap. Tests cover
unsigned counts through255, nonrepeating tails, actual authenticated ciphertext
and production-loop no-retry/no-confirmation/no-state-event behavior. CBC/provider
errors, full native commit/ACK policy and other causes remain open; no live traffic.

## One-shot action callbacks stay with their connection — 2026-11-05

**Confirmed static / inferred ownership / implemented offline:**
[selected sent-map flush, callback claim and Rust lifetime guard](apk/2.26.43.72-action-callback-lifetime/README.md).
Normal receive now requires private LiveReplies ownership from a successful local
action write on that drive. Exact result/error consumes the claim before durable
settlement; duplicates or a local failure cannot claim twice. Reconnect/reopen
never reconstructs callbacks from held Sent rows or numeric epochs.

Regression covers reserved-only rows, wrong kind/type/ID, result/error, duplicate,
local transactional failure and disk reopen. Production-loop failed-write then
reconnect leaves the old action Sent despite a late result, while the new action
settles normally. Public low-level/captured-reply reconciliation and separately
owned group membership waits remain unchanged. No replay/status reset, new route
or live account operation. Complete action deadlines, upstream/native lifecycle
branches and queued-callback policy remain open.

The same chunk rechecks the queue's saved-wire/row ID identity before bulk, single
and feature-owned reservation and before settlement. Malformed/mismatched stored
wire fails atomically instead of registering a callback under another request's
ID. Corruption regressions verify no row/status changes and no identifier echo.
A fast reply queued before local write completion still settles after success.

## AB-properties callback timer ordering — 2026-11-05

**Confirmed static / inferred race boundary:** current APK HBE.A00 passes
32000ms to0Sf.A0W/A0A; ready admission registers0ln.A08 before dispatch.
The positive-duration timer starts with queued ownership and is not restarted
byA0C promotion. Expiry handler4/A03 removes callback ownership; HyT selector2
BsS returns without scheduling a retry. Response lookup cancels the tracked
timer before its later response claim, without an independent age comparison.
An overdue timer is not proof that its callback has already expired.
[Complete producer/admission/expiry exports and limits](apk/2.26.43.72-abprops-callback-timer/README.md).
Rust's connection-scoped action owner still lacks per-action expiry. Porting
requires producer-specific timing and competing claims; a universal32s cutoff
or fresh post-write budget is unsupported. No runtime behavior or live traffic
changes in this research chunk.

## Initial AB-properties callback expiry ported — 2026-11-05

**Implemented offline / confirmed static timer:** the normal connection now
expires the initial abt/get/props protocol1 empty-hash callback32s from the
pre-write boundary, including local-write delay. A dedicated select branch
releases callback authority without changing the durable Sent row, replaying,
applying configuration or stopping receive. Response claims cancel the timer;
an overdue timer not yet dispatched remains a competing event, not a separate
reply-age predicate. Other action producers receive no invented default timeout.
[Producer, not-ready placeholder removal, implementation and regressions](apk/2.26.43.72-abprops-callback-timer/README.md).
Paused-clock unit and normal-driver tests cover expiry, reply cancellation,
late replies, write delay, unrelated IQs and continued receive. Rust serializes
expiry after awaited local writes; full Android concurrent scheduling, other
producer durations and configuration scheduling remain open. No live traffic.

## Block-list read/change callback timing corrected — 2026-11-05

**Confirmed static / implemented offline:** current APK1dH.A05 block/unblock
uses20000ms through0Sf.A0W/A0A;1dH.A0M fetch passes0 throughA0Q/A06,
which supplies no positive-duration replacement. The normal Rust loop now
expires block/unblock live callbacks20s from the pre-write boundary, retaining
Sent uncertainty without list/hash mutation, rollback or retry. Fetch receives
no invented20/32s timeout. Reply cancellation and expiry remain competing claims.
[Complete producer, failure callback, tests and limits](apk/2.26.43.72-blocklist-callback-timing/README.md).
Paused-clock tests cover19/20s, delayed write, result/error cancellation and
fetch reply at60s. Normal-driver delayed-write/late-reply tests now cover both
changes and initial AB-properties, continued receive and no resend/projection.
The native change failure callback reaches2KT.CE5(false); its concrete downstream
side effects remain open. Full native scheduler parity and other producers'
deadlines remain unfinished. No live account operation.

## Account-default disappearing timer callback expiry — 2026-11-05

**Confirmed static / implemented offline:** current APK ChangeDMSettingActivity.A0c
set and0oy.A00 get both pass20000ms to0Sf.A0W. Normal live action ownership
now assigns20s to these account-default shapes from the pre-write boundary.
Expiry preserves durable Sent uncertainty without replay, applying the requested
value or local rollback. Result/error cancels that timer. Per-chat E2E timer
messages and usync disappearing_mode children do not inherit it.
[Producer, native success/failure distinction, tests and limits](apk/2.26.43.72-default-timer-callback/README.md).
Paused-clock tests cover19/20s, delayed write, result/error and a separate
AB-properties callback surviving until32s. The normal-driver expiry test also
covers default-timer write delay, late reply, receive ACK and no resend.
Native timeout publishes the saved duration through observers; successful
callback has a distinct preference-write path. Complete Rust default-setting
adoption, alternate-mode gates, native error/observer behavior and exact
concurrent scheduling remain open. No live account operation.

## About reply semantics and UI/network lifetime distinguished — 2026-11-05

**Confirmed static / implemented offline:** current APK FxR.A04 sends writer29,
which storesJAj in a legacy map shared by1rj/1rh/1rD. SetAboutInfo.A5f's32s
UI wait does not itself remove the callback in the selected handler. No32s
network timer is invented for About. Selected reader teardown invokes failure
callbacks then clears this map; full lifecycle/concurrency policy remains open.
Native outer result removes the callback andJAj.A02 ignores the payload, so a
synthetic nested error extension must not turn typed About success into refusal.
Rust now preserves this producer-specific meaning; outer error/code and other
producers' nested refusals remain intact. [Full alias/dispatch evidence and regression limits](apk/2.26.43.72-about-callback-lifetime/README.md).
Paused-clock regression covers empty/nonempty About at60s, independent32s
AB-properties expiry, outer result/error and duplicate claims. Normal-driver
regression verifiesOk/no refusal code, continued receipt ACK and no resend.
Saved About adoption, UI observers and full native lifecycle remain unfinished;
no live account operation or observed server extension response is claimed.

## Profile-photo callback construction and upload result — 2026-11-05

**Confirmed static / implemented offline:** current Hyd.A00's namespace helper
also returns the to=s.whatsapp.net attribute, which Hyd appends; the existing
Rust address is confirmed rather than inferred. 6p1 lookup builds that address
explicitly. Both selected producers pass duration0 through distinctA0W/A0R
wrappers, so no generic20/32s callback timer is assigned. Upload success requires
the first child picture and ignores later error extensions; Rust now preserves
this typed success shape instead of reporting a synthetic nested refusal.
Outer errors and other action refusal parsers remain intact.
[Producer/helpers, separate lookup parser, tests and limits](apk/2.26.43.72-profile-photo-callbacks/README.md).
Tests cover exact4/5 attribute counts, server address, same-connection claims at60s,
outer result/error, duplicate/mislabeled requests and normal receive continuation.
Lookup result adoption, full malformed-result admission, cache/removal/account-center,
target normalization and image transforms remain unfinished. No live traffic or
observed server extension response is claimed.

## Two-step verification callback and diagnostic privacy — 2026-11-05

**Confirmed static / implemented offline:** current1HL.A07 passes32000ms to
0Sf.A0W; FUS.CK4 schedules original supplied values without reading response
payload. Normal two-factor-set callbacks now expire32s from pre-write start,
retaining Sent uncertainty without retry or credential adoption. Typed outer
result ignores nested error extensions; outer error/code remains separate.
[Producer, timeout/success paths, tests and limits](apk/2.26.43.72-two-factor-callback/README.md).
TwoFactorChange's derived Debug exposed PIN/email; custom Debug now preserves
field presence and redacts values without changing inputs or request bytes.
Paused-clock tests cover31/32s, delayed write, PIN/disable/email-only changes,
result/error/duplicate and unchanged durable rows after expiry. Normal-driver
late-result/receive/no-resend test includes this producer. Native credential
adoption, encryption, email-status job and error/observer behavior remain open;
no live account operation or server extension response is claimed.

## Trusted-contact token callback and execution-time gap — 2026-11-05

**Confirmed static / implemented offline:** current infra/privacytoken
GeneratePrivacyTokenJob.A0L request299 uses32000ms through0Sf.A0W. This is
separate from client-auth request314. Normal trusted-contact callbacks now
expire32s from pre-write start, retaining Sent uncertainty and scheduled state
without retry/window reset. Typed result follows8R8 selector3's captured-data
success rather than generic nested-refusal parsing; outer errors stay separate.
[Full job/callback evidence, tests and important remaining gap](apk/2.26.43.72-privacy-token-callback/README.md).
PN/LID tests cover31/32s, delayed write, result/error/duplicate, checkpoint
preservation, unchanged same-window scheduling and same-namespace exclusions.
Normal-driver late-reply/receive/no-resend test includes the actual scheduler.
**Remaining confirmed gap:** the APK reloads scheduled state and checks the
execution-time threshold before constructing the request; Rust freezes t at
queue time. Queued supersession/coalescing, effective expiry/clock, success-store
and native job retry policy need a separate port. Sent rows must not be rewritten
or replayed as a workaround. No live account operation.

## Trusted-contact queued timestamp reload and age floor — 2026-11-05

**Confirmed static / implemented offline:**87x.A04 advances scheduled state before
pending-set admission; the job reloads the stored timestamp at execution. Queued
Rust token actions now coalesce by user/schedule owner and refresh their stored
wire from current state in the same transaction that marks Sent. Reserved bytes,
IDs and send facts are never rewritten or replayed. Legacy duplicate ownership
is recovered without resurrecting a current Sent/Ok/Error action.
[Native age-floor proof, ownership, clock and regressions](apk/2.26.43.72-token-reservation-refresh/README.md).
0hu's max-per-branch then min construction proves a mandatory182-day lower bound.
Reservation defers missing state, below-floor time and superseded rows, recording
optional ActionOutcome.deferred_reason for actions plain/JSON; unrelated drain
and receive continue. Scheduling/reservation use confirmed A04 clock semantics,
without inferred stanza time. Boundary, reopen, corruption rollback, legacy and
normal-driver tests cover these paths. Stricter configured bucket admission,
full native job/pending/retry/counterpart and success-store policy remain open.
No live account operation; the age floor is necessary, not full eligibility.

## Token bucket configuration follow-up — 2026-11-05

**Confirmed static, 2.26.43.72:** selected 08Y.A0Q packaged integer entries
assign 604800 to 865/996 and 4 to 909/997. With these effective values,
0hu expiry would be 21 to under 28 days old, not the Rust 182-day fallback.
This conditional calculation is not a claim about live account configuration.
00E.A0a/A02/A06/A01/A0h show cache, preference/default and guarded MobileConfig
paths; a returned AB property is insufficient to prove the effective getter.
Whole-map final defaults, adapters and complete override precedence remain open.
[Selected entries, complete resolver methods and reproduction](apk/2.26.43.72-token-bucket-configuration/README.md).
No send policy changed; no live traffic. Next work is effective-policy resolution,
then configured scheduling/admission rather than silently hardcoding these entries.

## Token bucket returned-value inspection — 2026-11-05

Existing abprops-query --show reports token_bucket_values: selected 865/909/996/997
signed integers only. Missing keys are absent/inapplicable, null means invalid
last value; duplicates replace earlier values, including valid-to-invalid.
Full hashless replies expose none; delta/hash rules match the existing parser.
Legacy views deserialize with an empty map; saved replies reparse after reopen.
This does not resolve cache/preferences/MobileConfig or apply values to sending.
[Evidence, reused current-APK adapters, tests and unresolved mapping](apk/2.26.43.72-token-bucket-configuration/README.md).

## Token MobileConfig binding and one ready transition — 2026-11-05

**Confirmed static, 2.26.43.72:** concrete 0A8.B8Q indexes 0El's table.
865/909/996/997 all have nonzero SDK specifiers; missing raw AB rows do not
imply an unmapped/zero SDK property. ID66 binds08Y, ID252 binds0A8. A retained
0A8.BPZ branch sets08Y's A08 ready true and resets its SDK cache viaA0t.
One producer is closed, not the complete initialization or override lifecycle.
[Exact specifiers, full methods, derivation and binary-asset boundary](apk/2.26.43.72-token-mobileconfig-binding/README.md).
Effective account integers, parameter-map defaults and configured token policy
remain unported. No send policy changed and no live account traffic occurred.

## Trusted-contact successful issue state — 2026-11-05

Confirmed current 0ht.A0V/8R8.CK4 selected timestamp persistence now accompanies
Rust token action settlement atomically. Captured request time confirms a matching
schedule/nulls real metadata, or advances nonnull real time while protecting a
newer schedule; missing state inserts captured completion, dominating state stays
unchanged. A0W carries nonnegative decoded issue time: known SQL NULL means prior
sent time, raw zero means unissued. Missing/negative projection initializes zero.
Legacy unknown completion history stays distinct.
Existing actions plain/JSON reports optional token_issue_completion without users
or times. No historical Ok replay/backfill; legacy missing metadata is unknown.
[Native source, independent SQL vectors, reorder/reopen/rollback tests and limits](apk/2.26.43.72-token-success-state/README.md).
Configured bucket policy, canonical counterparts and full job/cache lifecycle remain
unfinished. No live account operation or automatic replay is introduced.

## Token namespace selection and existing bot guard —2026-11-05

Confirmed current87x.A00 uses14303 to choose pending/job user, while0ht.A0L
independently normalizes PN storage through0Ws.A0E. Counterpart deletion also
requires15491=false and original==normalized before0Ws.A0K; a mapped PN does
not automatically authorize deletion of its original row. Lookup readiness,
own-user/ranked-row authority and effective flags remain unported.
Current getType26/27 are bot user/device identities (@bot), explaining87x.A04's
existing exclusion. Rust's PN/LID allowlist already excludes them; a regression
proves no action/state mutation and retains PN/LID positive controls.
[Current methods, conditional namespace table and mapping limits](apk/2.26.43.72-token-namespace-selection/README.md).
No automatic alias merge/deletion or live account operation was introduced.

## Contact-check preparation and reservation atomicity —2026-11-05

Confirmed local fault: separate action/correlation commits could leave a sendable
orphan on disk. queue_check now prepares both atomically. Real full-disk and
uncertain-fsync tests verify neither/both consistency after reopen. Existing
Queued requests lacking correlation defer with contact_check_correlation_missing;
invalid/mismatched correlation refuses locally, and unrelated eligible traffic
drain continues. No automatic correlation repair, Sent reset or replay.
[Fault evidence, native caller-dependent usync timers and tests](apk/2.26.43.72-contact-queue-atomicity/README.md).
Current0pD.A07 receives a caller timeout: selected HxV lightweight call is32s,
while another0p7 call is128s. Rust contact-check's exact factory/deadline remains
unported; no universal usync timer is inferred. No live traffic was sent.

## Contact protocol metadata/backoff obligation —2026-11-05

Confirmed current0pD.CK4/A01 retains contact protocol success/integrity/error,
refresh and backoff separately from per-user in/out/invalid. Error backoff uses
seconds*1000 capped at3600000ms, with a missing-attribute default before the cap.
Selected observers persist positive deadlines using08q.A00;503 also writes
contact-executor global_backoff_time, while429 has an additional guarded scope.
Regular and lightweight query observers write different per-scope preferences.
[Bindings, full selected methods, strict deadline gate and remaining port requirements](apk/2.26.43.72-contact-protocol-backoff/README.md).
Rust still lacks full metadata/wait adoption. Outer IQ success alone is not contact
protocol success. No new wait, retry or account-state mutation is implemented here.

## Single-contact projection bound to original request —2026-11-05

A reproduced local defect let a successful contact-check reply overwrite status
for any returned phone. Projection now reloads/validates saved request provenance
and applies only a uniquely matching requested number. Unrelated/duplicate/
missing-origin/malformed cases preserve contact data and record a content-free
ActionOutcome.contact_projection through existing actions inspection. Status,
projection diagnostic and once-only marker commit atomically; settled wire/reply/
sequence and historical consumed markers stay unchanged. No query is replayed.
[Red regression, disk-fault/reopen tests, inspection and native-policy limits](contact-projection-correlation.md).
Current native protocol integrity/backoff and exact factory/deadline remain open.

## Contact factory flag comparison —2026-11-05

Current HxV lightweight A0B leavesHck.A0F false; H6w maps it toA0J and0pD
uses that flag to emit disappearing_mode. The regular A0C/common factory sets
it true alongside additional picture/business/username flags. Rust's fixed
contact/lid/disappearing_mode request is therefore not exact lightweight parity.
Both selected native phone-query dispatch paths are32s, but other usync callers
use128s; equal visible mode/context does not bind a complete factory policy.
[Full constructors, field transfer, writer and caller comparison](apk/2.26.43.72-contact-factory-selection/README.md).
Factory/configuration/observer/backoff integration remains open. No request or
callback policy changed and no live account operation occurred.

## Usync outer-refusal numeric metadata —2026-11-05

Confirmed current0pD.Bvj uses absent code0/backoff-1, unlike protocol-result
A01's absent-backoff default before its cap. Present values use signed JVM
parse and wrapping seconds*1000 capped at3600000ms. Durable usync get-action
outer errors now save optional ActionOutcome.usync_refusal with presence flags
and fixed numeric parse failures, exposed through existing actions plain/JSON.
Metadata survives projection/reopen; duplicate replies do not rewrite it.
[Full callback, independent JVM vectors, tests and scope limits](apk/2.26.43.72-usync-outer-refusal/README.md).
This does not enforce a wait, select an observer/deadline or replay historical
callbacks. Factory/read/reset/clock adoption remains unfinished; no live traffic.


## Contact backoff gates and reset callers — 2026-11-05

**Confirmed static**, APK2.26.43.72: the selected contact executor filters
protocols individually; its FULL contact/sidelist wait-key selection depends on
property37559. Exact0pQ.A02 direct callers are registration contact-sync start
and IBi selector16 scheduled by a contact `sync` notification. This event-bound
reset does not clear every wait key and is not generic retry permission.
[Evidence, delay arithmetic, branch scope and remaining port requirements](apk/2.26.43.72-contact-backoff-gates/README.md).
Rust eligibility/reset behavior is unchanged; query reachability, clock authority,
authenticated notification/reset adoption and effective configuration remain open.
No live account operations or server acceptance claims.


## Contact protocol refusal inspection — 2026-11-05

**Confirmed static and locally tested**, APK2.26.43.72: an outer usync IQ result
can carry a separate contact-protocol refusal. Actions now durably expose
`contact_protocol_refusal` with native backoff-first parsing, absentcode-1 and
absentbackoff3600000ms. Outer-IQ refusal defaults remain separate. Numeric
parse failures are fixed/redacted diagnostics, not guessed waits. Existing
plain/JSON actions inspection retains the field across projection and reopen;
duplicates cannot settle again. Legacy absent fields remain unknown.
[Source readers, JVM oracle, tests and unported adoption/wait policy](apk/2.26.43.72-contact-protocol-refusal/README.md).
Outer action status/per-user projection remain separate; no automatic retry,
wait enforcement, live operations or complete native integrity-policy claim.


## Pinned contact backoff clock and NTP scheduling — 2026-11-05

**Confirmed static**, APK2.26.43.72: contact backoffs use NTP-preferred08q.A00,
whereas token scheduling uses08q.A04. Constructor NTP cache admission is less
than one day; selected in-process getters do not expire it. Newly traced
property170 selects WorkManager12h/6h-flex versus12h elapsed alarm scheduling;
both setup paths also invoke acquisition immediately. A separate six-hour
process-static recent-success guard may skip it. Property2225 conditionally
controls the dual one-day adoption rejection; neither effective value is known.
[Source exports, field/caller census, search limits and port obligations](apk/2.26.43.72-contact-backoff-clock/README.md).
No Rust clock/eligibility, NTP acquisition, retry/reset or live account changes.


## Client keepalive error response correction — 2026-11-05

**Confirmed static / locally corrected**, APK2.26.43.72:1xa.Bvj routes a ping
IQ error through the sameFea(24) manager response as validated success. Rust
previously ended receive as nonrecoverable keepalive refused. A matched live
ping error now clears only that ping and receive continues with a fixed
redacted diagnostic. Duplicate old errors do not consume successor pings;
non-replies/late replies and unowned IQs retain existing admission. No clock,
account, request retry or restriction-reset authority is added.
[Callback binding, red/green regression and open cadence/schema gaps](apk/2.26.43.72-client-ping-errors/README.md).
Full native keepalive scheduling/configuration and successful-result schema
remain unported; no live account operation or causal ban-prevention claim.


## Client pong success schema and consumed callback — 2026-11-05

**Confirmed static / corrected locally**, APK2.26.43.72:1xa.CK4 requires parsed
from, original-request ID equality, bounded signed-long t and exactresult.
Malformed responses throw recoverableLIL before the manager response, after
one-shot IQ callback removal. Rust now separates callback claim from ping timer:
malformedresult consumes the claim, leaves the original timeout active and
continues receive; a later same-ID validresult/error cannot revive it. Fixed
field diagnostics contain no supplied values. Error-response behavior remains
separate. Existing Rust JID text bounds are a local supported-path limit.
[Reader/exception/ownership proof, regressions and remaining gaps](apk/2.26.43.72-client-pong-schema/README.md).
No clock adoption, replay, new request, cadence/timeout change or live traffic.


## Client ping interval restarts on response — 2026-11-05

**Confirmed static / locally corrected**, APK2.26.43.72:1A7.A05 schedules the
next periodic ping from its accepted response. Rust's fixed shared tick could
send again1second after a response delayed19seconds. Separate maintenance and
next-ping deadlines now restart only the ping interval; queued work still drains
at its original local maintenance deadline. Invalid/duplicate/unowned replies
do not restart it. Red/green actor regression covers bothresult anderror.
[Evidence, packaged15s/240s/32s entries and resolver limits](apk/2.26.43.72-client-ping-scheduling/README.md).
Effective native durations/configuration/mode remain unported; existing local
values, wire requests, uncertainty guards and no-live-operation boundary remain.


## Client ping first-access configuration precedence — 2026-11-05

**Confirmed static**, APK2.26.43.72: ping getters use00G.A02 with
firstAccessValue=true, selectingA07 rather than the token getters' ordinary
cache. Selected readiness resetA0t replacesA0d, notA07. The SDK comparison
branch clones exposure options while preserving first-access semantics; a
successful legacy read can populate the winner before SDKputIfAbsent. Direct
SDK selection remains distinct. No effective value is inferred from raw AB
rows or missing specifiers, and cache owner/replacement scope remains open.
[Complete options/getters, field census and conditional precedence proof](apk/2.26.43.72-client-ping-first-access/README.md).
Research only; native effective timing/configuration remains unported. No live,
request, timer, retry/reset, account or clock behavior changed.

### 2026-11-05: final compiled ping defaults

**Confirmed static / derived static, APK 2.26.43.72:** whole integer-builder
argument-flow analysis of 08Y.<init>, A0B, A0I and A0Q finds 4158 integer puts,
3585 distinct keys, and exactly one write each for 15103=32000, 15183=240000
and 15270=15000. The final snapshot binds to this.A02; no later target write
replaces those entries. This closes the selected-assignment-only caveat for
these three keys. **Unknown/unported:** effective account values and configuration
owner lifetime. No runtime timer, retry, replay or live-account behavior changes.
[Reproducible bounded proof and limits](apk/2.26.43.72-client-ping-final-defaults/README.md).

### 2026-11-05: ping channel lifecycle and scheduler-mode authority

**Confirmed static, APK 2.26.43.72:** 0YB.A01 -> 2JJ selector 2 binds startup.
Same connected callback identity returns without resetting; new accepted channel
resets the ping counter/outstanding timestamp/timeout flag and schedules the
initial interval. Counter reset does not reset the configuration first-access
cache. Scheduler mode derives from push-registration preference presence and
logins_with_messages < 3, then effective property 6493 for Runnable selection.
**Unknown/unported:** actual preference/config values, complete owner lifetime,
on-demand/channel-replacement callback interactions and receiver failure paths.
[Source, exact field census and transition table](apk/2.26.43.72-client-ping-channel-lifecycle/README.md).
No runtime policy or live-account behavior changed.

### 2026-11-05: normal pings and ACK-confirmation callbacks differ

**Confirmed static, APK 2.26.43.72:** A08 uses normal A04 pending guards; direct
A0B callers also include offline-resume and coroutine pre-ACK confirmation.
Bound Fef14 and Feq9 continuations request A0J(marker) only after 1xa's validated
success, not IQ error or delivery failure. Enqueued manager-liveness work and
synchronous success completion have no join establishing a fixed execution order.
A0A additionally serves arbitrary calling/signaling callback families; shared
request shape does not establish shared response policy. **Still open:** registry
cleanup across channel replacement, coroutine cancellation and complete durable
replay/retirement parity. No Rust timing, retry, replay or account behavior changed.
[Producer census, constructor bindings and limitations](apk/2.26.43.72-client-ping-producer-families/README.md).

### 2026-11-05: ping registry reset and already-claimed callbacks

**Confirmed static, APK 2.26.43.72:** 1xZ.BVv selects gated alternate dispatch;
nonnull success continuations use 1xa's serial path. Reset A0F posts handler 3,
which snapshots/clears awaiting-response map A0H and schedules delivery failure.
Queued A0I and already-claimed worker ownership are distinct; no synchronous
all-callback cancellation follows. **Verified locally:** stale result/error from
a prior closed link cannot settle or extend the restarted link's ping after store
reopen. **Open:** complete queued disposal, native executor/registration ordering
and claimed-worker cancellation. No production timing/retry policy changed.
[Evidence, regression and limits](apk/2.26.43.72-client-ping-registry-lifecycle/README.md).

### 2026-11-05: disconnect cancels a scoped retriable-IQ job map

**Confirmed static, APK 2.26.43.72:** 0Sf.A0E holds delayed T56/TKl retry jobs,
not queued IQ callback ownership. Disconnect requests job cancellation; selected
coroutine cancellation forwards original delivery failure. Scheduling requires
remaining budget and callback CmQ permission; errors additionally require allowed
500..599 code excluding the retained eight-code set. Direct application caller
is a feature-gated status-privacy job, not every IQ. **Unported/unknown:** caller
callback eligibility, full job-queue/cancellation ordering and durable recovery.
[Complete callback/budget bodies, caller census and boundaries](apk/2.26.43.72-iq-retry-job-cancellation/README.md).
No runtime retry, replay, timer, account or persistence policy changed.

### 2026-11-05: status-privacy callback distinguishes root and nested errors

**Confirmed static, APK 2.26.43.72:** original 8RJ17 permits bounded wrapper retry,
but T56 reads root IQ code/backoff while the original reads nested error code.
Nested 500 alone reaches original code/future handling and the job's exception;
404 logs no settings without success adoption. Successful parser requires privacy
child/type before conditional audience persistence. **Open:** effective flags,
store transactions, scheduler limits and durable recovery. Rust has no matching
status-audience fetch/adoption path; no runtime retry or settings change added.
[Exact callback binding, branch table and limits](apk/2.26.43.72-status-privacy-fetch-callback/README.md).

### 2026-11-05: preserve About refusal callback codes

**Confirmed static / implemented offline, APK 2.26.43.72:** the ordinary About
error path processes signed Java Integer codes for each direct error child in
order, stopping on malformed present input. Existing generic u16 metadata lost
signed/large/subsequent values. Optional ActionOutcome.about_refusal now retains
the codes and malformed boundary atomically with settlement, exposed by actions
plain/JSON. Legacy absence remains unknown. Duplicate/reopen/no-replay and JVM
oracle tests cover the correction. No retry, cooldown or saved About adoption.
[Exact source, oracle and limits](apk/2.26.43.72-about-error-codes/README.md).

### 2026-11-05: current prekey error backoff has scoped state

**Confirmed static, APK 2.26.43.72:** non-PQ error selector 76 moves affected
existing in-flight entries to pending only for 500..599. A01 mode and A00 posted
flag control scheduling; the 10/610 generator starts at zero delay. Delayed drain
clears posted flag but retains mode; explicit reset differs. Complete normal
admission does not establish a blanket stop of all key queries. **Unknown:**
network-to-listener attribution, external reset authority and durable job/write
recovery. No Rust retry, admission or reservation policy changed.
[Exact anchors, source transitions and delay derivation](apk/2.26.43.72-prekey-error-backoff/README.md).

### 2026-11-05: current key-fetch error attribution and signed codes

**Confirmed static / implemented offline, APK 2.26.43.72:** writer binds Dxh to
captured devices; outer errors map to that set and per-user errors are grouped
only while iterating original requested devices. A1S/A0e constructs event 76
matching the backoff listener. Signed Java integer code grammar is retained.
DirectoryInspection.server_error_code widens from Option<u16> to Option<i32>,
preserving valid negative/large/Unicode codes after existing unique-context guards.
JVM-case regressions cover root/user attribution, redaction, reopen and no refresh
or resend. Other inspectors/generic action fields remain unchanged. Full native
registration, duplicate/missing-device and durable retry ownership remain open.
[Source, API consequence and limits](apk/2.26.43.72-prekey-wire-error-attribution/README.md).

### 2026-11-05: non-reply IQ cannot consume a pending key page

**Implemented offline, confirmed APK43.72 outer dispatch:** key-batch settle now
returns false without store mutation for IQs lacking result/error type, even with
matching ID and error child. Previously reusable settlement could terminally
classify such input and lose the later actual reply. Normal group-driver already
filtered these types. Red/green regression covers state equality, blocked reserve,
reopen, later valid settlement, duplicate refusal and complete local assembly.
No new callback authority, key adoption, timeout or automatic retry is added.
[Evidence, scope and unresolved native lifetime](device-key-batches.md#key-batch-reply-admission-correction-2026-11-05).

### Ordinary message ACK forwarding (2026-11-05)

**Confirmed static, APK43.72:** selected ordinary message success and refusal
event paths both jump back to generic A1D with the original node/Eoh key.
This closes those forwarding edges into the previously traced concrete callback
bridge. Error459 and malformed/early paths remain separately qualified. The
destination rewrite helper inspected here is FBID-migration-specific; Efa's
null type default does not prove the out-of-band producer leaves type unset.
[Exact offsets, retained exports and remaining producer gap](apk/2.26.43.72-message-ack-forwarding/README.md).
No runtime guard, replay policy or live account operation changed.

### Sender-key requirement/session boundary (2026-11-05)

**Confirmed static, APK43.72:** the sender-key requirement inherits conditional
recipient selection and pairwise session readiness. A selected hash mismatch
retains the original candidate set instead of applying later sent-key filtering.
The latched A07/A08 booleans come from session/PQ policy, not distribution ACKs.
The exact native ordinary direct-SKDM producer is still unlocated; the status
requester and include flag do not establish it.
[Methods, offsets, search scope and unknowns](apk/2.26.43.72-sender-key-session-requirement/README.md).
No Rust runtime, replay or live account behavior changed.

### Membership guard across distribution ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** after reserving a reaction's
pairwise distributions and group ciphertext, a saved membership change while
waiting for a distribution ACK did not stop the next distribution or final
group stanza. Query/key preparation had a frozen-membership digest guard, but
the ACK-driven continuation omitted it.

Every continuation now compares the saved group snapshot digest before returning
another stanza, including the initial distribution return and final group return.
A mismatch persists the existing `membership_changed` preparation block and
returns no dependent ciphertext. Already exposed keys, pairwise sessions,
original encrypted attempt, uncertainty and reservations stay held; no replay,
rollback, fresh IDs, automatic membership query or rotation is authorized.

The regression `membership_change_during_distribution_cannot_release_more_ciphertext`
prepares two real pairwise distributions, applies the real remove-notification
snapshot handler while either the first or second ACK is pending, then supplies
the matching successful ACK. Old code failed by releasing stale ciphertext.
The corrected path stops before the remaining distribution or final group send,
preserves all crypto/outbox state and all non-preparation checkpoints, and stays
nonselectable after reopen. Existing unchanged-membership crypto/decrypt tests
continue to cover successful release.

**APK corroboration, not scheduler parity:** APK43.72's session requirement
A01 01de..0206 distinguishes a changed participant hash from equal-hash
sent-key filtering, as retained in
[the requirement bundle](apk/2.26.43.72-sender-key-session-requirement/README.md).
This fix extends the existing local frozen-snapshot policy; native per-ACK
revalidation or identical blocking semantics have not been proven. The check
is a saved-state observation before returning a node, not an atomic transaction
with a remote socket write; the entire snapshot digest also conservatively
includes non-membership group fields under the existing policy. Native direct
SKDM producer/address ownership remains unresolved. No live account operation.

### Authority guard across distribution ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** distribution continuations now
revalidate post-reservation routing and identity authority, preserving accepted
pin replacements from the original transaction. Changed saved device lists,
peer pins or own authority block with `routing_changed`; ratchets are excluded.
[Red/green regression, exact scope and native/local limits](normal-group-outbox.md#authority-guard-across-distribution-ack-waits-2026-11-05).
Original ciphertext, keys and uncertainty remain reserved; reopen cannot replay.
No live operation.

### Sender-key validity across distribution ACK waits (2026-11-05)

**Confirmed local defect / corrected offline:** a distribution ACK can no longer
release dependent old-chain ciphertext after saved exposure invalidation or
chain replacement. The reservation captures chain authority, and continuation
checks it alongside routing. Same-chain delivery bookkeeping remains allowed.
[Red/green invalidation, replacement and positive-control cases](normal-group-outbox.md#sender-key-validity-across-distribution-ack-waits-2026-11-05).
Original reservations/uncertainty remain held; no reset, replay or live operation.

### Device-refresh revision guard (2026-11-05)

**Confirmed local defect / corrected offline:** saved recipient device-sync
revisions now participate in fanout OCC and post-reservation continuation.
New affected-recipient refresh work blocks stale ciphertext even before a new
list arrives; unrelated-contact work still permits continuation.
[Red/green ACK-wait regression, fresh/cached race and limits](normal-group-outbox.md#device-refresh-revisions-across-preparation-and-ack-waits-2026-11-05).
No pending-work deletion, counter reset, replay or live account operation.

### Cancelled group preparation replies (2026-11-05)

**Confirmed local defect / corrected offline:** a cancelled pre-reservation group
intent now settles its owned current query and stops with intent_cancelled,
without another query, crypto adoption or actor disconnect. Cancellation after
InFlight remains rejected by the existing API.
[Actor red/green test, reply history, reopen and boundaries](normal-group-outbox.md#cancellation-while-a-preparation-query-is-pending-2026-11-05).
No reset, replay or live account operation.

### Direct cancellation reply boundary (2026-11-05)

**Confirmed local / actor regression:** matching result/error replies and their
redelivery after direct intent cancellation emit no ciphertext, adopt no keys,
and preserve receive service. Original cancelled intent/query reservation stay
held across reopen. This was already correct; no production code changed.
[Actor proof, diagnostic distinction and remaining malformed-input scope](messaging-transport.md#direct-send-cancellation-reply-audit-2026-11-05).
No live operation or replay authority.

### Direct directory wire refusal isolation (2026-11-05)

**Confirmed local defect / corrected offline:** selected-user/classic-bundle
parser failures suspend the original direct intent with invalid_directory_reply
instead of ending receive. New-session classic signature refusal uses existing
invalid_signed_bundle. Neither permits resume; no keys/ciphertext are adopted.
[Actor red/green matrix, reopen and stored-corruption control](messaging-transport.md#malformed-direct-directory-replies-stay-intent-local-2026-11-05).
No live operation, server-refusal fabrication or replay authority.

### Invalid-directory explicit resume audit (2026-11-05)

**Confirmed local:** after reopening, the actual direct resume API refuses all
seven bad-directory/signature cases under both identity policies without any
table or journal mutation. CLI suspension wording now includes directory
validation failures; fixed reason fields are unchanged.
[API-entry regression and boundaries](messaging-transport.md#explicit-resume-refusal-for-invalid-directory-suspensions-2026-11-05).
No runtime resume policy, live operation or replay authority changed.

### Unique group-info recovery projection (2026-11-05)

**Confirmed local defect / corrected offline:** ambiguous queued group-info
replies can no longer bypass normal-send validation through restart recovery.
Shared projection rejects multiple group elements; missing queued group data
stays unapplied. Prior snapshot, pending marker and reply remain preserved.
[Red/green recovery, direct-recorder control and native/local scope](normal-group-outbox.md#unique-group-info-projection-across-recovery-2026-11-05).
Valid single groups still apply once. No query replay or live account operation.

### Unusable group-info identity stays pending (2026-11-05)

**Confirmed local defect / corrected offline:** missing/nontext ID in a selected
group-info element no longer consumes its pending projection as a successful
no-op. Recovery preserves reply, prior snapshot, checkpoints and journal.
[Red/green cases and bookkeeping boundary](normal-group-outbox.md#unusable-group-info-identity-preserves-pending-projection-2026-11-05).
No network retry, cache fallback or live account operation.

### Combined media route/wait preservation (2026-11-05)

**Confirmed local / regression:** six malformed refresh results preserve the
installed route and saved 507 wait together, with raw reply retained. Exact
duplicates and late conflicts cannot mutate them; reopening stays throttled.
A valid-result control still applies routing and the existing wait reset.
[Combined matrix, native evidence and history-scan limits](media-network-outcomes.md#malformed-refresh-replies-preserve-route-and-wait-together-2026-11-05).
No runtime policy change, new query or live operation.

### Current group-info MEX timer boundary (2026-11-05)

**Confirmed static, APK43.72:** GetGroupInfoProtocolHelper.A00 builds
QueryGroupInfo GraphQL input; the selected non-Pando dispatch serializes a
w:mex IQ and forwards the new operation's32000ms default to0Sf.A0W. The
helper's30-second initializer calculation is discarded, not timer evidence.
[Complete selected writer/dispatcher methods, branch gates and unknowns](apk/2.26.43.72-group-info-mex-timeout/README.md).
Rust's classic group IQ remains an inferred separate shape; neither its
30-second local preparation limit nor all group-info deadlines gain native
parity from this MEX trace. No runtime policy or live operation changed.

### Missing participant-identification context lifecycle (2026-11-05)

**Confirmed static, APK43.72:** the MEX override set is stored in preferences.
0zu.A02 adds a group when the selected marker child is present; absence does
not clear it.0co.A03 ->82U.A0Y ->0wE.A0e/A0b carries the group-info boolean
to an add/remove operation before later group processing.
[Persisted-set methods, exact register mapping, ingress gates and unknowns](apk/2.26.43.72-group-missing-identification/README.md).
Rust already exposes the explicit MEX context vocabulary; automatic lifecycle
and fully admitted ingress parity remain unported. No runtime/live operation.


### Missing-identification numeric handler boundary (2026-11-05)

**Confirmed static, APK2.26.43.72:** marker consumers0w9/19k construct
BaseNotificationHandler with numeric registrations209/189. The base checks
that integer before processing IncomingStanza; factory construction alone does
not establish wire notification type or authorization. Runnable selector paths
and the upstream integer producer remain unknown. No runtime port or live
operation. [Retained constructors, base handler, factory excerpts and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Current notification type mapping and picture gap (2026-11-05)

**Confirmed static, APK2.26.43.72:** EL2 maps w:gp2 to209 and picture to189;
1rN.A05 reads IncomingStanza type and passes that code as Message.arg1 via
2Ai.A00. Marker admission remains feature/group/child dependent; picture uses
set/delete. Rust has w:gp2 ACK handling but no picture consumer/ACK or automatic
missing-identification lifecycle. Native deferral and state ordering still need
reconciliation before porting. [Retained map, consumer and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Picture notification base ACK (2026-11-05)

**Confirmed static / locally implemented:** current19k picture handler has only
constructor/A09 and no base A07 deferral calls;8aj selector1 returns processing
to0v6.A01. Rust now emits the shared base ACK for picture after durable capture.
Malformed envelopes log/no-ACK and leave receive running; redelivery is ACKed
again. Photo updates and missing-ID lifecycle remain unported, with no automatic
queries or live validation. Actor regression verifies capture-before-write, exact
participant routing, redelivery, ping after refusal and reopen persistence.
[Source and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Picture ACK wire-field provenance (2026-11-05)

**Confirmed static, APK2.26.43.72:** integer76 dispatches0v0 ->0v2.A02 ->
F3e.A05, which builds the final ACK. F0R's fresh notification key populates
from/id/participant/type/class; recipient/edit/error/offline are not copied.
The existing picture actor regression now supplies those attributes and requires
the same exact base ACK. Native photo exception/CCQ/full address parity remains
unverified. [Retained final builder, key fields and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Notification address diagnostic boundary (2026-11-05)

**Confirmed static, APK2.26.43.72:** DBe.A1A/0Tl preserve Jid objects and raw
strings for ACK attributes.0Tp.A0A logs failed isProtocolCompliant then continues
toward class conversion/return; it does not reject solely on that boolean. A
proposed blanket rejection based on this diagnostic was ruled out. Current
binary-reader zero-AD normalization and subtype writer parity remain unclosed;
no shared Rust address policy changed. [Retained source and limits](apk/2.26.43.72-group-missing-identification/README.md).


### Current-build ACK zero-device normalization (2026-11-05)

**Confirmed static, APK2.26.43.72 / offline regression:**123.A01/A02/A04
reconstruct binary zero-device PN/LID without a device suffix;1r2.A00 chooses
pair for zero compliant getters and AD for positive device/agent. The existing
picture actor test now covers zero/nonzero PN/LID sender and participant forms.
Native domain flags are distinct from Rust's agent field. No runtime policy
change; all subtype/hosted/interop behavior and live acceptance are not claimed.
[Current source and census](apk/2.26.43.72-group-missing-identification/README.md).


### Current notification constructor census (2026-11-05)

**Confirmed static:**34 physical0v6 constructor callers are retained with their
numeric code/base-executor boolean. The type map has38 entries; these counts
are different populations and do not measure completeness. server_sync210 maps
to14l (typed collection/bootstrap work), server235 to15I (log/abprops/push-config
follow-ups). Rust has no dedicated handler/ACK for either. Full state/helper/job
and ACK ordering remain unported; no generic ACK, token fabrication or runtime
change was added. [Census and complete selected handlers](apk/2.26.43.72-notification-handler-census/README.md).


### Server-sync callback ownership reconciliation (2026-11-05)

**Confirmed static:**14l.C6N ->2JI selector6 emits count-minus-one aggregate
metrics and clears its counter map; it is not an ACK completion gate. The
existing lifecycle audit already proves dirty-version assignment and normal
ACK-before-async-result ordering. The constructor census now links that evidence
instead of presenting it as newly unknown. Rust role/recovery/bootstrap and
durable notification scheduling remain unported. No runtime change.
[Callback evidence and limits](apk/2.26.43.72-notification-handler-census/README.md).


### Rust server-sync gate mapping audit (2026-11-05)

**Confirmed source:** current primary-login checks, held-fetch admission, pending
patch exclusion and receipt-time server backoff were mapped individually against
the native lifecycle. None supplies the missing bootstrap/recovery/ready-mutation
gates by default. Generic reservation/ActionStatus::Ok is not sync-flight ownership
or authenticated collection advancement. [Mapping, source boundaries and integration consequences](server-sync-rust-state-mapping.md). No runtime or live operation changed.


### Registered device-index writer provenance (2026-11-05)

**Confirmed static:** exact registration_device_id string scan finds8 references.
Selected107.BLn device-JID getter feeds the saved value/cache;09O.A01 and
G0j.A02 have zero resets;0Qq.A0a reads selected-account state. Existing role/
Me-ingress audits qualify the newer mapping's unknowns. Full Rust role migration
and producer/account binding remain unported. [Writer/reader census and limits](apk/2.26.43.72-notification-handler-census/README.md). No runtime reset, query or live operation.


### Group notification storage failures stop before ACK (2026-11-05)

**Confirmed local defect/fix:** the actor no longer logs a failed group snapshot
update then emits w:gp2 ACK. Store/transaction errors now surface first; wire
skip behavior is unchanged. Red/green actor regression proves no ACK after a
corrupt-snapshot read, unchanged saved bytes and durable capture across reopen.
No automatic replay/repair or live operation. [Full correction and APK boundary](notification-ack-audit.md#group-notification-storage-failures-stop-before-ack-2026-11-05).


### Group envelope admission precedes membership updates (2026-11-05)

**Confirmed local defect/fix:** invalid group ACK envelopes previously changed
saved membership before refusal. The actor now validates the existing envelope,
commits the supported change, then writes ACK. Missing/empty ID and unusable
participant regressions preserve exact snapshot bytes and receive continuity;
a valid control commits before ACK and survives reopen. Store faults remain
visible; groups_dirty retains its own path. [Evidence, red/green test and limits](notification-ack-audit.md#validate-group-ack-envelope-before-membership-mutation-2026-11-05).


### Public group-change envelope guard (2026-11-05)

**Confirmed local defect/fix:** public groups::apply_change_notification now
validates the same ACK envelope before store access; direct callers cannot
bypass the receive actor's prerequisite. Three invalid-envelope regressions
preserve exact snapshot bytes and journal head. A routing-race fixture gained
a synthetic notification ID without weakening its assertions. The helper sends
nothing. [Details and red/green validation](groups.md#public-group-change-admission-matches-receive-2026-11-05).


### First group event owns the supported change (2026-11-05)

**Confirmed static / locally fixed:** APK43.72 0Tp.A0D/0w9.A09 selects first
child as the event. Rust previously applied every outer sibling; unsupported+
remove could remove membership and add+remove applied two changes. The helper
now selects only the first event, retaining inner participant iteration and
existing ACK/envelope/store guards. Red/green regression preserves unselected
membership and standalone valid updates. Extra metadata siblings are permitted;
full native event/timestamp/alias parity is not claimed.
[Checked selection evidence](apk/2.26.43.72-group-event-selection/README.md).


### Dirty group event with metadata siblings (2026-11-05)

**Confirmed static / locally fixed:** current0w9 selects first-child groups_dirty
without requiring exactly one outer child. Rust previously skipped refresh for
metadata siblings and could generic-ACK without scheduling. Dedicated dirty
handling now selects the first event; generic ACK stays suppressed for it.
Existing durable queue/dedup/envelope/inner-list guards remain. The expanded
queue-before-ACK regression covers metadata sibling, sole ACK ownership and
reopen without another reservation. Missing-identification state remains unported.
[Evidence and limits](apk/2.26.43.72-group-event-selection/README.md).


### Dirty list filters named group entries (2026-11-05)

**Confirmed static / locally fixed:**0w9.A09 calls0Tp.A0N(group), which selects
matching children and skips others. Rust's whole-list rejection on unrelated
children suppressed valid refresh/ACK. It now filters group entries; selected
identity/duplicate/device, empty-work, envelope and256-total-child guards remain.
Red/green queue-before-ACK/reopen regression includes metadata with an unusable
jid beside a valid group. Unknown-only remains unsupported.
[Exact helper and limits](apk/2.26.43.72-group-event-selection/README.md).


### Privacy-token legacy agent qualifier (2026-11-05)

**Confirmed local defect/fix:** supported primary from/sender_lid admission
checked device0 but not agent0; a qualified address could write token state.
Those guards now require both zero, retaining PN/LID namespace handling and
existing invalid-alias policy. Native10G.A09 uses UserJid getters; Rust's LID
domain bit is not its legacy agent field. Red/green typed/text PN/LID and alias
regression preserves checkpoint/pin/session snapshots across reopen. Reach-out
restriction envelope admission was separately rechecked and already guarded.
[Complete current source and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### First named privacy-token container (2026-11-05)

**Confirmed static / locally fixed:**10G.A09 ->0Tp.A0G/A0F selects the first
named tokens container. Rust's uniqueness guard suppressed valid first data/ACK
when another container followed. Selection now matches the APK; later containers
cannot replace state or rescue a malformed first. Existing selected-list/time/
size/address/atomic guards remain. Red/green regression includes leading metadata,
later newer/malformed data and invalid-first/valid-second.
[Complete helper evidence and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### Consume token time only for supported data (2026-11-05)

**Confirmed static / locally fixed:**10G.A09 reads parent t only inside the
trusted_contact branch. Rust's eager timestamp parse suppressed ACK for empty
or ignored-only lists. Parent time is now parsed only for supported entries;
supported-token Java/range/atomic checks remain. Red/green empty/unknown-only
regression covers absent, malformed, negative and overflowing unused t without
changing prior state. [Retained branch proof and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### Ignored privacy tokens do not consume sender alias (2026-11-05)

**Confirmed static / locally fixed:**10G.A09 reads sender_lid only after selecting
trusted_contact. Rust's eager alias resolution suppressed ACK for empty/ignored
lists with invalid unused metadata. After the existing container/list checks it
now returns no updates before alias/time resolution when no supported entries
exist. Supported alias/time/primary/atomic guards remain. Red/green regression
combines ignored lists with malformed or agent-qualified unused alias and time,
preserving prior token state. [Evidence and limits](apk/2.26.43.72-privacy-token-primary-address/README.md).


### Account privacy first-event admission (2026-11-05)

**Confirmed static / locally fixed:**19N.A09 selects the first event. Rust's
exact-one outer child guard suppressed supported privacy adoption/ACK with later
siblings. It now requires the first event to be supported privacy and ignores
later event siblings; unsupported/malformed first events cannot be rescued or
overridden. New red/green regression preserves prior state for those refusals.
Category/envelope/pending-intent/transaction guards remain.
[Evidence and boundary](account-privacy-notifications.md#first-event-selection-with-later-siblings-2026-11-05).


### Account privacy named-category selection (2026-11-05)

**Confirmed static / locally fixed:**19N.A09 uses0Tp.A0N(category). Rust's
whole-child parsing rejected unrelated metadata beside supported readreceipts.
It now filters named categories before validation, retaining64-total-child,
selected category/value/body, envelope/pending/transaction guards. Metadata-only
remains refused. Red/green regression ensures metadata cannot adopt preference
state and does not suppress the selected valid category.
[Source and limits](account-privacy-notifications.md#named-category-filtering-2026-11-05).


### Count ACK optional source and malformed-source refusal (2026-11-05)

**Confirmed static / locally fixed:** deferred89Y preserves original Eoh; the
common final builder omits absent to. Rust's server fallback fabricated explicit
to for absent source and redirected explicit parse failure. Count ACK now omits
to for absence, retains normal parsed routing and locally refuses explicit
invalid source before ACK/upload. Red/green actor coverage checks source-less
PQ no-upload and malformed-source capture/receive continuation; valid legacy
upload controls remain. Native ACK-after-work timing remains unported.
[Complete deferred chain, wire proof and limits](apk/2.26.43.72-prekey-count-ack-source/README.md).

### Transactional stanza-to-ACK dispatch (2026-11-05)

**Confirmed static, APK2.26.43.72:** Flr selector7 has a conditional outer
transaction that persists selected stanzas and computed pre-ACKs before
Euj.A02 send admission. A thrown outer close propagates before that send call.
The alternate branch persists stanzas without this explicit outer pre-ACK
insertion. Both operate on a filtered return list, not all captured input;
A0I is set after send admission returns, not after server confirmation.
[Five exact methods, offsets and limitations](apk/2.26.43.72-preack-transactional-dispatch/README.md).
This closes the Flr insertion-caller gap, not effective configuration,
count-specific bulk membership, crash replay or upload/socket ordering.
No Rust runtime behavior or live account operation changed.

### Malformed count content still receives its control ACK (2026-11-05)

**Confirmed static and fixed offline:** current15v.A09 throws LIL for a
malformed present PQ count before deferring metadata or queueing upload work.
0v6.A01 catches LIL and dispatches the still-present original ACK context;
SQLite full/corruption failures instead rethrow. Rust now ACKs supported count
envelopes on content refusal without upload. Invalid IDs/sources still refuse
ACK; storage and uncertain write failures still propagate. This supersedes the
earlier local no-ACK policy for malformed PQ content.
[Exact parser/catch paths, incoming-kind census, regression and remaining queue limits](apk/2.26.43.72-count-content-error-ack/README.md).
Valid-count upload/ACK ordering and full queue replay remain open. No live operations.

### Deferred ACK six-second watchdog is not a send deadline (2026-11-05)

**Confirmed static,43.72:** the0v6.A07 scheduled8bZ selector41 reports pending
ACK failures and clears tracking maps; it does not ACK, cancel upload work or
consume held89Y tokens. Deferral resets the shared6000ms schedule. A later
89Y completion ignores whether removal found an entry and can still dispatch
its one-use token. [Exact schedule, branch and token distinction](apk/2.26.43.72-deferred-ack-watchdog/README.md).
This closes the prior timer-action unknown, not count-specific routing,
socket ordering or recovery parity. No Rust timer/replay or live operation added.

### Registered signed-prekey absence versus stored-state faults (2026-11-05)

**Confirmed local defect, fixed offline:** actor maybe_upload caught every
error from registered_signed_prekey and reported a normal missing-key skip.
That hid corrupt saved registration JSON, account/material mismatches and
other lookup/validation failures. The shared factory now has an internal
optional lookup: only an absent registration-lab-material row returns None.
Existing JSON/account/signature/private-key/serialization errors propagate.
The public required-key lookup and digest caller still require a row and
return their existing unavailable error for absence. No identity, signed key
or one-time key is generated to repair a failed lookup.

The actor skips only None and propagates errors before generating/reserving
or writing an upload. An injected-link regression reproduces the old swallowed
corrupt-row error (actor never ended), then checks clean absence skip, Storage
for corrupt JSON and InvalidInput for a mismatched account. Saved material and
prekey rows are unchanged. Existing valid-material upload controls remain.
This is local diagnosability/integrity behavior, not a claim about exact native
signed-key fallback. The native base-handler SQLite exception distinction is
retained in [the count error audit](apk/2.26.43.72-count-content-error-ack/README.md).
Count ACK-before-upload ordering remains unported and unchanged, so this does
not claim all count work errors occur before its control ACK. No live operations.

### Signed-prekey public/private consistency before upload (2026-11-05)

**Confirmed local defect, fixed offline:** registered signed-prekey lookup
verified the identity signature over the saved public key but constructed a
SignedPreKeyRecord from an independently deserialized private half without
checking that the halves matched. A valid public signature does not validate
that private half. The shared optional/required lookup now derives its public
key and refuses a mismatch with a secret-free InvalidInput reason before
serialization. Upload admission propagates that error before generating,
reserving or writing any upload. The digest required-key lookup also uses the
same validation; no private bytes are exposed or repaired.

Expanded actor corruption regression keeps the registered signed public key
and its valid signature unchanged but substitutes the independently generated
identity private key. Old code reached an upload write instead of returning an
error (the actor-end expectation timed out). Corrected code returns the exact
pair-mismatch error and leaves saved material and prekey rows unchanged.
Existing clean-material upload and absence/corrupt-JSON/account-mismatch
controls remain. This is a local cryptographic consistency check, not a new
APK-derived wire requirement or proof of native corrupt-store handling.
Registration request-factory loading outside this signed-key lookup is not
claimed audited by this change. No identity rotation, key replacement, new
retry or live account operation; valid-count ACK ordering remains unchanged.

### Registration material key-pair integrity before request construction (2026-11-05)

**Confirmed local defect, fixed offline:** StoredComparisonRequestFactory.load
checked frozen attempt/configuration, account public metadata and the signed
public-key signature, but did not prove the saved private halves matched the
advertised Noise, Signal identity or signed-prekey public keys. Its account
comparison also ignored private-half corruption in the separate AccountRecord.
A request could therefore be constructed from material unusable for the later
private-key operation, even with a valid signed public-key signature.

Material.check_account now derives/checks all three material key pairs plus
the account Noise and Signal identity pairs after its existing account/public
metadata check. It reports only a fixed pair-specific InvalidInput reason.
Both normal request loading and repeated/corrected preparation use this guard.
The registered-signed-prekey lookup also shares check_account. Derivation
compares effective public keys, not raw private-byte identity; no new key,
request field, registration retry or account generation is manufactured.

Regression: for each of five stored private-half substitutions (material Noise,
identity, signed; account Noise, identity), retain public keys/signature/config,
then require code/exist/complete/security request construction and repeated
prepare to fail. Exact Account table bytes and journal head remain unchanged;
reopen still refuses and retains bytes. Clean construction is a control in
each case. Old code failed the Noise case because it built the request.
This is local cryptographic integrity, not an APK wire-field requirement or
proof of native corrupt-store behavior. No HTTP/account operation ran.

### Count ingress has a gated tracked fallback (2026-11-05)

**Confirmed static,43.72:** notification reader routing is conditional on
provider selection and configuration. With no named provider,3992 true creates
a tracked1k4 wrapper for recognized encrypt; false reaches direct handling.
A bounded census of ten physical routing-provider constructors declares no
encrypt entry, but does not establish the full effective injected set.
[Registry identity, enum, wrapper, persistence-policy methods and exact limits](apk/2.26.43.72-count-ingress-routing/README.md).
The reader inverts buffering-policy output for a wrapper Boolean; its durable
meaning and count-specific bulk ownership remain unproven. No Rust routing,
ACK order, default flag or replay changed; no live operations.

### Tracked count queue admission is separate from disk commit (2026-11-05)

**Confirmed static,43.72:** reader buffering-policy inversion becomes wrapper
A0D;1k4 selects1ix. A0D=false requires a storage backend, but1id.A0P first
adds the stanza to memory. Its successful return is not commit proof.1ix
A0L=false disables the generic inline-state promotion. A conditional500ms
disk schedule cancels/replaces a prior job and launches the already-traced
Flr selector7 transactional writer.
[Seven complete methods, Boolean mapping and processing-gate limits](apk/2.26.43.72-count-queue-admission/README.md).
The arrival hook reaches1il/2Ek processing helpers, still untraced here. No
Rust timer, ACK ordering, replay or live account behavior changed.

### In-memory processing selection is not disk-commit proof (2026-11-05)

**Confirmed static,43.72:** consumer scheduling reaches1id.A08 selection. Its
first memory predicate requiresA0H=A05 and optional ordering, not a row ID;
new wrappers start inA05. Selection changes memory state toA06, then routes
selected1k4 entries by class into1rA.A03. SQLite full/corruption failures
rethrow; other selected handler failures go to1ic.A0E.
[Fourteen exact methods and remaining injection/hold/refill/error limits](apk/2.26.43.72-count-queue-processing/README.md).
This refutes a blanket inference that all processing must await disk commit.
ACK durability remains a separate conditional writer boundary. No Rust queue,
ACK ordering, replay or live account behavior changed.

### Escaped handler errors have separate NACK and completion paths (2026-11-05)

**Confirmed static,43.72:** queued router storage full/corruption errors rethrow.
Other selected escaped errors reach1ic.A0E, requesting487 for LIL or500 for
other failures and constructing completion metadata with ACK-list flagfalse.
Selected completion marks FAILED_TO_PROCESS; that false flag prevents adding
another ElB ACK from its list-generation branch. Queue enum names confirm
QUEUED/READY_TO_PROCESS/PROCESSING/PROCESSED/FAILED_TO_PROCESS are distinct
from persistence and server confirmation.
[Eleven exact methods, response/completion separation and callback limits](apk/2.26.43.72-count-handler-error-completion/README.md).
Count PQ parse LIL caught inside0v6 still follows its original normal ACK,
not this escaped-error487 path. No Rust NACK/retry/replay or live operation added.

### Failed completion disk retirement and cache removal differ (2026-11-05)

**Confirmed static,43.72:** failure callback Feh(46) calls1id.A0J for cache
retirement. In the selected transaction route,1ic.A0A separately collects
nonnull row IDs for backend1ih.A04 deletion; the callback is normally released
after successful outer completion. Its no-work path can invoke the callback
without a transaction. Cache removal alone is not universal commit proof.
1iq.A0F checks key membership only, not row/account/content identity.
[Exact methods,975-ID batches and backend/rollback/helper limits](apk/2.26.43.72-count-completion-retirement/README.md).
This is stanza-row retirement, distinct from confirmed pre-ACK watermark
deletion. No Rust deletion, replay, retry or live operation added.

### Queue transaction and retirement share a memoized database provider (2026-11-05)

**Confirmed static,43.72:** manager transaction supplier and stanza retirement
supplier both resolve provider0xc3e, then0Xd.A00. Global provider-ID caching
and owner lazy caching support the same memoized0Xp within a stable cache
lifetime. Its646a backend choice occurs once per owner, not per operation.
[Seventeen exact methods and stable-lifetime/configuration limits](apk/2.26.43.72-count-database-provider/README.md).
0Xp.A07 can construct0aq or0ar; effective branch, connection ownership,
provider reset and crash/disk ambiguity still need tracing. Shared helper
identity alone is not full atomicity proof. No Rust backend switch, ACK,
replay or live account operation changed.

### Base and alternate database wrappers share transaction primitives (2026-11-05)

**Confirmed static,43.72:**0ar delegates acquisition to0aq; both use the helper
writable-handle cache and shared0HC listener. Both virtual transaction factories
construct1hk, passing the same handle/listener and differing health state.
Within a stable provider/open-handle interval, synchronous nested retirement
reaches the same SQLiteDatabase, narrowing the normal transaction identity gap.
[Eight complete methods, assumptions and exceptional-close limits](apk/2.26.43.72-count-database-wrappers/README.md).
Health callback failure, base close, replacement/reset, ambiguous disk commit
and crash recovery remain open. No Rust backend, ACK, replay or live operation changed.

### Owned prekey inventory keys cannot be ignored or alias an ID (2026-11-05)

**Confirmed local defect, fixed offline:** prekeys.scan_prefix discarded invalid
UTF-8 names even under its owned prefix, while meta_map and record_ids silently
filtered unparsable ID suffixes. Parsed alternate spellings such as01 collapsed
onto the same numeric ID; metadata collection could thereby merge distinct
stored names. This can make corrupt publication state look like absent state
or an otherwise usable inventory. It is not a demonstrated server restriction
trigger or proof that every alias causes a resend.

Owned namespaces now reject invalid UTF-8, and record/metadata ID suffixes must
be exactly canonical decimal u32 text. Existing writers already use that form.
No new numeric wire-width/zero-ID policy is introduced; existing serialized
record/ID and upload validation remains. Unrelated namespace keys remain outside
the selected scan. The caller receives a fixed secret-free InvalidInput reason
instead of a partial inventory; no metadata is repaired, merged or deleted.

Regression replaces canonical record or outcome-unknown metadata names with
leading-zero, invalid, overflowing or invalid-UTF8 names. Old code accepted the
first alias. Inventory lookup, generation and upload admission now refuse; exact
prekey/checkpoint bytes remain unchanged and reopening still refuses. This is
local storage-integrity behavior, not a claim that native SQL IDs use this Rust
key format. Normal native inventory/wire evidence remains in this document.
No live operation, new retry, reservation reset or key regeneration was added.

### Legacy one-time prekey private/public consistency (2026-11-05)

**Confirmed local defect, fixed offline:** legacy_record_in checked the
serialized ID against its store key, and upload construction decoded the
public key, but neither derived public from the saved private half. The
inspected locked wacore-libsignal0.7.0 PreKeyRecord deserializer parses the
record/normalizes stored public encoding; its getters do not establish pair
consistency. A well-formed record could therefore publish a public key whose
saved private key cannot perform the corresponding private operation.

The shared legacy record lookup now derives and compares the public key,
returning fixed InvalidInput on mismatch or unusable private data. Upload
selection and local digest inspection use that lookup. This is local crypto
integrity, not an APK wire-field change or a claim about native corrupt-row
recovery. No key repair, replacement, rotation or retry is authorized.

Regression corrupts record50 in a generated50-key batch while retaining its
valid ID/public key. Old code returned a reserved upload. Corrected admission
returns the exact pair-mismatch error; all prekey and checkpoint bytes remain
unchanged, including rollback of earlier keys' tentative reservations. Reopen
still refuses with identical rows. Existing normal upload and independent
digest controls remain. No live account or network operation ran.

### Direct upload API validates the supplied signed private half (2026-11-05)

**Confirmed local defect, fixed offline:** begin_upload and its shared selected
upload implementation accept a caller-supplied serialized registered signed
prekey. They checked wire ID, public key and identity signature but did not
validate the private half. The earlier factory lookup guard did not protect
callers using this API directly. A valid public signature therefore allowed a
record with a mismatched private key to reserve an upload.

The shared upload entry now decodes signed.private_key, derives public and
compares it with signed_public before allocating the IQ ID or entering the
reservation transaction. Unusable private data and pair mismatch produce fixed
InvalidInput reasons. Signature and account/store ownership checks remain.
No key is repaired or replaced, and no alternative identity or request is
created. This is local input integrity, not a new APK wire requirement or a
claim about native corrupt-store fallback.

Regression preserves the signed ID/public/signature/timestamp while substituting
an independently generated private key. It explicitly confirms the public
signature still verifies. Old code returned a reserved upload. Corrected code
returns pair-mismatch, preserves exact prekey/checkpoint bytes and still refuses
after reopen; the original valid signed record then reserves successfully.
This direct-API boundary is separate from factory loading and does not claim
every other raw serialized-key API was audited. No live account operation.

### KEM batch scans refuse corrupt owned inventory (2026-11-05)

**Confirmed local defect, fixed offline:** kem_prekeys.all_rows skipped JSON
parse failures and accepted parsed records without binding their IDs to their
owned storage names. Migration could inspect sent/last_resort from rows whose
material or key binding had not been validated; replenishment could count a
partial inventory and generate around malformed records. This was distinct
from the existing strict point lookup.

The write-side batch scan now parses every owned row or refuses, requires the
exact canonical record_key(row.id), and applies existing row.validate before
returning any inventory. This covers migration detection, SET selection and
ADD replenishment. Corrupt sent or otherwise unselected rows also refuse the
whole write-side inventory. Diagnostic counts retain their documented ability
to count malformed rows; they are not upload or migration authority.

**Refuted suspected gap:** point lookup already checks row ID, lengths, signature
length and the public key embedded in expanded secret. Those checks were not
missing and were not reimplemented. This change applies them to batch admission;
it does not prove additional algebraic secret/public consistency or native
corrupt-store recovery. Existing signature/public-key upload validation remains.

Regression injects malformed JSON, mismatched row ID, leading-zero alias,
invalid-UTF8 suffix and a sent last-resort row with mismatched embedded public
material. Migration, SET selection and ADD replenishment all refuse without
changing exact prekey/checkpoint bytes; reopen still refuses. Old code failed
the malformed-JSON migration case by returning a partial result. No identity
rotation, replacement keys, new retry, live account or network operation.

### Restored KEM rows cannot override consumption markers (2026-11-05)

**Confirmed local defect, fixed offline:** consume_in atomically deletes an
ordinary KEM row and records authenticated consumption. Generation already
refuses IDs with a marker, and a missing consumed row may settle a late upload
ACK. But a stale row restored beside its consumption marker could still load,
enter batch inventory, be consumed again or be marked sent. Row presence was
incorrectly sufficient to bypass the durable marker's one-time meaning.

A live-row/marker conflict now refuses point lookup, batch inventory, repeated
consumption and present-row upload settlement. Any marker bytes beside a live
row hold that ID; malformed marker contents do not grant authority. Missing-row
late-ACK validation remains unchanged and still checks the saved marker. No
marker is erased, no key is repaired and last-resort retention policy remains.
This protects against conflicting local restore/corruption, not wholesale
rollback of both row and marker or proof of native recovery parity.

Regression installs and consumes a real KEM row, verifies missing-row late ACK
succeeds without restoring it, then restores the original row under its correct
key. Lookup, repeated consumption, migration, SET/ADD selection and present-row
settlement all refuse. Exact row/checkpoint bytes survive the refusals and
reopen. Old code failed by returning the restored row. Existing consume-before-
ACK and last-resort controls remain. No live operation or new retry authority.

### KEM upload settlement is bound to reserved public material (2026-11-05)

**Confirmed local defect, fixed offline:** PendingUpload.settle bound durable
KEM ID lists to the callback, but mark_sent_in loaded whichever valid rows
occupied those IDs. An accepted reply could mark a substituted public key as
uploaded. A missing-row consumption marker was checked by ID alone even though
it already retained a public fingerprint. IDs are not exact publication evidence.

New Batch.kem_public_sha256 stores SHA256 of every selected KEM public key,
in kem_ids wire order, in the same reservation transaction as frozen request
bytes. PendingUpload retains the identical list. All settlement first requires
list equality and matching cardinality; an accepted result then checks each
current row's validated public fingerprint or the consumed marker's ID/hash
before writing batch/key status. Existing live-row/marker conflict guards
remain. Any mismatch aborts the full transaction and preserves uncertainty.
Refusals still do not adopt current KEM public material or retry uploads.

The additive field defaults empty when reading historical Batch records.
Historical batches remain inspection/hold evidence; no callback reconstruction,
missing-hash inference, automatic migration or replay is added. Binding is
public-key identity, not a new signature/private-key equality claim or APK
wire-field requirement. Whole-store rollback remains outside this local guard.

Expanded existing uncertainty regression replaces a live KEM row with a different
well-formed pair under the same ID: old code accepted the reply. New code refuses
with exact row/checkpoint preservation. It also rejects durable fingerprint-list
tampering for result/error and a wrong consumed-public hash. Existing missing-row
refusal, consumed-before-ACK success, reopen and uncertain-no-replay controls
remain. Synthetic fixture restoration only continues independent controls;
production does not repair reservations to make replies pass. No live operation.

### Legacy upload public identity gap (2026-11-05)

**Initial finding, fixed below:** a synthetic same-ID valid legacy key replacement
was accepted by upload settlement at 9b26f558. Direct and group receive delete
legacy rows atomically but retain no consumed-public fingerprint. Full
reproducer, source path analysis and required fix matrix are retained in
[the audit](legacy-prekey-publication-binding.md). KEM binding does not cover
legacy keys. No live operation or new runtime guarantee.

### Legacy upload public binding and atomic consumption (2026-11-05)

**Confirmed local fix:** reservations now retain legacy public fingerprints;
accepted settlement checks live rows or authenticated consumed ID/hash evidence.
Direct/group receive commit consumption evidence atomically with row deletion.
Restored conflicts, unexplained loss and altered fingerprints refuse without
settlement writes; legitimate missing-row session continuation remains supported.
Historical state is not inferred or replayed. [Implementation and regression
scope](legacy-prekey-publication-binding.md#implemented-public-binding-and-atomic-consumption-2026-11-05).

### Legacy inventory validates uploaded and unselected rows (2026-11-05)

**Confirmed local defect, fixed offline:** record_ids decoded JSON byte arrays
and checked canonical store names, but did not validate the serialized prekey.
unsent_in then filtered Uploaded/OutcomeUnknown rows before strict point lookup.
A malformed, wrong-ID, private/public-inconsistent or restored-consumed row
marked Uploaded could therefore be ignored while inventory counted available
keys, generation replenished, or another upload reserved different rows.

record_ids now applies legacy_record_in to every owned row before returning IDs.
This reuses three-byte ID, stored ID, serialized-record, private/public pair and
consumption-marker conflict validation. No row is skipped or repaired. Inventory,
replenishment and upload decisions that consult it refuse the complete snapshot;
summary also refuses invalid legacy material rather than reporting normal counts.
An earlier saved-refusal hold may return without consulting inventory, as before;
it grants no reservation or retry authority. KEM diagnostic counts retain their
separate existing behavior. Valid uploaded/uncertain rows remain excluded from
unsent capacity only after validation. There is no new APK wire-field claim.

Regression legacy_inventory_refuses_corrupt_unselected_uploaded_rows marks row50
Uploaded, leaving49 valid unsent rows. Four independent fixtures inject malformed
serialized bytes, a different stored ID, a validly encoded mismatched private
half, or a consumption marker beside the live row. Inventory, wants_upload,
generation and upload all refuse without changing exact key/checkpoint rows,
including after reopen. Old code failed the first inventory refusal assertion.
The existing target-count generation test now uses valid serialized records in
its synthetic bulk inventory instead of one-byte placeholders, preserving its
count/target/wrap assertions. No production regeneration around corrupt rows or
live account operation was added. Whole-store rollback remains outside the guard.

### PQ digest uses validated KEM inventory (2026-11-05)

**Confirmed local fix:** gated LR digest selection now shares all_rows validation
with publication. Corrupt unselected ordinary/unsent records can no longer hide
behind LR/status filtering and produce Matches. Four injected corruption cases
preserve exact rows on refusal; existing published-signature controls remain.
[Scope, reproducer and limits](prekey-count-notifications.md#pq-digest-rejects-corrupt-unselected-kem-rows-2026-11-05).
No live request, resynchronization or replay authority.

### Upload settlement checks reserved account authority (2026-11-05)

**Confirmed local fix:** PendingUpload now retains a publication-authority hash
and persists it in Batch. Settlement rereads the account and checks transaction
generation, registered authority and durable binding before status writes.
Nine changed/missing authority cases refuse result/error across reopen; unrelated
authentication-time updates still settle. [Scope and remaining write/rotation
limits](prekey-count-notifications.md#upload-settlement-retains-publication-account-authority-2026-11-05).
No live traffic, replay or inferred historical binding.

### Count completion key removal and conditional rescheduling (2026-11-05)

**Confirmed static, APK 2.26.43.72:** FgR selector8 removes every cached stanza
whose 2DG key is in the completion set; only removed non-QUEUED entries increment
its delta counter. 07W.A0S/A0T/A0R prove remove-if semantics on RandomAccess and
iterator paths. 1iq.A03 sends the negated nonzero count to its optional callback.
These are memory/counter effects, not SQL retirement or server confirmation.

1id.A0J later calls 1im.CHt with remaining noncompleted cache count. Physical CHt
census finds the abstract interface and 1il/1in/1iw implementations. Base1il
ignores the integer and conditionally calls its existing scheduler through A06:
1iz uses config0x6f06; other selected kinds use1kB.A00. Effective DI/configuration,
counter storage, key lifetime and complete rejection/recovery policy remain
open. Completion is a scheduling stimulus, not eventual work or ACK ordering.
[Complete exports, commands, checksum manifest and boundaries](apk/2.26.43.72-count-completion-rescheduling/README.md).
No Rust runtime or live account operation changed.

### Count queue consumer construction is resolved (2026-11-05)

**Confirmed static, APK2.26.43.72:** queue1ix provider0xd04 follows1Uw.A00
page0 ->A01 range(0xc10,0xd29] ->A0T case3332 ->0Zy.A0J ->new1iz.
This closes the selected stable-provider construction unknown:1iz inherits
base1il.CHt and uses its0x6f06 configuration branch. It does not use the
separate1in/1iw ordered overrides on that construction path.

The1iz hold-owner binding0xd09 resolves through case3337 ->0Zy.A0I ->new1k7;
its A0A AtomicBoolean initializes false. Later mutation/value/release is open.
00E.A15 passes null Boolean to A0D, not false; neither the declared default
nor effective0x6f06 value is established. Shared provider-cache lifetime,
pause/hold/capacity/CAS, rejection and crash boundaries remain distinct from
job execution, ACK ordering or disk/server confirmation.
[Complete branch exports, reproduction and limits](apk/2.26.43.72-count-consumer-binding/README.md).
No Rust runtime change, replay authority or live operation.

### Unordered hold mutation and conditional release (2026-11-05)

**Confirmed static, APK2.26.43.72:** selected2JI runner holds unordered pulls
behind loaded ordered metadata when0av.A0U is true. Arm numberA00, change
counterA0B, hold atomicA0A and one-batch bypassA01 have distinct roles.
Ordered completion1k7.A03 clears the hold and asks the scheduler to run only
when no loaded ordered metadata is pending and the old hold was true.

The FeQ selector21 timer checks its captured arm. With ordered work pending,
either unchanged nonnull saved head (OR, not both) or two matching null head
pairs can permit a one-batch bypass; changed heads otherwise rearm the timer.
The runner separately detects concurrent release throughA0B. Pending checks
skip unloaded caches and inspect key-map size, not SQL or in-flight states.
No fixed hold delay/default is inferred from0av.A06, and full executor/timer
failure semantics and ACK ordering remain open.
[Full exports, exact predicates, direct field census and limits](apk/2.26.43.72-count-hold-release/README.md).
No Rust queue change, retry/replay authority or live account operation.

### Hold gate/delay descriptor and registry inputs (2026-11-05)

**Confirmed static, APK2.26.43.72:**0av constructor samples hold gateA0U from
0ax.A0C(key0x9952), whose descriptor Boolean inputs are false/false. When false,
it stores delayA06=0; when true, it reads0ax.A0G(key0x9953), widens the signed
int and stores it, without a local clamp. That descriptor's inputs are60000/
60000; the timer log labels the value milliseconds. Scheduler gate0x6f06 is
separate. Direct write censuses find one constructor write forA0U andA06 each.

Selected08Y default-map entries showfalse for0x9952/0x6f06 and60000 for0x9953.
The selected fallback resolver checks cached values, registry defaults before
descriptor fallback, then saved preferences. These declared inputs/entries do
not establish effective runtime configuration or enable a fixed Rust timer.
Full outer MobileConfig precedence, refresh/provider lifetime, complete map
collision/value-flow and scheduler handling remain open. Large source methods
are retained as explicit bounded excerpts with full-source hashes/ranges.
[Exports, selected data flow and coverage limits](apk/2.26.43.72-count-hold-config/README.md).
No Rust network change, replay permission or live operation.

### Queued prekey uploads recheck ownership at writer dequeue (2026-11-05)

**Confirmed local defect, fixed offline:** begin_upload_selected checks account
authority when reserving and settle checks it on reply, but NodeLink previously
handed a queued Node directly to the writer. An account change committed after
reservation/handoff could therefore reach encoding and transmission before a
later settlement refusal. Reservation-time checks are not writer admission.

Outbound is now an internal envelope with an optional prekey guard. All three
maybe_upload triggers attach the reserved PendingUpload and store handle;
ordinary sends retain their existing path. After dequeue, before encode or
writer.write, shared Outbound.prepare invokes check_write_authority in one
read snapshot. It requires registered current authority, matching transaction
generation and the callback's account hash, plus an outcome_unknown batch with
unchanged account binding, legacy/KEM IDs and public-hash vectors/cardinality.
A refusal sends the actual local error to the waiting actor and stops the real
writer epoch before encoding/writing that node. It does not settle, clear,
repair or replay the reserved batch. Rounds/pending ownership stay consumed.

queued_prekey_upload_rechecks_authority_before_writer_encoding holds an actual
injected outbound envelope after reservation/handoff, then revokes the account
or tampers its durable batch binding. Shared preparation refuses before the
test's encoding counter advances; Account/Prekeys/Checkpoints remain byte-for-
byte unchanged by preparation and the caller receives failure with its pending
claim/round retained. An authentication-time-only update still encodes once.
The initial regression used an accepting preflight to preserve the previous
unguarded writer behavior and failed on revocation. Real writer code invokes
the same preparation immediately before encoding; this is not a live socket
observation. Existing fake-writer helpers now execute preparation too.

This detects changes already committed when the writer's read snapshot opens.
It does not lock account/store state across encoding or an awaited socket write,
prevent changes after preflight, validate same-identity signed-key rotation, or
make network and store effects atomic. Direct callers that transmit the returned
Node through their own I/O do not gain this actor guard. Local key mutation after
reservation and before write remains a separate material-lifetime concern.
No new APK callback policy, wire fields, reconnect/retry authority or live
account operation is claimed.

### Queued publication requires live original key material (2026-11-06)

**Confirmed local defect, fixed offline:** writer dequeue checked account and
durable batch bindings but did not reread selected key rows. Replacing a legacy
row with a different valid pair under the reserved ID still let the old frozen
public bytes reach encoding even though their private key was no longer held.
Missing/consumed KEM or legacy rows had the same admission gap. Later settlement
checks could refuse an ACK, but could not undo that publication.

check_write_authority now checks every selected legacy key's OutcomeUnknown
metadata and IQ owner, loads it through strict legacy_record_in, requires a live
row and compares its public fingerprint with the reservation. Every selected
KEM key must likewise pass strict load (including live-row/consumption-marker
conflict validation), exist and match the original public fingerprint. All checks
share the existing read snapshot at dequeue, before encoding/socket write.

A valid consumed-public marker is deliberately insufficient before first write:
publication needs a live private row, whereas settlement after transmission can
legitimately accept authenticated consume-before-ACK evidence. The fix leaves
that late-ACK behavior intact. There is no marker deletion, private-key recovery,
new key generation or reservation release to make admission pass.

Expanded queued_prekey_upload_rechecks_authority_before_writer_encoding uses
actual held injected outbound envelopes. New cases cover a well-formed legacy
pair replacement, missing legacy row, real legacy consumption, changed legacy
metadata status, well-formed KEM pair replacement, missing KEM row, real KEM
consumption, and restoration beside its real marker. Every case refuses before
encoding, preserves Account/Prekeys/Checkpoints exactly after the injection,
and leaves the pending claim/session round consumed. Existing revocation,
batch-binding and authentication-time positive controls remain. The old code
failed the first added replacement case. These are synthetic local-store/channel
checks, not server or real socket observations.

Snapshot-to-write races remain after admission. This binds selected public
identity and reuses existing private/material consistency validation; it does
not prove arbitrary KEM private algebraic consistency or bind every signature,
role/status metadata byte to the original row. Same-identity signed-key rotation,
custom direct I/O and whole-store rollback remain outside this guard. No APK
wire change, replay authority or live account operation.

### Upload deadline fences writer and direct settlement admission (2026-11-06)

**Confirmed local defects, fixed offline:** a reserved upload could wait in the
outbound queue until its30-second reply deadline, then still reach encoding and
transmission. Separately, PendingUpload.settle itself did not check that deadline;
only the actor's reply dispatch did, so direct library callers could adopt an
expired result/error. These are local claim-lifetime gaps, not newly inferred
APK timeout values or observed server restriction causes.

A shared check_deadline rejects Instant::now() >= the original pending deadline.
Writer preflight checks before store/material inspection and again at its end,
so validation cannot deliberately renew the claim or admit an already expired
request. Direct settle checks after exact IQ matching and again in the write
transaction after ownership/material validation, before status writes. Deadline
failure leaves the uncertain batch/key ownership untouched. The existing writer
envelope reports its local error and stops the epoch before encoding that node.
No fresh deadline, metadata release, replacement upload or automatic replay.
Expiry uses the dedicated local InvalidInput field prekey_upload_deadline.
The actor maps only that context to a captured late reply and continues receive
if the deadline crosses during settlement; storage/ownership errors still
propagate. The queued-expiry regression also exercises this dispatch helper.

queued_prekey_upload_cannot_start_at_or_after_reply_deadline holds a real
injected outbound envelope and uses paused monotonic time:29,999ms admits one
encoding;30,000ms and30,001ms refuse without encoding. It preserves exact
Account/Prekeys/Checkpoints, pending/round consumption and50 uncertain keys,
including after reopen; there is no replacement write. The old writer failed
at exactly30,000ms.

Direct regression direct_upload_settlement_requires_an_unexpired_callback_claim
covers result and error at the same three boundaries. Before the deadline both
retain their normal meaning; at/after it neither changes keys/checkpoints,
including on a repeated held callback after reopening. Old direct settlement
failed on the exactly-expired result. Existing actor late-reply capture and
consume-before-ACK controls remain separate tests.

This uses the existing session's monotonic deadline, not reserved_at wall time,
server timestamps or inferred historical age. It does not restore callbacks on
reconnect/restart or guarantee the deadline stays unexpired across later encoding,
socket I/O or transaction commit. Those operations are not atomic with the clock.
No APK wire field, configured timeout length, native hold delay, retry authority
or live account behavior changed. Tests use synthetic stores/injected channels.

### Device-query deadlines fence ingestion and queued writes (2026-11-06)

**Confirmed local fixes:** public direct/cursor ingestion now rejects expired
claims with transaction rollback; cursor expiry defers users despite fresh
revisions. Standalone receive continues on classified expiry; grouped sends
block without adopting lists. Standalone/group device-query envelopes retain
the original deadline and refuse expired writes before encoding. Existing
prekey guards remain in the typed envelope. [Boundary regressions, coverage and
remaining limits](device-query-deadlines.md). No wire/timeout-length change or
live account operation.

### Device-query durable user/revision ownership (2026-11-06)

**Confirmed local fix:** writer dequeue and ingestion now require the saved
uncertain query's users/revisions to exactly match the original callback.
Changed/missing bindings refuse before encoding or application; historical
missing revisions are not inferred. Newer pending notifications remain distinct
from the original query binding. [Regression matrix, snapshot scope and limits](device-query-deadlines.md#durable-userrevision-ownership-2026-11-06).
No repair, new retry policy or live operation.

### Key-batch continuation validates saved answered evidence (2026-11-06)

**Confirmed local fix:** reserve_next now re-decodes and validates every saved
Answered reply before reserving another page or returning completion. It shares
exact reply/query/target/bundle checks with assembled. Corrupt earlier wire can
no longer authorize an extra query from its status flag alone. Stored summary
counts remain diagnostic. [Regression matrix and scope](device-key-batches.md#saved-answered-pages-cannot-authorize-continuation-by-status-alone-2026-11-06).
No retry, pin replacement, wire change or live operation.

Directory key mixed-result parsing: shared bundle and batch adoption now follow APK43.72 outer-type dispatch; retained offsets, regression and separate diagnostic limits in [device-key-batches](device-key-batches.md#directory-result-and-error-branch-selection-2026-11-06). Offline only.

Peer directory result/error attribution now follows saved outer type and selected user independently of generic action status. Typed peer-key root-error fallback corrected; historical outcomes retained without refresh/replay authority. [APK offsets, tests and remaining identity-only/foreign-action limits](peer-identity.md#peer-directory-inspection-uses-reply-context-2026-11-06). Offline only.

Identity-only reply context and signed error codes now match the pinned43.72 FeM/FUP/0ln path. [Trace, regression, API type widening and remaining timer/adoption limits](peer-identity.md#identity-only-reply-context-and-signed-codes-2026-11-06). Offline only.

Full-directory action refusal now binds to the original selected peer instead of first foreign nested error. Shared context guard, ordering/reopen tests and negative duplicate-list refresh check: [audit](peer-identity.md#full-directory-action-errors-belong-to-the-original-peer-2026-11-06). Historical Error and other-family semantics unchanged; offline only.

Local directory refresh and inspection now validate original single-peer request ownership, action kind and reply correlation before relying on signed success. [Eight corruption/reopen cases, unchanged state and limits](peer-identity.md#explicit-directory-refresh-retains-request-ownership-2026-11-06). Offline integrity guard, not an added server requirement.

Identity-refresh staging/application now validates producer request ownership and freezes exact request bytes plus action kind across derivation/commit. [Confirmed race regression, local request-shape checks and remaining prior-notification gate](identity-refresh-application.md#original-request-authority-across-staging-and-commit-2026-11-06). No live operation or new server requirement.

Prior identity-notification success gate now validates action kind, original request/device and correlated reply before another read. Removed the last reply-only success helper. [Seven reopen/unchanged-state cases and native-policy limits](identity-notification-key-read.md#prior-notification-success-retains-request-ownership-2026-11-06). Offline only.

Identity-refresh typed action/inspection now follow pinned43.72 outer-type and exact-device error context; signed codes retained as i32. [Verified writer/callback hashes, end-to-end guarded adoption controls and limits](identity-refresh-application.md#identity-refresh-resulterror-context-2026-11-06). Offline only.

Outer typed directory refusals now take generic action diagnostics only from one enclosing root error, never nested device errors. [Pinned callback sources, missing/duplicate-root matrix, reopen and no-replay controls](peer-identity.md#outer-directory-errors-cannot-inherit-nested-refusal-diagnostics-2026-11-06). Scope is diagnostic; no request/retry changes.

Identity-only callback now has its pinned32-second session timer, anchored before send; expiry preserves uncertain Sent and never recreates retry authority. [Producer/registration evidence, PN/LID paused-clock cases, competing claims and writer limits](peer-identity.md#identity-only-callback-lifetime-2026-11-06). Offline only.

Current43.72 identity ACK dispatch re-anchored through15v/Fc1/17o/0YM/0v6 plus held-key lookup: normal ACK does not require bundle adoption, and general exceptions differ from LIL. [Complete methods, negative comparison and scheduling/routing limits](identity-notification-ack.md#current-apk-identity-dispatch-and-ack-independence-2026-11-06). Research only; no ACK widening or live operation.

Current identity-triggered manager flags and suppression scoped: conditional PQ fetch is not a session upgrade; REGULAR/PQ differ in in-flight keys; strict60-second uptime cleanup is not durable retry authority. [Complete current methods, gates and remaining job/wire limits](bulk-prekey-refresh.md#current-notification-fetch-flags-and-manager-suppression-2026-11-06). Research only; no additional request/replay.

Concrete bulk future/runner trace closes a prior1ue-effects gap: IQ result/error fallback can complete the job normally; disconnect/future exception is separate from server refusal and key acceptance. [Typed future, runtime-exception rethrow, requirement/defer and cancellation limits](bulk-prekey-refresh.md#bulk-job-completion-is-not-fetched-key-acceptance-2026-11-06). Research only; no retries added.

Current43.72 bulk-key preflight is conditional on filtered candidates and waits before dispatch; queue submission is not wire success or mapping adoption. [Conditional missing-LID preflight and remaining wire/adoption limits](bulk-prekey-refresh.md#bulk-pre-key-missing-lid-preflight-2026-11-06). Research only; no live operation.

APK43.72 missing-LID preflight initial flags and actual queue combination retained. [Preset, queue combination and remaining wire limits](bulk-prekey-refresh.md#missing-lid-preset-and-queue-combination-2026-11-06). Final wire/adoption remains unknown; no live operation.

Current43.72 missing-LID preset-to-multi-query route and nested128-second send/wait boundaries retained. [Multi-query route and nested timing limits](bulk-prekey-refresh.md#missing-lid-multi-query-route-and-nested-timing-2026-11-06). No universal timeout or runtime parity claim; offline only.

Confirmed local LID-repair authority gap corrected: original request and current correlated result must qualify atomically before metadata repair. [Original request authority, nine corruption/reopen cases and limits](chat-actions.md#lid-repair-retains-original-request-authority-2026-11-06). No native wire or live-operation claim.

Local contact projection split-read gap corrected: enumeration no longer supplies stale reply/status authority across a later write transaction. [Transaction snapshot fix, stale-enumeration controls and limits](contact-projection-correlation.md#transaction-bound-settlement-projection-2026-11-06). No native wire or live-operation claim.

Confirmed local nested-field ambiguity corrected for contact projection/repair, with both-order and reopen controls. [Nested duplicate-field controls and native-policy limits](contact-projection-correlation.md#duplicate-field-authority-is-not-selected-by-order-2026-11-06). Android duplicate semantics remain unverified; no live operation.

APK43.72 LID Boolean/mask mapping and conditional MEX final dispatcher retained; classic usync writer alone is not this producer's wire oracle. [LID field chain, conditional MEX selection and remaining serializer/callback limits](bulk-prekey-refresh.md#missing-lid-field-mapping-and-mex-selection-2026-11-06). Offline research only.

Current43.72 missing-LID MEX handlers bound by constructors: request refusal can release a future normally; result-state installation and scoped errors remain separate. [MEX request/scoped/exception outcomes and remaining lifecycle limits](bulk-prekey-refresh.md#missing-lid-mex-callback-completion-and-error-scope-2026-11-06). Offline only.

Current43.72 response-state lifecycle narrowed: selected nonempty-read reset and drained/exception cleanup exist; per-batch and late-callback guarantees remain unknown. [Explicit resets/cleanup and remaining late-callback limits](bulk-prekey-refresh.md#missing-lid-shared-response-state-resets-2026-11-06). No native race or live-operation claim.

Current43.72 MEX lookup operation ownership/default and conditional non-Pando send timeout retained; no blanket cancellation or native-race claim. [Operation handle ownership, conditional32-second send and remaining transport limits](bulk-prekey-refresh.md#missing-lid-mex-operation-ownership-and-nested-send-timeout-2026-11-06). No runtime or live operations.

APK43.72 non-Pando MEX expiry removes ordinary pending ownership before scheduling delivery failure; local471 reaches exceptional future completion. [Registry expiry, typed delivery failure and late-reply limits](bulk-prekey-refresh.md#missing-lid-non-pando-mex-expiry-outcome-2026-11-06). No runtime or live operations.

APK43.72 explicit exactly-one MEX result check closes a Rust first-child ambiguity; duplicate result cannot fall back to update. [Pinned cardinality evidence, both-order regression and limits](mex.md#duplicate-result-payload-rejection-2026-11-06). No live operation.

Current43.72 MEX envelope expected values are checked, not defaults: original request ID and typed to/from equality precede selected model parsing. [Request ID/from equality evidence and payload-only Rust boundary](mex.md#original-request-mex-envelope-correlation-2026-11-06). No live operation or complete Rust admission claim.

APK43.72 selected MEX server JID equality and diagnostic-only compliance path retained; no blanket parser rejection or alias policy inferred. [Concrete server equality, diagnostic-only compliance check and remaining decoder limits](mex.md#selected-mex-server-jid-equality-2026-11-06). Offline only.

Current43.72 shared MEX request envelope comparison found no correction justified; native assertion/return semantics retained. [Current shared-envelope comparison and assertion limits](mex.md#current-mex-request-envelope-comparison-2026-11-06). No live operations.

Current43.72 UsyncQuery packaged server ID and resolver chain retained with exact decimal identity; runtime/Pando/adoption remain separate. [Pinned server ID versus client ID, exact asset and resolver scope](bulk-prekey-refresh.md#missing-lid-usyncquery-server-id-2026-11-06). No live operation.

Current43.72 UsyncQuery variable spine retained: raw typed JID, nested uppercase context, conditional flags without automatic false defaults. [Raw-JID targets, MESSAGE telemetry and omitted-flag semantics](bulk-prekey-refresh.md#missing-lid-usyncquery-variables-spine-2026-11-06). No live operation.

Current43.72 MEX cached device scalar types/gates retained; null initialization differs from omitted include flags, with final null serialization unresolved. [Device cache gates, decimal-string timestamps and unresolved authority](bulk-prekey-refresh.md#missing-lid-mex-device-cache-fields-2026-11-06). No live operations.

Current43.72 device-hash bodies/input selection reanchored; no Rust mismatch found for explicit formats and other-user primary inclusion. [Current formatter comparison, existing vectors and scope limits](cross-device-sync.md#current-apk-device-hash-format-comparison-2026-11-06). Offline only.

Current43.72 self-device hash helper gap narrowed with source filtering and own-account address conversions; not a generic alias rewrite. [Own-account source filtering, device-preserving mapping and limits](cross-device-sync.md#current-self-device-hash-input-normalization-2026-11-06). No live operation.

APK43.72 hosted LID device99 distinction retained; current Rust domain guard avoids unsupported normalization. [Hosted LID device99 boundary and remaining dispatch/authority limits](cross-device-sync.md#hosted-lid-device99-hash-boundary-2026-11-06). No live operation.

MEX map-constructor null behavior narrowed using pinned AOSP sources and APK call site; put(null) removal is a different API. [MEX null serialization platform boundary](bulk-prekey-refresh.md#missing-lid-mex-null-serialization-platform-boundary-2026-11-06). No live operation.

APK43.72 MEX IQ error matchers/wrappers traced; ignored backoff read does not establish a retry duration. [MEX IQ error families and backoff boundary](mex.md#current-mex-iq-error-families-and-backoff-boundary-2026-11-06). Offline only.

APK43.72 MEX error telemetry/product dispatch gap closed; exact raw-error field inventories bounded without claiming global nonuse. [MEX error delivery versus telemetry](mex.md#current-mex-error-delivery-versus-telemetry-2026-11-06). No live operation.

APK43.72 authkey_rotation request/ACK ordering and response/local-key replacement boundaries retained. [Auth-key rotation lifecycle and missing Rust obligations](notification-ack-audit.md#auth-key-rotation-is-a-state-lifecycle-2026-11-06). No Rust runtime change.

APK43.72 auth-key public-wire/private-public-storage distinction and bounded readback retry retained. [Auth-key candidate and storage verification](notification-ack-audit.md#auth-key-candidate-and-storage-verification-2026-11-06). No live operation.

APK43.72 auth-key commit versus counter apply, record parser and reload gates retained; abstract provider declarations explicitly unresolved. [Auth-key commit and reload boundary](notification-ack-audit.md#auth-key-commit-and-reload-boundary-2026-11-06). No live operation.

APK43.72 auth-key slot0x1a3e concrete provider and local crypto record field roles retained, without claiming platform execution or wire-format necessity. [Auth-key concrete storage provider](notification-ack-audit.md#auth-key-concrete-storage-provider-2026-11-06). No live operation.

APK43.72 rotation trigger inventory and saved-token send ordering retained; exact candidate-field census does not establish restart recovery. [Auth-key rotation triggers and candidate ownership](notification-ack-audit.md#auth-key-rotation-triggers-and-candidate-ownership-2026-11-06). No live operation.

Auth-key/token evidence chains reconciled; earlier token findings replace globally stale unknowns in recent scope notes. [Canonical auth-key implementation boundary](authkey-lifecycle.md#current-implementation-boundary-reconciled-2026-11-06). No runtime change or live operation.

Confirmed local login material-copy mismatch now refuses before network/reservation; existing optional-record and feature boundaries retained. [Registration-copy consistency before login](authkey-lifecycle.md#registration-copy-consistency-before-login-2026-11-06). No native wire or live claim.

Confirmed local await-interleaving gap closed at login reservation using zeroized account/material snapshot comparison. [Login reservation binds its preflight snapshot](authkey-lifecycle.md#login-reservation-binds-its-preflight-snapshot-2026-11-06). Post-reservation generation ownership remains open; no live operation.

Local login result now requires current credential snapshot and exact unknown reservation owner before atomic settlement. [Login reply reservation and credential ownership](authkey-lifecycle.md#login-reply-owns-its-reservation-and-credentials-2026-11-06). Success/failure, stale/duplicate and reopen regressions; no live operation.

CLI explicit refusal recheck now checks exact current row/history in its archive-delete transaction; stale reads cannot clear uncertainty. [CLI refusal recheck authority](authkey-lifecycle.md#cli-refusal-recheck-retains-current-authority-2026-11-06). No live operation or server-policy claim.

Local CLI Noise-half consistency now checked before side effects; fixed diagnostics and valid/mutated/absent controls retained. [CLI Noise preflight before side effects](authkey-lifecycle.md#cli-validates-noise-halves-before-side-effects-2026-11-06). No wire or live acceptance claim.

CLI account preflight now validates present registration material through the existing shared invariant, preserving optional absence and refusing malformed state before side effects. [CLI registration-copy preflight](authkey-lifecycle.md#cli-account-load-checks-the-registration-copy-2026-11-06). Offline only.

CLI reservation/result await gaps closed using shared redacted LoginSnapshot and local owner ID, retaining durable capture before stale refusal. [CLI login snapshot and reservation ownership](authkey-lifecycle.md#cli-login-snapshot-and-reservation-ownership-2026-11-06). Not a session-wide lock or rotation recovery claim.

Local CLI malformed-present LID omission closed via shared success identity helper, keeping canonical/legacy LID coherent and partial fields intact. [Shared CLI login identity validation](login-identity-updates.md#cli-shares-the-authenticated-identity-update-2026-11-06). No native or live acceptance claim.

Local CLI direct-send cached-intent overwrite corrected with transaction-bound expected/current equality before ACK settlement. [CLI direct-send ACK intent authority](replay-safety-audit.md#cli-direct-send-ack-retains-intent-authority-2026-11-06). Reopen/duplicate/session-preservation controls; no native wire change.

CLI group ACK cached-intent overwrite/promotion gap closed using shared direct-send ownership guard before distribution confirmation. [Group ACK intent and distribution authority](replay-safety-audit.md#cli-group-ack-owns-intent-before-distribution-promotion-2026-11-06). No wire or handset-delivery claim.

Local out-of-band sender-key child write/settlement ownership now durable before network write and bound to current parent intent. [Separate distribution reservation ownership](replay-safety-audit.md#cli-separate-distributions-reserve-before-write-2026-11-06). No retry, wire grammar or native-policy claim.

Local media-upload cached-state overwrite gaps corrected across header/response/uploaded transitions with independent HTTP evidence capture. [Media-upload response reservation ownership](replay-safety-audit.md#media-upload-response-transitions-retain-reservation-ownership-2026-11-06). No wire-schema or live acceptance claim.

Ordinary media GET settlement now binds success and HTTP refusal to a unique local reservation; client no-redirect/no-retry settings verified unchanged. [Media GET result reservation ownership](media-network-outcomes.md#download-result-reservation-ownership-2026-11-06). No native timeout or live claim.

Local public upload eligibility omission corrected at shared entry, including cached-success preflight and unchanged-state controls. [Shared upload account/config preflight](media-network-outcomes.md#shared-upload-preflight-precedes-cache-and-reservation-2026-11-06). No native account-gate or live claim.

2026-11-06 — **Confirmed Rust/offline, negative audit:** existing media refresh ownership, conditional host merge and TTL guards verified; stale first-route-false documentation corrected. No runtime change or new APK/live evidence. [Scope and remaining gaps](media-network-outcomes.md#refresh-authority-and-lifetime-audit-2026-11-06).

2026-11-06 — **Confirmed Rust/offline:** retry key-refetch ACK/query queue ownership gap fixed with the reservation-time snapshot at each writer admission. Changed/missing/settled controls preserve checkpoints across reopen; no new timer or native-format claim. [Evidence and boundaries](retry-ack-dispatch.md#queued-key-refetch-writes-retain-reservation-ownership-2026-11-06).

2026-11-06 — **Confirmed Rust/offline:** live retry-refetch callbacks now retain the written job snapshot, preventing same-ID replacement jobs from being consumed by an old response. Registration ownership/collision checks and result/error/reopen controls; no new APK timing or live claim. [Written-job callback binding](retry-answers.md#live-callbacks-retain-the-written-job-2026-11-06).

2026-11-06 — **Confirmed Rust/offline:** saved-session retry, fresh-key query reservation and fresh-bundle resend now recheck local Signal/account authority in their commit transaction. Stale-account real-crypto controls and generation/key/address/metadata/reopen cases; existing original/peer guards retained. [Account authority boundary](retry-answers.md#retry-reservations-recheck-local-account-authority-2026-11-06). No new native storage or live claim.

2026-11-06 — **Confirmed Rust/offline:** canonical retry plaintext retention is immutable/idempotent; saved-session and fresh-bundle resend commits now bind the source snapshot used by crypto. Conflict/corruption/transaction rollback/reopen controls; no native storage-format or live claim. [Source integrity boundary](retry-answers.md#original-retry-plaintext-retention-and-commit-binding-2026-11-06).

2026-11-06 — **Confirmed static/Rust offline:** current presence reader bound through1rA.A02 to1rF and inherited A03; missing type takes available branch. Rust metadata default corrected; separate device/group routes and last-seen signed-long/clock projection gaps retained. [Current presence audit](presence.md#current-readerdefault-type-audit-2026-11-06). No outgoing traffic or live acceptance claim.

2026-11-06 — **Confirmed Rust/offline:** normal queued action writes now retain exact Sent-row ownership and existing producer deadline through writer admission. Replacement/terminal/expiry/untimed/slow-completion/reopen controls; no new native duration or replay claim. [Writer authority and remaining callback gap](action-settlement-ordering.md#queued-action-writer-ownership-and-deadlines-2026-11-06).

2026-11-06 — **Confirmed Rust/offline:** action callbacks retain the written Sent-row snapshot and validate it in the settlement transaction before response interpretation/sequence allocation. Duplicate owner registration refuses; failed/expired/disconnected callbacks cannot revive. Result/error/reopen/journal controls; no new native timing or live claim. [Written action callback ownership](action-settlement-ordering.md#action-callbacks-retain-the-written-reservation-2026-11-06).

2026-11-06 — **Confirmed Rust/offline:** normal queued writer admission now rechecks recorded group-create reach-out restrictions and explicit app-state fetch/patch/held-fetch backoff in the same read snapshot as reservation ownership. Late-gate, inactive/expired/malformed, unrelated and reopen controls; no new native duration or replay. [Late writer admission](action-settlement-ordering.md#writer-rechecks-recorded-restrictions-and-explicit-backoff-2026-11-06).

2026-11-06 — **Confirmed Rust/offline:** actor and CLI receipt writes now retain every reserved batch member and recheck exact ownership plus the existing late automatic-read privacy gate before encoding. Single/mixed/group/non-root/correlation/origin/reopen controls; legacy low-level contracts retained. [Receipt writer handoff](replay-safety-audit.md#receipt-writer-owns-every-reserved-batch-member-2026-11-06). No new wire/native timing or live claim.

2026-11-06 — **Confirmed static/Rust offline:** pinned pre-class numeric459 diversion now excludes ordinary receipt ACK settlement too. Failing-before/passing-after direct/group/single/batched/Unicode/reopen controls preserve uncertainty; normal ACK/refusal controls still settle once. [Receipt auth-challenge exclusion](receipt-ack-special.md#current-auth-challenge-diversion-precedes-receipt-settlement-2026-11-06). No challenge recovery, retry or live claim.

2026-11-06 — **Confirmed static/Rust offline:** shared root ACK signed-integer admission excludes malformed/out-of-i32 errors before CLI classification, direct send, distribution and receipt terminal settlement. Failing-before classifier regression, stored reservation/reopen controls and existing reader LIL catch interpreted; no native error-response/recovery parity claim. [ACK numeric boundary](message-ack.md#integral-error-admission-before-ordinary-ack-outcomes-2026-11-06).

2026-11-06 — **Confirmed static/Rust offline:**1rg.A04's initialized recoverable root-tag set excludes ACK; the selected outer reader catch reports error2 and returns. Actor/CLI stateful receive and captured waits now stop after capture without settling/resetting pending work; legacy wait stops under its existing contract. [Reader lifetime, regressions and remaining recovery limits](message-ack.md#malformed-ack-consumes-the-receive-epoch-after-capture-2026-11-06). No NACK-to-ACK, automatic reconnect or live claim.

- **2026-11-06 confirmed pinned static / Rust correction:** ACK fact construction now reuses the signed Java-i32 admission parser instead of reparsing as ASCII u32. Signed/Unicode codes survive reopen; historical u32 facts remain readable in the wider i64 model. No new settlement/replay authority. [Evidence, regression and compatibility](message-ack.md#preserve-admitted-signed-ack-error-codes-2026-11-06).

- **2026-11-06 confirmed pinned static / Rust correction:** 1rH.A03 calls required 0Tp.A0L(id) before error459/class dispatch; absent id throws LIL into the selected reader exit. Shared stateful validation now captures then stops; empty text is permitted by the accessor. [Evidence and regression scope](message-ack.md#required-ack-id-before-error-and-class-routing-2026-11-06).

- **2026-11-06 confirmed pinned static / negative finding:** ACK optional typed JID accessor logs protocol-compliance failures and returns null on parse/requested-class failure; this does not justify a blanket fatal-address gate. Recipient-preferred message routing and generic callback authority remain open. [Complete selected exports and limits](message-ack.md#optional-typed-ack-addresses-are-not-a-blanket-fatal-gate-2026-11-06).

- **2026-11-06 confirmed pinned static / Rust correction:** message ACK key prefers typed recipient, otherwise normalized from. New selected-method/type-census evidence binds plain PN/LID behavior; ordinary outbox settlement now respects it, retaining conflicting reservations and once-only facts. Other families/adapters remain open. [Evidence and regression](message-ack.md#select-the-message-ack-chat-before-outbox-correlation-2026-11-06).

- **2026-11-06 confirmed pinned static / Rust correction:** retained native message-key address selection now also guards distribution Pending matches and CLI direct/group/status/distribution outcome classification. Actual destination plus original id are required; wrong-chat results retain uncertainty. [Evidence and tests](message-ack.md#bind-distribution-and-comparison-acks-to-the-expected-chat-2026-11-06).

- **2026-11-06 confirmed pinned selected static / Rust correction:** refreshed43.72 participant conversion, event19 envelope and success consumer with explicit bot-feedback exception. Ordinary PN/LID participant guard is now shared across outbox, distribution and CLI expected-chat classification. Refusal/challenge paths remain distinct. [Evidence, tests and limits](message-ack.md#share-ordinary-participant-success-admission-2026-11-06).

- **2026-11-06 confirmed selected pinned static / negative finding:** receipt generic ACK callback removes a full Eoh key, with no receipt edited-key fallback; callback BsT(node) alone is not success. Subsequent independent id-based maps still change and can deliver another callback, so universal full-key ownership remains unproven. Current strict Rust receipt correlation unchanged. [Sources and remaining authority gaps](receipt-ack-special.md#current-generic-callback-key-and-separate-id-maps-2026-11-06).

- **2026-11-06 confirmed selected pinned static / negative finding:** direct field/caller census binds ID-map insertion to tracking flag, retained original Message/callback/count, confirmed VoIP source and native reconnect/challenge resubmission. No ordinary receipt producer established; strict Rust receipt matching and no-replay policy unchanged. [Evidence and scope](receipt-ack-special.md#bound-the-independent-id-map-producers-before-changing-receipts-2026-11-06).

- **2026-11-06 confirmed Rust local ownership correction:** receipt queued-writer guard previously omitted reservation account authority. It now binds store/account generation, routing/registration and Signal identity halves before encoding, retaining uncertain rows on change. No native-format/account-reset claim. [Scope and actual writer regression](replay-safety-audit.md#receipt-writer-retains-its-reservation-account-authority-2026-11-06).

- **2026-11-06 confirmed Rust local ownership correction:** persist receipt reservation account authority atomically and compare it in ACK settlement. Changed/missing owner or unbound legacy uncertainty cannot settle; no guessed migration/replay. Separate from native callback-map ownership and proof of socket write. [Evidence and 44-case reopen regression](replay-safety-audit.md#persist-receipt-reservation-authority-through-ack-settlement-2026-11-06).

- **2026-11-06 confirmed Rust local callback-ownership correction:** receipt reservations retain durable exact member snapshots; ACK settlement compares all original rows, restricts transitions to those keys and consumes owner atomically. Preserves original late ACK after partial write/reconnect without resend. [Evidence, rejected connection-only restriction and regression](replay-safety-audit.md#durable-original-receipt-callback-members-2026-11-06).

- **2026-11-06 confirmed Rust local write-authority correction:** queued receipt guards now bind exact durable callback bytes as well as account/members/privacy. Missing/consumed/corrupt owner refuses before encoding, retaining uncertainty. [Scope and 105-case regression](replay-safety-audit.md#receipt-writer-also-owns-the-durable-callback-record-2026-11-06).

- **2026-11-06 confirmed Rust local metadata-validation correction:** durable receipt callback members must be unique bounded receipt keys with matching uncertain state, account/owner and full supported ACK correlation; exact bytes alone no longer authorize unrelated/rebound updates. [Gap and 24-case regression](replay-safety-audit.md#validate-durable-callback-member-semantics-before-settlement-2026-11-06).

- **2026-11-06 confirmed pinned selected static / Rust correction:** ordinary call ACK requires nonempty id/type and EW4 permits only A0R sender types; known group/newsletter types reject through LIL. Shared captured receive now follows these confirmed cases after459 diversion, retaining pending uncertainty. [Sources, tests and missing-sender/payload limits](message-ack.md#ordinary-call-ack-text-fields-and-known-sender-rejection-2026-11-06).

- **2026-11-06 confirmed pinned selected static / Rust ordering correction:** known-null call sender throws IllegalStateException before class fields; selected reader Exception handler invokes1rg.A00 and continues, unlike LIL. Absent/empty from now bypasses later fatal call-field checks while global id/numeric precedence remains. [Methods, catch offsets and regression](message-ack.md#known-null-call-sender-precedes-fatal-class-field-checks-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** definite-null call sender text now follows the recoverable route before call fields; native bare names and cache aliases are not guessed null. Retained full bare/cache methods, bounded field/caller census and continuation controls. [Scope and unresolved constructor/cache behavior](message-ack.md#definite-null-call-sender-text-versus-native-bare-cache-values-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:**0Dx requires nonzero-leading1..15 ASCII digits; lid/hosted.lid share its lid-valued constructor. Plain ACK text now applies null/fallback/canonical alias semantics across call admission and selected message chat, without global JID rewrite or PN/LID merge. [Exact re-export and regressions](message-ack.md#plain-lid-text-uses-constructor-grammar-and-canonical-server-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** explicit LID ACK text retains null/user/device class, Java-i32/byte-narrowed construction, ignored dot suffix and hosted original99 rule. Call recovery, selected chat and ordinary participant success share this adapter; opaque binary JIDs/global model remain separate. [Sources and regressions](message-ack.md#explicit-lid-device-ack-text-preserves-native-class-and-narrowing-2026-11-06).

- **2026-11-06 confirmed pinned selected static / Rust correction:** ordinary PN/hosted ACK text now follows PhoneUserJid ->0Dl.A02 (5..20 ASCII, nonzero,10 only1008), device class/narrowing and hosted99. Shared call/chat/participant paths retain special0/Server/gdpr as unclassified. [Exports, regression and limits](message-ack.md#pn-ack-text-follows-its-own-constructor-grammar-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** completeDCF/Dv7/J9O classes resolve plain0/Server/gdpr ACK class differences. Shared call rejection, recipient cast/fallback and participant conversion now preserve their native types. Explicit special devices remain open. [Evidence and regressions](message-ack.md#plain-special-pn-ack-identities-preserve-native-class-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:**0EE/Dv4 accept special UserJid bases0/Server; explicit device class changes call and participant admission independently of plain base types. Shared ACK text now applies native byte-range/hosted99, recipient cast and base unwrap semantics. [Sources and regressions](message-ack.md#explicit-special-pn-devices-preserve-device-class-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** group constructor and newsletter factory now govern shared ACK text admission/selection. Newsletter error wording does not establish numeric grammar; actual factory only checks Java-trimmed nonempty/nonzero-leading text and keeps original user. [Sources, corrected interpretation and limits](message-ack.md#group-and-newsletter-ack-text-uses-native-construction-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** broadcast list constructor and exactstatus/location subtypes now govern shared ACK text null/class admission and selected chat. All valid broadcast types are excluded call senders; malformed text retains capture/continuation. [Exports, negative base-method query and regression](message-ack.md#broadcast-ack-text-preserves-constructor-and-special-subtypes-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** callDv5 requires18..32 ASCII hex and has no0DY message-chat identity; temp7Kr requires a hyphen and is excluded call sender type2. Shared null ordering and selected-chat casts now follow those distinctions. [Complete sources and regressions](message-ack.md#call-and-temporary-ack-jid-text-keep-distinct-native-types-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** barePN/group/call classes and bounded cache insertion proof establish types13/16/20. Call sender rejection and null message-chat/recipient casts now follow those distinctions; status_me/lid_me aliases remain separate. [Evidence and actor controls](message-ack.md#bare-server-ack-identities-follow-bounded-native-subtypes-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** selfUserJid0EG/2LK primary-device overrides throw before ordinary message success/refusal dispatch for participant/originalfrom. Shared outcome/outbox guards retain uncertainty; self recipient cannot fall back and call type11 rejects. [Complete classes, cache boundary and reopen matrix](message-ack.md#self-user-ack-conversion-failure-is-not-an-ordinary-participant-result-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** exactis_group_status=true diverts before ordinary ACK outcomes; downstream admission ignores selectedPN/LID. Shared outbox/expected-chat paths now retain uncertainty on success/refusal. Status/group/newsletter async and scheduled state4 callbacks remain open. [Predicates, branch offsets and matrix](message-ack.md#direct-group-status-message-ack-diversion-precedes-terminal-outcomes-2026-11-06).

- **2026-11-06 confirmed selected pinned static / Rust correction:** asyncFcQ status ACK parses edit as Javai32 before success/refusal; malformed text now retains outcome uncertainty without ending receive. Multipart state handlers and newsletter404/edit7/8 success exception retained as separate unported authority. [Complete exports, matrix and limits](message-ack.md#async-status-ack-edit-parsing-precedes-persistence-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research and negative results:** status privateA01/hash repair continuation, exact state enum, backend-specificDyI updates and SQL state reread are retained. Timestamp behavior agrees with existing parser; no hardphash rejection or native once-only claim. Full repair/writer/callback authority remains open. [Sources, table and checks](message-ack.md#status-persistence-has-state-and-key-authority-beyond-ack-parsing-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:** status hash repair enqueuesSyncDeviceAndResendStatusJob with operation/key/recipient/expiry checks, device sync, saved-row reload and row/part-owned canonical ACK wait. Report submission is separately0NX. Native retry authority remains unported and does not loosen Rust reservations. [Complete exports and exact remaining gaps](message-ack.md#status-hash-repair-queues-a-guarded-sync-and-resend-job-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:** canonical wait reason/key/owner identity and committed terminal part notification now traced. DCC is process-local full-key dedup; status cleanup remains bounded unknown. No ID-only ACK wake or Rust replay expansion. [Sources, transition table and remaining producers](message-ack.md#canonical-status-ack-waits-require-committed-part-and-callback-ownership-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:** builder canonical-part predicate requires SENT+positive ACK timestamp; parentrow/UUID guarded possible-transmission markers and durable enqueue failures bind further work. Reason-field/exception-call censuses retained; actual renderer/ciphertext authority remains open. [Exports and Rust guard comparison](message-ack.md#status-builder-distinguishes-canonical-part-ack-from-possible-transmission-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:** status job persists part hash and zero-valued device wire marker before0Sf submission; renderer fields/child metadata and feature-controlled ACK wait are retained. No invented mandatory fields/native size cap or timeout. [Complete exports and limits](groups.md#current-status-job-wire-rendering-and-pre-submit-markers-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:**0Sf registers before queueing; status wrapper completes future withnull, not acceptance data. Key-device0 normalization, duplicate replacement and timeout key removal retained without native uniqueness/replay claims. [Complete sources and remaining concrete writer](message-ack.md#status-submit-future-completes-callbacks-without-carrying-acceptance-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:** status type8 final handler binds prepared message to codec/counter/framed output; post-write notification is local, partial output remains uncertain. Existing Rust channel-consumption controls verified. [Complete evidence and remaining lifecycle gaps](message-ack.md#status-final-writer-consumes-the-transport-counter-before-output-2026-11-06).

- **2026-11-06 confirmed pinned static / implemented:** native serializer omits empty child-array content; Rust now matches with literal failing-before/passing-after fixtures. Conditional serialization budget and size-failure ordering retained; optional colon packing remains open. [Sources and limits](node-serializer.md).

- **2026-11-06 confirmed pinned static / implemented:** packed nibble12 is colon; actual13..15 rejected, oddpadding ignored before validation, negative digit count rejected. Rust encoder/decoder corrected with failing-before literal fixtures. [Reader/writer evidence and remaining context gaps](node-serializer.md#packed-nibble-colon-and-padding-match-the-apk-2026-11-06).

- **2026-11-06 confirmed pinned static / implemented:** names/JID servers disable packing after token lookup; node text content is raw bytes. Rust contexts corrected with failing-before literal fixtures; auto text-to-JID and special tuple parity remain open. [Sources and limits](node-serializer.md#names-and-body-text-use-different-string-rules-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:** message3 snapshots/clearsA04 before local disconnected failure; row-reloading worker and separate directA05 cleanup retained. Offline writer drop remains IQ-ID-only, not status settlement. [Complete sources and durable Rust comparison](message-ack.md#disconnect-clears-native-callbacks-without-proving-non-delivery-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:** disconnectF1z buildsEuU and reaches message-send telemetry; result4/reason are logging fields, not proven durable delivery-state writes. Preferences/helper effects and actual retry producers remain open. [Complete exports and exact field binding](message-ack.md#disconnect-row-bookkeeping-feeds-logging-rather-than-a-proven-state-write-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:** ChatConnectionRequirement testsA04==2; selector requirements/handoff and slow-producer diagnostic separated from network deadlines. Zero exact constructor calls refuted as absence proof by positive optimized allocation. [Complete evidence and remaining lifecycle gaps](message-ack.md#job-connection-requirement-is-local-admission-not-replay-authority-2026-11-06).

- **2026-11-06 confirmed selected pinned static / research:** concreteRNL observer and managerQnY33 wake reopen selector condition; executor refusal remains possible. Persistent admission/injection/list ordering and distinct requeue/group/restore wakes retained without wire/replay guarantees. [Complete evidence and remaining provider/restore gaps](message-ack.md#connection-observers-wake-the-selector-without-granting-send-authority-2026-11-06).

- **2026-11-06 confirmed pinned static / local identity correction:** native244 carries integrator in address; existing reference245 decoder erased nonzero integrator and re-encodedzero. Nonrepresentable legacy inputs now explicitly refuse, with failing-before literal fixtures. Native244 support remains open. [Exact schemas and limits](node-serializer.md#legacy-interop-tuples-cannot-erase-a-nonzero-integrator-2026-11-06).

- **2026-11-06 confirmed pinned static / implemented:** token0 attribute value becomesnull in native reader and fails0Tl checked constructor; Rust no longer synthesizes empty text. Literal failing-before fixture preserves explicitempty-id/body controls. [Exact evidence and routing limits](node-serializer.md#null-attribute-values-cannot-become-empty-text-2026-11-06).

- **2026-11-06 confirmed pinned static / unresolved runtime gap:** rawroot descriptiontoken2 returnsnull and native reader logs out/returns through concrete1lY/0ZJ chain; Rust lacks coordinated actor/low-level termination. Existing terminal control is not coverage of this gap. [Sources, required regressions and reconnect limits](node-serializer.md#root-stream-end-token-requires-coordinated-receive-termination-2026-11-06).

- **2026-11-06 confirmed pinned static / implemented offline:** root token2 classifiedStreamEnd; actor/split epoch and low-level channels consume on end, capture-owning paths retain exact frame and stop. Failing-before fixtures, compressed/captured waits, real receipt uncertainty/reopen and encrypted-record tests retained. [Scope and native reconnect limits](node-serializer.md#root-stream-end-captures-and-consumes-the-receive-epoch-2026-11-06).

- **2026-11-06 confirmed pinned static / implemented offline:** 123.A08 rejects header-only records andmask0x01 fragmentation; compression mask0x02 corroborated. Rust now refuses/captures/consumes consistently across actor and low-level/CLI paths usingterminal_frame_error/PeerProtocol. Failing-before ping fixture and encrypted receive tests retained. [Scope and empty-record gap](frame-admission.md).

- **2026-11-06 confirmed pinned static / implemented offline:** native123.A08 loops after authenticated length0; Rust normal reader/read_node/read_frame now skip before stanza diagnostics while preserving cipher counter consumption and caller-owned deadlines. Failing-before encrypted IQ fixture and bounded-read cancellation controls retained. [Scope and remaining gaps](frame-admission.md#authenticated-empty-records-are-skipped-2026-11-06).

- **2026-11-06 confirmed selected static / implemented offline:**123.A08 inflation I/O escapes to1s2.run reader-map cleanup and terminal onError; concrete1lY postswhat2 and selected manager proxy retained. Rust compression diagnostics now capture/consume consistently. Failing-before encrypted/actor tests and real uncertain receipt reopen/no-replay controls retained. [Sources, local output-cap distinction and lifecycle limits](frame-admission.md#compression-failure-consumes-the-receive-epoch-2026-11-06).

- **2026-11-06 confirmed selected static / implemented offline:**123.A06 full-buffer read precedes A07 nibble validation; missing buffer bytes throwIOException, missing list/string markers selectLIL, and initial parse failures exit1s2.run. Rust packed error priority, unexpected_eof and complete invalid_packed_digit receive termination corrected; native negative-count LIL gets distinct publicInvalidPackedLength/classinvalid_packed_length without making genericInvalidNode terminal. Literal/encrypted failing-before controls, actor durable uncertainty/reopen/no-replay and captured-wait retention tested. [Sources and local-policy/native-parity limits](frame-admission.md#truncated-nodes-consume-the-receive-epoch-2026-11-06).

- **2026-11-06 confirmed selected static / implemented offline:**123.A00/A01 list/count/null-tag LIL and0Tl/00l.A0F null-value NPE leave initial reader via distinct outer handlers. Four checks now emit publicInvalidWireStructure and capture/consume consistently; generic unknown/local refusals and nested null-child boundary stay distinct. Failing-before classification/encrypted fixtures, all-family actor uncertainty/reopen/no-replay and compressed captures retained. [Sources, tests and remaining parity limits](frame-admission.md#selected-wire-structure-failures-are-terminal-2026-11-06).

- **2026-11-06 confirmed selected static / implemented offline:** media IQ124 registers32000ms through0Sf.A0R; explicit CLI operation now uses the family prewrite deadline instead of generic30s. Configured I/O bounds remain independent. Failing-before31s encrypted reply and expiry/reopen/no-resend controls retained. Selected sender-check absence is bounded, not a global admission proof. [Source and limits](media-network-outcomes.md#media-refresh-reply-window-2026-11-06).

- **2026-11-06 confirmed local defect / implemented offline:** experimental direct CLI key query had no durable prewrite reservation; normal outbox safety did not cover it. Existing device_key_batches now freezes recipient/query ID before the exchange, blocks uncertain/refused/foreign outcomes, and revalidates retained replies. Two failing-before and three adapter fault/legacy/correlation/reopen controls retained. No new native duration inferred. [Scope, inspection and remaining contracts](replay-safety-audit.md#experimental-direct-cli-key-query-reservation-2026-11-06).

- **2026-11-06 confirmed selected static / research only:** concreteDxh inherits bare-return1ru.A00, so reader-map cleanup is separate from bulkFfK. Two physical0Th.A03 callers and provider0x90 lazy/direct global-cache binding retained with reset/account-switch limits. No full-query32s timer, safe replay or usable-key success inferred. REA missing-Java provider error retained; supported DEX fallback completed trace. [Sources and unchanged Rust boundary](retry-answers.md#full-key-cleanup-and-future-failure-are-separate-2026-11-06).

- **2026-11-06 observed host repair / confirmed selected static:** fixed only persisted REA JAVA_HOME to installed Temurin21 after private backup; fresh CLI now produces complete pinned-APK Evidence, while current MCP remains stale. Provider-cache census/accessors show selected optional initialization, not proven reset/account switch. Complete records/DEX and remaining concurrency/lifecycle boundaries retained. [Repair and evidence](apk-analysis.md#rea-android-java-path-repair-and-cli-proof-2026-11-06).

- Confirmed selected APK 2.26.43.72 post-publication follow-up: `0xc4338`
  looks up `mc_1st_set_latest_cache`, checks a version predicate and dispatches
  mode/strings through receiver virtual+0x50; reader/status are not arguments.
  Concrete receiver effects and activation remain unknown. See
  [retained trace](apk/2.26.43.72-mobileconfig-handle-publication/followup.md).

- Confirmed selected APK 2.26.43.72 StorageManager slot+0x58 (`0x83dc8`)
  consumes the shared snapshot but ignores its status byte, checking availability
  and a nonnull-root probe. Full acceptance remains unknown. See
  [consumer evidence](apk/2.26.43.72-mobileconfig-handle-publication/consumer.md).

- Confirmed selected APK 2.26.43.72 `reloadLatestCacheOnDisk` branches:
  hash match exits; schema bit 0/8 selection controls translation versus
  explicit republication. Translation failure logs then exits, with no local
  publication fallback. See [reload trace](apk/2.26.43.72-mobileconfig-handle-publication/reload.md).

- Confirmed selected APK 2.26.43.72 disk selector ranks `.mctable` filenames
  by parsed numeric prefix, requires prefix-derived `.meta` existence and keeps
  the first equal index encountered. This does not establish body validity or
  timestamp order. See [selection evidence](apk/2.26.43.72-mobileconfig-handle-publication/selection.md).

- Confirmed selected APK 2.26.43.72 translation helper has its own shared
  publication suffix after mode-specific translator readiness; stores root-probe
  status 4/5 while returning retained readiness. Nested mapping construction and
  durable acceptance remain unknown. See [translation evidence](apk/2.26.43.72-mobileconfig-handle-publication/translation.md).

- Confirmed selected APK 2.26.43.72 vector translation uses `0xffff` slot
  sentinels and can install partially populated vectors after bounds logging.
  Lookup preserves upper 48 bits and returns low-16 sentinel for out-of-range
  inputs; downstream sentinel handling remains unknown. See [mapping trace](apk/2.26.43.72-mobileconfig-handle-publication/mapping.md).
