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
