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
