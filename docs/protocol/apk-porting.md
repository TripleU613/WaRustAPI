# APK-derived Rust client and release maintenance

Owner steering, 2026-10-07: prioritize comprehensive APK reverse engineering into
Rust and a quick, file-based comparison/patch workflow for new releases. The
normal client remains the result; repeated isolated live probes are not the main
work queue. Existing saved identities, keys, requests and server waits remain
preserved. APK-derived compatibility does not establish immunity from account
restrictions, and does not authorize avoiding verification, challenges or limits.

## Release workflow

1. Keep the source APK local. Pin its SHA-256 and Android version in the generated
   manifest; never apply another release's obfuscated selectors blindly.
2. Re-anchor client behavior by strings, descriptors and callers. Put the reviewed
   topic/class/method mapping in a release recipe, with the Rust files it affects.
3. Export reproducible media-free files under stable topic filenames. Keep older
   files intact. Compare cached exports quickly without rescanning the APK.
4. Read changed instructions and their callers/conditions. Separate class-renaming
   and code-layout churn from behavior changes; the diff does not make this
   interpretation automatically.
5. Patch the linked Rust modules, evidence-backed fixtures and API/CLI contract in
   the same change. Preserve unsupported/unknown branches explicitly.
6. Verify offline equivalence, persistence and refusal handling first; perform
   bounded authorized live validation when it adds evidence. Do not repeat
   acknowledged/uncertain sends or manufacture telemetry to obtain a response.

The first versioned recipe is
[2.26.37.71-groups.json](apk/recipes/2.26.37.71-groups.json), covering group
notification/ACK, refresh job and subject freshness helpers. This establishes
release-aware extraction and comparison; it does **not** cover the whole APK.
The existing [37.70 snapshot](apk/README.md) remains intact.

```sh
cargo run --locked -p xtask -- apk-research-bundle /path/to/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-groups.json \
  --output lab/releases/2.26.37.71-groups

# After re-anchoring/exporting the next release with the same topic names:
target/debug/xtask apk-research-diff \
  lab/releases/2.26.37.71-groups lab/releases/next-groups
```

The JSON comparison gives added/removed/changed/unchanged topics, both evidence
file paths, both class descriptors, and affected Rust paths. Non-topic metadata
and dictionary changes appear in the separate artifacts list. Use `git diff --no-index`
on a selected old/new file to read its actual instruction changes. The tool
ignores only the provenance hash line; it does not erase offsets or names to
make incompatible code look identical. Cached comparisons validate file hashes.
Fresh extraction still scans APK code and takes longer than cached comparison.
The first real 37.71 group export took 16.768 seconds; three cached self-comparisons
took 0.0183..0.0215 seconds. Cross-release behavior differences were not proven
by that self-comparison; synthetic fixtures verify changed/added/removed reporting.

## Work queue and completion standard

Expand release recipes and interpreted Rust mappings across onboarding/request
construction, authentication/Noise/certificates, binary dictionaries and framing,
protobuf schemas, Signal identity/session/device behavior, direct/group messaging,
media, receipts, notifications, app-state/history/cross-device synchronization,
and relevant native algorithms. Existing evidence and implementations are reused;
new findings require explicit confirmed/inferred/observed/unknown labels.

An area is complete only when outbound construction, defaults/conditions,
response handling, state transitions and applicable native dependencies are
traced and reproduced in Rust with meaningful fixtures. A disassembly file,
green unit test or server ACK alone does not establish full equivalence.

The group recipe now has eleven topics: the new contact subject writer and
ordered JID-map producer/storage helpers extend the eight-topic baseline.
[Storage and transaction interpretation](groups.md#subject-contact-storage-and-alias-transaction-closure-2026-10-07)
records display-name persistence, cache invalidation, successful insert filtering,
rollback cleanup and full-signature constructor resolution. All eleven topics
remap uniquely on 37.71. Cached comparison shows three added topics and eight
unchanged code topics; the 16-file export is 423,242 bytes. Both APK hashes are
identical, so this establishes source coverage rather than release equivalence.
Subject notifications and downstream alias listener behavior remain unfinished
in Rust; the ownership paths identify where further implementation belongs.

Still required for broad release maintenance: comprehensive coverage accounting,
broader anchor coverage and caller-assisted disambiguation, normalized
method-level comparison where sound, protobuf/token/config extraction recipes,
and native-code change attribution. UI/media payload extraction is unnecessary
for protocol reproduction. Bulk analysis trees and native binaries stay local;
compact attributable evidence and interpreted code stay in Git.

## Whole-APK continuation queue (2026-11-04)

The existing `xtask apk-file-index` also scanned all **15,728 archive members**,
including **13 DEX entries** and **12 native libraries** across arm64-v8a,
armeabi-v7a, x86 and x86_64. Its full hashed file inventory stays local outside
the checkout. These are inventory counts, not semantic/port coverage.

The owner's renewed objective is the whole APK's behavior represented through
this existing Rust system. The earlier v0.1 boundaries and parked features remain
historical milestones, not a reason to drop calls, channels, backups or other
unported systems from the end-state queue. Continue the current actors, store,
CLI, recipes, evidence and tests; do not create a second client or replace working
ports with generated approximations.

**Observed static inventory:** REA inspected `com.whatsapp` **2.26.43.72**,
version code **264307238**, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Its JADX metadata contains **106,897 classes** and **15,728 resources**
([retained package summary](rea/2.26.43.72-registration-recovery/package-summary.json), Evidence `ev_6fb00cc767d6b36a2539c5ba9319de05fa4c7cc95d3bb6c034a118c4a5468581`,
parent `ev_59f5d29110e9a43c336fef62c574a9dd0e0e60d7a58c05caee041460e5cf06b3`).
These are inventory counts, not classes understood, methods reproduced, or a
conversion percentage. Manifest/resource decoding does not trace their consumers.
Native methods, reflection, callbacks, configured branches and external services
remain separate coverage obligations.

The **2.26.36.75 default profile**, **2.26.37.70 comparison recipe**, extensive
**2.26.37.71 method evidence**, and **2.26.43.72** input remain separate identities.
Do not relabel old evidence as current merely because a string or class name
matches. Existing [coverage accounting](apk-coverage.md),
[release comparison](apk-2.26.43.72-comparison.md),
[handoff review](project-handoff.md) and the dated [status](../../STATUS.md)
are inputs to this queue. Some older feature-matrix blockers are superseded by
November changes; the linked source/evidence must be checked before restarting
an allegedly missing port.

### Resume record and per-boundary gates

Each row below is an enduring task ID, not a claim that a subsystem is one atomic
patch. For each next boundary, retain: **task ID; exact APK/hash; entry point and
caller/producer; REA Evidence IDs and/or reproducible recipe; affected Rust files;
traced branches and explicit unknowns; fixtures; CLI/API exposure; last test;
next unresolved question**. Keep the latest record in its linked protocol document
and update this queue when a boundary closes. Raw APKs and bulk analysis stay local.

Use five distinct states: **inventoried**, **traced**, **ported**, **offline-verified**,
and **live-observed**. A row can contain different states for different branches;
none promotes the entire subsystem. A prior live ACK does not establish rendering,
playback, restart behavior or another release's equivalence. For an absent system,
first locate entry points and persistence/callback boundaries; create Rust modules
only when there is an evidenced behavior to implement.

Every port must pass these gates:

1. **Evidence:** request/response construction, defaults, gates, producers,
   errors, timing/units and durable state changes have attributable source.
   JNI/native dependencies and Android-only behavior are followed or left unknown.
2. **Implementation:** patch the existing owner, preserve identity, ratchet,
   journal and reservation invariants, and refuse unsupported branches honestly.
3. **Offline verification:** independent APK-derived bytes or another attributed
   oracle cover success and refusal/partial cases, restart/rollback and duplicate
   behavior where applicable. Passing Rust round trips alone is insufficient.
4. **Exposure:** the existing CLI/library/API surface and OpenAPI describe actual
   capability and transport; regenerate CLI docs when commands change.
5. **Interoperability:** record a separately authorized, bounded live observation
   only when available. Otherwise leave that gate open and continue static work.

### Ordered system map

Rust owners below are relative to `crates/whatsapp-api/src/` unless stated
otherwise. **Historical live** means the repository records a past experiment;
this queue does not revalidate accounts or repeat requests. **Unknown** means
not established by this review, not proof that code is absent from the APK.

| ID / system | Reuse and current boundary | Next REA question → existing Rust owner | Acceptance boundary |
| --- | --- | --- | --- |
| A00 — Release, DEX, resources and native inventory | Current APK inventoried; most retained semantic evidence belongs to 37.71. Existing `xtask` recipes/remap/diff and REA envelopes are retained. | Map current class/method identities and native dependencies to existing topic/Rust ownership; distinguish changed, unchanged, ambiguous and untraced. Owners: `xtask/src/`, [APK coverage](apk-coverage.md), [file diff](apk-file-diff.md). | Every selected code/config/native topic has an exact release/hash and owner. Unassigned inventory remains visible; inventories and decompilation are never counted as ports. |
| A01 — Onboarding and account verification | Partial trace/port; historical SMS/OTP/DOB success. PIN/email branches offline. Source already returns `stale`/`incorrect`/`mismatch` OTP answers to code entry with the evidenced `retry_after`/60-second answer gate; the older missing-reentry recommendation is stale. Current-build `guess_wait` preservation and explicit PIN re-entry for `mismatch`/`guessed_too_fast` are now ported offline with restart/early-submission tests. Other PIN/password branches, `ok`/`attached`, old-device and passkey branches still require review. | Trace each current response field through its specific consumer, timer and next permitted action; do not substitute PIN wait for OTP wait. Owners: `registration/`, CLI `signup.rs` / `main.rs`. [Existing registration evidence](registration-lab.md). | Preserve saved identity, challenge and waits across every branch/restart; invalid/wrong/repeated codes have evidenced transitions and no generic refusal reset. |
| A02 — Authentication, transport and connection lifecycle | Comparison Noise/certificates/login historically live; default profile remains gated; bounded recoverable reconnect is local policy with offline tests. | Re-anchor current handshake/login/version and terminal/recoverable paths, request startup order, framing limits and reconnect producers. Owners: `transport/{classic,handshake,noise,cert,socket,framing,login,connection}.rs`, `runtime/`. [Transport evidence](messaging-transport.md). | Attributed bytes and certificate checks; stable app-version identity; recoverable loss preserves uncertainty; refusals do not reconnect. |
| A03 — Classic Signal, PQ and key inventory | Classic sessions historically live; hybrid/KEM primitives and selected bootstrap/fan-out paths offline. KEM upload now exists behind property 21198; native generation/initial ID selection is still untraced, rather than upload being wholly absent. | Follow current bundle/key-generation/rotation/last-resort producers, effective PQ gate and malformed-key paths. Owners: `signal/{direct,group,hybrid,kem}.rs`, `transport/{prekeys,kem_prekeys,device_key_batches}.rs`. [KEM inventory](kem-prekey-inventory.md), [native vectors](native-kem-vectors.md). | Independent vectors plus transactional session/inventory fixtures; upload gate, counts, IDs and consumption match traced producers; no fabricated key provenance. |
| A04 — Identity, PN/LID and device lifecycle | Partial trace/port, directory-query historical live, identity/companion changes largely offline. Own pairing was previously parked, not implemented. | Close admission, authenticated namespace mapping, identity-change cleanup, initial companion bootstrap and pairing completion. Owners: `signal/adv/`, `transport/{devices,jid_map,identity_notifications,companion_admission,own_device}.rs`. [Cross-device research](cross-device-sync.md). | Verify identity authority, device membership and old-key cleanup atomically; device replacement never silently adopts unauthenticated state. Pairing has its own evidenced completion before being advertised. |
| A05 — Direct messages and message mutations | Historical live text, reply, reaction, poll, edit/revoke; selective receive/own-device/mutation paths offline. Existing outbox/journal are authoritative. | Trace remaining content combinations, peer edit/revoke policy, quote attribution, expiration and keep/unkeep writers. Owners: `outbox.rs`, `transport/{outgoing,messaging,quotes,reactions,polls,chat_settings}.rs`, `transport/inbox.rs`. [Mutation gaps](unknowns.md), [polls](poll-votes.md). | Wire fixtures, immutable original-target/ownership guards, duplicate suppression and durable events cover each supported combination; no broad claim from one plain-text send. |
| A06 — Receipts, retries and send status | Historical delivery/read results; many device/group/aggregate projections offline. Normal-driver retry answer and fresh-bundle refetch are implemented, superseding the older matrix's absent-runtime statement. | Outgoing retry-count propagation is now traced/ported offline for both direct count-1/2 paths. Close the count-2 base-key record and higher-count comparison, receipt-carried keys, group/broadcast retry, primary cleanup and remaining privacy/read-self gates. Owners: `transport/{receipts,retry_answer,send_versions}.rs`, `transport/connection.rs`, `model/receipt.rs`. [Retry evidence](retry-answers.md), [receipt roles](receipt-user.md). | One original-recipient reservation per answer; no duplicate ratchet/send after restart; refused or unknown branches remain inspectable; each device/time fact retains actual authority. |
| A07 — Groups and communities | Historical group messaging and selected create/info operations; offline normal-outbox and metadata branches. Group reactions require distribution-shape review; community classification exists but management is absent. | Trace membership/version changes, sender-key exposure/distribution, subject recovery/MEX callbacks and community create/link/announcement. Owners: `transport/{groups,group_fanout,outgoing_group,group_inbox,sender_key_distribution}.rs`, `transport/mex/`. [Groups](groups.md), [normal outbox](normal-group-outbox.md). | Membership changes and retries cannot leak to removed devices or reuse stale sender-key exposure; management callbacks durably apply only validated complete replies. |
| A08 — Media, view-once, stickers and richer payloads | Historical image/voice/view-once and November document/video results; receive forms and quoted media have offline gaps. Sticker send, HD variants, thumbnails and several caption/format paths remain incomplete. | Follow descriptor/key-domain/upload/download producers, thumbnail/HD paths, view-once consumption/opened synchronization and richer media envelopes. Owners: `signal/image.rs`, `transport/{media,image_upload,image_download,outgoing_media,voice_note}.rs`. [Media](video-media.md), [documents](document-media.md), [stickers](stickers.md). | Bounded streaming/authentication, correct metadata and durable upload/send uncertainty; view-once cannot reopen after consumption/crash; per-format playback/display remains a separate live gate. |
| A09 — Contacts, presence, profiles and local chat actions | Historical contacts/typing/block/profile-picture results; contact-state/backoff, avatar transformations and several chat actions partial. | Trace current sync types, visibility/typing timing, profile image removal/preview/crop and local-vs-synchronized actions. Owners: `transport/{contacts,presence,profile_picture,about,blocklist,chat_clear,actions}.rs`. [Chat actions](chat-actions.md), [presence](presence.md). | Correct address/state transitions and bounded cache updates; local-only changes do not claim remote synchronization or imply a setter was accepted. |
| A10 — Feature configuration, privacy and account settings | ABProps parsing/storage and MobileConfig traces exist; effective values and complete application remain partial. Several features depend on unresolved selection gates. | Follow effective getter precedence, defaults, snapshots/deltas, account/build binding, refresh and setters before activating gated branches. Owners: `transport/{abprops,selector,two_factor}.rs`, `registration/request_config.rs`. [Feature configuration](feature-configuration.md), [MobileConfig](mobileconfig-table.md), [read policy](read-setting-sync.md). | Typed effective values retain source/version/freshness, actual gates feed callers, unknown values do not become invented defaults or automatic mutations. |
| A11 — Status and broadcasts | Historical text status/viewer/like receive results; image/audio/HD/reshares/privacy and ordinary broadcast send/list management incomplete. | Trace status audience/visibility selection, reshare/like/media producers, broadcast list membership and ordinary-list stanza/receipt differences. Owners: `transport/{groups,group_send,status_receipts,outgoing_group}.rs`. [Status](status.md), [broadcast](broadcast-dispatch.md). | Separate status from ordinary broadcast admission; audience/membership and original attribution survive restart; text success does not claim media/audience parity. |
| A12 — App-state, history and own-device synchronization | Cryptographic/action/snapshot primitives and selected contact/name writes exist; history notices/queue admission partial; full history application and own-device bootstrap incomplete. | Follow snapshot reconciliation, ordering/version gaps, complete action mutation set, authenticated history transfer and native store admission. Owners: `signal/app_state*`, `transport/{app_state_sync,history_notice,own_device}.rs`, `store/`, `sync.rs`. [Cross-device](cross-device-sync.md), [history queue](history-queue.md). | No unauthenticated or partial state becomes authority; replay/order/version and rollback cases have fixtures; missing history stays an explicit gap. |
| A13 — Channels / newsletters | Namespace modeled; previously parked; no full subscribe/publish/admin client port established. | Locate channel creation/discovery/subscription, message/receipt, signature/identity and moderation entry points and storage, then map onto existing client/events. Owners start with `model/jid.rs`, `model/message.rs`, `sync.rs`; no empty transport module. [Feature matrix](../feature-parity.md). | Separate namespace trust and permission semantics; an evidenced vertical slice precedes support claims. Ordinary group support does not establish channel support. |
| A14 — Voice/video calls | Absent from the current client feature matrix; protocol/native/media dependencies unresearched here. | Inventory actual call signaling/state machine, key material, media transport/codec JNI boundaries, permissions and teardown before choosing Rust adapters. Owners start at `transport/connection.rs`, `runtime/`, `sync.rs`; dedicated implementation only after trace. | Evidenced setup/accept/reject/end and error lifecycle, resource bounds and key cleanup; signaling decode alone is not an interoperable call or audible/video media proof. |
| A15 — Business, commerce, payments and optional APK services | No complete client feature ports established by this review. Presence of generated protobuf types or notification ACKs is not commerce/payment support. | Identify account/region/feature gates, actual request/response/state owners, message forms and external-service boundaries from current APK. Map evidenced messaging behavior into existing models/actions; keep external-service requirements explicit. | No guessed credentials, eligibility or transaction fields; each operation has permission/error/state fixtures and separate interoperability evidence. No live financial transaction is implied by static research. |
| A16 — Persistence, migrations, backup/restore and data lifecycle | Encrypted redb transactions, key rotation, outbox/inbox journal and crash tests exist; full consistent backup/restore and APK backup/history import equivalence remain unfinished. | Trace relevant backup/export/import admission, schema/version migration, retention/deletion and lifecycle behavior; distinguish local Rust format from APK format. Owners: `store/`, `session.rs`, `sync.rs`, CLI store commands. [Persistence design](../design/persistence.md), [replay audit](replay-safety-audit.md). | Consistent crypto+pending-work+idempotency snapshot, rollback detection and restart tests; never resume old Noise counters or claim a key-only dump is full restore. |
| A17 — Android UI, resources and platform adaptation | UI/Android services are not a Rust API port. Current resource count is inventory only; user-visible behavior, prompts, timers, defaults and platform callbacks can still affect protocol semantics. | Classify each discovered UI/platform dependency as behavior to preserve, platform adapter needed, reusable third party or presentation-only adaptation, with reason and caller. Owners: existing CLI/API and `model/`, `runtime/`; no wholesale Android UI clone assumed. | The Rust system exposes equivalent actionable states/errors and required inputs; presentation adaptation is explicit, never counted as protocol code reproduced. Asset payloads stay out of research bundles. |
| A18 — Native libraries and bundled dependencies | Selected 37.71 loader reconstruction/JNI KEM and independent vectors exist, not complete current-native coverage. Other libraries/dependencies remain to classify. | Inventory each current ELF/JNI export/dependency and Java caller, identify relevant unresolved algorithms/state owners, compare versions, then use Ghidra/REA on exact native targets. Reuse `xtask` native tools and `signal/kem.rs`. [Unpacking](native-unpacking.md), [native KEM](native-kem.md). | Correct artifact/address binding and caller-to-export chain; distinguish reconstructed native instructions, emulator execution and Rust equivalence. Reuse approved maintained components where appropriate; no needless rewrite of bundled runtimes. |
| A19 — Library/CLI/HTTP integration and release | Existing actor/event architecture, CLI and selective HTTP inspection routes exist; OpenAPI also describes CLI-only contracts. No production-complete client established. | For each closed boundary, wire existing facade/actor/CLI, error and event contracts; align actual HTTP routes, features, licensing and package graph. Owners: `client.rs`, `runtime/`, `sync.rs`, CLI, `docs/api/openapi.json`. [API contract](../api/README.md). | Default and supported-feature builds, meaningful tests, strict Clippy, docs/hygiene and honest capability matrix pass; advertised operation has actual transport and evidence, with live gaps retained. |

### Execution order and resumption

Start with A00 plus the shared A01/A02/A03/A04/A06/A10 boundaries that can change
all later behavior. The current focused investigation begins with A01 response/wait
parity, reusing the existing wrong-OTP recovery; it does **not** replace this whole-system scope. Continue A05/A07/A08/
A09/A11/A12 using the existing ports and fixtures, while A18 follows any native
dependency exposed by those traces. A13/A14/A15/A16/A17 remain in the queue even
where an earlier milestone deferred them. Apply A19 to each completed chunk,
not once after a speculative full rewrite.

Before resuming any row, read its linked latest findings, recipe/REA evidence
and actual Rust call sites. Pick the first unresolved branch, export only the
needed attributed methods/callers/config, implement and test it, then record the
next boundary. Do not turn all 106,897 class names into implementation tickets:
class names are locators, while behavior and state transitions are port units.
Unknown/unassigned classes and relevant resources/native code still remain in
inventory accounting until classified; no class is silently declared complete.

No background conversion service or unattended account test is asserted by this
queue. Research/porting progresses through recorded, reviewed chunks. The latest
historical status records banned lab accounts and a pending verification; static
work is independent of those accounts. Future live work must inspect selected
private saved state and have the required authorization, without replacing keys,
resetting refusals or replaying acknowledged/uncertain sends.

## Anchor-assisted relocation

Recipe targets now accept `anchors`: exact string constants chosen from the
interpreted source. Use specific diagnostic/protocol constants, preferably more
than one independent string. Common strings such as `type` are weak locators.
Local bounds are eight nonempty anchors per target, each at most 4096 bytes.

```sh
target/debug/xtask apk-research-remap /path/to/new-WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-groups.json \
  --output lab/releases/proposed-next-groups.json
```

The scanner runs all anchors in one DEX pass. It uses checked string/class-data/
instruction extents and fails reported DEX/structural errors rather than using
the exploratory `apk-xref` warning-and-continue path. This does not constitute
full VM verification or recovery of unknown/native behavior.

Method targets intersect physical `(DEX, class, method-index)` identities;
whole-class targets intersect `(DEX, class)`. Repeated loads in a method do not
create ambiguity. Same name/shorty with different indices and same index in
another DEX remain distinct. Whole-class targets retain their whole-class filter,
instead of silently narrowing to the method that loaded an anchor.

JSON statuses are `unique`, `missing`, `ambiguous`, `unanchored`, and
`no_common_scope` (all strings exist but share no required scope). Both complete
candidate locations and per-anchor occurrence lists are retained. Index/offset
values are APK-local locators, never stable cross-release identifiers. Original
class/method/Rust ownership remains in the report for comparison. No old-name
fallback is made when anchors disappear or move to separate helpers.

Without `--output`, the command only inspects. With `--output`, every topic must
be unique or the command refuses to write, after printing its diagnostics.
Existing output files are never overwritten. A generated proposal still needs
source/caller/condition review before Rust behavior is changed; matching strings
alone cannot establish equivalence. Method-substring filters become the observed
method name in proposals; broad substring scopes require explicit review.

The committed six-topic group recipe includes scoped anchors from its existing
bytecode evidence. [Current-APK relocation evidence](apk/remap-evidence/2.26.37.71-groups.json)
records every candidate and occurrence. This verifies relocation against the
current baseline; a distinct new-release APK comparison remains unverified.

Confirmed static validation (2026-10-07): APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
yielded unique candidates for all six topics in 5.225 seconds, matching the
reviewed class/method selectors exactly. The linked report carries the DEX,
method indices and instruction offsets needed to repeat each match.
A negative probe combining `GET_GROUP_NOTIFICATION_VERSION_SQL` and
`UPDATE_GROUP_NOTIFICATION_SUBJECT_TIMESTAMP` found both strings but no common
method scope: output was refused, exit 1, with no proposal file created.
An existing proposal was refused without changing its bytes. Adding anchors
changed only the target metadata in the cached comparison: zero code-topic
changes. The regenerated baseline contains 11 files/392,185 bytes.
These are local static observations, not a new-release or live protocol test.

The cache-lifecycle follow-up adds a seventh insertion topic to the current
recipe; all seven anchors resolve uniquely in the same 37.71 APK. The updated
baseline is 12 files/395,753 bytes. Newly named standard arithmetic opcodes
change disassembly rendering, not the APK hash or demonstrated runtime behavior.
See [trace](groups.md#subject-cache-invalidation-and-timestamp-decoding-2026-10-07).

The complete-group classification follow-up adds an eighth reviewed topic
(13 files/397,778 bytes). Its four anchors resolve to **two** methods in 37.71:
ADs.A01 and AZK.AAK. The exact reviewed baseline selector remains ADs.A01, but
automatic proposal writing correctly refuses the ambiguous topic. Seven other
topics remain unique. This is evidence of an actual duplicated locator scope,
not a reason to choose the old obfuscated name in a new release. Caller-assisted
disambiguation remains needed.


## Caller anchors for ambiguous methods (2026-10-07)

`caller_anchors` is an optional exact-string array with the same eight-string /
4096-byte bounds as target anchors. It requires target anchors and method scope;
whole-class caller filtering is deliberately unsupported. All caller anchors
must occur in one physical caller method, and that method must symbolically
invoke the candidate. Multiple qualifying callers are allowed; multiple surviving
candidates remain ambiguous. No old class/name fallback is used.

The scanner maps declaring class, method name and **full** parameter/return
signature across DEX pools. Shorties cannot distinguish different object overloads;
[DEX method/prototype/type-list tables](https://source.android.com/docs/core/runtime/dex-format)
provide the actual descriptors. Code streams use the checked reference walker,
including payload exclusion. Reported read/structure/metadata failures propagate;
this is not complete DEX VM verification, reflective/native reachability or
runtime virtual dispatch resolution.

Extra unresolved statuses: `missing_caller_anchor`, `no_common_caller_scope`,
`no_matching_caller`. Configured topics retain `initial_candidates`,
`caller_anchor_sites`, `qualifying_callers`, `caller_matches` (target, full
signature and all symbolic caller sites); `candidates` is the filtered set.
These are source locators, not proof of unchanged behavior or automatic patches.

**Confirmed static:** APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The group-type recipe's target strings originally matched ADs.A01 and AZK.AAK.
Adding actual caller anchor `group_response` resolves ADs.A01 through its caller
A0C and AWc.C8f; A0C loads the string at method-relative 0236 and invokes the
classifier at 0248. [Evidence](group-type-caller-evidence.txt) records all eight
symbolic classifier invokes and five string-anchor scopes. The strict report
retains zero direct invokes for the competing schema reader; it does not claim
that reader is unused or cannot be reached indirectly.

The real scan resolved all eight topics and wrote a new proposal in 10.012s.
All selected class/method pairs match the reviewed current recipe. The updated
baseline is 13 files/397,832 bytes; cached before/after comparison found all
8 code topics unchanged and only targets metadata changed. A distinct new
release remains unverified. This supersedes the earlier current-APK inability
to generate an all-unique proposal, not the need for semantic review.

Three actual current-APK negative fixtures used a missing caller string,
existing strings split across caller scopes, and a valid unrelated writer
anchor. They respectively reported missing_caller_anchor, no_common_caller_scope
and no_matching_caller; every requested proposal exited 1 with no output file.
Synthetic DEX fixtures separately cover same-shorty object overloads, cross-DEX
matching, payload invoke bytes, invalid parameter indices/truncated lists and
physical caller/callee index separation. An initial JSON-bound fixture syntax
error was fixed before the full test rerun. Strict Clippy requested array chunks
for two-byte type entries; the same checked even-sized extent is retained.

The [core release review](core-release-review.md) expands this workflow to 22
registration, Noise, framing, socket and dictionary topics. A genuine cached
37.70→37.71 comparison completes in 0.0512 seconds and points to affected Rust
files. The [token extraction correction](token-extraction.md) demonstrates the
porting loop: trace actual indexed APK writes, preserve refuted historical
results, install a derived Rust fixture, and test literal wire bytes. Broad APK
coverage and complete client behavior remain unfinished.

## Checked shorty constraints

Recipe method targets may specify `method_shorty`, an exact DEX shorty (return
character followed by parameter characters). References/arrays collapse to L;
V is legal only in the return position. Local bound is 256 characters. A shorty
requires a method filter and cannot replace full-signature caller evidence.
Export and remap both apply it; absent matches diagnose `no_matching_shorty`.
The remap report preserves `before_shorty_candidates` and the chosen constraint.
Unknown fields, malformed shorties and a constraint without a method refuse.

The 37.71 subject timestamp parser's diagnostics occur in three methods. The
reviewed JLL return/parameter shape selects 0hs.A09 instead of integer parser
0hs.A06 or reference-valued 8lB.A0L. An intentionally wrong VLL constraint
must refuse proposal creation; it must not fall back to names or another return
type. Two unanchored subject helper topics remain unresolved automatically.
[Typed subject evidence](groups.md#subject-typed-branches-and-signed-timestamp-contract-2026-10-07)
records the resulting source coverage and unfinished Rust notification behavior.

Current typed-subject coverage is fifteen topics (20 files/430,619 bytes).
The [cached coverage diff](apk/subject-types-coverage-diff.json) took 0.0227
seconds: four added topics, two signed-annotation changes, nine unchanged topics.
The [reviewed remap](apk/remap-evidence/2.26.37.71-subject-types.json) has thirteen
unique and two unanchored topics; full proposal creation still refuses.
[Wrong-shorty evidence](apk/remap-evidence/2.26.37.71-subject-wrong-shorty.json)
preserves all three diagnostic matches and their rejection by VLL. Current core
27-file export remains byte-identical. This is same-APK coverage expansion and
rendering correction, not a new release or completed subject receive behavior.

The [recovery follow-up](groups.md#subject-recovery-mex-dispatch-and-local-guard-2026-10-07)
adds five topics for twenty total group topics (25 files/448,127 bytes).
[Cached coverage comparison](apk/group-info-recovery-coverage-diff.json) took
0.0204 seconds, with fifteen previous code topics unchanged. Sixteen remaps are
unique and four helpers are unanchored. MEX QueryGroupInfo is now a concrete
missing Rust path; the recovered local monotonic guard, context enums and
callback contract identify the work needed before replacing classic lookup.
The MEX document/envelope, complete result conversion and recovery callbacks
must still be traced. Static discovery does not authorize testing invented
query identifiers or replaying uncertain requests.

## Persisted operation IDs and request ports

[The MEX trace and port](mex.md) adds selected APK asset extraction alongside
method evidence. `apk-mex-queries` exports version/hash-scoped operation records
with exact server IDs, preserving client IDs separately. Compare these small
JSON files directly for the next release; avoid scanning disassembly to patch
a data-only ID change. `mex-group-preview` invokes the actual Rust group request
builder using the selected file, making its ordered node and nested JSON
inspectable without an account. Both tools are offline. The 37.71 QueryGroupInfo
fixture is reproduced from the real APK; complete response conversion and
durable live dispatch remain the next port, not a finished group implementation.

MEX response-parts parsing and group callback gating now have offline inspection.
The [eighteen-topic MEX recipe](apk/recipes/2.26.37.71-mex.json) exports 23 files/
76,908 bytes including checksums, linked to the actual Rust module. The
[callback coverage comparison](apk/mex-callback-coverage-diff.json) adds six
methods with twelve unchanged sources in 0.0069 seconds; both sides use the
same APK, so this is coverage expansion rather than a release change. Remap
has eight unique/seven unanchored/three ambiguous mappings. Ambiguous
is_retryable/shorty-Z matches include I1H.A06 and 916.BGC/BQ2; unresolved
mappings require review before next-release patching. [Callback interpretation](mex.md#group-callback-gate-and-inherited-dispatch-2026-10-07)
distinguishes a permitted callback from complete group validity and state
application. A parser inspection or self-diff is not full APK/client parity.


## Checksum coverage correction (2026-10-07)

**Observed local checker limitation:** `.github/ci/hygiene.rs` checked file
budgets/links across tracked files, but its research-checksum section hardcoded
only `docs/protocol/apk/2.26.37.70`. Thus prior hygiene output of 564315 bytes
proved that older bundle's checksums, not every newer group/MEX snapshot.
Checksum-verifying `apk-research-diff` had separately validated the scoped MEX
exports; the old hygiene output must not be promoted to whole-snapshot coverage.

The checker now discovers direct snapshot directories under `docs/protocol/apk`
with a checksums.json, sorts them, and validates every selected bundle's exact
file list, byte sizes, SHA-256 values and separate 2 MiB budget. No snapshot
allows undeclared files; an empty discovery refuses. Recipe directories without
checksums are outside snapshot verification. This is CI/research validation,
not a new protocol requirement or client/server capability.

The current run verified **four bundles / 2029764 bytes**, plus 621 local links,
tracked-file budgets and package licenses. A dependency-free test creates its
own temporary bundle and proves same-size hash corruption, changed size and an
unexpected file are rejected; CI runs it before the real repository checker.
No committed research artifact was changed to make an invalid checksum pass.
Current APK evidence remains build 2.26.37.71 with the recorded hash; this
repository-checker finding is observed tooling behavior, not APK behavior.
