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
