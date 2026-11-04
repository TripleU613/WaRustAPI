# Reach-out state update branches

**Confirmed static**, APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX text exports, not runtime execution or REA Evidence envelopes.

8AF.A06 maps7MU to NotificationUserReachoutTimelockUpdate.7MU constructor
loads7vL state manager.8AF.A07 obtains active/string/enum through typed
GraphQL accessors and invokes7vL.A00. The field-ID/name mapping, upstream
MEX sender authority and enum policy remain untraced.

7vL.A00 enters active handling only if is_active and config1f48 are true.
There null duration logs/returns without state update; NumberFormatException
also logs/returns.6oe.A03 uses Long.parseLong then multiplies by1000 for
local milliseconds. Full Java overflow/negative policy is not ported by the
Rust nonnegative-i64 local guard. The other branch resets active/start/end/
type/visible preferences, including explicit is_active=false regardless
of supplied duration. Feature-disabled reset is retained evidence, not
authority to invent the APK configuration value in Rust.

**Refuted previous local assumption:** expiry is not required for an explicit
inactive update. Rust now stores active=false, ends=0 without parsing expiry,
restoring valid lock-release behavior while malformed **active** updates still
refuse before write/ACK. Regression covers both branches against a saved
active restriction. The earlier source-fix entry is historical; its claim
about malformed inactive updates is superseded by this evidence.

No proof of sender authorization follows from this leaf. No new from-JID
allowlist is guessed. ACK behavior for malformed active content remains an
explicit local refusal policy, not claimed native base-handler parity.
No live account operations or full restriction-policy parity is claimed.

Reproduce apk-smali8AF.A06/A07,7MU whole class,7vL whole class and
6oe.A03 with --signatures --exceptions. The full manager is retained so
reset/feature branches can be checked without excerpt truncation.

- [operation-map.txt](operation-map.txt) — SHA-256 `578d0b5a8fcbf1d4334ee2ac99bad0fae8432a75243beebcba49799cdefcb797`.
- [handler-constructor.txt](handler-constructor.txt) — SHA-256 `af3629bcee481549eaf8cd40d3439167c9b25a0881c6c4d8074f4f50ffe7e2db`.
- [dispatch.txt](dispatch.txt) — SHA-256 `0e98b666ff2af77b1419ad7295f1f8156e6cb143ba3eaa07a687ff566a20df83`.
- [state-manager.txt](state-manager.txt) — SHA-256 `3fd80e8c1a011eb4ff77fea251c28dd17646d83ed39de1529d0bfa3035e13d07`.
- [expiry-parser.txt](expiry-parser.txt) — SHA-256 `b4301282d7f2533ca007e17512aedadaeda2bb1f5f1cc6056275fd8a56fb37f4`.

## Current MEX operation routing (2026-11-04, offline corrected)

**Confirmed static/source mismatch fixed:**0v7.A09 gets first update child
via0Tp.A0F, requires nonempty op_name, lowercases with Locale.US and looks
up the registered handler. It parses optional outer t and passes original
node plus timestamp to8AF.A09.8AF invokes7jz JSON adapter and typed0is
conversion before state dispatch, throwing on conversion failure. Full
upstream sender authority and conversion semantics remain untraced.
No from-JID check appears in the retained operation-router leaf; absence
there does not prove it is absent upstream.

Rust previously scanned every child by op_name and exact case, so unrelated
child tags or a later update could change restriction state. It now selects
only the first update, and matches the supported ASCII operation name
case-insensitively. Non-ASCII Locale.US case conversion equivalence is not
claimed. Malformed/unknown first update does not fall through to later ones.
Known JSON/expiry validation and commit-before-ACK ordering remain unchanged.
Regression verifies wrong tag and later-update refusal plus lowercase routing.
No live operation, generic handler expansion or sender allowlist is introduced.

Discovery limitation: short exact log prefixes had no xrefs; filtered strings
revealed full log literals and anchored0v7.A09. Its error text mentions
operation_name, but actual instruction reads op_name; follow the instruction.
Reproduce apk-strings --contains mexnotificationrouter, apk-xref full literal,
apk-smali0v7.A09 and8AF.A09 with signatures/exceptions.

- [mex-router.txt](mex-router.txt) — SHA-256 `0517c6e39e4707b2e412caabfd1d983af47b4a988d33be9808dfec22eabf6ea7`.
- [json-handler-adapter.txt](json-handler-adapter.txt) — SHA-256 `856817a3ffc55eabef8b4352c3ca3284b835aca434b9bc47384b0587f5a979a2`.

## Base dispatch and local envelope-before-state guard (2026-11-04)

**Confirmed static**, same pinned APK:0v6.A01 owns handler dispatch,
bookkeeping, exception handling and completion;0v6.A00 routes ACK through
1ic/A0G for one wrapper kind or0Sf.A0O otherwise. Full base methods retained.
Two direct A01 callers are8aj.run/8bl.run; their construction and upstream
sender authorization remain untraced. No account-server sender allowlist is
proved by these leaves.

**Confirmed local source defect corrected offline:** reach-out state could
commit before mex_ack rejected missing/oversized ID, invalid from or invalid
participant. record_reachout_timelock now invokes the existing base_ack
validation before any checkpoint write. It only constructs/discards the
prospective ACK; actual ACK remains after successful persistence in receive.
Malformed envelope returns InvalidInput through the existing captured/no-ACK/
continue path. Store failures still propagate. This is an existing local
envelope invariant moved before mutation, not a port of untraced native
origin authentication. Valid from remains a parsed JID, not evidence it is
authorized to set account state. That separate audit gap stays open.

Real-store regression preserves an active lock across missing ID/from, empty
ID and invalid from/participant, then permits explicit inactive update with
a valid envelope and builds its ACK. Existing data tests now include valid
synthetic envelopes so content tests cannot pass through malformed wrappers.
No live notification, new sender allowlist or account operation was executed.

Reproduce apk-smali0v6.A01/A00 with signatures/exceptions; apk-callers
0v6.A01 for the bounded two-caller result. No upstream completeness claimed.

- [base-notification-dispatch.txt](base-notification-dispatch.txt) — SHA-256 `9527dd697ff8e0341a263ba6fd73ffadf213f40d8b996b170663b15e3f981c7c`.
- [base-notification-ack.txt](base-notification-ack.txt) — SHA-256 `42e84943e757ca535f95aa5b5cc6d5957a2550d12099df726335b99f3bdeb224`.

Envelope test correction: the shared JID parser accepts unfamiliar server-only
names as syntactic Server::Other. The regression uses malformed device syntax
for invalid-address cases. This further demonstrates that this local guard
does not establish trusted sender authority.

## Message wrapper and reach-out ordering reservation (2026-11-04)

**Confirmed static**, same pinned APK.0v7 constructor registers message
code0x102 with0v6.0v7.BLn reads IncomingStanza node, selects first update,
lowercases supported op and invokes handler.A08 before superclass dispatch.
It catches pre-reservation exceptions and still delegates to0v6.BLn.
8AF.A08 default is return-void; the retained7MU class declares only its
constructor, therefore inherits this no-op reservation for reach-out. This
closes a specific ordering-reservation question without inventing a durable
queue requirement for that handler. Other subclasses may override it.

0v6.BLn first checks message code membership, casts Message.obj to1k2,
requires nonnull Ay8 node, creates Eoh using loggableStanzaId and invokes
8bl selector13. That branch records first child tag plus offline presence
under stanza key, then dispatches0v6.A01 directly if A09 executor is null
or schedules8aj(selector1) otherwise.8aj forwards wrapper/stanza metadata
to0v6.A01. These are exact local worker facts, not server-origin validation.
The first-child tag used for bookkeeping must not be confused with the
first-update child used for operation routing.

No sender-JID check is established in these wrapper leaves; upstream1k2
construction, type0x102 producer and central handler selection remain open.
No claim is made that every sender is accepted by the native network stack.
Rust keeps its current local envelope-before-state guard. No new allowlist,
automatic ACK policy, runtime ordering change or live traffic is introduced.

Reproduce apk-smali0v7.BLn/<init>,0v6.BLn,8bl.run,8aj.run,8AF.A08
with signatures/exceptions. Full shared-selector worker methods retained;
conclusions above apply only to selector13/1, not unrelated worker cases.

- [worker-dispatch.txt](worker-dispatch.txt) — SHA-256 `833b7aee149ebd574a2679fe14e0e510fc597d5f98f0cfa1976a55d046a0e817`.
- [worker-ingress.txt](worker-ingress.txt) — SHA-256 `e772370b831918d13e9b020e2d44203fb55abf2c178117f136c04a62ddb67164`.
- [router-ingress.txt](router-ingress.txt) — SHA-256 `7e73960546ed2e3c7d9b51d1a487c00b4ff1f43b6ef7df8b0c52f8a6dd3283d0`.
- [ordering-default.txt](ordering-default.txt) — SHA-256 `7de34f7d8b2bbc61f7e5181ef9cb33baaf3cec32534983708311973eb413be92`.
- [base-message-ingress.txt](base-message-ingress.txt) — SHA-256 `373f7d0ef24ac4cc3a864a4f653d8a424ce4b79d36272099a87c06ee5a275b62`.
- [router-registration.txt](router-registration.txt) — SHA-256 `9b540763ed7b01519d39a17d14cb99576b39a91f805910fbf4ca7064b851880d`.

## Receive registry discovery and schema false lead (2026-11-04)

**Confirmed static**, same pinned APK. Exact mex anchor census finds six
const-string sites: KWt.invoke,1s2.run,7Ou constructor and E88/E8e/EL2
class initializers. These are discovery candidates, not a list of required
network requests or six equivalent entry points.

1s2.run's notification branch obtains Jb9/JoW registry, lowercases type with
Locale.ROOT and looks up a type handler. If absent and type=mex, it selects
first update/op_name, lowercases it and queries the separate operation map.
J9D handler kind and config4632 participate in a gated dispatch branch;
it invokes0ZP.A05 and constructs an Eza wrapper with receive bookkeeping.
Full gate meaning, fallback control flow, wrapper producer and authentication
remain untraced. KWt.invoke builds the registry maps and treats mex
registration specially. This alternate registry is not yet bound to every
legacy0x102 route or the reach-out handler. No completeness claim follows.

**Negative result:**7Ou's mex literal belongs to overloaded constructor
(0Tp,0mK) validating an **IQ** result shape, not its notification constructor
(0Tp,int), which serves other types. The14v caller is a companion
side-channel notification family, not proof of MEX reach-out sender rules.
E88's mex enum item likewise labels a type without establishing its authority.
These full exports are retained so future traces can avoid reusing the
wrong overload or assuming obfuscated class identity implies one schema.

Exact direct-call scan0v7.BLn returns0; interface/base dispatch remains
possible and was already observed through base BLn workers. This bounded
negative does not prove unreachable code or absence of native handling.
Sender authority is still open at the receive-wrapper/central dispatch
boundary. Rust policy and network behavior remain unchanged.

Reproduce apk-xref --anchor mex, apk-smali1s2.run,KWt.invoke,7Ou whole,
E88.<clinit>,14v.A09 with signatures/exceptions; apk-callers0v7.BLn.
Only inspected selected candidates are retained, not all six initializers.

- [mex-anchor-census.txt](mex-anchor-census.txt) — SHA-256 `6d7970564e717086380f4d985b843888070ac73259b61ddffd548cea1c422012`.
- [receive-notification-registry.txt](receive-notification-registry.txt) — SHA-256 `01cb40753a2d16b60eb82cbb32563689ddbd855f07220ad810a2c7a306ecfcd6`.
- [shared-schema-overloads.txt](shared-schema-overloads.txt) — SHA-256 `8a2569401e9890eebe527b5d35d33c1dbe42a022ed4e75f7afe0322624f04ff0`.
- [registry-builder.txt](registry-builder.txt) — SHA-256 `6cbd72b0cc9faab5d81d3bc34dc700ee763bf3046ba1ee5aea432cffd9e07f0c`.
- [notification-type-enum.txt](notification-type-enum.txt) — SHA-256 `b6d076a5c48e696d2900d40af2625952b78084af021c1067dd6ca17091e91d6c`.
- [unrelated-notification-family.txt](unrelated-notification-family.txt) — SHA-256 `97778d4062f2a02cd21c448bd5428b278d178dc4b5a08658b8366a849842fab9`.

## Receive gate is persistence, not origin validation (2026-11-04)

**Confirmed static**, same pinned APK.0ZP.A04 reads numeric offline with
default-1, obtains configurable threshold through A00(config3cd4,-1,100),
and can latch A01 buffering when offline count reaches threshold. Its log
explicitly calls this shouldPersistStanza/offline buffering. The latch can
affect later stanzas; this leaf does not establish reset lifetime.
Absent offline returns true if configured threshold is-1, otherwise follows
current buffering state. A00 config helper/rollout semantics are untraced.

0ZP.A05 combines consumer readiness (1ic-selected consumer/A0N), special
consumer-presence and0av.A0J checks, falling back to A04 in a branch. It
contains no from-JID read. In the retained1s2 notification paths its boolean
is inverted and passed to Eza wrapper construction. That proves a wrapper
flag dataflow, not that the flag is an authentication verdict. Exact wrapper
flag semantics and full fallback binding remain open.

**Negative result:**0ZP.A03 is scheduleQueueFlush, not the guessed legacy
notification forwarding method. Full output retained; scheduler lifetime,
actual queue worker and fallback consumer are not reconstructed here.
Searches based on short obfuscated method names must not turn these leaves
into origin requirements. No sender allowlist or persist/ACK behavior in Rust
is changed. Native buffering configuration is not copied as a universal
request quota or cooldown.

Reproduce apk-smali0ZP.A04/A05/A03 with signatures/exceptions. Native
1s2 registry output already retained provides call sites and inverted flag.
Upstream authenticated input and specific message0x102 producer remain open.
No live network/account operations were performed.

- [receive-persistence-gate.txt](receive-persistence-gate.txt) — SHA-256 `c26dc224e247484e137de52a393f6fe657863b4dfa662c4b8113cef8a2aed141`.
- [offline-buffering-gate.txt](offline-buffering-gate.txt) — SHA-256 `d84ef20313a7bcfd61e0c665dc7b343fe3d318240935af34792a77d83b99862b`.
- [queue-flush-scheduler.txt](queue-flush-scheduler.txt) — SHA-256 `c279994b64e063d34756f5d1c6199907f960a9b7c50c9c4b25067a5a97c81860`.
