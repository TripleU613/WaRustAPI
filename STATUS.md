2026-11-06 — **Bound upload result adoption and new cache rows to their owner.**

**Strict store-failure scope located.** The traced flag setter wraps migration
bootstrap decryption and restores prior thread context. Persistence exceptions
remain distinct from content failures; ordinary retry linkage is unproven.
[Evidence](docs/protocol/retry-answers.md#strict-store-failure-flag-is-migration-scoped-2026-11-07). Research only; no runtime change.

**Session/identity transaction scopes traced.** Native adapters discard save
booleans; local SQLite scopes and conditional IOException propagation are
retained. Shared outer ownership/crash semantics remain open. [Evidence](docs/protocol/retry-answers.md#concrete-session-and-identity-transaction-scopes-2026-11-07). Research only.

**Earlier identity AEA write located.** Session processing writes identity-table
AEA before returning, followed by a separate caller update. Update count is
discarded; exception status does not prove rollback. Concrete transaction/crash
semantics remain open. [Evidence](docs/protocol/retry-answers.md#session-processing-contains-an-earlier-aea-write-2026-11-07). Research only.

**Receipt-key AEA input and ordering traced.** Native signature-domain bytes,
state enums, result distinctions and post-session state-update call are retained.
Full verification/persistence port remains open; Rust key-adoption refusal stays
in place. [Evidence](docs/protocol/retry-answers.md#receipt-key-aea-admission-and-state-ordering-2026-11-07).

**Prepared retry wire integrity closed.** New saved-account retries freeze a
wire digest at preparation; changed bytes cannot become the first reservation
original. Registration-byte substitution is tested across reopen without row
mutation on refusal. Legacy missing-digest limits remain. [Evidence](docs/protocol/outgoing-retry-receipts.md#prepared-retry-wire-integrity-before-reservation-2026-11-07).

**Retry preparation ownership gap closed.** New retries with saved account
state bind registration/identity authority at preparation and recheck it before
reservation. Stale snapshots and intervening changes refuse without rewriting
pending wire, including after reopen. [Evidence](docs/protocol/outgoing-retry-receipts.md#retry-preparation-account-binding-2026-11-07).

**Stateless retry configuration inputs narrowed.** Selected compiled0/false
entries and zero initial SDK specifiers are reproducible; effective preferences/
readiness and full precedence remain unproven. No raw-AB shortcut or runtime
policy change added. [Evidence](docs/protocol/outgoing-retry-receipts.md#stateless-retry-configuration-authority-2026-11-07).

**Minimal retry count reset closed.** Incoming nonzero/invalid counts can no
longer create a constant count1 request. Aggregate0 retains the existing minimal
request; advanced progression/carried keys remain explicitly unported. Durable
state, reopen and once-only behavior are tested. [Evidence](docs/protocol/outgoing-retry-receipts.md#incoming-count-cannot-reset-the-minimal-retry-2026-11-07).

**Retry key publication ownership traced.** Native carried prekeys are marked
directly distributed before retry submission; selected key/skey/pq wire mapping
is now bound. Rust integration still needs durable ownership and upload exclusion;
no key emission or mutation added. [Evidence](docs/protocol/outgoing-retry-receipts.md#retry-prekeys-direct-distribution-ownership-2026-11-07).

**Stateless/PQ retry inputs traced.** Incoming enc flags feed native named job
fields and conditional key preparation; Rust policy integration remains unproven.
Native also has a minimal branch, so no speculative gate or key emission was
added. [Evidence](docs/protocol/outgoing-retry-receipts.md#statelesspq-retry-preparation-inputs-2026-11-07).

**Overwritten missing-body bypass closed.** All immediate enc children now receive
the native null-body admission check before routing/selection. Exact live-location
exception and null-versus-empty distinction are retained. Real ciphertext verifies
refusal without durable state advancement. [Evidence](docs/protocol/media-network-outcomes.md#ciphertext-body-presence-before-descriptor-overwrite-2026-11-07).

**Group ciphertext selection corrected.** Receive now uses the last descriptor
per native parser slot, with matching pairwise type/session routing. A corrupted
last packet cannot fall back to an earlier valid packet and earn ACK authority;
reversing the order decrypts once. [Evidence](docs/protocol/media-network-outcomes.md#last-group-ciphertext-descriptor-and-consistent-routing-2026-11-07).

**Mixed encrypted-child type bypass closed.** A missing/unknown or unported enc
can no longer be ignored while a valid sibling decrypts and earns ACK authority.
Both child orders are tested with valid ciphertext and unchanged state on refusal.
Native-valid unported types remain explicit coverage gaps. [Evidence](docs/protocol/media-network-outcomes.md#mixed-encrypted-child-type-admission-2026-11-07).

**Encrypted child admission corrected.** Exact outer version2 and native ordered
retry-count checks now precede inbox decryption/ACK authority. Valid-ciphertext
regressions cover refusal without sender-key/journal advancement, then successful
corrected decryption. [Evidence](docs/protocol/media-network-outcomes.md#encrypted-child-metadata-before-decryption-2026-11-07).

**Retry state restoration and receipt pass traced.** Native retry state can be
restored from saved metadata; selected first-pass receipt permission depends on
secondary-descriptor absence. Later-pass/durability/classification gaps remain;
no runtime response change. [Evidence](docs/protocol/media-network-outcomes.md#retry-metadata-and-receipt-pass-selection-2026-11-06).

**Media error response modes narrowed.** Native sendReceipt differs by callback;
isSendRetry can be set before retry-job enqueue, and typed stop paths clear
action permission. Evidence retained; Rust error491 remains unimplemented until
classification and response ownership are proven. [Trace](docs/protocol/media-network-outcomes.md#media-error-response-mode-provenance-2026-11-06).

**Media parser error-ACK boundary traced.** Native Dxp reasons flow conditionally
through a distinct mapper/envelope path: common media reasons map to error491
with optional original failure_reason metadata. This is not receipt-parser487 or
a Signal retry cause. Rust integration remains unproven; no wire change.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-media-parser-error-ack-boundary-2026-11-06).

**View-once cached MIME bypass closed.** Shared whole-image MIME admission now
precedes view-once descriptor selection, cached prefetch reporting and one-shot
plaintext consumption. Direct flags and all supported wrappers are regression
tested with valid ciphertext and unchanged state on refusal/reopen.
[Evidence](docs/protocol/media-network-outcomes.md#view-once-image-mime-before-cache-and-consumption-2026-11-06).

**Image/video whole-MIME preflight corrected.** A shared supported-format check
now gates descriptors and URLs, preventing parameterized/padded visual metadata
from authorizing cached plaintext or a new GET. Image ASCII case works; MP4 remains
exact. Native WebP context and3GPP support remain separately bounded.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-visual-whole-mime-preflight-2026-11-06).

**Sticker MIME preflight corrected.** Pinned Di4/FPx core fields and whole-string
WebP comparison confirm parameters/padding must not authorize the supported path.
ASCII case variants now work; Lottie remains excluded before cache/network use.
The new audio oneof remains explicitly outside confirmed complete handling.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-sticker-ingress-and-whole-mime-preflight-2026-11-06).

**Current document ingress traced.** Core tags/hash guards match Rust; native
parser permits zero length and stores MIME without the audio-style allowlist.
Rust's nonempty/size/MIME export bounds remain labelled local; zero-byte transfer
parity remains unverified. No runtime change.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-current-document-ingress-2026-11-06).

**Opus voice-note preflight corrected.** Pinned audio schema/converters and exact
MIME selector show contradictory codec metadata must not authorize the supported
Opus path. Only canonical Opus PTT is selectable/cache-reusable; regression covers
valid ciphertext, reopen and no reservation.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-audio-ingress-and-opus-mime-preflight-2026-11-06).

**Current video ingress checked.** Core hash/key/length/viewOnce tags match Rust;
selected native converters separate ordinary/view-once ownership before common
projection. Sidecar helper checks key presence, not sidecar presence; descriptor
indexes are independently checked offline. Runtime behavior unchanged.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-current-video-ingress-2026-11-06).

**Current image protobuf hash ingress confirmed.** Pinned DiL schema/EwA parser
match Rust's core image tags and trace hashes through message setters into the
download model. Sidecar validation is separate; absence is not a universal image
refusal. Runtime behavior unchanged.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-current-image-protobuf-ingress-2026-11-06).

**Message-to-download model traced.** Confirmed stored encrypted/plaintext hash
and length projection, register-correct constructor binding and lazy sidecar
preparation. Transfer category depends on message/provider predicates, not MIME
alone. No runtime or request-policy change.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-message-to-download-model-2026-11-06).

**Download factories traced.** Confirmed PUSH/PULL enum-to-handler mapping,
config-dependent mode selection, category eligibility and sidecar Callable
behavior. The selected chat branch fails on missing required sidecar rather than
automatically falling back. No runtime policy change.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-download-factory-selection-2026-11-06).

**Download range/result consumers traced.** Pinned MuY/MuZ wrappers lead to
stream/window handlers; MuY packages a chunk-MAC decrypter separately from HTTP
status. Normal finalization checks transfer/plaintext hashes and cancellation.
No universal206/416 success rule or Rust status change is established.
[Evidence](docs/protocol/media-network-outcomes.md#pinned-download-range-and-result-consumers-2026-11-06).

**Pinned download HTTP leaves audited.** Confirmed identity and conditional Range
construction, distinct native timeouts,200/206 leaf acceptance and416 special
handling. Rust remains200-only pending operation/caller binding; no guessed
header or live traffic. [Evidence](docs/protocol/media-network-outcomes.md#pinned-shared-download-http-leaves-2026-11-06).

**Ordinary media source binding.** Reservation and successful ciphertext adoption
now atomically compare the selected inbox bytes; plaintext handoff rechecks the
eligible source. Refusal observations remain attempt-owned. Offline regression
covers replacement/removal/metadata changes and reopen. Native lifecycle remains
unverified. [Details](docs/protocol/media-network-outcomes.md#ordinary-download-source-binding-2026-11-06).
New encrypted phone/generation metadata gates final uploaded promotion and cache
reuse. Revoked/replaced accounts cannot adopt late payloads, while HTTP evidence
remains durable. Tests cover owner/metadata/stale-row faults, every-table/journal
preservation, reopen and public cross-phone cache refusal. Legacy stays unbound.
[Format, tested guarantees and remaining lifecycle scope](docs/protocol/media-network-outcomes.md#upload-response-adoption-and-prospective-cache-owner-2026-11-06). No live operation.

2026-11-06 — **Fenced media upload reservation with current route/account state.**
After preparation, expiry/host/auth/registration/phone/generation are rechecked
transactionally. Seed and pending upload commit together; failed checks/builds
leave neither new state nor a POST. Tests cover every-table/journal preservation,
reopen, duplicate refusal and write-local coordinator/corruption visibility.
[Evidence, local policy and remaining in-flight boundaries](docs/protocol/media-network-outcomes.md#atomic-upload-routeaccount-fence-at-reservation-2026-11-06). No live operation.

2026-11-06 — **Separated media refresh deadlines from hard auth validity.**
Exact field/method tracing closes120s refresh trigger,60s timer and request-aware
20s wait inputs, with distinct elapsed/uptime clocks and old-auth comparison.
Selected upload paths have additional feature/offline/pre-transfer gates; neither
120s nor route TTL is proven a universal hard upload cutoff. Research-only chunk.
[Complete evidence and unported runtime boundaries](docs/protocol/apk/2.26.43.72-media-deadline-consumers/README.md). No live account operation.

2026-11-06 — **Stopped saturated media lifetimes from extending auth eligibility.**
Upload route selection now wraps each seconds-to-ms product before applying its
existing age/minimum guard, matching the pinned constructor operation. Tests cover
negative/zero/short/usable positive wrapped durations for both fields, with all
store tables/journal unchanged across reopen. No automatic refresh or HTTP call.
[Evidence and remaining clock/deadline scope](docs/protocol/media-network-outcomes.md#wrapped-lifetime-conversion-before-local-route-eligibility-2026-11-06).

2026-11-06 — **Unified retry number admission and required-field ordering.**
Java-reference BMP counts/times now pass the shared parser and cannot hide a later
missing required field. Independent host Java178-case fixture checks int/long
admission and error order; the saved-session resend proves Unicode/ASCII count1
share one reservation. Non-ASCII failures retain conservative error-ACK limits.
[APK calls, Android contract, host-reference scope and reproduction](docs/protocol/retry-parser-admission.md#shared-java-reference-numeric-admission-and-later-error-ordering-2026-11-06). No live account operation.

2026-11-06 — **Shared native-null factory rules with receipt contexts.**
Unknown bare/domain and empty optional participant text now follows the retained
nullable factory rather than blocking retry487/completion ACKs. Typed values and
recognized bare objects remain distinct. Count2 saved-session query/refetch tests
exercise the corrected omission with existing ACK/state/identity safeguards.
[Evidence, cache boundaries and corrected expectations](docs/protocol/receipt-optional-addresses.md#shared-definite-null-text-factorycache-boundary-2026-11-06). No live account operation.

2026-11-06 — **Preflighted retry ACK construction/encoding before direct work.**
Eligible direct retries cannot reserve a resend/query or advance a session when
their confirmation is unsupported or unencodable. Both answer entrypoints share
the guard. Known-null optional participants are omitted; valid refetch still
works. Tests preserve every retry-state row on rejected confirmation contexts.
[Evidence and ordering boundaries](docs/protocol/retry-ack-dispatch.md#ack-capability-preflight-before-direct-retry-work-2026-11-06). No live account operation.

2026-11-06 — **Selected the correct requesting device for retry receipts.**
Direct retries now use the root sender; group/broadcast use the participant,
matching the native original-chat branch. Known-null text addresses cannot gain
retry authority through generic parsing. Tests cover misleading participants,
real saved-session resend/duplicates and unchanged state for invalid sources.
[Exact branch evidence and remaining limits](docs/protocol/apk/2.26.43.72-retry-device-source/README.md). No live account operation.

2026-11-06 — **Blocked group facts from constructor-null incoming addresses.**
Group source/actor selection now preserves native text null before matching saved
rows. A root-source guard also precedes aggregate expansion. Tests seed matching
historical targets/chats and verify no facts or changes to any table/journal,
including reopen/redelivery; stored rows and typed carriers remain unchanged.
[Evidence and remaining boundaries](docs/protocol/group-receipts.md#constructor-null-incoming-addresses-cannot-match-saved-rows-2026-11-06). No live account operation.

2026-11-06 — **Kept failed non-user receipt address construction null.**
Receipt success/copied/fresh ACKs now omit invalid group/broadcast/newsletter/
temp/call participants instead of resurrecting them through the generic parser.
Direct facts use the same null result; typed objects and valid writer refusals
remain distinct. Tests cover codec/facts and receive capture/redelivery/no-replay.
[Evidence and limits](docs/protocol/receipt-optional-addresses.md#non-user-text-constructor-failures-in-receipts-2026-11-06). No live account operation.

2026-11-06 — **Added copied-context487 for supported unhandled receipt roots.**
Response selection follows the upstream router and preserves original recipient/
edit instead of using fresh parse-error context. Special types and early message_id
routes remain distinct. Receive-loop tests cover capture ordering, explicit
redelivery and write-cut/reopen without replay; broader receipt parity remains open.
[Evidence, field semantics and local limits](docs/protocol/apk/2.26.43.72-unhandled-receipt-root/README.md). No live account operation.

2026-11-06 — **Restored proven peer-retry schema/version error responses.**
The peer handler calls the same retry parser before its separate responder.
Shared error classification now permits peer fresh/copied487 responses while
ordinary success ACK, resend and key-refetch remain excluded. Leading aggregate
precedence remains. Full peer success handling is still unported.
[Complete handler, census and corrected assumption](docs/protocol/apk/2.26.43.72-peer-retry-handler/README.md). No live account operation.

2026-11-06 — **Stopped key-aggregate validation from blocking valid retry routes.**
A leading participants child without message_id selects key aggregation only for
the native ordinary status set. Retry/special/unknown roots now retain their own
consumer policy. Tests preserve ordinary missing-key errors and exercise a real
saved-session retry through validation, recording and durable response handling.
[Evidence and remaining boundaries](docs/protocol/apk/2.26.43.72-retry-upstream-routing/README.md). No live account operation.

2026-11-06 — **Stopped ordinary retry processing on two alternate native routes.**
Leading participants/message_id takes precedence over type=retry; category=peer
selects a separate handler. Shared routing now prevents a false ordinary487,
success ACK, resend or key query on those inputs. Saved-session tests preserve
outbox, checkpoint, ratchet and identity state. Full router parity remains open.
[Evidence, registration and negative findings](docs/protocol/apk/2.26.43.72-retry-upstream-routing/README.md). No live account operation.

2026-11-06 — **Located the upstream receipt router and unknown-type context split.**
The inherited generic route dispatches to1rU.A05, whose ordered child/type branches
precede ordinary F10 parsing. An admitted unhandled root uses copied-context487,
not fresh parse-error487; a leading participants/message_id can select ordinary
processing first. No runtime behavior changed; the retained routing matrix now
guides the next response correction.
[Complete adapter/sets, dispatch evidence, negative results and remaining boundaries](docs/protocol/apk/2.26.43.72-receipt-router-boundary/README.md). No live account operation.

2026-11-06 — **Added missing aggregate-key error487 with local-ID boundaries.**
Only proved required key absence in the supported key branch gains a native error
cause and fresh ROOT-ID response. Empty/overlong inner IDs remain local refusals,
with tests preventing NACK authority or root-fact fallback. The two aggregate
branches have distinct chat/ID read order, retained with exact evidence.
[Required-ID/constructor evidence, regressions and remaining scope](docs/protocol/apk/2.26.43.72-receipt-aggregate-id-boundaries/README.md). No live account operation.

2026-11-06 — **Corrected offline receipt timestamp/status error precedence.**
Required item time is parsed before unknown status conversion, matching the APK.
An unknown kind no longer masks an earlier timestamp487 cause. Valid-time unknown
kinds remain outside487; native queued500 versus legacy rethrow is retained as a
separate unported policy rather than guessed into the wire behavior.
[Read-order/exception evidence, failing-before regression and boundaries](docs/protocol/apk/2.26.43.72-receipt-item-status-order/README.md). No live account operation.

2026-11-06 — **Verified lenient ordinary root timestamp handling in the current APK.**
The root parser defaults missing/empty/malformed/overflow time to0 instead of
throwing aggregate-style parse errors. Existing runtime policy is retained;
a new codec/store regression prevents blanket487 NACKs and fabricated supplied
zero times while preserving read facts and duplicate/reopen behavior.
[Current parser evidence, negative hypothesis, regression and limits](docs/protocol/apk/2.26.43.72-receipt-root-time-fallback/README.md). No live account operation.

2026-11-06 — **Added evidenced aggregate timestamp error487 responses.**
Missing required item time and invalid supported signed-long text now carry a
native timestamp cause shared by the receive/NACK paths. Key-item absence still
defaults; valid signed/BMP/extreme integers and opaque local values stay distinct.
A delivery-only key-routing hypothesis was refuted by the complete branch join;
existing read/played key semantics remain unchanged.
[Parser/error evidence, regression checks, negative branch result and open scope](docs/protocol/apk/2.26.43.72-receipt-timestamp-errors/README.md). No live account operation.

2026-11-06 — **Implemented ordinary required-device parse-error ACKs.**
Proven native required-actor failures now get a fresh root error487 response after
durable capture in the normal actor and selected CLI receive windows. Local limits,
unverified binary failures and storage errors remain separate. Recipient-bearing
offline aggregates no longer bypass this grammar check; no facts or retry work
are authorized by a NACK. Synthetic cut/reopen/redelivery checks retain no-replay.
[Cause/output contract, retained native evidence, verification and open scope](docs/protocol/retry-ack-dispatch.md#ordinary-required-device-error-responses-implemented-2026-11-06). No live account operation.

2026-11-06 — **Traced ordinary receipt error487 dispatch and corrected an evidence error.**
Both selected native wrapper paths reach the parsing-error handler for LIL.
A fresh export refuted the old claim that A07(true) returns without sending:
it logs then falls through to the same Message76 path. The prior README and
checksum now explicitly record the correction. Rust runtime remains unchanged;
ordinary error response classification is the next implementation gap.
[Complete handler/wrapper/error evidence, byte comparisons and limits](docs/protocol/apk/2.26.43.72-ordinary-receipt-parse-error-dispatch/README.md). No live account operation.

2026-11-06 — **Refused malformed required actors in offline aggregates.**
The message_id branch now distinguishes required DeviceJid failure from nullable
key-aggregate skipping. Known constructor-null, wrong-class and bounded binary
actor failures refuse ordinary processing before any batch facts are written.
[Required accessor/conversion evidence, atomic refusal tests and remaining response-policy gap](docs/protocol/apk/2.26.43.72-offline-required-receipt-actor/README.md). No live account operation.

2026-11-06 — **Preserved valid aggregate receipts around native-null actors.**
Key aggregates now skip known constructor-null participant text before parsing
its timestamp, matching the pinned parser branch. A malformed time on such an
ignored item no longer discards valid neighbors. Offline message_id aggregates
and nonnull conversion failures remain separate.
[Branch evidence, mixed-row persistence regression and limits](docs/protocol/receipt-canonicalizer.md#constructor-null-aggregate-actors-skip-before-timestamp-parsing-2026-11-06). No live account operation.

2026-11-06 — **Aligned direct receipt fact handling for native-null participants.**
Known constructor-null PN/LID/hosted text now follows the existing supported
sender-device fact fallback rather than becoming a false supplied target.
Valid participants retain original-target checks. Current APK parser/event/worker
traces and encrypted-store regressions bind the distinction and remaining limits.
[Evidence, persistence checks and open branches](docs/protocol/apk/2.26.43.72-receipt-null-fact-actor/README.md). No live account operation.

2026-11-06 — **Distinguished null receipt participants from unwritable objects.**
Receipt ACK output now omits known constructor-null text participants rather than
resurrecting them through the broader generic Jid parser. Nonnull unwritable
objects still refuse the ACK. Native context construction accepts nullable fields;
concrete handler admission and this client's supported sender requirement remain distinct.
[Context/constructor proof, regressions and boundaries](docs/protocol/apk/2.26.43.72-ack-nullable-context/README.md). No live account operation.

2026-11-06 — **Guarded original device getter range before ACK output.**
The pinned writer rejects AD device getters outside 0..255 despite constructor
byte narrowing. Selected unwritable ACK fields are now refused; valid negative
PN devices use the native pair branch. Recipient omission remains distinct.
[Complete byte-writer evidence, regressions and limits](docs/protocol/apk/2.26.43.72-jid-device-byte-range/README.md). No live account operation.

2026-11-06 — **Preserved literal zero-device LID ACK wire class.**
Source-aware ACK output now keeps literal DeviceLid0 as AD-JID while decoded
binary zero-device users still reply with JID_PAIR. Receipt and shared notification
builders have offline kind/byte regressions, including signed and Unicode zero.
Incoming captures and existing persisted selectors remain unchanged.
[Evidence, boundaries and remaining gaps](docs/protocol/receipt-primary-ack.md#literal-zero-device-ack-wire-class-2026-11-06). No live account operation.

2026-11-06 — **Guarded selected automatic ACK writer compliance.**
Notification and receipt ACKs no longer copy known noncompliant native address
objects. Omitted recipients and fresh retry contexts remain distinct from copied
fields; valid service-backed devices are not rejected by name.
[Current declaration census, class/writer evidence and codec regressions](docs/protocol/apk/2.26.43.72-jid-write-compliance/README.md).
General grammar/recovery remain open; no live account operation.

2026-11-06 — **Bound bare self-user pairs to native ACK and receipt semantics.**
Two native symbolic pair forms now decode with the general server grammar intact.
Self message ACKs remain uncertain; binary bare call sender restrictions match
text. Unsupported self receipt metadata cannot create ordinary facts or ACKs,
and unwritable self notification routes remain refused before work/write.
[Codec, conversion, writer evidence and state/actor regressions](docs/protocol/message-ack.md#bare-self-user-pairs-and-carrier-independent-guards-2026-11-06).
Qualified aliases/other classes remain open; no live account operation.

2026-11-06 — **Separated binary ACK construction from optional casts and outcomes.**
Known invalid binary participant/recipient fields can no longer settle through
ordinary refusal or from fallback. Hosted binary participants now retain the
ordinary-success exclusion; valid device/non-user recipient casts and special
participant classes remain distinct from nullable malformed text.
[Current evidence, expected-chat/outbox regressions and limits](docs/protocol/message-ack.md#binary-optional-address-construction-and-participant-class-2026-11-06).
No live account operation; other families/decoder paths remain under audit.

2026-11-06 — **Validated binary ACK from before projection and recipient selection.**
Hosted binary sources now project to PN/LID; invalid binary device sources cannot
settle through primary normalization or a valid recipient. Valid source/recipient
precedence and original outcome/journal/no-resend guards remain.
[Current constructor/order evidence and full binary matrix](docs/protocol/message-ack.md#binary-from-construction-precedes-chat-projection-2026-11-06).
Other decoder fields/families remain an explicit audit; no live account operation.

2026-11-06 — **Made native long-PN ACK admission carrier-independent.**
A matching native-valid20-digit PN recipient now correlates in binary primary/
hosted pairs and AD0 as it already did in text. Constructor-invalid long users
remain unsettled; LID's separate bound and no-resend/reopen guards remain.
[Current constructor/reader evidence and regression matrix](docs/protocol/message-ack.md#long-phone-user-recipient-admission-is-independent-of-carrier-2026-11-06).
No live account operation or outgoing target-builder change.

2026-11-06 — **Preserved binary user-recipient precedence.**
Message ACKs no longer ignore a binary zero-device user recipient and settle
through an unrelated from. Hosted binary recipients project to their underlying
PN/LID user; receipt ACK output now uses that native user address as well.
[Current reader/factory/writer evidence and outbox regressions](docs/protocol/receipt-user-recipient.md#hosted-user-output-and-binary-recipient-precedence-2026-11-06).
Success/refusal, mismatch, deduplication and reopen/no-resend controls retained.
No live account operation.

2026-11-06 — **Corrected receipt recipient text-class admission.**
Dot-qualified zero/ignored suffixes and invalid PN/LID users no longer become
primary-user recipient qualifiers through generic parsing. ACKs omit the field;
ordinary matching facts retain original actor/attempt/uncertainty and deduplicate
across reopen. Binary-zero users and strict receipt ACK settlement remain.
[Current class evidence, regressions and limits](docs/protocol/receipt-user-recipient.md#current-text-class-admission-including-dot-qualifiers-2026-11-06).
No live account operation.

2026-11-06 — **Closed raw-route gaps in remaining notification ACK builders.**
Device, prekey-count, dirty-group and account-privacy envelope readers now share
empty-user text refusal. Malformed text cannot gain local server settings
admission or queue dirty-group refreshes. Absent-source count ACKs remain valid.
[Evidence, state/receive regressions and remaining limits](docs/protocol/notification-ack-audit.md#remaining-ack-builders-preserve-raw-route-refusal-2026-11-06).
Updated the ACK inventory to include existing picture/account-privacy paths.
No live account operation.

2026-11-06 — **Closed separate notification address-reader gaps.**
Identity ACK participant routing now shares malformed empty-user text refusal.
Privacy-token sender/consumed alias readers distinguish explicit .0/:0 text
from primary user authority, preserving binary zero-device handling. Receive
continuation, token/state preservation and original-query recovery are covered.
[Evidence, regressions and native-policy limits](docs/protocol/notification-ack-audit.md#separate-identity-and-privacy-token-address-readers-2026-11-06).
No live account operation.

2026-11-06 — **Refused malformed empty-user notification ACK routes.**
Shared ACK handling no longer repairs text such as @s.whatsapp.net into an
outbound server destination; both sender and participant are guarded. Bare
server and typed device routes remain. Native nullable-address behavior and
Rust's stricter local ID/routing limits are explicitly distinguished.
[Current evidence, regression and limits](docs/protocol/notification-ack-audit.md#shared-notification-text-routing-guard-2026-11-06).
No live account operation.

2026-11-06 — **Bound PSA watchdog and wakeup completion semantics.**
Linked previously retained diagnostic-only watchdog proof; it does not expire
held ACK ownership. Traced scheduler units/cancellation and wakeup normal versus
Exception completion. Promotion work is not presumed to be a network fetch.
[Evidence and remaining runtime obligations](docs/protocol/psa-ack-lifecycle.md#wakeup-completion-and-scheduler-follow-up-2026-11-06).
Runtime policy unchanged; no live operation.

2026-11-06 — **Traced current PSA work-before-ACK obligations.**
Native feature/campaign/revoke branches and one-shot deferred completion exceed
the existing legacy PSA helper. Corrected universal ACK claims; effective flags,
worker outcome and durable recovery remain open. Runtime policy unchanged.
[Branch matrix, retained evidence and next obligations](docs/protocol/psa-ack-lifecycle.md).
No live account operation.

2026-11-06 — **Preserved explicit hosted zero-device text qualification.**
Raw hosted/hosted.lid zero qualifiers no longer become plain user metadata;
valid plain/99 and ordinary PN device0 behavior remain. Original text and shared
JID/store/hash behavior are unchanged.
[Native parser evidence and regression](docs/protocol/hosted-chatstate-text.md).
No live account operation.

2026-11-06 — **Projected hosted typing addresses through their native user identity.**
Hosted/hosted.lid aliases and device99 now yield PN/LID display metadata while
preserving original addresses. Unsupported explicit device forms stay omitted;
hash/store/outbound identity handling is unchanged.
[Factory evidence, regression and remaining parser gaps](docs/protocol/presence.md#hosted-incoming-typing-identities-2026-11-06).
No live account operation.

2026-11-06 — **Corrected incoming PN/LID typing address projection.**
Device-qualified from/participant now unwrap to user metadata; known group and
server-only authors cannot appear as the supplied user. Original frames and
saved identities stay unchanged. Other families remain an explicit open audit.
[Current reader, regression and limits](docs/protocol/presence.md#incoming-chatstate-routing-2026-11-06).
No ACK, automatic traffic or live account operation.

2026-11-06 — **Refused unsupported presence/typing targets before construction.**
Empty-user server addresses, agent/device metadata (including explicit AD0/0)
and malformed group contexts no longer pass the supported outbound APIs.
Pinned current writers/type hierarchy and event-driven timer gates retained.
[Evidence and regression](docs/protocol/presence.md#current-target-types-and-outbound-guard-2026-11-06).
No traffic, identity normalization or scheduler feature was added.

2026-11-06 — **Corrected report result extensions; resolved native list bounds.**
Valid report results no longer become refusals due to unrelated error extensions.
Current constructors permit empty lists; optional report/id bounds and exact ID
correlation are traced without inventing source or acceptance requirements.
[Evidence, regression and remaining schema gaps](docs/protocol/chat-actions.md#report-response-and-list-bounds-2026-11-06).
No live account operation.

2026-11-06 — **Bound individual report callback ownership to native32s.**
Pinned RPC/wrapper evidence supplies32000 to the shared IQ registration path.
Expiry now drops the live claim while preserving uncertain Sent state and no
resend after reopen. Located report/id and current flow-schema limits without
inventing response adoption or optional metadata. No live operation.
[Evidence and boundary regression](docs/protocol/chat-actions.md#individual-report-callback-timer-2026-11-06).

2026-11-06 — **Corrected photo lookup result dispatch.**
Unrelated root/nested error extensions no longer turn a matching lookup result
into a refusal. Retained first-named-child and constructor proof; result
payload validation/cache adoption remain unported. No retry or live operation.
[Regression, evidence and limits](docs/protocol/chat-actions.md#photo-lookup-result-extensions-2026-11-06).

2026-11-06 — **Corrected photo outer-error selection and retained callback outcome.**
Upload and lookup now select the first direct coded error, matching the pinned
APK, instead of a competing nested error. Optional durable photo_error keeps
signed code and lookup404/410 empty-metadata classification; status remains the
outer IQ Error. No cache mutation, retry or live operation.
[Evidence, regression and limits](docs/protocol/chat-actions.md#photo-error-callback-outcomes-2026-11-06).

2026-11-06 — **Held MEX privacy setter intent before notification ACK.**
The pinned mutation serializes as w:mex IQ/get, so the classic-only guard missed
it. Exact direct query_id recognition now preserves the hold across queued,
sent, success and refusal outcomes and store reopen. Getter/body/nested-ID
negatives remain admitted. Native typed ownership remains unported.
[Wire proof and regression](docs/protocol/account-privacy-notifications.md#mex-mutation-intent-guard-2026-11-06).
1534 workspace tests passed; no live account operation.

2026-11-06 — **Traced privacy MEX setter input and response admission.**
SetPrivacySettingMutation uses nested enum-valued setting input and a distinct
persisted document. Null-response fallback and enum-translation failure stay
separate; successful maps return to category/pending reconciliation.
[Evidence, SDK limits and Rust boundary](docs/protocol/privacy-mex-config.md#privacy-setting-mutation-input-and-response-rules-2026-11-06).
No production request or live account change.

2026-11-06 — **Re-anchored read-self ACK privacy synchronization.**
Current227 handler can disable reads from none, preserves localfalse on all,
and can queue a corrective setter. Producer/message, interface dispatch and
normal enabled provider binding are retained; runtime/recovery gaps stay explicit.
[Current evidence and Rust boundary](docs/protocol/read-setting-sync.md#current-build-read-self-ack-setting-synchronization-2026-11-06).
No production or live account change.

2026-11-06 — **Separated optimistic privacy views from confirmation.**
Setter follow-up routes an observer; pending integer overlays and existence
checks are distinct from the stored receipt Boolean. Literal/UI references
are not preference-write proof; listener/writer/recovery gaps remain explicit.
[Evidence and limits](docs/protocol/account-privacy-notifications.md#optimistic-privacy-views-are-not-server-confirmation-2026-11-06).
No production or live account change.

2026-11-06 — **Fixed privacy intent release after IQ refusal.**
Native setter failure does not retire its pending category. Rust now preserves
its unowned-setter hold after a real refusal rather than adopting/ACKing a
later notification solely from that error. Saved refusal remains inspectable.
[Evidence, session/reopen regression and remaining ownership](docs/protocol/account-privacy-notifications.md#setter-refusal-does-not-retire-pending-category-ownership-2026-11-06).
No action retry, state reset or live account operation.

2026-11-06 — **Fixed privacy duplicate-value validation order.**
Supported readreceipts notifications now select the last raw value before enum
validation, matching the native map producer. An overwritten unsupported text
cannot suppress adoption/ACK; required text/structure and final validation stay.
[Evidence, session ACK-cut test and limits](docs/protocol/account-privacy-notifications.md#duplicate-category-values-are-validated-after-selection-2026-11-06).
No live account or automatic refresh operation.

2026-11-06 — **Preserved native signed blocklist error codes.**
Added optional blocklist_error_code to saved outcomes and CLI views; Java
signed32/default0 parsing no longer loses values through the generic u16 field.
Legacy rows remain unknown; request/retry behavior is unchanged.
[Evidence, Unicode/range tests and limits](docs/protocol/chat-actions.md#signed-blocklist-error-diagnostics-2026-11-06).
No live account operation.

2026-11-06 — **Fixed blocklist reply dispatch and error attribution.**
Owned outer results cannot be turned into failures by unrelated error
extensions; the separate snapshot validator still applies. Outer errors use
the first direct coded child rather than nested or uncoded errors.
[Regression and native400 reconciliation limits](docs/protocol/chat-actions.md#blocklist-outer-result-and-coded-error-settlement-2026-11-06).
No automatic retry, hash reset or live account operation.

2026-11-06 — **Location-disable lifecycle and retry eligibility resolved.**
Persistent job retains its sequence, rechecks sharing on each execution, and
uses ChatConnectionRequirement. Runtime exceptions bypass its retry decision;
other exceptions can retry/defer within the native runner's attempt budget.
[Evidence and remaining recovery limits](docs/protocol/chat-actions.md#location-disable-lifecycle-and-retry-boundary-2026-11-06).
No production retry change or live account operation.

2026-11-06 — **Blocklist location-disable wire and stale-job gate traced.**
Existing share state gates cancellation; the queued job rechecks current sharing
before sending notification/location with a saved sequence in disable.id.
Outer request ID, destination mapping and ACK future contract remain distinct.
[Evidence and unported state/recovery](docs/protocol/chat-actions.md#blocklist-observer-location-disable-obligation-2026-11-06).
No production protocol or live account change.

2026-11-06 — **Conditional blocklist observer identity query traced.**
DC3's selected job can call the existing missing-LID pipeline before location
state updates; per-chat state and helper filtering gate it. Network-unavailable
sentinel skips the updates; downstream location wire effects remain open.
[Evidence and limits](docs/protocol/chat-actions.md#blocklist-location-observer-query-boundary-2026-11-06).
No production protocol change or live account operation.

2026-11-06 — **Offline interface census closes blocklist observer handoff.**
Added apk-virtual-overrides --include-interfaces with checked DEX type lists and
transitive ancestry. Pinned callback routes to0d0.Blg;37 declared bodies retained,
without claiming active registrations or complete runtime dispatch.
[Tool, evidence and remaining jobs](docs/protocol/chat-actions.md#blocklist-observer-interface-census-2026-11-06).
No production protocol or live account change.

2026-11-06 — **Blocklist delta storage and side-effect boundaries retained.**
The async worker copies current membership, rewrites its table, then separately
applies hash preferences/metadata/follow-up work. Transaction failures and ignored
row results are distinguished; local system-message handoffs are traced.
[Evidence and concurrency limits](docs/protocol/chat-actions.md#blocklist-delta-storage-and-side-effect-boundaries-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist account-sync delta/refresh obligations traced.**
Matching previous hash/admission permits effective deltas; modify/mismatch or
failed admission queues kind4 refresh. Async persistence is separate from the
normal ACK boundary. Rust still captures this unsupported child without ACK.
[Processing and remaining ownership](docs/protocol/chat-actions.md#blocklist-account-sync-notification-obligation-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Dirty blocklist fetch role/admission gates resolved.**
Post-adoption dirty reconciliation is primary-role-only;09F.BU7 is companion
mode, not a feature flag. Readiness false bypasses mode checks; otherwise
blocklist mode/global chat-migration authority controls the selected path.
[Exact branches and missing Rust authority](docs/protocol/chat-actions.md#blocklist-dirty-fetch-role-and-admission-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Fetch results use the first direct named list.**
Removed the non-native duplicate-list veto; later children cannot stall the
selected valid snapshot. Malformed first matches still refuse without fallback,
and nested-only lists do not replace cached state. Five-case reopen regression.
[Evidence and limits](docs/protocol/chat-actions.md#blocklist-fetch-first-named-child-2026-11-06).
No new request or live account operation.

2026-11-06 — **Set-result hash follows requested-target membership change.**
Already-satisfied block/unblock retain cache/hash bytes; newly applied changes
adopt or clear the selected-list hash. Clarified that the native backend Boolean
means execution success, not affected-row change. Twelve-case reopen regression.
[Evidence and separate-state limits](docs/protocol/chat-actions.md#blocklist-set-hash-membership-gate-2026-11-06).
No new request or live account operation.

2026-11-06 — **Blocklist outbound hash omission follows distinct APK rules.**
Get omits null/empty; block/unblock also omit Java-blank values. Nonblank hashes
remain byte-for-byte unchanged. Controlled Java oracle checks every UTF16 unit;
synthetic get/block/unblock controls retain request/checkpoint evidence.
[Details and limits](docs/protocol/chat-actions.md#blocklist-outbound-hash-omission-2026-11-06).
No live account operation or automatic request.

2026-11-06 — **Full blocklist snapshots clear an absent hash.**
Corrected stale checkpoint retention after an admitted no-hash full snapshot.
No-list/malformed controls preserve prior state; offline get/block construction
controls and reopen regression retain evidence.
[Native/API authority and set-result limits](docs/protocol/chat-actions.md#blocklist-full-snapshot-nullable-hash-2026-11-06).
No automatic request or live account operation.

2026-11-06 — **Blocklist set results require the native first list.**
Corrected unsupported empty-result acceptance and root/trailing hash/branch
selection. The selected list is validated before cache/hash/reconciliation
projection; synthetic block/unblock refusal/reopen and first-list controls.
[Evidence and limits](docs/protocol/chat-actions.md#blocklist-set-first-list-admission-2026-11-06).
No new request or live account operation.

2026-11-06 — **Blocklist default metadata provider binding resolved.**
Pinned provider/cache/factory instructions connect7to provider0xb43 ->0x430
->0bZ, with delegated12d and metadata helper12g. This establishes packaged
default construction, not current runtime cache or complete backend ownership.
[Exact cases and boundaries](docs/protocol/chat-actions.md#blocklist-default-provider-binding-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist metadata concrete writer boundaries retained.**
Traced candidate store declarations, username trim/leading-@ normalization,
database-readiness/delegation and separate write/notification stages. Active
interface binding and atomic backend outcomes remain unproven; REA writer
body was unavailable while DEX instructions were retained.
[Findings and limits](docs/protocol/chat-actions.md#blocklist-metadata-concrete-write-boundaries-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist metadata worker and existing-value authority traced.**
Native metadata updates are separately scheduled. Username proposals require
absence in two consulted maps; display names preserve an existing nonblank
value. Associations sort active first, then lexical user, before ordered
submission. [Evidence and backend limits](docs/protocol/chat-actions.md#blocklist-metadata-adoption-authority-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist typed fallback and metadata priority traced.**
Pinned helpers show nullable requested-class conversion and type18-only
name -> PN association -> username priority, independent of block membership.
Retained six DEX helpers and complete REA Evidence; current strict Rust identity
admission and unified metadata rows remain identified gaps.
[Details and boundaries](docs/protocol/chat-actions.md#blocklist-typed-identity-and-metadata-priority-2026-11-06).
No Rust runtime or live account change.

2026-11-06 — **Blocklist snapshot membership follows the last repeated item.**
Corrected the confirmed APK Map.put mismatch: later non-block actions remove
earlier blocks and duplicate blocks no longer stall projection. Synthetic
ordering/reopen regression and retained pinned parser evidence;
[scope and limits](docs/protocol/chat-actions.md#blocklist-snapshot-repeated-identity-membership-2026-11-06).
No automatic request or live account operation.

2026-11-06 — **Migration JSON null/escaping reference executed offline.**

Reused existing shared envelope/AOSP inspection and added a hash-pinned,
controlled JSON-library oracle: four migration-shaped cases verify explicit
nulls, conditional Boolean fields, escaping/UTF8 and empty arrays. It executes
no WhatsApp class or device. [Authority, fixtures and limits](docs/protocol/chat-actions.md#migration-json-controlled-reference-2026-11-06).
No Rust sender or live request added; Pando/device behavior remains open.

2026-11-06 — **Migration intermediate values and coroutine lifetime narrowed.**

Direct GraphQL setters retain null through TreeMap export; wrapper omission
rules differ. Selected error policy is literal false and forwards Giw failure
to the waiting coroutine. Fresh operation supplies32s only to its non-Pando
send branch; final wire null encoding/Pando lifetime remain open.
[Evidence and limits](docs/protocol/chat-actions.md#migration-values-and-coroutine-completion-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Blocklist migration request and state authority traced.**

Pinned MigrateBlocklistLid MEX operation has a distinct input/hash/list/dirty-ack
builder and response path. Selected worker0 adopts a supplied snapshot; worker1
resolves/merges identities before migration. Persisted addressing mode and the
isRollback guard control state writes; completion log alone proves neither.
[Evidence, identifiers and unported boundaries](docs/protocol/chat-actions.md#blocklist-migration-request-and-state-authority-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Malformed blocklist children cannot erase the cache.**

Pinned parser requires every list child to be item. Rust now refuses unknown
children instead of projecting a partial/empty list, preserving every checkpoint,
hash, reconciliation warning and journal across reopen. Valid empty snapshots
remain valid; no request or retry added. [Evidence and regression](docs/protocol/chat-actions.md#blocklist-snapshot-child-admission-2026-11-06).
1,517 workspace tests (four threads), strict Clippy and repository gates pass.

2026-11-06 — **Unmatched blocklist results no longer falsely change the cache.**

Pinned set callback diverts list matched=false into reconciliation. Rust now
preserves blocked rows/hash, records a visible durable reconciliation warning,
and marks the result inspected atomically without replaying the set. An admitted
later full fetch repairs the cache; no-list replies cannot clear the warning.
[Evidence, regression and limits](docs/protocol/chat-actions.md#unmatched-blocklist-set-results-2026-11-06).
1,516 workspace tests (four threads), strict Clippy and repository gates pass.

2026-11-06 — **Automatic direct played receipts now honor known-disabled privacy.**

Pinned played/played-self selection exposed an unconditional Rust played queue.
New viewer-completion intents retain automatic provenance and the original wire;
known-disabled server preference holds them at queue/reservation and refuses a
late writer handoff. Consumption and uncertain reservations never reset.
Unknown/legacy provenance and group/explicit-read behavior remain unchanged.
[Evidence, regression and limits](docs/protocol/read-receipt-policy-current.md#automatic-played-privacy-correction-2026-11-06).
1,515 workspace tests (four threads), strict Clippy and repository gates pass.

2026-11-06 — **Native played-receipt exception retry eligibility established.**

Pinned constructor binds persistence, per-chat group, chat requirement and100
job-attempt budget; exception hook returns true. Shared worker still distinguishes
runtime exceptions, requirement deferral and normal callback completion. Rust's
uncertain-receipt no-replay policy remains an explicit lifecycle compatibility gap;
no unsafe retry loop or live operation added. [Evidence and limits](docs/protocol/receipt-ack-special.md#played-job-retry-eligibility-and-native-parity-gap-2026-11-06).

2026-11-06 — **Receipt dispatch drop/exception boundaries confirmed.**

Pinned APK dispatch can return without sending or completing its registered
callback. Selected played preparation preserves the original Message. Failure
censuses separate local future exception, size failure and disconnect cleanup;
none proves non-delivery or safe replay. No Rust behavior change or live action.
[Evidence and open boundaries](docs/protocol/receipt-ack-special.md#receipt-dispatch-drop-and-failure-boundaries-2026-11-06).

2026-11-06 — **Pinned receipt callback registration/completion traced.**

Current APK played job registers a full-key callback before dispatch and waits
on a future completed with null by the selected wrapper. This is local completion,
not server acceptance. No Rust behavior change or live operation.
[Selected methods, callers, reproduction and limits](docs/protocol/receipt-ack-special.md#current-receipt-registration-and-future-completion-2026-11-06).

2026-11-06 — **Opened-receipt collision and replay protections verified offline.**

New encrypted-store regression covers missing/unready sources, malformed routes,
and existing pending, uncertain, acknowledged or refused receipt intents. Refusals
preserve checkpoint bytes and journal across reopen. No production defect found
or behavior changed; queued intent is not transmission or acceptance.
1,513 workspace tests, strict Clippy and repository hygiene pass.
[Audit and limits](docs/protocol/media-network-outcomes.md#opened-receipt-collision-and-replay-audit-2026-11-06).

2026-11-06 — **View-once consumption now verifies the selected source identity.**

A still-decrypted replacement inbox row can no longer be overwritten while the
old image is consumed. Exact source check precedes decrypt/consumed writes;
valid one-time consumption and awaiting-viewer state remain intact.
[Regression, source and limits](docs/protocol/media-network-outcomes.md#view-once-consumption-source-identity-2026-11-06).
1,512 workspace tests and strict Clippy pass; no user content opened or live
account operation.

2026-11-06 — **View-once GET settlement now binds its reservation and inbox source.**

Late success/refusal cannot recreate removed state or overwrite changed outcomes;
the pre-network transaction also checks the original source row. Existing
uncertain outcomes remain no-retry. [Regression, evidence and limits](docs/protocol/media-network-outcomes.md#view-once-download-result-ownership-2026-11-06).
1,511 workspace tests and strict Clippy pass; no live account operation.

2026-11-06 — **Standard upload category/purpose native transfer confirmed.**

Little-endian serialization and checked native register/member flow connect
category8/purpose0 to WA priority context. The optional property3 suffix remains
unknown. [Reader, instruction checks and scope](docs/protocol/media-network-outcomes.md#standard-upload-categorypurpose-native-binding-closed-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Priority property-origin search expanded across Java/native boundaries.**

All24 physical key allocations include a dynamic decoder; request bridging also
uses it. Checked83 native members and direct setter calls; no original key3
producer proven. [Evidence, typed lookup and bounded negatives](docs/protocol/media-network-outcomes.md#priority-property-origin-java-and-cross-library-boundary-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Priority service registration and normal graph invocation confirmed.**

Named JNI binding and16-stage reverse walk close two previous boundaries;
priority stage wraps an existing chain, with no separate selection flag found.
Optional r property's producer/first-upload value remains unknown.
[Evidence and bounded negative searches](docs/protocol/media-network-outcomes.md#priority-jni-registration-and-normal-graph-activation-resolved-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **Downstream native Priority header construction found.**

Prioritization service calls calculator and builds u/i/p plus optional positive
property-driven r. Standard MOu metadata yields category8/purpose0; its WA
calculator prefix is u=3, i. Graph activation/key3 presence remain open before
Rust policy changes. [Complete evidence and boundaries](docs/protocol/media-network-outcomes.md#native-priority-header-producer-and-standard-upload-prefix-2026-11-06).
No Rust behavior change or live account operation.

2026-11-06 — **App priority calculator selection and enable-key source traced.**

0eA.A0z lazily constructs the holder from configuration key38178; the default
false constructor is not the selected path. Effective runtime value and wire
priority serialization remain unknown. [Evidence and scoped offset distinction](docs/protocol/media-network-outcomes.md#selected-priority-calculator-flag-source-2026-11-06).
No Rust request change or live operation.

2026-11-06 — **Native priority interceptor attaches typed scheduling context.**

Confirmed static registration/factory/vtable/invocation and calculator numeric
branches; no direct header insertion established in this interceptor. No Rust
wire change or live operation. [Evidence and remaining scheduling boundary](docs/protocol/media-network-outcomes.md#native-http-priority-context-separated-from-headers-2026-11-06).

2026-11-06 — **New media uploads require an explicit configured User-Agent.**

APK native HTTPS interceptor fills a missing header; Rust now refuses missing
client.user_agent before new upload state or transmission. Saved outcomes remain
unchanged. [Evidence, regression and boundaries](docs/protocol/media-network-outcomes.md#new-media-upload-user-agent-preflight-2026-11-06).
1,509 workspace tests and strict Clippy pass; no live operation or ban guarantee.

2026-11-06 — **Native request header member and timeout conversion traced.**

Decoded header map moves to request+0x130; four native timeout inputs pass through
validateTimeout. [Offsets, helper and local-bound distinction](docs/protocol/media-network-outcomes.md#native-header-transfer-and-timeout-normalization-2026-11-06).
No Rust timing/header change or live operation; downstream middleware remains open.

2026-11-06 — **Upload provider JNI class/table binding confirmed statically.**

Named registration binds inherited factory and stream/service methods; JNI ABI
slot independently checked. [Complete routine, tuple and source checks](docs/protocol/media-network-outcomes.md#upload-jni-class-to-table-registration-closed-statically-2026-11-06).
Runtime registration success unobserved; no Rust behavior change or live traffic.

2026-11-06 — **Selected JNI submission connected to native Java-body adapter.**

Exact tuples/factories/vtables identify a conditional upload-provider chain to previously
traced stream start; effective wrapper gate and downstream headers remain open.
[Static chain and retained scopes](docs/protocol/media-network-outcomes.md#selected-jni-upload-provider-adapter-connected-2026-11-06).
No Rust behavior change, replay or live operation.

2026-11-06 — **Selected Java upload header serialization traced to JNI.**

Existing header map is serialized separately from body provider; native internal
-1 candidate is not bound to ordinary uploads. [Full handoff/census and limits](docs/protocol/media-network-outcomes.md#java-upload-header-submission-boundary-2026-11-06).
No mandatory-header mismatch, Rust behavior change or live operation established.

2026-11-06 — **Native queue expected size traced to Content-Length.**

Header key verified by relocation; paired store initializes expected/queued
fields. [Exact producer, conversion and search exclusions](docs/protocol/media-network-outcomes.md#queue-expected-size-is-initialized-from-content-length-2026-11-06).
No Rust behavior change; preserve actual sealed-body length and uncertain rows.
No live operation or complete wire-parity claim.

2026-11-06 — **Native upload queue bound found; Pando flag scope separated.**

Selected queue rejects bytes above declared bound except-1 and rolls back this
increment. Disable-header getter belongs to Pando configuration. [Evidence and
unresolved expected-size producer](docs/protocol/media-network-outcomes.md#native-queue-bound-and-disable-header-namespace-2026-11-06).
No Rust behavior change or live operation; local error never authorizes replay.

2026-11-06 — **Native stored upload length reaches count/total callback.**

Selected consumer pairs declared length with accumulated count; progress semantics
are inferred, exact callback target remains unknown. [Evidence and physical-search
limits](docs/protocol/media-network-outcomes.md#stored-length-reaches-a-count-and-total-callback-2026-11-06).
No evidence of framing enforcement here; no Rust behavior change or live traffic.

2026-11-06 — **Conditional native upload length handlers identified.**

RTTI/vtable checks resolve limiting/MNS/HUC slot0x30 candidates. Limiting forwards
unchanged; selected MNS owner stores length under mutex. [Evidence and dynamic
binding limits](docs/protocol/media-network-outcomes.md#native-length-handler-candidates-resolved-2026-11-06).
No Rust behavior change or live traffic.

2026-11-06 — **Upload body provider traced to native length forwarding.**

Declared length reporting is separate from callback/EOF bytes; selected provider
compression is disabled. Native JNI wrapper reaches unresolved virtual slot0x30.
[Complete methods, pinned ELF tuples and limits](docs/protocol/media-network-outcomes.md#native-upload-length-handoff-2026-11-06).
No inferred truncation, Rust behavior change or live traffic.

2026-11-06 — **Media raw-body and fixed-length gates distinguished.**

Both transport gates derive from0x666; disabling them does not select multipart.
[Exact branches, constructors, complete REA and remaining provider inputs](docs/protocol/media-network-outcomes.md#raw-body-and-fixed-length-gates-are-separate-2026-11-06).
No proven mandatory framing mismatch or Rust behavior change; no live operation.

2026-11-06 — **Original ordinary-upload extra-header map traced.**

Ordinary message assembly supplies null; selected factories do likewise and
copies preserve it. Nonempty authorization/metadata map belongs to backup.
[Complete pinned origins and remaining transport/body-mode scope](docs/protocol/media-network-outcomes.md#original-media-extra-headers-traced-2026-11-06).
No Rust header change or live operation.

2026-11-06 — **Media header producer scope narrowed.**

Provider aggregation keys only newsletter; descriptor maps override defaults.
Standard voice descriptor copying preserves its prior map, so MetaAI null maps
are not ordinary-upload proof. [Evidence, exact provenance and open producers](docs/protocol/media-network-outcomes.md#media-header-producer-scope-2026-11-06).
Static research only; no Rust header change or live operation.

REA Android works in a fresh CLI after a backed-up, REA-only Java-path correction to the installed Temurin21 runtime. [Complete pinned-APK Evidence and provider-cache initialization trace](docs/protocol/apk-analysis.md#rea-android-java-path-repair-and-cli-proof-2026-11-06). The running MCP session still retains the old environment; no shared process was restarted. No Rust protocol, timer, retry or live-account behavior changed.

Full-key lifecycle trace now separates Dxh no-op reader cleanup from bulk FfK completion/failure, with exact failure callers and conditional shared-provider0x90 binding. [Pinned sources and cache/account-switch limits](docs/protocol/retry-answers.md#full-key-cleanup-and-future-failure-are-separate-2026-11-06). Research only: no inferred32s full-key timer, retry or live operation. REA Android call remains blocked by its configured missing Java21 executable; DEX fallback used.

Experimental direct-send key queries now reserve durable frozen-recipient ownership before the writer, reusing the existing key-batch mechanism. [Failing-before cut/cache fixtures, refusal/foreign-ID and reopen/no-replay controls](docs/protocol/replay-safety-audit.md#experimental-direct-cli-key-query-reservation-2026-11-06). Unbound legacy caches remain preserved and refuse use; verified answered plans avoid another query. Full-key native timeout remains unproven, so no guessed timer or live operation was added.

Explicit media refresh now uses the pinned nominal32s callback window instead of the generic30s wait, starting before write. Configured shorter socket deadlines still win. [Encrypted31s acceptance,32s expiry,30s I/O cap and reopen/no-resend controls](docs/protocol/media-network-outcomes.md#media-refresh-reply-window-2026-11-06). No live operation, automatic refresh or replay authority.

Four selected native wire-structure failures now capture/consume the receive epoch: invalid list marker, zero node count, null description and null attribute value. [Pinned list/constructor/exception evidence and encrypted/captured/uncertainty regressions](docs/protocol/frame-admission.md#selected-wire-structure-failures-are-terminal-2026-11-06). New InvalidWireStructure stays separate from unknown formats and local InvalidNode refusals; root/nested control boundaries are explicit. No live operation, replay permission or saved-state migration.

Truncated records and invalid packed values now capture/consume the receive epoch before later ACK or ping handling; packed bytes are checked completely before digit validation, matching the pinned reader order. [Native EOF routes, failing-before controls and durable receipt reopen/no-replay tests](docs/protocol/frame-admission.md#truncated-nodes-consume-the-receive-epoch-2026-11-06). Negative packed counts have their own InvalidPackedLength diagnostic; generic InvalidNode is not made terminal. Authenticated-empty-record skipping remains before decoding. No live operation or replay authority.

Compression failures now capture/consume the receive epoch before later ACK or ping handling. [Native I/O error route, failing-before encrypted/actor fixtures and durable receipt reopen/no-replay controls](docs/protocol/frame-admission.md#compression-failure-consumes-the-receive-epoch-2026-11-06). The existing local inflation cap also terminates; it is not a native size-limit claim. No live operation or replay authority.

Fixed authenticated empty-record admission: normal reader and both stanza receive APIs skip zero-length plaintext before diagnostics or routing, retaining counter consumption and caller-owned deadlines. [Pinned evidence, failing-before encrypted fixture and cancellation controls](docs/protocol/frame-admission.md#authenticated-empty-records-are-skipped-2026-11-06). No new sends, retry or live account operation.

Fixed frame-admission mismatch: header-only andmask0x01 fragmented records now capture/consume the epoch before parsing or ACK/ping handling. [Native refusal evidence, shared classifier and encrypted-record tests](docs/protocol/frame-admission.md). Existing transport recovery category retained; no retry/NACK/settlement or live operation added. Empty authenticated-record skipping is now implemented as described above.

Fixed coordinated root stream-end handling: rawroot token2 now captures/consumes the receive epoch and stops actor, captured waits and CLI windows before later ACK/ping processing. [Literal, real receipt/reopen and encrypted-record regressions](docs/protocol/node-serializer.md#root-stream-end-captures-and-consumes-the-receive-epoch-2026-11-06). Existing PeerClosed recovery category retained; no replay authority or live operation.

Identified unresolved stream-end parity gap: native rawroot token2 terminates reader/logout reporting; Rust actor/low-level paths still need coordinated capture-and-stop handling. [Concrete callback trace, regression requirements and policy limits](docs/protocol/node-serializer.md#root-stream-end-token-requires-coordinated-receive-termination-2026-11-06). Research only; no runtime fix, reconnect change or live operation claimed.

Fixed null-versus-empty attribute decoding: token0 values now explicitly fail instead of being treated as present empty text; explicitempty strings/bodies retain behavior. [Pinned constructor/non-null checks and literal controls](docs/protocol/node-serializer.md#null-attribute-values-cannot-become-empty-text-2026-11-06). No live operation, retry/NACK or saved-state migration.

Fixed legacy interop decoder identity loss: nonzero integrators that the current model cannot retain now explicitly refuse instead of silently re-encodingzero. [Literal fixtures and distinct native244 evidence](docs/protocol/node-serializer.md#legacy-interop-tuples-cannot-erase-a-nonzero-integrator-2026-11-06). Native244 support remains unported; no saved-state migration, replay or live operation.

Queue wake research now binds connection observer registration, conditional manager association and selector-condition wake, with executor-refusal and distinct requeue/persistence boundaries. [Pinned sources and remaining provider/restore authority](docs/protocol/message-ack.md#connection-observers-wake-the-selector-without-granting-send-authority-2026-11-06). No runtime/retry or live operation added.

Job requirement research binds localA04==2 admission and queue handoff, separating it from full writer/account readiness and replay authority. [Pinned requirement/selector evidence](docs/protocol/message-ack.md#job-connection-requirement-is-local-admission-not-replay-authority-2026-11-06). No runtime/retry change; provider wake, persistence/dedup and generation ordering remain open.

Disconnect follow-through identifies F1z/EuU message-send logging and exact reason mapping, correcting the unproven persistence/retry assumption. [Pinned consumer and remaining helper effects](docs/protocol/message-ack.md#disconnect-row-bookkeeping-feeds-logging-rather-than-a-proven-state-write-2026-11-06). No runtime/replay/telemetry change; actual durable retry and lifecycle authority remain open.

Disconnect research now separates full-key callback snapshot/clear/local failure, saved-row bookkeeping and directA05 cleanup. [Pinned handler/worker evidence and durable Rust controls](docs/protocol/message-ack.md#disconnect-clears-native-callbacks-without-proving-non-delivery-2026-11-06). No runtime/replay change; complete bookkeeping persistence, lifecycle races and restart ownership remain open.

Fixed APK-confirmed string-context encoding differences: names/JID_PAIR servers retain token lookup but disable packing, and node text bodies serialize as raw bytes. [Literal fixtures and constructor/writer evidence](docs/protocol/node-serializer.md#names-and-body-text-use-different-string-rules-2026-11-06). No live operation, auto text-to-JID conversion or retry/settlement change.

Fixed APK-confirmed packed-string mismatches: colon nibble12 now encodes/decodes, invalid actual13..15 reject, odd padding is ignored before validation, and negative digit counts reject. [Literal fixtures and native reader/writer evidence](docs/protocol/node-serializer.md#packed-nibble-colon-and-padding-match-the-apk-2026-11-06). No live operation or retry/settlement change.

Fixed APK-confirmed binary-node encoding mismatch: empty child arrays now omit content instead of adding LIST_EMPTY; empty bytes remain present. [Literal fixtures, conditional serializer budget and remaining codec gaps](docs/protocol/node-serializer.md). No live operation, new retry or settlement change.

Status final writer now traced through prepared message construction, counter-before-output, three-byte framing/flush and size-failure callback routing. [Pinned evidence and Rust uncertainty controls](docs/protocol/message-ack.md#status-final-writer-consumes-the-transport-counter-before-output-2026-11-06). No runtime/replay change; full serializer, underlying socket and disconnect ownership remain open.

Status submit callback research now separates registration, queue submission, null-valued future completion and ordinary ACK outcomes. Device0 normalization/duplicate replacement/timeout removal are retained with ownership limits. [Pinned sources and concrete writer gap](docs/protocol/message-ack.md#status-submit-future-completes-callbacks-without-carrying-acceptance-2026-11-06). No runtime/replay change; final writer/ciphertext lifecycle remains open.

Status-send research now reaches persisted hash/device-attempt markers, conditional root/child rendering, expected ACK key and feature-controlled wait after0Sf submission. [Pinned job/helpers and remaining writer/value authority](docs/protocol/groups.md#current-status-job-wire-rendering-and-pre-submit-markers-2026-11-06). No runtime/replay change; actual writer/ciphertext/callback contract remains open.

Status builder research now binds canonical ACK readiness, stored batch/part selection and row/UUID-protected possible-transmission markers. [Complete producer exports and guard comparison](docs/protocol/message-ack.md#status-builder-distinguishes-canonical-part-ack-from-possible-transmission-2026-11-06). No runtime/replay change; ciphertext, actual status renderer and final writer authority remain open.

Canonical status ACK wait research now binds exact exception reasons, row/part/owner identity and commit-before-callback wake ordering. [Complete evidence, transition table and dedup limits](docs/protocol/message-ack.md#canonical-status-ack-waits-require-committed-part-and-callback-ownership-2026-11-06). No runtime/replay change; current-batch producers, restart/clock and actual sender construction remain open.

Status hash-repair research now identifies the actual sync/resend job, operation/expiry gates and canonical ACK row/part wait ownership. [Pinned producers/job and remaining writer/callback gaps](docs/protocol/message-ack.md#status-hash-repair-queues-a-guarded-sync-and-resend-job-2026-11-06). No runtime change or replay expansion; actual request construction and full job admission remain open.

Status ACK persistence research now binds key/row authority, hash-repair continuation, exact state predicates and backend-specific updates. Timestamp scaling matches the existing Rust parser. [Complete evidence, state table and limits](docs/protocol/message-ack.md#status-persistence-has-state-and-key-authority-beyond-ack-parsing-2026-11-06). No runtime change; repair producers, complete DB/callback ownership and scheduled operations remain open.

Async status ACK edit admission now retains uncertainty for malformed/out-of-i32 text before success/refusal, with signed/Unicode and459 controls. Reader remains usable. [Pinned runnable, downstream findings and reopen matrix](docs/protocol/message-ack.md#async-status-ack-edit-parsing-precedes-persistence-2026-11-06). Offline only; private status/multipart persistence,404 revoke handling and scheduled callbacks remain open.

Direct PN/LID message ACKs flagged exactis_group_status=true now retain uncertainty on success/refusal, matching the native diversion's ignored-chat admission. Selected recipient still takes precedence;459 remains separate. [Pinned predicates and reopen matrix](docs/protocol/message-ack.md#direct-group-status-message-ack-diversion-precedes-terminal-outcomes-2026-11-06). Offline only; async status/group/newsletter and scheduled callback semantics remain open.

Self-user ACK conversion now retains uncertainty for barestatus_me/lid_me in participant/originalfrom on both success and refusal; selected recipient cannot hide the conversion failure. Self recipient precedence and call type11 rejection are preserved. [Pinned overrides and reopen matrix](docs/protocol/message-ack.md#self-user-ack-conversion-failure-is-not-an-ordinary-participant-result-2026-11-06). Offline only; qualified cache aliases and full self-chat behavior remain open.

BarePN/group/call ACK identities now retain native subtype distinctions under the traced cache producers: barePN/group reject call sender admission, barecall passes; all three fail message-chat/UserJid casts. [Complete classes, cache proof and actor controls](docs/protocol/message-ack.md#bare-server-ack-identities-follow-bounded-native-subtypes-2026-11-06). Offline only; user cache aliases and binary/payload behavior remain open.

Call/temp ACK text now follows pinned constructor and subtype rules. Invalid text preserves null ordering; validcall/type10 passes sender-type admission, temp/type2 rejects. Call JID cannot establish a message chat; temp selected-from preserves original user. [Evidence and controls](docs/protocol/message-ack.md#call-and-temporary-ack-jid-text-keep-distinct-native-types-2026-11-06). Offline only; bare/binary/cache/payload behavior remains open.

Broadcast ACK text now follows native numeric-list and exactstatus/location construction. Valid broadcast call senders reject; malformed text retains null ordering and capture/continuation. NonUser recipient cast and original selected-from text are preserved. [Pinned sources and controls](docs/protocol/message-ack.md#broadcast-ack-text-preserves-constructor-and-special-subtypes-2026-11-06). Offline only; binary/cache and generic receipt behavior remain open.

Group/newsletter ACK text now uses pinned native constructor rules before call admission and selected-chat matching. Newsletter validation preserves original user and does not require digits; dot suffix is not device syntax. Malformed group/newsletter senders retain native null ordering. [Evidence and boundary controls](docs/protocol/message-ack.md#group-and-newsletter-ack-text-uses-native-construction-2026-11-06). Offline only; generic receipt and binary/cache behavior remain open.

Explicit0/Server PN device ACK text now follows native constructor/class rules: valid devices pass sender-type admission and convert as participants, includingServer devices; invalid construction stays null. Recipient device0 falls back to from, selected device from unwraps canonical base. [Evidence and reopen controls](docs/protocol/message-ack.md#explicit-special-pn-devices-preserve-device-class-2026-11-06). Offline only; binary/cache/service-chat behavior remains open.

Plain special PN ACK identities now retain native class:0/Server are UserJid, gdpr is not. Call rejection, recipient fallback/precedence and participant-success admission follow those distinctions. Explicit special-user devices remain open. [Pinned classes and reopen controls](docs/protocol/message-ack.md#plain-special-pn-ack-identities-preserve-native-class-2026-11-06). Offline only.

Ordinary PN/hosted ACK text now follows its pinned constructor grammar and canonical server, including explicit device0 class and hosted99 rules. Invalid recipient falls back to from; call ordering and ordinary-success participant checks share that result. Special0/Server/gdpr remain unclassified. [Evidence, reopen controls and limits](docs/protocol/message-ack.md#pn-ack-text-follows-its-own-constructor-grammar-2026-11-06). Offline only.

Explicit LID ACK text now preserves native user/device/null class, signed integer parsing, byte-range construction and hosted99 behavior. Selected chat and participant-success guards use that result; device0 is not mistaken for a plain recipient. [Pinned constructors, ordering and reopen controls](docs/protocol/message-ack.md#explicit-lid-device-ack-text-preserves-native-class-and-narrowing-2026-11-06). Offline only.

Plain LID ACK text now follows the pinned ASCII/nonzero/length constructor: invalid sender text retains call receive continuation; invalid recipient falls back to from. Valid hosted.lid text canonicalizes to lid before saved-chat matching. [Constructor evidence, boundary/reopen controls and limits](docs/protocol/message-ack.md#plain-lid-text-uses-constructor-grammar-and-canonical-server-2026-11-06). Offline only.

Call ACK receive admission now preserves the pinned recoverable path for definite invalid text senders (unknown bare/domain and empty-user forms), without guessing native null from Rust JID parse failure. Native bare values/cache aliases retain separate handling. [Factory/cache evidence and continuation controls](docs/protocol/message-ack.md#definite-null-call-sender-text-versus-native-bare-cache-values-2026-11-06). Offline only.

Corrected call-ACK admission ordering: absent/empty-text sender follows the pinned generic Exception path and keeps receiving, even if later call fields are empty/missing. Required-id/numeric admission still runs earlier; fatal nonnull-sender cases remain. [Selected catches and continuation regression](docs/protocol/message-ack.md#known-null-call-sender-precedes-fatal-class-field-checks-2026-11-06). Offline only.

Pinned ordinary call ACK admission now rejects empty id, missing/empty type and confirmed group/newsletter sender types through shared capture-before-stop handling. Numeric459 keeps its pre-class diversion; other classes retain existing rules. [Exact trace, controls and remaining call gaps](docs/protocol/message-ack.md#ordinary-call-ack-text-fields-and-known-sender-rejection-2026-11-06). Offline only.

Durable receipt callback settlement now validates every member’s receipt key, uniqueness, status, account/owner binding and ACK route before committing. Equal bytes alone can no longer authorize unrelated or rebound checkpoint updates. [Local integrity gap and 24-case regression](docs/protocol/replay-safety-audit.md#validate-durable-callback-member-semantics-before-settlement-2026-11-06). Offline only.

Owned receipt writers now recheck the exact durable callback record alongside account/member/privacy authority. Late owner deletion, consumption or corruption blocks the entire queued batch before encoding. [Local gap and 105-case actual writer regression](docs/protocol/replay-safety-audit.md#receipt-writer-also-owns-the-durable-callback-record-2026-11-06). Offline only.

Receipt ACKs now settle only durable original batch members, atomically consuming their encrypted callback owner. Row replacements and same-wire foreign rows cannot be mistaken for that callback. Original late ACKs after partial write/reconnect still settle unchanged reservations without resend. [Ownership, compatibility and tests](docs/protocol/replay-safety-audit.md#durable-original-receipt-callback-members-2026-11-06). Offline only.

Receipt reservations now persist their local account-authority digest through ACK settlement. Changed account/Signal identity or missing/corrupt legacy binding leaves matching rows uncertain instead of recording success/refusal. [Encrypted metadata, compatibility and reopen regression](docs/protocol/replay-safety-audit.md#persist-receipt-reservation-authority-through-ack-settlement-2026-11-06). Offline only.

Owned receipt batches now bind account/Signal authority at reservation and recheck it at the actual writer alongside member/privacy guards. Late account replacement or identity change blocks the whole batch and retains uncertainty; routine credential/time observations remain allowed. [Local ownership evidence and 90-case regression](docs/protocol/replay-safety-audit.md#receipt-writer-retains-its-reservation-account-authority-2026-11-06). Offline only.

The native ID-only ACK maps now have a bounded producer trace: explicit tracked-send records include VoIP plus reconnect/challenge resubmission; no ordinary receipt producer was established. This does not authorize Rust uncertain-send replay or loosen receipt matching. [Evidence and limits](docs/protocol/receipt-ack-special.md#bound-the-independent-id-map-producers-before-changing-receipts-2026-11-06). Static/offline only.

Pinned generic receipt-ACK audit confirms a full-key callback lookup including type/participant, followed by independent ID-based cleanup/callback maps. Their producers remain untraced; no blanket native ownership claim or Rust settlement change. [Evidence and negative finding](docs/protocol/receipt-ack-special.md#current-generic-callback-key-and-separate-id-maps-2026-11-06). Static/offline only.

Ordinary participant-success admission is now shared across outbox, distribution waits and CLI expected-chat outcomes. Plain/device PN/LID participant success stays uncertain; ordinary refusals and459 remain separate. Refreshed43.72 evidence retains the unported bot-feedback exception explicitly. [Evidence and tests](docs/protocol/message-ack.md#share-ordinary-participant-success-admission-2026-11-06). Offline only.

Out-of-band distribution ACK waits now bind their actual reserved destination as well as ID; wrong-chat ACKs cannot release dependent group ciphertext. CLI direct/group/status/distribution results share selected-chat classification and retain outcome_unknown for address mismatch. [Evidence and regression scope](docs/protocol/message-ack.md#bind-distribution-and-comparison-acks-to-the-expected-chat-2026-11-06). Offline only.

Ordinary outbox ACK settlement now selects supported numeric PN/LID recipient before from, and unwraps PN/LID device from when no user recipient applies. Conflicting recipient preserves uncertainty. [Pinned selection, tests and remaining adapters](docs/protocol/message-ack.md#select-the-message-ack-chat-before-outbox-correlation-2026-11-06). Offline only.

Pinned ACK address-accessor audit retained a negative finding: optional address parse/cast failure does not establish the required-ID fatal behavior. No blanket fatal address gate was added. Native recipient-preferred message routing and generic callback ownership remain open. [Evidence and next scope](docs/protocol/message-ack.md#optional-typed-ack-addresses-are-not-a-blanket-fatal-gate-2026-11-06). Static/offline only.

Missing ACK id now stops stateful reception after capture, before error459/class dispatch, matching the pinned required accessor. Pending work remains uncertain; empty textual id retains native null-check behavior. [Evidence and limits](docs/protocol/message-ack.md#required-ack-id-before-error-and-class-routing-2026-11-06). Offline only.

Valid signed/Unicode ACK errors now retain their numeric delivery-fact code across reopen, using the same pinned parser as admission. Historical unsigned JSON facts remain readable; public DeliveryFact.code widens to Option<i64>. [Evidence and compatibility](docs/protocol/message-ack.md#preserve-admitted-signed-ack-error-codes-2026-11-06). Offline only.

Malformed ACK numeric errors now stop normal actor/CLI stateful receive and captured waits after preserving evidence, matching the selected pinned reader exit. Pending work stays uncertain; no generated ACK response or automatic reconnect. [Native routing and limits](docs/protocol/message-ack.md#malformed-ack-consumes-the-receive-epoch-after-capture-2026-11-06). Offline only.

Malformed/out-of-i32 root ACK errors now preserve uncertain state across direct, distribution, receipt and CLI classification instead of becoming ordinary failures. Valid signed/Unicode errors retain prior behavior. [Pinned parser/catch evidence, tests and remaining scope](docs/protocol/message-ack.md#integral-error-admission-before-ordinary-ack-outcomes-2026-11-06). Offline only.

Receipt ACK numeric459 now preserves single/batched reservations instead of recording an ordinary refusal, matching the pinned pre-class auth-challenge branch. [Regression, source and open recovery boundary](docs/protocol/receipt-ack-special.md#current-auth-challenge-diversion-precedes-receipt-settlement-2026-11-06). Offline only; no challenge bypass or replay.

Actor and CLI receipt writes now retain every reserved batch member and recheck ownership plus the existing automatic direct-read privacy gate before encoding. Refusals preserve unknown reservations without replay. [Owned receipt handoff and limits](docs/protocol/replay-safety-audit.md#receipt-writer-owns-every-reserved-batch-member-2026-11-06). Offline only.

Queued normal actions now recheck late-recorded reach-out restrictions for group creation and server backoff for app-state fetch/patch/held-fetch before encoding. Refusals preserve intent and waits without replay. [Writer admission and limits](docs/protocol/action-settlement-ordering.md#writer-rechecks-recorded-restrictions-and-explicit-backoff-2026-11-06). Offline only.

Normal action callbacks now retain the written reservation through atomic reply settlement; old replies cannot consume replaced rows under the same ID. Duplicate live registration preserves the first owner/deadline. [Action callback binding and limits](docs/protocol/action-settlement-ordering.md#action-callbacks-retain-the-written-reservation-2026-11-06). Offline only.

Normal queued actions now require their exact reserved row and original producer deadline before encoding. Changed, missing or expired intents refuse without replay; untimed requests remain untimed. [Action writer boundary and remaining gaps](docs/protocol/action-settlement-ordering.md#queued-action-writer-ownership-and-deadlines-2026-11-06). Offline only.

Pinned presence reader now traced; missing type correctly reports available for primary-user metadata. Device/group routing and signed last-seen/default-clock projection remain partial. [Current presence audit](docs/protocol/presence.md#current-readerdefault-type-audit-2026-11-06). No outgoing request or live operation.

Retry plaintext retention now refuses conflicting saved bytes; both resend commits recheck the source snapshot used by crypto. Exact repeats remain idempotent, and conflicts preserve state. [Source fidelity and limits](docs/protocol/retry-answers.md#original-retry-plaintext-retention-and-commit-binding-2026-11-06). Offline only.

Retry reservations now recheck registered local account, store generation, address and Signal identity at commit across saved-session, fresh-key query and fresh-bundle paths. Observation-only updates remain allowed. [Account guard and limits](docs/protocol/retry-answers.md#retry-reservations-recheck-local-account-authority-2026-11-06). Offline only.

Live retry-refetch callbacks now retain the written job snapshot; old replies cannot settle a replacement job under the same query ID. Duplicate active IDs refuse without replacing owners. [Mechanism and limits](docs/protocol/retry-answers.md#live-callbacks-retain-the-written-job-2026-11-06). Offline only.

Retry key-refetch ACK/query writes now recheck the reserved job and fetching answer at writer admission. Queued stale work refuses before encoding; no replay or guessed deadline. [Ownership fix and boundaries](docs/protocol/retry-ack-dispatch.md#queued-key-refetch-writes-retain-reservation-ownership-2026-11-06).

Media refresh audit verified existing request/prior-route ownership and request-time TTL guards; corrected stale first-route false documentation. No runtime change. [Audit scope and open boundaries](docs/protocol/media-network-outcomes.md#refresh-authority-and-lifetime-audit-2026-11-06).

Shared upload entry now checks config and registered matching account before cache reuse, route selection or persisted randomness/reservation. [Shared upload account/config preflight](docs/protocol/media-network-outcomes.md#shared-upload-preflight-precedes-cache-and-reservation-2026-11-06). Both public entry points have offline positive/refusal controls.

Ordinary media GET success/refusal now require the exact initiating reservation; late results cannot overwrite changed or settled state. [Media GET result reservation ownership](docs/protocol/media-network-outcomes.md#download-result-reservation-ownership-2026-11-06). Shared HTTP client already disables redirects/retries; offline verification only.

Media upload headers/body/protobuf transitions now require exact expected reservation state; independent encrypted HTTP observations survive stale settlement refusal. [Media-upload response reservation ownership](docs/protocol/replay-safety-audit.md#media-upload-response-transitions-retain-reservation-ownership-2026-11-06). Existing one-POST/no-retry behavior retained; offline only.

Separate CLI group distributions now reserve frozen child wire/ID before write and require exact parent/child authority at settlement. [Separate distribution reservation ownership](docs/protocol/replay-safety-audit.md#cli-separate-distributions-reserve-before-write-2026-11-06). Duplicate/changed/reopen paths refuse; no live operation.

CLI group ACK now requires exact current uncertain intent before status settlement or sender-key distribution confirmation. [Group ACK intent and distribution authority](docs/protocol/replay-safety-audit.md#cli-group-ack-owns-intent-before-distribution-promotion-2026-11-06). Real distribution-state stale/duplicate/reopen controls; no live operation.

CLI direct-send ACK settlement now requires the exact current uncertain intent; stale or duplicate responses cannot overwrite changed/refused/acknowledged state. [CLI direct-send ACK intent authority](docs/protocol/replay-safety-audit.md#cli-direct-send-ack-retains-intent-authority-2026-11-06). Captures retained; no live operation.

CLI login-success identity writes now share canonical PN/LID validation and partial-update rules; malformed supplied fields refuse atomically instead of becoming absent. [Shared CLI login identity validation](docs/protocol/login-identity-updates.md#cli-shares-the-authenticated-identity-update-2026-11-06). Capture remains durable; no live operation.

CLI login now shares actor credential snapshots and checks unique reservation ownership before result/account writes; decoded reply capture commits first. [CLI login snapshot and reservation ownership](docs/protocol/authkey-lifecycle.md#cli-login-snapshot-and-reservation-ownership-2026-11-06). Offline stale/duplicate/reopen controls; no live operation.

CLI initial account load now uses the shared registration-material consistency guard in one read snapshot, before send-path claims or other side effects. [CLI registration-copy preflight](docs/protocol/authkey-lifecycle.md#cli-account-load-checks-the-registration-copy-2026-11-06). Awaited reservation/result ownership remains open.

Fixed CLI Noise preflight omission: saved halves are checked before refusal-history writes, image preparation and sockets using the shared Curve25519 validator. [CLI Noise preflight before side effects](docs/protocol/authkey-lifecycle.md#cli-validates-noise-halves-before-side-effects-2026-11-06). CLI registration-copy/ownership gaps remain separate.

Fixed CLI refusal-recheck stale deletion and history collision: archive/delete requires the exact current failure and absent/identical archive. [CLI refusal recheck authority](docs/protocol/authkey-lifecycle.md#cli-refusal-recheck-retains-current-authority-2026-11-06). Unknown/newer login state stays intact; offline regression only.

Fixed comparison connection login-result ownership: stale success/failure cannot settle another reservation or apply clock/account state after credentials change. [Login reply reservation and credential ownership](docs/protocol/authkey-lifecycle.md#login-reply-owns-its-reservation-and-credentials-2026-11-06). Capture remains first; separate CLI lab path remains outside this helper.

Fixed stale-preflight login reservation: current account/material must match the prepared snapshot atomically before outcome_unknown is written. [Login reservation binds its preflight snapshot](docs/protocol/authkey-lifecycle.md#login-reservation-binds-its-preflight-snapshot-2026-11-06). Reopen and fresh/duplicate reservation controls pass; no live operation.

Fixed local login preflight gap: with Signal enabled, a valid account Noise pair cannot silently disagree with present registration material. [Registration-copy consistency before login](docs/protocol/authkey-lifecycle.md#registration-copy-consistency-before-login-2026-11-06). Regression covers unchanged state and reopen; no rotation or live operation.

Auth-key/token audit reconciled with earlier retained evidence: selected token grammar/reply/failure paths are already traced; lost-result recovery and atomic Rust login-material installation remain open. [Canonical auth-key implementation boundary](docs/protocol/authkey-lifecycle.md#current-implementation-boundary-reconciled-2026-11-06). Documentation correction only.

Pinned rotation now linked to saved-token connection send and token-response callback, with bounded candidate reuse evidence. [Auth-key rotation triggers and candidate ownership](docs/protocol/notification-ack-audit.md#auth-key-rotation-triggers-and-candidate-ownership-2026-11-06). Durable recovery remains unverified; no runtime change.

Pinned auth-key concrete provider binding and type2 encrypted-record roles now traced. [Auth-key concrete storage provider](docs/protocol/notification-ack-audit.md#auth-key-concrete-storage-provider-2026-11-06). Android storage format remains separate from Rust protocol requirements; runtime lifecycle unfinished.

Pinned auth-key replacement uses checked commit before readback; reload/type/length gates retained. [Auth-key commit and reload boundary](docs/protocol/notification-ack-audit.md#auth-key-commit-and-reload-boundary-2026-11-06). Concrete encryption provider and recovery remain incomplete; research only.

Pinned authentication-key candidate layout and password-backed replacement readback now traced. [Auth-key candidate and storage verification](docs/protocol/notification-ack-audit.md#auth-key-candidate-and-storage-verification-2026-11-06). Local storage retry does not authorize network resend; runtime lifecycle remains unported.

Pinned authkey_rotation notification now traced through counter/time gates, w:auth:key publication and distinct callback outcomes. [Auth-key rotation lifecycle and missing Rust obligations](docs/protocol/notification-ack-audit.md#auth-key-rotation-is-a-state-lifecycle-2026-11-06). Missing lifecycle remains explicit; no ACK-only port or live operation.

Pinned MEX error delivery now separated from telemetry: exception callback wraps a fresh Hbw before the product error handler. [MEX error delivery versus telemetry](docs/protocol/mex.md#current-mex-error-delivery-versus-telemetry-2026-11-06). Broader backoff consumers remain unverified; no runtime change.

Pinned MEX IQ request/server error families and exact-one server-error child retained; transport errors remain distinct from GraphQL scoped backoff. [MEX IQ error families and backoff boundary](docs/protocol/mex.md#current-mex-iq-error-families-and-backoff-boundary-2026-11-06). Research only; no guessed retry or runtime change.

Pinned Android16 JSONObject(Map) preserves MEX cached-device nulls; APK call site supports a platform-qualified explicit-null prediction. [MEX null serialization platform boundary](docs/protocol/bulk-prekey-refresh.md#missing-lid-mex-null-serialization-platform-boundary-2026-11-06). Pando/runtime identity remain unverified; research only.

Pinned device99 LID constructor uses a distinct hosted.lid domain; Rust hash/reconciliation remains restricted to ordinary PN/LID. [Hosted LID device99 boundary and remaining dispatch/authority limits](docs/protocol/cross-device-sync.md#hosted-lid-device99-hash-boundary-2026-11-06). Research only; no runtime widening or live operation.

Pinned self-PN/LID hash input helpers now traced through record filtering and account-specific device-number-preserving normalization. [Own-account source filtering, device-preserving mapping and limits](docs/protocol/cross-device-sync.md#current-self-device-hash-input-normalization-2026-11-06). Research only; general alias/identity authority unchanged.

Pinned43.72 device-hash format bodies now match both existing Rust versions in the selected other-user scope; existing vectors pass. [Current formatter comparison, existing vectors and scope limits](docs/protocol/cross-device-sync.md#current-apk-device-hash-format-comparison-2026-11-06). Effective version/self-device/cache authority remain incomplete.

Pinned MEX per-user device cache fields now traced: nonempty hash gate, decimal-string timestamp fields and strict expected-timestamp comparison. [Device cache gates, decimal-string timestamps and unresolved authority](docs/protocol/bulk-prekey-refresh.md#missing-lid-mex-device-cache-fields-2026-11-06). Research only; cache/hash/null serialization remain incomplete.

Pinned UsyncQuery variables now traced through raw-JID targets, nested MESSAGE telemetry and conditionally omitted include flags. [Raw-JID targets, MESSAGE telemetry and omitted-flag semantics](docs/protocol/bulk-prekey-refresh.md#missing-lid-usyncquery-variables-spine-2026-11-06). Research only; complete per-user variables remain unfinished.

Pinned UsyncQuery server doc_id now recovered with exact asset and normal resolver chain; client_doc_id remains a distinct value. [Pinned server ID versus client ID, exact asset and resolver scope](docs/protocol/bulk-prekey-refresh.md#missing-lid-usyncquery-server-id-2026-11-06). Research only; no runtime lookup added.

Current APK shared MEX request wrapper matches the existing Rust envelope in the selected fields; query-specific parity remains incomplete. [Current shared-envelope comparison and assertion limits](docs/protocol/mex.md#current-mex-request-envelope-comparison-2026-11-06). Research only.

Pinned MEX server equality narrowed: request-to and exact s.whatsapp.net text resolve to0mL; user/server/type determine inherited equality. [Concrete server equality, diagnostic-only compliance check and remaining decoder limits](docs/protocol/mex.md#selected-mex-server-jid-equality-2026-11-06). Research only; binary/general aliases remain unverified.

Pinned MEX constructors now traced through original-ID and typed reply-from/request-to equality; Rust JSON payload parsing remains separate from live envelope admission. [Request ID/from equality evidence and payload-only Rust boundary](docs/protocol/mex.md#original-request-mex-envelope-correlation-2026-11-06). Research only; no guessed sender policy added.

MEX JSON parser now rejects duplicate result payloads instead of choosing the first, matching the pinned non-Pando success constructor cardinality. [Pinned cardinality evidence, both-order regression and limits](docs/protocol/mex.md#duplicate-result-payload-rejection-2026-11-06). Offline only; no query/retry added.

Non-Pando MEX expiry now traced from registry removal through typed local delivery failure to exceptional future completion. [Registry expiry, typed delivery failure and late-reply limits](docs/protocol/bulk-prekey-refresh.md#missing-lid-non-pando-mex-expiry-outcome-2026-11-06). Research only; no server471/retry or complete late-reply guarantee.

Missing-LID MEX lookup now traced through fresh operation/default timeout and discarded start handle; non-Pando send uses32seconds while outer wait uses128seconds. [Operation handle ownership, conditional32-second send and remaining transport limits](docs/protocol/bulk-prekey-refresh.md#missing-lid-mex-operation-ownership-and-nested-send-timeout-2026-11-06). Research only; Pando/late-callback limits remain.

Native missing-LID response-state reset/cleanup now retained with exact field/caller census; late callback fencing remains unverified. [Explicit resets/cleanup and remaining late-callback limits](docs/protocol/bulk-prekey-refresh.md#missing-lid-shared-response-state-resets-2026-11-06). Research only, no runtime change.

Missing-LID MEX callbacks now traced through request/scoped/exception outcomes and response-state-before-future completion. [MEX request/scoped/exception outcomes and remaining lifecycle limits](docs/protocol/bulk-prekey-refresh.md#missing-lid-mex-callback-completion-and-error-scope-2026-11-06). Research only; no mapping acceptance or runtime parity claim.

Pinned APK tracing now shows a LID-only multi-query branch selects MEX UsyncQuery/include_lid, while devices uses a separate flag. [LID field chain, conditional MEX selection and remaining serializer/callback limits](docs/protocol/bulk-prekey-refresh.md#missing-lid-field-mapping-and-mex-selection-2026-11-06). Research only; no runtime request added.

Contact projection and LID repair now withhold LID on duplicate protocol metadata/user fields; duplicate contact fields cannot choose status by order. [Nested duplicate-field controls and native-policy limits](docs/protocol/contact-projection-correlation.md#duplicate-field-authority-is-not-selected-by-order-2026-11-06). Local ambiguity policy, offline only.

Contact-result ingestion now reads current settlement, request and reply together before projecting status, retaining settlement order and reply correlation. [Transaction snapshot fix, stale-enumeration controls and limits](docs/protocol/contact-projection-correlation.md#transaction-bound-settlement-projection-2026-11-06). Offline only; no query or replay added.

Local contact LID repair now reads and validates action, original request and correlated result inside its mapping update transaction, rejecting ambiguous containers. [Original request authority, nine corruption/reopen cases and limits](docs/protocol/chat-actions.md#lid-repair-retains-original-request-authority-2026-11-06). Offline only; no lookup/replay added.

Missing-LID preflight now bound to the native multi-query handler, with separate128-second send/wait arguments on its selected dispatch. [Multi-query route and nested timing limits](docs/protocol/bulk-prekey-refresh.md#missing-lid-multi-query-route-and-nested-timing-2026-11-06). Research only; final wire and adoption still incomplete.

Missing-LID preflight preset is now bound to LID_REQUEST/message query; initial LID-only flags can be combined by the native queue. [Preset, queue combination and remaining wire limits](docs/protocol/bulk-prekey-refresh.md#missing-lid-preset-and-queue-combination-2026-11-06). Research only, no runtime request change.

Bulk-key missing-LID preflight now traced through filtering, sync queue submission and untimed wait before key dispatch. [Conditional missing-LID preflight and remaining wire/adoption limits](docs/protocol/bulk-prekey-refresh.md#bulk-pre-key-missing-lid-preflight-2026-11-06). Research only; no extra lookup or retry added.

Bulk-key job completion now has a concrete future/runner trace: an IQ error
can finish the job normally while separate callbacks report refusal. Runtime
exceptions are rethrown; exception retry/defer is not server-error acceptance.
[Completion evidence and limits](docs/protocol/bulk-prekey-refresh.md#bulk-job-completion-is-not-fetched-key-acceptance-2026-11-06).

Current APK tracing separates ordinary and conditional PQ identity fetches:
both are identity-change reads, neither is a PQ session upgrade. Their native
in-flight keys include mode; 60-second cleanup does not authorize durable replay.
[Manager evidence and remaining gates](docs/protocol/bulk-prekey-refresh.md#current-notification-fetch-flags-and-manager-suppression-2026-11-06).

Fresh APK43.72 tracing finds no explicit fetched-key/adoption dependency
in normal identity notification ACK dispatch. General failure handling
differs from parser LIL; no ACK widening was justified or implemented.
[Current dispatch evidence and limits](docs/protocol/identity-notification-ack.md#current-apk-identity-dispatch-and-ack-independence-2026-11-06).

Identity-only reads now use the APK43.72 producer-specific32-second callback
timer from the pre-send timestamp. Expiry preserves uncertain reservation;
late post-expiry replies and restart cannot recreate callback/replay authority.
[Timing audit and writer limits](docs/protocol/peer-identity.md#identity-only-callback-lifetime-2026-11-06).

Outer full-directory and identity-only refusals now retain whole-query error
context in action diagnostics. Nested device codes/text cannot replace a root
refusal or fill an absent/ambiguous root; Error still blocks replay/refresh.
[Callback audit](docs/protocol/peer-identity.md#outer-directory-errors-cannot-inherit-nested-refusal-diagnostics-2026-11-06).

Identity-refresh reply handling now follows APK43.72 outer IQ type and
selected-device errors. Root/foreign errors cannot mask valid signed data;
outer or selected refusal still blocks adoption. Inspection codes use i32.
[Callback evidence and adoption tests](docs/protocol/identity-refresh-application.md#identity-refresh-resulterror-context-2026-11-06).

A new identity notification now requires an owned, correlated prior success
before it can reserve another key read. Wrong kind/request/device/reply cannot
reuse signed data as permission; duplicate events still return the original ID.
[Gate audit](docs/protocol/identity-notification-key-read.md#prior-notification-success-retains-request-ownership-2026-11-06).

Identity-refresh adoption now rechecks original request bytes and action kind
after asynchronous derivation. Inspection, preview and staging validate the
original typed IQ first; changed authority cannot commit a new pin/session.
[Race fix and limits](docs/protocol/identity-refresh-application.md#original-request-authority-across-staging-and-commit-2026-11-06).

Explicit directory refresh now requires matching saved action kind, original
request/peer and correlated reply ID before signed success can authorize work.
Inspection checks the original request too; damaged evidence remains unchanged.
[Local ownership audit](docs/protocol/peer-identity.md#explicit-directory-refresh-retains-request-ownership-2026-11-06).

Full-directory action refusal attribution now follows the original peer.
Foreign user errors cannot poison a valid selected bundle or its explicit
refresh gate. Duplicate result lists were verified already blocked.
[Audit and limits](docs/protocol/peer-identity.md#full-directory-action-errors-belong-to-the-original-peer-2026-11-06).

Identity-only replies now follow pinned APK43.72 outer-type/per-user error
context. Root extensions cannot override the selected identity or refusal.
Inspection retains signed i32 server codes; no identity adoption or replay.
[Trace and limits](docs/protocol/peer-identity.md#identity-only-reply-context-and-signed-codes-2026-11-06).

Peer directory diagnostics now classify saved reply context independently of
generic action status. Typed peer-key results ignore root extension errors;
selected-user refusals and signature verdicts remain distinct. Historical
Error records stay unchanged and do not gain refresh authority.
[Audit](docs/protocol/peer-identity.md#peer-directory-inspection-uses-reply-context-2026-11-06).

Directory-key adoption now follows APK outer IQ type: a stray root error does
not override a valid result list. Per-user refusal and crypto guards remain.
Diagnostic mixed-result attribution is a separate open gap.
[Evidence](docs/protocol/device-key-batches.md#directory-result-and-error-branch-selection-2026-11-06).

Directory-key batches now revalidate saved Answered replies before reserving a
later page or reporting completion. Corrupt evidence refuses without changing
outcomes; summary flags remain diagnostic rather than cryptographic authority.
[Audit](docs/protocol/device-key-batches.md#saved-answered-pages-cannot-authorize-continuation-by-status-alone-2026-11-06).

Device-query writer/ingestion admission now checks exact durable users and
revision bindings against the original callback, not just uncertain status.
Changed or missing ownership refuses without encoding/application or repair.
[Audit](docs/protocol/device-query-deadlines.md#durable-userrevision-ownership-2026-11-06).

Device queries now enforce their original deadline in direct/cursor ingestion
and standalone/group writer handoffs. Expiry preserves pending/query state;
group preparation blocks and receive handles late replies without hiding faults.
[Audit and regression scope](docs/protocol/device-query-deadlines.md).

Prekey upload expiry now fences both writer dequeue and direct result/error
settlement, using the original monotonic deadline with an exact >= boundary.
Expired claims preserve uncertainty without encoding, release or replay.
[Audit](docs/protocol/prekey-count-notifications.md#upload-deadline-fences-writer-and-direct-settlement-admission-2026-11-06).

Queued prekey publication now requires live original legacy/KEM material and
legacy per-key ownership at writer dequeue. Missing, consumed or substituted
keys refuse before encoding; valid consume-before-ACK settlement remains intact.
[Audit](docs/protocol/prekey-count-notifications.md#queued-publication-requires-live-original-key-material-2026-11-06).

Queued prekey uploads now recheck reserved account/batch authority at writer
dequeue, before encoding or socket write. Revocation or changed bindings refuse
without releasing the batch; changes after the snapshot remain a stated limit.
[Audit](docs/protocol/prekey-count-notifications.md#queued-prekey-uploads-recheck-ownership-at-writer-dequeue-2026-11-05).

Native hold configuration now has traced descriptor and selected registry inputs:
gatefalse, delay60000 when enabled, and stored delay0 when disabled. These are
constructor/default facts, not effective runtime values or Rust timing changes.
[Evidence and limits](docs/protocol/apk/2.26.43.72-count-hold-config/README.md).

Native unordered hold/release now has traced ordered-pending checks, arm and
change counters, conditional completion release and a timer-driven one-batch
bypass. Effective gates/delay, failure recovery and wire ordering remain open.
[Evidence](docs/protocol/apk/2.26.43.72-count-hold-release/README.md).

Native count queue provider0xd04 now resolves to a concrete1iz construction,
confirming the selected inherited completion/configuration branch. Hold starts
false; later hold state and effective configuration remain unproven.
[Evidence](docs/protocol/apk/2.26.43.72-count-consumer-binding/README.md).

Native count completion now has traced key-based cache removal, non-QUEUED
count adjustment and conditional consumer rescheduling. These remain separate
from disk retirement and server confirmation; effective DI/config is still open.
[Evidence](docs/protocol/apk/2.26.43.72-count-completion-rescheduling/README.md).

Upload reply settlement now rechecks the originally reserved account authority
inside the settlement transaction. Changed/missing/revoked authority refuses
without altering key or batch status; authentication-time updates still work.
[Audit and remaining limits](docs/protocol/prekey-count-notifications.md#upload-settlement-retains-publication-account-authority-2026-11-05).

PQ digest inspection now shares publication's validated KEM inventory for gated
last-resort selection. Corrupt unselected records refuse instead of yielding a
normal Matches result. Read-only comparison remains local evidence.
[Audit](docs/protocol/prekey-count-notifications.md#pq-digest-rejects-corrupt-unselected-kem-rows-2026-11-05).

Legacy inventory now validates every owned record before status filtering or
replenishment. Corrupt uploaded/unselected rows and restored-consumption conflicts
refuse inventory-driven writes without changing key/checkpoint state.
[Audit](docs/protocol/prekey-count-notifications.md#legacy-inventory-validates-uploaded-and-unselected-rows-2026-11-05).

Legacy upload ACKs now bind original public material through durable reservation
fingerprints and atomic direct/group consumption markers. Replaced/missing keys
and restored-marker conflicts refuse; legitimate consume-before-ACK remains.
[Fix and limits](docs/protocol/legacy-prekey-publication-binding.md#implemented-public-binding-and-atomic-consumption-2026-11-05).

KEM upload settlement now binds accepted replies to reserved public-key
fingerprints, including consumed-before-ACK markers. Same IDs with replacement
keys or changed durable bindings cannot mark material uploaded. Historical
batches remain held/inspectable without invented fingerprints or replay.
[Audit](docs/protocol/kem-prekey-inventory.md#kem-upload-settlement-is-bound-to-reserved-public-material-2026-11-05).

Live KEM rows now conflict with durable consumption markers: lookup, batch
selection, repeated consumption and present-row ACK settlement refuse a stale
restored row. Legitimate missing-row late ACK still settles without restoration.
[Audit](docs/protocol/kem-prekey-inventory.md#restored-kem-rows-cannot-override-consumption-markers-2026-11-05).

KEM write-side batch scans now refuse malformed, aliased, key/ID-mismatched
or invalid owned records before migration/SET/ADD decisions. Existing strict
point validation is reused; diagnostic counts remain diagnostic.
[Audit](docs/protocol/kem-prekey-inventory.md#kem-batch-scans-refuse-corrupt-owned-inventory-2026-11-05).

Direct prekey upload API now validates the supplied signed private/public
pair before IQ ID allocation or reservation, closing the path around factory
validation. A valid public signature no longer admits a mismatched private half.
[Local audit](docs/protocol/prekey-count-notifications.md#direct-upload-api-validates-the-supplied-signed-private-half-2026-11-05).

One-time prekey lookup now rejects mismatched public/private halves before
upload reservation or local digest use. A corrupted last key in a50-key batch
refuses the entire transaction without changing earlier metadata or key rows,
including after reopen. [Audit](docs/protocol/prekey-count-notifications.md#legacy-one-time-prekey-privatepublic-consistency-2026-11-05).

Owned prekey inventory scans now reject malformed/noncanonical ID keys rather
than ignoring them or merging numeric aliases. Inventory, generation and upload
admission refuse corruption without repairing records, including after reopen.
[Local audit](docs/protocol/prekey-count-notifications.md#owned-prekey-inventory-keys-cannot-be-ignored-or-alias-an-id-2026-11-05).

Native0aq/0ar sessions now share a traced writable-handle/listener and1hk
transaction chain. Stable-provider/open-handle assumptions establish normal
nested identity; exceptional close and crash/disk outcomes remain open.
[Evidence](docs/protocol/apk/2.26.43.72-count-database-wrappers/README.md).
Offline research; Rust backend/queue behavior unchanged.

Queue transaction and stanza-retirement suppliers now share a traced
provider-ID/global-cache/memoized-owner chain. Stable-lifetime helper identity
is established conditionally;0aq/0ar connection and crash semantics remain open.
[Evidence](docs/protocol/apk/2.26.43.72-count-database-provider/README.md).
Offline research; Rust backend/queue behavior unchanged.

Native failed completion now has a retained disk/cache retirement trace:
transaction work deletes explicit stanza row IDs; Feh(46) separately removes
cache entries. The no-work callback path is not disk-commit proof.
[Evidence and recovery limits](docs/protocol/apk/2.26.43.72-count-completion-retirement/README.md).
Offline research; Rust deletion/replay behavior unchanged.

Native escaped-handler errors now have a retained NACK/completion trace:
487 for parser LIL,500 for other selected failures, separate failed queue
completion with ACK-list flagfalse. SQLite full/corruption still rethrow;
count parser errors caught internally retain their normal ACK path.
[Evidence and limits](docs/protocol/apk/2.26.43.72-count-handler-error-completion/README.md).
Offline research; Rust error/queue behavior unchanged.

Tracked queue processing now has a retained selection-to-handler trace:
new memory entries can satisfy the state predicate without a database row ID.
This is not disk-commit proof; ACK durability remains a separate boundary.
[Evidence and unresolved gates](docs/protocol/apk/2.26.43.72-count-queue-processing/README.md).
Static/offline research; Rust processing/ACK behavior unchanged.

Tracked count ingress now connects to native1ix queue admission and its
Flr disk-write scheduler. Admission is initially in memory; A0D=false requires
a backend and1ix disables the generic inline-state promotion.
[Evidence and remaining processing gate](docs/protocol/apk/2.26.43.72-count-queue-admission/README.md).
Static/offline research; Rust queue/ACK behavior unchanged.

Count-notification ingress now has an evidence-backed conditional routing
map: unmatched-provider fallback can create tracked encrypt wrappers under
flag3992; its false branch reaches direct handling. Effective configuration
and persistence ownership remain open. [Audit](docs/protocol/apk/2.26.43.72-count-ingress-routing/README.md).
Offline research; Rust routing unchanged.

Registration request construction and repeated preparation now reject
inconsistent saved Noise/identity/signed key pairs, including private-half
corruption in the separate account record. Five substitution cases preserve
public keys/signature and verify refusal without repair across reopen.
[Audit](docs/protocol/registration-safety-audit.md#registration-material-key-pair-integrity-before-request-construction-2026-11-05).

Registered signed-prekey lookup now rejects public/private pair mismatches,
even when the saved public key still has a valid identity signature. The actor
surfaces the error before upload generation/reservation/write and preserves
the saved material. [Local audit and limits](docs/protocol/prekey-count-notifications.md#signed-prekey-publicprivate-consistency-before-upload-2026-11-05).

Prekey upload lookup now distinguishes absent registration material from
corrupt/mismatched saved state: absence skips, stored-state errors propagate
before upload generation/reservation/write. Existing required lookup semantics
remain. [Audit and limits](docs/protocol/prekey-count-notifications.md#registered-signed-prekey-absence-versus-stored-state-faults-2026-11-05).

Confirmed native deferred-ACK watchdog behavior: its shared six-second timer
logs and clears tracking maps without sending ACKs, canceling workers or
consuming their held metadata tokens. Late completion can still dispatch.
[Evidence and port limits](docs/protocol/apk/2.26.43.72-deferred-ack-watchdog/README.md).
Offline research; Rust scheduling unchanged.

Malformed PQ count content now receives its normal control ACK after capture,
without upload work, following current native parser/base-handler catch flow.
Invalid ACK envelopes remain refused; storage failures propagate.
[Evidence and limits](docs/protocol/apk/2.26.43.72-count-content-error-ack/README.md).

Confirmed another native ACK persistence boundary: Flr selector7 can persist
selected stanzas and pre-ACKs before ACK send admission, propagating outer-close
failures before send. The alternate branch and selection limits are retained in
[the audit](docs/protocol/apk/2.26.43.72-preack-transactional-dispatch/README.md).
Count-specific branch selection and Rust queue parity remain open; offline research only.

Count ACKs no longer invent a server destination: missing from omits to;
explicit parse failure is refused before ACK/upload. Actor regression covers
source-less PQ and malformed source; native ACK timing remains unported.
[Evidence](docs/protocol/apk/2.26.43.72-prekey-count-ack-source/README.md).

Account privacy now filters named category children like the APK, allowing
unrelated metadata beside supported readreceipts. Actual unsupported categories
and metadata-only events remain refused; regression preserves prior state.
[Evidence](docs/protocol/account-privacy-notifications.md#named-category-filtering-2026-11-05).

Account privacy now selects the first supported event with later siblings,
matching current APK dispatch. Later events cannot override or rescue invalid
first data; existing category/transaction/pending guards remain.
[Evidence and regression](docs/protocol/account-privacy-notifications.md#first-event-selection-with-later-siblings-2026-11-05).

Fixed unused sender_lid validation suppressing privacy-token ACKs for empty
or ignored-only lists. No supported data now means no token-target resolution
or write transaction. Supported-data alias/primary/time checks remain.
[Evidence](docs/protocol/apk/2.26.43.72-privacy-token-primary-address/README.md).

Fixed unused parent timestamps suppressing privacy-token ACKs for empty or
ignored-only lists. Supported entries still require valid parent time and retain
all existing data guards. Red/green regression expanded.
[Evidence](docs/protocol/apk/2.26.43.72-privacy-token-primary-address/README.md).

Fixed extra privacy-token containers suppressing valid first-container data
and ACK. First named selection now matches current APK; later containers cannot
replace state or rescue malformed selected data. Red/green regression expanded.
[Evidence](docs/protocol/apk/2.26.43.72-privacy-token-primary-address/README.md).

Privacy-token primary filters now exclude nonzero legacy agent qualifiers
before token storage/ACK. Existing PN/LID primaries remain supported; explicit
alias errors preserve prior state. Red/green/reopen regression expanded.
[Evidence](docs/protocol/apk/2.26.43.72-privacy-token-primary-address/README.md).

Fixed unrelated children inside groups_dirty suppressing valid refresh/ACK.
Only named group entries are parsed; existing selected-entry and total-list
guards remain. Queue/dedup/reopen regression expanded.
[Evidence](docs/protocol/apk/2.26.43.72-group-event-selection/README.md).

Fixed groups_dirty plus metadata siblings being ACKed without refresh work.
Dedicated first-event dirty handling now retains scheduling before ACK; generic
ACK stays suppressed. Queue/dedup/reopen regression expanded.
[Evidence](docs/protocol/apk/2.26.43.72-group-event-selection/README.md).

Fixed group notifications applying later sibling events to membership.
Only the first event now selects the supported change; metadata siblings stay
permitted. Current APK selection evidence and red/green regression retained.
[Details](docs/protocol/apk/2.26.43.72-group-event-selection/README.md).

Public group-change API now shares the receive envelope prerequisite before
store access. Invalid-envelope regression preserves snapshot bytes and journal;
valid routing-race coverage remains. No live validation.
[Details](docs/protocol/groups.md#public-group-change-admission-matches-receive-2026-11-05).

Fixed invalid group ACK envelopes mutating membership before refusal.
Envelope validation now precedes the update; commit still precedes ACK write.
Actor regression includes three refusals and a valid/reopen control.
[Details](docs/protocol/notification-ack-audit.md).

Fixed group notification storage failures being logged then ACKed. The actor
now surfaces the fault before ACK; regression preserves corrupt snapshot bytes
and durable capture across reopen. No live validation.
[Details](docs/protocol/notification-ack-audit.md).

Saved role-ID reference census retained: pairing device-JID value writes,
zero resets and selected-account readers are distinguished. Existing role/Me
audits linked; full Rust migration/account authority remains unported.
[Evidence](docs/protocol/apk/2.26.43.72-notification-handler-census/README.md).

Server-sync Rust state mapping audited: existing login/admission/backoff
authorities do not supply native bootstrap/recovery/ready-mutation gates.
[Concrete mapping and remaining integration](docs/protocol/server-sync-rust-state-mapping.md).

Server-sync completion callback identified as metrics/counter cleanup, not
an ACK gate. Census reconciled with existing dirty-version/scheduling evidence;
Rust lifecycle state mapping remains unported.
[Evidence](docs/protocol/apk/2.26.43.72-notification-handler-census/README.md).

Current notification census retains34 base-handler constructor bindings.
Selected server_sync/server handlers expose collection/bootstrap/configuration
obligations still absent from Rust; no generic ACK inferred.
[Evidence](docs/protocol/apk/2.26.43.72-notification-handler-census/README.md).

Current APK binary reader/writer confirms selected zero-device ACK pair
normalization. Picture actor regression expanded to zero/nonzero PN/LID sender
and participant forms; no runtime policy change.
[Evidence](docs/protocol/apk/2.26.43.72-group-missing-identification/README.md).

ACK address audit: current APK typed getter logs protocol-compliance failures
but continues; no new Rust rejection inferred. Attribute helper preserves Jid
objects; current binary reader/writer subtype parity remains unclosed.
[Evidence](docs/protocol/apk/2.26.43.72-group-missing-identification/README.md).

Picture ACK final builder and fresh notification-key fields now traced in
current APK; regression rejects accidental copying of incoming recipient/edit/
error/offline attributes. No runtime change or live validation.
[Evidence](docs/protocol/apk/2.26.43.72-group-missing-identification/README.md).

Picture notifications now receive the shared base ACK after durable capture.
Actor regression covers redelivery, participant routing, malformed envelope
continuation and reopen. Photo state/missing-ID lifecycle remain unported;
no live validation. [Evidence](docs/protocol/apk/2.26.43.72-group-missing-identification/README.md).

Current APK mapping confirmed: w:gp2 ->209, picture ->189. Rust picture
consumer/ACK and automatic missing-identification lifecycle remain unported;
native admission/deferral must be reconciled first.
[Evidence and limits](docs/protocol/apk/2.26.43.72-group-missing-identification/README.md).

Research follow-up: current missing-identification handlers now traced through
numeric registration and shared handler filtering; wire-type admission remains
unclosed. No Rust runtime change or live operation. See the
[retained boundary](docs/protocol/apk/2.26.43.72-group-missing-identification/README.md).

2026-11-05 — **Missing participant-identification context lifecycle connected.**

The current APK persists a group set: selected marker presence adds, and group
info's mapped missing-identification boolean later adds/removes. Rust's explicit
MEX context vocabulary already exists; automatic lifecycle/admission remains
unported and no partial ingress policy was guessed.
[Source, register proof and limits](docs/protocol/apk/2.26.43.72-group-missing-identification/README.md).
Validation: fmt, strict Clippy, CLI reference, OpenAPI, APK coverage, pinned
APK identity and staged checksum/link hygiene passed. No new workspace test
run for this research-only chunk. No runtime or live account operation changed.

2026-11-05 — **Current group-info MEX writer and selected timeout connected.**

The current helper builds QueryGroupInfo GraphQL input, not Rust's classic
query shape. Its selected non-Pando MEX path carries the operation's32000ms
default; the helper's30-second calculation is discarded. Classic wire/timer
parity remains unproven and no speculative policy was ported.
[Writer, dispatch, physical writer census and limits](docs/protocol/apk/2.26.43.72-group-info-mex-timeout/README.md).
Validation: fmt, strict Clippy, CLI reference, OpenAPI, APK coverage, pinned
APK identity and staged checksum/link hygiene passed. No new workspace test
run for this research-only chunk. No runtime or live account operation changed.

2026-11-05 — **Media refresh route and throttle preservation verified together.**

Six malformed results retain installed route bytes and the saved 507 wait;
duplicate/conflicting receipts and reopening do not bypass either. A valid
result control still applies routing and the existing reset transition.
[Regression, evidence and history bounds](docs/protocol/media-network-outcomes.md#malformed-refresh-replies-preserve-route-and-wait-together-2026-11-05).
Validation: combined coordinator regression, all 1388 workspace/all-feature
tests, fmt, strict Clippy, CLI/OpenAPI, APK coverage, pinned APK identity and
staged checksum/link hygiene passed. Regression/documentation only; no runtime
policy or live account operation.

2026-11-05 — **Unusable group-info identity no longer consumes pending projection.**

Missing/nontext group ID now leaves queued group-info unapplied instead of
marking a no-op successful and deleting its pending marker. Prior snapshots,
action replies, checkpoints and journal remain preserved across recovery.
[Regression and scope](docs/protocol/normal-group-outbox.md#unusable-group-info-identity-preserves-pending-projection-2026-11-05).
Validation: expanded recovery regression failed before the fix and passes
afterward. All 1387 workspace/all-feature tests, fmt, strict Clippy, CLI/OpenAPI,
APK coverage, pinned APK identity and staged checksum/link hygiene passed.
No live account operation or replay authority.

2026-11-05 — **Ambiguous group-info replies stay unapplied across recovery.**

Shared projection now rejects multiple outer group children and missing queued
group-info data, closing a first-child fallback after normal send validation.
Invalid recovery preserves saved replies, markers, prior snapshots and journal;
a valid single correlated group still applies once.
[Defect, red/green test and scope](docs/protocol/normal-group-outbox.md#unique-group-info-projection-across-recovery-2026-11-05).
Validation: recovery/direct projection regressions failed before the fix and
pass afterward. All 1387 workspace/all-feature tests, fmt, strict Clippy,
CLI/OpenAPI, APK coverage, pinned APK identity and staged checksum/link hygiene
passed. No live account operation or query replay.

2026-11-05 — **Invalid-directory resume refusal verified through the actual API.**

After reopen, all seven bad-reply/signature cases return no stanza under both
identity policies and preserve every saved table/journal head. Suspension CLI
wording now covers directory validation failures rather than only identity/PQ.
[Regression and scope](docs/protocol/messaging-transport.md#explicit-resume-refusal-for-invalid-directory-suspensions-2026-11-05).
Validation: strengthened actor/API regression, all 1386 workspace/all-feature
tests, fmt, strict Clippy, regenerated CLI reference, OpenAPI, APK coverage,
pinned APK identity and staged checksum/link hygiene passed. No runtime
resume-policy change or live account operation.

2026-11-05 — **Malformed direct key replies suspend one intent, preserving receive.**

Pure directory parser errors now produce non-resumable invalid_directory_reply;
a new classic session's invalid signature uses existing invalid_signed_bundle.
The reply stays encrypted, no keys/send are adopted, and store faults remain
visible rather than being swallowed as bad wire input.
[Actor red/green cases and limits](docs/protocol/messaging-transport.md#malformed-direct-directory-replies-stay-intent-local-2026-11-05).
Validation: red/green actor regression covers seven wire cases and a stored
corruption control. All 1386 workspace/all-feature tests, fmt, strict Clippy,
CLI/OpenAPI, APK coverage, pinned APK identity and staged checksum/link hygiene
passed. Existing foreign-device fixture now checks the intent-local refusal
contract. No live account operation.

2026-11-05 — **Direct cancellation reply behavior verified through the actor.**

Matching result/error replies and redelivery after cancellation already stop
before directory parsing. New actor regression verifies no key adoption or
follow-up send, continued ping service and byte-preserved reservations on reopen.
[Evidence and diagnostic boundary](docs/protocol/messaging-transport.md#direct-send-cancellation-reply-audit-2026-11-05).
Validation: targeted actor regression, all 1385 workspace/all-feature tests,
fmt, strict Clippy, CLI/OpenAPI, APK coverage, pinned artifact identity and
staged checksum/link hygiene passed. Regression/documentation only; no
production behavior or live account operation.

2026-11-05 — **Cancelled group preparation settles its reply and stops cleanly.**

Cancellation during membership/device/key waits no longer progresses to another
query or ends the actor through the unchanged-intent guard. The owned reply is
settled, intent_cancelled is visible, and cancelled work stays nonselectable.
[Actor regression and cancellation limits](docs/protocol/normal-group-outbox.md#cancellation-while-a-preparation-query-is-pending-2026-11-05).
Validation: actor regression failed before the fix and passes at all three
cancellation stages afterward. All 1384 workspace/all-feature tests, fmt,
strict Clippy, CLI/OpenAPI, APK coverage, pinned APK identity and staged
checksum/link hygiene passed. No live account operation.

2026-11-05 — **Fresh recipient device-sync work stops stale group continuation.**

Recipient sync revisions now guard fanout reservation and later ACK-driven
stanzas, closing the interval before a new device list arrives. An unrelated
contact refresh still permits continuation. Original pending work, ciphertext
and uncertain reservations remain held.
[Defect, red/green test and scope](docs/protocol/normal-group-outbox.md#device-refresh-revisions-across-preparation-and-ack-waits-2026-11-05).
Validation: the new regression failed before the guard and passes afterward;
all 1383 workspace/all-feature tests, fmt, strict Clippy, CLI/OpenAPI, APK
coverage, pinned APK identity and staged checksum/link hygiene passed. No live
account operation.

2026-11-05 — **Invalidated sender-key chains stop ACK-driven continuation.**

The reservation captures chain identity; later distribution ACKs cannot release
old ciphertext after rotation invalidation or a replacement chain. Same-chain
ACK bookkeeping remains permitted and original uncertainty stays held.
[Defect, regressions and limits](docs/protocol/normal-group-outbox.md#sender-key-validity-across-distribution-ack-waits-2026-11-05).
Validation: the invalidation regression failed before the guard; four targeted
continuation tests and all 1382 workspace/all-feature tests passed afterward.
Fmt, strict Clippy, CLI/OpenAPI, APK coverage, pinned APK identity and staged
checksum/link hygiene passed. No live account operation.

2026-11-05 — **Changed routing authority stops distribution continuation.**

Post-reservation device-list, primary/device pin and own identity authority is
rechecked before subsequent distributions or final group ciphertext. The fixed
routing_changed diagnostic preserves all original reservations and uncertainty.
[Defect, red/green cases and scope](docs/protocol/normal-group-outbox.md#authority-guard-across-distribution-ack-waits-2026-11-05).
Validation: new regression failed before the guard; both continuation tests
pass afterward. All 1380 workspace/all-feature tests, fmt, strict Clippy,
CLI/OpenAPI, APK coverage, pinned APK identity and staged checksum/link hygiene
passed. No live account operation.

2026-11-05 — **Membership changes stop ACK-driven distribution continuation.**

The existing frozen-snapshot guard now covers every distribution continuation,
including the final group stanza. Changes observed while waiting for an ACK
persist membership_changed without releasing more ciphertext, resetting keys
or replaying the reserved intent.
[Defect, red/green regression and native/local boundary](docs/protocol/normal-group-outbox.md#membership-guard-across-distribution-ack-waits-2026-11-05).
Validation: the regression failed before the guard and passes afterward; all
1379 workspace/all-feature tests, fmt, strict Clippy, CLI/OpenAPI, APK coverage,
pinned artifact identity and staged checksum/link hygiene passed. No live
account operation.

2026-11-05 — **Sender-key prerequisite selection separated from delivery ACKs.**

Native selected hash mismatch retains original session candidates; inherited
session/PQ checks do not establish a separate distribution ACK producer.
[Retained methods, branch limits and remaining producer gap](docs/protocol/apk/2.26.43.72-sender-key-session-requirement/README.md).
Validation: fmt, strict Clippy, CLI reference, OpenAPI, APK coverage, pinned
APK identity and staged checksum/link hygiene passed. No new workspace test
run for this research-only chunk. No Rust source or live account operation
changed.

2026-11-05 — **Selected ordinary message ACK forwarding connected.**

Success/refusal event branches return to original-key callback lookup. The
investigated destination helper is FBID-migration-specific; the exact native
out-of-band sender-key producer remains unconfirmed, so no speculative address
or type guard was added.
[Evidence and limits](docs/protocol/apk/2.26.43.72-message-ack-forwarding/README.md).
Validation: fmt, strict Clippy, CLI reference, OpenAPI, APK coverage, pinned
artifact identity and staged checksum/link hygiene passed. No new workspace
test run for this research-only chunk; previous 1378-test result is unchanged
code evidence. No runtime change or live account operation.

2026-11-05 — **Non-reply IQs cannot consume reusable key-batch ownership.**

The settlement helper now rejects missing/get/set/unsupported/nontext type before
store mutation. Matching ID or error child alone cannot terminally hold a page
and prevent the later valid response. The normal group driver already filtered
these types. No new replay or callback authority is granted.
[Red/green durable regression and APK dispatch evidence](docs/protocol/device-key-batches.md#key-batch-reply-admission-correction-2026-11-05).
Validation: red/green regression, all 1378 workspace/all-feature tests, seven
focused key-batch tests, fmt, strict Clippy, CLI/OpenAPI, coverage, pinned APK
identity and staged checksum/link hygiene passed. No live account operation.

2026-11-05 — **Key-fetch wire error attribution and signed inspection corrected.**

Current writer binds Dxh to original devices; its error event matches backoff
selector 76. Directory inspection now preserves signed Java integer codes rather
than dropping negative/large/Unicode values. Unique-context, no-refresh and
no-replay guards remain; the public Rust field widens from u16 to i32.
[Evidence, API consequence and durable regression](docs/protocol/apk/2.26.43.72-prekey-wire-error-attribution/README.md).
Validation: all 1377 workspace/all-feature tests passed, including 26 root/user
JVM-case inspections, reopen/no-refresh/no-resend checks. Fmt, strict Clippy,
CLI/OpenAPI, coverage, pinned APK identity and staged checksum/link hygiene passed.
No live account operation.

2026-11-05 — **Current prekey error backoff re-anchored and scoped.**

Non-PQ 5xx moves affected in-flight requests to pending; mode and posted flags
have distinct lifetimes. The native local delay sequence starts at zero, and
normal admission/reset does not prove a blanket server cooldown for all queries.
[Source, derived delays and unported ingress/recovery](docs/protocol/apk/2.26.43.72-prekey-error-backoff/README.md).
Research only; no runtime or live account operation changed.
Validation: pinned APK identity, fmt, strict Clippy, CLI/OpenAPI, APK coverage and
staged checksum/link hygiene passed. No new full workspace run; shared Rust
remains unchanged from the preceding 1376-test validated commit.

2026-11-05 — **About refusals retain native signed and ordered codes.**

Optional about_refusal metadata preserves direct-child Java integer codes and
malformed-input boundary, atomically with settlement. Existing actions plain/JSON
shows it. Generic compatibility fields and runtime retry policy are unchanged.
[Evidence, executed JVM oracle and durable regression](docs/protocol/apk/2.26.43.72-about-error-codes/README.md).
Validation: all 1376 workspace/all-feature tests, executed 13-case JVM oracle,
fmt, strict Clippy, CLI/OpenAPI, coverage and staged checksum/link hygiene passed.
No live account request.

2026-11-05 — **Status-privacy retry eligibility and error nesting traced.**

The wrapper's root IQ code/backoff lookup differs from its original callback's
nested error parser. A 404 does not adopt a fallback audience; success requires
privacy child/type before conditional persistence. Rust fetch/adoption is absent.
[Evidence, branch table and unported boundaries](docs/protocol/apk/2.26.43.72-status-privacy-fetch-callback/README.md).
Research only; no runtime or live-account operation changed.
Validation: pinned APK identity, fmt, strict Clippy, CLI/OpenAPI, coverage and
staged checksum/link hygiene passed. No new full workspace run; shared Rust
remains unchanged from the preceding 1374-test validated commit.

2026-11-05 — **Disconnect cancellation-map owner and bounded IQ retry path traced.**

The adjacent map holds delayed retriable-IQ jobs, not queued callback ownership.
Retry requires shared budget, remaining window and callback permission; the
application caller found is a feature-gated status-privacy fetch. Cancellation
forwards delivery failure. No generic retry/replay policy was added.
[Source, exact callers and unresolved durable/cancellation boundaries](docs/protocol/apk/2.26.43.72-iq-retry-job-cancellation/README.md).
Research only; no runtime or live-account behavior change.
Validation: pinned APK identity, fmt, strict Clippy, CLI/OpenAPI, APK coverage,
checksum/link hygiene and staged diff checks passed. No new full workspace run;
shared Rust is unchanged from the preceding 1374-test validated commit.

2026-11-05 — **Ping registry reset scope and stale-link ownership verified.**

The native reset clears awaiting-response callbacks asynchronously; it does not
prove cancellation of already-claimed workers. 1xZ.BVv's dispatch role is traced.
A new regression verifies stale result/error cannot clear or extend a restarted
Rust link's ping after store reopen. Production policy remains unchanged.
[Source, regression and unresolved queue/ordering boundaries](docs/protocol/apk/2.26.43.72-client-ping-registry-lifecycle/README.md).
Validation: all 1374 workspace/all-feature tests passed, including both stale-reply
cases; removing ID ownership makes the new regression fail. Guard restored.
Pinned APK identity, fmt, strict Clippy, CLI/OpenAPI, coverage and staged hygiene passed.

2026-11-05 — **Ping producer and durable-ACK completion authority separated.**

Normal on-demand pings use the existing pending guard; direct confirmation pings
have success-only retirement continuations. Bound offline-resume/coroutine paths
must not retire markers on IQ error or delivery failure. Arbitrary calling callbacks
remain separate from normal keepalive behavior.
[Source, caller census and unresolved replacement/cancellation boundaries](docs/protocol/apk/2.26.43.72-client-ping-producer-families/README.md).
Research only; no runtime policy or live account operation changed.
Validation: pinned APK hash, fresh callback export equality, fmt, strict Clippy,
CLI/OpenAPI, coverage and staged checksum/link hygiene passed. No new full
workspace run for research only; shared Rust retains its preceding 1373-test result.

2026-11-05 — **Ping channel startup and scheduler-mode authority traced.**

A new accepted channel resets the ping counter; duplicate same-channel callback
leaves it alone. Push-registration preferences and login count determine the
native short-mode input, distinct from effective interval values and cache lifetime.
[Transition table, field census and remaining gaps](docs/protocol/apk/2.26.43.72-client-ping-channel-lifecycle/README.md).
Research only; runtime timing and live-account behavior unchanged.
Validation: pinned APK hash and exact prior-export comparisons, fmt, strict Clippy,
CLI/OpenAPI, APK coverage and staged hygiene passed. No new full workspace test
run for this research-only chunk; shared Rust retains its preceding 1373-test result.

2026-11-05 — **Final compiled ping defaults verified across the complete integer builder.**

The three selected timing keys occur once each and survive into the final map:
32s, 240s and 15s. This closes later-default-write uncertainty; it does not establish
effective account settings. Runtime timing remains unchanged.
[Proof, model guards and remaining configuration gaps](docs/protocol/apk/2.26.43.72-client-ping-final-defaults/README.md).
Validation: fresh pinned-APK proof reproduction, four rejecting model guards,
fmt, strict Clippy, CLI/OpenAPI, coverage and staged hygiene passed.
Shared Rust is unchanged from the preceding 1373-test validated commit.

2026-11-05 — **Client timing first-access cache and reset scope traced.**

Native ping getters retain selected first-access values in a distinct cache;
SDK readiness reset does not directly clear it. Conditional SDK comparison and
direct selection differ, so raw AB rows/default entries cannot identify the
actual timer. Effective configuration and owner lifetime remain open.
[Evidence and concrete port requirements](docs/protocol/apk/2.26.43.72-client-ping-first-access/README.md).
Research only: no runtime timing/configuration or live account operation changed.
Validation: pinned APK hash, exact cache-method re-exports, checksum/link hygiene,
APK coverage, CLI/OpenAPI, fmt and strict Clippy passed. Shared Rust is unchanged
from the preceding 1373-test validated commit.

2026-11-05 — **Next client ping waits a full interval after its response.**

A delayed reply no longer allows an almost-immediate next ping. Independent
maintenance still drains queued work on its existing cadence; malformed and
unowned replies do not restart ping scheduling. Red/green regression covers
both result/error, preserved uncertainty and exact paused-time delay.
[Source, packaged timing entries and unported effective configuration](docs/protocol/apk/2.26.43.72-client-ping-scheduling/README.md).
No duration/configuration, request shape, retry/reset or live account operation added.
Validation:1373 workspace/all-feature tests, strict Clippy, fmt, CLI/OpenAPI,
APK coverage, pinned APK hash, exact specifier derivation and checksum/link
hygiene passed. The delayed-response regression failed against the prior driver.

2026-11-05 — **Malformed client pong results cannot clear liveness timers.**

The driver validates required supported sender JID, original ID and native
inclusive timestamp bounds. Schema failure consumes its callback claim but
retains the original timer, so later same-ID replies cannot resurrect it.
[Source, exception lifecycle, regression and remaining parity limits](docs/protocol/apk/2.26.43.72-client-pong-schema/README.md).
No request/cadence/timeout, clock, retry/reset or live account behavior added.
Validation:1372 workspace/all-feature tests, strict Clippy, fmt, CLI/OpenAPI,
APK coverage, pinned APK hash and checksum/link hygiene passed. Actor regression
failed against parent code; intended success fixtures now include native fields.

2026-11-05 — **Matched keepalive errors preserve the connection.**

A red regression reproduced the prior receive termination. Native error and
success callbacks share the manager response path; Rust now clears the matching
live ping without treating its error as a connection refusal. Duplicate errors
cannot consume a successor; unanswered successors still time out.
[Evidence, tests and open cadence/result-schema gaps](docs/protocol/apk/2.26.43.72-client-ping-errors/README.md).
No request cadence/timeout, account policy, automatic retry/reset or live traffic change.
Validation:1370 workspace/all-feature tests, strict Clippy, fmt, CLI/OpenAPI,
APK coverage and checksum/link hygiene passed. New regression failed before fix.

2026-11-05 — **Contact backoff clock reanchored; NTP scheduler traced.**

Pinned43.72 evidence separates NTP-preferred contact time from token time,
12-hour periodic scheduling, six-hour flex/recent-success guards and one-day
restart cache admission. Both selected scheduling branches also attempt sync
immediately. Effective flags and full runtime clock/backoff port remain open.
[Evidence and remaining implementation requirements](docs/protocol/apk/2.26.43.72-contact-backoff-clock/README.md).
Research only: no request, clock, timer, retry/reset or live account state changed.
Validation: pinned APK hash, checksum/link hygiene, APK coverage, CLI reference,
OpenAPI library contract, formatting and strict Clippy passed. Shared Rust code
is unchanged from the preceding 1369-test validated commit.

2026-11-05 — **Contact-protocol refusals survive outer-IQ success.**

Actions now expose durable contact_protocol_refusal independently of outer-IQ
status and per-user contact projection. Native defaults/parse order, malformed
values, Unicode digits and wrapping arithmetic are checked against a JVM oracle.
[Evidence, tests and remaining policy gaps](docs/protocol/apk/2.26.43.72-contact-protocol-refusal/README.md).
No wait enforcement, automatic retry, request change or live account operation.
Validation:1369 workspace/all-feature tests, strict Clippy, fmt, CLI/OpenAPI,
APK coverage, exact JVM oracle regeneration and checksum/link hygiene passed.

2026-11-05 — **Contact wait gates and reset events traced further.**

The native executor filters protocols separately and uses effective configuration
to select contact/sidelist wait keys. Reset callers are registration sync startup
and a specifically scheduled contact-sync notification worker; the reset list
omits two wait keys read by that executor. These are bounded static findings,
not a completed query/backoff policy port.
[Evidence and remaining implementation requirements](docs/protocol/apk/2.26.43.72-contact-backoff-gates/README.md).
No Rust eligibility, automatic retry, reset command or live account operation added.
Validation: pinned APK hash, checksum/link hygiene, APK coverage, CLI reference,
OpenAPI library contract, formatting and strict Clippy passed. Research only;
shared Rust code is unchanged from the preceding 1366-test validated commit.

2026-11-05 — **Usync outer-error waits have native numeric inspection.**

Absent and malformed backoff now remain distinguishable from explicit signed
values in durable action metadata. Outer-error defaults stay separate from
protocol-result defaults. Independent JVM vectors verify parsing/wrap/cap.
Existing actions plain/JSON exposes the metadata through reopen/projection.
[Evidence, ownership tests and unported wait enforcement](docs/protocol/apk/2.26.43.72-usync-outer-refusal/README.md).
No timeout/backoff enforcement, automatic retry or live account operation added.
Validation:1366 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage, exact JVM oracle regeneration and repository hygiene passed.

2026-11-05 — **Contact factory comparison refutes lightweight parity shortcut.**

Native lightweight phone query omits disappearing_mode; the regular factory
selects it with additional flags. Rust's three-protocol request is still a local
composition. Constructor/field-transfer evidence and selected32s dispatch paths
are now retained together; complete native factory/backoff policy stays open.
[Source and concrete next port requirements](docs/protocol/apk/2.26.43.72-contact-factory-selection/README.md).
Validation: pinned APK hash, checksum/link hygiene, coverage, CLI/OpenAPI,
fmt and strict Clippy passed. Research only; no request, timeout or live
account behavior changed.

2026-11-05 — **Single-contact replies cannot overwrite unrelated contact statuses.**

A red regression reproduced the wrong-target projection. Saved request provenance
now selects one matching user; ambiguous/missing/malformed cases preserve data.
Existing actions inspection shows the fixed projection result. Status, diagnostic
and once-only marker commit together under real disk/fsync fault tests.
[Evidence, recovery and unported native protocol policy](docs/protocol/contact-projection-correlation.md).
Validation:1363 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. Wrong-target regression failed before the fix.
No action replay or live account traffic; broader protocol work remains unfinished.

2026-11-05 — **Contact protocol backoff producer/observer scopes traced.**

Native contact error metadata carries capped millisecond waits separately from
per-user contact type. Regular/lightweight observers persist different scopes;
selected503 responses also set a contact-executor global deadline checked before
dispatch. Rust metadata/wait adoption remains unported.
[Source, observer bindings, clock domain and concrete port requirements](docs/protocol/apk/2.26.43.72-contact-protocol-backoff/README.md).
Validation: pinned APK hash, checksum/link hygiene, coverage, CLI/OpenAPI,
fmt and strict Clippy passed. Research only; no sending/wait policy changes
or live account operations.

2026-11-05 — **Contact checks prepare request and correlation atomically.**

A full-disk regression reproduced the prior sendable-orphan defect. One transaction
now commits both records. Historical queued requests with missing provenance hold
locally; mismatches refuse before reservation. Existing actions shows the hold.
[Fault tests, source and caller-specific usync timeout boundary](docs/protocol/apk/2.26.43.72-contact-queue-atomicity/README.md).
Validation:1360 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. Full-disk regression failed before the fix.
No automatic retry, action reset or live account traffic. Contact factory/deadline
parity and the broader network audit remain unfinished.

2026-11-05 — **Token target/storage namespace rules re-anchored to current APK.**

Wire/pending user and store canonicalization use different rules; counterpart
cleanup has a separate flag/equality guard. Mapping authority remains unported.
Native excluded types26/27 are bots. The existing PN/LID allowlist correctly
excludes them; a new no-mutation regression includes positive direct controls.
[Evidence, conditional table and unresolved mapping policy](docs/protocol/apk/2.26.43.72-token-namespace-selection/README.md).
Validation:1357 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. No sending policy change, automatic alias
deletion or live traffic.

2026-11-05 — **Token results now persist captured success timestamps atomically.**

Older completions preserve newer schedules; matching completions clear real-time
metadata. Scheduling carries/reset metadata using the current APK branch.
Existing actions inspection reports content-free completion effects. Independent
SQL vectors and reorder/reopen/rollback tests cover the port.
[Evidence and unported configured/counterpart lifecycle](docs/protocol/apk/2.26.43.72-token-success-state/README.md).
Validation:1356 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. Independent SQL/decoder vectors reproduce.
No live account operations; the broader protocol audit remains unfinished.

2026-11-05 — **Token MobileConfig lookup and one readiness producer traced.**

All four token IDs have nonzero SDK specifiers. Concrete DI bindings are closed,
and one native initialization branch enables SDK reads and resets their cache.
[Static evidence and reproducible table derivation](docs/protocol/apk/2.26.43.72-token-mobileconfig-binding/README.md).
Effective account settings/parameter defaults and the configured token policy
remain unfinished. Research only; send behavior unchanged and no live traffic.
Validation: pinned APK hash and all 2065 map writes checked; derivation matches
retained JSON. CLI/OpenAPI, fmt, strict Clippy, coverage and hygiene passed.

2026-11-05 — **Saved AB replies expose token bucket integer evidence.**

abprops-query --show includes returned 865/909/996/997 values, preserving missing,
invalid-last, zero/negative and full/delta distinctions across reopen. These do
not select effective policy. Current-APK adapter evidence is linked and reused;
lookup specifiers and ready-state transitions remain open. No live traffic.
Validation:1352 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and repository hygiene passed.
[Inspection contract and limits](docs/protocol/apk/2.26.43.72-token-bucket-configuration/README.md).

2026-11-05 — **Token bucket configuration traced further in the current APK.**

Selected packaged entries are seven-day buckets/four valid buckets; if effective,
expiry is 21 to under 28 days old. The 182-day floor remains necessary only.
Cache/preferences/MobileConfig prevent treating raw AB rows as effective values.
[Evidence and remaining resolver work](docs/protocol/apk/2.26.43.72-token-bucket-configuration/README.md).
Research only; send policy unchanged, no live traffic. Effective policy remains open.

2026-11-05 — **Queued trusted-contact tokens now reload current state at reservation.**

Unwritten actions coalesce; reservations persist refreshed stored timestamps
atomically. Missing/expired/superseded requests stay queued with inspection reasons;
other traffic continues. Sent bytes and IDs remain frozen across reopen.
[Age-floor proof, legacy ownership, tests and remaining configured-policy gap](docs/protocol/apk/2.26.43.72-token-reservation-refresh/README.md).
Validation:1351 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. No live account operation; full configured
expiry/job policy remains unfinished.

2026-11-05 — **Trusted-contact token callbacks now use their native32s timer.**

Expiry preserves uncertain actions and scheduled markers without retry; typed
results follow captured-data success. Account-privacy setters remain separate.
[Evidence/tests and newly confirmed send-time validation gap](docs/protocol/apk/2.26.43.72-privacy-token-callback/README.md).
Validation:1345 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. Queued token-state reload/expiry/coalescing
remains unfinished; no live traffic.

2026-11-05 — **Two-step verification callbacks now use native32s expiry.**

Typed outer results preserve the native payload-ignoring success semantics;
expiry retains uncertainty without retry. Debug no longer exposes PIN/email.
[Evidence, tests and unported credential lifecycle](docs/protocol/apk/2.26.43.72-two-factor-callback/README.md).
Validation:1344 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. No live account operation; the broader audit
remains unfinished.

2026-11-05 — **Profile upload success no longer inherits generic nested refusals.**

A typed upload result with first picture child retains native success when
later error extensions exist; outer errors remain refusals. Both photo paths'
zero timer and existing server address are confirmed from the current APK.
[Evidence, tests and unported parser/cache branches](docs/protocol/apk/2.26.43.72-profile-photo-callbacks/README.md).
Validation:1342 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. The existing registration redaction test now
normalizes unrelated random/clock metadata to avoid false PIN matches.
No live account traffic; the broader audit remains unfinished.

2026-11-05 — **About results no longer inherit unrelated nested-refusal rules.**

Typed About outer result follows the APK payload-ignoring success callback;
outer error/code stays a refusal. Its32s screen wait is not a network expiry.
[Native map identity, callback paths and regressions](docs/protocol/apk/2.26.43.72-about-callback-lifetime/README.md).
Validation:1340 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. No replay or live account traffic; broader
audit remains unfinished.

2026-11-05 — **Account-default timer callbacks now have native20s expiry.**

The normal action loop covers default-timer set and empty get shapes, without
replay or adopting an uncertain setting. Other pending deadlines stay independent.
[APK success/failure paths, tests and remaining policy gaps](docs/protocol/apk/2.26.43.72-default-timer-callback/README.md).
Validation:1338 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. No live account traffic; the broader audit
remains unfinished.

2026-11-05 — **Block/unblock callback expiry now follows its20s APK timer.**

Fetch uses a distinct native zero-duration path and receives no invented timer.
Expired changes stay uncertain without replay or local block/hash mutation;
normal receive continues. [Evidence, tests and downstream limits](docs/protocol/apk/2.26.43.72-blocklist-callback-timing/README.md).
Validation:1337 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. No live account traffic; other callback
producers remain under audit.

2026-11-05 — **Initial AB-properties callback now expires without replay.**

The normal loop uses its confirmed32s pre-write timer, including write delay.
Expiry preserves the uncertain Sent row and receive continues; late replies
cannot reclaim an expired callback. Other IQs retain their own policies.
[Evidence, tests and concurrency limits](docs/protocol/apk/2.26.43.72-abprops-callback-timer/README.md).
Validation:1336 workspace/all-feature tests, strict Clippy, fmt, CLI reference,
APK coverage and hygiene passed. No live account traffic.

2026-11-05 — **AB-properties callback timer traced on the current APK.**

Confirmed32s starts at callback registration before writer dispatch, with no
restart at promotion. Expiry competes for callback ownership; the selected
failure callback schedules no retry. Rust per-action expiry remains unported.
[Evidence and precise implementation boundary](docs/protocol/apk/2.26.43.72-abprops-callback-timer/README.md).
Research only; no live traffic or runtime behavior change.

2026-11-05 — **Normal action replies now require live connection ownership.**

Late replies cannot resurrect held action callbacks after reconnect/reopen;
duplicates/local failure cannot claim twice. Saved wire must match its durable
row ID at reservation/settlement. Current and fast replies still settle.
[Native sent-map evidence, tests and remaining limits](docs/protocol/replay-safety-audit.md#one-shot-action-callbacks-stay-with-their-connection--2026-11-05).
Validation:1334 tests, strict Clippy, fmt, CLI reference, APK coverage and hygiene
passed. No replay/status reset or live account traffic.

2026-11-05 — **Application-padding admission now follows the APK count rule.**

Receive accepts count-only nonrepeating tails and counts above16, rejects empty/
zero/entire/oversized framing, and no longer requests retries for these exclusions.
Crypto authentication/block padding stays intact. [Evidence, production-loop test and native durability/ACK limits](docs/protocol/outgoing-retry-receipts.md#application-padding-count-and-retry-admission-corrected--2026-11-05).
Validation:1332 tests, strict Clippy, fmt, CLI reference, regenerated coverage
and hygiene passed. No live account traffic; full audit remains unfinished.

2026-11-05 — **Newer unknown skmsg versions now carry retry error4.**

The typed parser cause survives normal receive, preserving empty-record/length/
version/signature precedence. Legacy versions remain no-retry; other formats
retain separate policies. [Evidence, regressions and remaining limits](docs/protocol/outgoing-retry-receipts.md#newer-unknown-sender-key-versions-retain-retry-error4--2026-11-05).
Validation:1329 tests, strict Clippy, fmt, CLI reference, APK coverage and
hygiene passed. No live account traffic.

2026-11-05 — **Classic local-prekey failures now carry retry error3.**

Local key records decode only when requested, preserving matched-session reuse.
Direct/classic group cases retain cause and uncertain-write reservations; nested
MAC test mutations now preserve outer key selectors. [Evidence and remaining limits](docs/protocol/outgoing-retry-receipts.md#classic-incoming-local-prekey-retry-error-and-lazy-lookup--2026-11-05).
Validation:1329 tests, strict Clippy, fmt, CLI reference, APK coverage and hygiene
passed. No live account traffic; factory/hybrid/full retry policy remains unfinished.

2026-11-05 — **Missing sender-key chains now carry retry error4.**

A stored empty record retains error1 before parsing; an unknown chain in a
nonempty record preserves its distinct wrapped native cause. Redelivery after
uncertain write remains held and sender keys stay unchanged. [Evidence, platform contract and regressions](docs/protocol/outgoing-retry-receipts.md#empty-sender-key-records-and-missing-chains--2026-11-05).
Validation:1329 tests, strict Clippy, fmt, CLI reference, APK coverage, hygiene
and local platform-contract fixture passed. No live traffic; complete native retry
policy remains unfinished.

2026-11-05 — **Confirmed sender-key future jumps now carry retry error6.**

Typed classification uses the effective matching saved chain, preserves signature
precedence and leaves high-bit arithmetic unmapped. Boundary/older-chain/reservation
regressions and normal-loop no-replay checks added. [Evidence and remaining scope](docs/protocol/outgoing-retry-receipts.md#sender-key-future-counter-retry-error-preserved--2026-11-05).
Validation:1329 tests, strict Clippy, fmt, CLI reference, APK coverage and
hygiene passed. No live account traffic.

2026-11-05 — **Future-counter retry prerequisites identified.**

Native sender-key future rejection maps to wire6 after signature verification;
backend shares the generic error with CBC failure. Matching-chain access,
reservation-aware projection and signed/unsigned counter boundaries prevent a
blanket mapping. [Evidence and next implementation requirements](docs/protocol/outgoing-retry-receipts.md#future-counter-typed-mapping-prerequisites--2026-11-05).
Research only; no runtime change or live traffic.

2026-11-05 — **Sender-key signature retries now carry error5.**

Receive preserves message-signature failure separately from generic decryption,
with a typed cause through durable retry construction. Missing-session error1
and once-only reservations remain intact. [Evidence, regression and limits](docs/protocol/outgoing-retry-receipts.md#sender-key-signature-retry-error-preserved--2026-11-05).
Validation:1328 tests, strict Clippy, fmt, CLI reference, APK coverage and
hygiene passed. No live traffic; other producer-specific policies remain unfinished.

2026-11-05 — **Legacy sender-key messages no longer trigger retries.**

A typed legacy skmsg rejection preserves capture without retry, delivery ACK or
state commit. Newer unknown versions, short inputs and other message producers
retain separate policy. [Evidence, regressions and remaining work](docs/protocol/outgoing-retry-receipts.md#legacy-sender-key-message-retry-exclusion--2026-11-05).
Validation:1328 tests, strict Clippy, fmt, CLI reference, APK coverage and
hygiene passed. No live traffic.

2026-11-05 — **Message-version retry distinctions recovered.**

Legacy and higher unknown versions take different exception/status paths;
prekey constructor catch regions differ too. Native direct version2 gate exceeds
the backend's supported range, with full primitive parity unproven. ED6 does not
prove an identity refusal. [Evidence and limits](docs/protocol/outgoing-retry-receipts.md#version-gates-and-no-retry-limits--2026-11-05).
Research only; no runtime change or live traffic.

2026-11-05 — **Normal missing-session retries now include error1.**

Direct syntax admission precedes absent-session classification; group absent
sender-key record retains its own ordering. The normal retry path preserves
this cause and keeps original timestamp/count/reservation behavior. Offline CLI
replay remains offline. [Implementation, evidence and remaining gaps](docs/protocol/outgoing-retry-receipts.md#missing-session-retry-error-preserved--2026-11-05).
Validation:1328 workspace tests plus historical-wire regression, strict Clippy,
fmt, CLI reference, APK coverage and hygiene passed. Other error policies remain
unfinished; no live traffic.

2026-11-05 — **Direct retry cause chains and attribution corrected.**

Ordinary direct A0N independently confirms old-counter exclusion. Missing
session -> wire1 and aggregated No valid sessions. -> wire8 traced; selected
group signature -> wire5 distinguished from bundle signatures. Earlier A0P
label corrected to fast-ratchet group decrypt. [Evidence and remaining work](docs/protocol/outgoing-retry-receipts.md#direct-coordinator-attribution-and-cause-aggregation--2026-11-05).
Research only; no production behavior change or live traffic.

2026-11-05 — **Sender-key old counters now avoid automatic retries.**

Group replay preserves the typed old-counter condition; group diagnostics no
longer format provider payloads. Other key/signature retry mappings remain
producer-specific and unported. [Evidence and limits](docs/protocol/outgoing-retry-receipts.md#group-counter-and-producer-specific-errors--2026-11-05).
Validation:1328 tests, strict Clippy, fmt, CLI reference, APK coverage and
hygiene passed. No live traffic.

2026-11-05 — **Consumed Signal counters no longer trigger automatic retries.**

Typed direct crypto rejection continues receive without retry or delivery ACK;
authenticated stored duplicates keep their existing confirmation behavior.
[APK evidence, offline regression and remaining gaps](docs/protocol/outgoing-retry-receipts.md#old-counters-no-longer-request-retries--2026-11-05).
Validation:1327 workspace tests, strict Clippy, fmt, CLI reference, coverage
and hygiene passed. No live traffic or zero-ban guarantee.

2026-11-05 — **Outgoing retry error domains/eligibility mapped.**

Crypto status, internal failure kind and wire error are distinct; selected
crypto eligibility and guarded refinements recovered. Other producers bypass
the converter. [Evidence and remaining typed-cause work](docs/protocol/outgoing-retry-receipts.md#error-domains-and-eligibility-recovered--2026-11-05).
No blanket error default, runtime behavior change or live traffic.

2026-11-05 — **Outgoing retries now preserve original message timestamp.**

Automatic retry.t uses failed message root t instead of local queue time.
Missing/unusable timestamps refuse without queue writes; held retries do not
reset on changed metadata. [Native chain, regressions and limits](docs/protocol/outgoing-retry-receipts.md).
Validation:1327 tests, strict Clippy, fmt, CLI reference and hygiene passed.
Error-code/key/count progression remains incomplete; no live traffic.

2026-11-05 — **Retry refetch callbacks now belong to their live connection.**

Only a locally written query can complete through normal receive; result/error
consumes its callback once. Disconnect/restart never resurrects callbacks from
held jobs. Validation:1326 tests, strict Clippy, fmt, CLI reference and hygiene
passed. [Native cancellation evidence, tests and limits](docs/protocol/retry-answers.md#live-refetch-callback-lifetime-2026-11-05).
No guessed timeout, new request, reset or automatic replay; no live traffic.

2026-11-05 — **Retry key-query manager-to-wire construction audited.**

Classic non-identity/context-disabled IQ baseline matches Rust. Source12 is not
a wire field; identity/PQ/context attrs have separate predicates. Manager entry
expiry is not a proven IQ deadline. [Evidence and remaining gaps](docs/protocol/retry-key-query-wire.md).
No production field/backoff/replay change or live traffic.

2026-11-05 — **Durable retry key queries now ACK first.**

Selected post-ACK job/session-requirement fetch entry traced. Normal wrapper
attaches root receipt ACK only after pending job and fetching marker commit;
ACK/query failure stops without replay. [Evidence, tests and limits](docs/protocol/retry-ack-dispatch.md#durable-retry-key-query-ack-ordering--2026-11-05).
Validation:1325 tests, strict Clippy, fmt, CLI reference and hygiene passed.
No key-request field/backoff change or live traffic; recovery remains incomplete.

2026-11-05 — **Reserved direct retries now ACK before resend write.**

Normal receive attaches success ACK only after durable message reservation and
uses root receipt correlation. ACK failure prevents resend submission; any
uncertain work stays held. Queries/refusals/duplicates/group paths remain outside
this ACK authority. Validation:1325 tests, strict Clippy, fmt, CLI reference
and hygiene passed. [Evidence, tests and limits](docs/protocol/retry-ack-dispatch.md#reserved-direct-resend-success-ack--2026-11-05). No live traffic.

2026-11-05 — **Retry refetch settlement ownership strengthened.**

Refusal and resend reservation now recheck the pending job snapshot and exact
fetching/query/count checkpoint before consumption. Changed jobs/terminal or
legacy markers remain held without overwriting peer or retry state.
Validation:1324 tests, strict Clippy, fmt, CLI reference and hygiene passed.
[Race regressions and limits](docs/protocol/retry-answers.md#refetch-completion-ownership-guard-2026-11-05). No new send or success ACK.

2026-11-05 — **Retry schema-error ACK context corrected.**

Confirmed required-field/ASCII numeric errors now receive487 with fresh native
context, omitting recipient/edit. Unsupported-version487 keeps its copied context;
first-error order and local capability refusals remain distinct.
Validation:1323 tests, strict Clippy, fmt and CLI reference passed.
[Evidence, tests and limits](docs/protocol/retry-ack-dispatch.md#required-field-parsing-error-responses--2026-11-05). No live traffic.

2026-11-05 — **Unsupported retry versions now receive error487.**

Shared confirmation mirrors supported receipt context for explicit non-1 text
versions after four-byte registration admission. Writer binding and deferred
callback ordering traced. [Evidence, regressions and limits](docs/protocol/retry-ack-dispatch.md#unsupported-version-error-response-correction--2026-11-05).
Other retry ACK/error/recovery policy remains incomplete; no live traffic.
Validation:1322 tests, strict Clippy, fmt, CLI reference and research hygiene passed.

2026-11-05 — **Retry ACK dispatch traced; plain-ACK suspicion refuted.**

Current Rust excludes retry ACKs. Selected APK handler separates handled,
unhandled/error487 and parse-exception paths; actual receipt context factory
located. Final Message76 writer binding/async order remain open.
[Evidence and limits](docs/protocol/retry-ack-dispatch.md). No runtime policy change or live request.

2026-11-05 — **Retry original-chat selection/correlation strengthened.**

Shared parser separates explicit recipient/normalized chat from asking device.
Normal direct retry checks the selected chat against original peer/known LID
before session work or a query. [Evidence and regressions](docs/protocol/retry-original-chat.md).
Validation:1320 tests, strict Clippy, fmt and CLI reference passed.
No receipt-derived alias adoption, live request or full native routing claim.

2026-11-05 — **Retry original-message correlation corrected.**

Shared parser now uses required nested retry.id for original lookup and requires
signed64-bit retry.t. Root receipt ID is independent; malformed nested fields
cannot mutate/query retry state. [Evidence and regressions](docs/protocol/retry-parser-admission.md#nested-message-key-and-timestamp-correction-2026-11-05).
Validation:1318 tests, strict Clippy, fmt and CLI reference passed.
No live request or new timestamp/ACK policy.

2026-11-05 — **Comparison group retry enc counts corrected from pinned DEX.**

Received count1/2 now reaches root skmsg and pairwise distribution enc children
unchanged; original first sends still omit it. Ciphertext/ID/addressing remain
preserved. [Binding, wire regression and limits](docs/protocol/group-retry-count.md).
Validation:1317 tests, strict Clippy, fmt and CLI reference passed.
No live request or full group retry parity claimed.

2026-11-05 — **Comparison group retry peer state and write settlement fixed.**

Reservation checks identity/session snapshots and holds advanced peer state with
a reserved checkpoint before I/O. Owned settlement records written or uncertain;
legacy/held checkpoints never replay. [Tests and limits](docs/protocol/group-retry-admission.md#peer-snapshot-and-write-settlement-correction-2026-11-05).
Validation:1316 tests, strict Clippy, fmt and CLI reference passed.
No live socket, key fetch or resend operation.

2026-11-05 — **Comparison group retry admission strengthened.**

Before reserving a device or querying keys, the window checks group-chat/message
correlation, supported count, carried-key exclusion and original/duplicate device.
[Evidence, tests and remaining issues](docs/protocol/group-retry-admission.md). No live request or group policy
completion claimed. Validation:1314 tests, strict Clippy, fmt and CLI
reference passed.

2026-11-05 — **Retry parser registration/version/defaults corrected.**

Shared parser requires four registration bytes, defaults missing v/count to1
and refuses unsupported version/malformed count. Invalid registration no longer
falls through to a key query. [Evidence and tests](docs/protocol/retry-parser-admission.md).
Validation:1313 tests, strict Clippy, fmt and CLI reference passed.
No live request or expanded higher-count/key handling.

2026-11-05 — **Unported receipt-key retries no longer fall through.**

Normal direct retry driver records receipt_keys_unsupported and returns no
resend/key query when keys are carried. Supported no-keys retries still work.
[Evidence, local limit and tests](docs/protocol/retry-answers.md#receipt-carried-keys-capability-boundary-2026-11-05).
Validation:1312 tests, strict Clippy, fmt, CLI reference and hygiene passed.
No key adoption or live requests.

2026-11-05 — **Retry refetch query reservation now rechecks its inputs.**

Original row/age and identity/session snapshots are checked atomically with
pending query records; stale inputs produce no query job. Duplicate answer
reservation remains once-only. [Correction and regressions](docs/protocol/retry-answers.md#refetch-query-reservation-inputs-2026-11-05).
Validation:1312 workspace tests, strict Clippy, fmt and CLI reference passed.
No live requests or expanded retry counts/root behavior.

2026-11-05 — **Saved-session retry reservation race fixed offline.**

Retry reservation now rechecks original outbox snapshot, identity/session and
age after async crypto, preserving newer delivery/failure/attempt/peer state.
Fresh and saved paths share guards. [Correction and tests](docs/protocol/retry-answers.md#saved-session-retry-reservation-guard-2026-11-05).
Also retained checked current-APK root-removal gates without guessing their
applicability or changing root attributes. Validation:1311 workspace tests,
strict Clippy, fmt and CLI reference passed. No live retry operation.

2026-11-05 — **Recovered metrics consumer checksum and task admission traced.**

Version3 payload Adler32 is checked after file loading; versions1/2 skip this
consumer comparison. Mismatch increments a counter and ends the remaining
snapshot walk. Passing outgoing entries reach conditional transport handoff.
[Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-recovery-consumer/README.md).
Validation:12 native checksum cases and mismatch-counter slices plus pinned
binding/windows. Startup flag/lifetime and actual replay remain unobserved.
No real I/O, Rust runtime change or live traffic.

2026-11-05 — **Metrics buffer loader admission and bulk-delete distinction traced.**

Loader accepts selected prefix/type and WMT versions1/2/3 with positive bounded
payload; restores checksum/state without payload comparison in this body.
Separate directory walk is bulk deletion. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-buffer-recovery/README.md).
Validation:18 native header cases and pinned loader/helper windows. Remaining
files can pass local gates; automatic restart replay and cleanup policy remain
unproven. No real I/O, Rust runtime change or live traffic.

2026-11-05 — **Metrics completion-to-file-removal chain traced.**

Selected regular table now binds owner callbacks: null-error success queues
buffer removal; terminal local error skips that deletion path despite set
removal. Worker remove failures report errno separately. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-buffer-deletion/README.md).
Validation:4 controlled full-worker cases and pinned construction/function
windows. Downstream bookkeeping/recovery remains open; no real files removed,
Rust runtime change or live traffic.

2026-11-05 — **Regular metrics recovery timer and provider gates traced.**

Matching provider/status0 with idle low bit starts a drain and interval30 timer;
status1 cancels the existing timer and signals drain error. Timer callbacks
require ready and idle low bits. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-regular-recovery/README.md).
Validation:72 native status-decision and16 timer-decision fixtures plus pinned
window resolver. Weak resolution, actual timers and durable ownership remain
separate; no Rust runtime or live traffic changed.

2026-11-05 — **Metrics error subcodes and credential effects traced.**

Pinned native paths recognize3067002/3067003/3067004, release the current
metrics credential and promote a spare or request gated renewal. Failure-reason
suffix parsing is distinct from response-body parsing; diagnostic numbers are
not account restriction codes. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-error-subcodes/README.md).
Validation:24 synthetic body-parser cases,7 worker integer-slice cases, pinned
resolver and bundle checks. Full retry/durable ownership and server meanings
remain open. No Rust runtime or live traffic changed.

2026-11-05 — **Regular metrics IQ producer and response bridge traced.**

Native job reaches selector6 w:stats; callback-backed32s wait maps exact result
to success0, other outcomes to failure3. Corrected earlier Java length-enforcement
claim: A04 logs bounds through nonthrowing A0C. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-regular-iq/README.md).
Validation: pinned window/JNI/interface resolvers, fresh Java exports, coverage
and bundle/link hygiene. Full durable/retry ownership remains open. No Rust
runtime or live traffic changed.

2026-11-05 — **Regular metrics drain notifier separated from task ownership.**

Fourteen native helper cases verify one-shot flag/status/error behavior. This
helper does not implement backoff or mutate persistent buffers. Task outcome
retention/removal remains separate. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-native-drain-outcomes/README.md).
Validation: pinned resolver and fixtures, coverage, staged diff and bundle/link
hygiene. Full upper callback/retry/durable ownership remains open. No Rust
runtime or live traffic changed.

2026-11-05 — **Native metrics task cancellation ordering traced.**

Twelve bounded native cases verify handle cancel/callback arguments and local
cleanup. Destructor clears callback table before cancellation; endpoint map and
durable delivery remain separate. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-native-task-cancellation/README.md).
Validation: pinned window/caller resolver, fixture reproduction, coverage, staged
diff and bundle/link hygiene. Full callback/replay ownership and real cancel
completion remain open. No Rust runtime or live traffic changed.

2026-11-05 — **Native metrics connection-status timer semantics traced.**

Corrected provisional load-completion label: connection status starts/stops
the repeating task timer. Fifteen bounded native cases verify arm/event/cancel
and readiness gating. [Evidence and concurrency limits](docs/protocol/apk/2.26.43.72-metrics-native-recovery-timer/README.md).
Validation: pinned resolver/fixtures, prior credential-file repro after rename,
coverage and bundle/link hygiene. No justified universal Rust IQ-origin change
found; no Rust runtime or live traffic changed.

2026-11-05 — **Selected native analytics bytes verified end to end offline.**

19 full-builder cases match expected bytes with controlled libc adapters,
including buffer growth, retry truncation and raw-object/quote behavior.
[Native harness and scope](docs/protocol/apk/2.26.43.72-metrics-native-request-tags/README.md#complete-selected-serialization-fixtures-2026-11-05).
Earlier24 branch cases remain distinct. Validation: pinned resolver and both
harness reproductions, coverage, staged diff and bundle/link hygiene. General
inputs/allocator failures and live acceptance remain open. No Rust/live change.

2026-11-05 — **Native request analytics mappings and metadata traced.**

Selected purpose1=prefetch and category4=analytics; metadata Boolean/config
sources distinguished.24 native branch cases pass with controlled pair/zero
format adapters. [Evidence and serialization limits](docs/protocol/apk/2.26.43.72-metrics-native-request-tags/README.md).
Full formatting, upstream product, selector plaintext and crypto remain open.
Validation: pinned table/window resolver and fixtures, coverage, staged diff
and bundle/link hygiene. No Rust runtime or live traffic changed.

2026-11-05 — **Native metrics endpoint list production traced.**

Java provider entries are converted in order, invalid conversions skipped and
fallback appended last; selected response retries consume the stored list.
Seven bounded append/store cases pass. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-native-endpoints/README.md).
Actual endpoint values, full retry/generation policy and crypto remain open.
Validation: pinned native/constant resolver, fixtures, exact Java export, coverage
and bundle/link hygiene. No Rust runtime or live traffic changed.

2026-11-05 — **Native metrics credential file and restoration traced.**

Paired save/load retains expiry/use count instead of relying on constructor
defaults.23 bounded native header cases verify version/range/length checks.
[Evidence, write/close handling and durability limits](docs/protocol/apk/2.26.43.72-metrics-native-credential-file/README.md).
Header admission is not a valid credential. Endpoints, load scheduling and
crypto remain open. Validation: pinned resolver/fixture reproduction, coverage,
checksums/links and staged diff. No Rust runtime or live traffic changed.

2026-11-05 — **Outbound metrics JNI method-cache binding resolved.**

JNI_OnLoad's indexed lookup/store links native cache0xc467f0 to the exact Java
credential producer. Five bounded native-loop fixtures cover complete/partial
lookup and aggregate initialization flag. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-jni-method-cache/README.md).
Live initialization, whole-loader recovery and crypto remain open. Validation:
pinned ELF symbol/table resolver and fixture reproduction, coverage, staged diff
and bundle/link hygiene. No Rust runtime or live traffic changed.

2026-11-05 — **Credential callback timeout and parse-failure ownership traced.**

Pinned class_def proves Hyg's ordinary dispatch path. Timeout claims callback
before BsS/native result1; malformed required fields can consume ownership
without reaching native completion. [Scope and unresolved recovery](docs/protocol/apk/2.26.43.72-metrics-credential-iq/README.md#credential-callback-timeout-ownership-and-schema-failure-2026-11-05).
Validation: interface reproduction, fresh shared-body comparisons, coverage,
checksum/link hygiene and staged diff. No Rust runtime or live traffic changed.

2026-11-05 — **Metrics credential IQ schema and callback mapping traced.**

Java selector7 builds diagnostic32-byte blinded credential bounds; independent
callback registration precedes dispatch. Required response fields and JNI4
failure-to-native-raw1 mapping retained. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-credential-iq/README.md).
Native outbound method-cache binding, crypto and full timeout/error dispatch
remain open. No Rust runtime or live traffic changed. Validation: pinned native
resolver reproduction, exact selected smali export, JSON, coverage/link hygiene.

2026-11-05 — **Native metrics credential gates and refresh ownership traced.**

17 bounded native fixtures verify constructor storage/defaults, counter setter
and refresh guards. Admission, mode-dependent count override, result switch and
temporary cleanup retained statically. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-native-credential-lifecycle/README.md).
Provider selector7, crypto, clock units and complete callback ownership remain
open. Validation: pinned resolver/fixture reproduction, coverage, checksum/link
hygiene and staged diff. No Rust runtime or live requests changed.

2026-11-05 — **Native metrics POST and multipart framing traced.**

Five header sources and four body sections retained with exact literals; endpoint
consumption precedes construction. Selector decoding and full credential/schema
contracts remain open. [Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-native-request-builder/README.md).
Validation: pinned image, literal/relocation resolver reproduction, JSON, coverage
and bundle/link hygiene. No Rust behavior or live operation changed.

2026-11-05 — **Native metrics transport status/endpoint ordering traced.**

HTTP request-object creation, async completion, ordinary 200/no-error success,
distinct 401 and regular task outcome labels now have static evidence.
[Branch ordering and limits](docs/protocol/apk/2.26.43.72-metrics-native-transport-responses/README.md).
Request fields, retry timing and durable buffer ownership remain incomplete.
Validation: pinned image, mapped byte digests, outcome/thunk resolver reproduction,
JSON, coverage and bundle/link hygiene. No client behavior or live traffic changed.

2026-11-05 — **Native metrics integer bytes verified at width boundaries.**

42 bounded cases confirm signed-width payloads and scalar-zero/one special tags;
worker-to-field encoding and selected header packing now have static evidence.
[Fixtures and remaining framing work](docs/protocol/apk/2.26.43.72-metrics-native-integer-encoding/README.md).
No complete event/send/reply proof. Validation: pinned image, byte windows,
42 native fixtures/reproduction, JSON, coverage and checksum/link hygiene.
No Rust telemetry/runtime change or live traffic.

2026-11-05 — **Native metrics snapshot validator checked with 43 bounded cases.**

Exact per-group ranges/error precedence and rejected-state replacement now have
static and synthetic native-code evidence.
[Harness, results and scope](docs/protocol/apk/2.26.43.72-metrics-native-snapshot-validation/README.md).
Local metadata validation only; full lifecycle and transport/replies remain open.
Validation: pinned image, raw byte digests, 43 native fixtures/reproduction,
JSON, coverage and bundle/link hygiene. No Rust/live account/network change.

2026-11-05 — **Native metrics metadata file path and recovery envelope mapped.**

Paired writer/recovery evidence identifies /wam-metadata, rb/wb modes, rewind,
1692-byte MTD/version-4 records and a compatible normal-path snapshot guard.
[Evidence and remaining limits](docs/protocol/apk/2.26.43.72-metrics-native-metadata-file/README.md).
Full validation, durability, buffer format and transport/replies remain open.
Validation: pinned image, section-aware digests, mode/direct-branch resolver
reproduction, JSON, coverage and bundle/link hygiene. No Rust/live change.

2026-11-05 — **Native metrics pending worker and selected persistence traced.**

Async drain removes before processing and adds channel-specific time metadata;
selected snapshot helpers reach fwrite/fflush with explicit unresolved guards.
[Evidence and remaining boundaries](docs/protocol/apk/2.26.43.72-metrics-native-drain-persistence/README.md).
No delivery or crash-durability proof. Actual transport/replies remain open.
Validation: pinned image, mapped byte digests, thunk/clock resolver reproduction,
JSON, coverage and checksum/link hygiene. No Rust runtime change or live traffic.

2026-11-05 — **Native metrics event/setup JNI entries and local routing mapped.**

Recovered libwhatsapp tuples now connect Java events to typed native fields and
state-dependent local manager handoff. Normal event return is literal zero.
[Static evidence and limits](docs/protocol/apk/2.26.43.72-metrics-native-event-routing/README.md).
Serialization, persistence, flush, transport and server replies remain unverified.
Validation: image identity, exact tuple signatures, section-aware byte digests,
resolver reproduction, JSON, coverage and checksum/link hygiene. No Rust/live change.

2026-11-05 — **All 83 packed ARM64 libraries reconstructed and metadata-checked.**

65766648 recovered bytes match metadata lengths/XXH64 and pass ELF layout checks.
The recovered WhatsApp libraries now unblock static native JNI/metrics analysis.
[Harness, verification and limits](docs/protocol/apk/2.26.43.72-superpack-reconstructed-members/README.md).
Bounded decoder/rebuilder emulation only; no reconstructed library execution,
Rust runtime change or live traffic. Validation: full output reproduction,
83 metadata/hash/layout checks, sort fixtures, coverage and bundle/link hygiene.

2026-11-05 — **All 213 framed blocks decoded; so-driver reconstruction next.**

140 bounded XZ decodes and 73 constrained emulations produced 81747344 bytes.
These are intermediate buffers, not recovered ELF libraries.
[Manifest, driver mapping and limits](docs/protocol/apk/2.26.43.72-superpack-all-blocks/README.md).
Validation: pinned hashes, complete frame EOF/length checks, emulation return
records, all saved-buffer digests, JSON, coverage and bundle/link hygiene.
No Rust behavior, live account operation or network traffic changed.

2026-11-05 — **Type-42 decoder produced three bounded emulated block candidates.**

The largest selected block returned 17778932 bytes and reproduced exactly.
These are intermediate buffers, not usable native libraries.
[Harness, results and limits](docs/protocol/apk/2.26.43.72-superpack-kind42-emulation/README.md).
Selected target instructions ran under Unicorn with controlled memory imports;
no Android app, account or network operation. Rust client behavior unchanged.
Validation: pinned hashes, candidate reproduction, helper comparison, JSON,
coverage, byte-window/bundle checksums and link hygiene.

2026-11-05 — **Entire Superpack payload mapped as 213 framed blocks.**

The spo descriptor and parser account for the exact archive boundary: 140 XZ
blocks and 73 blocks dispatched to a separate kind-42 decoder.
[Reproduction and limits](docs/protocol/apk/2.26.43.72-superpack-transform-blocks/README.md).
Research only; complete native images/metrics transport remain unverified.
Validation: pinned hashes, exact inspector reproduction, assembly-range digests,
JSON, coverage, checksum/link hygiene. No runtime change or live traffic.

2026-11-05 — **Structured inventory identifies 83 packed native members.**

Pinned parser-derived metadata now locates whatsapp/whatsappmerged records and
the remaining payload boundary. Complete native images remain unreconstructed.
[Inspector, evidence and limits](docs/protocol/apk/2.26.43.72-superpack-metadata/README.md).
Research only; no runtime change or live traffic. Validation: pinned APK/member
hashes, inspector output reproduction, raw assembly digests, JSON, coverage/hygiene.

2026-11-05 — **Superpack member ranges and disk-output boundary traced.**

The core selects iterator ordinals and returns names after selected writes;
its writer does not establish crash-durable multi-file extraction.
[Evidence and remaining parser work](docs/protocol/apk/2.26.43.72-superpack-member-output/README.md).
Research only; no runtime change or live traffic. Validation: pinned APK/member
identity, byte-window digests, host JNI layout, JSON, coverage and bundle hygiene.

2026-11-05 — **From-so decoder connected to verified carrier archive range.**

Default ranges, spo type gate, symbol lookup and cached/error branches now lead
to the identified decode core; full unpacking remains unverified.
[Evidence and limits](docs/protocol/apk/2.26.43.72-superpack-from-so/README.md).
Research only; no runtime changes or live traffic. Validation: pinned/range
digests, mapped Strings, bounded disassembly, checksums, JSON, coverage and hygiene.

2026-11-05 — **Thirteen decoder/reader JNI entries mapped to native addresses.**

Relocated tables and registration call sites agree with exact DEX signatures.
This enables targeted decoder tracing; full inner libraries remain unrecovered.
[Evidence and limits](docs/protocol/apk/2.26.43.72-superpack-jni-registration/README.md).
Research only; no runtime changes or live traffic. Validation: pinned digests,
reproducible resolver, signature/layout checks, checksums, JSON, coverage and hygiene.

2026-11-05 — **Decoder usability export inspected without execution.**

The selected native probe writes three fixed bytes, not a decompression test.
Openbox reader boundaries are identified; inner-library recovery remains open.
[Evidence and limits](docs/protocol/apk/2.26.43.72-superpack-decoder-entry/README.md).
Research only; no runtime changes or live traffic. Validation: member/function
digests, exact-size disassembly, host ABI helper, checksums, JSON, coverage and hygiene.

2026-11-05 — **Superpack loader and bounded XZ probe retained.**

Twelve embedded streams decode reproducibly but do not start with ELF. Full
inner-library reconstruction and JNI binding remain unverified.
[Evidence and limits](docs/protocol/apk/2.26.43.72-superpack-unpack-boundary/README.md).
Research only; no runtime changes or live traffic. Validation: pinned digests,
probe rerun/output match, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Metrics native declarations and ARM64 carrier pinned.**

The selected carrier exposes superpack archive markers; JNI implementation
requires inner-library extraction/registration proof. Native REA is unavailable.
[Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-native-carrier/README.md).
Research only; no runtime changes or live traffic. Validation: APK/member digests,
exact declarations, readelf output, checksums, JSON, coverage and repository hygiene.

2026-11-05 — **Metrics FIFO and exceptional latch-release ordering verified.**

The serial wrapper advances after thrown initialization work as well as normal
return, so subsequent latch release is not a checked native-success result.
[Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-readiness-latch/README.md#fifo-completion-advances-even-after-an-initialization-exception).
Research only; no runtime changes or live traffic. Validation: two complete
export/source matches, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Metrics readiness latch release and native setup traced.**

Release is a separate queued task; static count zero and per-instance activation
are distinct from a checked native initialization result.
[Evidence and limits](docs/protocol/apk/2.26.43.72-metrics-readiness-latch/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
source/excerpt match, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Primary metrics worker validation and JNI handoff traced.**

Validation diagnostics do not automatically drop the original event. Latch return
and a discarded native result are not proof of network or server delivery.
[Evidence and limits](docs/protocol/apk/2.26.43.72-startup-metrics-native-handoff/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
source/excerpt match, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Startup metrics sink factory and primary sampling traced.**

Configuration selects one of three sinks; primary admission can drop or queue.
Neither a metrics call nor scheduling proves a network write or delivery.
[Evidence and limits](docs/protocol/apk/2.26.43.72-startup-metrics-sink/README.md).
Research only; no runtime changes or live traffic. Validation: pinned APK digest,
checked exports/excerpt, checksums, JSON, coverage freshness and hygiene.

2026-11-05 — **Post-availability session bookkeeping and metrics traced.**

Selected hooks record session/uptime and conditionally hand offline-resume
metrics to a sink. Neither scheduling nor handoff proves network delivery.
[Evidence and limits](docs/protocol/apk/2.26.43.72-startup-session-hooks/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
source/excerpt match, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Current-build bare availability and startup conditions traced.**

The selected post-config/unified-session write is conditional bare availability.
Rust shape agrees; complete mode policy and server ACK behavior remain open.
[Evidence and limits](docs/protocol/apk/2.26.43.72-startup-availability/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
source/excerpt match, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Unified-session generic envelope and writer path traced.**

Kind447 uses envelope6, bypasses selected pre-ACK normalization and has no IQ-ID
promotion here. Offline defer and I/O requeue are local, not delivery proof.
[Evidence and limits](docs/protocol/apk/2.26.43.72-unified-session-writer/README.md).
Research only; no runtime changes or live traffic. Validation: reused writer
match, checked excerpts, checksums, JSON, coverage freshness and hygiene.

2026-11-05 — **Unified-session eligibility provider resolved.**

The selected predicate loads and tests Me-record presence, not socket readiness.
Cached-ID return remains separate from refresh/identity-change handling.
[Evidence and limits](docs/protocol/apk/2.26.43.72-unified-session-eligibility/README.md).
Research only; no runtime changes or live traffic. Validation: pinned APK digest,
checked exports, reused-source match, checksums, JSON, coverage and hygiene.

2026-11-05 — **Unified-session startup stanza and ID lifecycle identified.**

The post-config node is ib/unified_session. Cached ID generation, refresh and
identity-observer paths are distinct from IQ callback ownership.
[Evidence and limits](docs/protocol/apk/2.26.43.72-unified-session-startup/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **FBNS message ACK eligibility and JSON-failure continuation traced.**

The provider IPC ACK can follow empty or invalid JSON data. Its completion flag
is not payload-success, durable application or WhatsApp stanza-ACK proof.
[Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-fbns-token-received/README.md#provider-message-ack-after-normal-handling-or-json-failure).
Research only; no runtime changes or live traffic. Validation: checked exports,
excerpt/source match, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **FBNS callback persistence and optimistic marker traced.**

The callback can write last_server_fbns_token after SET submission without
checking its admission result; that preference is not server-acceptance proof.
[Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-fbns-token-received/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
excerpt/source match, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **FBNS reconciliation gate and preference ordering traced.**

Null config removes the saved server token view, not the local token here.
Refresh conditionally enters Android provider IPC before any later SET.
[Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-fbns-reconciliation/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Push-config startup order and404 reconciliation work traced.**

The selected GET follows token drain/conditional rotation. Its404 error queues
VERIFY with null String inputs rather than directly retrying the GET.
[Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-trigger-error/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
source/excerpt match, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Alternate push-config admission and Long syntax traced.**

Non-Android parsed entries still consume the reply collection allowance, despite
no Android holder projection. Numeric syntax and field bounds remain separate.
[Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-variant-admission/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Android push-config wire fields and null gates traced.**

Platform/token gate the selected variant; optional field failures do not all
reject it. Internal projection is now connected to actual wire attributes.
[Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-android-grammar/README.md).
Research only; no runtime changes or live traffic. Validation: pinned APK hash,
checked exports, checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Push-config reply collection bound traced.**

The0..2 bound counts converted wrappers; null variant conversions are skipped.
Thrown converter failures remain distinct. [Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-reply-collection/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
checksums, JSON, coverage freshness and repository hygiene.

2026-11-05 — **Corrective push-config sender shape and admission traced.**

Same-tag builder merges yield one config child. The submit gate and offline
defer policy differ from client-auth-token submission; its boolean proves
local admission only. [Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-corrective-submit/README.md).
Research only; no runtime changes or live traffic. Validation: pinned APK hash,
checked exports, checksums, JSON, coverage freshness and repository hygiene.

2026-11-04 — **Push-config verification work and fallback values traced.**

Config handling queues VERIFY work, omits empty String extras, and defaults
account count/VOIP parse failures to1/0. Later service gates control corrective
send invocation. [Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-verification-work/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
checksums, JSON, coverage freshness and repository hygiene.

2026-11-04 — **Push-config internal fields and selected consumer traced.**

Eight projected values concern push registration, mute/account/key state and
VOIP payload type; FBNS follows a separate conditional consumer call.
[Evidence and limits](docs/protocol/apk/2.26.43.72-push-config-projection/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
checksums, JSON, coverage freshness and repository hygiene.

2026-11-04 — **Lower-level IQ reader/writer map identity traced.**

Push-config GET registers in the shared map before writing and has its own
result bridge. The generic token writer does not explicitly add that record.
[Evidence and scoped limits](docs/protocol/apk/2.26.43.72-lower-level-iq-ownership/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
full-source/excerpt match, checksums, JSON, coverage freshness and hygiene.

2026-11-04 — **IQ reader type/ID admission and fallback traced.**

The selected response path requires result/error plus nonnull ID. Lower-level
callbacks and an unmatched-response fallback have separate ownership.
[Evidence and limits](docs/protocol/apk/2.26.43.72-iq-reader-admission/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
digest-bound metadata extraction, checksums, JSON, coverage freshness and hygiene.

2026-11-04 — **IQ response ownership and token schema-catch chain traced.**

Timer cancellation, callback claim and parsing are separate stages. A schema
failure does not restore the pending nonce in the selected catch body.
[Evidence and limits](docs/protocol/apk/2.26.43.72-iq-response-ownership/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
digest-bound metadata extraction, checksums, JSON, coverage freshness and hygiene.

2026-11-04 — **Channel shutdown predicate resolved to queued-work checks.**

The selected predicate combines two handler queues and a deferred deque; it
cannot prove delivery or completion of an executing write. Connection drains
the deque through existing send gates. [Evidence and limits](docs/protocol/apk/2.26.43.72-channel-queue-predicate/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
checksums, JSON, coverage freshness and repository hygiene.

2026-11-04 — **IQ shutdown arguments and cancellation-map boundary traced.**

Disconnect and selected login-back failure pass different flush arguments.
The channel predicate includes a queued-message check; adjacent cancellation
uses a separate map. [Evidence and limits](docs/protocol/apk/2.26.43.72-iq-shutdown-eligibility/README.md).
Research only; no runtime changes or live traffic. Validation: checked exports,
checksums, JSON, coverage freshness and repository hygiene.

2026-11-04 — **Partial encrypted write uncertainty now has a regression test.**

A local verified-Noise peer receives a ciphertext prefix, then disconnects.
The comparison helper reports BrokenPipe, consumes the channel and preserves
a synthetic uncertain intent across store reopen. This is local duplex I/O,
not a live-server test or a production outbox reservation test.
[Scope and limits](docs/protocol/replay-safety-audit.md).
No runtime behavior change or live account operation.
Validation:1,310 workspace tests, strict all-target/all-feature Clippy,
CLI-reference regeneration, formatting, coverage freshness and hygiene.

2026-11-04 — **IQ callback promotion is confirmed pre-write.**

The native sent registry does not prove socket delivery. Presence checks and
promotion are separate; selected shutdown handling flushes the sent map,
not every queued callback. [Evidence and limits](docs/protocol/apk/2.26.43.72-iq-promotion-shutdown/README.md).
Research only; no runtime changes or live traffic. Validation: checked writer,
registry and handler exports, checksums, JSON, coverage freshness and hygiene.

2026-11-04 — **Token delivery-failure and IQ-error effects distinguished.**

Delivery failure restores a nonce only into an empty pending slot; IQ error
only logs in the selected callback. Submit-time readiness has an additional
flag check. [Evidence and limits](docs/protocol/apk/2.26.43.72-client-auth-token-failure/README.md).
Research only; no sender or live traffic. Validation: checked callback/helper
exports, checksums, JSON, coverage freshness and repository hygiene.

2026-11-04 — **Token-manager nonce register flow verified statically.**

Conservative control-flow analysis confirms the original nonnull push nonce
reaches the manager call despite register reuse elsewhere.
[Checker and limits](docs/protocol/apk/2.26.43.72-client-auth-token-nonce-flow/README.md).
Research only; no sender or live traffic. Validation: fresh APK export match,
digest-bound CFG check, checksums, JSON, coverage freshness and hygiene.

2026-11-04 — **Token-manager observer membership confirmed statically.**

Provider binding and DEX interface inheritance connect readiness to the07v
scheduled path, whose delivery re-enumerates observers before the token
manager's readiness recheck. [Evidence and limits](docs/protocol/apk/2.26.43.72-client-auth-token-membership/README.md).
Research only; no sender or live traffic. Validation: digest-bound declaration
extractor, checksums, JSON, coverage freshness and repository hygiene.

2026-11-04 — **Client-auth-token observer topology narrowed.**

Provider-backed membership and inline/scheduled dispatch are distinct proof
obligations; a separate connection-thread drain is now located.
[Evidence and limits](docs/protocol/apk/2.26.43.72-client-auth-token-observer/README.md).
Research only; no runtime changes or live traffic. Validation: checksum,
JSON, coverage freshness and repository hygiene.

2026-11-04 — **Client-auth-token readiness producer traced.**

Connected-handler branches set state2; notifier and manager callback connect
to a replaceable pending slot. Observer registration and lifecycle ordering
remain open. [Evidence](docs/protocol/apk/2.26.43.72-client-auth-token-readiness/README.md).
Research only; no runtime changes or live traffic. Validation: checksum,
JSON, coverage freshness and repository-hygiene checks.

2026-11-04 — **Client-auth-token trigger boundary narrowed.**

GCM pn maps to the pushNonce argument; numeric gate is state==2. Trusted
provenance, full consumer control flow and state2 producer remain open.
[Evidence and limits](docs/protocol/apk/2.26.43.72-client-auth-token-trigger/README.md).
Research only; no sender or live traffic. Validation: checksums, JSON,
coverage freshness and repository hygiene.

2026-11-04 — **Client-auth-token reply validation traced.**

Original request ID/destination and result type bind token parsing; cat holds
1..1024 opaque bytes. Invalid-header/value outcomes differ. Input authenticity,
full JID parsing and recovery remain unfinished.
[Evidence and limits](docs/protocol/apk/2.26.43.72-client-auth-token-reply/README.md).
Validation: checksums, JSON, coverage freshness and repository hygiene.
Research only; no production changes or live traffic.

2026-11-04 — **Client-auth-token request writer traced.**

Selected constructor emits server IQ GET/pn input; length checks are diagnostic
and do not prove a numeric ID requirement or mandatory phone-number field.
[Evidence and unported eligibility/recovery](docs/protocol/apk/2.26.43.72-client-auth-token-writer/README.md).
Validation: checksums, JSON, coverage freshness and repository hygiene.
Research only; no sender or live traffic.

2026-11-04 — **Challenged-request map ownership traced.**

Held selection retains eligible entries; token-result and connected paths reach
conditional re-dispatch. Callback/count/restart boundaries remain unported;
Rust uncertainty holds are unchanged.
[Evidence and port limits](docs/protocol/apk/2.26.43.72-challenged-request-ownership/README.md).
Validation: evidence checksums, JSON, coverage freshness and repository hygiene.
Research only; no live traffic.

2026-11-04 — **Comparison request waits keep pre-write deadlines.**

Nine existing comparison write/wait pairs now share one deadline instead of
starting a fresh window after a slow write. Payloads and no-retry policy remain.
[Evidence, regression and limits](docs/protocol/network-correctness-audit.md#comparison-request-deadlines-start-before-write-2026-11-04).
Validation:1,309 workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Activation admission and deadline ownership hardened.**

Comparison receive ignores request-shaped activation IQs. Actor/CLI activation
checks expiry before advancing, and request writes no longer restart the window.
[Native evidence, regressions and limits](docs/protocol/network-correctness-audit.md#activation-reply-grammar-and-deadline-ownership-2026-11-04).
Validation:1,308 workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Malformed comparison wait records captured before exit.**

Successfully decrypted but undecodable records now survive comparison wait
failure, including invalid compression. The request stays uncertain and no
later send is authorized. Existing offline undecodable inspection applies.
[Tests and capture limits](docs/protocol/network-correctness-audit.md#undecodable-comparison-wait-frames-retained-2026-11-04).
Validation:1,306 workspace/all-feature tests, strict Clippy, regenerated CLI
reference, formatting, JSON, coverage freshness and repository hygiene passed.
The final full run includes the updated CLI description/reference.
No live traffic.

2026-11-04 — **Comparison failed-wait captures retained.**

Repository CLI waits now commit decoded nodes before matching or exiting.
Timeout/error/count/terminal paths retain evidence without settling or replaying
the uncertain request. Legacy no-store API and undecodable frames remain limited.
[Evidence and reopen regression](docs/protocol/network-correctness-audit.md#comparison-wait-captures-survive-failed-exits-2026-11-04).
Validation:1,306 workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Comparison IQ waits reject request-shaped replies.**

The shared waiter requires exact result/error for same-ID IQ responses.
Native dispatch evidence reused; timeout-path pending capture loss remains open.
[Evidence and regression scope](docs/protocol/network-correctness-audit.md#comparison-iq-waiter-reply-admission-2026-11-04).
Validation:1,305 workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Comparison send ACK outcomes hardened.**

Direct/group comparison sends preserve auth challenges and unrelated ACKs as
uncertain. Group distributions require message ACKs and preserve the matched
reply before allowing the next send.
[Evidence, tests and limits](docs/protocol/network-correctness-audit.md#comparison-sends-share-message-ack-classification-2026-11-04).
Validation:1,304 workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Auth-challenge ACKs no longer become ordinary send failures.**

Numeric459 retains uncertain direct/group reservations; native diversion and
manager-map transfer traced. Challenge-controller recovery remains unported.
[Evidence and limits](docs/protocol/apk/2.26.43.72-ack-auth-challenge/README.md).
Validation:1,303 workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed. An unrelated
timestamp-tie fixture was corrected before the passing full rerun.
No live traffic or challenge retry.

2026-11-04 — **ACK API result variants and location distribution distinguished.**

Null-returning ACK wrappers differ from node-returning message callbacks.
Location notification distribution is not proof of ordinary direct SKDM shape;
the historical inference is now qualified.
[Evidence and remaining ownership gaps](docs/protocol/apk/2.26.43.72-ack-api-variants/README.md).
Validation: JSON, checksum/link hygiene and coverage freshness. Evidence/docs
only; no production changes or live traffic.

2026-11-04 — **ACK key propagation and normalization traced.**

Device-zero canonicalization and exact-key failure removal distinguished from
generic inbound forwarding. Distribution address admission remains bounded by
untraced producer/parser inputs.
[Evidence and limits](docs/protocol/apk/2.26.43.72-ack-key-normalization/README.md).
Validation: checksum/link hygiene, JSON and coverage freshness. Evidence only;
no production changes or live traffic.

2026-11-04 — **Sender-key ACK marking and one-time reset distinguished.**

Native callback registration/wait, normal SQL marking, conditional reset and
pre-ACK confirmation wrapper origin traced. Per-distribution Rust durable
confirmation remains incomplete; no speculative recovery behavior added.
[Evidence and next boundary](docs/protocol/apk/2.26.43.72-sender-key-ack-confirmation/README.md).
Validation: evidence checksums, JSON, coverage freshness and repository hygiene.
Docs/evidence only; no live traffic.

2026-11-04 — **Distribution ACK family admission corrected.**

Group preparation now requires message-class ACKs before advancing or refusing
a distribution waiter. Native class-key evidence and offline before/after
regression retained; sender normalization remains untraced.
[Evidence and limits](docs/protocol/apk/2.26.43.72-distribution-ack-admission/README.md).
Validation:1,303 workspace/all-feature tests, strict Clippy, regenerated CLI
reference, formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Outgoing preparation deadlines enforced independently.**

Late group device results cannot advance into key queries after expiry.
Direct/group waiter admission now checks deadlines; existing uncertainty and
no-retry behavior remain. Distribution ACK ownership is a retained audit lead.
[Evidence and limits](docs/protocol/network-correctness-audit.md#outgoing-preparation-owns-its-deadline-2026-11-04).
Validation: workspace/all-feature tests (bounded concurrency), strict Clippy,
CLI reference regeneration, formatting, JSON, coverage freshness and repository
hygiene passed.
No live traffic or ban-prevention guarantee.

2026-11-04 — **Offline digest inspection implemented.**

Savedcapturegrammar/localkeycomparison produces redacteddiagnostics without
resigning orstatechanges. FrozenLRsignature provenance required.
[Usage, regressions and authority limits](docs/protocol/prekey-count-notifications.md#offline-digest-inspection-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, regenerated CLI
reference/coverage report, formatting, JSON and repository hygiene passed.
No live traffic.

2026-11-04 — **Digest404/503 recovery and metadata scope traced.**

404 resync differs from503 attemptclear;resync SQL doesnot imply private
key deletion or authority to release uncertain Rust reservations.
[Evidence and port boundaries](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#digest-not-foundserver-error-and-metadata-resync-scope-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Digest local verifier and resync reasons traced.**

Ordered hashpreimage/SHA1-SHA256 selection retained; parsed digest isnot
proof of matching local material. Recovery integration remains unfinished.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#digest-local-material-verification-and-resynchronization-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Digest request/reply wire grammar traced.**

GETdigest writer,reply registration,requiredbytefields and20/32-bytehash
branches retained. Local digest matching/startup integration stillopen.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#digest-request-and-wire-callback-grammar-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **New-store startup and digest entry distinguished.**

Persisted new-store flag controls generate-or-send;separate digest flag
uses readiness/uptime guard. Rust inventory-only startup remains incomplete.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#new-store-startup-and-digest-entry-distinction-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Count notifications select migrated ADD or legacy SET.**

Actor preserves entry reason;PQ-only default-flag notifications remainACK-only,
independent of PQ upload-result demand. Uncertainty/refusal holds retained.
[Evidence, regressions and gaps](docs/protocol/prekey-count-notifications.md#count-notification-setadd-routing-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Migrated success demand uses ADD reservations.**

Legacy/PQ flags select families independently after uploaded-LR migration.
Exact ADD shape, default inventory/batching and frozen reservation integrated.
[Evidence, regressions and remaining gaps](docs/protocol/prekey-count-notifications.md#migrated-success-followup-add-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Legacy success-to-ADD admission edge proven.**

Both demand flags pass throughFcj/Fc8 toA0V after mark-sent work.
Current Rust followup strategy mismatch remains open for implementation.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-success-to-migrated-add-admission-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Refusal hold verified through actor/reopen.**

Count notifications remain ACKed; no replacement upload is written across
two actor epochs and exact-store reopen. Private prekey rows unchanged.
[Regression and limits](docs/protocol/prekey-count-notifications.md#refusal-hold-actor-verification-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Last-resort generation lifecycle traced.**

Newest unsentLR reuse/generation/cursor/storage traced; timestamp may precede
ACK and cannot establish migration. Current strategy port remains incomplete.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#last-resort-producer-and-pre-upload-timestamp-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **SQLite reset base and helper selection traced.**

Cleanup attempts database deletion; helper variants are flag-selected.
Full reset/reconstruction effects remain unproven; Rust hold retained.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#sqlite-reset-base-implementation-and-helper-selection-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Legacy refusal workers and reset boundary traced.**

406 legacy continuation differs from modern PQ SET;5xx scheduled work
rechecks admission. Full reset effects remain unknown; Rust hold retained.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-refusal-retry-workers-and-reset-boundary-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Saved prekey refusals hold implicit retries.**

Notifications/reconnect cannot reserve another upload after a saved refusal.
Status exposes the hold; native retry/backoff/regeneration remains unported.
[Source audit, local policy and regression](docs/protocol/prekey-count-notifications.md#durable-refusal-upload-hold-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **PQ count-low replies retained independently.**

New batch/outcome flag and historical acknowledged-batch status count preserve
PQ demand separately; automatic PQ replenishment remains incomplete.
[Evidence and regression](docs/protocol/prekey-count-notifications.md#independent-pq-count-low-retention-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **KEM upload IDs bound to original callback reservation.**

Changed durable KEM lists refuse settlement before any batch/key writes.
Existing uncertainty/consumption regression covers both reply kinds.
[Local invariant and limits](docs/protocol/prekey-count-notifications.md#kem-upload-reservation-binding-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No new retry behavior or live traffic.

2026-11-04 — **Upload settlement verifies durable reservation ownership.**

Ordered batch IDs and each key's pending owner/status are rechecked before
settlement writes. Fault/reopen regression covers stale ownership.
[Local invariant and limits](docs/protocol/prekey-count-notifications.md#upload-settlement-reservation-ownership-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No new retry behavior or live traffic.

2026-11-04 — **Upload refusals retain the native numeric code.**

Code-less children are skipped; first present code parses as signed Java
i32/default0, without truncation. Historical batches remain untouched.
[Evidence and regression](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#rust-coded-error-selection-and-numeric-conversion-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, CLI reference,
formatting, JSON, coverage freshness and repository hygiene passed.
No new retry behavior or live traffic.

2026-11-04 — **Upload reply fields follow selected callback.**

Results no longer retain error metadata; refusals no longer retain
count_low flags. Mixed-body regression verifies persistence after reopen.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#rust-reply-field-separation-2026-11-04).
Validation: workspace/all-feature tests, strict Clippy, regenerated CLI
reference, formatting, JSON, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Disabled-flag upload event registry proven.**

Successful startup registry initialization maps77/78 to14o even with
A0Q disabled. Earlier fallback disposition is now explicitly qualified.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#disabled-flag-registry-route-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Configured legacy IQ factory membership proven.**

Set7466 includes binding126 allocating1rC; constructor inlining explains
earlier zero caller search. Runtime routing value and lifecycle remain open.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#configured-legacy-iq-factory-membership-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Legacy fallback route narrowed.**

Selected cold-start fallback does not handle upload events77/78;
normal routing configuration and reconnect lifecycle remain unverified.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-fallback-event-route-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Legacy enabled-route manager binding traced.**

Provider6735 binds selectors77/78 to14o when A0Q is enabled.
Alternate routing and runtime configuration remain open.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-enabled-route-manager-binding-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Concrete legacy listener handoff traced.**

Normal setup binds1lZ to1lY. Conditional async routes retained;
final manager binding remains open.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-concrete-listener-handoff-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Legacy IQ handler registration boundary traced.**

Inherited handler declares `iq`; registration rejects duplicate tags.
Factory membership remains unknown after a bounded direct-constructor search.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-iq-handler-registration-boundary-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Legacy writer/reader reply-map alias traced.**

Normal setup passes one ConcurrentHashMap through both writer and reader
factory context; configured factory membership/lifetime remains open.
[Evidence and boundaries](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-writerreader-map-alias-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Legacy prekey reply removal and error event traced.**

Exact result/error removes the legacy map entry;290 error extracts code/text
and emits arg1=78. Generic callback assumption refuted.
[Evidence and map/listener limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-reply-removal-and-error-event-path-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Legacy SET writer and result-event path traced.**

Retained writer case arg1=85 confirms fullSET/op fields and separate reply map;
290/1ru result produces success arg1=77. Prior what85 wording corrected.
[Evidence and lifecycle limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-writer-shape-and-success-event-path-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Legacy SET snapshot and message85 handoff traced.**

A0S/A03/Fdv retains a separate legacy path from modern SET/ADD, including
reason-dependent PQ ensure argument and selected-array ownership.
[Evidence and writer/entry limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-set-snapshot-and-message85-handoff-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Native notification/daily prekey routing traced.**

43.72 distinguishes migrated ADD, legacy-only ADD and non-migrated SET;
daily missing-migration path reaches PQ-only SET. Worker ACK decision order
retained. [Evidence and remaining scope](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#notification-and-daily-pq-entry-routing-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Consumed KEM keys can settle later upload ACKs without resurrection.**

Atomic nonsecret inbox marker distinguishes legitimate consumption from missing
material. Stale allocator reuse refuses; rollback/reopen paths tested.
[Prior-guard correction and scope](docs/protocol/network-correctness-audit.md#consumed-kem-key-before-upload-ack-2026-11-04).
Validation: **1,285 workspace/all-feature tests**, targeted consumption checks,
strict Clippy, CLI reference, formatting, coverage freshness and repository
hygiene passed. No live traffic.

2026-11-04 — **KEM acknowledgement cannot silently skip lost key material.**

Settlement requires/validates every reserved KEM row. Missing-second-row
regression proves earlier writes and Batch status roll back across reopen.
[Details and scope](docs/protocol/network-correctness-audit.md#kem-acknowledgement-inventory-integrity-2026-11-04).
Validation: **1,285 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Uncertain KEM batches cannot ride a fresh upload again.**

Transactional hold prevents sent=false KEM rows being selected under a new IQ
with newly generated legacy keys. Full-store regression covers reopen and
release only by original settlement. [Details and limits](docs/protocol/network-correctness-audit.md#uncertain-kem-batch-replay-admission-2026-11-04).
Validation: **1,285 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **New prekey reservations freeze their encoded request.**

Wire payload/mode/dictionary checkpoint commits with keys and Batch. Reopen
regression proves unchanged provenance, no replay and no historical backfill.
[Scope and pending ADD strategy](docs/protocol/network-correctness-audit.md#frozen-prekey-request-provenance-2026-11-04).
Validation: **1,284 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **SET/ADD manager entry and migration predicates traced.**

Low-count ADD requires21198 plus a sent last-resort PQ row; PQ-only SET has
separate entry. Shared readiness/inflight/backoff guard retained.
[Evidence and upstream limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#setadd-entry-admission-and-migrated-state-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Prekey construction failures roll back upload reservation.**

Identity validation/KEM child signing moved inside the reservation transaction.
PQ-off/on regression proves complete key/checkpoint preservation and reopen,
then valid reservation/settlement. [Details and remaining ADD port](docs/protocol/network-correctness-audit.md#prekey-construction-and-reservation-atomicity-2026-11-04).
Validation: **1,283 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **PQ generation target and upload batch defaults distinguished.**

43.72 provider21200/default200 governs inventory;21201/default10 governs
batch. Matching REA registry reused; runtime values/lifecycle remain open.
[Evidence and Rust gap](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#pq-target-provider-and-declared-defaults-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Current-build PQ signing/wire provenance traced.**

ML-KEM-1024 enum and same-public-array signing/upload linkage retained.
Native byte lengths/prefix remain outside the Java wrapper proof.
[Evidence and Rust comparison](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#pq-signing-and-wire-byte-provenance-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **ADD ordinary-PQ selection/generation boundary traced.**

Non-last-resort SQL selection, supplied generation target, native-loader gate
and chunked cursor path retained; target/signing providers remain open.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#add-ordinary-pq-selection-and-generation-dispatch-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Native prekey cursor ownership and creation range traced.**

Self-row reader throws when missing; creation independently seeds positive
legacy/PQ cursors. Existing local history must not be reinterpreted or reseeded.
[Evidence and remaining schema/JNI paths](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#native-cursor-ownership-and-creation-seed-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Current-build legacy generation loop/cursor gap confirmed.**

43.72 fills the 812 deficit in <=50-key chunks; local generation stops at one
chunk and uses different cursor convention. Explicit store compatibility is
required before porting. [Evidence](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#legacy-generation-loop-and-cursor-convention-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **ADD prekey selection and exact child layout traced.**

43.72 SQL filter, key conversion/ID bytes and ADD-only wire shape retained.
Generation/configuration/durable mode remain open before runtime port.
[Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#add-selection-and-exact-child-layout-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Prekey success continuation ADD construction gap confirmed.**

43.72 Fcd selects ADD/configured batching; Rust follows older SET/812 layout.
Generation dispatch and callback binding retained; full shape/selection and
persistence compatibility remain open. [Evidence](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#successful-continuation-selects-add-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Limited prekey retry-mode writer scope traced.**

FDh config allowance is conditional, not a general startup default; visible
exception paths and 14o reset/consumption retained.
[Evidence and unknown lifecycle](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#limited-mode-producer-boundary-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Prekey retry boundaries and delay inputs traced.**

43.72 13z.A02 separates limited allowance from backoff counter bounds;
406/503 limited mode uses configured delay. Runtime gate/mode producers
remain open. [Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#retry-limits-and-delay-inputs-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Prekey storage effects and gated failure branches traced.**

43.72 result worker marks legacy/Kyber sent before completion. EcZ has
configuration-dependent 406/5xx branches; helper bounds and runnable effects
remain open. [Evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#store-effects-and-completion-policy-follow-up-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. No production change or live traffic.

2026-11-04 — **Current-build prekey reply callbacks retained.**

43.72 140/141 callbacks confirm optional first list/pq_list count_low parsing;
error extraction and result worker retained with downstream policy gaps explicit.
[Static evidence and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md).
Validation: APK identity, JSON parsing, coverage freshness, staged diff and
repository hygiene passed. No production change or live traffic.

2026-11-04 — **Retry refetch rechecks original age before resend.**

Completion and reservation enforce the existing strict 60-day limit. Shared
comparison avoids signed overflow; regression covers aged pending fetch and
boundary/extreme timestamps. [Evidence and approximation limits](docs/protocol/network-correctness-audit.md#retry-refetch-age-eligibility-recheck-2026-11-04).
Validation: **1,282 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Delivery received during key refetch suppresses retry resend.**

Completion rechecks acceptance/delivery and atomically guards the original send
row against changes during preparation. Real signed-bundle workflow covers both
delivered and undelivered outcomes. [Evidence and limits](docs/protocol/network-correctness-audit.md#retry-refetch-delivery-recheck-2026-11-04).
Validation: **1,281 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Retry refetch refuses to overwrite newer peer state.**

Identity/session are rechecked inside resend reservation after async preparation.
Regression independently changes each and proves newer state and pending fetch
are preserved. [Evidence and remaining concurrency scope](docs/protocol/network-correctness-audit.md#retry-refetch-peer-state-reservation-guard-2026-11-04).
Validation: **1,281 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Retry-refetch retirement commits atomically with its outcome.**

Separate early job deletion removed; refusal records or reserved replacement
send/session now share the retirement transaction. Fault regression proves
rollback retains pending work. [Details and native parity limits](docs/protocol/network-correctness-audit.md#retry-refetch-atomic-settlement-2026-11-04).
Validation: **1,280 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Media refresh admission and durable route preservation verified.**

Existing type/ID/tag/generation guard reviewed; new regression preserves full
checkpoint state across invalid replies and reopen, then applies a valid reply
idempotently. Production behavior unchanged.
[Evidence and remaining scope](docs/protocol/network-correctness-audit.md#media-refresh-settlement-admission-review-2026-11-04).
Validation: **1,280 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Keepalive waits for an actual result/error reply.**

Same-ID non-reply IQs neither terminate receive nor clear the outstanding ping.
Actor regression covers continued processing, later result, actual error and
unanswered timeout. [Evidence and scope](docs/protocol/network-correctness-audit.md#keepalive-reply-type-admission-2026-11-04).
Validation: **1,279 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Non-reply IQs no longer terminate activation as refused.**

Exact result/error admission now precedes activation completion/refusal.
Actor regression checks continued ping handling, capture, pending readiness,
unchanged account/journal, then valid activation. Actual error stays terminal.
[Evidence and limits](docs/protocol/network-correctness-audit.md#activation-reply-type-admission-2026-11-04).
Validation: **1,278 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Retry refetch preserves pending state for non-reply IQs.**

Exact result/error is admitted before loading/deleting a refetch checkpoint.
Encrypted-store regression covers reopen, unchanged retry/send/session history,
one genuine error and duplicate; the real signed-bundle result workflow also
remains available after non-replies.
[Evidence and scope](docs/protocol/network-correctness-audit.md#retry-refetch-reply-type-admission-2026-11-04).
Validation: **1,277 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Direct/group send preparation requires actual IQ reply types.**

Same-ID non-reply IQs no longer fail direct intents or consume group membership/
key waits. APK 43.72 0ln.A01 supports exact result/error callback admission.
Encrypted-store regressions retain reservations and original intent across reopen.
[Evidence, test scope and remaining gaps](docs/protocol/network-correctness-audit.md#send-preparation-reply-type-admission-2026-11-04).
Validation: **1,276 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Device-query non-reply IQs preserve uncertain state.**

Exact result/error now required before matching and ingestion, grounded in
43.72 0ln.A01. Regression covers full query preservation, pending users,
reopen and one valid settlement. Validation: **1274 workspace/all-feature tests**,
strict Clippy, CLI reference, formatting, coverage freshness and repository
hygiene passed. No live traffic.
[Evidence and limits](docs/protocol/network-correctness-audit.md#device-query-reply-type-admission-2026-11-04).

2026-11-04 — **Prekey upload no longer treats non-reply IQs as refusal:**

PendingUpload now requires matching iq/id and exact result/error before
settlement. Missing/get/set/uppercase/unknown types preserve the uncertain
batch and reserved keys instead of returning them to Unsent. Regression
checks complete batch preservation, reopen and one valid settlement.
[Native dispatch and source scope](docs/protocol/network-correctness-audit.md#prekey-upload-reply-type-admission-2026-11-04).
Validation: **1,273 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Timed-IQ writer admission and pre-write callback transfer traced:**

Concrete observer binding closes queued-callback check -> reply-map transfer
-> raw-node serialization -> output write/flush. Transfer precedes write;
local I/O requeue is not universal resend authority.
[Evidence and remaining queue/account lifetime](docs/protocol/rea/2.26.43.72-reachout-state/README.md#writer-callback-admission-before-output-stream-write-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. Static research only; no runtime change/live traffic.

2026-11-04 — **Disconnect callback cleanup traced; restriction reopen test strengthened:**

Native cleanup distinguishes queued callbacks from reply-map callbacks and
conditionally resets timers. Existing encrypted-store regression now reopens
while restriction-held and after sent reservation, proving preserved intent
and no replay across those boundaries. Production behavior unchanged.
[Evidence and crash/account-lifecycle limits](docs/protocol/rea/2.26.43.72-reachout-state/README.md#disconnect-callback-cleanup-and-durable-reopen-regression-2026-11-04).
Validation: **1,272 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage freshness and repository hygiene passed. No live traffic.

2026-11-04 — **Refresh pending-callback and timeout ownership traced:**

Native registration precedes send handoff; 32000 delay is milliseconds.
Locked pending/reply map removals and timeout callback route retained.
These are in-memory lifetimes, not durable request/account ownership.
[Evidence and remaining writer/teardown work](docs/protocol/rea/2.26.43.72-reachout-state/README.md#pending-iq-registration-dispatch-and-timeout-ownership-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. Static research only; no runtime change/live traffic.

2026-11-04 — **Pinned JSON decoder and data/errors failure matrix traced:**

DI key 4417 resolves 0iz. Configuration selects stream/JSONTokener paths;
empty input and data/errors extraction defaults/failures are retained.
Malformed errors with good data synthesize an error array, suppressing
refresh application. [Evidence and remaining work](docs/protocol/rea/2.26.43.72-reachout-state/README.md#pinned-json-decoder-branches-and-extraction-failures-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. Static research only; no runtime change/live traffic.

2026-11-04 — **Refresh response ID/sender equality and JSON selection traced:**

Selected success constructor compares response id/from with original id/to,
requires one result, and checks parent result type. JSON alternative can
return its body despite a discarded format-check failure; decoding and
callback admission still apply. [Evidence and limits](docs/protocol/rea/2.26.43.72-reachout-state/README.md#request-bound-equality-and-json-alternative-error-propagation-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. Static research only; no runtime change/live traffic.

2026-11-04 — **Fetch accessor defaults and Argo/JSON selectors resolved:**

Pinned JSONObject fetch requires its nested object, uses a coercing/defaulted
boolean, and nullable string access. Push notification accessors remain
separate. Body selector domains mapped through 8TV/7NM/7jy to parser type.
[Evidence and remaining decoder/correlation work](docs/protocol/rea/2.26.43.72-reachout-state/README.md#fetch-field-defaults-and-body-format-selectors-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. Static research only; no runtime change/live traffic.

2026-11-04 — **Refresh callback data/error admission traced:**

Pinned request-bound success/request-error/server-error constructors and
typed callback chain retained. Refresh error callback returns false,
suppressing partial data application; empty typed data also declines.
[Evidence and remaining body/correlation/default semantics](docs/protocol/rea/2.26.43.72-reachout-state/README.md#iq-response-conversion-and-refresh-callback-admission-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. Static research only; no runtime change/live traffic.

2026-11-04 — **Normal refresh executor registry-to-asset chain resolved:**

0ig registers MEX through 1Wy/0ii/0im; CRh reads the asset's server doc_id.
Separate client-ID gates do not select that method's result. Alternate
registrants and startup ordering remain open. Automatic refresh unported.
[Registration and exact remaining scope](docs/protocol/rea/2.26.43.72-reachout-state/README.md#executor-registration-and-asset-backed-server-id-lookup-2026-11-04).
Validation: JSON parsing, persisted-ID provenance check, coverage freshness,
staged diff and repository hygiene passed. No runtime change/live traffic.

2026-11-04 — **Restriction refresh request traced through XMPP selection:**

Pinned empty variables/spec, non-Pando selection, XMPP construction and
bounded persisted-ID asset export retained. Runtime registry fallback is
null: asset presence alone does not establish a dispatchable native query.
[Construction and remaining persistence/response obligations](docs/protocol/rea/2.26.43.72-reachout-state/README.md#refresh-request-construction-and-runtime-document-lookup-2026-11-04).
Validation: persisted-ID provenance --check, JSON parsing, coverage
freshness, staged diff and repository hygiene passed. Static research only;
automatic refresh remains unported and no live traffic was sent.

2026-11-04 — **Previously queued group creates recheck restriction at reservation:**

A create queued before an active update no longer bypasses the queue-time
guard. Bulk/actor/selected reservation checks recorded active in the same
store transaction before marking sent. Blocked rows retain original intent;
unrelated actions proceed. Explicit inactive releases the same create once.
Already-reserved uncertain actions are never reset or replayed.
[Source audit and regression](docs/protocol/network-correctness-audit.md#restriction-recheck-at-action-reservation-2026-11-04).
Validation: **1,272 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage and repository hygiene passed. No live traffic.

2026-11-04 — **Elapsed deadline no longer releases the group-create guard:**

Pinned countdown completion fetches server restriction state rather than
clearing active locally. Group creation now honors the recorded active flag
until an explicit inactive update; the displayed deadline is diagnostic.
Real-store regression checks no action is queued/refused state is retained,
then verifies explicit inactive allows queueing. Automatic fetch remains
unported; a stale active record can continue refusing.
[Evidence and limits](docs/protocol/rea/2.26.43.72-reachout-state/README.md#deadline-is-a-refresh-trigger-not-an-inactive-update-2026-11-04).
Validation: **1,271 workspace/all-feature tests**, strict Clippy, CLI reference,
formatting, coverage and repository hygiene passed. No live traffic.

2026-11-04 — **Outreach caller semantics: call preflight and sticker refusal:**

Pinned 1v7 call branch and ShareMediaViewModel sticker refusal are traced;
Jmt inverts aggregate eligibility. Corrected earlier message shorthand:
0wj is obtained as a JID-bound cached record, not a wire message payload.
[Evidence, branches and remaining port obligations](docs/protocol/rea/2.26.43.72-reachout-state/README.md#caller-semantics-call-gate-and-sticker-refusal-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. Static evidence only; no runtime changes or live traffic.

2026-11-04 — **Native outreach decision and recipient-set provenance traced:**

Pinned 6po cached active/visible checks, 6pn per-message decision, trusted-
contact set population and a higher decision caller are retained. Rust's
group-create guard is not full native outreach coverage. Cache/expiry
lifetimes, remaining caller meanings and account predicates remain open.
[Evidence and exact scope](docs/protocol/rea/2.26.43.72-reachout-state/README.md#outreach-decision-chain-and-recipient-set-2026-11-04).
Validation: JSON parsing, coverage freshness, staged diff and repository
hygiene passed. Static research only; no runtime changes or live requests.

2026-11-04 — **Pinned restriction enum and stored types resolved:**

Retained all 22 named enum ordinals, internal boxed values and the distinct
saved-type mapping. Banner consumer traced; another reader discards its
result. Complete send/companion policy remains open; no runtime changes.
[Table and bounded consumer evidence](docs/protocol/rea/2.26.43.72-reachout-state/README.md#enforcement-names-and-stored-type-mapping-2026-11-04).
Validation: APK checksum, JSON parsing, coverage freshness, staged diff and
repository hygiene passed. No live account operations.

2026-11-04 — **Reach-out expiry uses Java decimal string rules:**

Active expiry strings reuse the existing Java-style BMP decimal reader;
nonnegative local bounds, JSON integer compatibility and inactive clearing
remain. Regression verifies accepted strings and prior-lock preservation.
[Evidence and limits](docs/protocol/rea/2.26.43.72-reachout-state/README.md#json-adapter-and-java-decimal-expiry-2026-11-04).
Validation: **1,270 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic or complete enforcement-policy parity claim.

2026-11-04 — **Malformed recognized reach-out bodies refuse explicitly:**

Missing body/bad JSON/missing or nonboolean active now refuse before storage,
using existing capture/no-ACK/continue handling. Unknown operations remain
separate. Real-store regression preserves prior lock across malformed bodies.
[Scope](docs/protocol/network-correctness-audit.md#recognized-reach-out-body-refusal-2026-11-04).
Validation: **1,269 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic or full native error-ACK parity claim.

2026-11-04 — **Offline device classifier branches resolved:**

Retained exact category table and nullable/user/device conversion. Selected
values route high-offline callbacks; they do not establish universal sender
trust. Account predicate semantics and sender authority remain open.
[Evidence/table](docs/protocol/rea/2.26.43.72-reachout-state/README.md#offline-device-classification-table-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,441 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Notification offline preflight/map construction traced:**

Retained conditional offline>=11 JID classification and duplicate-checked
tag registration mechanics. Neither establishes universal MEX origin authority;
classifier details/configured factory ownership remain open.
[Evidence](docs/protocol/rea/2.26.43.72-reachout-state/README.md#high-offline-preflight-and-handler-map-construction-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,433 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Legacy notification-to-MEX code bound:**

Fallback tag-map wrapper and1rc notification branch connect literal mex
to0x102, matching router registration. Separated message-only JID checks
from notification authority; map/consumer and sender details remain open.
[Evidence](docs/protocol/rea/2.26.43.72-reachout-state/README.md#legacy-notification-to-mex-code-binding-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,424 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Receive gate classified as persistence policy:**

Pinned offline-count/buffering and consumer-readiness helpers feed wrapper
flags; they are not sender-authentication evidence. Queue scheduler false
lead retained; legacy fallback and buffering lifetime remain open.
[Evidence](docs/protocol/rea/2.26.43.72-reachout-state/README.md#receive-gate-is-persistence-not-origin-validation-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,415 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **MEX receive registry discovery retained:**

Located type/operation maps and gated receive wrapper dispatch; connection
to every legacy route remains untraced. Refuted an IQ-result schema overload
as notification-origin evidence. Sender authentication remains open.
[Evidence/negative results](docs/protocol/rea/2.26.43.72-reachout-state/README.md#receive-registry-discovery-and-schema-false-lead-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,409 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **MEX message/worker ordering boundary traced:**

Pinned router registration and wrappers now connect to native handler worker.
Reach-out inherits a no-op ordering reservation; direct/executor dispatch is
retained separately from origin checks. Upstream sender authority remains open.
[Evidence](docs/protocol/rea/2.26.43.72-reachout-state/README.md#message-wrapper-and-reach-out-ordering-reservation-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,400 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Reach-out envelope validation precedes state writes:**

Missing/invalid ACK routing fields cannot change restriction state before
later ACK refusal. Existing validation now precedes mutation; actual ACK
still follows successful persistence. Sender authority remains separate/open.
[Evidence and real-store regression](docs/protocol/rea/2.26.43.72-reachout-state/README.md#base-dispatch-and-local-envelope-before-state-guard-2026-11-04).
Validation: **1,268 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live account traffic.

2026-11-04 — **Reach-out MEX operation routing aligned:**

Only first update can supply restriction state; supported operation matching
is ASCII case-insensitive, following pinned router behavior. Wrong tags and
later matching updates no longer mutate state. Sender authority remains open.
[Evidence/tests](docs/protocol/rea/2.26.43.72-reachout-state/README.md#current-mex-operation-routing-2026-11-04-offline-corrected).
Validation: **1,267 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live account traffic.

2026-11-04 — **Explicit inactive reach-out update aligned with pinned handler:**

Native7vL resets inactive state without reading expiry. Rust now permits
that release; malformed active updates still preserve the prior restriction.
Retained dispatch/state/parser and feature-policy limitations.
[Evidence](docs/protocol/rea/2.26.43.72-reachout-state/README.md).
Validation: **1,266 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic; sender authority/config policy remain untraced.

2026-11-04 — **Malformed reach-out expiry no longer clears local lock:**

Known expiry updates now refuse missing/malformed/negative/overflow values
before write. Receive preserves capture/prior lock, skips ACK and continues;
storage failures still propagate. Valid expiry behavior stays covered.
[Source defect and real-store regression](docs/protocol/network-correctness-audit.md#malformed-reach-out-expiry-preservation-2026-11-04).
Validation: **1,266 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live account traffic or native timestamp grammar claim.

2026-11-04 — **Upload ordinary path query fields corrected (offline):**

New reply acceptance now requires native oe/oh query presence without
rewriting accepted paths or interpreting expiry/signature. Missing fields
refuse payload construction after encrypted reply retention; no upload replay.
[Evidence and regression cases](docs/protocol/rea/2.26.43.72-media-upload-response/README.md#ordinary-path-query-presence-correction-2026-11-04-offline).
Validation: **1,265 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live upload or server acceptance test.

2026-11-04 — **Upload scheduled-cross-reference mode identified:**

Pinned writer/consumer evidence binds UUID handling to scheduled cross-reference
adoption, separately from ordinary path. Mode flags and whole-string match
semantics are traced; ordinary normalization and higher metadata input remain open.
[Evidence](docs/protocol/rea/2.26.43.72-media-upload-response/README.md#mode-producers-and-scheduled-cross-reference-consumer-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,366 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Upload reply mode boundary traced:**

Retained native callback's optional fields, error flag and mode-dependent
opaque-handle/direct-path split. UUID pattern must not be imposed on slash
paths. Runtime remains the local HTTP200+URL/path subset; native mode
producers and downstream field requirements remain open.
[Evidence](docs/protocol/rea/2.26.43.72-media-upload-response/README.md).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,356 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Sealed voice checkpoint continuity verified offline:**

Real-store regression checks exact sealed request fields after header receipt
and reopen, refusal of replay and legacy completed payload reuse without
rewriting storage. No additional runtime mismatch found in this checkpoint pass.
[Scope](docs/protocol/media-network-outcomes.md#sealed-voice-checkpoint-reopen-regression-2026-11-04).
Validation: **1,264 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No runtime behavior change or live traffic.

2026-11-04 — **Finished voice upload seal corrected (offline):**

New primary one-shot voice uploads append native sent/hash metadata with
Android slash escaping, pinned framing/size limit and separate cipher hash.
New reservations retain exact wire bytes/version/size; old reservations are
neither rewritten nor replayed. Other media wire bytes stay unchanged.
[Evidence and regression scope](docs/protocol/rea/2.26.43.72-media-upload-http/README.md#finished-voice-seal-integration-2026-11-04-offline-validated).
Validation: **1,263 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live upload or server acceptance test.

2026-11-04 — **Standard finished voice seal path bound:**

Factory/interface evidence closes custom processor override for standard
LKZ. Native finished non-newsletter voice appends a seal absent in Rust;
exact JSON/wire serialization and saved-reservation compatibility remain
the implementation work.
[Evidence](docs/protocol/rea/2.26.43.72-media-upload-http/README.md#standard-finished-voice-processor-binding-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,343 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Standard voice seal digest boundary traced:**

Pinned standard non-chunk encrypted stream hashes ciphertext+10-byte MAC,
then appends metadata outside the digest. Retained constructor field mapping,
stream/filter implementations and branch limits. Higher-level operation and
processor override binding remain open before changing upload bytes.
[Evidence](docs/protocol/rea/2.26.43.72-media-upload-http/README.md#digest-stream-includes-cipher-mac-excludes-transport-seal-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,332 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Finished voice flags and seal hash formatting traced:**

Retained finished plugin construction, complete boolean writer census and
digest-to-Base64 flag2 mapping. Exact digest stream and operation/override
binding remain open before fixing the potential missing voice seal.
[Evidence](docs/protocol/rea/2.26.43.72-media-upload-http/README.md#finished-voice-construction-and-digest-formatting-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,322 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Voice seal owner and encoding helpers traced:**

Default LKZ processor excludes newsletter-ptt and permits custom override.
Closed UTF-8/little-endian helpers and retained final ciphertext-hash leaf.
Rust has no separate voice seal: potential omission needs exact finished-file
constructor/override/hash binding before a correct wire change.
[Evidence](docs/protocol/rea/2.26.43.72-media-upload-http/README.md#seal-owner-excluded-kind-and-encoding-helpers-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,311 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Optional upload EOF processor boundary traced:**

Closed single-file predicate and retained null-versus-specialised seal
processing, factory leaves and eligibility gate. Concrete operation binding,
kind/encoding helpers and full seal bytes remain untraced.
[Evidence](docs/protocol/rea/2.26.43.72-media-upload-http/README.md#optional-eof-processor-and-single-file-predicate-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,302 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Route-bound raw upload body branch traced:**

Pinned body writer confirms raw single-stream versus multipart conditions,
stream offset/length mappings, chunk copy and optional EOF processing.
Supports raw request shape; trailer/resume provenance remains open.
[Evidence](docs/protocol/rea/2.26.43.72-media-upload-http/README.md#raw-stream-versus-multipart-body-writer-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,293 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Upload HTTP transport/header boundary traced:**

Retained pinned transport selection, route-auth construction and concrete
JAS/JAT implementations. JAS conditional headers/body modes distinguish
annotations from HTTP fields. Active-path/input provenance remains open;
no mandatory-header mismatch is established for Rust's supported upload.
[Evidence](docs/protocol/rea/2.26.43.72-media-upload-http/README.md).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,285 local links) passed. Documentation/evidence only;
no runtime change or live traffic.

2026-11-04 — **Upload bytecode mnemonic gaps closed:**

Developer disassembler now names sget-boolean and mul-float/2addr. Two
regressions check operand decoding, boundaries and truncated prefixes.
Regenerated pinned upload method has no unnamed instructions.
[Evidence](docs/protocol/rea/2.26.43.72-media-transfer-results/README.md#checked-upload-mnemonic-correction-2026-11-04).
Validation: **1,261 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No WhatsApp runtime behavior changed or live traffic sent.

2026-11-04 — **Media result flags and cancellation entry points traced:**

Retained result factory matrix and distinguished MS4 default false predicate
from upload's caller-supplied predicate. Auth retry precedes terminal result
handling; exact HTTP leaf values are documented without a generic retry rule.
[Evidence](docs/protocol/rea/2.26.43.72-media-transfer-results/README.md).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,271 local links) passed. Documentation/evidence only;
no Rust change or live traffic.

2026-11-04 — **Media auth readiness and distinct delay suppliers traced:**

CW7 pre-attempt refresh uses the shared wait budget and separate auth
thresholds. Mtv exponential delay and LIY inherited fixed delay are distinct;
retained one wrapper construction branch. Operation replay and remaining
configuration producers stay open.
[Evidence](docs/protocol/rea/2.26.43.72-media-http-auth-refresh/README.md#readiness-and-distinct-delay-suppliers-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,256 local links) passed. Documentation/evidence only;
no Rust change or live traffic.

2026-11-04 — **Media auth callback continuation traced:**

Pinned static evidence now reaches MS4.A03 through Nd8.BkV: true retry
eligibility goes through delay, route reload and cancellation/readiness checks
before another transfer callback. The supplier enable flag is constructor-fed.
Operation-specific replay rules and delay/readiness policy remain untraced.
[Evidence](docs/protocol/rea/2.26.43.72-media-http-auth-refresh/README.md#transfer-loop-continuation-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,244 local links) passed. Rust source unchanged.
Documentation/evidence only; no runtime change or live account traffic.

2026-11-04 — **Conditional media HTTP auth refresh callers traced:**

Retained all three direct401/403 route-reporting callers, changed-auth
predicate, two-retry cap and elapsed wait-budget accounting. Caller loops,
flag values and transfer loops remain untraced; no generic resend enabled.
[Evidence](docs/protocol/rea/2.26.43.72-media-http-auth-refresh/README.md).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,235 local links) passed.
Documentation/evidence only, no Rust change or live traffic.

2026-11-04 — **Media disconnect cancellation and demand triggers traced:**

Pinned APK static evidence closes message0 cancellation in1iM.A02 and
distinguishes two-minute demand checks from the one-minute auth timer.
HTTP401/403 invoke the existing request gate; caller provenance remains open.
[Retained output and scope](docs/protocol/rea/2.26.43.72-media-retry-scheduling/README.md#disconnect-cancellation-and-additional-triggers-2026-11-04).
Validation: JSON parsing, coverage freshness, cached diff checks and repository
hygiene (4,223 local links) passed. Documentation/evidence only; Rust unchanged.
No live traffic or automatic scheduling enabled.

2026-11-04 — **Native media retry/timer trace and checked arithmetic:**

Checked DEX closes the Fibonacci leaf and Handler dispatch: base0,1,1,2,... caps
at987 seconds; positive-base jitter may exceed that base cap. The same message0
request gate serves error retry and auth-expiry-minus60s scheduling. Successful
route installation/merge resets the Fibonacci state. Full lifetime/cancellation
and other trigger producers remain untraced; no automatic runtime loop is added.
The disassembler now names both missing arithmetic opcodes, with two literal-byte
width/register/boundary regressions. [Files, hashes and provider limits](docs/protocol/rea/2.26.43.72-media-retry-scheduling/README.md).
Validation: **1,259 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic. Remote CI passed the two preceding recovery chunks; latest run
was still in progress at this pass's check.

2026-11-04 — **Legacy media refusal recovery observations (offline):**

Recovery now recognizes the old producer's correlated outer-error captures and
original receipt times without rebuilding request/prior/order context. Possible
active507 bounds block new preparation; an unordered legacy503 cannot clear them.
Corrupt/mismatched captures fail explicitly. The new offline media-route-recover
command reports safe counts/deadlines and never logs in or creates a missing store.
[Producer provenance, two real-store tests and interpretation limits](docs/protocol/media-network-outcomes.md#legacy-refusal-observations-and-offline-command-2026-11-04).
Validation: **1,257 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic. Historic build/clock provenance and native retry ownership remain
unproved.

2026-11-04 — **Bounded media wait recovery (offline):**

Explicit refresh preparation now restores known waits from validated receipt order
and original timestamps before reserving another query. Missing/duplicate receipt
history refuses partial repair. Active unsequenced waits remain ambiguous and
block preparation through their original bound; records are never restamped.
Existing uncertain deadlines are preserved rather than shortened. Safe recovery
counts expose remaining unbound legacy history without raw response data.
[Contract, four real-store tests and remaining gaps](docs/protocol/media-network-outcomes.md#bounded-historical-wait-recovery-2026-11-04).
Validation: **1,255 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic. Unbound legacy recovery and native retry coordination remain open.

2026-11-04 — **Durable media refresh receipt order (offline):**

New settlements now receive an atomic positive sequence across result, refusal,
unsupported and stale outcomes. It commits with raw/effective reply, current
pointer and throttle effects. Duplicates cannot restamp; lost/zero/exhausted
counters refuse and transaction cuts roll everything back. Legacy records remain
unsequenced. The existing CLI reports receipt_order without private response data.
[Tests and recovery boundary](docs/protocol/media-network-outcomes.md#durable-refresh-receipt-order-2026-11-04).
Validation: **1,251 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic. Historical throttle reconciliation remains unfinished.

2026-11-04 — **Explicit media refusal throttle (static/offline):**

Pinned 1xY.Bvj code507 positive backoff is seconds capped at 10800. Its deadline
now commits with the refusal receipt and blocks new explicit refresh preparation.
Duplicates cannot extend it; native-qualified 503 and applied success clear it.
No generic delay or automatic query is invented. Read-only deadline inspection
and cap/reopen/expiry/selection/reset tests retain the original error evidence.
[Native source, arithmetic caveat and local policy](docs/protocol/media-network-outcomes.md#explicit-media-throttle-2026-11-04).
Validation: **1,248 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic. The native Fibonacci retry path and full volatile owner remain
unfinished.

2026-11-04 — **Prospective media refresh integration (offline):**

The explicit CLI route query now freezes prior applied state/request bytes,
conditionally sends last_id, and commits original reply, derived host merge and
current pointer atomically. Bound non-new and first-route false replies are
supported; legacy unbound histories are not reinterpreted. Stale/conflicting
settlements cannot overwrite the current route. Upload selection follows the
applied pointer with expiry checks, avoiding timestamp/random-ID ordering.
[Tests, provenance and lifecycle limits](docs/protocol/media-network-outcomes.md#prospective-refresh-integration-2026-11-04).
Validation: **1,246 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic. Native flight/throttle/startup-restore and full host policy remain
unfinished; no automatic query or retry is introduced.

2026-11-04 — **Native media host projection core (offline):**

A pure projector now selects first/new reply hosts or exact prior hosts for a
non-new update, always using the incoming auth/lifetime/ID attributes. Cached
upload selection uses the shared parsing/projection for its supported branch.
Three branch/metadata/immutability tests exercise the merge operation. Durable
prospective prior-context capture and atomic refresh projection remain unfinished;
non-new cached replies still refuse rather than borrowing arbitrary prior state.
[Implemented core and limits](docs/protocol/media-network-outcomes.md#pure-host-projection-implemented-2026-11-04).
Validation: **1,239 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Conditional media host retention (static/offline):**

Pinned 0VM.A0A retains old hosts for a non-new response when a current route
exists, while updating auth/lifetimes. Without a current route it installs the
whole reply. Rust ignored is_new; it now explicitly refuses non-new cached
replies pending prospective prior-host binding/merge, preserving the original
response without selecting a potentially incorrect host list. This is a local
support limit, including the native first-route false branch, not full parity.
[Evidence and required merge](docs/protocol/media-network-outcomes.md#conditional-prior-host-retention-2026-11-04).
Validation: **1,236 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic. Full refresh coordination remains unfinished.

2026-11-04 — **Media route request-time expiry (static/offline):**

Pinned 0VM.A07 writes the monotonic request handoff time consumed by the media
lifetime callback. Rust's result commit previously replaced its reservation time
with receipt time, extending cache freshness by response delay. The CLI now
preserves requested_at and upload expiry requires it. Legacy/missing timestamps
require explicit refresh; no response-time guess or automatic query is made.
[REA, checked DEX and field census](docs/protocol/rea/2.26.43.72-media-route-lifetimes/README.md#request-clock-origin-follow-up).
Delayed-receipt tests cover expired, missing, future and zero request times.
Local wall-clock reservation is conservative; full native throttle/refresh/host
policy remains open. Validation: **1,235 workspace/all-feature tests**, strict
Clippy, CLI-reference, formatting, coverage freshness and repository hygiene passed.
No live traffic.

2026-11-04 — **Pinned media lifetime parser correction (static/offline):**

APK 2.26.43.72 1xY.CK4 requires both route ttl and auth_ttl via the Java-long
reader. This refutes the prior local missing-auth fallback, now removed. Both
Rust lifetimes use the existing Java-compatible helper. Encoded-route tests
refuse missing/invalid/expired values and accept supported BMP digit spellings.
[Six retained REA methods, checked DEX and limits](docs/protocol/rea/2.26.43.72-media-route-lifetimes/README.md).
The native constructor confirms millisecond deadline arithmetic; conditional
cache restore checks auth expiry. Base origin and complete refresh/host policy
remain open. Validation: **1,234 workspace/all-feature tests**, strict Clippy,
CLI-reference, formatting, coverage freshness and repository hygiene passed.
No live traffic or automatic retry/query.

2026-11-04 — **Explicit media auth lifetime validation (offline):**

Upload route selection no longer treats a present malformed/overflowing auth_ttl
as absent. Only absence falls back to route TTL. Validity/expiry refusals remain
read-only and issue no refresh or upload request. Real-store encoded-route tests
cover valid, absent, malformed, overflow, zero, negative and expired auth TTLs.
[Source defect and current-build limits](docs/protocol/media-network-outcomes.md#explicit-auth-lifetime-validation-2026-11-04).
Validation: **1,234 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic or native expiry-policy parity claim.

2026-11-04 — **Upload response-header persistence (offline):**

Upload status is now stored before response-body streaming. A truncated or
oversized response leaves response_headers_received plus its numeric status,
rather than erasing an observed response behind outcome_unknown. This is not
upload success and does not authorize replay. Tests cover reopen, bound material
preservation, replay refusal and protection of existing settled rows.
[Source defect and transition](docs/protocol/media-network-outcomes.md#upload-response-header-persistence-follow-up).
Validation: **1,233 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic or native HTTP parity claim.

2026-11-04 — **View-once retrieval preflight (offline):**

View-once GET now validates config and registered-account/phone agreement before
selection, cache return or reservation. Its descriptor must contain both 32-byte
hashes, avoiding a one-shot download that cannot later authenticate plaintext.
Tests cover account/config refusal without checkpoint changes and malformed hashes.
[Source defects and boundaries](docs/protocol/media-network-outcomes.md#view-once-preflight-2026-11-04).
Validation: **1,232 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live traffic or native HTTP parity claim.

2026-11-04 — **Ordinary media HTTP refusal persistence (offline):**

A definite non-200 media GET no longer remains labelled only outcome unknown.
Its numeric status is committed under server_refused and reported on later
invocations without another request. The transition requires the original
reservation and cannot overwrite settled/downloaded state. Transport and
verification uncertainty retain their existing reservations.
[Source evidence, tests and current-build gaps](docs/protocol/media-network-outcomes.md).
Validation: **1,230 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live account traffic occurred; native media header/timing parity remains open.

2026-11-04 — **Java decimal settlement parity (offline):**

Accepted-write acknowledgements and conflict codes now reuse the APK-compatible
signed-decimal helper rather than Rust unsigned parsing/literal ASCII comparison.
Prepared-state, exact-version, baseline and authentication checks remain required.
Tests cover alternate BMP digit spellings plus malformed/mismatched values,
checkpoint preservation, authenticated promotion and no replay.
[Evidence and limits](docs/protocol/app-state-patch-atomicity.md#java-decimal-acknowledgement-and-conflict-spelling-2026-11-04).
Validation: **1,229 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live account traffic occurred; full fetch application remains unfinished.

2026-11-04 — **409 whole-collection preflight (offline):**

Conflict catch-up now uses the current structural classifier/header stage before
held-state loading or key resolution. An accompanying snapshot reference can no
longer be ignored while inline patches advance state; unsupported external data
refuses. All selected nested patch headers are checked before verification.
Real-store tests preserve the entire checkpoint table and empty journal for
snapshot, duplicate-version and mixed-source refusals.
[Defect, evidence and limits](docs/protocol/app-state-catchup-atomicity.md#whole-collection-preflight-before-conflict-verification-2026-11-04).
Validation: **1,227 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live account traffic occurred; successful-fetch application remains unfinished.

2026-11-04 — **409 verified-action retention (offline):**

Conflict catch-up no longer discards decrypted actions after verifying and
advancing integrity state. Ordered SET/REMOVE actions are retained encrypted,
bound to the action, collection, version, generation and exact stored wire-pair
fingerprints. Journal, held checkpoint and pending-write release commit together;
a later bad patch rolls them all back. Reopen and repeat ingestion preserve the
original journal without replay. [Evidence, tests and limits](docs/protocol/app-state-catchup-atomicity.md#authenticated-action-retention-2026-11-04).

This preserves the existing whole-chain utility contract. Native per-patch commit
parity, successful held-fetch processing and action projection remain unfinished.
Validation: **1,226 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live account traffic occurred.

2026-11-04 — **Stored held-fetch response provenance (offline):**

Response inspection now requires sealed prospective admission for successful
held-fetch actions and checks phase, settlement order, exact original wire,
generation and preserved reply in one consistent read before structural binding.
`admission_bound` distinguishes binding success/refusal from legacy/write/error
scope. Missing or corrupt required provenance fails; structural refusal preserves
safe response diagnostics. A later changed held state does not rewrite historical
provenance, and version acknowledgements cannot advance that state. MAC verification
and per-patch current-state/cursor/journal commits remain unfinished.
[Boundary and tests](docs/protocol/app-state-held-fetch-admission-design.md#preserved-response-provenance-inspection).
Validation: **1,225 workspace/all-feature tests**, strict Clippy, CLI-reference,
formatting, coverage freshness and repository hygiene passed.
No live account traffic occurred.

2026-11-04 — **Durable prospective held-fetch admission (offline):**

Explicit `contacts fetch-app-state --held` captures existing validated collection
checkpoints, exact queued request bytes and account generation atomically.
Reservation rechecks the complete baseline, pending-write ownership and known
server backoff before marking the action sent. Baseline drift causes a durable
local refusal; corrupt admission data remains an error. Legacy fetches receive
no retrospective baseline. `actions --show` exposes safe admission/status fields
from one store read. [Design and limits](docs/protocol/app-state-held-fetch-admission-design.md).

An encrypted, transaction-scoped authenticated-mutation journal primitive now
has offline tests for retention, conflicts, rollback and generation fencing.
It has no production caller yet: response verification, per-patch cursor/state
application, external downloads and native flight ownership remain unfinished.
Validation: **1,223 workspace/all-feature tests**, strict all-target/all-feature
Clippy and CLI-reference checking passed. No live account traffic occurred.

2026-11-04 — **Bound fetch candidates and whole-collection headers:**

Pure fetch binding now checks exact encoded request identity, local generation
claims, explicit requested versions and complete validated baseline/domain
agreement. It reclassifies and compares public response models before accepting
their summaries. Fetch acknowledgements cannot advance held state; missing,
snapshot, external-mutation and refusal cases stay distinct. Request-order
membership retains original response indices for a future native-order worker.
These are admission claims, not proof of store origin or current state equality.

Recursive header inspection parses all selected patch protobufs before semantic
checks, separates native fatal reasons from local restrictions, and defers
external data. The existing crypto decoder is unchanged. The offline response
inspector now exposes safe header outcomes, versions and counts with integrity
still false. No admission record, key lookup, MAC verification, state/cursor
commit or network operation is introduced by these pure stages.
[Binding](docs/protocol/app-state-inline-fetch-application-spec.md#implemented-pure-binder)
and [headers](docs/protocol/app-state-structural-headers.md).

Validation: **1,205 workspace/all-feature tests**, strict all-target/all-feature
Clippy, CLI-reference checking, formatting, coverage freshness and repository
hygiene passed. No live account traffic occurred. Durable admission capture and
authenticated cursor/state application remain unimplemented.

2026-11-04 — **Patch header audit and inline application specification:**

The suspected silent external/exit-field decoder bug is refuted: current patch
and prepared-wire decoders reject those fields before crypto. Seven checked
current-build exports independently pin the field tags. A future fetch worker
still needs whole-collection protobuf/header preflight before per-patch commits;
numeric exit outcomes, external-only deferral and duplicate-version handling
must remain separate from MAC verification.
[Audit and next structural stage](docs/protocol/app-state-patch-header-audit.md).

The prospective inline-application contract now pins original request/version
and admitted held baseline, real store-generation fencing, per-patch atomic
cursor/state commits, restart behavior and unported external/continuation gates.
A pure fetch version acknowledgement cannot create an authenticated held hash.
[Implementation specification](docs/protocol/app-state-inline-fetch-application-spec.md).
This chunk changes evidence/design only; no account state or network activity.

2026-11-04 — **Structural app-state result classification:**

The current-build response category model now distinguishes collection errors,
version acknowledgements, already-synced names and opaque snapshot/patch data in
original order. Snapshot bodies are external blob references, not snapshots.
Unknown names/children remain in borrowed originals, with redacted Debug output.
Offline `app-state-response-inspect ACTION_ID` reports categories/counts/scalars
from one preserved correlated result without changing state or downloading data.
Its `integrity_verified` value is always false. Protobuf semantic checks, keys,
MACs, requested-collection admission, generation and result application remain
required work. [Implementation and limits](docs/protocol/app-state-fetch-completion.md).

New evidence traces external download authentication precisely (without claiming
every downstream encrypted-hash check) and authenticated reader/certificate gates
plus primary Me creation/persistence. Logout/switch generation fencing remains
open. [Session/source evidence](docs/protocol/session-source-me-current.md).
No live account operation occurred.

Validation: **1,195 workspace/all-feature tests**, strict all-target/all-feature
Clippy, CLI-reference regeneration and built-command help, formatting, coverage
freshness and repository hygiene passed. Retained accounting is 932 REA records,
576 for the current APK, 72 explicit ownership mappings; no completion percentage
or account-safety claim follows from those counts.

2026-11-04 — **Explicit fetch refusal waits:**

App-state fetch errors now preserve explicit outer-error backoff instead of
returning early for lack of patch preparation. Startup recovers valid waits
after a settlement/application cut; repeated processing uses the original receipt
time and cannot extend it. Queued fetches/patches defer, while unrelated controls
continue. Malformed hints remain preserved with static diagnostics; explicit new
fetch preparation reports unsupported saved refusals. Inner errors and successful
fetch contents are outside this correction. No held state, keys or request bytes
are changed, and no retry is added.
[Evidence, recovery and limits](docs/protocol/app-state-patch-atomicity.md#fetch-specific-recovery-correction).
No live account operation occurred.

Validation: **1,184 workspace/all-feature tests**, strict all-target/all-feature
Clippy, formatting, coverage freshness and repository hygiene passed.

2026-11-04 — **Responsive one-at-a-time action pacing:**

Generic action drains now reserve one request only when its monotonic pacing
deadline is due. Activation/heartbeat retain a sixteen-action drain budget;
repeated triggers do not stack or reset an active drain. Inbound replies, stop
and other deadlines can run between writes instead of waiting inside a batch
sleep. Failed/uncertain writes retain their reservation while later actions stay
queued; zero-delay sends still return through the actor loop. Readiness denotes
authenticated activation, not a drained queue. Socket-write completion itself
still awaits; full app-state job ownership is a separate unported lifecycle.
[Implementation and limits](docs/protocol/app-state-connection-flight-design.md).

Pairing research additionally traces current input/generation gates and request
callback correlation; authenticated account/source authority and primary Me
creation remain unresolved. [Evidence](docs/protocol/primary-pairing-auth-current.md).

Validation: **1,181 workspace/all-feature tests**, strict all-target/all-feature
Clippy, CLI-reference checking, formatting, coverage freshness and repository
hygiene passed. Research accounting is 817 REA records, 461 for the current APK,
68 ownership mappings; these are not completeness or account-safety percentages.
No live account operation occurred.

2026-11-04 — **Session group recovery ordering follow-up:**

The session-specific pending group recovery path now uses sequenced settlement
order across group action kinds rather than random pending IDs. Unsequenced
legacy pending replies remain untouched and inspectable. The restart regression
covers reversed IDs, latest projection, once-only replay and unchanged legacy
checkpoints/reservations. Native generation admission and ordering against
unsequenced notifications remain open.
[Correction and remaining boundaries](docs/protocol/action-settlement-ordering.md).
No live account operation occurred.

Validation: **1,178 workspace/all-feature tests**, strict all-target/all-feature
Clippy, formatting, coverage freshness and repository hygiene passed.

2026-11-04 — **Durable settlement order and atomic blocklist replay:**

New correlated terminal IQ replies now atomically acquire monotonic settlement
order. Contacts/groups/blocklist replay uses that order across relevant kinds;
blocklist projection, hash and once-only marker share one transaction. Original
requests supply block/unblock targets. New counter damage refuses without reset.
Legacy missing order remains inspectable and is excluded from those automatic
projection paths; it cannot be manufactured from IDs or timestamps. Native
generation/fetch freshness admission remains open.
[Correction and legacy boundaries](docs/protocol/action-settlement-ordering.md).

New current-build evidence distinguishes live manager-flight timeout ownership
from durable `Sent` history and traces own-device primary pairing separately
from companion metadata ingestion. No speculative flight guard or read-self
sender was added. [Scheduler](docs/protocol/server-sync-trigger-flight.md) and
[own-companion ingress](docs/protocol/own-companion-ingress-current.md).
No live network request or state reset occurred.

Validation: **1,177 workspace/all-feature tests**, strict all-target/all-feature
Clippy, CLI-reference regeneration, formatting, coverage freshness and repository
hygiene passed. Research accounting: 777 distinct REA records, 421 for the current
APK, 68 explicit ownership mappings. Counts are not a completion percentage.

2026-11-04 — **Legacy preparation compatibility without rebuilding requests:**

Old prepared-only and bound patch checkpoints may now acquire missing logical
indexes from their exact original authenticated wire. The current baseline must
already have authenticated provenance, and every frozen integrity field must
match. Reservation atomically stores the completed preparation; accepted
settlement stores only the authenticated derived result. Supplied incompatible
indexes, unknown baseline provenance, stale state and missing entire preparations
still refuse. Original requests, IDs and keys remain unchanged; no send is reset.
[Compatibility matrix and unresolved boundaries](docs/protocol/app-state-patch-atomicity.md).

Validation: **1,167 workspace/all-feature tests**, strict all-target/all-feature
Clippy, formatting, coverage freshness and repository hygiene passed. No live
account traffic occurred.

2026-11-04 — **Logical-index integrity across mutation keys:**

- App-state SET/REMOVE now select the previous value MAC by authenticated logical
  index, so the same item under a different mutation key replaces/removes the
  held entry. Patch and snapshot verification reject duplicate logical indexes.
  Saved state validates duplicate/cardinality/reference checks and recomputes
  its LT hash from retained MACs.
- Legacy nonempty MAC maps remain inspectable but cannot authorize incremental
  verification or construction without authenticated index provenance. A
  matching same-version snapshot can reconstruct metadata without advancing
  versions or replaying actions. Both storage formats retain the metadata.
- Offline `app-state-inspect` shows versions/counts/provenance. Its optional
  `--reconstruct-snapshot PATH --collection NAME` uses already-held keys and
  updates only matching existing transport checkpoint metadata; pending
  preparations block reconstruction. No automatic fetch or reset was added.
  [Evidence, migration and limits](docs/protocol/app-state-logical-index.md).

Full sync scheduling, action projection and historical orphan reconciliation
remain open. No live account traffic occurred.

Validation: **1,166 workspace/all-feature tests**, **287 API default-feature
tests**, strict workspace/all-target/all-feature Clippy, CLI-reference
regeneration, formatting, coverage freshness and repository hygiene passed.
Retained research now contains 721 distinct REA records, 365 for the current
APK, with 68 explicit Rust ownership mappings. These counts are not a port
percentage or proof against account restrictions.

Current read-self evidence now closes the Me gate, logout filtering and writer
route, but the existing contact device list cannot supply own-companion authority.
Own-device ingress and lifecycle remain prerequisites; no speculative read-self
send was added. [Current authority trace](docs/protocol/read-self-authority-current.md).

2026-11-04 — **Read policy and conflict-key resolution:**

- Newly automatic direct-read intents retain their provenance and defer while
  the durable server read-receipt preference is known disabled, checked both
  during queueing and immediately before reservation. Deferred work remains
  inspectable without a send reservation. Explicit reads, groups and legacy
  intents with unknown provenance retain their existing behavior. This closes a
  proven negative gate; native read-self routing remains unported.
- Conflict catch-up resolves each record's key independently of its patch's
  signing key, using already-held keys in the same transaction. A missing key
  refuses the whole transition. The enclosing patch key need not equal record
  keys, as confirmed in current-build `SyncResponseHandler.A07`.
  [Evidence and regression](docs/protocol/app-state-catchup-atomicity.md).
- A separate confirmed defect remains under correction: native mutation state
  is keyed by authenticated logical index; Rust's wire-index-MAC map can count
  a cross-key replacement twice or refuse a removal. This chunk does not claim
  to fix logical-index tracking or complete server-sync scheduling.

Validation: **1,156 workspace/all-feature tests**, strict all-target/all-feature
Clippy, CLI-reference regeneration, formatting, coverage freshness and repository
hygiene passed. Research accounting is 690 distinct REA records, 334 from the
current build, with 60 explicit Rust ownership mappings; these are not completion
percentages. [Current read-policy evidence](docs/protocol/read-receipt-policy-current.md)
and [logical-index defect](docs/protocol/app-state-logical-index.md).

No live account traffic occurred. These are protocol-correctness improvements,
not evidence of server acceptance or a guarantee against restrictions.

2026-11-04 — **Atomic patch lifecycle, explicit server waits and privacy adoption:**

- App-state key selection, baseline, original action wire and prepared state now
  commit together. Single-flight checks and reservation-time MAC/baseline checks
  prevent duplicate/stale writes. Existing prepared-only rows reverify their
  original wire; missing/corrupt preparations refuse before sending.
- Exact collection/version matching and baseline checks prevent an accepted
  response from overwriting later state. Conflict keys, complete patch-chain
  verification and pending release share one transaction. Malformed later
  patches roll back the entire application. Historic already-sent orphan state
  still needs reconciliation; no identity/history reset or replay was added.
- Integral explicit outer-error backoff persists from the original receipt
  time. New app-state requests respect its deadline; never-sent queued fetches/patches defer while
  unrelated control work can continue. Unknown IQ types remain sent/uncertain
  rather than becoming false refusals.
  [Corrections, tests and remaining boundaries](docs/protocol/app-state-patch-atomicity.md).
- A fully supported non-modify readreceipts-only account_sync notification now
  commits validated preference/deduplication before ACK. Unsupported/mixed/modify
  branches and unowned setter outcomes remain unACKed. Offline
  `account-privacy-inspect` shows the saved preference; read/read-self policy is
  unchanged. [Port and current MEX binding](docs/protocol/account-privacy-notifications.md).
- Native server-sync role/recovery gates are traced further. Primary/companion
  and re-registration/reset distinctions are now concrete; native mutation-row
  eligibility still requires a faithful Rust mapping before automatic scheduling.
  [Role and recovery](docs/protocol/server-sync-role-recovery.md).

Validation: **1,152 workspace/all-feature tests**, **439 default-feature tests**,
strict all-target/all-feature Clippy, CLI-reference regeneration, formatting,
coverage freshness and repository hygiene passed. New tests use real encrypted
stores, concurrent publishers, independent crypto vectors, rollback/reopen,
legacy rows, malformed replies, explicit waits and failed ACK writes. Retained
research totals 625 distinct REA records (269 current build), 57 explicitly
mapped to Rust. These counts do not establish completeness or account safety.
No live account operation occurred. The full network goal remains active.

2026-11-04 — **Held-version fetch construction and request-outcome tracing:**

- Added library `fetch_request_at_versions`: held verified versions are emitted
  unchanged; absent versions are omitted; snapshots are requested only when
  explicitly requested and version is absent/zero. Duplicate collections and
  unsigned values above the APK signed-Long range refuse locally. Existing
  valid explicit full-snapshot CLI requests retain their wire shape. Scheduler,
  role/gate selection and notification ACK wiring remain unfinished.
  [Current producer, wire and tests](docs/protocol/app-state-versioned-fetch.md).
- Privacy MEX IDs/converters and legacy-versus-MobileConfig selection are
  retained. A compiled default does not replace hydrated configuration. Current
  readreceipts admission has additional false-valued forms; no receipt policy
  changed without a durable preference owner.
  [Configuration and MEX limits](docs/protocol/privacy-mex-config.md).
- Rotation IQ queue acceptance, definite error, timeout and result have distinct
  effects. The native timeout invokes delivery failure rather than definite
  refusal bookkeeping. CAT startup traffic requires a real stored server token;
  push-origin maintenance is conditional, not a guessed registration field.
  [Outcome trace and pending lifecycle work](docs/protocol/authkey-outcomes.md).

Validation: 1,132 workspace/all-feature tests, strict all-target/all-feature
Clippy, CLI-reference regeneration, formatting, coverage freshness and repository
hygiene passed. Twelve focused request-builder tests cover native snapshot
branches, exact large-integer wire roundtrips, signed overflow and duplicate
inputs. Evidence accounting retains 560 distinct REA records (204 current build),
52 with explicit Rust ownership. Counts are not conversion or ban-risk metrics.
No live account request, token fabrication or key rotation occurred. The full
network audit remains active. Previous source commit 6de02da passed remote CI.

2026-11-04 — **Noise-pair preflight and notification lifecycle audit:**

- Comparison login now verifies persisted Curve25519 public/private consistency
  before endpoint/DNS/TCP/Noise. A mismatch refuses with a redacted error; no
  repair, key replacement or login reservation occurs. Regression inputs use
  the independent registration key producer and survive encrypted-store reopen.
- Auth-key storage/replacement and the connection getter are traced; candidate
  encoding, request-manager outcomes and lost-result recovery remain open.
  [Lifecycle and local correction](docs/protocol/authkey-lifecycle.md).
- Server-sync dirty versions are distinct from verified versions/LT hashes;
  incremental scheduling cannot reuse the existing all-version-zero snapshot
  fetch. Privacy refresh selects SMAX/MEX/shadow through an effective property,
  then applies guarded values. These are required remaining stateful ports; no
  ACK-only shortcut or invented effective configuration was added.
  [Server lifecycle](docs/protocol/server-sync-lifecycle.md),
  [privacy refresh](docs/protocol/account-privacy-refresh.md).
- Reduced Noise/transport compilation exposed an inbox-dependent presence helper
  missing its Signal feature gate. Its callers already require that feature;
  the gate now matches the dependency without changing full-client traffic.

Validation: 1,128 workspace/all-feature tests and strict all-target/all-feature
Clippy passed; standalone Noise tests, reduced Noise/transport checks/tests,
CLI-reference regeneration, formatting, coverage freshness and repository hygiene
were checked. The research report retains 500 distinct REA records (144 current
build) and 45 explicit Rust ownership mappings. Counts do not establish completed
ports or ban protection. No live account request or key rotation occurred.
The full network audit remains active and incomplete.

2026-11-04 — **Current server-ping response fixed; deeper network gaps retained:**

- The 2.26.43.72 reader-to-writer trace establishes a no-ID IQ result for an
  incoming server ping. Rust now sends that exact response after capture and
  clock observation. It neither activates the session nor replays historical
  captures; write failures remain visible. Three regressions cover shape,
  negative routing, persistence/reopen and failed writes.
  [Evidence and bounded correction](docs/protocol/server-pong.md).
- Current activation IQ construction already matches Rust; no extra fields
  were needed. Registration review identifies six constructed endpoint families
  versus 17 cataloged paths and traces 16 parameter helpers. Conditional fields
  and existing defaults are distinguished from unported runtime/native data.
  The native JNI-produced request map remains a high-priority unknown.
  [Registration audit](docs/protocol/registration-network-audit.md).
- `account_sync`, `server_sync` and `authkey_rotation` require stateful handling,
  version/callback policy and durable key lifecycle, not an ACK-only addition.
  Exact prerequisite traces are retained; those ports remain open.
  [Notification obligations](docs/protocol/notification-sync-obligations.md).

Validation: **1,127 workspace/all-feature tests** and strict all-target/all-feature
Clippy passed. CLI-reference, formatting, coverage freshness, whitespace and
repository hygiene were checked. Retained evidence now totals 430 distinct REA
records (74 current-build), with 45 explicit Rust ownership mappings. These are
research counts, not completion or restriction-risk percentages. No live account
traffic occurred and no zero-ban guarantee or causal ban diagnosis is established.
First hardening batch af2f31a also passed remote CI. Continue from the
[network ledger](docs/protocol/network-correctness-audit.md), prioritizing protocol
obligations over feature breadth.

2026-11-04 — **Network correctness takes priority over features:**

- Account creation and token verification reject unknown/malformed APK builds,
  foreign packages and material/User-Agent mismatches. The current 2.26.43.72
  APK was refused against the 2.26.37.70 template before creating any files.
  This checks declared metadata, not signature authenticity.
- Login rejects explicit malformed/non-UTF8/conflicting saved versions instead
  of silently falling back. Read-only account/config/payload preflight now runs
  before endpoint resolution, TCP or Noise. Genuine absent legacy fallback stays.
- Current APK privacy-token dispatch skips unknown token kinds then ACKs.
  Rust now preserves that ACK after supported updates commit; unknown outer
  types and malformed supported entries retain their existing refusals.

Validation: 1,124 workspace/all-feature tests passed. Default application suites
passed; their generated coverage freshness check initially caught the newly
retained evidence, then passed after report regeneration (439 total tests across
those suites). Strict all-target/all-feature Clippy, CLI-reference regeneration,
formatting, 16 CI-routing tests and whitespace checks passed. New evidence raises
retained REA records to 387, including 31 current-build records, and explicit Rust
ownership records to 25. These counts do not measure protocol completeness.
No live registration, login or message request occurred. No cause of account
restrictions or zero-ban guarantee is established. The [network audit ledger](docs/protocol/network-correctness-audit.md)
tracks remaining request/response families, ordering, acknowledgements and durable
state obligations; full current bootstrap and optional-traffic classification
remain open.

2026-11-04 — **Three APK-derived ports completed offline:**

- PIN `mismatch`/`guessed_too_fast` responses under HTTP 2xx now return the same
  attempt to explicit PIN entry with a new bound handle and persisted guessing
  deadline. The saved deadline survives reopening and cannot be erased by a
  caller editing its handle copy; early answers neither consume the step nor
  send. Existing generic PIN waits also bind. CLI output exposes the deadline.
  Other responses and earlier blocked attempts gain no reset/re-entry path.
  [Current enum/consumer proof and tests](docs/protocol/rea/2.26.43.72-registration-recovery/README.md#follow-on-port-explicit-pin-re-entry).
- Direct/group undo-keep now emits `edit="7"` only when the original key is
  from-me, mirroring the existing keep protobuf. Direct direction remains
  caller-supplied, not authenticated stored ownership; group reference guards
  remain stronger. Encryption and reservations are unchanged.
  [Predicate, writer proof and scope](docs/protocol/own-unkeep-edit.md).
- Both direct retry-answer paths now emit the received `enc count` unchanged
  for counts 1/2. The current APK producer-to-writer chain is retained. Higher
  counts, count-2 Alice-base-key storage/comparison, receipt keys and complete
  root-routing normalization remain unported; no broader retry support is claimed.
  [Trace, implementation and next obligation](docs/protocol/retry-enc-count.md).

Validation: **1,119 workspace/all-feature tests**, **438 default-feature tests**,
strict all-target/all-feature Clippy, CLI-reference regeneration, formatting,
16 CI-routing tests and whitespace checks passed. Focused tests cover real
crypto, original-target fields, store reopen, stale/forged handle deadlines,
once-only reservations, no automatic retry and negative status/reason cases.
The coverage report now accounts for 384 distinct REA records, 28 current-build
records and 22 explicit Rust ownership records. Counts are not completed ports.
No APK execution, login, registration or live message operation occurred.
The whole-APK program remains incomplete and continues from its existing queue.

2026-11-04 — **REA continuation started (offline):** reused the current Rust
actors/store/CLI and versioned research workflow. REA loaded the exact
2.26.43.72 APK (106,897 parsed classes); whole-file indexing scanned 15,728
members, 13 DEX entries and 12 native libraries. A resumable A00–A19 queue
covers the complete feature objective, including previously deferred systems.
The existing `xtask apk-coverage` now reports retained REA records separately
from bundle topics, validates identities and explicit Rust ownership, and is
selected by CI when REA JSON inputs change. Current totals: 366 distinct REA
records, including 10 current-build records and 7 explicit ownership mappings.
Neither inventory nor a complete method export counts as a completed port.
[Queue](docs/protocol/apk-porting.md#whole-apk-continuation-queue-2026-11-04) ·
[Coverage](docs/protocol/apk-coverage.md).

**First port:** traced `guess_wait` from current OTP/security-code response
constructors to password/PIN consumers, seconds parsing and the persisted input
timer. Rust now parses it, reports `guess_wait_seconds` and retains its deadline
on refused/cooling-down states without enabling unsupported retries. Ordinary
wrong-OTP re-entry was already implemented; the old blocker note was corrected.
[Evidence, interpretation and residual gaps](docs/protocol/rea/2.26.43.72-registration-recovery/README.md).
No account operation or APK execution occurred. Full APK conversion remains
unfinished; per-subsystem evidence/port/offline/live gates are explicit.

Validation for this continuation: **435 default-feature tests** and **1,114
workspace/all-feature tests** passed; strict all-target/all-feature Clippy,
formatting, CLI-reference regeneration, nine focused coverage tests and all 16
CI-routing tests passed. Repository hygiene checked 3,778 local links and 167
checksum bundles (32,149,347 bytes). No live validation is claimed.

2026-11-04 — **Fixed/offline (default build):** `chat_settings::edit_payload`
now shares its optional Signal dependency's feature gate. Its unconditional call
broke the documented default build and xtask commands (E0433), while all-feature
checks passed. Existing experimental callers and wire encoding are unchanged.
[Review and validation](docs/protocol/project-handoff.md#default-build-correction-found-during-review).

2026-11-04 — **Offline project handoff review:** reviewed main at `a13021b`,
confirmed the existing comparison-driver architecture, and recorded stale summary
claims and a prioritized continuation. Fresh workspace/all-feature tests passed
(1,105 tests). The published 2.26.43.72 APK was downloaded outside the checkout;
its hash, package version and signatures were verified. Historical account bans
and unconfirmed registration remain unresolved; no live operation or account
state change occurred. [Review and validation](docs/protocol/project-handoff.md).

2026-11-02 — **Observed live (corrected voice resend):** one authorized
voice resend on the saved pending attempt returned HTTP 200 / `sent`, with
`retry_after=305`, `voice_wait=305`, and `sms_wait=65` seconds. The request
used the rebuilt client with history-derived code metrics; the same identity
and attempt were preserved. No automatic retry followed. Call receipt and
completed registration remain unconfirmed; the code must be entered locally.
[Voice resend evidence](docs/protocol/registration-lab.md#corrected-voice-resend-2026-11-02).

2026-11-02 — **Fixed/offline (code-request metrics):** SMS/voice requests now
refresh configured byte-path `client_metrics.attempts` from complete saved
preparation history instead of reusing the template's 1 on every resend.
Malformed metrics and incomplete history refuse locally; identity and frozen
config remain unchanged. Regression tests cover a resend switching to voice,
other metrics properties and truncated history. Missing calls after `sent`
remain unexplained; no live request was made for this fix.
[Evidence and limits](docs/protocol/registration-lab.md#code-request-metrics-correction-2026-11-02).

2026-11-02 — **New signups refused (438-476 block):** after `q`'s ban, one SMS
`resend-code` for `r` (9825) was answered `no_routes` (cooldown about one hour)
and one for `s` (9826) `temporarily_unavailable` (cooldown). Both numbers are
in the 438-476 block whose other registered accounts (`l`, `m`, `p`) are
banned. No further requests were sent. (A first `r` attempt was refused
locally, not sent: its frozen signup config no longer matched after
`mark_read` was edited; the edit was reverted for `r`, `s`, `t`.)

2026-11-02 — **`q` banned (403):** `q`'s 16:10 UTC login was refused
`<failure reason=403 vt=15>`; its last successful login was about 04:26 UTC.
Its login presented 2.26.37.70 as always (it registered as 2.26.37.70), so the
version fix is not involved. In its ~30 hours `q` had: messaged the owner
(Scam Alert, then a six-hour reach-out timelock, many 463 refusals), joined a
group, sent group texts and reactions, edits and revokes, a document and a
video, and roughly 30 logins. Accumulated volume and pattern from a new number
is the likely cause (inferred; WhatsApp gives no reason). All lab accounts that
were registered (`l`, `m`, `p`, `q`) are now banned. Live work is stopped.
Also: `transport.mark_read` now defaults to `true` (owner's request).

2026-11-02 — **Fixed (login version matches registration):** `m` and `p` were
registered with the 2.26.43.72 User-Agent but every login presented 2.26.37.70
(hard-coded); both were banned (403) within an hour. A phone never registers
as one build and logs in as another, so this is the most suspicious thing our
client did (inferred, WhatsApp gives no reason; the accounts also messaged
strangers immediately and their numbers had earlier `blocked` history).
Signup now records the registered build (`registered_app_version`) and login
presents it, falling back to the config User-Agent, then 2.26.37.70. `q`
(registered and logging in as 2.26.37.70) is unchanged.

2026-11-02 — **Live (video send) / `p` banned:** `q` sent the owner a 3-second
320×240 MP4 via `outbox-send` with private JSON `video` (`VideoMessage`,
2.26.37.71 `BZE` field numbers, Video Keys domain, `/mms/video`,
`mediatype="video"`): server-accepted, then the owner's phone sent delivery
and read receipts. Meanwhile `q` received an undecryptable direct message
(most likely `p`'s delayed first text) and sent a retry receipt, but `p`'s
next login was refused `403` (banned, under an hour after registration), so
the retry answer could not run. Media upload routes expire within minutes;
refresh with `lab-transport --login --media-route` before an upload.

2026-11-02 — **Live (document send):** `q` sent the owner a small text file via
`outbox-send` with private JSON `document`: uploaded to `/mms/document`
(Document Keys domain), sent as `DocumentMessage` (2.26.37.71 `BZ6` field
numbers, no url) with `mediatype="document"`; server-accepted, and a delivery
receipt from the owner's phone followed. Handset display not yet confirmed.

2026-11-02 — **Live (group edit and delete-for-everyone):** in the owner's
group, `q` edited its earlier group text and then revoked it through the normal
group outbox (sender-key message, stanza `edit="1"` then `edit="7"`). Both
were server-accepted and the owner's phone returned a delivery receipt for each.

2026-11-02 — **Live (edit and delete-for-everyone):** new `edit-message` and
`revoke` commands. From `q` to the owner: an edit of a sent text (stanza
`edit="1"`, `MESSAGE_EDIT` with the replacement) and then a revoke of the same
message (`edit="7"`, `REVOKE`) were each server-accepted and got a delivery
receipt from the owner's phone. The `edit` values come from the APK's
outgoing message schema (`CpG.A0U`). Display on the handset ("edited" /
"This message was deleted") not yet confirmed by the owner.

2026-11-02 — **Observed live (retry test blocked):** for the item 6 live test,
`p` (registered minutes earlier) sent `q` a first direct text: server-accepted
(`ServerAccepted`), then the server ended `p`'s stream with `stream:error
code=503`. `q` received nothing from `p` in two later `listen` windows, so no
undecryptable message existed to retry. Live retry answers remain unverified;
no further sends from `p` were made.

2026-11-02 — **Observed live (two accounts, key-first group reaction; `m` banned):**
the 2.26.43.72 recipe registered `m` (8868) and `p` (8871) after
`retry-after-block`; four 845 numbers were still refused `blocked` at the code
request although `/v2/exist` answered `incorrect` (exist does not reveal the
block), and 9825–9827 got codes via `accounts new` + `begin`. Item 9's
key-first path then worked between our own accounts: `m` created a group with
`p`, `p` sent a group text that `m` decrypted, and `m`'s first group message,
a reaction, went out as a direct sender-key message plus a reaction without
`<participants>`; `p` decrypted both. Minutes later `m`'s next login was
refused `<failure reason=403>` (banned): registration, one login, one group
create, one reaction and a queued direct text in about five minutes. Live use
of brand-new accounts is paused; the retry-receipt live test was not reached.

2026-11-02 — **Observed live (2.26.43.72 token accepted):** lab number `m`
(refused `blocked` in September) answered `/v2/exist` with `incorrect` and
SMS/voice offered, so the block was gone. The new `lab-signup retry-after-block`
sent one corrected code request with the 2.26.43.72 User-Agent and a token
derived from the 2.26.43.72 APK, keeping identity and keys: `status: sent`
(SMS). Registration tokens from 2.26.43.72 are valid with its own User-Agent.
The code has not been entered yet.

2026-11-01 — **Retry answers complete the APK's two branches (offline) / group
reaction confirmed (live):** a peer's retry with a changed registration now
fetches the device's fresh bundle and resends under a new session with the same
message id (`151.A00`); matching registrations still resend under the saved
session. The owner confirmed the outbox group reaction (❤️) displayed on their
phone.

2026-11-01 — **Explained (group creation 463):** WhatsApp pushed `q` a six-hour
reach-out timelock (`mex` `NotificationUserReachoutTimelockUpdate`, active,
ends 2026-11-02 05:13 UTC). `listen` now records and logs it, and `group create`
refuses locally with the end time instead of sending into a refusal.

2026-11-01 — **Observed live (group on a real phone):** the owner created a
group with `q`. Their group text decrypted on `q`. Fixes found by the test:
the group outbox read only the group own-LID checkpoint and blocked a LID group
as `own_identity_missing` though login had established the LID (now uses the
shared resolver); and the out-of-band sender-key direct message lacked the
recipient's `<tctoken>` and was refused 463 (now attached, shared
`presence::held_tctoken`). Then a group text from `q` went out with embedded
distributions: server-accepted, and the owner's phone returned delivery and
read receipts (item 8). A group reaction from the outbox was server-accepted
(owner to confirm display). Read receipts: the owner confirmed blue ticks on
their "Nice" (item 7). The out-of-band distribution path with the token is not
yet re-tested live (the owner's devices now hold the chain).

2026-11-01 — **Group create still restricted:** group creation now adds each
participant's held trusted-contact token as `<participant><privacy>` (2.26.37.71
`AWf.A00 0112..0166`). One create from `q` with the owner's token was still
refused 463 `account_reachout_restricted`, while 1:1 messages to the owner now
work. So group creation by this new account has a separate lock; the owner
creating the group and adding `q` is the remaining path for the group checks.
The owner's reply in that window decrypted, and their phone sent read receipts.

2026-11-01 — **Fixed + observed live (463):** direct messages now carry the
recipient's trusted-contact token as a `<message>` child when we hold one. The
first such text from `q` to the owner was server-accepted and the owner's
handset sent a delivery receipt; the 463 refusals were the missing token
(Baileys: "1:1 message missing privacy token"), and the intermediate 479 was
schema validation (`SmaxInvalid`) of a participants wrapping we tried first.
The trusted-contact token issue after the ACK (item 3) went out and was
answered OK.

2026-11-01 — **Observed live (463 → 479):** at the owner's request one more
plain direct text from `q` got 463 again. Our direct sends carry no
`<tctoken>`; 2.26.37.71 `Cp3.A00/A01` place the recipient's token inside its
`<to>` in `<participants>` (gated by property 20605, default false; its
MobileConfig value is unknown). One send in that form was refused with
**479** rather than 463, so the shape is incomplete. The change was reverted
and live attempts stopped. [unknowns](docs/protocol/unknowns.md#463-follow-up-the-trusted-contact-token-2026-11-01).

2026-11-01 — **Implemented/offline (Kyber prekey upload, gated):** the upload
IQ now carries `pq_list` (up to 10 one-time ML-KEM-1024 keys) and
`pq_last_resort_key` whenever the server's A/B property 21198 is on, following
2.26.37.71 `13c.A03`/`0jc.A1C`/`0jc.A0c`. Once the last-resort key is
acknowledged, later uploads carry none. `pqsupport` is fully traced: it also
needs property 24160 (default false) and is only used in a `key_fetch` request
this client does not send. For `q` both properties are off, so nothing PQ goes
out, exactly as with the official app.

2026-11-01 — **Observed live (account `q` restricted):** one `listen
--send-outbox` session queued a group creation with the owner's number and one
direct text to the owner. The server refused both with error **463**: the group
create IQ answered `account_reachout_restricted`, and the text's message ACK
carried `error=463` (outbox fact `Failed`, code 463). Earlier the same day `q`
had messaged the owner successfully and the owner had replied. This is a
WhatsApp-side restriction on the new account initiating contact, not a client
bug. Live sends from `q` are stopped; nothing will be retried, re-identified or
worked around. It blocks the live checks for retry answers, group sender keys,
out-of-band group reactions and the trusted-contact token issue until the
restriction lifts or the owner messages `q` first.

2026-11-01 — **Traced + observed (post-quantum keys):** the APK uploads Kyber
prekeys and sends `pqsupport` only when A/B property 21198
(`PQ_KEYS_UPLOAD_CODE`) is true. Every `Can` predicate requires it, and its
built-in default is false (`07t.A0L`). `q`'s first A/B read (7,235 rows) does
not set it, so the official client would upload no Kyber keys for this account,
and neither do we. `abprops-query --show` now prints `pq_keys_upload_value`.
The read had first been refused by our own 4,096-child decoder bound (now
16,384); `undecodable --settle-replies` settled the captured reply offline.
[kem-prekey-inventory](docs/protocol/kem-prekey-inventory.md).

2026-11-01 — **Compared (2.26.43.72):** against what we rely on, the new build
has the same signing certificate, an identical binary token dictionary, and
an unchanged `about_logo.png`. `classes.dex` changed, so its registration
tokens differ and must pair with its own User-Agent (not verified live; the
recipe stays on 2.26.37.70). None of our wire literals was removed. A refused
login now keeps the server's `reason` code (403 banned; 405 outdated client in
public clients, reference only) instead of dropping it.
[comparison](docs/protocol/apk-2.26.43.72-comparison.md).

2026-11-01 — **Implemented/offline (group reactions in the outbox):** a normal
outbox group reaction whose devices still need our sender key is no longer
blocked. Each missing distribution goes out first as a direct message, each
server ACK is awaited, then the reaction is sent without `<participants>`. That
shape was decrypted by our own receiving client (account `q`), not by a handset; the only handset test of out-of-band distribution (2026-09-22) still showed a placeholder and remains unexplained. A refused distribution blocks as
`distribution_refused` (reaction reserved, unsent, uncertain). Risk: after the
reaction's ACK those devices are marked as holding the chain, so if a handset
silently drops the direct distribution, later group texts skip it too. The
handset check (owner's group with `q`) is what validates this.

2026-11-01 — **Implemented/offline (group changes):** `listen` now applies
`w:gp2` add/remove/promote/demote/subject notifications to an existing saved
group snapshot before ACKing (2.26.37.71 `0yv.A09`). Not live-verified.
[groups](docs/protocol/groups.md#change-notifications-applied-to-the-saved-snapshot-2026-11-01).

2026-11-01 — **Observed live (read receipt):** on `q`, `mark-read` of the
owner's direct reply plus one 40 s `listen` sent the read receipt; the server
answered `ack class=receipt type=read`. Handset blue ticks are not yet
confirmed by the owner. The same window received a `mex` notification (now
ACKed) and stopped cleanly.

2026-11-01 — **Implemented/offline (retry receipts):** `listen` now answers a
peer's `receipt type=retry` for our server-accepted direct sends, following the
2.26.37.71 `RetryReceiptHandler` policy (count ≤ 4, ≤ 60 days, not already
delivered, same session when the registration id matches): the original
stanza re-encrypted from retained plaintext, same message id, reserved once per
(message, device, count). Count 3–4, a changed registration id, groups and
older sends are recorded as `retry-unanswered` with a reason. Not live-verified
(a retry cannot be provoked without breaking a session).
[retry-answers](docs/protocol/retry-answers.md).

2026-11-01 — **Observed live (profile photo):** on `q`, `profile-picture set` with a
640×640 JPEG was answered `iq type=result` with a new `<picture id=…>`, and
`profile-picture get` for the owner's number was answered `result` with the
picture id, hash and URL. Display name, About and photo are now all set on `q`.

2026-11-01 — **Observed live (profile on `q`):** `push-name` and `about` were
queued and one 30 s `listen` sent them: the `pushNameSetting` app-state patch and
the About set were both answered `Ok`, and the new queued display-name presence
went out right after availability (fire-and-forget; handset view unconfirmed).
The same window received the owner's delivery receipt, a `privacy_token`
notification from the owner's phone, and the owner's text reply (decrypted).
Earlier, `q`'s first message to the owner showed WhatsApp's Scam Alert (new
number, not a contact, no name/photo, account-sounding text); the name is the
first remedy, profile photo and our own trusted-contact tokens are next.

2026-11-01 — **Observed live (w:gp2 ACK):** one 60 s receive-only `listen` on
account `q` (no sends) received a `w:gp2` group notification and stayed up to
`stopped cleanly`; before the fix an unacknowledged `w:gp2` ended sessions. The
sender-key and reaction changes are not live-verified: `q`'s only group
includes the banned `l`, and no second working account exists.

2026-11-01 — **Fixed/offline (sender-key distribution):** group sends no longer
re-send our sender-key distribution to every device on every message. Per
2.26.37.71 (`group_participant_device.sent_sender_key`, set by
`SendE2EMessageJob` only after the server ACK), devices that already received
the current chain are skipped; rotation still redistributes to all. Normal
outbox group reactions now go out when every device already holds the chain.
Not yet live-verified. [group-fanout](docs/protocol/group-fanout.md#distribution-only-to-devices-that-lack-the-chain-2026-11-01).

2026-11-01 — **Fixed/offline (wrong OTP):** `incorrect` and `mismatch` refusals of a
code no longer block the attempt. Per 2.26.37.71 `VerifyPhoneNumber.A5O`
(default branch) and `A1v`, both return to code entry; `mismatch` gates the next
code for `retry_after` or 60 s (new `answer_not_before` on the challenge,
absent in older saved attempts). [after-otp](docs/protocol/after-otp.md#wrong-code-results-2026-11-01).

2026-11-01 — **Fixed/offline (notification ACKs):** `w:gp2` group notifications
other than `groups_dirty`, and `disappearing_mode` notifications, are now ACKed
like the APK's `BaseNotificationHandler` does; unACKed `w:gp2` had ended
sessions. One shared base ACK builder serves `psa`, `w:gp2` and
`disappearing_mode`. `business` notifications stay unACKed (no handler traced).
Not yet live-verified; live checks are paused after the `l` ban.
[unknowns](docs/protocol/unknowns.md#other-unacknowledged-notification-types-2026-11-01).

2026-11-01 — **Observed live: account `l` banned (login `failure reason=403`,
appeal token present).** It followed, within about 35 minutes on that account:
one group creation with the new account `q`, about 12 separate fresh logins
(each `lab-transport` run is a new connection), two group-info queries to a
malformed masked id, three group texts, and seven group reactions, five of them
deliberately varied experiments, every one carrying a rotated sender key and
most silently dropped by the server. Earlier sessions also ended on
`stream:error` echoing unACKed `w:gp2` group notifications. **Not
established:** which of these triggered the ban. **Lesson recorded:** live
experiments must be few, spaced out and on one long-lived connection; never
iterate stanza variants against the server from a real account. No appeal was
submitted and no re-registration attempted (owner's decision). `q` is unaffected
(`login_checkpoint=success`). Next code bug from this run: `w:gp2`
notifications that are not dirty refreshes are never ACKed.

2026-11-01 — **Fixed and observed live (group reactions never delivered):** the
server ACKs a group `type=reaction` stanza that carries `<participants>`
(sender-key distributions) but delivers it to nobody; our paths always embedded
them. Proven with two own accounts and five controlled variants; with the
distribution sent first as a direct message, the reaction was delivered and
decrypted. `sender_key_stanza` refuses the dropped shape, the comparison group
send distributes out of band for reactions, and the normal group outbox blocks
reactions (`reaction_distribution_unsupported`) until it can do the same.
Handset rendering still to be confirmed. Also fixed: a masked display group id
(`***…@g.us`) is refused as input instead of being sent to a group that cannot
exist. [groups](docs/protocol/groups.md#root-cause-a-group-reaction-must-not-carry-distributions-2026-11-01).

2026-11-01 — **Fixed and observed live (PSA notification ACK):** the client never
ACKed `psa` notifications, so the server ended sessions about 15 s after one
arrived (`stream:error` echoing `ack class=notification type=psa`). Per
2.26.37.71 `PSANotificationHandler` (`17o`, a `BaseNotificationHandler`) every
psa is now ACKed, with `error="487"` for the unimplemented `surfaces`/
`psa_wakeup` features. Live on the new account: both redelivered PSAs
(broadcast and direct) were ACKed and a 45 s `listen` stayed up and
`stopped cleanly`. [unknowns](docs/protocol/unknowns.md#psa-notification-ack-ends-the-session-observed-2026-11-01).

2026-11-01 — **Observed live (new number fully registered):** a new US number went
`/v2/code` sent → first OTP `stale` (entered ~2 h late) → `resend-code` sent →
fresh OTP accepted with `fail`/`consent`, `pending=dob` → `submit-consent` (DOB,
confirmed) → HTTP 200 `ok`, registration completed with the unchanged recipe
(`enc_empty_h`, 2.26.37.70 token material). The first `listen` authenticated
(`success`, `offline count=0`, `login_checkpoint=success`), then about 2 s later
ended on a terminal `stream:error` whose only child is
`ack class=notification type=psa` (our ACK of a PSA notification; it carried no
`to`), the same shape as the unresolved 2026-10-08 observation. That points at
our notification-ACK encoding for `psa`, not at the account; not yet traced.

2026-11-01 — **Fixed (expired OTP no longer strands an attempt):** a code entered
about two hours after `status: sent` returned HTTP 200 `fail`/`stale` (observed
live, submitted once). Our flow had made that a terminal block with no resend
path, which is our limitation, not WhatsApp's: in 2.26.37.71
`VerifyPhoneNumber.A5O` case 9 only forgets the stored code, clears the input
and stays on code entry. `stale` now returns to a pending challenge on a new
step, and `resend-code` also accepts attempts saved as blocked by `stale`.
[after-otp](docs/protocol/after-otp.md).

2026-11-01 — **Research tooling ported to Rust (verified against the originals):**
the two vector generators (`xtask research-vectors`), the two Java-oracle
drivers (`xtask java-oracle`, including an exact CPython `random.Random`
port), the native schema reader (`xtask native-schema`, std-only ELF), the
KEM trace (`xtask native-kem-trace`), the QEMU runner (`xtask
native-qemu-check`) and the C decoder probe with both import shims (`no_std`
Rust, `tools/apk-research/native-probe`). Every port reproduced its committed
output byte for byte; the Rust probe's full-archive extraction matched all 79
library hashes. `dumpm.py`/`desc.py` were removed as duplicates of `apk-xref`/
`apk-smali` and `apk-proto-fields --literal-order`. No C remains. Still Python:
six Unicorn/Qiling native-execution checkers (no mature pure-Rust ARM64
emulator) and the SQLite JID-map oracle (only C-backed SQLite exists in Rust);
their outputs are committed and consumed by Rust tests.

2026-11-01 — **Observed live (registration recipe still accepted):** a new
number prepared with `accounts new` (body mode `enc_empty_h`, 2.26.37.70
User-Agent and token material, eight observed `code_bytes` fields) received
HTTP 200 `status: sent` from `/v2/code` (SMS wait 125 s, voice wait 65 s). The
code has not been entered yet, so registration of that number is unverified.
`accounts new` also created the store, so `begin` needed no `--create-store`.

2026-11-01 — **Maintenance (CI, dependencies, docs):** `main` had failed CI
since the rename (a rustfmt diff and 10 RustSec advisories, one critical in
`zip` 0.6). PR #8 restores it: rustfmt, patched `rustls`/`rustls-webpki`/
`webpki-roots`/`tokio`, and `zip` 9.2 on the pure-Rust zlib-rs backend. zip 9
collapses duplicate entry names before `by_index`, so the APK readers that
relied on seeing duplicates now refuse them via `has_duplicate_names()`; real
2.26.37.71 APK output (file index, token dictionary, a research bundle) is
byte-identical. PR #9 moves hmac/sha1/pbkdf2/md-5 together to digest 0.11 and
protox to 0.10; token derivation is unchanged (`token verify` matches).
Both are open at this entry. The newest APK, **2.26.43.72**, is available to
cloud sessions as a GitHub pre-release asset
([how](docs/protocol/apk-analysis.md#obtaining-the-apk)); nothing is attributed
to it yet. AGENTS.md is now the single agent working agreement; CLAUDE.md
imports it.

2026-10-13 — **Tested/offline (disk full):** a simulated full disk (a
test-only redb storage backend, not a real filesystem) shows a commit that
cannot write is all-or-nothing, after which redb refuses every write on that
handle until the store is reopened; reopening after space returns recovers the
committed state. On the real session loop, a receipt that cannot be stored is
not ACKed and ends the session; its redelivery after reopen is stored and
ACKed. A failed `fsync` can leave a commit visible even though `write`
returned an error, so a store write error now documents as "maybe committed".
A read of every caller found none that sends, resends or compensates on a
failed write, and the direct outbox key-query reservation is fault-tested: a
reservation that failed but landed is never sent after reopen. `store rotate-key` now reports a failed commit as an unknown
outcome ("keep both keys") instead of a generic error that could lead to
discarding the key in force.
[Audit](docs/protocol/replay-safety-audit.md#disk-full-and-failed-commits-2026-10-13).

2026-10-13 — **Implemented/offline (HTTP parity: media list and event kinds):**
`serve --store-routes` now also answers `GET /v1/media` (the `media-list`
selectors: SHA-256 of the inbox key, snake_case kind, advertised size; no URL,
MIME, key or chat) and `GET /v1/events/summary?consumer=<name>` (the
consumer's cursors and the next 50 event kinds; no content, chat or message
ID). The summary never acknowledges; acknowledging and full events stay
CLI-only. A consumer whose cursor precedes trimmed events answers 409
`cursor_expired` naming the last trimmed cursor. The CLI gained the same
JSON: `media-list --json` and `events --summary --json`. Corrected: the
OpenAPI `MediaItem` schema listed `mime`, `downloaded`, `opened` and a
`view_once_image` kind that the list never produced.
[API README](docs/api/README.md#store-routes-opt-in).

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
[Audit](docs/protocol/registration-safety-audit.md), [CI routing](docs/ci.md).

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
new module (unescaped `<…>`) is fixed. [Notes](docs/protocol/apk-analysis.md#field-numbers-from-retained-bundles-retained-proto-fields-2026-10-13).

2026-10-13 — **Changed/offline (research tooling):** `xtask apk-proto-fields`
now uses the same exact objects-array simulator as `retained-proto-fields`,
reading static helper bodies from the APK, instead of pairing names in method
order; `--literal-order` keeps the old pairing, labelled unverified. Not yet
run against an APK (none in this container).
[Notes](docs/protocol/apk-analysis.md#reading-protobuf-field-numbers-apk-proto-fields).

2026-10-13 — **Traced/static (sender edit values, retained bundles):** the
send runnable `DNu` keys a scheduled send on the message's own edit field
(`178.A00`), and the retained writers of that field give sender revoke 7,
admin revoke 8, poll edit 1, and undo-keep of one's own message 7 (keep 0).
Text-edit and pin values are not retained. The job constructor mapping and
the stanza writer remain inferred links. This client's `keep --undo` omits
the value (recorded divergence; the one retained receiver check, `D9L`, does
not look at it on undo). No sender
changed. [Trace](docs/protocol/unknowns.md#outgoing-revoke-and-edit-2026-10-13).

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
descriptor. [Evidence](docs/protocol/apk-analysis.md#field-numbers-from-retained-bundles-retained-proto-fields-2026-10-13).

2026-10-13 — **Fixed/offline (CLI parity):** `registration status --json` now
prints the saved attempt as redacted JSON, the same as `lab-signup status
--json` and the `GET /v1/signup/status` store route (one shared redaction);
`--json` used to be ignored there.

2026-10-13 — **Implemented/offline (HTTP parity: registration status):** `serve
--store-routes` also answers `GET /v1/signup/status` with the saved
registration attempt exactly as `lab-signup status --json` prints it (phone and
consent secrets redacted), or JSON null, so an agent can follow onboarding
state without the CLI. [API README](docs/api/README.md#store-routes-opt-in).

2026-10-13 — **Corrected/offline (third registration review):** a review of
the exact-restore change with probe tests found it sound, and one gap: an
unreadable 5xx or 429 email-code answer still allowed a second email request.
Only a definite 4xx (not 408 or 429) now counts as a refusal. The default
(`UnsupportedFlow`) flow now restores a challenge or existing attempt it did
not create instead of blocking over it; the `accounts new` scan is bounded and
refuses past its limit; failed-restore and unsent-attempt refusals say why.
Recorded: records restored before the exact restore keep one extra counted
preparation each. [Audit](docs/protocol/registration-safety-audit.md#third-review-2026-10-13).

2026-10-13 — **Implemented/offline (HTTP parity: store inspect):** `serve
--store-routes` now also answers `GET /v1/store` with the `store inspect`
report (table counts, generation, registration status, checkpoint kinds), the
same JSON as the CLI from one shared function. A checkpoint kind is printed by
name only when it is a lowercase record name and not hex-like; any other key
prefix (for example a key without `:` whose text is an address or digest) is
counted as `other`, for the CLI too. [API README](docs/api/README.md#store-routes-opt-in).

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
failed restore reports its cause. [Audit](docs/protocol/registration-safety-audit.md#second-review-2026-10-13).

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
[Audit](docs/protocol/registration-safety-audit.md#plausible-not-verified).

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
consume bounded history. [Audit](docs/protocol/registration-safety-audit.md#review-corrections-2026-10-13).

2026-10-13 — **Fixed/offline (registration: stores and status):** `lab-signup
begin` no longer creates a missing store unless `--create-store` is given;
`accounts new` creates the profile's store and refuses a number another profile
configures. A forgotten `--profile` or a mistyped path can no longer mint a
fresh identity for a number with a real store. `registration begin` no longer
silently replaces another number's or profile's attempt, and `registration
reset` no longer deletes a lab attempt. `accounts status` now shows the real
registration state. The `exist` probe is refused mid-request, once registered
and during a cooldown. Profiles made before this need `begin --create-store`
once. [Audit](docs/protocol/registration-safety-audit.md).

2026-10-13 — **Fixed/offline (registration: requests that never left):** a
registration request that could not be built, or whose connection failed
before any byte was written, was recorded as an unknown outcome. The HTTPS flow
cannot reconcile that, so a network outage on `begin` stranded the store,
leaving a fresh identity as the only way on. Such failures now return
`RequestNotSent` and restore the attempt's previous state (a fresh `begin`
resumes later with the same attempt and keys). Failures that may have reached
the server still record an unknown outcome. That connect errors occur before
writing is inferred from the HTTP library. [Audit](docs/protocol/registration-safety-audit.md).

2026-10-13 — **Fixed/offline (registration safety audit, waits and email
recovery):** `resume-email-challenge` is now allowed once per recovery kind per
attempt, so an uncertain email-code request can no longer be re-sent in a
loop; the `challenge` refusal's `email_otp_wait` is kept (inferred meaning,
longer value); and a shorter SMS/voice wait no longer replaces a longer
`retry_after`. Open: a `/v2/code` refusal that stays blocked does not keep its
SMS/voice wait (needs a saved-state field). [Audit](docs/protocol/registration-safety-audit.md).

2026-10-13 — **Corrected/offline (review of the receive hardening):** the
first undecodable-frame change kept receiving after any undecodable frame, so an
undecodable `stream:error` or `failure` would have been followed by
reconnects, retrying a session the server had ended. The reader now reads such
a frame's name leniently: a terminal stanza stops without reconnecting, and so
does a server close right after a stanza whose name is unreadable. The lab
group-send window no longer answers a retry receipt whose content was refused.
The memory-amplification note is corrected and recorded open (a burst of
crafted server frames could exhaust memory). [Audit](docs/protocol/replay-safety-audit.md#network-input-panic-audit-2026-10-13).

2026-10-13 — **Fixed/offline (registration safety audit, first two):** a
read-only audit of the onboarding HTTPS path found that `lab-signup status
--json` printed the consent context's intermediate login and parent consent URL
(now redacted at any depth), and that the code-request diagnostics and
`retry-corrected-token` accepted a `bad_param`/`bad_token` refusal from an OTP
submission or other request, where they would have asked for a new SMS (now
they require the refusal to come from a code request). The remaining findings
are listed and being verified. Nothing was sent. [Audit](docs/protocol/registration-safety-audit.md).

2026-10-13 — **Implemented/offline (lab windows):** `lab-transport` receive,
post-send and group-send windows now capture a frame that does not decode and
keep reading, like `listen`, instead of ending the window. Login, handshake and
the bounded reply waits (send ACKs, iq and key replies) still need a decodable
frame. [Audit](docs/protocol/replay-safety-audit.md#network-input-panic-audit-2026-10-13).

2026-10-13 — **Implemented/offline (inspection):** `whatsapp-cli undecodable`
(also `GET /v1/inbox/undecodable` under `serve --store-routes`) lists the frames
`listen` captured because it could not decode them: capture digest, error class
then and now, size, epoch. "Now" re-runs the current decoder, so a dictionary
update shows which captures it would read. Read-only and redacted; nothing is
reprocessed or acknowledged. [Audit](docs/protocol/replay-safety-audit.md#network-input-panic-audit-2026-10-13).

2026-10-13 — **Fixed/offline (network-input panic audit):** a read-only audit
for panics and aborts reachable from received data found one. Nested JID tags
made the binary decoder recurse until the stack overflowed: a 4 KB compressed
frame from the server would abort the process, uncaptured, at every login.
The decoder now refuses those tags before reading them. Also, a frame the
decoder refuses (for example a newer dictionary token) no longer ends `listen`
uncaptured: it is kept as `inbound-undecodable` with a fixed error class and
skipped. No other reachable panic was found. Only the server can send such
frames. [Audit](docs/protocol/replay-safety-audit.md#network-input-panic-audit-2026-10-13).

2026-10-13 — **Checked/offline (live-shape regression sweep):** every stanza
shape the docs record as seen live was compared with the current receive code.
No other regression like the status-receipt one was found. Fixed on the way:
`lab-transport --receive` and the lab group-send window now validate receipt
content like `listen`; a devices or prekey-count notification without `id` no
longer ends `listen`. Open: the 2026-10-11 ACK and confirmation encoding changes
have never been checked live; the next live receive window should confirm no
redelivery after `confirmed=true`. [Sweep](docs/protocol/replay-safety-audit.md#live-shape-regression-sweep-2026-10-13).

2026-10-13 — **Corrected/offline (review of the new store routes):** no
security finding. Three protocol docs and the feature-parity exposure note still
said these inspections were not served over HTTP; corrected. The key-query
schema lacked the `unsupported_bundle` reason the library already returns.
A damaged saved group send now answers 422 with its fixed reason, like key
queries and own LID, and the own-LID refusal reasons are now fixed by type.
Tests now cover the 128-byte boundary, 405, the disabled 404, 400-before-401,
non-null answers and the OpenAPI parameter names. [Store routes](docs/api/README.md#store-routes-opt-in).

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
[Corrections](docs/protocol/replay-safety-audit.md#review-corrections-same-day).

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
queues or exposes an address or message content. [Store routes](docs/api/README.md#store-routes-opt-in).

2026-10-13 — **Corrected/offline (receive-path audit):** a read-only audit of
the receive loop found that a receipt, `privacy_token` notification or
`pq_count` whose content we refuse, or an own-companion message with an
undecodable app-state key share, ended the session before its ACK. The server
then redelivered it at every login, so every later `listen` ended on the same
stanza. These are now logged, captured without ACK, and receive continues.
A viewed group view-once is a duplicate on redelivery, and our own LID no
longer queues a contact device sync. Four lower findings are recorded open.
Not seen live. [Audit](docs/protocol/replay-safety-audit.md#receive-side-audit-2026-10-13).

2026-10-13 — **Measured (cross-version coverage):** new `xtask apk-coverage`
reports what the committed research bundles cover. The pinned 2.26.36.75 has no
retained bundle; 2.26.37.70 has one (21 core topics); 2.26.37.71 has 166.
Only those 21 transport/Noise/registration topics have evidence from two
releases. 64 of 142 library files are cited by no bundle topic (many cite APK
classes in comments or REA envelopes instead). [Coverage](docs/protocol/apk-coverage.md).

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
guessable hash, use `127.0.0.1`). [API README](docs/api/README.md#store-routes-opt-in).

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
is unchanged; nothing that sends is served. [Design](docs/feature-parity.md#http-api-parity).

2026-10-13 — **Docs corrected (feature-parity audit):** an independent audit
of every row in [feature-parity](docs/feature-parity.md) against its evidence
found stale or overstated rows; corrected after spot-checking the cited lines.
Keep, disappearing and limit-sharing are acked (handset unconfirmed), not live;
group info/refresh, contact sync/publish and device-list queries were live but
listed offline; several CLI cells named the wrong command (`send` is text-only,
`lab-pair` is not pairing); 2FA and quote rows linked the wrong evidence.
Three missing rows added (incoming image/voice, privacy tokens, AB properties).

2026-10-13 — **Corrected/offline (review of quoted media sending):** as first
committed, group JSON image/voice-note replies were still refused by the JSON
reader (it required `text` on every reply), so they were reachable only through
the library despite the docs; fixed and tested through the reader. JSON
reactions and `quoted_text` are again refused before the send-path claim.
Direct quotes are now built at selection, so a wrong-chat parent, missing own
identity or over-32-KiB quote fails locally instead of after the key query.
Class names corrected to 2.26.37.71 `BZD`/`BZ0`/`BZL`. [Details](docs/protocol/quoted-media.md#sending-2026-10-13).

2026-10-13 — **Implemented/offline:** an uploaded image or voice note can reply
to a message (direct and normal group outbox): the media message carries the
same `BWx` contextInfo a text reply does, at the field the receive path reads.
`outbox-send` group JSON now accepts replies and, after a reader fix, group
image/voice note JSON, which had been refused by the shared reader despite being
documented. Our sent messages never carry `contextInfo.expiration` (recorded
as an unknown). No live quoted media send.
[Sending](docs/protocol/quoted-media.md#sending-2026-10-13), [expiration gap](docs/protocol/unknowns.md#outgoing-disappearing-context-2026-10-13).

2026-10-13 — **Corrected/offline (second review and replay audit):** a damaged
same-key held session no longer makes identity-refresh application error on
every connection start; it defers as `held_session_inconsistent` (preview
reports the same) and the row is kept. Once-only retry receipts now also probe
the sender's other PN/LID address, so a redelivery under the other address
cannot queue a second retry. The reconnect budget resets only after an
activated connection stayed up for `max_delay_ms`, bounding activate-then-drop
login loops. The hybrid receive helper refuses companion senders itself.
Status-reply quotes (`remoteJid`) remain unread and unverified.
[Held sessions](docs/protocol/hybrid-identity-refresh.md#damaged-held-sessions-2026-10-13), [retry aliases](docs/protocol/messaging-transport.md#retry-once-only-across-pn-and-lid-2026-10-13), [reconnect](docs/protocol/messaging-transport.md#reconnect-budget-stability-2026-10-13), [audit](docs/protocol/replay-safety-audit.md).

2026-10-13 — **Corrected/offline (independent review):** hybrid receive now
classifies missing or unusable local keys (including an unparsable KEM row)
as Undecryptable like classic; a group companion's hybrid pairwise child is
skipped without blocking its `skmsg`; fan-out/batches refuse attestation or
malformed-`identity_auth` PQ rows like the direct path (`unsupported_bundle`);
unusable outbox PQ replies suspend the intent with a reason instead of ending
the session. [Inbox](docs/protocol/kem-prekey-inventory.md), [fan-out](docs/protocol/hybrid-fanout.md), [outbox](docs/protocol/hybrid-outbox.md).

2026-10-13 — **Implemented/offline:** incoming media that quotes another
message (BWx stanzaId/participant/quotedMessage in contextInfo 17) now projects
as a media event with `quoted`, attributed by the same rule as text replies,
instead of an opaque placeholder. Stanza-less quotes stay opaque; edits and
sending are unchanged; no live sample. [Behavior and tests](docs/protocol/quoted-media.md).

2026-10-13 — **Implemented/offline:** the lab group-send retry answer now
derives a native hybrid session for a device whose fresh directory row carries
a verified `pqkey`, via the new `signal::hybrid::selects_pq_key` selector and
the shared fan-out derivation. Classic rows are unchanged; no live retry answer.
[Lab retry answers](docs/protocol/hybrid-fanout.md#lab-retry-answers-follow-up-same-day).

2026-10-13 — **Implemented/offline:** a verified PQ directory reply for a
contact with a held classic session no longer errors the direct send; the held
ratchet continues as on the classic path (no PQ upgrade, which stays unported).
Well-formed log-only `identity_auth` no longer blocks hybrid sends. A coordinator
test that nested `pqkey` inside `skey`, and so never reached the hybrid branch,
is corrected. [Follow-up and correction](docs/protocol/hybrid-outbox.md).

2026-10-13 — **Implemented/offline:** device-key batches now answer rows whose
`pqkey` verifies, and group/status fan-out sends those devices native 0x44
hybrid distributions. Previously one PQ row blocked the whole send. Primary,
companion, cached-continuation and signed-replacement cases open with the
device's own keys. Invalid KEM signature/encoding still blocks. `pqsupport` is
not sent; no APK was available; no live test.
[Behavior, evidence and tests](docs/protocol/hybrid-fanout.md).

2026-10-13 — **Implemented/offline:** identity-refresh preview and application
now stage a version-4 hybrid session from a verified primary `pqkey` reply:
changed keys derive fresh, admitted same-key refreshes archive held state. The
refreshed record opens a 0x44 exchange the peer authenticates. Invalid PQ defers
as `pq_bundle_invalid`; companion/attestation PQ stays deferred. `key_validation`
now reports ML-KEM-1024 encoding. No APK was available; no live test.
[Behavior, reused evidence and tests](docs/protocol/hybrid-identity-refresh.md).

2026-10-13 — **Implemented/offline:** the normal direct and group inboxes now
route primary PN/LID version-4 KEM prekeys through the hybrid receive path.
Matching held sessions decrypt without keys; fresh bootstrap reads a local
ML-KEM-1024 row and consumes non-last-resort rows atomically. Replies decrypt as
0x44 msg. KEM generation/upload stays untraced and absent; own-device/companion
KEM is refused unchanged. No APK was available; no live test.
[Inventory, routing table and encrypted tests](docs/protocol/kem-prekey-inventory.md).

2026-10-12 — **Implemented/offline:** staged incoming hybrid prekeys now
bootstrap from exact local EC/KEM keys and authenticate nested ciphertext before
reporting consumption. Matching current/archived sessions decrypt without prekey
lookup; last-resort KEM and ordinary ID 16777215 remain reusable. Encrypted tests
cover pin policy, delayed envelopes, replay, malformed keys and account binding.
Normal inbox/key-inventory integration remains open.
[Native lookup/deletion evidence, API and tests](docs/protocol/hybrid-incoming.md).

2026-10-12 — **Implemented/offline:** normal outgoing key replies now explicitly
select verified hybrid material for primary PN/LID peers. Fresh first-use and
policy-approved identity replacement produce native 0x44 candidates through the
existing atomic outbox transaction; held hybrid records continue without reset.
Encrypted fixtures verify request/attempt preservation and no replay after reopen.
Companion/auth/attestation, classic migration and incoming bootstrap remain open.
[Coordinator behavior, evidence and tests](docs/protocol/hybrid-outbox.md).

2026-10-12 — **Implemented/offline:** fresh exact-recipient hybrid bundles now
verify both signed EC and original KEM value bytes, validate ML-KEM-1024 encoding,
and stage a native version-4 initial message/session candidate. PN/LID encrypted
fixtures cover optional ordinary prekeys, malformed metadata and pin/account
refusals. Directory authority/adoption and incoming bootstrap remain unfinished;
normal PQ guards are unchanged. [API, evidence and scope](docs/protocol/hybrid-bundle.md).

2026-10-12 — **Implemented/offline:** the shared send adapter now prepares
native version-4 ciphertext from held hybrid records after account, registration,
peer-pin and ratchet-pair checks. Text and encoded payload exchange works through
normal APIs; an encrypted-store outbox test commits the same request/wire ID with
DeliveryUnknown and refuses replay after reopen. Fetched signatures remain
required. Fresh PQ directory/receive/bootstrap guards and live gaps remain.
[Shared send integration, ownership guards and coordinator test](docs/protocol/held-hybrid-send.md).

2026-10-12 — **Implemented/offline:** staged hybrid Alice/Bob records now
perform real EC/KEM bootstrap, native 0x44 encryption, authenticated reply and
continued ratcheting with/without an ordinary one-time key. Pending KEM metadata
survives canonical reopen and clears after a valid reply; counter leasing and
source-record preservation remain effective. Normal client adoption/send/key
inventory is not connected; PQ guards and live-interoperability gaps remain.
[Session API, native state/chain traces and encrypted exchange test](docs/protocol/hybrid-session.md).

2026-10-12 — **Implemented/offline:** low-level Signal wire encoders now
construct native 0x33/0x44 version bytes before HMAC and retain paired outer KEM
fields. Complete bytes match 24 independent nested and eight outer fixtures;
version-3 backend equivalence and version-byte tamper authentication are checked.
Existing application/backend send paths and PQ guards remain unchanged; hybrid
state/coordinator integration and live interoperability remain unfinished.
[Native schemas/MAC, encoder API and independent fixtures](docs/protocol/signal-wire.md).

2026-10-12 — **Implemented/offline:** incoming prekey APIs retain original
paired KEM ID/ciphertext metadata and refuse incomplete pairs. A valid classic
MAC no longer makes an envelope with ignored KEM fields consumable. Direct/group
receive defer before conflict observation or prekey lookup; all saved tables and
journal remain unchanged after capture and reopen. Native paired fields and
0x44 version-4 encoding are traced; outbound/hybrid integration remains open.
[Wire evidence, metadata API and preservation tests](docs/protocol/prekey-kem-wire.md).

2026-10-12 — **Implemented/offline:** initial hybrid root/chain derivation
now follows the APK's discontinuity prefix, ordered EC agreements, optional
one-time agreement, final KEM secret and exact hybrid HKDF label. Native extract,
counter-one expansion and 64-byte split are retained; Rust outputs are checked
against 16 independent synthetic HMAC fixtures. This is a low-level KDF helper;
PQ admission, pending wire/state integration and live sessions remain unfinished.
[Native construction, independent fixtures and API](docs/protocol/hybrid-kdf.md).

2026-10-12 — **Implemented/offline:** experimental `signal::kem` exposes
Rust ML-KEM-1024 generation, encapsulation and expanded-key decapsulation using
pinned RustCrypto with zeroization and redacted secrets. The same 16 APK/NIST
vectors exercise serialized keys, ciphertexts and shared secrets. This is a
primitive API, not PQ session admission; all current bundle/refresh guards remain.
Hybrid sessions and live interoperability remain unfinished.
[Primitive API, dependency limits and tests](docs/protocol/rust-kem.md).

2026-10-12 — **Confirmed/offline execution:** the pinned APK's native
ML-KEM-1024 arithmetic/hashing matches 16 independent NIST FIPS203 cases:
key generation, deterministic encapsulation and valid/modified-ciphertext
decapsulation. Only libc memory/allocation and once initialization are modeled.
No JNI, native RNG, account or hybrid session execution is claimed. The selected
primitive vectors pass; Rust integration and live interoperability remain open.
[Native execution, NIST provenance and reproducer](docs/protocol/native-kem-vectors.md).

2026-10-12 — **Confirmed/static:** native JNI public-key length dispatch calls
ML-KEM-512/768/1024; the Java CRYSTALS-KYBER-1024 HKDF label stays unchanged.
The Java seed array is length-checked but never consumed; native encapsulation
requests its own 32-byte randomness. Exact JNI widths and import/export thunks
are retained with a hash-bound reproducer. Primitive equivalence and Rust hybrid
sessions remain unverified; no account or network change.
[Native dispatch, sizes and randomness](docs/protocol/native-kem.md).

2026-10-12 — **Implemented/offline:** identity refresh diagnostics now expose
redacted PQ signature verification over the original directory value bytes,
using the returned candidate identity. Native KEMPublicKey stores/returns those
bytes unchanged; the hybrid HKDF suite label names CRYSTALS-KYBER-1024, while
JNI algorithm revision/format and hybrid sessions remain unported. A verified
signature is not KEM validity or adoption: preview/application stay deferred,
all saved state is preserved, and the consumable PQ bundle guard remains active.
[Native byte domain and diagnostic/admission tests](docs/protocol/pq-signature.md).

2026-10-11 — **Implemented/offline:** selected directory PQ key input now
fails explicitly instead of silently becoming a consumable classic bundle.
Direct preparation and device batches share the boundary; unrelated devices
and classic metadata inspection retain their behavior. Identity preview/application
still defer PQ without derivation or adoption. Saved keys, sessions, IDs, uncertain
sends and waits stay intact. Full native KEM and fresh live interoperability remain
unfinished; this is an unsupported-backend guard, not PQ support or native policy.
[Native bundle path, backend audit and preservation tests](docs/protocol/pq-bundle-boundary.md).

2026-10-11 — **Implemented/offline:** own ordinary video/GIF and document
caption edits now preserve original attachments and document filenames. Native
kind equality and the one-way video-to-GIF-parent exception are enforced using
saved synchronized payloads. The deeper decoder trace corrects image removal:
ordinary media edits require a nonempty caption regardless of own ownership.
Empty/missing edits decline without receive-state mutation; Unicode and whitespace
are preserved within the existing byte bound. Peer/special/native media hooks,
modeled document title, large Unicode payloads and fresh live delivery remain open.
[Decoder context, kind and preservation tests](docs/protocol/media-caption-edit.md).

2026-10-11 — **Implemented/offline:** authenticated own-device ordinary image
caption edits now emit Edited while retaining the original sealed attachment
reference. Replacement keys/URLs do not redirect the attachment. The later
[ordinary caption decoder correction](docs/protocol/media-caption-edit.md) rejects
caption removal; the original permissive claim is superseded. Original parent bytes, send facts, IDs and waits remain unchanged.
Kind, terminal and ordering guards still apply; other media, peer configured
window, special/orphan/expiry lifecycles and fresh live delivery remain open.
[Native media clone and receive tests](docs/protocol/image-caption-edit.md).

2026-10-11 — **Implemented/offline:** held authenticated own-device sessions
receive ordinary text edits as MessageUpdated/Edited for loaded own text parents
and attributed outgoing text intents. Parent chat/direction/ID stay intact;
embedded ownership metadata cannot select another owner or chat. Strictly older
edits and edits after an own revoke stay opaque with atomic ratchet/capture
commit; equality is accepted. Existing confirmation, reopen deduplication and
clearing paths remain effective, and uncertain send facts/IDs/waits stay unchanged.
Peer configured-window resolution, media/special/orphan/expiry lifecycles and
fresh live edit interoperability remain unfinished or unverified.
[APK decoder, ordering and receive tests](docs/protocol/message-edit.md).

2026-10-11 — **Implemented/offline:** supported direct peer sender revokes now
honor the native strict three-day receive rejection when both explicit times and
an authenticated parent are available. Signed wrapping subtraction is preserved;
missing time stays unknown. Ignored controls get a final opaque marker, so replay
or later metadata cannot turn them into updates. Own-device sender revokes retain
their separate path, and published history/parents/IDs/waits stay intact. Full
eligibility, physical deletion and live interoperability remain open.
[Native timing flow and receive/recovery tests](docs/protocol/sender-revoke-window.md).

2026-10-11 — **Implemented/offline:** protocol type and keep type now use the
pinned closed-enum rules: unknown numbers retain the last recognized value,
wrong-wire enum fields stay unknown, and singular message fragments merge.
Absent/unknown-only type never becomes a revoke. Shared borrowed-byte parsing
reuses the attestation primitives; all prior BP2 vectors remain valid. 2,040
independent Java vectors and encrypted receive/reopen tests pass. No encoding,
saved account/request/wait or live change. [Enum source, codec fix and limits](docs/protocol/control-enum.md).

2026-10-11 — **Implemented/offline:** ordinary direct sender revocations now
project into MessageUpdated/Revoked for peers and held own-device wrappers.
Envelope ownership overrides embedded flags/remotes; saved peer routes retain
original namespace/device. Native edit parsing admits the sender path, while
edited non-revoke bodies and forged edited redelivery of saved text refuse.
Signal, reopen/deduplication, unchanged uncertain sends and ratchet continuation
are covered. Physical deletion, native timing/cleanup, group/admin/status and
outgoing revoke/edit remain open. One bounded receive-only window authenticated
and exited cleanly; action/receipt/outbox views and account/journal summaries
stayed equal, with ten new inbound captures. No fresh live revoke verification.
[Native sources, receive tests and live limits](docs/protocol/sender-revoke.md).

2026-10-11 — **Implemented/offline:** identity-refresh session preview now
supports primary changed keys through the existing verified classic application
helper. Changed candidates derive fresh with zero old archives; same-key
candidates retain receiving archives. Supported changed-key attestation fallback
keeps failed verification separate from derivation. Saved tables/journal remain
unchanged, including after reopen; no request, key adoption or live transition.
[Primary preview, native trace and boundaries](docs/protocol/session-refresh-preview.md).

2026-10-11 — **Implemented/offline:** held own-device sessions synchronize
ordinary keep/unkeep with original target ownership, authenticated own actor,
sender timestamp and update ID. Shared keep decode validates the supported full
parent key; keep and reactions reuse source/parent checks. Signal confirmation,
reopen deduplication, clearing and continued ratchet use are covered. Native
permission, physical retention/expiry, conflict ordering and live interoperability
remain open. [Own-device keep evidence and tests](docs/protocol/own-device-keep.md).

2026-10-11 — **Implemented/offline:** held own-device sessions now synchronize
legacy disappearing-timer set/off/sync events. Shared decode clamps signed
nonpositive native duration bits to zero and fixes BW7 device-string/initiated
fields 3/4. Send intents reject unsigned values that native clients would disable.
After-read payloads decline pending property 25649 and second-duration state;
full setting application/ordering/expiry and live interoperability remain open.
[Native timer builders, mode correction and receive tests](docs/protocol/own-device-timer.md).

2026-10-11 — **Implemented/offline:** held own-device sessions now synchronize
ordinary reaction set/removal. Embedded fromMe preserves own/peer targets;
registered own source supplies the actor. Known original keys and unloaded typed
references work without fetching history or changing send facts/IDs/waits.
The shared own-parent lookup also serves polls, retaining local reservation
priority and ambiguity guards. Encrypted/special reactions, temporal policy and
fresh live interoperability remain unfinished/unverified.
[Own-device reactions and mapper/source checks](docs/protocol/own-device-reaction.md).

2026-10-11 — **Implemented/offline:** reaction receive/send text now follows
the native 30 Unicode-code-point bound instead of 64 bytes. Thirty supplementary
emoji and bounded ZWJ/combining sequences work; 31 ASCII characters refuse.
Direct/group/outbox/eligibility and status-like decode share the predicate.
Native control-character behavior and removal are preserved; saved requests,
facts, IDs and waits are not rewritten. Own-device reactions and fresh live
acceptance remain unfinished/unverified.
[Native predicate, boundary regressions and API contract](docs/protocol/reaction-text.md).

2026-10-11 — **Implemented/offline:** held own-device sessions now synchronize
ordinary poll votes into MessageUpdated/PollVote. Embedded fromMe preserves
own/peer parent ownership; AEAD must identify a saved own primary voter.
Foreign voters, special parents/routes and edited metadata refuse without a
ratchet commit or confirmation. Reopen deduplication and sender confirmations
use existing paths; no send intent or status mutation. Full edit/special/native
parent policy and fresh live interoperability remain unfinished/unverified.
[Own-device votes and native mapper closure](docs/protocol/own-device-vote.md).

2026-10-11 — **Implemented/offline:** voting can now resolve a synchronized
own poll through its authenticated wrapper when no local secret reservation
exists. Incoming peer vote context, outgoing vote encryption and binding
inspection share the lookup. Local reservations retain priority; foreign or
ambiguous parents refuse. No cache import or state mutation; synchronized
selection count is enforced. Own-device vote updates and live interoperability
remain unfinished/unverified.
[Parent/secret evidence and vote support](docs/protocol/synchronized-poll-secret.md).

2026-10-11 — **Implemented/offline:** held own-device sessions now receive
ordinary poll creations into outgoing events, retaining the original wrapper and
outer-context secret. Sender confirmations, reopen deduplication and clearing
work through existing paths. Quiz/timed/anonymous/add-option fields refuse rather
than being flattened. Synchronized vote-secret lookup and updates remain unported;
no fresh live acceptance.
[Own-device polls](docs/protocol/own-device-poll.md).

2026-10-11 — **Implemented/offline:** `actions --preview-session` now derives
and discards authenticated companion refresh candidates from one saved snapshot.
Same-key candidates archive receiving state; changed-key candidates start fresh.
Existing ADV/membership guards report concrete refusals, including returning-device
checks. No key, list, pending refresh, journal or network mutation. Full native
admission and fresh live application remain unverified.
[Companion preview](docs/protocol/companion-attestation-admission.md#companion-session-preview--2026-10-11).

2026-10-11 — **Implemented/offline:** authenticated own-device text/media
echoes now correlate to an existing normal outgoing intent by unique saved wire
ID and exact/authenticated chat association. They retain the original wrapper
and advance the held session without a duplicate NewMessage or any send-status
change. InFlight/DeliveryUnknown, immutable intent, IDs and waits stay intact.
Cross-source/placeholder reconciliation and fresh live echo acceptance remain open.
[Own-device echo correlation](docs/protocol/own-device-echo.md).

2026-10-11 — **Implemented/offline:** held, pinned own-device sessions now
synchronize supported ordinary media into outgoing events with sealed attachment
references and sender confirmations. Existing explicit retrieval, expiry and
clearing apply to the retained original wrapper. No automatic download or new
send intent. Bootstrap, control/view-once synchronization, echo reconciliation
and fresh live acceptance remain open.
[Own-device media](docs/protocol/own-device-media.md).

2026-10-11 — **Build fix:** TCP/Noise now compiles without Signal crypto.
Six stateful transport modules require their existing Signal dependencies;
the lightweight socket/login/handshake build stays available. CI checks TCP,
TCP/Noise (261 local tests), and the minimal three-feature messaging build.
No account-state or wire-format changes and no fresh live validation.
[Feature boundaries](crates/whatsapp-api/README.md#features).

2026-10-11 — **Implemented/offline:** remote status confirmations now omit
sender type; the actor adds it only for participants matching saved own PN/LID,
including own companions. Bp1.A05 is traced through parsed fromMe and native
BO0, with all direct field writers retained. Peer ownership/alias claims do not
select this branch. Invalid own-LID authority refuses without repair. Broader
status admission, broadcast-list class and fresh live acceptance remain open.
[Ownership flow, contextual actor and source/guard tests](docs/protocol/status-ownership.md).

2026-10-11 — **Implemented/offline:** message delivery/read/opened confirmation
addresses now match native primary devices: PN zero uses JID_PAIR and LID zero
uses AD. Factory/subtype and direct writer closure refute the old attribute-copy
assumption. Companion devices and raw incoming routes remain intact; durable
LID reads settle plain ACKs after reopen without uncertain replay. Full native
admission, hosted/interop and fresh live acceptance remain open.
[Factory, writer closure and regression/recovery checks](docs/protocol/message-primary-device.md).

2026-10-11 — **Implemented/offline / evidence correction:** status delivery
confirmations no longer send sts=65535. Exact Cp2.A06 DEX loads signed -1;
Cp2.A00 emits sts only for a positive long. The old signed-literal explanation
was wrong and is replaced with full descriptors and independent REA evidence.
Original IDs, poster device and sender/class fields remain intact. Bp1.A05
producer policy, primary device wire normalization and live acceptance remain open.
[Exact producer/gate, runtime fix and trace boundaries](docs/protocol/status-receipt.md).

2026-10-11 — **Implemented/offline:** identity, device-list, count, group-dirty
and privacy-token ACKs now share the APK-derived zero-device PN/LID user-pair
conversion with receipt ACKs. Literal-wire regressions reproduced identity and
all four other notification failures. Durable control effects and nonzero device
routes remain intact; the identity reply diagnostic no longer falsely says its
ACK is deferred. Full class grammar/native codecs and live acceptance remain open.
[Shared emission, source evidence and regression coverage](docs/protocol/receipt-primary-ack.md).

2026-10-11 — **Implemented/offline:** receipt ACK `to`/`participant` now use
JID_PAIR for decoded binary zero-device PN/LID users. Complete writer override
censuses distinguish these users from primary DeviceLid objects, which may still
require AD. Input capture, nonzero device paths and the global encoder remain
intact. Hosted/text-zero/JNI class fidelity and live acceptance remain open.
[Writer census, runtime correction and reproduction](docs/protocol/receipt-primary-ack.md).

2026-10-11 — **Implemented/offline:** APK AD JID decoding now uses LID/hosted
flag bits instead of inventing an agent from other flag values. The encoder
refuses nonzero agents in AD fields. The actual Java reader omits a zero device
suffix before JID construction, so zero-component binary AD user recipients
are admitted; literal text :0 remains a device-class mismatch. Native JNI/interop
codec paths and full self-status handling remain open; no live request.
[Decoder/writer evidence, correction and checks](docs/protocol/ad-jid-decoding.md).

2026-10-11 — **Implemented/offline:** receipt recipient admission now follows
UserJid rather than generic Jid. Recognized group/status/device recipients are
omitted from ACKs and no longer hide matching direct/group delivery/read/played
facts. Generic participant routing and original chat/device/mapping guards remain
separate. Full special grammar and self-status handling remain unfinished; no
live request or account-state migration.
[Native class evidence, runtime and checks](docs/protocol/receipt-user-recipient.md).

2026-10-11 — **Confirmed static / documentation correction:** incoming status-17
self-read submission and common state pools are now separated by exact branch
targets. Failed A09 eligibility reaches common state processing; qualifying
messages reach the separate grouped submission/local status-16 path. Rechecking
the contact policy corrected prior prose: an absent contact is allowed, while
the premium-message query applies to a present smb: contact. Original structural
bytes and complete REA smali agree. Runtime self-read handling remains unported;
no account or network change.
[Worker paths, corrected policy and reproduction](docs/protocol/self-read-worker.md).

2026-10-11 — **Confirmed static / implemented analysis tooling:** exact DEX
virtual-override lookup now follows superclass ancestry and encoded virtual
method declarations. The pinned APK contains four A0l()Z declarations across
101,453 physical class definitions: base true, three subtype overrides false.
Shared smali parsing now refuses malformed declarations/body extents instead of
silently omitting them. Complete self-read state/submission effects, native
message-class mapping and effective ordinary-chat preference checks remain open.
No runtime/account/network change. 82 xtask tests, strict workspace Clippy and
Rustdoc, CLI build/help and pinned-APK scan/refusal checks passed.
[Source, override census and update workflow](docs/protocol/self-read-eligibility.md).

2026-10-11 — **Implemented/offline:** receipt fact routing now distinguishes
nullable optional addresses from raw attribute presence. Namespace-invalid
participant/recipient fields follow the native null path instead of hiding
matching direct/group observations. Recognized qualifiers and original chat,
message-ID, mapping and addressed-device guards remain effective. Raw capture,
read policy, keys, requests and waits stay intact. Full special JID subtype
rules/self-status handling and fresh live projection remain unported/unverified.
No network request. 917 workspace tests, strict Clippy/Rustdoc and CLI checks passed.
[Native accessors, runtime and tests](docs/protocol/receipt-optional-addresses.md).

2026-10-11 — **Implemented/offline:** structurally valid companion attestation
metadata now skips primary CPB policy, matching the native early Bjo return.
Known same-key refresh and signed ADV changed-key/returning-device application
retain their exact original-state, prekey, pin, membership and atomic guards.
Inspection preserves failed/null attestation verification; preview remains
read-only and companion admission there remains unported. Native C0F state,
non-E2EE routing and fresh live application remain unfinished/unverified.
No network request. 915 workspace tests, strict Clippy/Rustdoc and CLI checks passed.
[Source, runtime guards and verification](docs/protocol/companion-attestation-admission.md).

2026-10-11 — **Implemented/offline:** receipt ACKs now share strict receipt
kind and network namespace checks. A present non-text type no longer defaults to
delivery; unknown namespaces cannot become ACK destinations or optional fields.
Bare lid/broadcast aggregate targets skip before time parsing. Unsupported
receipts remain captured and reception continues. Keys, reservations, IDs, read
policy and waits stay intact; specialized JID grammar and fresh live acceptance
remain unported/unverified. No network request.
913 workspace tests, strict Clippy/Rustdoc and CLI checks passed.
[Source, runtime behavior and regressions](docs/protocol/receipt-ack-namespace.md).

2026-10-10 — **Implemented/offline:** matching queued read receipts now share
native extra-ID lists. Per-message selectors and original saved wire stay intact;
all batch members reserve atomically before writing, settle on the matching root
ACK and never replay unknown writes. Group member devices, destinations and
played/retry receipts remain separate. Public queue counts still count IDs.
Fresh live batch ACK/UI read indicators and full native privacy/read-self policy
remain unverified/unported. No network request.
911 workspace tests, strict Clippy/Rustdoc and CLI checks passed. New group read
attributes also follow the native order; historical wire remains unchanged.
[Native shape, recovery and API scope](docs/protocol/read-receipt-batching.md).

2026-10-10 — **Implemented/offline:** primary attestation no longer causes a
blanket identity-refresh refusal. Verified same-key hybrid/non-E2EE metadata
permits guarded session refresh without an override child; structurally valid
changed-key metadata follows the APK E2EE fallback even when verification fails.
Classic signed-prekey, original-state, policy and own-identity guards still apply;
invalid same-key, ambiguous/missing byte fields, PQ and companion metadata defer.
Inspection preserves failed verification and preview remains read-only. Native
attestation-state storage and non-E2EE routing remain unfinished. No live request.
908 workspace tests, strict Clippy/Rustdoc and CLI checks passed.
[Native branches, runtime scope and tests](docs/protocol/primary-attestation-admission.md).

2026-10-10 — **Implemented/offline tooling:** whole-APK SHA-256 file indexing
and fast cached diffs now cover every non-directory ZIP entry. The committed
2.26.37.71 baseline lists 14,967 files without retaining file bodies; new releases
can report added/removed/changed paths and hashes without the old APK. Existing
reviewed topic/remapping tools then locate methods and affected Rust files.
[File-diff workflow and validation](docs/protocol/apk-file-diff.md).

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
[Multi-target receipt evidence and runtime](docs/protocol/offline-aggregated-receipts.md#key-based-multi-target-receipts--2026-10-10).

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
[Identity-auth admission and static evidence](docs/protocol/identity-auth.md).

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
[Identity-auth evidence and inspection](docs/protocol/identity-auth.md).

Validation: 896 workspace/all-feature tests, strict Clippy, warnings-denied
Rustdoc, default CLI/API-symbol tests, CLI-reference regeneration and rebuilt
experimental CLI passed. Offline CLI inspection of one saved refresh reports
the new field as null; before/after store summaries are equal. No qualifying
identity-auth sample or live verification is claimed. The default build retains
the existing unused clock::observe_login warning. Repository hygiene passed
2,537 local links and 128 checksum-verified APK bundles / 27,307,426 bytes.

2026-10-10 — **Implemented/offline:** ordinary receipt list selection now matches
Cmk.A03: only a sole list child expands message IDs. A list beside business
metadata, another list or unrelated children selects only root ID. Previously
Rust could add receipt facts to an extra intent the native branch did not select.
The shared direct/group helper is corrected; encrypted-store tests prove the
failure before the fix and duplicate-free reopening afterward.
[Parser selection and evidence](docs/protocol/direct-receipts.md#single-list-branch-correction--2026-10-10).

A single bounded 20-second receive window on the preceding aggregate commit
19ca798 authenticated and stopped cleanly, with no queued sends/actions/receipts,
no automatic reconnect or read marking. Registration summary, generation,
journal head and action/receipt summaries stayed unchanged. Nine checkpoints
were retained. Fresh aggregate/list projection remains unverified.
[Redacted live report](docs/protocol/offline-aggregated-receipts-live.json).

Validation: 894 workspace/all-feature tests passed (753 library, four codec,
nine crash, 54 CLI, one API-symbol and 73 tooling tests). Strict Clippy,
warnings-denied Rustdoc, 25 default CLI tests plus API-symbol coverage,
CLI-reference regeneration and the experimental CLI build passed. The default
build retains the existing unused `clock::observe_login` warning. No live request
was made after the list-selection change; the preceding window is recorded
separately above. Repository hygiene passed 2,521 local links and 128
checksum-verified APK bundles totaling 27,307,426 bytes.

2026-10-10 — **Implemented/offline:** direct/group offline device aggregates
now use inner message_id and independent entry actor/status/required time.
The shared handler applies the entire batch atomically through existing original
target guards; malformed input or late saved-map failure leaves no partial facts,
row times or journal events. The unported key-based aggregate stays raw instead
of being misattributed to root ID. REA/JADX now reads the pinned APK, and complete
Evidence plus exact DDF methods are saved. Native primary cleanup and complete
receipt/status parity remain unfinished; fresh live aggregates are unverified.
[Evidence, runtime scope and reproduction](docs/protocol/offline-aggregated-receipts.md).

Validation: 893 workspace/all-feature tests passed, including five new encrypted-store
aggregate tests. Strict all-target Clippy, warnings-denied Rustdoc, default CLI
tests and CLI-reference regeneration passed. The default build retains the
existing unused `clock::observe_login` warning. No live request was made for
this chunk; fresh server aggregate projection remains unverified. Repository
hygiene passed 2,506 local links and 128 checksum-verified APK research bundles
totaling 27,307,426 bytes.

2026-10-10 — **Implemented/offline:** direct participant-qualified delivery,
read and played receipts now retain the actual recipient device when the
saved chat and immutable original send target both match. Authenticated PN/LID
mapping can join namespaces at the same device ID; current membership cannot
fill historical target gaps. Previously this whole direct route was discarded.
Independent facts and server times survive restart without duplicate events;
send progress, attempts, uncertainty and legacy lab summaries are preserved.
No fresh live participant-qualified receipt has validated this extension.
[APK branch, guards and inspection](docs/protocol/direct-receipts.md#direct-participant-device-receipts--2026-10-10).

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
[Exact branches, scans and source](docs/protocol/receipt-message-exclusion.md).

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
[Evidence and inspection](docs/protocol/receipt-fact-time.md).

Live receive validation: one 20-second receive-only window authenticated
and stopped cleanly (33.67 seconds total). Actions 21, receipt rows 9 and
eligible outbox 0 remained unchanged; generation/registration/journal-head
summaries stayed equal. Only unsupported controls appeared; no qualifying
receipt projection or decrypted message was observed. Automatic reads,
outbox sends and reconnects were disabled. Summary/count equality is not
all-record byte equality. [Redacted result](docs/protocol/receipt-fact-time-live.json).

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
[Evidence and exact scan boundaries](docs/protocol/companion-addition-observers.md).

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
remain unknown. [Evidence and reproducible recipe](docs/protocol/companion-addition-group-state.md).

Validation: exact APK export check verified 13 topics / 18 files / 103,661
bytes; cached self-diff reports unchanged retained topics and metadata.
CLI-reference regeneration, OpenAPI/library-symbol validation, formatting and
whitespace checks passed. Repository hygiene checked 2,463 local links and
126 bundles / 26,527,619 bytes. No Rust runtime changed; no fresh workspace
runtime test or live account operation is claimed.

# Status — 2026-10-07

Returning companion directory addition (2026-10-09, **implemented/offline**):
an already pinned but unlisted remote companion now requires full signed newer
ADV for equal or changed keys. Membership/provenance/hash invalidation/pending
sync and session/history commit atomically under the original guard and policy.
Listed equal-key shortcuts and existing indices remain unchanged. actions
--show reports ready_addition_pending. Own/teardown/collision/overflow/old ADV
and stale state refuse. No first-seen identity or canonical alias is inferred;
fresh live restoration remains unknown.
Validation: 885 workspace/all-feature tests, strict Clippy, Rustdoc,
default CLI/API symbols, CLI reference and formatting passed. Bounded
28-topic / 33-file APK export reproduces exactly; cached self-diff unchanged.
Live: one 20-second receive-only window authenticated/stopped cleanly;
actions/receipts/outbox/generation/registration/head unchanged. Six additional
checkpoints; no qualifying restoration or new application claimed.
[Trace and limits](docs/protocol/companion-directory-addition.md).


Changed companion directory application (2026-10-09, **implemented/offline**):
guarded changed classic keys now validate both ADV signatures against the
actual candidate and independent primary, with current E2EE/raw-ID/membership
baseline. Fresh session and old encrypted transition history commit once with
applicable cleanup and newer-ADV pending sync. Existing device index/primary
are preserved. Shared ADV floor/expected comparisons now use APK signed Java
long semantics. actions --show adds companion_changed_key_authority. Own,
unsupported metadata, first-seen and primary/raw-ID changes stay deferred;
fresh live application unknown.
Validation: 882 workspace/all-feature tests, strict Clippy, Rustdoc, default
CLI/API symbols and CLI reference passed. 19-topic / 24-file source export
reproduces exactly; cached self-diff unchanged.
Live: one 45-second receive-only window authenticated/stopped cleanly;
actions/receipts/outbox/generation/registration/head remained equal. Eight
additional checkpoints, no qualifying companion reply or new application.
[Trace and limits](docs/protocol/companion-changed-key-refresh.md).


Known companion same-key refresh (2026-10-09, **implemented/offline**): guarded
classic directory replies now refresh an already listed companion with the
same held key under apk_authenticated. APK 11T.A0D returns before ADV parsing
on this subset; a unique nonnull byte body is required, not claimed verified.
Current list/primary binding, own identities and exact commit snapshot stay
protected. Old receiving ratchet is archived, device-list/event/pin state
unchanged, repeat/reopen recovery once-only. Changed keys and unsupported
metadata remain deferred. actions --show adds companion_same_key_authority.
Fresh live application unknown.
Validation: 878 workspace/all-feature tests, strict Clippy, Rustdoc,
default CLI/API symbols and CLI reference passed. Exact 10-topic / 15-file
APK export reproduced and cached self-diff unchanged; hygiene passed.
Live: one 45-second receive-only window authenticated/stopped cleanly;
actions/receipts/outbox/generation/registration/head remained equal. No new
qualifying companion reply, no live application claimed.
[Evidence and boundaries](docs/protocol/companion-same-key-refresh.md).


Companion ADV serialization (2026-10-09, **implemented/offline**): signature
input now matches APK GsB parse/serialize, including normalized known fields and
retained lite unknown fields/groups. Original-byte verification was our bug.
Azc NON_E2EE=2 is recognized in the local schema/parser; E2EE receiving admission
remains separate and unchanged. Independent 637-case Java oracle and incoming
reordered companion fixture cover the correction. No qualifying live companion
input is claimed. [Evidence and limits](docs/protocol/adv-details-serialization.md).
Validation: all 874 workspace/all-feature tests, strict Clippy, Rustdoc, default
CLI/API symbols/reference and proto-check passed. All 637 Java vectors reproduced;
10-topic / 15-file / 113,397-byte APK export reproduces and self-diffs unchanged.
Hygiene passed (2,423 links / 122 bundles).
Live: one 45-second receive-only window authenticated/activated/stopped cleanly;
actions/receipts/outbox/generation/head stayed equal, 8 new inbound checkpoints.
No qualifying companion ADV or decrypted message; live branch parity unknown.


Same-key primary refresh (2026-10-09, **implemented/offline**): a guarded classic
reply carrying selected-user force_override_e2ee_state now refreshes under the
existing apk_authenticated policy. Native admission is invariant across unknown
state/property values for this subset. Old receiving ratchet is archived inside
the fresh session; pin/transition/verification/sender-key state and event head
stay unchanged. No override is invented or sent. Inspection adds nullable
force_override_e2ee_state. No-child/companion/metadata cases remain deferred;
fresh live application is unverified. [Evidence and limits](docs/protocol/same-key-refresh.md).
Validation: 872 workspace/all-feature tests, strict Clippy, Rustdoc, default
CLI/API-symbol and reference checks passed. The 6-topic / 11-file / 97,899-byte
source bundle reproduces exactly and self-diffs unchanged; hygiene passed
(2,409 links / 121 bundles).
Live check: one 45-second receive-only window authenticated/stopped cleanly;
actions/receipts/pending/account generation/journal head stayed equal, ten new
inbound checkpoints. Historical read remains untracked with server override
absent and applied=false. No new qualifying reply; live application unknown.
[Redacted validation](docs/protocol/same-key-refresh.md).


Identity ACK follow-up (2026-10-09, **confirmed static / observed**): legacy
arg1=76 now traces through 0y0.A00 → 0y2.A02 → Cp2.A05 → 1Rd.A04.
The 3-topic / 8-file / 177,076-byte source bundle reproduces exactly. One
45-second saved-account receive-only window authenticated/stopped cleanly;
actions/receipts/eligible outbox/account generation/journal head stayed equal,
with eight new inbound checkpoints. No qualifying identity control arrived;
server ACK acceptance remains unknown. [Evidence and validation](docs/protocol/identity-notification-ack.md).


Identity notification ACK (2026-10-09, **implemented/offline**): APK 37.71
identity dispatch schedules its worker without registering ACK deferral. The
actor now durably handles supported controls then ACKs original metadata,
independently of key adoption. Fresh redelivery can repeat that ACK; uncertain
key reads retain their reservation. Startup does not reconstruct control ACKs.
Live qualifying-control acceptance remains unverified.
Validation: 869 workspace/all-feature tests and strict Clippy passed. The
14-topic / 19-file / 186,462-byte source bundle reproduces exactly and cached
self-diff is unchanged. Warning-denying Rustdoc, default CLI/API symbols, CLI-reference regeneration,
formatting, whitespace and hygiene passed (2,395 links / 119 bundles).
[Evidence and boundaries](docs/protocol/identity-notification-ack.md).


Identity-refresh application (2026-10-09, **implemented/offline**): the actor
now adopts a verified changed-primary classic reply under the existing explicit
apk_authenticated receive policy. Original notification guards, exact session
bytes, saved account/reply and own PN/LID are rechecked in the atomic commit.
Old pin/session, transition invalidations and once-only application history
are preserved; startup recovers the reply-settlement cut without IQ replay.
`actions --show` adds applied as a historical fact. Same-key, companion,
attestation/PQ/identity_auth admission and identity ACK remain unported.
[APK evidence, inspection and boundaries](docs/protocol/identity-refresh-application.md).

Validation: all 867 workspace/all-feature tests, strict Clippy, Rustdoc, default
CLI/API-symbol and reference checks passed. The 8-topic / 13-file / 119,638-byte
APK bundle reproduces exactly and self-diffs unchanged. One receive-only
45-second window authenticated and stopped cleanly with sends, automatic reads
and reconnects disabled: actions/receipts/eligible outbox and journal head
were unchanged. Eight inbound checkpoints were added, with 11 unsupported
controls and no decrypted message. Historical untracked read stays applied=false;
no new qualifying reply or live changed-key adoption is claimed.


Attestation storage trace (2026-10-09, **confirmed static**): native schema
defaults to NOT_SET; a conditional hosted-account backfill promotes primary
rows to E2EE. State replacement/update and own-account/gossip producers are
kept distinct. Admission properties 28527/29847 have baked false entries;
effective account values remain unknown. No Rust state backfill, session
adoption, identity ACK or network request is performed.
[Evidence and reproducible APK comparison recipe](docs/protocol/attestation-state.md).

Validation: the 19-topic / 24-file / 1,266,707-byte bundle reproduces
exactly; cached self-diff verifies all topics and four metadata artifacts
unchanged. CLI-reference regeneration, OpenAPI/library-symbol checks,
formatting, whitespace and hygiene passed (2,372 links / 117 bundles).
Research/docs only: no fresh workspace runtime or live test is claimed.


Attestation byte verification (2026-10-09, **confirmed static / implemented offline**):
`actions --show` now verifies selected-device non_e2ee_attestation using APK
prefix 06 07 plus original protobuf bytes. Account-type field/enum and declared
type checks are retained; a 1,502-case independent Java descriptor oracle checks
closed-enum/varint/group semantics. Valid signatures remain separate from held
pins, original guards and native admission. Stored attestation state, full CPB
policy, adoption and identity ACK remain unported. [Evidence and limits](docs/protocol/attestation-verification.md).

Validation: 862 workspace/all-feature tests passed, plus strict Clippy,
warning-denying Rustdoc, default CLI (25 plus API-symbol), CLI-reference
regeneration, formatting and whitespace. All 1,502 Java oracle results were
independently reproduced. The 18-topic / 23-file / 95,100-byte APK export
reproduces exactly and self-diffs unchanged. Hygiene passed (2,344 links / 116
bundles). The historical primary-LID reply reports attestation absent with
unchanged store summary; no new network request or identity adoption occurred.


Original identity refresh guards (2026-10-09, **implemented/offline**):
new notification reads capture immutable pin, exact-device authenticated
transition, primary-user version and account generation. Delayed A->B->A replies
remain stale for primary and companion peers; duplicate events preserve the
baseline. `actions --show` exposes redacted guard labels and session preview
skips known stale reads. Historical missing snapshots stay untracked.
Native admission, final atomic adoption and identity ACK remain unported.
[Evidence and limits](docs/protocol/identity-refresh-guards.md).

Validation: 858 workspace/all-feature tests passed, plus strict Clippy,
warning-denying Rustdoc, default CLI (25 plus API-symbol), CLI-reference
regeneration, formatting and whitespace. Hygiene passed (2,317 links / 115
bundles). Historical saved-account inspection correctly reports untracked with
a verified matching bundle and unchanged store summary; no new live query,
identity transition, candidate commit or ACK was performed.


Identity session preview (2026-10-09, **confirmed static / implemented offline**):
`actions --show ID --preview-session` derives and discards a primary classic
same-identity session without constructing a chat or changing saved state.
Held current states are archived in the candidate. Native stored attestation
states/defaults and admission branches are retained; historical Rust absence
is not assigned NOT_SET/E2EE. Companion/PQ/attestation/changed-key adoption,
stale-response guards and identity ACK remain unported. [Evidence and limits](docs/protocol/session-refresh-preview.md).

Validation: 854 workspace/all-feature tests passed, plus strict Clippy,
warning-denying Rustdoc, default CLI (25 plus API-symbol), regenerated CLI
reference, formatting and whitespace checks. The eight-topic / 13-file /
103,739-byte export reproduces exactly and self-diffs unchanged; hygiene passed
(2,309 links / 115 bundles). The existing saved primary-LID reply produced a
candidate with two archived states offline; store summary stayed equal. No
network request or candidate commit occurred. Native admission remains unported.


Identity notification key reads (2026-10-09, **confirmed static / implemented offline**):
the normal callback target is root from, with device preserved and participant
separate. Authenticated receive now queues reason=identity reads for supported
existing remote PN/LID pins, with durable event/device guards and no replay of
uncertain/refused/invalid reads. Four tests cover routes, guards, signed prior
results and an actor write cut/reopen. Response adoption, full native admission
and identity ACK lifecycle remain unported. [Evidence and limits](docs/protocol/identity-notification-key-read.md).


Validation: 852 workspace/all-feature tests passed. Strict Clippy, warnings-
denied Rustdoc, default CLI tests (25 plus API-symbol), CLI-reference regeneration,
formatting and whitespace passed. Exact export check verified 11 topics / 16 files /
102,122 bytes; checksum-verified cached self-diff found 11 unchanged topics and
four metadata artifacts. Hygiene passed (2,294 links / 114 bundles).
One 45-second authenticated receive window observed a
primary-LID read with verified signature and matching pin; no identity transition,
chat send or adoption was validated. [Redacted live evidence](docs/protocol/identity-notification-key-read-live.json).


Supplied bundle authentication (2026-10-09, **confirmed static / implemented offline**):
common direct/pairwise encryption verifies supplied signed device keys even with
an existing session. A failing-before generated-key regression proves forged
same-identity refusal; valid/no-bundle session reuse and identity policy remain.
Notification callback construction is retained through CSu.A02; its upstream
routing and native automatic refresh remain unknown/unported. No account or
fresh live operation. [Validation and limits](docs/protocol/peer-identity.md#supplied-bundle-validation-with-a-held-session-2026-10-09);
[address evidence](docs/protocol/identity-notifications.md#callback-address-construction-continuation-2026-10-09).


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
unported. No real account or live operation. [Evidence and limits](docs/protocol/bulk-prekey-processing.md).

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
operation. [Evidence and limits](docs/protocol/bulk-prekey-refresh.md).

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
runtime, account or live change. [Evidence and limits](docs/protocol/identity-notifications.md).

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
No real account or fresh live operation. [Evidence and limits](docs/protocol/prekey-count-notifications.md).

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
or peer transition observed. [Redacted evidence and limits](docs/protocol/login-identity-updates.md#bounded-live-regression-2026-10-09).

Live-check validation: the experimental CLI build, CLI-reference regeneration,
API/library-symbol checks, formatting, whitespace and repository hygiene passed
(2,186 links/108 APK bundles). One bounded live receive run completed; no fresh
workspace runtime suite was rerun for this evidence-only change. The preceding
840-test implementation validation remains separate.


Partial login identity persistence (2026-10-09, **implemented/offline tested**):
PN-only/LID-only authenticated success updates preserve the other saved namespace.
Malformed replacements/history refuse atomically without changing account state,
clock or uncertain login reservation. A failing-before regression covers the
lost own LID; no real account or live operation. [Evidence and limits](docs/protocol/login-identity-updates.md).

Validation: 840 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, formatting and whitespace checks passed. Repository hygiene
checked 2,182 local links and 108 existing APK bundles. No fresh live test.


Incoming recipient-count source (2026-10-09, **confirmed static**): root
`count` defaults only on absence; malformed present input throws from the
signed-int helper. Own group/status-envelope transfer reaches native message.A06
through Cih/CaF/Bp2, with distinct branch guards. This closes the count producer,
not receipt_user membership or primary cleanup. No runtime/live change.
[Evidence and limits](docs/protocol/message-recipient-count.md).

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
across reopen. No live operation. [Evidence and limits](docs/protocol/peer-identity.md#empty-peer-guard-on-key-queries-and-pin-inspection-2026-10-09).

Validation: 837 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, formatting and whitespace checks passed. Repository hygiene
checked 2,162 local links and 107 existing APK bundles. No fresh live test.


Attachment expiry continuation (2026-10-09, **implemented/offline tested**):
ordinary media retrieval shares the explicit incoming Java timestamp parser,
fixing event/retrieval disagreement for Unicode-decimal time. Missing-time and
missing-route refusals remain; no live download or account operation.
[Evidence and tests](docs/protocol/message-timestamps.md#attachment-expiry-continuation).

Validation: 836 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, formatting and whitespace checks passed. Repository hygiene
checked 2,159 local links and 107 APK bundles. No fresh live test.


Incoming timestamp parsing (2026-10-09, **implemented/offline tested**):
explicit valid root `t` uses APK-derived Java signed-decimal parsing and wrapping
millisecond conversion across incoming event projections. Missing/malformed
historical timestamps remain unknown; expiration and once-only journal guards
remain intact. No account or live operation. [Evidence and limits](docs/protocol/message-timestamps.md).

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
change. [Evidence and limits](docs/protocol/status-observers.md).

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
No Rust, saved-account or live behavior change. [Evidence and limits](docs/protocol/send-job-state.md).

Send-job evidence validation: exact 14-topic export (19 files/268,818 bytes),
cached self-diff, raw listener switch/branch and state-handoff assertions passed.
CLI reference, API-symbol, formatting, whitespace and hygiene checks passed
(2,138 links/106 APK bundles). Research/docs only; no fresh runtime or live test.

ACK timing (2026-10-09, **implemented/offline tested**): matching success
ACK facts retain parsed server time separately from local observation time,
using the APK's Java default-zero and wrapping semantics. Historical rows remain
unknown; duplicate terminal ACKs cannot rewrite the timestamp. The status API
schemas now describe actual ticket/progress/facts JSON. No new live send or
native receipt admission. [Evidence and limits](docs/protocol/ack-timestamps.md).

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
[Evidence and limits](docs/protocol/ack-correlation-pages.md).

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
[Evidence and limits](docs/protocol/text-before-hooks.md).

Before-hook validation: exact 21-topic export (26 files/83,350 bytes), cached
self-diff, 13 ordered dispatch/factory/body assertions, CLI reference,
API/library-symbol checks, formatting and whitespace passed. No fresh runtime
suite or live account test is claimed.

Scheduled arithmetic follow-up (2026-10-09, **confirmed static**): DEX static
wide loads/stores now have mnemonics and an APK-byte regression. D7g passes
scheduled_time plus a 30-day millisecond constant to DGf.A03. Complete expiry
semantics and receipt admission remain unknown/unported. No account/live change.
[Evidence and limits](docs/protocol/scheduled-deadline.md).

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
[Evidence and limits](docs/protocol/text-final-after-hooks.md).

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
change. [Evidence and limits](docs/protocol/text-association-after-hooks.md).

Association-hook validation: exact 22-topic export and cached self-diff, scoped
source checks, CLI reference, API/library-symbol, formatting, whitespace and
hygiene passed. No fresh runtime suite or live account test is claimed.

Storage/poll/indexing after-hooks (2026-10-09, **confirmed static**): seven
more registered entries and nine selected delegates are attributed. Writes target
ephemeral, media/thumbnail or search tables; thumbnail post-processing clears a
marker, not stored bytes/view-once content. Current mapping is 36/54 with 18
remaining after entries. Transitive receipt admission and primary cleanup remain
unknown/unported. No runtime/account/live change. [Evidence and limits](docs/protocol/text-storage-after-hooks.md).

Storage-hook validation: exact 27-topic export and cached self-diff, direct
source checks, CLI reference, API/library-symbol, formatting, whitespace and
hygiene passed. No fresh runtime suite or live account test is claimed.

Additional metadata hooks (2026-10-09, **confirmed static**): eight bucket-one
entries now have registered factory/CHJ boundaries, with inline-video, mention
and question helpers traced. Question SQL failure throws for -1; selected other
negative writes only log or discard IDs. Immediate after coverage is 29/54,
with 25 entries and transitive receipt admission still unknown/unported. Primary
cleanup remains unfinished. No runtime/account/live change.
[Evidence and limits](docs/protocol/text-metadata-after-hooks.md).

Metadata-hook validation: exact twelve-topic export and cached self-diff, CLI
reference, API/library-symbol checks, formatting, whitespace and hygiene passed.
No fresh runtime suite or live account test is claimed; see linked evidence.

Bot metadata after-hooks (2026-10-09, **confirmed static**): providers
0xf4b/0xf53 resolve to D7a/D7j. Selected delegated writers target bot_message_info,
bot_plugin_metadata and bot_message_sharing_info, discard SQL return IDs and do
not directly write receipt_user or call the native status setter. This brings
immediate after-hook mapping to 21/54; 33 entries and transitive admission effects
remain unknown/unported. Primary cleanup remains unfinished. No runtime/account/
live change. [Evidence and limits](docs/protocol/text-bot-after-hooks.md).

Bot-hook validation: exact nine-topic export and cached self-diff, CLI reference,
API/library-symbol, formatting, whitespace and hygiene passed. No fresh runtime
suite or live account test is claimed; see the linked evidence for counts.

Main-row update (2026-10-09, **confirmed static**): selector 6 reaches
the existing-row message writer; status reads B3b, affected-row count is ignored
and a missing lookup has no inserted-row fallback. Link flag A0n feeds the
computed lookup mask; origination_flags instead reads A0A, with a conditional
bit clear. Hook effects, native user-row admission and primary cleanup remain
unknown/unported. No runtime/account/live change. [Evidence and limits](docs/protocol/text-main-update.md).

Common link/span after-hook (2026-10-09, **confirmed static**): Ce8 writes
span indices and ORs A0n flag 0x10000, distinct from A08 status, then queues
reason -1 / Ahi selector 5 / scheduling 22. The update wrapper separates
before/callback/CgJ/after hooks and unsupported-type false returns. Transitive
update effects, user-row admission and primary cleanup remain unknown/unported.
No runtime/account/live change. [Evidence and limits](docs/protocol/text-link-after-hook.md).

Selected after-insert hooks (2026-10-09, **confirmed static**): 19 of 54
registered entries now have concrete provider/CHJ boundaries. Selected bodies
write metadata or delegate to helpers; no direct native status setter or
receipt_user SQL is present. Helper effects, 35 remaining entries, user-row
admission and primary cleanup remain unknown/unported. No runtime/account/live
change. [Evidence and limits](docs/protocol/text-after-hooks.md).

Text type inserter (2026-10-09, **confirmed static**): type zero resolves
through provider 0x14c1 to 1hl.BJK and optional message_text persistence.
D7J recipient-count preprocessing is now confirmed in the 13-entry before set
at index 8 / provider 0x3c5; 54 after entries are retained. Other hook effects,
full send progression, user-row admission and primary cleanup remain unported.
No runtime/account/live change. [Evidence and limits](docs/protocol/text-type-inserter.md).

Main message row (2026-10-09, **confirmed static**): the insertion callback
writes table message before the type-specific inserter. A0h(1) sets storage
type A0f; SQL status reads B3b instead. Main-row ID failures and lazy registry
resolution are retained; concrete type-zero mapping, native user-row admission
and primary cleanup remain unknown/unported. No runtime/account/live change.
[Evidence and limits](docs/protocol/text-main-row.md).

Text store insertion (2026-10-09, **confirmed static**): four topics connect
ordinary authored-message insertion to its type registry, transaction wrapper
and before/lambda/inserter/after hook order. The insertion argument 1 is not a
status assignment; void return is not durable-row proof. Concrete type-zero
inserter/provider effects and primary cleanup remain unknown/unported.
No runtime/account/live change. [Evidence and limits](docs/protocol/text-store-insertion.md).

Text store submission (2026-10-09, **confirmed static**): nine topics
connect B1l to the deferred 3Pn selector-4 store worker. Reason -1/1/22 and
scheduler 26 are not status assignments. Three setter candidates are separately
attributed to payment, ACK and unsent recovery. Full normal send progression
and primary cleanup remain unported; no runtime/account/live change.
[Evidence and limits](docs/protocol/text-store-submission.md).

Receipt count preprocessing (2026-10-09, **confirmed static**): four topics
close participant-map counts and mode-dependent local-user membership; reused
sources attribute copy/builder/ACK writes. The shared group write site is not
always literal two. All ten physical sites have immediate value attribution,
but normal-send and transitive producer authority remain incomplete/unported.
No runtime/account/live change. [Evidence and limits](docs/protocol/receipt-count-preprocessing.md).

Recipient-count producers (2026-10-09, **confirmed static**): five topics
separate status-audience size excluding the local user from conditionally written
broadcast clone-entry counts; cursor restoration loads recipient_count directly.
Ordinary send/copy writers and full native admission remain untraced/unported.
No runtime, account or live change.
[Evidence and limits](docs/protocol/receipt-recipient-count.md).

Receipt message projection (2026-10-09, **confirmed static**): four topics
close status-only existing-row updates and the conditional view-once state hook.
Multipart status counts each user's highest state against native A06; computed
downgrades log and still write. Threshold producers and complete user admission
remain unknown/unported; no runtime/account/live change.
[Evidence and limits](docs/protocol/receipt-projection.md).

Status-array renderer correction (2026-10-09, **confirmed local/static**):
the APK-derived regression exposes the old 64-byte truncation; the bounded
256-byte preview now preserves all 22 receipt-status ranks and reports omitted
bytes on larger arrays. A new pinned snapshot preserves historical exports.
No messaging/account/live change; primary cleanup remains incomplete.
[Evidence and limits](docs/protocol/receipt-admission.md#complete-status-array-rendering-2026-10-09).

Blank-receipt failure continuation (2026-10-09, **confirmed static**):
four source topics identify Cnn/0zO as message-send event logging and separate
builder result 3 from native message status 20. No direct status setter or
receipt_user access is established; helper effects and primary cleanup remain
unknown/unported. No runtime, account or live change.
[Evidence and limits](docs/protocol/message-status.md#blank-receipt-failure-logging-continuation-2026-10-09).

Offline account diagnostics (2026-10-09, **observed/local**): the original
private key enabled saved-profile receipt inspection: nine ACKed reservations
(eight read, one retry), identical logical results across reopen; no live request.
Account pending-work counts now exclude retained cancelled/failed/in-flight
history and share the pending-list progress predicate. No resend/state reset.
[Count evidence](docs/protocol/account-status-counts.md). [Receipt check](docs/protocol/receipt-ack-correlation.md#original-saved-account-inspection-restored-2026-10-09).

Key-based receipt lookup (2026-10-09, **confirmed static**): seven topics
close ordinary/add-on overrides and show missing ordinary messages select the
add-on keyed receipt join. Pending membership is distinct from delivered/user
rows; primary cleanup remains unfinished. Offline saved-account receipts check
stopped before store open because its master-key environment input was absent.
No account mutation, runtime change or live request.
[Evidence and limits](docs/protocol/receipt-key-lookup.md).

Receipt inspection correction (2026-10-09, **confirmed local**): offline
receipts inspection now diagnoses malformed saved wire/IDs with fixed redacted
errors instead of silently dropping/truncating bytes or returning an empty ID.
Reservations and outcomes remain unchanged across failure/reopen; primary
cleanup and native user-row admission remain unfinished. No live operation.
[Evidence and limits](docs/protocol/receipt-ack-correlation.md#saved-receipt-inspection-diagnostics-2026-10-09).

Read persistence release relocation (2026-10-09, **confirmed static/local**):
a separate checked recipe locates six methods; signature plus physical caller
resolves a three-way field-writer ambiguity. Four string-free topics remain
manual and prevent automatic proposal. Cached ten-topic self-diff took
0.014920 seconds; no second release/runtime/live parity is claimed.
[Evidence and limits](docs/protocol/read-receipt-persistence.md#release-relocation-2026-10-09).

Deferred blank receipt expansion (2026-10-09, **confirmed static**): eight
source topics close DNt selector 6 as status-target device-set subtraction
and blank-device insertion, not normal receipt_user seeding. Existing-target
property and abstract override boundaries remain unknown; primary cleanup
still requires actual user-row membership. No runtime/live change.
[Evidence and limits](docs/protocol/blank-receipt-deferred.md).

Read-receipt persistence (2026-10-09, **confirmed static**): ten source topics
connect local status marking to existing message-row updates and separate
chat row/sort and status-table sent positions. IOException may be logged and
swallowed after the in-memory mark; no ACK wait is established. Complete
queue/store reconciliation remains unknown/unported; no runtime/live change.
[Evidence and limits](docs/protocol/read-receipt-persistence.md).

Read-receipt submission (2026-10-09, **confirmed static**): thirteen source
topics separate grouped queueing from later per-message policy, 256-ID batches,
forced read-self and local status-16 marking. Maximum store-time is preserved;
worker/migration authority and queue atomicity remain unknown/unported.
No runtime or live change.
[Evidence and limits](docs/protocol/read-receipt-submission.md).

Read-setting observer binding (2026-10-09, **confirmed static**): sixteen
source topics connect generated observer membership to DK9 and distinguish
category/value Bsa from empty list callback C5W. Its read-work scan uses saved
read/receipt positions before played-receipt work; selection is not transmission.
Full sender/other-observer authority remains unknown; no runtime or live change.
[Evidence and limits](docs/protocol/read-setting-observers.md).

Read-setting pending lifecycle (2026-10-09, **confirmed static**): eight
source topics bind the shared 20-second main-Handler runnable to clearing all
pending settings and notifying observers, without a preference write or resend
in that branch. UI pending display differs from committed settings. Effective
state/observer authority remain unknown; no runtime or live change.
[Evidence and limits](docs/protocol/read-setting-pending.md).

Read-setting fetch and observers (2026-10-09, **confirmed static**): thirteen
source topics distinguish fetch mode 23463 from set mode 23639, retain SMAX
get/callback parsing, and identify played-receipt queueing in a settings observer.
Boolean local-set refusal and local submission versus server acceptance are
explicit. Effective settings/bootstrap/observer authority remain unknown;
no runtime or live change.
[Evidence and limits](docs/protocol/read-setting-bootstrap.md).

Read-setting wire producer (2026-10-09, **confirmed static**): a read-self
receipt ACK with a present readreceipts attribute emits arg1 event 227 and
still reaches generic ACK forwarding. Four source topics connect enabled
dispatch to the retained 14c preference consumer. Effective state and fallback
registration remain unknown; no runtime/live change.
[Evidence and limits](docs/protocol/read-setting-sync.md#read-self-ack-wire-producer-continuation).

Read-setting synchronization (2026-10-09, **confirmed static**): ten source
topics distinguish local-off-preserving discriminator 227 from validated
category-map adoption and pending-value mismatch guards. A0Q dispatches observers,
not a direct query. Effective account setting, bootstrap and complete lifecycle
remain unknown/unported; no runtime or live change.
[Evidence and limits](docs/protocol/read-setting-sync.md).

Device-receipt timestamp correction (2026-10-09, **confirmed local / APK-backed**):
Java BMP receipt times now reach earliest-positive mutable device rows rather
than remaining falsely pending. A failing-before test checks received-row
preservation through companion cleanup, rollback, immutable provenance and reopen;
normal-outbox receipt attribution is also exercised. Primary cleanup remains
unported; no real account/live change.
[Evidence and limits](docs/protocol/device-receipt-rows.md#java-receipt-time-and-companion-cleanup-correction-2026-10-09).

Ordinary text status source (2026-10-09, **confirmed static / bounded inference**):
user-input text construction uses 1Ze/1he and the NTP-preferred clock; its
plain construction boundary differs from media's explicit status 1. Seventeen
source topics retain callbacks and exclude unrelated setter workflows.
Full preparation/user-row admission and identity receipt cleanup remain unported.
[Evidence and limits](docs/protocol/message-status.md#ordinary-text-construction-and-excluded-setter-candidates-2026-10-09).

Trusted-contact timestamp parser (2026-10-09, **confirmed local / static-backed**):
parent/token times now use Java-compatible decimal-long parsing within the
existing nonnegative storage range. A failing-before regression covers BMP
digits, override/fallback, malformed-token rollback and reopening. No real
account/live change; complete native token admission remains unfinished.
[Evidence and limits](docs/protocol/token-notification-peer.md#timestamp-parser-correction-2026-10-09).

NTP clock preference (2026-10-09, **confirmed static**): nine source topics
establish separate NTP/WhatsApp baselines, offset sign, acquisition correlation,
cache/recent-sync checks and feature-controlled rejection. NTP fetching,
effective property 2225 and upstream scheduling remain unported/unknown.
No runtime or real-account/live change.
[Evidence and limits](docs/protocol/token-clock-producers.md#ntp-preference-and-acquisition-2026-10-09-continuation).

Confirmed clock runtime (2026-10-09, **confirmed local / APK-backed**):
authenticated login and server-ping time now save source/offset separately from
inferred stanza samples. Linux/Android use suspend-aware elapsed time during the
same process; restart uses the saved offset. Client mute resolution prefers this
confirmed source. Offline `clock-inspect` reports redacted provenance. NTP clock,
token scheduling and live parity remain unfinished; no real-account/live change.
[Evidence and limits](docs/protocol/token-clock-producers.md#confirmed-runtime-clock-inputs-2026-10-09).

Server-ping clock input (2026-10-09, **confirmed static/local audit**):
`iq type=get xmlns=urn:xmpp:ping` supplies envelope t through signed-long/fallback
parsing to Android arg1 selector 4. Result IQs do not use this clock branch.
Six source topics close wire provenance; downstream reply writing and Rust
clock parity remain unported. No runtime or real-account/live change.
[Evidence and limits](docs/protocol/token-clock-producers.md#server-ping-wire-producer-2026-10-09-continuation).

Token clock producers (2026-10-09, **confirmed static/local audit**):
APK `08B.A07` takes login `success@t` through `1Rd` or selector-4 Bundle time,
sets an elapsed baseline and persists client/server difference. Seven source
topics close those inputs; selector-4 wire provenance and full Rust clock parity
remain unknown/unported. Current stanza clock sampling is still an inference;
no real account or live change.
[Evidence and limits](docs/protocol/token-clock-producers.md).

Trusted-contact peer guard (2026-10-09, **confirmed local / static-backed**):
privacy-token receive now rejects empty PN/LID users before storage or ACK;
supplied empty sender_lid keeps a fixed diagnostic. Generic server-only JIDs
previously passed our namespace-only guard. A failing-before regression checks
text/Plain/AD inputs, unchanged tokens/pins/sessions and reopening; valid token
handling remains covered. No real account/live change.
[Evidence and limits](docs/protocol/token-notification-peer.md).

Token mapping consumers (2026-10-09, **confirmed static/local**):
selected table readers feed a separate specifier cache and native stable-ID list.
`00D.A0C` compares/logs values; its caller separately chooses native or fallback
return. Readiness and initial preference inputs are traced, effective account
settings remain unknown. Renderer now decodes APK shift opcodes b8/c5 with a
failing-before regression; seven source topics preserve the boundary.
[Evidence and limits](docs/protocol/token-feature-inputs.md#mapping-consumers-and-shadow-comparison-2026-10-09).

Token MobileConfig mapping (2026-10-09, **confirmed static**): provider
233 resolves to `09N`; its generated array has nonzero initial specifiers for
15491 (`0x811d83000016e2`) and 14303 (`0x811af9000014f5`). Eight source topics
preserve the binding and indexed lookup. Effective native values/readiness and
array consumers remain unknown; no runtime/live change.
[Evidence and limits](docs/protocol/token-feature-inputs.md#property-to-mobileconfig-mapping-2026-10-09).

Token Boolean provider bindings (2026-10-09, **confirmed static**):
`00D` binds `1Y4(4)` and `1Y5(0)` to its Boolean providers. Both resolve IDs
through `09M.B2y`; zero mapping returns false, nonzero calls MobileConfig
`AVn`. Six source topics preserve bindings/selectors/branches; actual mappings,
native values and effective precedence remain unknown. No runtime/live change.
[Evidence and limits](docs/protocol/token-feature-inputs.md#boolean-provider-bindings-2026-10-09-continuation).

Token feature inputs (2026-10-09, **confirmed static/local**): APK 37.71
registers properties 15491/14303 as false Booleans; its getter can use caches,
preferences or MobileConfig, so effective values remain unknown. Offline
`abprops-query --show` now reports selected returned token values and invalid
last-value flags, without default fallback or runtime activation. Twelve source
topics and regression/reopen tests preserve the evidence. No real account or
live change. [Evidence and limits](docs/protocol/token-feature-inputs.md).

Token canonicalization gates (2026-10-09, **confirmed static**): APK
37.71 normalizes type-zero phone users independently of property 15491, which
suppresses only counterpart lookup/cleanup. Separate property 14303 selects
pending-job namespace. The six-topic snapshot records incoming-byte versus
outbound timestamp getters and four physical counterpart-helper callers. Effective
flags, mapping authority and refresh runtime remain unknown/unported. No account
state or live request changed.
[Evidence and limits](docs/protocol/token-canonicalization.md).

Identity-only peer prerequisite (2026-10-09, **confirmed local**): public
query/inspection now refuses an empty peer user before reading or saving action
state; shared primary-address matching also refuses server-only values. The
regression failed before the fix and covers PN/LID, Plain/AD encodings, unchanged
pins/sessions/checkpoints and reopening without replay of an uncertain query.
This is our input guard, not a WhatsApp refusal; no real account or live change.
[Evidence and limits](docs/protocol/peer-identity.md#empty-peer-user-refusal-2026-10-09).

Receipt ACK state boundary (2026-10-09, **confirmed static/local**): APK
37.71 DDH preserves ACK status 10/12 and zero time; selected ordinary items
can reach 15A.A07 with a null callback through Cmn. Permanent-failure status
21 skips that update. Deferred jobs preserve key direction/status/time and use
a separate processing budget. The six-topic snapshot closes model/deferred-job
inputs, not complete persistence or primary user-row authority. Rust ACKs still
settle receipt reservations separately from peer facts; no runtime or live change.
[Evidence and limits](docs/protocol/receipt-ack-state.md).

Receipt ACK handler bindings (2026-10-09, **confirmed static/local**): APK
37.71 N12 providers resolve read selector 86 to 14c and state selector 262
to 17J. Read processing checks chat type 5; state processing enqueues Cmn
with arguments 0/5000/0. The offline switch renderer now retains every declared
case and rejects malformed tails, closing its former 64-case visibility gap.
The eighteen-topic snapshot preserves concrete worker boundaries; full persistence
and live parity remain unknown. No account state or live request changed.
[Evidence and limits](docs/protocol/receipt-ack-handlers.md).

Specialized receipt ACK producers (2026-10-09, **confirmed static**): APK
37.71 read ACKs produce selector 86 bundles; played/server-error produce selector
262 models with arguments 10/12 and zero time. The fourteen-topic snapshot
traces arg1 dispatch to N12 A0d/A0Z or registered/fallback handlers. Concrete
application and persistence remain unknown; these are not peer timestamps or
receipt_user membership. No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/receipt-ack-special.md).

Receipt ACK dispatch (2026-10-09, **confirmed static**): APK 37.71 generic
ACKs remove full Cb1 callback keys under a map lock, then separately handle ID
queues. Read-job registration normalizes only device-zero address forms; duplicate
registration replaces callbacks after logging. The ten-topic snapshot retains
null versus original-node callback payloads. Full queue ownership, timeout/error
lifecycle and receive effects remain unknown. No runtime, account state or live
request changed.
[Evidence and limits](docs/protocol/receipt-ack-dispatch.md).

Receipt ACK participant guard (2026-10-09, **confirmed static/local**): APK
37.71 Cb1 compares nullable participants. Rust now rejects a participant-less
ACK for a group read/retry reservation; incomplete error ACKs also leave the
saved outcome unknown. Regression coverage checks restart/no replay and complete
matching ACKs. The former CZH attribution is corrected and retained as a negative.
No private state or live request changed; full primary cleanup remains unfinished.
[Evidence and limits](docs/protocol/receipt-ack-correlation.md).

Linked metadata observers (2026-10-09, **confirmed static**): APK 37.71
case 26 invokes a no-payload Bjk callback. A physical scan finds thirteen empty
concrete bodies and two state-update consumers; transitive effects/registration
remain unknown. The five-topic snapshot also names the browser/desktop flag
and distinguishes view lookup from raw-cache lookup. No runtime, account state
or live request changed; this is not identity-verification or receive-replay proof.
[Evidence and limits](docs/protocol/linked-metadata-observers.md).

Linked-device runnable writes (2026-10-09, **confirmed static**): APK 37.71
DLo case 23, DM7 case 2 and DNh case 20 write history access, last-active time
and nickname. A three-topic anchored snapshot completes immediate classification
of the twenty physical cache stores; transitive authority remains unfinished.
Last-active mutates a record before SQL, and only its separate companion
preference update has a greater-than check. No runtime, account state or live
request changed.
[Evidence and limits](docs/protocol/linked-runnable-writes.md).

Linked-device metadata caches (2026-10-09, **confirmed static**): APK 37.71
place/instrumentation writers update SQL, null only the view map and mutate
one existing raw-map record. The five-topic snapshot distinguishes that behavior
from GEu's broader store-reset context. Location requirements, metadata authority,
remaining runnable writers and complete linked-device trust remain unknown.
No runtime, private account state or live request changed.
[Evidence and limits](docs/protocol/linked-metadata-cache.md).

Linked-device cache writes (2026-10-09, **confirmed static**): APK 37.71
has twenty physical raw/view cache writes across fourteen methods. The six-topic
snapshot traces metadata writes, logout marking versus row deletion, own-alias
selectors and bounded PN/LID migration resets. Full remaining writer/caller
authority and commit/rollback semantics remain unknown; no cache reset implies
verified identity or removal permission. No runtime, account state or live
request changed; linked-device receipt handling remains unfinished.
[Evidence and limits](docs/protocol/linked-cache-writes.md).

Read-job saved linked set (2026-10-09, **confirmed static**): APK 37.71
requires loaded Me and derives its set from saved devices, excluding positive
logout times. The nine-topic snapshot traces SQL loading and bounded own
LID-to-PN device conversion with existing-PN collision preference. Set membership
is not key verification; full ingestion/deletion authority, cache invalidation
and read-self fanout remain unknown. No runtime, account state or live request
changed; primary cleanup remains unfinished.
[Evidence and limits](docs/protocol/read-linked-set.md).

Outgoing read-job trace (2026-10-09, **confirmed static**): APK 37.71
SendReadReceiptJob uses Cp2's shared receipt builder and extra-ID list construction.
Its outgoing read/read-self selection differs from the incoming factory policy;
job admission also checks IDs, status age and a saved set. The eleven-topic
snapshot supersedes wire-format inference. Rust still sends one explicit ID
per receipt; complete policy, linked-device scope and scheduling remain unfinished.
No runtime behavior, account state or live request changed.
[Evidence and limits](docs/protocol/read-receipt-job.md).

Receipt read-policy classifiers (2026-10-09, **confirmed static**): APK 37.71
uses a configured JID-user list, chat codes -1/-3 and contact-addon presence.
The twelve-topic snapshot retains dependent-account age-state checks and a
predicate that can schedule a refresh before returning true. Effective state,
classifier writers and refresh completion remain unknown; full user-row
admission and primary cleanup remain unfinished. No runtime, account state
or live request changed.
[Evidence and limits](docs/protocol/receipt-read-classifiers.md).

Receipt factory read policy (2026-10-09, **confirmed static**): APK 37.71
uses read_receipts_enabled default true plus chat, contact and account predicates.
The eleven-topic snapshot retains literal smb:/ent: prefix checks, premium-message
existence queries, dependent-account preference and the entry thread guard.
Effective preferences/classifiers and settings synchronization remain unknown;
full user-row admission and primary cleanup remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](docs/protocol/receipt-read-policy.md).

Receipt state-only suppression (2026-10-09, **confirmed static**): APK 37.71
C8L selects incoming keys and mapped state arguments 13/16/17. Its state-only
caller skips A07 when the chat last-read message row is at least message A0j;
the missing-chat-model getter returns 1. A two-topic snapshot and eleven-write
inventory retain the immediate boundary. Full chat read-state lifecycle,
self/user-row admission and primary cleanup remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](docs/protocol/receipt-state-suppression.md).

Receipt state-only admission (2026-10-09, **confirmed static**): APK 37.71
keeps message-key direction separate from receipt status: read-self forces false,
while played-self retains the recipient-presence branch. The ten-topic snapshot
traces A07 advancement, played eligibility and core persistence. A07 true does
not prove every downstream database update succeeded; the base A0V helper tests
chat kind, not a view-once flag. Full self/user-row admission and primary cleanup
remain unfinished. No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/receipt-state-only.md).

History mode provenance (2026-10-09, **confirmed static**): APK 37.71
BMv reads positive registration_device_id from preferences; BNm tests loaded
Me presence. These are separate from own-user equality, registration state and
linked-device authentication. An eight-topic snapshot retains mode/getter/setter
and the bounded completion-handler source. Full pairing/crypto and receipt_user
admission remain unfinished. No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/history-mode.md).

History queue admission (2026-10-09, **confirmed static**): APK 37.71
stages history chunks in the peer handler after own-user and companion-mode
checks. SQL insertion-constraint recovery updates the original message ID;
queue selection and null/empty binding differences are retained. The four-topic
snapshot preserves full staging/store methods. Upstream crypto, linked-device
trust, media authentication and receipt_user creation remain unfinished.
No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/history-queue.md).

Mapping import entry points (2026-10-09, **confirmed static**): APK 37.71's
remaining conversation-mapping callers are chat-transfer import and the initial
history-bootstrap branch. The immediate history inputs are bounded file reads
or inflated inline chunks; nonce presence/storage is not demonstrated nonce
verification. A four-topic snapshot preserves the callers and full worker.
Upstream chunk/transfer authentication and receipt_user admission remain unknown.
No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/mapping-admission.md).

Chat counterpart provenance (2026-10-09, **confirmed static**): APK 37.71
uses a HashBiMap populated from distinct positive original chat-JID rows and
from parsed conversation account-LID/phone pairs. Reverse lookup, chat deletion,
cache clearing and a mapping-sync consumer are retained in a twelve-topic snapshot.
The backup restore caller is distinct from direct wire admission; upstream trust,
full sync receiver and receipt_user membership remain unfinished. No runtime,
account state or live request changed.
[Evidence and limits](docs/protocol/chat-counterparts.md).

Receipt counterpart selection (2026-10-09, **confirmed static**): APK 37.71
8G8.A02 expands selected JID rows only through the migration gate and the
0Yo.A0B(input, false) mapping path; its comparison uses object identity.
JID-row materialization does not establish receipt_user membership. The eleven-topic
snapshot retains the mapping/gate and full untraced chat-cache loader. Mapping
producers, full user-row admission and primary cleanup remain unported.
No runtime, account state or live request changed.
[Evidence and limits](docs/protocol/receipt-counterparts.md).

Receipt metadata provenance (2026-10-09, **confirmed static / inferred lifecycle**):
APK 37.71 has four physical route-flag writers and hydrates 1ic mentions from
message_mentions, separately from receipt_user. Clearing its list preserves
lookup bit 1 and the route flag; BotGroupMetadata attachment explicitly sets or
clears a separate bit. The fourteen-topic snapshot preserves these sources and
remaining constructor inventories. Complete user-row admission and primary
cleanup remain unported. No Rust runtime or account state changed.
[Evidence and limits](docs/protocol/receipt-metadata.md).

Receipt metadata route (2026-10-09, **confirmed static**): APK 37.71 special
classification requires chat exclusion, flag/first qualifying metadata user,
message A02 and descriptor absence. It does not count recipients; A0z bypasses
metadata selection only. The fourteen-topic snapshot retains the helper chain.
Disassembler opcode-c2 naming is corrected with a literal-byte regression.
Metadata producers, full user-row admission and primary cleanup remain unported.
No messaging runtime, saved identity or reservation changed.
[Evidence and limits](docs/protocol/receipt-route.md).

Ordinary cursor status (2026-10-09, **confirmed static**): APK 37.71 0VA.A00
reads status into the virtual message setter and loads server receipt time
without enum conversion or minimum selection. Two physical cursor entry points,
the type factory and one status-1 media preparation source are retained in a
five-topic snapshot. Full outer SQL/cache and user-row creation remain unknown;
no Rust runtime, reservation or account state changed.
[Evidence and limits](docs/protocol/message-loader.md).

FStatus conversion (2026-10-09, **confirmed static**): APK 37.71 7z7.A00
maps named 1cE enum ordinals with media/subtype conditions; 7zY.A01 keeps
native 178 integers separate. The ten-topic snapshot retains the transition
matrix and all six direct enum writes. This narrows the earlier restoration
trace: ordinary SQL status loading and universal outgoing initial status remain
unknown. No Rust runtime or account state changed.
[Evidence and limits](docs/protocol/status-enum.md).

Success ACK admission (2026-10-09, **confirmed static / tested offline**): APK 37.71
wire parser → device conversion → event 19 → handler proves that a nonnull
converted participant suppresses success acceptance. Rust now guards supported
PN/LID participants, including primary forms, while preserving error ACKs and
original uncertain reservations across restart. The twenty-topic snapshot also
retains native accepted-message SQL, status branches, failure helpers and scoped
poll-path negatives. Native extra user subtypes, complete status/user-row parity
and a qualifying live ACK remain unverified. No sends or account resets ran.
[Evidence and limits](docs/protocol/message-ack.md).

Device send preparation (2026-10-07): selected recipient queries now
cover all local 64-user batches once, without draining unrelated pending users.
Refused, partial and omitted-user replies block before message reservation even
with cached lists. Saved omitted counts remain null for historical/unknown
records; device-query inspects them offline without identifiers or resets.
130-target and cached-refusal/duplicate-omission regressions pass. A 20-second
receive-only live check saved one acknowledged device query with one ingested
user and zero errors/omissions; no chat sends or queued work remained. The list
is primary-only/no signed index list. Large-group batches and own device sync
remain unverified.
[Evidence and limits](docs/protocol/messaging-transport.md#scoped-device-batches-and-completeness-2026-10-07).

Group hash inventory (2026-10-07, offline): saved recipient companions now
contribute to the candidate V2 hash, alongside the known own primary. Previously
they could be addressed but omitted from phash. New group fanout-inspect shows
redacted input counts and own inventory unknown; explicit --hash reveals the
candidate. Complete own-account device sync, active feature selection and live
companion-group sending remain unverified. The 34-method group snapshot compares
in 0.045388 seconds. [Evidence and limits](docs/protocol/groups.md#saved-device-hash-planning-2026-10-07).

Participant hashing/cache trace (2026-10-07, offline): APK 37.71 confirms
UTF-16 ordering, V2 literal agent zero and feature-gated V3 no-agent hashing.
Pure Rust hashing and offline inspection match 684 executed host-Java reference
modes; default xtask feature-gating is fixed. Existing member updates preserve
devices; traced cache/alias writes do not prove a database transaction.
Automatic sends remain V2; the subsequent correction includes saved recipient
devices. Own inventory and active feature selection remain unfinished.
The expanded 86-method snapshot compares in 0.052231 seconds. No account state
or live operation changed. [Evidence and limits](docs/protocol/groups.md#participant-cache-and-versioned-hashes-2026-10-07).

What is true now. The dated experiment log that led here, including every
superseded conclusion and dead end, is in
[`docs/status-history.md`](docs/status-history.md); per-area evidence is in
`docs/protocol/`. Labels: **live** = seen working against the real server,
**acked** = the server accepted it but a handset result is not confirmed,
**offline** = tested locally only.

Receive recovery (2026-10-07): the saved account recovered its blocked direct
and group texts, then delivery-confirmed both on live redelivery. The two
recovered-message read receipts received matching server ACKs. Current receipt
state: eight acknowledged reads and one acknowledged retry. Handset blue ticks
and fresh-message automatic-read validation remain unconfirmed.

The causes in our implementation were blanket refusal of changed peer pins
and group pairwise selection of a PN ratchet despite a held LID session. An
explicit APK-compatible incoming policy now commits an authenticated peer
replacement atomically with encrypted prior-key/session history and an
`IdentityChanged` event. The library default remains `pinned`; profile `l`
selects `apk_authenticated` in its preserved private corrected configuration.
No local account identity, send ID or reservation was reset. Automatic reads
also cover eligible authenticated duplicates, with once-only write-cut coverage.
Human safety-number verification remains incomplete. Native V4 explicit-input
display, binary export/comparison and guarded local manual marks are implemented
and tested offline. Current account/peer scheme and complete inventory selection,
protobuf-C edge equivalence and companion verification synchronization remain
unverified.
[Safety-number evidence](docs/protocol/safety-number.md). Outgoing key-bundle
mismatches now suspend the original queued intent before ratchet use;
`outbox-suspended` inspects them offline. Independent
`transport.outgoing_identity_changes=apk_authenticated` now stages a signed
directory-bundle replacement and commits prior-state history, the identity
event and original send reservation atomically. Explicitly selected suspended
intents can use their saved reply without another key query; invalid signatures,
own-account changes, stale-pin rollback and uncertain sends remain blocked.
The canonical outgoing policy remains pinned; live replacement is unverified. `peer-identity` now inspects both namespaces offline;
the recovered direct peer has different PN/LID pins, each matching its own
held session. Authenticated history connects the PN pin to the updated LID pin.
A 25-second directory-only live check obtained a valid signed current bundle
matching the recovered LID pin and differing from the PN pin. Peer pin/session
summaries stayed unchanged. One private test draft awaits authorization; no
new chat was sent.

Private state was recovered from the former checkout in the desktop Trash,
with originals preserved. The selected profile has no eligible chat sends or
queued actions. The successful 45-second receive window and 20-second read
window sent no queued chats. Evidence: [peer identity](docs/protocol/peer-identity.md),
[group session selection](docs/protocol/groups.md#group-pairwise-pnlid-session-correction-2026-10-07),
[read recovery](docs/protocol/messaging-transport.md#authenticated-duplicate-auto-read-correction-2026-10-07).
The earlier interval with no incoming messages is still unexplained.

## Accounts

Registration uses the recipe in
[registration-lab](docs/protocol/registration-lab.md#working-recipe)
(`enc_empty_h`, 2.26.37.70 User-Agent and token material).
`lab-signup resend-code` asks for a new code on an existing pending attempt.

## Messaging

| Capability | State | Where |
| --- | --- | --- |
| Direct text, PN or LID addressed | live | [messaging-transport](docs/protocol/messaging-transport.md) |
| Direct image, voice note, view-once image | live (receipt confirmed; playback not) | same |
| Replies, reactions, polls (direct) | live | same, [chat-actions](docs/protocol/chat-actions.md) |
| Receive + decrypt direct and group messages | live | [groups](docs/protocol/groups.md) |
| Read receipts, group (`mark-read --chat …@g.us`) | acked 2026-09-25; handset "Read by" unconfirmed | [messaging-transport](docs/protocol/messaging-transport.md) |
| Automatic read receipts (`transport.mark_read`, `listen`) | offline | same |
| Group send (sender-key fan-out) | live — operator read it | [groups](docs/protocol/groups.md) |
| Group replies | live — shown on the handset (the operator reacted to one) | same |
| Group reactions | **not rendering**: payload, stanza and target match the official client's, but the handset shows nothing and the server's ack carries no `count`; cause unknown | same |
| Retry receipts: sending ours when we cannot decrypt | runtime retry server-acknowledged 2026-10-06; redelivery still undecryptable | [messaging-transport](docs/protocol/messaging-transport.md) |
| Retry receipts: answering theirs after a fan-out | offline; one live retry was received before the answer existed | [groups](docs/protocol/groups.md) |
| Status: receive | live | [status](docs/protocol/status.md) |
| Status: post styled text | live — operator saw it | same |
| Status: post image | acked as `type=text`; now `type=media`, **not yet sent live** | same |
| Status: viewers (delivered/read/played) and likes received | live (`status-viewers`) | same |
| Status: sending a like, HD dual upload, audio | not implemented | same |
| App-state writes (contact action) incl. 409 catch-up | live | [chat-actions](docs/protocol/chat-actions.md) |
| Block/unblock, contact check | live | same |
| Per-chat disappearing timer, keep-in-chat, limit sharing | acked 2026-09-20 (delivered; handset unconfirmed); default timer never sent | same |
| Two-step verification set/change/disable | live (set); the first set attempt stays Sent/unknown | [registration-lab](docs/protocol/registration-lab.md#two-step-verification-and-resolving-an-unknown-outcome-2026-09-24) |
| Display name (`presence name=…`) | written live, handset unconfirmed | [presence](docs/protocol/presence.md) |
| Display name app-state record (`pushNameSetting`, `critical_block`) | live — accepted at version 1 | same |
| About text (`iq xmlns=status`, `whatsapp-cli about`) | server-accepted 2026-09-28, as reported in the session export; handset unconfirmed | [presence](docs/protocol/presence.md) |
| Typing / recording indicators (`chatstate`) | typing handset-confirmed in the session export; recording and incoming indicators unconfirmed | same |
| Presence subscribe, incoming availability / last seen | live — server answered with the peer's presence | same |
| Prekey upload and replenishment | live | [messaging-completion](docs/messaging-completion.md) |
| ML-KEM-1024 hybrid (0x44) sessions: verified PQ bundles in direct, fan-out, refresh and inbox | offline; no KEM prekey upload, no live hybrid message | [hybrid-fanout](docs/protocol/hybrid-fanout.md), [kem-prekey-inventory](docs/protocol/kem-prekey-inventory.md) |

A single undecryptable incoming message no longer ends a session; it degrades
to `Undecryptable` and we send a retry receipt. Direct messages from a LID sender now decrypt with the LID
session our group sends established; that was why one of the operator's
messages stayed unreadable for four days (fixed 2026-09-28, verified by offline
replay). A group payload addressed only to this device (such as a
`messageHistoryBundle`) is now recorded and acknowledged instead of being
redelivered at every login.

## Not done

The per-area matrix with exposure and blockers is [feature-parity](docs/feature-parity.md).


- **Stickers and video notes.** Field numbers are decoded from the APK
  (`stickerMessage` 26, `ptvMessage` 66, `VideoMessage` inner fields);
  StickerMessage's 22-field schema and image-key domain are now traced in
  2.26.37.71. Ordinary URL-bearing WebP sticker listing, sealed references,
  authenticated retrieval and local visual playback are implemented offline.
  Lottie, directPath-only retrieval and sending remain unsupported; no incoming
  sticker/PTV stanza or sticker display has been captured. Ordinary URL-bearing
  MP4 video/video-note listing, caption/reference events and video-domain
  authenticated retrieval are now implemented offline; live playback, circular
  rendering, HD variants and sending remain unverified or unsupported. One rebuilt
  60-second receive window authenticated/activated and stopped cleanly, with the
  20-event journal and nine acknowledged receipts unchanged; no fresh sample.
  [Sticker evidence](docs/protocol/stickers.md), [video evidence](docs/protocol/video-media.md),
  [apk-analysis](docs/protocol/apk-analysis.md).
- Calls; profile picture.
- Privacy settings mutation. The SMAX `privacy` IQ shape is confirmed for the
  `readreceipts` category, but the MEX/SMAX selector (property 23639) and other
  category names are unknown ([read-setting-sync](docs/protocol/read-setting-sync.md)).
- Channels and QR/companion pairing — parked by the owner.
- Companion-device fan-out and own-account device sync.
- History-notice handling. APK 2.26.37.71 identifies the formerly unexplained
  49-byte `[102]` envelope as `messageHistoryNotice`; offline inspection validates
  supported metadata and reports redacted counts without applying history. Saved-account offline inspection
  confirmed the 49-byte notice metadata: one receiver, no non-receivers, context
  present, no optional bundle timestamp or bot metadata. The APK group-notice handler
  and metadata requirements are traced. Supported normal group/meta context now
  projects a distinct control event; historical group/broadcast backfill is fixed.
  Canonical offline backfill upgraded this one notice; repetition upgraded zero.
  APK feature selection, rendering, timestamp units and history processing remain
  unverified. [Cross-device evidence](docs/protocol/cross-device-sync.md).

## Offline verification

Native research now locates separate Superpack decoders and V3/V4 fingerprint
symbols in one decoded stream. The ARM64 decoder loads and initializes all
19 JNI methods locally, exiting zero. A bounded first-entry reconstruction
returns native `9/2` / `could not extract file from archive`, with no output;
real host memory metadata did not fix it. Worker tracing now identifies an
oversized-file comparison against zero requested bytes, despite the matching
125448 declared/produced sizes. This partial-range result does not establish
ordinary full extraction failure; bounded full-entry checks have reached wall
limits without output or native refusal. Hash-pinned branch/futex tracing is
available. The unmodified ordinary full decoder now succeeds under QEMU,
returning 80 entries (one duplicate) and 79 valid ARM64 ELFs; a second run
matches all 79 hashes. V3/V4 fingerprint code anchors and containing unwind
ranges are located in reconstructed `libwhatsapp.so`; the algorithms and active
scheme remain unfinished. The helper's previously ineffective threaded-runner
limits are corrected, with eight regression checks including actual guest
execution, resource guards and preflight/output preservation. These tooling
checks make no live account request.
[Evidence, setup failures and remaining warnings](docs/protocol/native-unpacking.md).

529 all-feature tests (`cargo test --locked --workspace --all-features`) and
223 default-feature tests (`cargo test --locked --workspace`) pass on
2026-10-07; Clippy with `-D warnings` is clean. Rust 1.98.1 locally and in CI. Store, codec, crash
matrix, registration state machine and transport coverage are described in
[docs/design](docs/design/README.md) and the per-area protocol docs.

Before any live check, rebuild the release binary: `cargo test` does not
refresh `target/release/whatsapp-cli`, and a stale one once produced a wrong
"cause eliminated" conclusion.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo build --release -p whatsapp-cli --features experimental-transport
cargo run --locked -p xtask -- proto-check
cargo run --locked -p xtask -- dependency-audit
```


Native safety-number follow-up traces the shared V3/V4 5200-round SHA-512
helper, sorted key concatenation and separate identifier-bound display seed.
A pinned ARM64 slice checker validates iteration/input ordering on synthetic
data with explicitly modeled SHA/MCF operations. Full input producers,
JNI normalization, QR semantics and current scheme/human verification remain
unfinished; no account verification state or live connection changed.
[Evidence and reproducible check](docs/protocol/safety-number.md).


The native identifier callback is now traced back into Java: PN shortening
uses HNy, LID output uses the parsed JID user, and the return conversion
requires ASCII. Native key constructors establish 32-byte sorting versus
33-byte serialized hashing inputs. Java V4 creation and result dispatch now
connect to the recovered routines; the traced result path emits QR version 0.
Current scheme selection, complete device inventory, individual QR fields,
comparison semantics and human verification remain unfinished.
[Callback and dispatch evidence](docs/protocol/safety-number.md#native-to-java-identifier-callback-and-dispatch-mapping).


Native QR descriptors and serialization are now recorded with a reproducible
metadata extractor. Version zero carries serialized keys; version one their
identifier-free digest. V4 comparison resolves peer orientation and checks
keys, LID and additional PN/username/hosted-state conditions; LID/key equality
alone cannot establish verification. Fourteen offline tooling checks pass.
Full comparator primitive/error semantics, UI selection/inventory and the
verified-state action remain unfinished; no account verification was performed.
[QR evidence and schema](docs/protocol/safety-number.md#native-qr-schema-serialization-and-v4-comparison).


Fingerprint comparison primitives now establish NUL/non-ASCII rejection, PN
byte equality, username strncasecmp and orientation-dependent native status
remapping in APK 37.71. The actual native byte leaf passed 862 synthetic
comparisons without modeled callees; all 16 offline tooling tests passed.
V4 comparison is mapped to jvidispatchIOO selector 4. Its Java caller, active
scheme/device inventory and human verification-state commit remain untraced.
No account verification or live request occurred.
[Evidence and reproducible check](docs/protocol/safety-number.md#comparison-primitives-and-orientation-dependent-statuses).


APK 37.71 Java comparison selection now connects KXj.A01 to native V4
selector 4 behind holder/feature gates and an exact zero-result success test.
All three comparison callers are located. Camera/NFC success reaches result
presentation; the traced path does not establish a persisted human-verification
bit. Delayed callbacks, the verification writer and complete scheme/device
inventory remain unfinished. No account state or live request changed.
[Java caller evidence](docs/protocol/safety-number.md#java-comparison-selection-and-scan-result-action).


APK 37.71 verification metadata writers are now traced: the local sender
rejects stale rendered identities, rechecks the stored key under lock, advances
the action sequence and writes existing metadata before companion fan-out.
Inbound peer actions are companion-only with target/key/sequence guards;
history verification has separate presence/key requirements. BWE/BT5 schemas
and ProtocolMessage field 32 are confirmed. Delayed scan callbacks only present
results. Full human-trigger wiring, active fingerprint scheme/device inventory
and client implementation remain unfinished; no account state changed.
[Writer and schema evidence](docs/protocol/peer-identity.md#verification-metadata-writers-and-rotation-guards-3771).


The APK local verification control now traces from saved metadata/rendered
identity to explicit mark confirmation and separate unmark execution. The
writer receives that captured key, allowing its rotation guards to refuse stale
confirmation. Feature/predicate gates and complete fingerprint inventory remain
unverified; client verification metadata remains unimplemented. Two missing
DEX field-write mnemonics are fixed with literal APK instruction regression
coverage. No live account operation occurred.
[Local control evidence](docs/protocol/peer-identity.md#local-verification-control-and-captured-identity-3771).

APK 37.71 manual-verification contact exclusions now resolve to a hosted-set
predicate and a saved contact-state value ==2 (semantic name still unknown).
Hosted device-account type 1 and identity attestation HYBRID_E2EE type 2 come
from separate SQL sources; neither is the native QR HostedState enum. Native
display getter tracing also confirms that QR retains the full LID JID while
its decimal seed uses the parsed LID user. Current account inventory, active
scheme and human-verification implementation remain incomplete. No account
state or live request changed. See the peer-identity contact-exclusion and
safety-number display follow-ups.

Native fingerprint display is now implemented as an offline library primitive
and `fingerprint-display --config` CLI command using explicit private normalized
identifiers and raw public key inventories. Three independent synthetic ARM64
hash-flow fixtures match, including duplicate-key preservation; result labels
state that scheme/input selection is unverified. The calculation does not open
account storage or mark a peer verified. Active scheme, complete device-key
inventory, QR comparison and human verification remain unfinished.

Fingerprint calculation validation passed 535 workspace/all-feature tests,
229 default-feature tests, strict Clippy, warning-free rustdoc, CLI reference,
formatting and repository hygiene. A command-level synthetic calculation
matched the fixture pair without creating the supplied unused store path.
No network request or current-account verification was performed.

APK peer fingerprint inventory now traces to a selected device set and a
nonempty identity map with equal cardinality. Feature 0x1da3 excludes hosted
99-device classes; the local key helper has separate primary/companion controls.
`peer-identity` now reports retained E2EE device-key coverage without claiming
complete APK inventory or fingerprint readiness. Current scheme/contact-state
values, source inventory freshness and full own-device synchronization remain
unknown. No account keys or verification metadata were changed.

The rebuilt peer inspector passed on an isolated saved-account copy: PN has
no saved device list; its mapped LID has one retained device/one pin and no
missing pins, but companions_verified=false (primary-only fallback).
Original encrypted database bytes were unchanged. These are saved-state
observations, not a live directory refresh or proof of complete inventory.
Validation passed 536 workspace/all-feature tests and strict Clippy.

Native V4 combined fingerprint protobuf export now supports QR versions 0/1
through the library and paired fingerprint-display --qr-version/--qr-output
flags. Independent synthetic wire fixtures preserve full LID, hosted-state
presence and local PN omission; version 1 uses identifier-free H(K,K).
Exports are private/create-only and do not open account storage. This is binary
comparison data, not a QR image or a verified peer comparison. Active scheme,
complete inventory, scanning/comparison and human-verification commits remain
unfinished; no current-account QR or live operation was performed.

Native QR export validation passed 540 workspace/all-feature and 233 default
feature tests, strict Clippy, warning-free rustdoc and CLI reference. The rebuilt
CLI matched independent 200/229-byte synthetic payloads for versions 0/1,
created mode-600 files and refused repeats without changing them or creating
an account store. A zero-qualified JID test exposed and fixed the new export's
canonical-input guard; the generic JID parser and account state were unchanged.

Local manual verification metadata (offline, 2026-10-07): `peer-verification`
records an explicit human mark/unmark bound to an inspected pin and sequence.
`peer-identity` reports the saved bit/sequence/pin agreement. Authenticated
incoming and signed-directory replacement clear the bit atomically, preserving
sequence; rotation back does not restore it. Tests cover restart, stale/own
refusals, unchanged session/pin and archive rollback. No actual account was
marked. Automated QR comparison and companion synchronization remain unfinished.
[Evidence and operator semantics](docs/protocol/peer-identity.md#local-manual-verification-metadata-2026-10-07).
Validation for this change: 545 workspace/all-feature tests passed; strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, formatting,
regenerated CLI reference and repository hygiene passed (304 local links).
The first suite run caught the new OpenAPI `x-cli` including flags instead of
the command name; corrected to the repository's command-name contract before
rerunning successfully.

Native V4 comparison follow-up (offline, 2026-10-07): 115 synthetic cases execute
actual APK 37.71 comparison/helper/remapper branches with explicit host-modeled
getters and string/key/LID primitives. All 81 hosted-state combinations confirm
that both scanned states absent takes the legacy success branch regardless of
known states. Missing-versus-unequal username results and version-selected key
checks are reproduced. Converter refusal codes are statically attributed.
Whole parsing/orientation/Android string behavior and production scanning remain
unimplemented; no account was marked or changed.
[Executed branches and boundaries](docs/protocol/safety-number.md#executed-native-v4-comparison-branches-2026-10-07).
Validation: all 19 pinned native tooling tests and 36 CLI/API tests passed;
CLI reference regeneration and repository hygiene passed (310 local links).
The final attributed observation artifact also regenerated identically and
passed its three focused preflight/execution checks. Rust runtime code did not
change in this research increment.

Native username primitive (offline, 2026-10-07): the V4 checker now optionally
executes actual APK 37.71 merged MCFStringCompare and the pinned public Bionic
strncasecmp fixture. All 115 core statuses agree; 268 additional accepted-ASCII,
every-position mismatch, prefix, empty and 256-byte cases confirm ASCII
case-insensitive equality plus exact length for these fixtures. Public Bionic
is not APK-bundled or proof of every Android build; object construction and
non-string dependencies remain modeled. QR parsing/orientation/current inventory
and production scanning remain unfinished. No account state changed.
[Native string evidence](docs/protocol/safety-number.md#actual-native-ascii-username-comparison-2026-10-07).
Validation for the native-string increment: all 22 native tooling tests passed
with all four fixtures configured, alongside 36 CLI/API tests, regenerated CLI
reference, formatting and repository hygiene (315 local links). Rust runtime
code did not change; the account database was not opened.

Explicit binary comparison (offline, 2026-10-07): `fingerprint-compare` now
compares private native V4 protobuf against explicit known JSON, with traced
conversion order, both scan orientations and native status remapping. Tests
match all 113 executed non-abort core cases and 268 native string cases;
independent wire fixtures cover both versions/orientations. Result reports
caller-input and bounded native-parser scope; complete protobuf-C edge equivalence,
current inventory and live scanning remain unverified. No account or human
verification metadata is changed by comparison.
[Comparison scope](docs/protocol/safety-number.md#explicit-input-native-v4-binary-comparison-2026-10-07).
Validation: 551 workspace/all-feature tests and 239 default-feature tests passed,
with strict all-target/all-feature Clippy, warning-free Rustdoc, formatting,
CLI reference and repository hygiene (317 local links). Command-level synthetic
version-0/1 fixtures matched without creating an unused account-store path;
malformed payload/private-permission checks passed. Both-scanned-LIDs-matching
priority also passed its focused regression. The initial documentation check
compiled before the new OpenAPI operation was added; completing the contract
and regenerating/rerunning resolved the coverage refusal.

Native parser correction (offline, 2026-10-07): actual APK 37.71 protobuf-C
unpack/init/merge/free now reproduces a 50-case synthetic corpus. Eight prost
acceptance differences were observed; a valid fingerprint plus an unknown group
could previously produce a false match. `fingerprint-compare` now uses a bounded
schema decoder matching native tag/wire/length/scalar/merge rules, validated for
acceptance and all recognized field values. Its parser-authority output reports
corpus validation with other edges still unverified. Unknown opaque fields are
skipped, not round-tripped; current inventory/live scanning remain unverified.
No account store, pin/session or verification metadata was changed.
[Native parser evidence](docs/protocol/safety-number.md#actual-protobuf-c-parsing-and-bounded-decoder-correction-2026-10-07).

Validation (offline): 552 workspace/all-feature tests, 240 default-feature tests
and all 25 fully configured native tooling checks passed, alongside strict
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference. A rebuilt CLI matched both independent version-0/1 fixtures and
returned -8 for the otherwise-valid-plus-group and malformed fixtures; the
unused account-store path remained absent. The corpus test also checks explicit
nested-holder presence, with its final focused rerun passing.
Repository hygiene passed with 323 local links; staged diff whitespace checks passed.

Own fingerprint inventory follow-up (static, 2026-10-07): APK 37.71 own-device
selection is now traced to the cached `devices` directory, strict positive
logout-time exclusion, self LID-to-PN projection with existing-PN precedence,
and later fingerprint-facing address remapping. BNm checks loaded Me presence,
not signed-list freshness. Add/remove paths invalidate both directory caches.
Current membership, authentication and active fingerprint inputs remain unknown;
no account state was opened or changed.
[Own-directory evidence](docs/protocol/safety-number.md#own-directory-source-logout-filtering-and-alias-precedence-2026-10-07).

Validation: the APK hash was rechecked, and an independent exact A08 method
extraction matched the scoped snapshot byte-for-byte. CLI reference regeneration
and its test passed; repository hygiene passed with 329 local links. The initial
snapshot's trailing blank line was removed; final staged whitespace checks passed.
This change adds static research evidence only; runtime code and API/CLI
capabilities are unchanged, and no live account validation is claimed.

Fingerprint contact-state follow-up (static, 2026-10-07): the reusable
`xtask apk-field-refs` command found 17 direct writes to APK 37.71 0Da.A04.
Read/write tracing identifies `external_user_state`, with guest/Cameo,
pending-invitation and normal-user transitions, rather than Signal attestation
or QR HostedState. Complete producer authority and active scheme remain open.
[Contact input evidence](docs/protocol/safety-number.md#fingerprint-contact-predicate-is-external-user-state-2026-10-07).

Validation: 555 workspace/all-feature tests and 243 default-feature tests
passed, including three new APK/DEX regression checks. Strict workspace
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference passed. Final tool-only tests/Clippy also cover filename sorting.
Command-level no-DEX and malformed-DEX archives refused with empty stdout;
wrong field type produced zero sites. The shared caller search reproduced all
134 previous BNm call-site records; the first byte-for-byte check differed
because ZIP filename iteration order varies, so record sets were compared.
The new field-search command sorts DEX names to make its output reproducible.
No live account or identity-change test was performed in this change.
Two sorted field-search runs matched byte-for-byte. Repository hygiene passed
with 334 local links; final staged whitespace checks passed.

Fingerprint selection follow-up (static, 2026-10-07): APK 37.71's captured/contact
gate selects native V3/V4; the later fresh read selects PN inclusion. Own-key
companion inclusion, empty-to-null strings, stored own username, remote `@`
prefixing and nullable `is_pn_shared` storage are now traced. The earlier
username-availability hypothesis for that Boolean was refuted. Smali branch
annotations now show signed jumps, with regression coverage. Current feature
values, authenticated input producers and native V3 comparison remain open;
no current account fingerprint or verification action occurred.
[Selection evidence](docs/protocol/safety-number.md#scheme-username-and-pn-sharing-input-branches-2026-10-07).

Validation (offline): 556 workspace/all-feature tests and 244 default-feature
tests passed. Strict workspace/all-target/all-feature Clippy, warning-free
Rustdoc, formatting and CLI reference regeneration/test passed. Fresh APK
method output confirms both backward jumps and matches the scoped selection
excerpt. No account state was opened or changed and no live request was sent.
Repository hygiene passed with 339 local links; staged whitespace checks passed.

Native V3 comparison follow-up (static, 2026-10-07): selector 19 now resolves
through the relocated JNI table to 0x71373c, with Java options 1/3 and distinct
PN-or-LID orientation, conversion errors, PN/username presence checks, hosted
omission and result remapping. Generation and scan selection use separate
feature flags. Native execution, MCFEqual/JID semantics and binary comparison
implementation remain open; no account state or live request changed.
[V3 evidence](docs/protocol/safety-number.md#native-v3-scan-dispatch-and-comparison-branches-2026-10-07).

Validation (static/offline): APK and reconstructed-library hashes matched the
pinned values; the option extraction returned one method. Independent fresh
native extraction matched all 424 converter/helper/core instructions in the
scoped snapshot. CLI reference regeneration/test passed; repository hygiene
passed with 344 local links and staged whitespace checks passed. This changes
research/docs only; native execution and live account validation are not claimed.

V3 native execution follow-up (offline, 2026-10-07): 264 synthetic cases now
execute the actual V3 core/optional-value/PN-or-LID helpers and remapper,
validating the traced presence asymmetry, option bits, hosted omission and
error ordering. Ten V3/shared-V4 tests passed, preserving existing V4
observations. Generic object equality and constructors remain modeled;
JID/equality semantics, decoder/orientation integration and runtime V3 binary
comparison remain open. No account or network state changed.
[V3 corpus](docs/protocol/safety-number.md#executed-v3-core-and-remapper-corpus-2026-10-07).

Final validation: CLI reference regeneration/test and OpenAPI library-symbol
check passed. Repository hygiene checked 349 local links; staged whitespace
checks passed. All ten V3/shared-V4 tests passed again after restricting hooks
to the selected scheme. Rust runtime code was unchanged; no live validation
is claimed for this offline research-tool change.

V3 equality/LID follow-up (static, 2026-10-07): registered ASCII equality is
length-sensitive byte equality, including case-sensitive usernames. JID
equality uses the constructed full string. The scanned LID path parses and
flattens device-zero objects before requiring primary category and LID kind;
nonzero companions fail that guard. Exact validator bounds are preserved.
Actual parser/class-runtime execution and full binary V3 comparison remain
open; no account or live action changed.
[Equality/LID evidence](docs/protocol/safety-number.md#v3-equality-callbacks-and-primary-lid-conversion-2026-10-07).

Validation (static/offline): APK, main and merged hashes matched the pinned
build. Fresh extraction matched all 18 ASCII comparator instructions. CLI
reference regeneration/test passed, repository hygiene checked 354 local
links and staged whitespace checks passed. This change adds attributed research
only; it does not execute the native parser or change runtime/account behavior.

V3 primary-LID execution follow-up (offline, 2026-10-07): 38 synthetic inputs
now execute native parser routing, validators, constructors, device-zero
flattening and kind selection. They confirm explicit zero suffix normalization,
nonzero companion rejection and exact base-number bounds within modeled MCF
primitive boundaries. Four utility tests passed. Native MCF/class/ownership
behavior and full binary V3 comparison remain unfinished; accounts unchanged.
[Executed LID decisions](docs/protocol/safety-number.md#executed-v3-primary-lid-parser-decisions-2026-10-07).

Final validation: CLI reference regeneration/test and OpenAPI library-symbol
check passed. Repository hygiene checked 359 local links and staged whitespace
checks passed. This adds an offline research utility and observations; Rust
runtime behavior and saved accounts are unchanged.

V3 client comparison follow-up (2026-10-07): explicit `compare_v3` and
`fingerprint-compare --scheme native-v3` now compare bounded binary payloads
using the APK's V3 converter/core/orientation/remapping. Optional `--v3-hosted`
selects APK options 3; otherwise 1. V3 permits PN-only known identities and
uses case-sensitive usernames. Native MCF splitting is statically confirmed.
The port matches 263 non-abort core and 38 LID branch observations; current
input selection, native MCF/JNI/ownership edges and real peer scan remain open.
No account or live action changed; V3 QR export remains unimplemented.
[Runtime comparison](docs/protocol/safety-number.md#explicit-native-v3-binary-comparison-in-the-client-2026-10-07).

Validation: workspace all-feature tests (561), default workspace tests (249),
strict all-target/all-feature Clippy, warning-free Rustdoc, native V3 utility
tests (4), CLI reference regeneration and formatting passed. Synthetic CLI
comparison confirmed options 1/3 and V3 case-mismatch -25 versus default V4 0,
with redacted metadata and no account-store creation.

Repository hygiene passed: 364 local links, tracked-file budgets, package
licenses and research checksums; staged whitespace checks passed.

V3 export follow-up (2026-10-07): explicit `encode_v3` and
`fingerprint-display --qr-scheme native-v3 --qr-version 0|1 --qr-output` now
produce V3 comparison bytes, preserving both PN fields with usernames. Static
combined/individual serializer instructions confirm field/order differences.
Independent Python wire/hash fixtures cover both versions, optional addresses,
zero suffix preservation and both comparison orientations. CLI V3 display/export
requires stable PN display input; library export also accepts LID-only sides.
Current feature/inventory selection and real-peer interoperability remain open.
No account or network action changed.
[V3 export evidence](docs/protocol/safety-number.md#explicit-native-v3-binary-export-2026-10-07).

Validation (offline): 564 workspace/all-feature tests and 252 default-feature
workspace tests passed. Strict workspace/all-target/all-feature Clippy,
warning-free Rustdoc, generated CLI reference, formatting and repository hygiene
(367 local links) passed. The initial workspace run preceded doc regeneration
and failed only its stale-reference test; after regeneration the full rerun
passed. A built-CLI synthetic PN-only export in versions 0/1 matched the exact
independent fixture, created mode-0600 files, compared with V3 status zero and
refused overwrite without changing bytes. No account store was created.

Ordinary document receive extension (2026-10-07): APK 37.71 BZB schema and
distinct document-key domain now support URL-bearing documents, sealed media
references and durable receive events. New media-save explicitly exports to a
create-only private operator path; filenames remain metadata. Document playback
refuses and names that command. Independent crypto/wire/store fixtures cover
authentication, restart, deduplication, labels and overwrite/symlink refusal.
Live incoming documents, directPath-only retrieval, large files and sending
remain unfinished. [Evidence and limits](docs/protocol/document-media.md).

Validation: 567 workspace/all-feature tests and 252 default-feature workspace
tests passed, plus strict all-target/all-feature Clippy, warning-free Rustdoc,
generated CLI reference, formatting and repository hygiene (370 local links).
The first Rustdoc run rejected an unescaped <clinit> in the new API comment;
after formatting that name as code, its rerun passed. No runtime change was
needed. Focused document/schema/crypto/restart/export tests passed.

A bounded 45-second receive-only check authenticated/activated and stopped
cleanly. Events/receipts/actions/outbox summaries were unchanged (20/9/12/0),
with the prior uncertain action preserved. No fresh encrypted message/document
arrived; live document retrieval is still unverified. Private backup/log paths
and notification observations are recorded in the document protocol page.

Group-notification follow-up (2026-10-07): capture inspection now preserves the
APK's exact w:gp2 label instead of hiding it because of its digit. Saved
create/groups_dirty shapes are visible with values redacted. Static 0yv.A09
shows groups_dirty queues FetchTruncatedGroupsJob; request/application and ACK
policy remain unfinished. No notification or membership was applied and no
network request was made. [Evidence and next boundary](docs/protocol/groups.md#saved-group-notifications-and-inspection-correction-2026-10-07).

Validation: 568 workspace/all-feature tests, 24 default-feature CLI tests,
42 final all-feature CLI tests, strict CLI all-target/all-feature Clippy,
CLI reference regeneration/test, formatting and repository hygiene (374 local
links) passed. Rebuilt-CLI saved-account inspection exposed the previously
hidden group type without network I/O, replay or notification application.


Group refresh serializer follow-up (2026-10-07, **offline/static**): APK
37.71 context A05 is truncatable dirty recovery. The concrete `AaH` →
`Aib` case 25 → `9QE`/`EJy` IQ constructor uses `w:g2`, `type=get`,
`query[context]` and per-group JID children; the feature-gated alternative
uses MEX `QueryBatchGetGroups`, `groups` and `request_context`.
Inherited `0yO.A01` dispatch explains the zero concrete-handler caller result
and reaches a queued/direct ACK boundary. Complete ACK serialization,
response/application guards and current feature values remain untraced;
notification implementation remains unchecked. The disassembler now names the
literal double multiply/divide used in the memory budget. See
[request/dispatch evidence](docs/protocol/groups.md#dirty-refresh-request-and-inherited-dispatch-trace-2026-10-07).
32 xtask tests, strict all-target xtask Clippy and an actual APK re-dump passed.
No account state was opened or live request sent for this research.

Final checks for this change: generated CLI reference test, all five OpenAPI
coverage/HTTP/library-symbol checks, formatting, staged whitespace and repository
hygiene (379 local links) passed. These are local checks, not live group refresh
validation.


Batch response/ACK follow-up (2026-10-07, **offline/static**): the 37.71
writer is Cp2.A05 reached from selector 76 through 0y2.A02. The response
parser distinguishes full/truncated/error groups and checks request-correlated
attributes. ID guards check string length; request child-count diagnostics log
without throwing. A truncated result is not a complete group snapshot. Full
membership commit, dirty-state clearing and inbound ACK-route production remain
untraced; notification handling remains incomplete. Empty apk-smali results now
name the active class/method filters without falsely claiming class absence.
32 xtask tests, strict Clippy and the actual APK diagnostic smoke passed.
[Evidence and remaining boundary](docs/protocol/groups.md#batch-response-and-current-build-ack-writer-2026-10-07).
No account state was opened or live request sent for this trace.

Final local checks for this trace: generated CLI reference, five OpenAPI
coverage/HTTP/library-symbol tests, formatting, staged whitespace and repository
hygiene (383 local links) passed. No live group notification application was
validated.


Truncated group snapshot correction (2026-10-07, **offline**): direct and
queued info reply ingestion now refuse truncated group data before replacing
members or emitting removal events. The original snapshot/events and saved
reply survive restart; no ingestion marker or automatic retry is created.
Invalid non-boolean truncation flags also refuse as local policy. APK A9s/8lQ
separate full and truncated processing; the underlying full membership commit
and dirty-state database semantics remain untraced. Notification handling is
still incomplete. [Correction and evidence](docs/protocol/groups.md#truncated-snapshot-protection-and-completion-boundary-2026-10-07).
571 workspace/all-feature and 24 default CLI tests, strict Clippy and Rustdoc
passed. No account was opened or live request sent.

Final checks: generated CLI reference, all five OpenAPI coverage/HTTP/library
checks, formatting, staged whitespace and repository hygiene (387 local links)
passed. The added non-text-flag regression also passed independently.


Atomic queued group ingestion (2026-10-07, **offline**): the once-only marker
now commits with the snapshot/events, invite update or leave deletion. An abort
and encrypted-store reopen test proves rollback across all three paths, then
one successful application and no repeat after another reopen. Truncated replies
still preserve state without an ingestion marker. Two APK completion helpers
include gated state handling and member-suggestion sync, not proven generic
dirty-state deletion. Full group notification parsing/application remains open.
[Atomic ingestion and helper evidence](docs/protocol/groups.md#atomic-queued-group-ingestion-2026-10-07).
No account state was opened or live request sent.

Validation: 572 workspace/all-feature tests, 24 default-feature CLI tests,
strict workspace/all-target/all-feature Clippy, warning-free Rustdoc, generated
CLI reference, five OpenAPI checks, formatting, staged whitespace and repository
hygiene (390 local links) passed. Group notification handling remains incomplete.


Typed group participants (2026-10-07, **offline**): the snapshot parser now
keeps supported user-level JIDs and applies APK linked-map duplicate behavior:
one entry per JID, last role wins, first position retained. Refused participant
entries remain excluded. The saved snapshot and GroupCreated journal regression
covers typed/text duplicates and missing/invalid/non-user/device entries.
APK alias/name/history/country metadata producers are traced but not applied;
role-change events and full group notification handling remain incomplete.
[Participant evidence](docs/protocol/groups.md#typed-participant-map-and-duplicate-roles-2026-10-07).
No account was opened or live request sent.

Validation: 573 workspace/all-feature tests, 24 default-feature CLI tests,
strict workspace/all-target/all-feature Clippy, warning-free Rustdoc, generated
CLI reference, five OpenAPI checks, formatting, staged whitespace and repository
hygiene (393 local links) passed. No live participant-map validation occurred.


Notification input trace (2026-10-07, **offline**): base handler BFC produces
notification ACK metadata through CmG.A02/A00 using incoming from/id/participant/type.
Receipt-only recipient/category/edit extraction is a different wrapper. Normal-edge
analysis confirms dirty refresh round zero and zero per-group integer; its meaning
and exception paths remain unknown. Alternate executor and upstream group dispatch
remain untraced; runtime group notification completion is still open.
[Producer and round evidence](docs/protocol/groups.md#notification-metadata-producer-and-initial-refresh-round-2026-10-07).
No account store was opened or live request sent.

Validation: all five OpenAPI checks, generated CLI reference, whitespace and
repository hygiene passed. No runtime behavior changed or live validation occurred.


Queued group replies now apply during receiving (2026-10-07, **offline tested**),
instead of waiting for `group list`. Settlement stores a durable projection marker;
startup recovers only these newly marked replies once. Truncated/invalid group
responses remain saved and diagnosable without blocking unrelated receiving;
storage failures still stop the session. No request is replayed or historical
account state backfilled. [Session projection](docs/protocol/groups.md#apply-queued-replies-during-receiving-2026-10-07).
One bounded 30-second live group-info check settled ok with a matching saved group
reply and four-member snapshot updated during receiving. Group list afterward
applied zero replies. No chat outbox sends occurred; the previously unresolved
two-factor action remained sent. Existing account identity, keys and config were
preserved. Unsolicited group notifications/batch refresh and live recovery remain
unverified.

Validation: 575 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, CLI reference
regeneration and formatting passed. Bounded live snapshot projection passed;
unsolicited group notification completion remains open.

Final OpenAPI checks (five), staged whitespace and repository hygiene
(399 local links) also passed.


Dirty batch partition/round trace (2026-10-07, **offline**): corrected missing
23x binary operands in xtask and independently checked raw job bytes. The APK
partitions by accumulated participant estimate, carries returned truncated sizes
with fallback to the previous estimate, and computes a bounded round limit when
none is provided. The all-zero dirty input derives bound 2, subject to outcomes.
The alternate notification executor also reaches application-before-ACK.
[Evidence and remaining boundaries](docs/protocol/groups.md#batch-partition-and-bounded-follow-up-rounds-2026-10-07).
Automatic dirty refresh/error/rate-limit application remains unfinished; no
account state was opened or live request sent for this trace.

Validation: 576 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, generated CLI
reference, five OpenAPI checks, formatting and repository hygiene passed.
Client wire behavior is unchanged by this disassembler/research change.


Classic batch refresh (2026-10-07): `group refresh --chat <group-jid>` (repeat
--chat) queues one classic per_group_dirty_recovery IQ. Complete groups/group
responses project only when all requested identities occur once, atomically with
events and the projection marker. Incomplete/truncated/error-bearing batches
preserve all snapshots and saved responses. [Batch queue and limitation](docs/protocol/groups.md#classic-batch-queue-and-complete-response-projection-2026-10-07).
Mixed-union application, follow-up rounds and automatic dirty notification
scheduling/ACK remain unfinished.

One two-group classic batch was live verified in a bounded 30-second receive
session: ok, two matching saved results and both snapshots updated during the
window (one/four members). Group list afterward applied zero actions. No chat
intent was sent; the unresolved two-factor action remained sent. Identity/keys
and existing request IDs were preserved. Mixed/truncated continuation remains
unimplemented and unverified. 577 workspace/all-feature and 24 default CLI tests,
strict Clippy, warning-free Rustdoc, CLI reference and five OpenAPI checks passed.

Final staged whitespace and repository hygiene (407 local links) passed.


Mixed batch projection (2026-10-07, **offline tested**): full snapshots now commit
beside APK-recognized truncated(size 0..19999), forbidden(403) and not-found(404)
results. Partial items preserve existing membership, and a durable outcome commits
with full updates/events and projection markers. `group refresh-status --id`
inspects only counts without queueing. [Union evidence and remaining work](docs/protocol/groups.md#mixed-batch-projection-and-outcome-inspection-2026-10-07).
No live query was repeated or saved account opened. Bounded follow-up and dirty
notification handling remain unfinished; mixed/report behavior is not live verified.

Validation: 579 workspace/all-feature and 24 default CLI tests, strict Clippy,
warning-free Rustdoc, regenerated CLI reference, all five OpenAPI checks,
formatting and repository hygiene passed. Mixed/report live validation remains
unverified.

Opt-in bounded group refresh continuation (2026-10-07, **offline tested**):
`group refresh --follow-up` uses the APK truncatable context and rounds 0..2.
Recognized results atomically project/report and queue one distinct next query;
uncertain, refused and unsupported results never continue. Original input order
and unsent partitions persist; local budget 10000 is conservative, not a claim
about APK runtime memory/features. Reports expose round, next action, pending
count and exhaustion. Default manual refresh remains single-query.
[Evidence and boundaries](docs/protocol/groups.md#opt-in-bounded-refresh-continuation-2026-10-07).
No live query repeated. Live continuation, unsolicited dirty scheduling/ACK and
APK error/wait helper handling remain unverified or unfinished.

Validation: 583 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, regenerated CLI
reference and formatting passed. Transaction/reopen and old-report inspection
checks are offline; live truncated continuation remains unverified.

Final five OpenAPI checks, whitespace and repository hygiene (413 local links)
also passed. Private account state and conversation exports remain untracked.

Automatic supported dirty-group notifications (2026-10-07, **offline tested**):
`listen` now schedules the bounded truncatable refresh atomically with a durable
source/ID deduplication marker before acknowledging w:gp2/groups_dirty. Duplicate
supported delivery repeats only the ACK; uncertain queries retain their IDs and
reservations. Unsupported/empty/conflicting inputs stay captured without ACK and
have redacted diagnostics. `actions` and `group refresh-status` inspect progress.
[Evidence and local restrictions](docs/protocol/groups.md#automatic-supported-dirty-notifications-2026-10-07).
No account opened or live request sent. Live dirty input/ACK, other notification
types and APK error/wait handling remain unverified or unfinished.

Live supported dirty notification projection subsequently passed in one bounded
45-second authenticated receive session: zero queued actions/chat intents at
preflight; one notification-derived refresh settled ok and committed one full
group at round 0, no pending or follow-up work. Session stopped cleanly; no
incoming user message or chat send. The unresolved two-factor action stayed sent.
ACK writing completed without session error, not independent server ACK proof.
Live truncated/deduplication/failure recovery remains unverified.

Validation: 586 workspace/all-feature and 24 default CLI tests, strict Clippy,
warning-free Rustdoc, regenerated CLI reference, five OpenAPI checks, formatting
and whitespace passed. The corrected malformed-JID fixture and full rerun are
recorded in protocol docs.

Final OpenAPI rerun, whitespace and repository hygiene (417 local links) passed.
Private receive evidence and conversation exports remain excluded from Git.

Owner steering (2026-10-07): comprehensive APK-derived Rust behavior and quick
release-to-release file comparison now drive the work queue. Normal onboarding,
messaging, identity/state preservation and honest live boundaries remain required.
[Porting and release workflow](docs/protocol/apk-porting.md).

Version-aware research export now accepts an exact-hash JSON recipe with stable
topic/class/optional method filter and affected Rust paths. The committed 37.71
six-topic group baseline has 11 files/391,666 bytes; initial extraction took
16.768 seconds. `apk-research-diff` checksum-verifies cached topic files and links
structural changes to evidence/Rust paths. Three real cached self-comparisons
returned six unchanged topics in 0.0183..0.0215 seconds. Added/removed/changed,
provenance-only changes and tampering are covered by distinct synthetic fixtures.
This is scoped evidence comparison, not complete APK coverage or automatic
semantic remapping/translation. Only the current APK was found in Downloads/lab;
the committed 37.70 snapshot remains available, but its group mapping is absent.

Subject freshness trace confirms a separate notification-version cache;
ordinary snapshot s_t is not proven equivalent. The parent propagation helper is
not the main subject writer. Application, alias effects and full cache lifecycle
remain unfinished. No saved account opened or live request sent for this change.

Final validation: 588 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, CLI reference,
five OpenAPI checks, formatting, staged whitespace and hygiene (425 local links)
passed. Exact 37.71 snapshot regeneration with --check passed in 17.467s. Final
cached reports (six topics plus four metadata/dictionary artifacts) took
0.0200..0.0213s; all unchanged in the real self-comparison.

Anchor-assisted APK release maintenance (2026-10-07): `apk-research-remap`
finds exact-string co-occurrences while preserving method versus whole-class
scope. Ambiguous, missing, split-scope and unanchored topics cannot produce a
recipe; existing proposals cannot be overwritten. Confirmed static scan of
2.26.37.71 uniquely recovered all six reviewed group targets in 5.225 seconds.
A real split-scope negative probe refused output; adding anchors changed only
metadata, with no code-topic differences. The baseline is now 392,185 bytes.
Broader APK/native coverage and a distinct next-release comparison remain
unfinished. No account state was opened or live request sent for this work.

Validation: 590 workspace/all-feature tests and 24 default CLI tests passed,
as did strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference, five OpenAPI checks, formatting and exact baseline
regeneration with --check.
Repository hygiene passed: tracked-file budgets, 427 local links, package
licenses and reference research checksums; staged whitespace passed.

Confirmed static APK 37.71 subject-cache follow-up: the post-commit Q4N selector
16 calls 118.A01 to remove the group from the cached version map. 118.A02
insertion preserves supplied versions and initializes participant_version=0;
the subject writer's update-miss branch supplies zeros for the other versions.
Literal DEX timestamp multiplication/division bytes are independently verified,
and the decoder now names all standard 90..af typed binary operations. The
versioned group recipe gains an insertion topic. This establishes research
boundaries, not a subject receive handler; full lifecycle/type/alias/system-message
parity remains open. No saved account opened or live request sent.

Validation: 591 workspace/all-feature tests and 24 default CLI tests passed,
plus strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference, five OpenAPI checks, formatting and exact snapshot
regeneration (--check, 12 files/395,753 bytes). All seven targets resolved uniquely.
Repository hygiene passed (429 local links, tracked-file budgets, licenses
and reference checksums), as did staged whitespace.

APK 37.71 complete-group classification is now persisted in Rust: parent,
default subgroup, general chat, linked subgroup, ordinary, with APK child
presence precedence and separate linked-parent identity. group list --json
exposes it; historical rows/local stubs remain null until a complete reply.
The actual snapshot API schema is corrected. Offline precedence, invalid-parent,
truncated and encrypted-store historical/reopen checks passed. Alias research
traces through 17H/0Xw to transactional contact mapping, not Signal pin replacement;
remaining storage/cache/callback bodies and subject handling are unfinished.

Observed bounded live classification check: one new info query for an already
known group settled ok and projected ordinary/null-parent with four members.
The unqueried historical row stayed unknown; zero queued actions/chat intents
remained after the clean 30-second session. The unresolved two-factor action
remained sent. Other group types and subject handling remain live unverified.
The eight-topic recipe has seven unique remaps and one two-candidate ambiguity;
proposal writing refuses rather than guessing. Validation: 593 workspace/all-
feature and 24 default CLI tests, strict workspace/all-target/all-feature Clippy,
warning-free Rustdoc, CLI reference, five OpenAPI checks, formatting and exact
snapshot regeneration (13 files/397,778 bytes) passed.
Repository hygiene passed (430 local links, tracked-file budgets, licenses
and reference checksums); staged whitespace and final OpenAPI rerun passed.

APK release maintenance now supports bounded method caller_anchors with full
cross-DEX signatures, preserving prefilter candidates/call sites and refusing
missing/split/noninvoking caller evidence. In APK 37.71, group_response caller
metadata resolves the classifier's ADs.A01/AZK.AAK ambiguity without old-name
fallback: all eight topics now unique (10.012s) and a new proposal was written.
Eight code topics remained unchanged in cached comparison; only target metadata
changed (baseline 397,832 bytes). Indirect/native dispatch and a distinct next
APK remain unverified. No saved account opened or live request sent this turn.

Validation: 595 workspace/all-feature and 24 default CLI tests, final strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, CLI reference,
five OpenAPI checks, formatting and exact snapshot regeneration passed. All
three actual caller-negative probes exited 1 without creating proposals.
Repository hygiene passed (432 local links, tracked-file budgets, licenses
and reference checksums); final staged whitespace checks passed.

APK 37.71 storage trace now separates group contact display-name writes from
row-ID JID mappings and closes alias cache commit/rollback cleanup. Full DEX
constructor signatures disambiguate four identical shorties; downstream listener
semantics, subject message parity and complete nested-transaction behavior remain
unknown. No Signal key replacement is established by this path. The group recipe
now has eleven unique remaps and a 16-file/423,242-byte baseline; three topics
were added and eight previous code topics stayed unchanged. Same-APK comparison
is coverage expansion, not a verified upstream release change. No saved account
opened, chat queued, or live request sent during this static work.
Validation: 40 xtask tests, five OpenAPI checks, regenerated CLI reference,
exact 16-file snapshot reproduction, all eleven selector comparisons and
repository hygiene passed. The unchanged runtime Rust code was not retested
across the whole workspace for this source-evidence change.
Final staged hygiene passed (435 local links, budgets, licenses and reference
checksums); staged whitespace and the five OpenAPI checks passed again.

APK-to-Rust release work now includes a reviewed 22-topic core 37.71 baseline
(27 files/731,482 bytes/150 methods) and a genuine cached 37.70→37.71 difference
in 0.0512 seconds. Fourteen old-topic relocations were unique; three ambiguous
and four unanchored selections required manual review. Automatic proposal
writing refused without creating an output. Text changes include obfuscation
and annotations and do not establish nineteen behavior changes.

Confirmed static APK 37.71 `1Rl.<clinit>` indexed stores exposed four corrupted
historical comparison token slots. The raw-byte heuristic is replaced with
bounded typed initializer evaluation. The runtime now uses a Derived 37.71
fixture, independently matching all 1,260 held web-v3 entries. Historical 37.70
artifacts remain immutable; the old raw APK is unavailable. Literal wire-byte
regressions cover all four corrected slots. Saved keys, reservations and event
markers are preserved; old projections are not silently replayed. See
[core review](docs/protocol/core-release-review.md) and
[token extraction](docs/protocol/token-extraction.md).

Validation: 598 workspace/all-feature tests, 24 default CLI tests, 42 final xtask
checks, strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference, five OpenAPI checks, formatting and exact core/group
snapshot reproduction passed. Final review also added a regression refusing a
const operand that resembles a final return opcode. A bounded 30-second live
receive session authenticated and exited 0; one automatic group refresh settled
ok, all sixteen existing outcomes stayed unchanged, and no queued actions or
eligible chat intents remained. No chats were sent. Individual corrected token
slots remain live unverified; fourteen unsupported incoming classifications are
recorded in the token extraction document. The two-factor sent action remains
unresolved. Full APK/client parity remains unfinished.

APK-to-Rust coverage now traces required subject timestamps to signed Long
parsing, cached group_type to its database column, and separate ordinary/parent
system-message construction. Fifteen group topics map to Rust ownership paths;
full subject application/message storage remains unfinished. A reusable checked
method_shorty recipe constraint resolves the three diagnostic-matching timestamp
parsers without obfuscated-name fallback, retaining prefilter evidence. Thirteen
topics remap uniquely; two string-free helpers still require manual relocation.
Wrong/malformed constraints refuse; whole-recipe proposal writing remains blocked
by those explicit unknowns. Signed 21s constants now show raw hex and negative
meaning, with independently checked APK insert-sentinel bytes.

The group baseline is 20 files/430,619 bytes; a cached comparison takes 0.0227
seconds (four topics added, two annotation-only changes, nine unchanged). The
core baseline remains byte-identical. No saved account opened or live request
sent during this static work. See [typed subject trace](docs/protocol/groups.md#subject-typed-branches-and-signed-timestamp-contract-2026-10-07).

Validation: 600 workspace/all-feature tests, 44 final xtask tests, 24 default
CLI tests, strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference, five OpenAPI checks, formatting, exact 20-file group
snapshot reproduction and byte-identical core comparison passed. Actual wrong
signature, malformed parameter and missing-method probes refused as documented;
no proposal was created for either unresolved full mapping or wrong signature.
Repository hygiene passed (467 local links, budgets, licenses and reference
checksums), with clean staged whitespace. Runtime/client behavior is unchanged
by this research-tool and source-coverage checkpoint.

Further APK 37.71 subject recovery tracing establishes a material runtime gap:
0Xm.A0F dispatches normal group info through MEX QueryGroupInfo, while Rust's
working group-info command uses classic IQ. Exact ten-reason context mapping,
private-set override, mode-5/6 in-flight guard and the separate 60,000-ms local
monotonic rate guard are preserved, including its confirmed map write. Error
429 is traced to an A0G time update and an untraced runnable, not a proven retry
wait. Envelope/document ID, response conversion and complete callbacks remain
unknown; no MEX endpoint or subject handler is claimed implemented.

Group source coverage is now twenty topics, 25 files/448,127 bytes. Sixteen
remaps are unique; four string-free helpers remain unresolved automatically.
Cached same-APK comparison took 0.0204 seconds (five added, fifteen unchanged).
No saved account opened or live request sent. See [recovery trace](docs/protocol/groups.md#subject-recovery-mex-dispatch-and-local-guard-2026-10-07).

Validation: 44 xtask tests, five OpenAPI checks, regenerated CLI reference,
exact 25-file/448,127-byte snapshot reproduction, current twenty-topic remap,
repository hygiene (477 local links, tracked-file budgets, licenses/reference
checksums) and clean staged whitespace passed. Shared runtime Rust was unchanged
and was not retested across the workspace for this source-only follow-up.

APK 37.71 MEX request construction now has an offline Rust port. Selected
QueryGroupInfo persisted IDs export as a small version/hash-scoped file; exact
server ID appears in trace, query attribute and nested JSON. The release CLI
checks the real APK export; an offline preview invokes the actual Rust builder.
This advances APK-to-Rust behavior and file-based release patching. It does not
queue a MEX action or overwrite group state; complete JSON/Argo/Pando response
handling and durable dispatch remain unfinished. Namespace initialization,
assertion failure semantics and timeout units remain explicit unknowns. See
[MEX source/port](docs/protocol/mex.md). No saved account opened or live request sent.

Validation: 604 workspace/all-feature tests and 24 default CLI tests passed;
final duplicate-ZIP correction passed all 47 xtask tests. Strict workspace
Clippy and warning-free Rustdoc passed, with the corrected xtask rechecked.
CLI reference was regenerated; all five API checks passed. Real APK export
including provenance matched the fixture after the correction; the Rust node
preview and five negative CLI probes passed (refusals emitted no partial stdout).
Repository hygiene checked 487 local links, tracked budgets, licenses and
reference checksums; formatting and staged whitespace passed. No account
opened, live request sent or existing saved state changed.

MEX JSON response provider 0x1763 is now traced through the actual DEX switch
entry to O4r selector 0. Nullable object data and array errors parse separately;
valid counterparts survive malformed fields. Rust reproduces these response
parts with explicit invalid states and redacted inspection, without inventing
Java exception strings or treating an IQ result as group success. The subject
getter is now confirmed as subject/8yq. Full callback/error schema, Argo/Pando,
group conversion and durable MEX dispatch remain unfinished. Sixteen synthetic
field-type combinations cover both result/update. See [parser trace](docs/protocol/mex.md#json-response-field-port-2026-10-07).
No saved account opened or live request sent in this work.

Validation: 607 workspace/all-feature tests, 24 default CLI tests, all 47 xtask
tests, regenerated CLI reference and five OpenAPI checks passed. Strict
workspace/all-target/all-feature Clippy, warning-free workspace Rustdoc and
formatting passed.
Five positive and three refusal CLI probes preserved counts/types without
content or partial failure output. The twelve-topic MEX baseline reproduced
exactly (17 files/58,761 bytes); cached comparison took 0.0064 seconds.
Remap has six unique/four unanchored/two ambiguous mappings, with no complete
proposal claimed. Repository hygiene and staged whitespace passed. No account
opened, outgoing intent queued or live request sent.

MEX group callback gate is now ported: supported partial data with nonempty
error objects blocks the success callback; nonobject error entries and absent
model are explicit separate states. Nonempty error-free data can invoke the
callback, but its feature gate and complete group conversion/application are
unfinished. --group-info exposes the redacted disposition offline. Direct DEX
superclass inspection closed the inherited 9Np/BaseMexCallback C2c path;
IWF error fields and partial ADv getters remain source-only where not ported.
See [callback trace](docs/protocol/mex.md#group-callback-gate-and-inherited-dispatch-2026-10-07).
No account opened, intent queued or live request sent.

Callback-gate validation: workspace/all-feature tests **609 passed**, default
CLI **24 passed**, CLI reference regeneration, OpenAPI library-symbol check,
strict workspace/all-target/all-feature Clippy, warning-free Rustdoc and
formatting passed. Actual CLI probes covered **20** callback cases across
result/update plus **10** default-mode checks with redacted output. The existing
37.71 MEX bundle reproduced exactly (**17 files, 58,761 bytes**). No live
account request or group-state application occurred in this chunk.

MEX callback source coverage is now versioned with the request/parser files:
18 topics, 23 files/76,908 bytes, exact APK reproduction passed. Cached
comparison against Git cdd4056 took 0.0069 seconds: six added/twelve unchanged
topics on the same APK. Remap: eight unique/seven unanchored/three ambiguous;
is_retryable/shorty-Z still matches three methods and needs review. Full group
conversion, Argo/Pando and durable live application remain unfinished.
All 48 xtask tests passed; no Rust behavior changed or account request sent.
See [versioned MEX files](docs/protocol/mex.md#versioned-parserrequest-files).

Coverage validation also passed exact checksum membership/18 single-method
selections, CLI reference regeneration, OpenAPI library-symbol validation,
repository hygiene (518 local links) and staged whitespace.

MEX converter static tracing now preserves full 0Xo.A03's four hash branches,
required group/participants/properties versus nullable subject, separate
1000-scaled group/subject times and unscaled announcement version, plus
feature-gated LID/country creator metadata. Wire type names, A4m slot mapping,
Java coercion/catch behavior and full membership conversion remain unfinished.
The disassembler now names the independently verified mul-long/2addr opcode;
this adds no runtime state application or live request. See [converter trace](docs/protocol/mex.md#group-converter-branches-and-time-arithmetic-2026-10-07).

Converter-trace validation: **610** workspace/all-feature tests (including
**49** xtask tests), strict workspace/all-target/all-feature Clippy, formatting,
CLI reference regeneration, OpenAPI library-symbol check, exact MEX snapshot
reproduction (23 files/76,908 bytes), hygiene (523 links) and staged whitespace
passed. No saved account opened, queued intent or live validation performed.

MEX result constructor mapping now confirms group type A03, distinct long
slots and participant/creator maps, preserving unknown flag/getter meanings.
The long parser's NumberFormatException catch returns the supplied zero;
reflection fallback catches four named exceptions, not arbitrary Throwable.
apk-smali --exceptions exposes checked code-unit ranges/types/targets, with
independent APK code_item regression. Complete numeric coercion, participant
conversion and durable group application remain unfinished. No live request.
See [constructor/catch trace](docs/protocol/mex.md#result-constructor-and-explicit-exception-inspection-2026-10-07).

Constructor/catch validation: **615** workspace/all-feature tests, including
**54** xtask tests, strict workspace/all-target/all-feature Clippy, warning-free
Rustdoc, formatting, CLI reference regeneration and OpenAPI library-symbol
validation passed. Actual CLI probes verified the typed numeric catch, four
reflection catches, explicit zero-range result, unchanged default output and
named-file/no-partial-output refusal for malformed later DEX. All three 37.71
bundles reproduce: MEX 23 files/76,908 bytes; groups 25/448,127; core 27/731,494.
Core's one prior op=bd unknown annotation was refreshed with identical opcode,
offsets and operands (one changed/21 unchanged topics, four unchanged artifacts,
same APK; cached presentation diff 0.0328 seconds). Hygiene (533 local links)
and staged whitespace passed. No live request or saved account mutation.

MEX numeric group conversion is now in Rust: distinct signed creation/subject
numbers scaled by 1000 and unscaled announcement version, zero fallback and
wrapping arithmetic. Independent host Java reference covers 479 cases and
all UTF-16 code units (370 decimal digits/37 BMP blocks); Android runtime/SDK
and full JSON lexical parity remain unverified. --group-numbers with
--group-info explicitly prints private numbers as decimal strings; default
summary/Debug remain redacted. Full membership/type/feature/state application
remains unfinished. No live request or saved account mutation. See [numeric
projection](docs/protocol/mex.md#numeric-group-projection-into-rust-2026-10-07).

Numeric-port validation: **618** workspace/all-feature tests, **24** default
CLI tests, strict workspace/all-target/all-feature Clippy, warning-free Rustdoc,
formatting, CLI reference regeneration and OpenAPI library-symbol validation
passed. Actual CLI probes: **70** numeric cases across result/update, **70**
unchanged redacted modes, flag dependency and six content-free/no-partial-output
refusals. Oracle runtime was checked directly as Temurin **21.0.12.1+1 LTS**;
479 corpus cases and all 65,536 UTF-16 units matched that independent reference.
Hygiene (540 local links) and staged whitespace passed. No saved account,
identity, action state or live server request was changed.

MEX participant roles now convert in Rust from nullable server strings using
APK 37.71 enum/fallback and exact member role mapping; missing/unknown versus
ordinary member stay distinct. Offline `mex-role-inspect` invokes that helper
without echoing input. Independent host Java reference contains 91 cases;
full Android Unicode-version parity remains unverified. The same trace
confirms phash retain semantics and separate BotJid/UserJid participant paths;
full member conversion and durable application remain unfinished. See
[role/participant evidence](docs/protocol/mex.md#participant-role-conversion-and-member-map-boundaries-2026-10-07).
No saved account, identity, queued action or live request changed.
Role-port validation: 619 workspace/all-feature tests and 24 default CLI tests,
strict Clippy, warning-free Rustdoc, formatting, CLI reference and API symbol
checks passed. Actual CLI matched 90 representable oracle inputs plus omission;
the embedded-NUL case is library-tested because OS argv cannot carry NUL.
Hygiene checked 548 local links and staged whitespace passed.

APK release comparison now includes 38 MEX methods / 43 files / 162,435 bytes,
with opt-in typed/catch-all exception tables. Default-format exports remain
unchanged. Actual cached diff took 0.023364 seconds; twenty new topics and
eighteen catch-annotation additions, with all prior instructions identical.
A catch-only mutation test now verifies handler changes are visible. Actual
remapping: 14 unique, 17 unanchored, 7 ambiguous; full automatic relocation
still requires review. Source coverage does not complete member/state ports.
[Coverage and catch evidence](docs/protocol/mex.md#exception-aware-numeric-and-participant-release-coverage-2026-10-07).
No account opened, state changed or live request sent.
Exception-aware snapshot validation: 620 workspace/all-feature tests (55
xtask), 24 default CLI tests, strict Clippy, warning-free Rustdoc, formatting,
CLI reference/API symbols, hygiene (557 local links), staged whitespace passed.
Both actual exports reproduced: new 43-file catch-aware snapshot and prior
23-file default-format snapshot, with complete checksums unchanged for the latter.

MEX object-edge roles now project in Rust and via explicit --edge-roles on
mex-response-inspect (--group-info required): ordered recognized enum strings,
nonobject skips and nullable/wrong-type list fallback. Eighteen synthetic cases
cover both payload children. This is a field projection before phash/member
validation, not a member map/state update. Participant constructor fields and
history/join boxing are now traced; literal class tables confirm BotJid is a
UserJid through abstract 0ZX, whose zero class-data pointer explains the empty
method lookup. Remaining full conversion/type/alias/boolean/application gaps
are [documented](docs/protocol/mex.md#participant-model-fields-and-object-edge-projection-2026-10-07).
No saved account, identity, action state or live request changed.
Edge projection validation: 621 workspace/all-feature tests, 24 default CLI
tests, strict Clippy, warning-free Rustdoc, formatting, CLI reference/API symbol
checks and hygiene (564 local links) passed. Actual CLI checked 36 edge modes,
36 combined numeric modes, 36 unchanged redacted defaults and the flag dependency;
blocked projections never emitted partial stdout or private sentinel content.

MEX phash/history/join scalar coercions now project in Rust and explicit
--participant-scalars (--group-info required), with redacted Debug/defaults.
408 independent pinned-AOSP/host-Java cases cover number wrapping, string/float
saturation, decimal/hex rounding and nullable values. A local serde JSON
precision bug falsely mapped 0.99999999999999994 to1/phash=true; float_roundtrip
fixes it with a regression. Full Android/JSONTokener and member-map/state
application remain unverified/unfinished. Snapshot owners now include the
coercion module, 43 files/162671 bytes; all 38 topic texts unchanged.
[Evidence and limits](docs/protocol/mex.md#participant-scalar-coercion-and-float-rounding-2026-10-07).
No saved account, keys, identity, action state or live request changed.
Scalar-port validation: 623 workspace/all-feature tests, 24 default CLI tests,
strict Clippy, warning-free Rustdoc, formatting, CLI reference/API symbols and
hygiene (578 local links) passed. All 408 oracle cases reproduced using the
committed annotation stubs; actual CLI checked 816 scalar modes, 816 combined
modes, 816 unchanged redacted defaults, 16 blocked/no-partial-output modes and
flag dependency. Actual 43-file/162671-byte APK snapshot re-export matched.

Phone/account inventory audit (2026-10-07, offline): 19 profile files reference
13 distinct country/national-number pairs across 16 existing configurations.

Participant address trace (2026-10-07, static): APK 37.71 strict member/alias
checks reject device wrappers (DeviceJid extends Jid, outside UserJid); plain
bot/LID/interop constructors qualify. Phone/LID digit and length rules, reserved
phone prefix, interop last-hyphen/signed parseInt behavior and plain hosted-domain
normalization are recorded with scoped instructions and thirteen literal class
definitions. Full singleton parsing/member assembly/durable MEX application remain
unfinished; no API/CLI change, saved account access or live request.
[Evidence and negative results](docs/protocol/mex.md#participant-address-validation-and-strict-type-boundary-2026-10-07).
Validation: APK hash and all thirteen class-definition literal bytes rechecked;
phone validation, full interop class and DeviceJid constructor re-exported exactly.
Repository hygiene passed (583 local links); staged whitespace passed. This
source-only chunk changes no runtime code, so no new runtime test result is claimed.


Strict MEX participant primary-address parser now implemented (2026-10-07):
constructor/type guards, plain hosted normalization, signed/Unicode interop
integrators and confirmed Server/0/Me singleton handling, separate from sending
eligibility. New offline inspector outputs subtype only and redacts failures.
283 independent Java references and nineteen domain class literals preserved;
full member/alias/durable MEX application and Android/cache parity remain open.
Release bundle expands to 50 methods / 55 files / 191422 bytes; 38 old texts
unchanged, cached diff0.051814s. No saved account access or live request.
[Evidence and limitations](docs/protocol/mex.md#strict-member-primary-address-parser-in-rust-2026-10-07).

Validation: 625 workspace/all-feature tests, 24 default CLI tests, strict workspace
all-target/all-feature Clippy, warning-free Rustdoc, formatting and regenerated
CLI reference/API-symbol checks passed. After the negative-prefix CLI correction,
all 56 xtask tests and strict xtask Clippy passed. Actual CLI:282 integrator cases,
seven subtype successes, eight content-free/no-stdout refusals; NUL library-only.
All nineteen domain class literal records were reread against the APK; final
snapshot checksums and repository hygiene passed (591 local links). Staged
whitespace passed. No complete member conversion or live validation is claimed.


Nullable ordered MEX member map implemented (2026-10-07): phash null remains a
separate variant, invalid nullable bots skip, strict primary/alias failures abort,
canonical duplicate keys replace values in first insertion order. Role/join/history/
display slots mapped, with explicit refusal for unported Java floating/structured
string coercions. Private --participant-members inspection composes with prior
projections; failed partial maps, Debug and default output remain redacted.
41 synthetic APK-branch fixtures and eight independently executed Java hashes
cover both payload paths. Actual typename names, complete group conversion and
live/durable application remain unknown/unverified. No saved account/live action.
[Sources and current boundary](docs/protocol/mex.md#nullable-ordered-participant-map-port-2026-10-07).

Validation: 628 workspace/all-feature tests, 24 default CLI tests, strict workspace
all-target/all-feature Clippy, warning-free Rustdoc, formatting, regenerated CLI
reference and API-symbol checks passed. Actual CLI:82 member modes,82 combined,
82 unchanged default summaries,56 no-partial-output refusals and flag dependency.

Release snapshot:59 methods/64 files/204600bytes; all 50 prior instruction texts
unchanged, checksums verified; cached comparison0.019005s. Builder numeric type
predicates are documented separately from strict alias instanceof checks; broader
32q callers remain untraced. No device-alias acceptance was introduced.

After diagnostic improvements, the full workspace tests/Clippy/Rustdoc reruns
passed. Refusals name the offending attribute without its value. Scoped model
constructors were confirmed as wrappers only; broader string formatting remains
unported. Repository hygiene passed (599 local links); staged whitespace passed.


MEX metadata strings (2026-10-07, offline): formerly unsupported floating,
object and array display metadata now convert against pinned AOSP/host-Java
references. Private ordered parsing preserves duplicate position, insertion order,
escaping and integer negative-zero classification; original public serde data,
requests and persisted serializers are unchanged. Independent exact-rational
Java 21 formatter adds no dependency or copied JDK source. 2242 IEEE and 424
raw JSON references, including 848 member conversion paths, passed. Android
runtime-version/lexical equivalence and full/live durable group conversion remain
unverified. Evidence and discovered reference-harness failure are preserved in
[the string trace](docs/protocol/mex.md#participant-metadata-string-conversion-2026-10-07).

Validation: 632 workspace/all-feature tests, 24 default CLI tests, strict Clippy,
warning-free Rustdoc, formatting and regenerated CLI reference passed. Actual CLI
checked 82 member modes, 82 combined modes, 82 default redacted modes and 40
blocked/no-partial-output cases, plus 848 raw structured member checks and 848
unchanged default summaries. APK owner-only re-export has 59 unchanged method
texts / 64 files / 205234 bytes; cached diff took 0.038028 seconds. No account
access, registration, live transmission or new number acquisition occurred.
Final repository hygiene passed: tracked-file budgets, 611 local links, package
licenses and 564315 bytes of APK research checksums; staged whitespace was clean.
The formatter corpus also passed after the strict-Clippy parity expression fix.


MEX member-storage consumers (2026-10-07, offline): APK 37.71 A4m.A0O has one
behavioral reader, 0yw.A0X; A0U skips member reconciliation for null but calls it
for nonnull empty maps. Consumer timestamp scaling/previous-clock fallback and
nullable-history integer preservation are now implemented as a pure Rust helper
and GroupParticipant method, with explicit private-JSON CLI inspection. 432 host
Java transcribed-branch reference cases and 432 actual CLI cases passed; nine
invalid/oversized input refusals stayed content-free with empty stdout. Full
membership/alias/device/own-account application, storage atomicity and live MEX
settlement remain unfinished. [Evidence](docs/protocol/mex.md#downstream-member-storage-and-null-map-2026-10-07).

Validation: 634 workspace/all-feature tests, 24 default CLI tests, regenerated CLI
reference, warning-free Rustdoc, formatting and final strict Clippy passed.
The initial Clippy run found redundant test-only qualifications; they were
removed and strict Clippy rerun successfully. No live request/account access or
identity/key change. Versioned bundle adds seven methods (66 methods / 71 files /
285828 bytes), preserving all 59 prior texts; cached diff 0.035567s. Baseline 8295e18.

Final hygiene corrected an existing coverage gap: its checksum section formerly
checked only 37.70, while MEX had separate diff verification. It now checks all
four discovered snapshot bundles (2029764 bytes), with 621 local links, tracked
budgets and package licenses passing. Its standalone corruption/size/extra-file
test passed and is wired into CI. Staged whitespace and CI-tool formatting passed.
[Scope correction](docs/protocol/apk-porting.md#checksum-coverage-correction-2026-10-07).
Final diagnostic correction reran all 634 workspace/all-feature tests and strict
Clippy successfully; all 432 actual CLI cases and nine named refusals passed.
Actionlint accepted the changed workflow. No new live validation is claimed.


Background device-sync correction (2026-10-07): durable local revisions keep
newer pending notifications when an older reply settles; notification pending
work commits before ACK writing. The receive actor continues eligible 64-user
batches while deferring refused/errored/omitted/unanswered users for this
connection. No wire/identity changes or automatic refusal replay. Offline
injected-link checks cover 130 users, newer-notification race, ACK socket-cut
reopen and timeout/late reply; live coverage of these branches remains unknown.
Own-account sync and notification-ID deduplication remain unfinished.
[Evidence and limits](docs/protocol/messaging-transport.md#background-device-revisions-and-continuation-2026-10-07).

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
[Evidence and unknowns](docs/protocol/cross-device-sync.md#contact-adv-verification-and-retention-2026-10-07).
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
[Evidence and limits](docs/protocol/cross-device-sync.md#rejected-device-candidate-retention-2026-10-07).

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
[Evidence and limits](docs/protocol/cross-device-sync.md#signed-device-authority-and-authenticated-refresh-2026-10-07).


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
[Source and limits](docs/protocol/cross-device-sync.md#adv-identity-follow-up-and-raw-id-cleanup-2026-10-07).

Primary identity-only read staging (2026-10-07): peer-identity-query queues the
APK 37.71 encrypt/identity read once; --show classifies saved primary PN/LID
public-key evidence with exact namespace pin match and redacted failures.
B7H type 5 / first 32 key bytes and companion dual-signature checks are traced.
Four offline regressions preserve state and uncertain/refused IDs. Automatic
ADV-failure scheduling, key adoption, companion policy and downstream success
event/session recovery remain unfinished; presence is not verification.
[Evidence and usage](docs/protocol/cross-device-sync.md#primary-identity-only-query-2026-10-07).

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
[Evidence and limits](docs/protocol/cross-device-sync.md#primary-save-callbacks-and-device-sender-key-storage-2026-10-07).

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
See [evidence and next implementation](docs/protocol/cross-device-sync.md#scoped-sender-key-removal-and-recipient-translation-2026-10-07).


Own sender-key exposure/rotation (2026-10-07): the comparison sender reserves
exact recipient-device exposure with key/session/send state. An authenticated
remote primary change marks indexed own keys pending in the same archive/event
transaction; the next new authorized group/status intent rotates with OCC and
private old-key archival. Incoming chains and uncertain intents are retained.
`peer-identity` exposes redacted pending_sender_key_rotations counts; zero does
not establish historical coverage. APK 37.71 D4x.A00/0jc.A19 motivate this local
subset; full bucket/alias/companion and normal actor group parity remain
unimplemented, and live rotation is unverified.
See [evidence and limits](docs/protocol/cross-device-sync.md#durable-own-sender-key-exposure-and-rotation-2026-10-07).

Validation of own-key exposure handling: 666 final workspace/all-feature tests,
24 default CLI tests, strict Clippy, warning-free Rustdoc, CLI/API checks passed.
A 20-second receive-only rebuilt-client check left peer/device/action/outbox/receipt
inspection unchanged. Live rotation remains unverified. The initial workspace
wrong-key reopen test failed AlreadyOwned; focused and full reruns passed without
relaxing locking or key validation.


Directory bundle selection (2026-10-07): shared parser now compares user/server/
agent/device without conflating wire-encoding kinds, refuses duplicate requested
user rows and optional one-time keys, and refuses contradictory outer errors.
These local ambiguity guards precede identity/session preparation; other batch
devices remain valid. Existing peer-keys inspection reports invalid_reply and
blocks refresh while preserving its selector and pins. APK 37.71 B7H.A06/CQc.A00
provide address/signature context; exact duplicate policy is not attributed to
the APK. [Evidence and tests](docs/protocol/peer-identity.md#exact-directory-bundle-selection-2026-10-07).

Validation of exact directory selection: 667 workspace/all-feature tests, 24
default CLI tests, strict Clippy, warning-free Rustdoc and CLI reference passed.
Rebuilt-client offline inspection retained verified_bundle/signature_valid=true
for the existing saved server response without changing protocol inspection.
No new live query or network session; malformed live responses remain unobserved.


Missing group sender-key correction (2026-10-07): supported nonempty skmsg with
no sender key is now Undecryptable, reaching the existing once-only retry queue;
it is no longer silently Unsupported. Empty/structured skmsg with a stored key
cannot be reported Decrypted or delivery-confirmed. Byte/Text ciphertext views
are consistent across direct/group receive. Real-crypto late-key recovery and
direct/group loop write-cut fixtures preserve captures, ratchets and retry
reservations. Handset retry shape is observed; APK 37.71 Cp2.A05 is ACK evidence,
not a confirmed missing-key retry count policy. Fresh live group recovery remains
unverified. [Evidence and limits](docs/protocol/messaging-transport.md#missing-group-sender-key-recovery-2026-10-07).

Validation of missing-group-key handling: 668 final workspace/all-feature tests,
24 default CLI tests, strict Clippy, warning-free Rustdoc and CLI reference passed.
A 20-second receive-only rebuilt-client session left protocol inspection unchanged;
no fresh missing-key group recovery was observed and no chat intent was added.


SKDM-only group control deduplication (2026-10-07): real Signal regression
reproduced Undecryptable after a previously accepted control was redelivered
across reopen. Newly authenticated controls now commit sender_key_distributed
markers with key/session state and return Duplicate without another decrypt.
They carry no user protobuf/event/read receipt; lab-inbox --inspect labels them
redacted. APK Message field 2 is existing schema evidence; this private marker
is a local atomicity fix, and APK internal duplicate policy is unknown. No
historical marker synthesis or ratchet rewind; fresh live control redelivery
remains unverified. [Evidence and regression](docs/protocol/messaging-transport.md#durable-sender-key-only-group-controls-2026-10-07).

Validation of SKDM-only control markers: 669 workspace/all-feature tests, 24
default CLI tests, strict Clippy, warning-free Rustdoc and CLI/API checks passed.
A 20-second receive-only rebuilt-client check left protocol inspection unchanged;
no fresh control/redelivery was observed, so that live branch remains unverified.


Retry/ACK encoding identity correction (2026-10-07): a reproduced second-retry
reservation caused only by JidKind differences is fixed. Canonical logical
selectors preserve user/server/agent/device and probe all historical kind keys
in the same transaction, retaining pending/unknown/refused/acknowledged records
and original wire. ACKs ignore encoding kind but keep exact logical routes.
Independent historical-hash, reopen/state-preservation, foreign-route ACK and
actual receive-loop cut/reconnect tests cover the correction. APK 37.71 Cp2.A05
is wire ACK context; APK duplicate/retry-limit policy remains unknown. Historical
duplicate rows are not consolidated; live encoding changes remain unverified.
[Evidence](docs/protocol/messaging-transport.md#retry-reservation-identity-across-jid-encodings-2026-10-07).

Retry encoding validation: 671 workspace/all-feature tests, 24 default CLI
tests, strict Clippy, warning-free Rustdoc and regenerated CLI/API checks passed.
No live retry was manufactured or settled request replayed; actual server
encoding-altered redelivery/ACK remains unverified.

Primary-change encoding correction (2026-10-07, confirmed offline): reproduced
missed own group-key rotation for a remote AgentDevice/0 primary and accidental
own-PN device refresh. Encoding is normalized only after primary agent/device
checks; PN/LID namespaces and companion boundaries remain exact. Redacted rotation
inspection now ignores kind without folding devices. PN incoming and LID signed
bundle archive/reopen, own exclusion, rollback and stale-reservation tests cover
the local correction. APK 37.71 D4x.A00/B7H.A06 remain source context; live alternate
primary encoding and subsequent group rotation remain unverified.
[Evidence](docs/protocol/cross-device-sync.md#encoding-independent-primary-change-follow-ups-2026-10-07).

Primary-change encoding validation: 673 workspace/all-feature tests, 24 default
CLI tests, strict Clippy, warning-free Rustdoc and CLI/API checks passed. Rebuilt
CLI read-only saved-account inspection left all inspected protocol state unchanged;
no login/query/send or manufactured identity change was used to test the branch.

Own sender account selection (2026-10-07, confirmed static APK 37.71): 08i
initializes Apw from self_lid and Apz from saved Me.jabber_id; both retain the
persisted registration_device_id (default 0). D4x.A00 selects LID versus PN using
0bA.A0k -> 1jL.A0Y (group row.A00 == 2), then wraps the own device via B7H.A04.
Me.lid consistency diagnostics are not an established immediate self_lid fallback;
1XY is a call callback, not run. Group-row mode production, async reconciliation,
complete bucket/actor semantics and Rust own-address parity remain unfinished.
No account state, keys, identity policy or live traffic changed.
[Sources and boundaries](docs/protocol/cross-device-sync.md#own-pn-lid-sender-selection-and-persisted-device-index-2026-10-07).

Own-account trace validation: 58 xtask tests, CLI reference regeneration and five
API checks passed. The core now holds 81 topics / 219 methods, 86 files /
1,586,459 bytes under its 2 MiB limit. The checksum-verified cached diff took
0.080554 seconds: 13 added topics, all 68 prior topics unchanged; manifest,
inventory and tokens unchanged. This measures cached scoped source comparison,
not full-app decompilation or automatic porting. No live request was needed.

Group-mode producer/storage trace (2026-10-07, confirmed static APK 37.71): five
1jL.A00 writes and its constructor input are retained. Cache-miss mode comes
from B7i.A00 chat predicates, not membership suffixes alone; contact mode is saved
through 0Y9.A0s -> 0gt.A0V -> 9R1.A0J in wa_group_admin_settings. Reload 0lO.A01
normalizes case-insensitive pn to pn, other returned values to lid. B7i lookup
and predicate have different absent-contact defaults; broadcasts have additional
diagnostic behavior. Full mode provenance, category predicates and Rust mode/
own-PN fallback integration remain incomplete. No live/account state changed.
[Evidence and negative results](docs/protocol/cross-device-sync.md#group-mode-producers-storage-and-cache-classification-2026-10-07).

Group-mode trace validation: 58 xtask tests, CLI reference regeneration and five
API checks passed. The core contains 100 topics / 238 methods, 105 files /
1,731,431 bytes under the 2 MiB budget. The checksum-verified cached diff took
0.075441 seconds: 19 added topics, all 81 prior topics unchanged; manifest,
inventory and tokens unchanged. No live/account request or runtime migration
was used for this source trace.

Group routing before device sync (2026-10-07, confirmed offline): the comparison
sender's catch-all plan fallback returned device-query targets even after route
refusals. The extracted old logic failed a mixed-membership regression. Shared
recipients_for_group now validates namespace/own membership before sync, allows
initial missing lists and excludes/deduplicates own/remote targets. Final plan
still validates device authority before reservation. APK mode/category state
integration and automatic own-PN selection remain incomplete; no new live send.
[Routing correction](docs/protocol/groups.md#group-recipient-validation-before-device-sync-2026-10-07).

Group-routing validation: 675 workspace/all-feature tests, 24 default CLI tests,
strict Clippy, warning-free Rustdoc and regenerated CLI/API checks passed. Rebuilt
CLI read-only inspection left outbox, actions, receipts, devices and pins unchanged.
The private evidence index is lab/releases/group-routing-inspection/active.json;
no login, device query, group send or identity replacement was manufactured.

Concrete group-mode kinds (2026-10-07, confirmed APK 37.71): 0DE predicates
resolve broadcast-list type 3, interop.g.us type 28, status@broadcast type 5,
and cache-group types 1=g.us/2=temp. A0n additionally includes type 28.
Temporary groups are not established as old PN g.us groups. Existing 0Fs cache
and ADs reply classifications are linked; numeric group-type agreement alone
does not establish complete callback/history provenance. Runtime mode/contact
synchronization and unsupported namespaces remain unported; no account request.
[Kinds and remaining gaps](docs/protocol/cross-device-sync.md#concrete-jid-kinds-in-group-mode-classification-2026-10-07).

Concrete-kind trace validation: 58 xtask tests, CLI reference regeneration and
five API checks passed. Core snapshot: 109 topics / 275 methods, 114 files /
1,755,155 bytes (2 MiB bound); supplementary A0n source is separate. The
checksum-verified cached diff took 0.197003 seconds: nine added topics, all
100 prior topics unchanged; manifest, inventory and tokens unchanged. These are
static/offline findings; no live behavior or automatic mode adoption is claimed.

Group-type persistence continuation (2026-10-07, **confirmed static APK 37.71**):
existing-chat updates write group_type through transactional database helpers;
missing chats are left absent. Selected subgroup/zero-update callers and secondary
type-4 model writes are retained without inventing complete-reply enum values.
A0Y's outer A0n guard is narrowed again by A0A/A0m, excluding interop type 28
from the type-3 branch. Runtime contact/type synchronization remains incomplete;
no account/network request. [Evidence and unknowns](docs/protocol/cross-device-sync.md#persisted-group-type-updates-feeding-sender-mode-2026-10-07).

Group-type trace validation: 58 xtask tests, regenerated CLI reference and five
API checks passed. The core checksum bundle now has 118 topics / 289 methods,
123 files / 1,829,892 bytes (2 MiB bound). The verified cached diff took
0.168698 seconds: nine added topics, all 109 previous topics unchanged;
manifest, inventory and tokens unchanged. Selected runnable excerpts remain
outside the checksum bundle. No runtime behavior or live validation is claimed.

Addressing-mode producer trace (2026-10-07, **confirmed static APK 37.71**):
complete binary group replies do not directly read the mode attribute. Generated
validators use exact pn/lid membership; incoming group-message parsing has a
separate case-insensitive PN check. Five contact-mode setter callers now have
selected paths retained, including gated info-error and PN-conversion branches;
triggers, full callback provenance and runtime integration remain unfinished.
No live request or account change. [Evidence and boundaries](docs/protocol/groups.md#addressing-mode-producers-and-separate-parser-rules-2026-10-07).

Addressing-mode trace validation: 58 xtask tests, regenerated CLI reference and
five API checks passed. Core checksum bundle: 129 topics / 304 methods,
134 files / 1,900,193 bytes (2 MiB bound). Verified cached comparison took
0.151473 seconds: eleven new topics, all 118 prior topics unchanged; manifest,
inventory and tokens unchanged. Scan/caller appendix remains outside the bundle.
No runtime migration, automatic error recovery or live validation is claimed.

Direct LID representation correction (2026-10-07, **confirmed offline**):
textual and typed sender JIDs now select the same held LID session. A real
Signal regression failed Undecryptable before the fix; both representations
now decrypt and cross-representation redelivery remains Duplicate after reopen,
with no PN session or identity rewrite. APK DEw.CF0 parsing is linked, while
full direct-session producer provenance remains incomplete.
[Evidence and boundaries](docs/protocol/messaging-transport.md#direct-lid-sessions-across-attribute-representations-2026-10-07).

Direct-LID correction validation: 676 workspace/all-feature tests and 24 default
CLI tests passed, along with strict all-target/all-feature Clippy, warning-free
Rustdoc, CLI reference/API checks and the experimental CLI build. The first
local preflight ran before that build was terminal and encountered an older
CLI without outbox-pending; it stopped before opening an account connection.
A subsequent preflight uses the completed build.

**Observed live follow-up:** the completed experimental CLI passed queue preflight
and authenticated for one 20-second receive-only window, then stopped cleanly.
Outbox, actions, receipts, retained devices and peer pins matched their preflight
inspections afterward. No fresh encrypted user-message stanza was logged, so
live text/typed-LID decryption is not established by this window. Private evidence
index: lab/releases/direct-lid-live/active.json. No queued chat transmission,
new identity query, reservation reset or manufactured recipient message.

Group pairwise companion correction (2026-10-07, **confirmed offline**):
held and pinned companion msg sessions retain their actual participant device;
primary/alias fallback, unpinned and prekey-bootstrap paths are excluded from
this local subset. Real Signal PN/LID primary/device-3 control fixtures now
install sender keys and retain duplicate markers across reopen. B7H forward/
reverse device construction is linked; CPQ.A00 is refuted as a 37.71 session
selector. Full companion bootstrap/identity policy and live device coverage
remain unfinished. [Evidence and scope](docs/protocol/groups.md#group-pairwise-session-device-correction-2026-10-07).

Companion pairwise refusal diagnostics name unsupported prekey or nonzero-agent
routing. Missing held/pinned state reports only session_held and identity_pinned
Booleans; no user/device address, key bytes, ciphertext or message content.

Group companion correction validation: 677 workspace/all-feature tests, 24
default CLI tests, strict all-target/all-feature Clippy, warning-free Rustdoc,
regenerated CLI reference/API checks and the experimental CLI build passed.
The selector regression returned a primary before the fix, and the encrypted
companion control fixture returned Unsupported before the correction.

Rebuilt-CLI offline inspection left outbox, actions, receipts, device state and
peer pins unchanged. Private evidence index:
lab/releases/group-device-inspection/active.json. No login/query/group send or
new companion identity was manufactured. Fresh live companion decryption and
full companion bootstrap remain unverified.

Incoming companion prekey caller trace (static, 2026-10-07): APK 37.71
CoU.A04 supplies a companion candidate to 11T.A0D before CQn.A00 dispatches
kind-1 ciphertext to 0jc.A0O. The latter's B77.A04 branch is STATELESS,
using fresh DS0 in-memory sessions and DRz's no-op identity save; ordinary
0ko.A04 uses durable adapters. This refutes conflating stateless mode with
companion addressing. Typed catches now associate obfuscated exceptions with
local decrypt results without inventing server refusal/ban interpretations.
DOL.call is a voice-call ADV gate, not messaging admission evidence. Full
sender/ADV/address producers, feature/dead-device conditions, transaction
coverage and companion bootstrap/application policy remain unported/unknown.
The group selector still accepts only held exact pinned companion msg sessions;
companion pkmsg remains our implementation limitation. No account/live changes.
Evidence: docs/protocol/peer-identity.md#outer-prekey-dispatch-and-companion-admission-2026-10-07.
Final APK regeneration verified 136 topics / 318 methods / 141 files, 1975976
bytes; seven additions and all 129 prior topics unchanged in a 0.072842-second
cached comparison. The 58 xtask tests, CLI reference, five API checks and formatting
passed. Four APK bundles total 3650447 bytes; appendix outside the core bundle.
Repository hygiene checked 850 local links, licenses and all four APK checksum
bundles. No saved state, request identity or live queue changed.

Companion ADV signature verification (2026-10-07): Rust now checks both account
and device signatures over exact details and ciphertext identity, using pinned
primary authority and APK 37.71 device-type-selected E2EE/HOSTED prefixes.
lab-inbox --inspect exposes bounded read-only companion_adv diagnostics, without
key/ADV/ciphertext metadata, query or state adoption. Four real-signature fixture
tests and one PN/LID direct/group read-only/refusal test passed. The saved-account
inspection produced no qualifying companion diagnostic; outbox/actions/receipts/
devices/pins were unchanged. Full workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI tests, CLI reference and final build passed.
APK export: 145 topics / 327 methods / 150 files / 2001638 bytes; nine additions,
all 136 prior topics unchanged in a 0.075721-second cached comparison. Four
bundles total 3676109 bytes; producer appendix outside core. Ordinary Bp2 ADV
producer and admission/freshness/raw-ID/teardown/atomic bootstrap remain open;
companion pkmsg is still unsupported by the runtime selector. No live/account
identity change. See docs/protocol/peer-identity.md#companion-adv-cryptographic-binding-and-inspection-2026-10-07.
Repository hygiene passed 861 local links, licenses and all four APK checksum
bundles. Private original state and unrelated exports remain outside Git.


Ordinary companion ADV producer and pending timestamp (2026-10-07,
**confirmed static APK 37.71**): DEs.CF2 -> Cih.A0V -> Bp2.A0L closes the
ordinary producer gap. Matched-primary/raw-ID/type admission rejects old/equal
candidates when the actual device index is absent/different. SQL read/write
and LbQ constructor mappings prove 0bN.A05 advances expected_timestamp while
preserving the verified list timestamp; treating incoming ADV as a verified
list update was refuted. Atomic companion pkmsg bootstrap, expected-refresh
application, new-device/alias mutation and own/raw-ID/type reconciliation remain
unfinished. No account/network change. [Evidence and unknowns](docs/protocol/peer-identity.md#ordinary-companion-adv-producer-and-baseline-timestamps-2026-10-07).
Export: 155 topics / 337 methods / 160 files / 2,036,151 bytes; cached comparison
0.071644 seconds, ten added topics and 145 prior topics unchanged. Thirteen
full-register sources re-exported exactly. xtask 58 tests, CLI reference,
OpenAPI library-symbol check, formatting and whitespace passed.
Repository hygiene passed 865 local links, package licenses and all four
checksum bundles (3,710,622 bytes total).


Known companion prekey bootstrap (2026-10-07, **implemented/offline**):
group/status version-2 pkmsg now admits an E2EE companion from a current signed
device list with matching primary binding, raw ID, exact device/key index and
non-newer timestamp. Both ADV signatures and nested Signal MAC are required.
Exact companion session/pin, one-time prekey and SKDM/control/message commit
atomically with primary/list/account/own-alias rechecks. Configured pinned versus
apk_authenticated policy remains effective; authenticated changes archive the
exact prior companion state. Own and unknown-own namespace candidates defer.
`lab-inbox --inspect` reports fixed matched-baseline admission diagnostics without
state mutation or values. Five real-crypto tests cover bootstrap, one-time
consumption, reopen dedupe, MAC/policy failures, nine deferred inputs, four
authority changes/rollback and held-skmsg fallback. Direct companion prekeys,
newer-ADV pending refresh/new-device addition and raw-ID/type/own reconciliation
remain unfinished; fresh live companion behavior is unverified.
[Evidence](docs/protocol/peer-identity.md#known-companion-prekey-bootstrap-2026-10-07).
Workspace/all-feature tests passed (687); strict Clippy, warning-free Rustdoc,
formatting and whitespace passed. Further CLI/API/build and saved-state inspection
results are recorded below when complete.


**Observed locally:** final experimental CLI build and saved-account read-only
inspection completed. Captures yielded no qualifying companion ADV/admission
sample, so no live bootstrap is claimed. Before/after outbox, actions, receipts,
device and pin summaries were identical; no login/query/chat was sent. Private
inspection records remain in ignored lab storage. Default CLI tests (24),
regenerated CLI reference, OpenAPI library-symbol check and final build passed.
Repository hygiene passed 869 local links, package licenses and all four
checksum bundles (3,710,622 bytes). No fresh companion sample was found.


Direct companion receive and receipts (2026-10-08, **implemented/offline**):
known-companion direct pkmsg reuses the APK-derived matched-baseline ADV/atomic
checks; held msg selects the actual device ratchet. Events and direct SKDM retain
that device, while chat/dedupe uses its primary PN/LID namespace. Optional PN alias
disappearance dedupes; authenticated aliases stay under the primary LID. Delivery/
read routing supports non-hosted companion destinations and primary LID read chats.
Static Cp2.A00/CmC.A06 supports the traced DeviceJid destination branch; Cp2.A06
was refuted as ordinary producer (it is Bp1/status). Upstream receipt model
producers remain untraced. Tests cover PN/LID bootstrap, ordinary-msg continuation,
one-time consumption, events/receipts, alias/reopen dedupe, direct MAC/policy
failures and delayed-authority actual-device SKDM installation. An own peer
control regression found by the full suite was corrected; receive and backfill
now share alias rules. Workspace/all-feature tests passed (689). Newer-ADV
refresh/new-device application, wider reconciliation, direct-control projection
parity and fresh live companion samples remain unfinished.
[Evidence and unknowns](docs/protocol/peer-identity.md#direct-companion-receive-and-receipts-2026-10-08).


**Validation completion:** all 689 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI tests (24), regenerated CLI reference, API
library-symbol check and final experimental CLI build passed. Four scoped APK
receipt source outputs reproduced exactly. Saved-account offline inspection
found no qualifying companion sample; before/after outbox/actions/receipts/device/
pin summaries were identical. No login, query or chat send occurred. Fresh live
companion interoperability remains unverified; private inspection records stay
in ignored lab storage.
Repository hygiene passed 873 local links, package licenses and all four
checksum bundles (3,710,622 bytes total).


Known companion newer ADV (2026-10-08, static/offline): matching-index direct/group pkmsg now authenticates and atomically records a pending expectation/device sync without advancing verified timestamp. Lower signed responses retain pending work; reaching responses settle it. Inspection reports ready_refresh_pending without mutation. New-device addition, own-account reconciliation and live newer-companion validation remain unfinished. Evidence: [peer identity refresh](docs/protocol/peer-identity.md#known-companion-newer-adv-refresh-2026-10-08).

Validation for newer-ADV refresh: 691 workspace/all-feature tests, strict all-target Clippy, warning-free Rustdoc, default CLI tests (24 including API symbols), regenerated CLI reference and experimental API-symbol check passed. Offline saved-capture inspection found no qualifying ADV/admission sample; before/after outbox, actions, receipts, devices and pins were identical. No login, query or chat send occurred. Private inspection artifact: `lab/releases/companion-refresh-inspection/active.json`.


Companion new-device membership trace (2026-10-08, **confirmed static 37.71**): `11T.A0D` removes already present addresses before add-only application; a newer different index does not itself overwrite existing membership. `0bN.A0G` filters ownership; `1IW.A02/A09` selects gated canonical users; `AzR.A00` rebinds device numbers; `B7j.A03` writes user_device rows separately from the verified LbQ timestamp. Added-member participant/observer calls and caller teardown Boolean are traced only to their boundaries. Runtime gate values, full side effects and Rust new-device application remain incomplete. Full raw methods and reasoning: [new-device application](docs/protocol/peer-identity.md#companion-new-device-membership-application-2026-10-08). No live request or saved account mutation occurred.

New-device trace validation: all nine selected APK methods exported successfully with exception tables; 58 xtask tests, regenerated CLI reference and experimental API-symbol check passed. Runtime Rust behavior and saved account state were not changed.


Companion teardown producer (2026-10-08, **confirmed static 37.71**): ordinary CoU.A04 combines 0bq.A04, the local dead_devices registry and a nonnull parsed offline Integer for its final admission Boolean. Offline metadata is separate from retry count/dhash. 1Uf.A06 persists removal markers; A02 rechecks current membership before cleanup, dropping returned-device markers without key teardown. Removal callbacks, transient A09 set and local worker bounds are distinct. Rust registry/reap and new-device application remain unported; no account mutation occurred. Disassembler b2 multiplication now has its proper label and independent literal regression. [Evidence and limits](docs/protocol/peer-identity.md#companion-teardown-producer-and-registry-2026-10-08).

Teardown trace validation: 692 workspace/all-feature tests, strict all-target Clippy, warning-free Rustdoc, default CLI/API-symbol checks (24), regenerated CLI reference and final experimental CLI build passed. Actual APK re-export renders 0CK.A07 multiplication at 0082 correctly. Four committed checksum bundles have no op=b2 and remain unchanged. No login, live request, key cleanup or account-state mutation was performed.


Remote newer-ADV application (2026-10-08, APK 37.71 static/offline): absent non-offline companions now enter authenticated direct/group receive and are added atomically with provenance/refresh, preserving verified timestamp. Existing different indices are retained until sync. devices exposes adv_added_devices and human expected_ts/count. Signed reconciliation retains newer allocations and clears provenance only on confirmation/removal. Absent offline candidates remain locally deferred pending teardown lifecycle; alias/participant/own reconciliation and fresh live validation remain incomplete. [Implementation and limits](docs/protocol/peer-identity.md#remote-newer-adv-device-application-in-rust-2026-10-08).

Remote device application validation: 692 workspace/all-feature tests, strict all-target Clippy, warning-free Rustdoc, default CLI/API checks (24), regenerated CLI reference and experimental API-symbol/build checks passed. Real PN/LID direct/group fixtures cover known/absent/different-index candidates, MAC refusal, atomic rollback and duplicates after reopen; legacy provenance defaults and signed confirmation are checked. Offline saved-capture inspection found no qualifying companion sample; outbox/actions/receipts/devices/pins matched before and after. No login, query or chat transmission occurred. Private artifact: `lab/releases/device-add-inspection/active.json`.


Pending removal registry (2026-10-08, **static/offline**): signed device reconciliation records disappeared companions and clears authenticated returned addresses in nullable pending_removed_devices. Legacy null stays unknown until signed reconciliation; unsigned responses do not initialize/clear it. Offline absent candidates proceed only with known no-marker state; pending_device_teardown refuses a marked address. Complete-list CAS includes removal state. Inspection exposes unknown/known-empty/count. No cleanup, identity/key deletion, reset, alias fan-out or worker parity is implemented. [Evidence and limits](docs/protocol/peer-identity.md#namespace-local-pending-removal-registry-2026-10-08).

Removal registry validation: 693 workspace/all-feature tests, strict all-target Clippy, warning-free Rustdoc, default CLI/API checks (24), regenerated CLI reference, experimental API-symbol check and final CLI build passed. Offline saved-capture inspection found no qualifying companion sample; before/after outbox, actions, receipts, devices and pins matched. No login, device query, message transmission or key cleanup occurred. Private artifact: `lab/releases/removal-registry-inspection/active.json`.


Nullable offline metadata (2026-10-08, **static/offline**): companion admission
now matches APK 37.71 nullable signed-Integer semantics instead of attribute
presence. First-match malformed/overflow values do not invoke removal guards;
zero/negative valid values do. Existing authority/MAC/atomic checks remain.
Independent host-Java corpus and encrypted PN/LID direct/group fixtures cover
the correction. Android Unicode/gates, cleanup and live companion validation
remain unknown. See [evidence and limits](docs/protocol/peer-identity.md#nullable-offline-companion-metadata-2026-10-08).

Nullable-offline validation: 694 workspace/all-feature tests, strict all-target
Clippy, warning-free Rustdoc, 24 default CLI/API checks, regenerated CLI
reference, experimental API-symbol check, final CLI build and formatting passed.
The repeated checks used system Rust 1.99.0 with isolated LLVM 22 after the host
package change; pinned 1.98.1 proof was not established. Repository hygiene
checked 896 local links. Offline saved-capture inspection found no qualifying
companion sample; outbox, actions, receipts, devices and pins matched before/after.
No login, device query, chat send or cleanup occurred. Private inspection artifact:
`lab/releases/offline-parser-inspection/active.json`.


Companion removal gate (2026-10-08, **confirmed static 37.71**): 0bq.A04
is ABPropBoolean property 35661, with false production/debug baked defaults.
Ordinary receive and worker predicates resolve it through 00D.A13; cached,
MobileConfig and preference branches can supersede the supplied default.
The effective account value remains unknown. Rust's current removal guard is
conservative local policy, not an unconditional WhatsApp requirement. No runtime
state, identity, keys, markers or waits changed. Nine bounded source topics are
added to the cached core release baseline; automatic relocation is unestablished.
[Evidence and limits](docs/protocol/peer-identity.md#companion-removal-property-and-effective-value-2026-10-08).

Feature-gate trace validation: thirteen complete selected method/class exports
(with exception tables), one field-reference scan and the current APK SHA-256
were checked. Actual core export: 164 topics / 349 methods / 169 files /
2,068,819 bytes, within the 2 MiB bound. Checksum-verified cached comparison:
nine additions, 155 unchanged, 0.0942 seconds. All 59 xtask tests, regenerated
CLI reference, experimental API-symbol check and formatting passed. Runtime
Rust behavior is unchanged; no account access, login, query, send or cleanup
occurred. Pinned-toolchain proof is not claimed; checks used system Rust 1.99.0.
Repository hygiene passed: 902 local links and 3,743,290 bytes across four
checksum bundles. Reproduction commands quote DEX descriptors/constructor names
for shell use. No runtime code or private state changed.


AB-property receive read (2026-10-08, static/offline): new abprops-query queues
one initial APK 37.71 abt/props read; --show inspects saved correlated values
without applying feature configuration. Boolean 35661 is registered in the
concrete APK table and parsed as signed Integer != 0. Full/delta/hashless and
last-duplicate semantics are distinguished. Empty text attributes now survive
encoding, preserving the initial empty hash. Effective MobileConfig precedence,
configuration application/scheduling remain unfinished; live read outcome is unknown.
No identity/key rotation, marker deletion or reservation replay is added.
[Evidence, diagnostics and limits](docs/protocol/feature-configuration.md).

AB-property validation (2026-10-08, **observed live**): the existing account
logged in and became active, then received a terminal stream:error (no outer
code; ack child) before the bounded window finished. One read is reserved/sent
with no correlated response: outcome_unknown, no returned/effective property
value. Its original ID is retained and repeats do not transmit. Pre/post
selected peer pins, devices, receipts and outbox matched; prior actions were
preserved. No chat send or identity rotation occurred. The terminal cause and
configuration application remain unresolved. All 699 workspace/all-feature
tests, strict Clippy, Rustdoc, default CLI checks, experimental API-symbol and
CLI-reference checks, experimental build and formatting passed on system Rust
1.99.0. See [feature configuration](docs/protocol/feature-configuration.md) for evidence and inspection limits.

Repository hygiene passed: 912 local links, package licenses, tracked-file
budgets and 3,840,464 bytes across five checksum-verified APK bundles.

Terminal ACK diagnostics (2026-10-08, **observed offline / attribution unknown**):
lab-inbox --inspect now prints fixed ACK class/type and ID/to presence flags,
redacting all echoed identifiers/unknown values and diagnosing duplicate ACKs.
Saved captures are content-addressed without timestamps/epochs, and inspection
covers all history. The stored notification-ACK stream:error cannot therefore
be attributed to the latest receive window from this inspection alone; prior
wording was too strong. No causal query/ban/retry interpretation is established.
No login, query replay, key change or capture mutation occurred in this chunk.
See [terminal evidence and scope](docs/protocol/terminal-errors.md).

Rebuilt offline inspection identifies the saved ACK type as w:gp2; no network
session was opened. Its capture origin and relation to recent closure remain
unknown. All 47 experimental CLI checks, strict CLI/all-target Clippy, regenerated
reference, experimental build and formatting passed on system Rust 1.99.0.

Terminal provenance (2026-10-08 local, **implemented/offline**): comparison login
and receive now atomically preserve terminal captures with first/last local
observation milliseconds, actor-local epoch, observation count and latest
terminal marker. Offline inspection exposes these fixed metadata fields; old
captures remain unknown and offline replay cannot backfill them. Epochs can
repeat and wall clocks can reverse; latest follows commit order. Atomic rollback,
reopen, repeated observation and legacy non-backfill are tested. No request IDs
or identities are changed. [Evidence and limits](docs/protocol/terminal-errors.md).

Terminal-provenance validation: all 702 workspace/all-feature checks, strict
Clippy, warning-free Rustdoc, default CLI checks, regenerated reference,
experimental build and formatting passed on system Rust 1.99.0. The session-loop
terminal fixture additionally confirms metadata persistence. One live receive-only
32-second window authenticated, stayed active and stopped cleanly (exit 0).
No terminal stanza was observed; the older terminal capture remains unknown.
Actions/devices/pins/receipts/outbox matched preflight, with zero new actions.
The original AB-property read remains outcome_unknown with its original ID;
no query was queued or replayed. Fresh incoming-message decryption and long-term
reliability are not established by this window.

ACK writer closure (2026-10-08, **confirmed static APK 37.71**): Android arg1
selectors 76/129 converge through Cp2.A05 and 1Rd.A04; Ck7 flag 1 serializes
the supplied node with raw leading zero. No destination/type rewrite appears
at the traced boundary; no APK-backed runtime correction was found. Historical
terminal ACK origin remains unknown. The expanded group bundle preserves 39
topics / 43 methods / 44 files / 928771 bytes with exception tables. Cached
same-APK comparison against 80f5636 took 0.039709 seconds: five additions,
34 annotation-changed files; all old instruction texts remain equal. Native
framing, metadata side effects and complete notification application remain
unfinished. No account access, live request, query replay or runtime change.
[Evidence and comparison](docs/protocol/groups.md#ack-dispatch-and-uncompressed-write-boundary-2026-10-08).

ACK evidence verification: all 59 xtask tests, actual bounded export, cached
checksum diff and old instruction equivalence, regenerated CLI reference,
experimental API-symbol check and formatting passed on system Rust 1.99.0.
No runtime Rust change, saved-account access or live request occurred.

Repository hygiene passed: 928 local links, tracked-file budgets, package
licenses and 3,996,167 bytes across five checksum-verified APK bundles.

Companion removal scope (2026-10-08, **confirmed static APK 37.71**): A0s/A0r
are session and identity removal, with device-specific recipient/type/device
SQL predicates. Feature-gated address mapping can select namespace counterparts;
CDO property 31194 has prod false/debug true defaults but unknown effective
state. The session callback also invalidates cache variants for that device.
Sender-key SQL is sender/type/device plus a selected group list. Transaction
callback delivery, full mapping/observer effects, group selection and effective
gates remain incomplete; no destructive Rust cleanup is enabled. A separate
17-topic / 17-method / 22-file / 181102-byte checksum bundle retains the complete
selected chain with exception tables. All 59 xtask tests passed. No saved account
was opened, no live request sent, and no keys or history were deleted.
[Scope and port boundaries](docs/protocol/peer-identity.md#companion-key-removal-scope-and-callback-boundaries-2026-10-08).

Cleanup evidence checks: 17 direct dumps match bundle instruction-offset/
mnemonic coverage and ordered catch ranges; direct operand details are retained
separately. Regenerated CLI reference, experimental API-symbol check and formatting
also passed on system Rust 1.99.0. Runtime Rust code is unchanged.

Repository hygiene passed: 935 local links, tracked-file budgets, package
licenses and 4,177,269 bytes across six checksum-verified APK bundles.


Cleanup callback closure and register-aware exports (2026-10-08): **confirmed
static APK 2.26.37.71** Cpf discriminator 1 uses Bb1 -> 8kU.A1P after successful
1gB.close/0IY.endTransaction through 0Gy.A02; rollback/discard uses Bax and does
not execute that selected callback. Immediate SQLite onCommit is not its run
point. Callback ordering, complete nested/concurrent behavior, effective gates
and full address/group cleanup remain unimplemented; destructive cleanup stays
disabled. **Implemented offline** recipe include_register_flow (boolean, default
false) retains supported register operands missing from legacy compact exports.
A literal CDO move-result destination regression verifies a register-only change
is visible with the option; default rendering and primary annotations are stable.
The cleanup bundle now has 23 topics/38 methods/28 files/214535 bytes. Cached
same-APK diff against 257f186 took 0.011257 seconds: six additions, sixteen
annotation-changed and one unchanged topic. All old offset/opcode/mnemonic and
catch ranges match; all six new direct dumps match those coverage lists.
No account was opened, live request sent, identity changed or keys deleted.
[Callback evidence and exporter limits](docs/protocol/peer-identity.md#cleanup-callback-commit-and-rollback-closure-2026-10-08).

Validation on system Rust 1.99.0: all 60 xtask tests, strict xtask/all-target
Clippy, xtask build, regenerated CLI reference, experimental OpenAPI library-symbol
check and formatting passed. Repository hygiene verified 941 local links, package
licenses, tracked-file budgets and 4210702 bytes across six checksum bundles.
Runtime Rust code is unchanged; workspace/runtime tests were not repeated for
this exporter-only Rust change.
Exact hash-pinned APK re-export with --check verified all 28 files/214535 bytes.


Sender-key selection and deletion observer closure (2026-10-08, **confirmed
static APK 2.26.37.71**): 0kX.A02 selects DISTINCT stored group IDs for the
translated sender/type/device; 1Uf.A02 slices eight groups per lock/delete block
and rechecks for at most three local passes. Its inner list helper receives
972/972 but operates on those at-most-eight groups; these are not server limits.
Argument concatenation puts the tuple collection before the group IDs. D3f
selector 41/0x29 is BqD identity-deleted, including notifyDeviceIdentityDeleted
and shared D4x effects; Fco deletion is a no-op for identity waits. The earlier
first-identity interpretation of 0k4.A0L is refuted and corrected: 0jc.A0K's
nonnull-key/no-old-value branch uses selector 42/BqB, while null-key/old-value
uses A0L/BqD. Complete effective gates, canonical mapping, cache/executor
semantics, concurrent scheduling and teardown runtime remain unimplemented.
Current cleanup bundle: 41 topics/56 selected declarations (one abstract),
46 files/395788 bytes. Same-APK cached diff against 72afc9d took 0.013700 seconds:
eighteen additions/twenty-three unchanged topics, all old topic texts equal.
All new direct declaration/instruction/catch coverage lists match. No account
was opened, live request sent, identity changed or keys/history deleted.
[Evidence and remaining port requirements](docs/protocol/peer-identity.md#sender-key-group-selection-and-deleted-identity-observers-2026-10-08).

Validation: actual hash-pinned bounded export, checksum-verified cached diff,
unchanged old topic bytes and all new direct declaration/instruction/catch
coverage checks passed. Regenerated CLI reference, experimental OpenAPI
library-symbol check, formatting and staged whitespace passed on system Rust
1.99.0. Hygiene verified 948 local links, licenses, tracked-file budgets and
4391955 bytes across six checksum bundles. No Rust runtime/tool source changed;
workspace tests/Clippy were not repeated for this source-only research chunk.


Canonical mapping row selector (2026-10-08): **confirmed static APK 2.26.37.71**
0Xx queries rank by descending sort/LID IDs, then 0eU's reverse TreeMap overwrites
equal sorts; the winner is greatest signed sort/smallest tied account row.
**Implemented/offline** default-library jid_map::select_account_row ports that
selection, preserving invalid winning rows before downstream checks. Independent
SQLite fixtures cover 36 synthetic cases, including all 24 permutations of a
tie/cross-phone set, signed extremes and invalid winners. New redacted
jid-map-row-inspect --input reports counts/presence/nonnegative/positive checks;
private values are omitted, known missing fields are diagnosed and malformed/
unknown/duplicate/type/budget errors refuse without partial stdout. Mapping
write authority, effective gates, freshness, complete reverse/bulk materialization,
worker scheduling and cleanup runtime remain unported. The extended cleanup
bundle has 57 topics/76 declarations (one abstract)/62 files/450854 bytes. Cached
same-APK comparison against eafb8fd took 0.028460 seconds: sixteen additions,
forty-one unchanged topic texts. Direct method/instruction/catch coverage matches.
No account was opened, live request sent, identity changed or keys/history deleted.
[Source, port and remaining mapping gaps](docs/protocol/peer-identity.md#canonical-key-mapping-and-row-selection-in-rust-2026-10-08).

Validation on system Rust 1.99.0: all 708 workspace/all-feature tests, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, 24 default CLI
checks, regenerated CLI reference, experimental OpenAPI library-symbol check,
xtask build and formatting passed. The independent SQLite fixture --check passed.
The compiled inspector selected an invalid synthetic winner without falling back;
an unknown-field invocation exited nonzero with no partial stdout/private-key
echo. Repository hygiene verified 957 local links, package licenses, tracked-file
budgets and 4447021 bytes across six checksum bundles. No account or live-server
access was involved; fresh runtime mapping/cleanup validation remains pending.


Contact LID ingestion (2026-10-08, **confirmed static APK 37.71**, offline-tested):
corrected per-user `lid@jid` to `lid@val`, gated by successful `usync/result/lid`
metadata and strict plain-LID validation. Wrong subtypes/zero device suffixes and
failed/missing protocol metadata cannot supply a LID. Existing CLI status exposes
the result. Historical ingested rows, identities and reservations remain intact.
Fresh live replies, refresh/backoff and full contact-type state parity remain
unverified. See [contact response evidence](docs/protocol/chat-actions.md#contact-lid-response-correction-2026-10-08).


Contact type/status inspection (2026-10-08, **confirmed static APK 37.71**):
newly ingested existence follows exact `in`, rather than supplied JID presence;
`out`/`invalid` are preserved in nullable `contact_type`. Missing/unsupported type
has a fixed local diagnostic. Historical rows keep their original boolean and
unknown type. `contacts status` inspects offline without queuing another request.
Fresh live types and full address-book state handling remain unverified.
[Evidence](docs/protocol/chat-actions.md#contact-type-and-offline-status-2026-10-08).


Contact receive projection/live check (2026-10-08): a 20-second receive-only
session authenticated cleanly and settled one original existing-peer contact
query as `in`. Contact replies now project automatically during receive and
recover at startup after a settlement/projection crash gap. Live evidence exposed
a local rejection of zero-component AD_JID LIDs; the parser now distinguishes
wire encoding from logical account type. A guarded offline `contacts repair-lid`
recovered the LID from that same consumed reply, retaining prior status/history;
repeat was unchanged. The 19 earlier actions, 9 acknowledged receipts and empty
outbox stayed unchanged. No query replay or chat send. Fresh post-fix replies,
native wire factory and full contact sync remain unverified.
[Evidence and negative results](docs/protocol/chat-actions.md#live-contact-projection-and-primary-wire-lid-2026-10-08).


Primary identity-read correlation (2026-10-08, offline): identity-only replies
and per-user directory refusals now compare actual address components across
Plain/AD_JID-zero encodings. Foreign namespaces/users, agents and companions stay
distinct; duplicate identity-only rows and unsupported kinds remain refused.
Successful signed-bundle matching was already component-based. Existing saved PN
inspection was already received; LID identity-only evidence was absent, so this
is fixture-demonstrated compatibility, not a new live identity milestone.
[Evidence and audit boundaries](docs/protocol/peer-identity.md#primary-wire-address-correlation-2026-10-08).

Directory refusal attribution (2026-10-08, offline): `peer-keys --show` now
reports fixed local invalid-reply diagnoses and binds numeric refusals to one
unambiguous selected-user/root context. Foreign first-error codes and duplicate
rows cannot masquerade as selected-peer refusals. Original action history and
retry reservations remain intact. APK 37.71 DF6 identity-only error handling
supports contextual attribution; duplicate guards are local policy and fresh
live refusal validation remains unverified.
[Evidence and limits](docs/protocol/peer-identity.md#directory-refusal-attribution-2026-10-08).

Identity-success wait consumer (2026-10-09, **confirmed static APK 37.71**):
the sole DEX Fco constructor caller is a feature-gated extensions/Flows path.
Its 30-second wait wakes callbacks after success or Exception timeout; those
callbacks reverify held keys. This does not establish incoming replay/session
repair. Eight new reproducible identity-wait topics preserve the caller, observer,
timed worker and verification chain; all other event consumers and complete
identity-only application remain unfinished. No account or network changed.
[Evidence and scope](docs/protocol/cross-device-sync.md#identity-wait-caller-and-extensions-verification-2026-10-09).

Identity-success consumers (2026-10-09, **confirmed static APK 37.71**): the
full DEX scan preserves 22 BdO/BdR declarations/bodies. Only Fco and AL2 have
nonempty BdO; the second waiter is a 30-second business-profile UI task.
No scanned callback directly decrypts or retries messages; transitive helpers
remain unknown. Primary LID receive's sender_pn prerequisite is a local guard,
not a proven APK requirement, and remains the next receive-routing trace.
No account or network changed. [Evidence and limits](docs/protocol/cross-device-sync.md#identity-success-consumer-inventory-2026-10-09).

Primary LID routing trace (2026-10-09, **confirmed static APK 37.71**): missing
sender_pn/username logs join nullable metadata construction rather than throwing.
The parsed username attribute is username; DFc's 13 field bindings are recovered.
Ordinary LID's type 18 selects the message's A05 route for sender-device lookup,
without consulting alias metadata. Rust's unconditional primary alias guard is
a local limitation; historical PN-marker stability and the raw stanza producer
remain next boundaries before a receive change. Six unrelated/special builder
write paths are recorded to avoid repeating them. No account or network changed.
[Evidence and limits](docs/protocol/peer-identity.md#primary-lid-route-and-nullable-aliases-2026-10-09).

Primary LID receive (2026-10-09, implemented/offline): new primary messages use
actual LID without requiring a phone alias. Original authenticated capture proofs
retain historical PN rows for duplicate/event/read lookup; no row rename, ratchet
replay or alias adoption from redelivery. Real-crypto msg/pkmsg, reopen/MAC failure,
legacy capture ambiguity and poll-target fixtures cover the correction. Fresh
live alias-less interoperability and full APK storage parity remain unverified.
[Evidence and scope](docs/protocol/peer-identity.md#primary-lid-receive-and-legacy-record-stability-2026-10-09).

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
[Evidence, negative findings and limits](docs/protocol/reactions.md).
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
[Sources, negative findings and fixtures](docs/protocol/reactions.md#outgoing-intent-attribution-and-quote-scope-2026-10-09).
Validation: 730 workspace/all-feature tests, strict Clippy, warning-free Rustdoc,
24 default CLI checks and regenerated CLI/API reference checks passed. Offline
reprojection upgraded zero events and kept retained summaries unchanged. A
45-second receive-only live smoke check authenticated and stopped cleanly, with
15 decrypted records remaining 15; no fresh reaction/DM established new target
interoperability. Original uncertain actions, outbox and receipts were preserved.

### Poll vote structural guard (2026-10-09)

APK 2.26.37.71 `Ban.A01` calls the same `C7G.A00` parent-key validator as
reactions; `Clc.A02` independently confirms the shared `BoQ.A00` mapper on
the poll path. The new seven-topic/seven-method checksummed source bundle
contains 12 files/69,090 bytes and reproduces with the exact APK/recipe.
Poll decoding now checks supported ID/remoteJid, timestamp presence and
ciphertext/IV presence with a twelve-byte IV. `lab-inbox --inspect` explains
refused saved updates with fixed redacted validation labels. Zero/negative
present timestamps, absent fromMe/metadata and present empty ciphertext are
not incorrectly rejected structurally; AEAD validation remains separate.
Native JID parity, feature selection, expiry/edit/quiz behavior and outgoing
poll chat binding remain unfinished. No new poll/vote was transmitted.
[Source, implementation and remaining chat issue](docs/protocol/poll-votes.md).

Validation: 732 workspace/all-feature tests, strict Clippy/Rustdoc, CLI/API
checks and hygiene passed. Recovered-profile offline projection upgraded zero
events; peer/action/receipt/outbox/event summaries and normalized capture
diagnostics were unchanged. That profile has no saved poll updates, so no
fresh or retained-poll interoperability is claimed.

### Own-poll chat binding (2026-10-09)

Own poll vote projection now binds the creation's existing saved chat to the
actual primary voter through exact PN/LID components or an existing authenticated
association. AEAD projection retains the original chat and actual actor;
historical markers remain unchanged. Missing legacy binding stays unknown,
unattributed/companion references stay opaque, and corrupt saved state refuses
locally. Vote preparation checks the actual saved chat before identity/entropy,
with fixed missing/foreign/invalid diagnoses. `lab-inbox --inspect` exposes a
read-only stored-selector binding label without printing identities or secrets.
The earlier structural guard remains effective. Native parent-store/feature/
expiry/edit/quiz/companion/group parity and fresh live poll attribution remain
incomplete. [Evidence and boundaries](docs/protocol/poll-votes.md#saved-chat-attribution-2026-10-09).

Validation: 734 workspace/all-feature tests, strict Clippy/Rustdoc, 24 default
CLI tests, CLI/API reference checks, formatting and hygiene passed. Offline
projection upgraded zero events. A 45-second receive-only live window was
active/authenticated and stopped cleanly; peer keys, action outcomes, receipts,
empty outbox and events stayed unchanged. Fifteen decrypted payloads remained
fifteen; no fresh DM/poll/vote proved new live attribution. Two uncertain sent
actions were preserved without replay; reconnect and chat sending were disabled.

### Identity-change receipt/version/token scope (2026-10-09)

Static APK 2.26.37.71 D4x receipt selection now has attributable helper coverage:
primary message-ID intersection, scoped null-timestamp companion/device/add-on
rows, coexistence/status paths, persistent primary_device_version increments
and conditional refresh of an existing unexpired privacy token. The reviewed
bundle preserves 23 topics/53 declarations, 28 files/323,094 bytes, with exact
register operands and catch metadata. Runtime mapping/migration, cache callback
closure and refresh-job lifecycle remain incomplete. Existing Rust incoming
token storage, authenticated identity archival and sender-key exposure policy
are unchanged; native receipt caches are not receipt-outbox ACK reservations.
No private state or server request was changed. [Full source and next port
boundaries](docs/protocol/identity-receipts.md).

Validation: exact APK/recipe re-export, CLI/API checks and hygiene passed. Cached
23-topic self-comparison took 0.026378 seconds; no cross-release equivalence or
new runtime/live behavior is claimed. Incoming PN/LID delivery-fact attribution
remains a separately identified receive gap to trace and correct next.

Direct receipt attribution (2026-10-09, **confirmed static / local correction**):
APK 37.71 `CmG`/`Cmk`/`DDH` keep message chat and receipt actor separate;
stale `ClU` attribution is corrected. Primary normal-outbox receipts now bind
the unique saved wire intent to the actual actor through exact PN/LID or an
existing authenticated association in either direction, within one transaction.
Actual actor, original IDs, progress, uncertainty and receipt-send reservations
are preserved. Legacy PN records page history; ambiguous/corrupt matches roll
back. Native canonicalizer gates, identity cleanup and fresh live PN/LID
receipt attribution remain unverified. [Evidence and boundaries](docs/protocol/direct-receipts.md).

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
No account identity, map or send state is reset. [Evidence and limits](docs/protocol/receipt-canonicalizer.md).

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
[Evidence and boundaries](docs/protocol/group-receipts.md).

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
[Evidence and port boundaries](docs/protocol/group-fanout.md).

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
[Evidence and boundaries](docs/protocol/device-key-batches.md).

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
[Evidence and limits](docs/protocol/group-fanout.md#routing-snapshot-correction-2026-10-09).

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
[Evidence, invocation and boundaries](docs/protocol/normal-group-outbox.md).

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
same-ID group collision behavior remain unknown. See [group inbox](docs/protocol/group-inbox.md).

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
reactions and native own/hosted normalization remain unknown. See [reactions](docs/protocol/reactions.md#incoming-group-original-author-2026-10-09).

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
or historical-marker rewrite is introduced. See [group mapper](docs/protocol/group-reaction-mapper.md).

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
reaction behavior remain unverified/unported. See [envelope direction](docs/protocol/group-envelope-own.md).

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
See [group keep](docs/protocol/group-keep.md).

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
[Evidence and implementation boundaries](docs/protocol/group-keep-send-key.md).

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
[Evidence and limits](docs/protocol/group-keep-outbox.md).

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
fresh live keep reception remain unverified. [Evidence and limits](docs/protocol/keep-timestamps.md).

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
untouched. [Evidence and port boundary](docs/protocol/identity-token-refresh.md).

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
[Evidence, invocation and boundaries](docs/protocol/token-issue.md).

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
[Evidence and reproduction](docs/protocol/own-verification.md).

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
[Evidence and limits](docs/protocol/token-import.md).

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
[Evidence and limits](docs/protocol/primary-device-version.md).

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
no saved reservation or account was changed. [Evidence and next input](docs/protocol/receipt-version.md).

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
[Evidence and limits](docs/protocol/send-version-snapshot.md).

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
operation or retry-guard relaxation was added. [Evidence and next port inputs](docs/protocol/sync-resend.md).

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
[Evidence and limits](docs/protocol/send-version-snapshot.md).

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
[Writer boundary and next inputs](docs/protocol/receipt-apply.md).

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
[Evidence and reproducible recipe](docs/protocol/receipt-factory.md).

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
[Evidence and comparison recipe](docs/protocol/receipt-clock.md); no runtime/account operation changed.

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
runtime remain unported. [Evidence and recipe](docs/protocol/receipt-cache.md).

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
[Evidence and next inputs](docs/protocol/receipt-load.md); no account or live operation changed.

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
[Evidence and reproducible recipe](docs/protocol/receipt-user.md); mutable Rust receipt runtime remains
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
[Evidence and next port inputs](docs/protocol/receipt-admission.md); no runtime/live changes in this chunk.

Receipt-admission validation: 13 source topics / 18 files / 121,518 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified 13 unchanged
topics and four metadata artifacts; raw 96-byte/22-entry status-order evidence
independently matches classes2.dex at its recorded offset. Repository hygiene
checked 1,393 local links and 33 APK bundles / 8,507,021 bytes; diff hygiene
passed. Research/docs only; no fresh runtime tests or live account operation
is claimed.

2026-10-09 — Original-device receipt rows (offline): new original reservations
now atomically seed pending rows with exact frozen targets and nullable versions.
Authenticated attributed supported root receipts retain earliest positive server
milliseconds, including earlier duplicate facts, without rewriting original sends.
`outbox-versions` exposes redacted rows; historical missing rows stay null.
Native cleanup, PN/LID row canonicalization, per-user admission and resend remain
unfinished. No fresh send/live capture is claimed.
[Evidence and limits](docs/protocol/device-receipt-rows.md).

Receipt-row validation: 802 workspace/all-feature tests, strict Clippy,
warning-denying Rustdoc, formatting, default CLI, API symbols, CLI reference,
experimental CLI build and hygiene passed. Retained-state inspection was
unchanged, with zero network commands and no retained group preparation sample.
A repeated parallel run exposed the previously recorded immediate-drop store
ownership refusal; owner teardown now explicitly releases its own lock after
closing the database. A shared-descriptor regression still rejects active second
owners and permits reopening after drop. No external store unlock/retry was added.
[Mechanism and attribution limits](docs/design/verification.md#temporary-store-ownership-refusal-observed-2026-10-09).

2026-10-09 — **Confirmed static:** receipt-manager A03 is populated from the
revoke receipt-deletion path after successful transaction close. The native
user/device DELETEs select the original supplied message row with no timestamp
predicate; manager exclusions occur after the earlier device-only pool. The
message A0l deletion/copy contexts are traced without inventing its persistence
or other writers. Signed negative 22b/22s arithmetic now renders explicitly,
including the native transaction -1 nesting adjustment. Rust revoke, per-user
admission and identity cleanup remain unfinished. No account/live operation.
[Evidence and reproducible recipe](docs/protocol/receipt-manager.md).

Receipt-manager validation: all 67 xtask tests, strict xtask Clippy, rebuilt
CLI, CLI reference/API symbols, formatting and hygiene passed. Eight topics /
13 files / 301,312 bytes reproduce exactly; cached self-diff verifies eight
unchanged topics plus four metadata artifacts. Raw signed decrement bytes match
the recorded APK offset. Hygiene: 1,416 links, 34 bundles / 8,808,333 bytes.
Research/tooling only; no new shared-runtime or live account test is claimed.

2026-10-09 — Stored own-LID guard correction: both remote primary-change hooks
now recognize authenticated login identity before a group-preparation checkpoint
exists. One shared read-only lookup refuses malformed/conflicting saved authority
without repair or echo. New `group own-lid` prints only lid_known. APK BO0/BO1
proxy/direct bodies and stored PN/LID producers are retained; BO0 equality is
own stored user namespaces, not a remote directory alias. Native own-primary
versions, full own-device state and mutable identity cleanup remain unfinished.
[Evidence and limits](docs/protocol/own-identity.md).

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
[Worker evidence and boundaries](docs/protocol/receipt-callbacks.md).

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
[APK evidence, diagnostics and boundary](docs/protocol/companion-receipt-cleanup.md).

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
[Evidence and remaining port inputs](docs/protocol/user-receipt-import.md).

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
[Complete methods and reasoning](docs/protocol/user-receipt-reference-roles.md).

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
[Consumer evidence and limits](docs/protocol/user-receipt-reference-roles.md#descriptor-consumers-follow-up).
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
[Source, branch arithmetic and remaining prerequisites](docs/protocol/receipt-ownership.md).
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
[Exact methods, provenance and remaining initialization inputs](docs/protocol/message-status.md).
Ordinary outgoing initialization, full admission and primary cleanup remain
unfinished. No Rust runtime, account or network behavior changed.

Message-status validation: eleven topics / 16 files / 79,539 bytes reproduce
exactly from APK 37.71. Cached self-diff verifies eleven unchanged topics and
four metadata artifacts, not cross-release semantic parity. CLI-reference
regeneration, OpenAPI/library-symbol checks, formatting, whitespace and hygiene
passed. Hygiene checked 1,499 local links and 40 bundles / 9,411,474 bytes.
Caller/writer inventories and the no-matching ordinary override body result
remain separately attributed. No fresh workspace/runtime or live test is claimed.

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
exports](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#count-notification-ack-token-and-queue-stages-2026-11-04).

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
methods, boundaries and 24 new checked exports](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#pre-ack-conversion-sqlite-persistence-and-retirement-2026-11-04).

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
purpose). [Request, timer methods and regression limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
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
exports and oracle](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#pre-ack-reload-capacity-and-connection-generation-ownership-2026-11-04).

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
of every count notification's ACK list. [Methods,19 new exports and limits](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#ack-transaction-completion-and-rollback-bridge-2026-11-04).

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
successful short reads. [Methods,17 new exports and boundaries](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#ack-projection-serialization-merge-order-and-refill-2026-11-04).

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
ID-specific Handler message. [Retained methods](docs/protocol/apk/2.26.43.72-prekey-upload-replies/README.md#confirmation-ping-request-and-independent-timeout-2026-11-04).
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
