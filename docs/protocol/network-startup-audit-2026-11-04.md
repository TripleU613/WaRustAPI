# Network startup and login consistency audit

2026-11-04. Offline source audit of `transport/connection.rs`,
`transport/login.rs` and `runtime/comparison.rs`, against the attributed
[comparison transport evidence](messaging-transport.md) and
[login identity contract](login-identity-updates.md). No account, socket, login,
OTP request or message was used for this audit. No causal explanation or
prevention guarantee for account restrictions is established.

## Evidence identities and scope

The retained transport findings concern APK **2.26.37.70**, SHA-256
`b2090b0207e9b97449a5b48e98eb4d1649ae8e5e738e9b7431d6242717fee732`,
and later selected **2.26.37.71** traces, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The available current input is **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Its package/class inventory does not establish that the entire current startup
sequence equals those comparison builds. This chunk closes local consistency
faults; it introduces no newly inferred mandatory server IQ or APK timer value.

The [earlier handoff](project-handoff.md) and November status record app-version
mismatches and account restrictions. Temporal association is not proof of
causation. An official app can change builds on upgrade; the comparison driver's
saved-version rule is its local frozen-recipe selection contract, not a general
claim that a WhatsApp account may never change versions.

## Confirmed local defect 1: invalid facts silently selected a different build

Before this change, `login_app_version` searched the last
`registered_app_version` credential and chained UTF-8/parse failure into the
configured User-Agent, then a hardcoded 2.26.37.70 default. Thus an explicit
malformed saved build was treated as though it were absent. Different saved
version credentials also silently selected the last one. The signup writer
normally replaces previous entries, so contradictory duplicates are an
inconsistent local state, not a demonstrated app-upgrade procedure.

**Fixed local contract:** every saved version credential must parse, and all
saved versions must agree. Identical duplicates remain usable; malformed,
non-UTF-8 or conflicting values refuse with a static `login` diagnostic that
contains no supplied value. A valid recorded version continues to take precedence
over mutable config. Only genuine absence consults the configured User-Agent;
an explicit unparseable configured version refuses instead of becoming another
build. If both sources are genuinely absent, the documented historical 2.26.37.70
comparison default remains unchanged.

This avoids a proven unintended selection in our code. It does not prove that
the fallback caused any restriction or that selecting one consistent build is
sufficient for protocol compatibility.

## Confirmed local defect 2: validation followed network work

Previously `connection::connect` opened the TCP socket and performed the
certificate-checked initial Noise exchange before calling `primary_login`.
A local missing/malformed numeric login or unusable version could therefore
cause an unnecessary network handshake before the request was refused locally.

**Fixed ordering:** the private `login_material` seam performs config validation,
registered-account/phone checks, prior-login refusal/uncertainty checks, payload
construction and saved Noise-material lookup using store reads only.
`connect` must obtain this result before building its endpoint or invoking the
DNS/TCP connector or Noise handshake. The existing login reservation is still
written only immediately before the client-finish payload, with the existing
concurrent reservation guard. No rejected preflight writes a reservation,
changes an account or weakens a previous refusal.

The offline fixture uses a real encrypted store and bad explicit saved build,
checks the static refusal and unchanged account rows, reopens the store and
checks the same refusal and absent login reservation. The tested preflight seam
contains no networking code; source ordering places all socket work after it.
No assertion depends on contacting a real endpoint to prove that ordering.

## Checked startup path and preserved boundaries

**Confirmed from current Rust source:** after one certificate-checked login
success, `drive` sends a correlated `iq/set xmlns=passive` with `active`, then
waits for the result. Non-result activation refuses. Availability presence,
queued display name, reserved receipts and queued actions follow successful
activation; readiness is signalled and legacy prekey upload/pending device sync
are considered. The injected-link tests already exercise successful activation,
refusal, interrupted writes, preserved captures and queued work.

**Confirmed retained static:** comparison `Avw.A00`, called by primary
`0iS.A0e`, constructs active/passive IQ selection. Comparison
`EIY.<init>` branch 15 constructs a keepalive `iq/get xmlns=w:p` to the server
with an ID and **no ping child**. These attributed shapes support the existing
builders; they do not establish that the current APK has no additional bootstrap
work or that every optional IQ is mandatory.

**Confirmed from current Rust source:** heartbeat sends only after activation,
tracks one pending ping and classifies an unanswered ping as recoverable. The
period is derived from configured local I/O timeout, clamped to 1–20 seconds.
Neither that period nor its coupling to I/O timeout was established as the
APK's own idle/keepalive schedule. It is left explicitly a local policy; no
invented replacement interval or extra ping request is added by this audit.

**Confirmed prior correction, retained:** automatic reconnect is limited to
classified recoverable loss. Refusals, revocation, uncertain login and worker
panic do not authorize another login. Immediate activate/drop connections no
longer reset the attempt budget; a sufficiently stable connection or explicit
operator `connect` resets it. The current budget/delay/stability rule is local
policy, not a recovered native schedule. A nudge may advance a pending
recoverable reconnect within that budget; it does not convert refusals into
recoverable state. [Existing audit and regression](messaging-transport.md#reconnect-budget-stability-2026-10-13).

## Unresolved protocol questions and risk labels

- **Unknown current parity:** exact initial IQ order, conditional mandatory
  bootstrap work, effective account configuration and when native availability
  is announced. No omitted mandatory IQ was proven by this bounded review;
  absence of proof is not proof the Rust sequence is complete.
- **Known implementation / untraced decision:** login currently emits fixed
  `UserActivated`, `passive=false`, `short_connect=false` values even on automatic
  reconnect. The comparison producers `1LC.AAU`, `1Ed.AAU` and `1IC.AAU` are cited
  in earlier research, but the complete current reconnect-specific selector
  was not followed here. Do not choose an enum by its name or fabricate device
  state to make the payload appear more official.
- **Known implementation / unknown native schedule:** periodic keepalive is
  independent of arbitrary application traffic. Whether the current APK resets
  its timer on inbound/outbound activity, its deadlines/backoff and how its
  configuration changes them remain questions for a focused producer/consumer
  trace before changing behavior.
- **Suspected risk only:** an incomplete startup/configuration contract or excess
  reconnect/ping traffic could contribute to incompatibility or service load.
  No specific account restriction is attributed to these unknowns.

The next protocol work should re-anchor those producers and follow the actual
native callers, response consumers and effective gates. Preserve the existing
identity, verification, cooldown and uncertainty rules. No detection evasion,
client-provenance fabrication or live account experiment follows from this audit.

## Focused validation

Passed:

```sh
cargo test --locked -p whatsapp-api --all-features transport::login::tests
cargo test --locked -p whatsapp-api --all-features invalid_login_material_is_refused_by_offline_preflight_without_reservation
```

The first runs three tests, including malformed/non-UTF-8/conflicting and
identical duplicate saved versions, saved-vs-config priority, malformed explicit
configured version, and genuine absent fallback. The second verifies offline
preflight and preserved encrypted-store state across reopen. Integration-wide
checks belong to the root audit chunk's status; no live validation is claimed.

## Current activation builder leaf

Current 2.26.43.72 [checked builder/JID/callback](rea/2.26.43.72-active-iq-audit/README.md)
confirms Rust `messaging::active_request` already matches the true-branch IQ
shape. No extra attribute was found in that builder and no wire change was needed.
This does not establish full bootstrap order or native readiness/presence parity.

[Current startup caller records](rea/2.26.43.72-startup-callers/README.md) retain
login-sequence/active selection/writer-ready scheduling observations and explicit
provider no-body limitations. They do not close full bootstrap equivalence.

The follow-on [Noise/auth-key audit](authkey-lifecycle.md) adds persisted
Curve25519 public/private consistency checking to preflight, without replacing
keys or implementing the still-unclosed server rotation lifecycle.

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

## Metrics subcode handling (2026-11-05)

2026-11-05 — **Metrics error subcodes and credential effects traced.**

Pinned native paths recognize3067002/3067003/3067004, release the current
metrics credential and promote a spare or request gated renewal. Failure-reason
suffix parsing is distinct from response-body parsing; diagnostic numbers are
not account restriction codes. [Evidence and limits](apk/2.26.43.72-metrics-error-subcodes/README.md).
Validation:24 synthetic body-parser cases,7 worker integer-slice cases, pinned
resolver and bundle checks. Full retry/durable ownership and server meanings
remain open. No Rust runtime or live traffic changed.

## Regular metrics recovery (2026-11-05)

2026-11-05 — **Regular metrics recovery timer and provider gates traced.**

Matching provider/status0 with idle low bit starts a drain and interval30 timer;
status1 cancels the existing timer and signals drain error. Timer callbacks
require ready and idle low bits. [Evidence and limits](apk/2.26.43.72-metrics-regular-recovery/README.md).
Validation:72 native status-decision and16 timer-decision fixtures plus pinned
window resolver. Weak resolution, actual timers and durable ownership remain
separate; no Rust runtime or live traffic changed.

## Metrics buffer deletion (2026-11-05)

2026-11-05 — **Metrics completion-to-file-removal chain traced.**

Selected regular table now binds owner callbacks: null-error success queues
buffer removal; terminal local error skips that deletion path despite set
removal. Worker remove failures report errno separately. [Evidence and limits](apk/2.26.43.72-metrics-buffer-deletion/README.md).
Validation:4 controlled full-worker cases and pinned construction/function
windows. Downstream bookkeeping/recovery remains open; no real files removed,
Rust runtime change or live traffic.

## Metrics file recovery (2026-11-05)

2026-11-05 — **Metrics buffer loader admission and bulk-delete distinction traced.**

Loader accepts selected prefix/type and WMT versions1/2/3 with positive bounded
payload; restores checksum/state without payload comparison in this body.
Separate directory walk is bulk deletion. [Evidence and limits](apk/2.26.43.72-metrics-buffer-recovery/README.md).
Validation:18 native header cases and pinned loader/helper windows. Remaining
files can pass local gates; automatic restart replay and cleanup policy remain
unproven. No real I/O, Rust runtime change or live traffic.

## Recovered metrics consumer (2026-11-05)

2026-11-05 — **Recovered metrics consumer checksum and task admission traced.**

Version3 payload Adler32 is checked after file loading; versions1/2 skip this
consumer comparison. Mismatch increments a counter and ends the remaining
snapshot walk. Passing outgoing entries reach conditional transport handoff.
[Evidence and limits](apk/2.26.43.72-metrics-recovery-consumer/README.md).
Validation:12 native checksum cases and mismatch-counter slices plus pinned
binding/windows. Startup flag/lifetime and actual replay remain unobserved.
No real I/O, Rust runtime change or live traffic.

## Saved-session retry reservation guard (2026-11-05)

**Confirmed local correction / implemented offline:** `answer` prepared crypto
outside the write transaction, then checked only the saved session and duplicate
answer marker. A concurrently committed delivery/failure/attempt update could
therefore be ignored, or a changed identity pin could be paired with the prepared
old-session result. The fresh-bundle path already guarded the original row and
peer state; both paths now share the same original/age and identity/session guards.

Immediately before reservation, the saved-session path requires the outbox row
to equal the snapshot used for eligibility/preparation, the current identity and
session to equal the captured inputs, the first attempt still to be no older
than60 days, and the answer marker to be absent. Any failed guard returns a
local error and rolls back the write transaction: no retry attempt, reserved
answer marker or prepared ratchet is committed. It does not overwrite newer
state or silently retry. The exact60-day boundary remains eligible.

The retained APK DNP.run delivery exclusion and150.A04 age checks motivate
rechecking eligibility before resend; transactional equality/CAS is our local
concurrency mechanism, not a claimed Android database implementation or new
server policy. Native count/root/addressing rules remain separate.

Regression tests simulate a captured snapshot followed by a committed delivery,
failure or extra attempt, then try the same reservation guard/transaction. They
verify unchanged tables on refusal and success at exactly60 days; another case
crosses the age boundary without a row change. Existing peer-state tests cover
identity/session changes and retained pending-job state, and the complete
saved/fresh retry workflow remains covered. These are deterministic store/guard
simulations, not an end-to-end scheduled thread race or live retry test.

The guard assumes every outbox/pin/session change uses the same ProtocolStore
transaction boundary. Other retry families, higher counts and complete current
APK root normalization remain open. No live account operation occurred.

[Retained current-APK root gate instructions](apk/2.26.43.72-retry-root-gates/README.md)
remain incomplete for a normalization port.

## Refetch query reservation inputs (2026-11-05)

**Confirmed local correction / implemented offline:** the initial retry key-query
reservation previously checked only for an existing answer marker. The eligibility
and peer snapshots came from earlier read transactions, so competing store work
could make them stale before the fetching marker/job were committed. Even though
this branch does not await crypto, other store users can commit between those
transactions. The later refetch-completion guard did not prevent that unnecessary
query reservation.

`reserve_refetch` now reads the current original via its wire-id index within
the same write transaction as the two pending records. It requires the original
outbox row to equal the eligibility snapshot, repeats the strict >60-day age
check and compares identity/session bytes with the captured inputs (including
an absent session). A stale/missing original or changed peer state returns a local
error with no fetching marker or job committed. An existing answer marker still
returns false without another job. Only a successful reservation allows `answer`
to return the key-request node to its caller; no socket write occurs here.

This uses the same local guards as saved-session/fresh-bundle retry reservation.
It preserves the retained APK original-recipient/delivery/age exclusions without
claiming Android uses Rust's transaction mechanism. It does not delete/replace
a session, relax counts, rotate identity or add automatic retry after refusal.

A deterministic regression calls the actual reservation helper after a competing
transaction commits delivery, failure, an extra attempt, identity/session change,
an absent→present session, or deletion of the original. An aged captured original
also refuses at this boundary. Each refusal preserves all checked outbox,
checkpoint, identity and session rows. The unchanged case commits one pending
query; a second query ID with the same answer key returns false and adds no job.
The existing full saved/fresh retry workflow remains covered. These are store
interleaving simulations, not scheduled multithread races or live key requests.

**Observed live:** none. Full current-APK root normalization and higher-count
base-key policy remain unresolved. No new CLI command, route or protocol feature.

## Receipt-carried keys capability boundary (2026-11-05)

**Confirmed / corrected offline:** the normal direct retry driver previously
ignored a receipt's `keys` child despite receipt-key handling being unported.
It could answer under the saved session or reserve a directory query for a
branch whose real key/session/verification policy was not implemented.

The driver now explicitly returns no send/query and records local
`receipt_keys_unsupported` after existing original/recipient/count/acceptance/age
eligibility checks. Any top-level keys child triggers that local limit, including
malformed/empty keys; no key material is adopted or validated here. Repeats keep
one unanswered checkpoint and leave normal answer/query markers absent. A later
supported receipt without keys can still follow the ordinary policy.

[Current-build checked parser and session policy](apk/2.26.43.72-retry-receipt-keys/README.md)
retain F10.A04,163.A02/A05 and the distinct163.A01 legacy/PQ cleanup helper.
The explicit refusal is our capability boundary, **not** a claim that Android
always rejects key-carrying retries. Its conditional AEA/session/registration and
legacy/PQ fallback behavior still requires a full port. Receipt ACK policy and
other consumers of the parser are unchanged.

The full retry workflow regression tests matching/changed registration with
empty, byte-valued and partial structured keys. It asserts no node, unchanged
outbox/session/pin rows, only one unanswered checkpoint, no refetch job and
successful later no-keys resend. These are synthetic offline fixtures, not valid
native key bundles or live compatibility proof.

## Retry parser field admission (2026-11-05)

2026-11-05 — **Retry parser registration/version/defaults corrected.**

Shared parser requires four registration bytes, defaults missing v/count to1
and refuses unsupported version/malformed count. Invalid registration no longer
falls through to a key query. [Evidence and tests](retry-parser-admission.md).
No live request or expanded higher-count/key handling.

Parser and full retry-workflow regressions verify malformed fields preserve all
retry state and default-count receipts can produce the supported count1 answer.
Receipt ACK and complete parser/routing parity remain separate.

## Comparison group retry admission (2026-11-05)

2026-11-05 — **Comparison group retry admission strengthened.**

Before reserving a device or querying keys, the window checks group-chat/message
correlation, supported count, carried-key exclusion and original/duplicate device.
[Evidence, tests and remaining issues](group-retry-admission.md). No live request or group policy
completion claimed.

Group retry write-checkpoint ordering, session commit guard and encryption-child
count remain explicit follow-ups; status routing equality is not inferred.

## Peer snapshot and write settlement correction (2026-11-05)

**Confirmed local correction / implemented offline:** group/status comparison
retry preparation now captures identity and session together in one read snapshot.
Its write transaction requires both to remain unchanged and refuses an existing
retry checkpoint before saving the prepared session/pin. A concurrent ratchet
advance, session appearance/removal or pin change therefore preserves newer
state and creates no replacement reservation. This is local concurrency control,
not a newly inferred native reset/fetch policy.

The prepared peer state and checkpoint commit atomically **before** socket I/O.
The checkpoint is now **reserved**, with the key-query ID as reservation token,
received count and reservation timestamp. A successful `write_node` is followed
by owned settlement to **written**; an error settles to **uncertain**. Settlement
requires the same token/count and still-reserved status. Duplicate/stale settlement
cannot overwrite another state. Existing checkpoints, including legacy written
records, are not re-reserved or migrated into a resend.

A failed/ambiguous write does not roll back the advanced session and is not
retried. A crash after reservation leaves reserved state, also not replayable.
If socket I/O succeeds but settlement fails, the reservation remains held and an
explicit diagnostic reports the socket outcome without claiming a settled send.
The caller no longer claims the recipient can read the message: written proves
only the local write operation completed, not server ACK, delivery or decryption.

Two deterministic store regressions exercise the actual reservation/settlement
helpers: unchanged input; changed identity/session; absent→present and removed
session; existing legacy checkpoint; interruption/reopen after reservation;
stale token/count settlement; duplicate reservation/settlement; successful and
uncertain outcomes across reopen. Assertions preserve all checked tables on
refusal and preserve advanced peer state after either outcome. Socket outcomes
are supplied to the settlement helper; no real socket, process crash, concurrent
thread, crypto or injected storage fault is executed by these tests.

The group retry encryption-child count/root policy, status addressing and full
native base-key/key handling remain open. This fixes the two source issues above
without claiming full group/status retry parity or live acceptance.

[Implementation and prior source findings](group-retry-admission.md).

## Group comparison retry enc count (2026-11-05)

2026-11-05 — **Comparison group retry enc counts corrected from pinned DEX.**

Received count1/2 now reaches root skmsg and pairwise distribution enc children
unchanged; original first sends still omit it. Ciphertext/ID/addressing remain
preserved. [Binding, wire regression and limits](group-retry-count.md).
No live request or full group retry parity claimed.

Checked callback construction/interface/composition and both builder branches
support the count correction; complete root/routing/key policy remains open.

## Nested message key and timestamp correction (2026-11-05)

**Confirmed static / corrected offline:** the shared parser used root receipt id
as RetryRequest.message_id and ignored nested retry.id/t. Current F10.A04 instead
requires nested t/id and builds the original message key from nested id. Complete
[required readers and message-key construction](apk/2.26.43.72-retry-message-correlation/README.md)
close that selected correlation gap.

The parser now returns **nested retry.id** for original-send lookup. It requires
nested t to parse as a signed64-bit integer and applies existing local identifier
bounds to nested id. Root receipt id remains separately validated; no equality
between root/nested IDs is invented. Missing/malformed nested fields produce a
redacted field diagnostic and no actionable retry, before any outbox lookup,
query reservation, crypto or retry checkpoint mutation.

The timestamp is validated, not turned into a new retry/expiry clock. Native
F10 multiplies its parsed seconds by1000; complete downstream timing consumers
remain open. Rust still checks age against the persisted original first attempt.
The model/API adds no timestamp field, and full Java Unicode numeric syntax is
not claimed. Receipt ACK construction is unchanged and retains its own root
correlation semantics; all native parser/ACK exception effects remain unproven.

A parser regression covers independent root/nested IDs, missing/empty/oversized
nested id, missing/nonintegral/overflow t and signed64-bit boundary values.
The full saved/fresh retry workflow verifies root W1 + nested OTHER sends nothing
and leaves all retry tables unchanged. Missing/invalid nested fields do likewise.
Conversely, root ROOT-RECEIPT + nested W1 reaches the intended W1 resend with the
same original wire ID and remains once-only. Existing parser/version/count/key
fixtures now include valid id/t so they continue testing their intended fields.
The group comparison admission fixtures likewise use nested IDs. No live receipt,
key query, resend or ACK was sent.

## Retry original-chat correlation (2026-11-05)

2026-11-05 — **Retry original-chat selection/correlation strengthened.**

Shared parser separates explicit recipient/normalized chat from asking device.
Normal direct retry checks the selected chat against original peer/known LID
before session work or a query. [Evidence and regressions](retry-original-chat.md).
No receipt-derived alias adoption, live request or full native routing claim.

Native effective alias/context binding and typed-reader fallback remain open;
malformed explicit recipients use a strict local supported-path refusal.

2026-11-05 — **Retry ACK dispatch traced; plain-ACK suspicion refuted.**

Current Rust excludes retry ACKs. Selected APK handler separates handled,
unhandled/error487 and parse-exception paths; actual receipt context factory
located. Final Message76 writer binding/async order remain open.
[Evidence and limits](retry-ack-dispatch.md). No runtime policy change or live request.

2026-11-05 — **Unsupported retry versions now receive error487.**

Shared confirmation mirrors supported receipt context for explicit non-1 text
versions after four-byte registration admission. Writer binding and deferred
callback ordering traced. [Evidence, regressions and limits](retry-ack-dispatch.md#unsupported-version-error-response-correction--2026-11-05).
Other retry ACK/error/recovery policy remains incomplete; no live traffic.

2026-11-05 — **Retry schema-error ACK context corrected.**

Confirmed required-field/ASCII numeric errors now receive487 with fresh native
context, omitting recipient/edit. Unsupported-version487 keeps its copied context;
first-error order and local capability refusals remain distinct.
[Evidence, tests and limits](retry-ack-dispatch.md#required-field-parsing-error-responses--2026-11-05). No live traffic.

2026-11-05 — **Retry refetch settlement ownership strengthened.**

Refusal and resend reservation now recheck the pending job snapshot and exact
fetching/query/count checkpoint before consumption. Changed jobs/terminal or
legacy markers remain held without overwriting peer or retry state.
[Race regressions and limits](retry-answers.md#refetch-completion-ownership-guard-2026-11-05). No new send or success ACK.

2026-11-05 — **Reserved direct retries now ACK before resend write.**

Normal receive attaches success ACK only after durable message reservation and
uses root receipt correlation. ACK failure prevents resend submission; any
uncertain work stays held. Queries/refusals/duplicates/group paths remain outside
this ACK authority. [Evidence, tests and limits](retry-ack-dispatch.md#reserved-direct-resend-success-ack--2026-11-05). No live traffic.

2026-11-05 — **Durable retry key queries now ACK first.**

Selected post-ACK job/session-requirement fetch entry traced. Normal wrapper
attaches root receipt ACK only after pending job and fetching marker commit;
ACK/query failure stops without replay. [Evidence, tests and limits](retry-ack-dispatch.md#durable-retry-key-query-ack-ordering--2026-11-05).
No key-request field/backoff change or live traffic; recovery remains incomplete.

2026-11-05 — **Retry key-query manager-to-wire construction audited.**

Classic non-identity/context-disabled IQ baseline matches Rust. Source12 is not
a wire field; identity/PQ/context attrs have separate predicates. Manager entry
expiry is not a proven IQ deadline. [Evidence and remaining gaps](retry-key-query-wire.md).
No production field/backoff/replay change or live traffic.

2026-11-05 — **Retry refetch callbacks now belong to their live connection.**

Only a locally written query can complete through normal receive; result/error
consumes its callback once. Disconnect/restart never resurrects callbacks from
held jobs. [Native cancellation evidence, tests and limits](retry-answers.md#live-refetch-callback-lifetime-2026-11-05).
No guessed timeout, new request, reset or automatic replay; no live traffic.

2026-11-05 — **Outgoing retries now preserve original message timestamp.**

Automatic retry.t uses failed message root t instead of local queue time.
Missing/unusable timestamps refuse without queue writes; held retries do not
reset on changed metadata. [Native chain, regressions and limits](outgoing-retry-receipts.md).
Error-code/key/count progression remains incomplete; no live traffic.

2026-11-05 — **Outgoing retry error domains/eligibility mapped.**

Crypto status, internal failure kind and wire error are distinct; selected
crypto eligibility and guarded refinements recovered. Other producers bypass
the converter. [Evidence and remaining typed-cause work](outgoing-retry-receipts.md#error-domains-and-eligibility-recovered--2026-11-05).
No blanket error default, runtime behavior change or live traffic.


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
