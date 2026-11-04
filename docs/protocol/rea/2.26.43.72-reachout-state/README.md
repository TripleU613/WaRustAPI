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

## Legacy notification-to-MEX code binding (2026-11-04)

**Confirmed static**, same pinned APK. In retained1s2.run fallback104a..1072,
1rh.A01 map is indexed by stanza description, result cast1rA, original node
wrapped1uU and passed to A03.1rh constructor preserves supplied map without
building its entries. This closes fallback invocation, not every map producer.

1rA.A03 is shared by several concrete subclasses. Its1rc branch selects
notification handling and invokes1rN.A05 after shared offline/metrics logic.
1rc constructor extends1rN and stores its0Ji consumer.1rN.A05's1rc branch
reads required id, type and optional offline, obtains EL2.A00 lookup, then
constructs Message through2Ai.A00(wrapper,loggable-id,code) and calls the
consumer. EL2.<clinit> explicitly maps literal mex to0x102. The previously
retained0v7 registration uses that same code. Thus legacy notification type
mapping is now concretely bound to the MEX message code, not just inferred
from nearby string literals. Type-map exact matching differs from lowercased
operation lookup in the downstream MEX router.

**Negative result:**1rR.A01, reached in a different1rN.A05 branch, processes
message-style Enx/F0Q/JID/peer fields. It is not the1rc notification branch.
Its apparent sender checks do not establish MEX notification authority. Full
method retained to preserve this false lead and avoid mixing shared class
branches in later audits. No JID allowlist is imported from message handling.

Remaining: exact map entry producer, shared handler preflight,0Ji consumer
selection, full alternate-registry fallback/gating and upstream sender authority.
The0ZJ.A0z reader-construction candidate was inspected and locates the1rh
constructor but its full assembly is not claimed reconstructed by this pass.
No Rust runtime change, ACK expansion or live operation was performed.

Reproduce apk-smali1rh.<init>,1rA.A03,1rN.A05,EL2.<clinit>,whole1rc
and1rR.A01 with signatures/exceptions. Existing1s2 fallback export carries
the104a call site, and router-registration.txt carries0x102 registration.

- [legacy-reader-layout.txt](legacy-reader-layout.txt) — SHA-256 `da060c1a7b6fc6729d81db716ffb5da9b278d6f797ee3cd6cc4cc6247951603c`.
- [legacy-handler-dispatch.txt](legacy-handler-dispatch.txt) — SHA-256 `28e18ba55fd1c87e6b8974285009ec71c52304bbcb3de606da8ff2259279c638`.
- [legacy-notification-routing.txt](legacy-notification-routing.txt) — SHA-256 `de79d68032919efc6a0ee7bd3a6c2aa16b28878fa6a42af337cf6780b167b8a1`.
- [legacy-notification-codes.txt](legacy-notification-codes.txt) — SHA-256 `0b4d28e2cc291529803ea15fcc136e0824cd05a6c01458c8627c68abc1bf467a`.
- [legacy-notification-constructor.txt](legacy-notification-constructor.txt) — SHA-256 `4e527236b06320d8fed6565d84a7a3ae98345ab17b2b7907b46a9739138e0903`.
- [message-parser-false-lead.txt](message-parser-false-lead.txt) — SHA-256 `4dc282b9a5112c3906f95873200911338fdb0a2031182e7e5f67a217d2d4a160`.

## High-offline preflight and handler-map construction (2026-11-04)

**Confirmed static**, same pinned APK.1rA.A03 invokes its A01 preflight
only when parsed offline count>=11, then uses2FP.A09(classification) to
select a callback path.2FP.A09 returns true for integer5/6/7/8 only.
The notification1rc branch of A01 records offline count/type and reads
optional from/participant as Jid, prefers participant when nonnull, obtains
DeviceJid via0EB.A00, and delegates2FP.A06 with final false. A06 forwards
Exz.A01 result into2FP.A07. Full classifier/helper semantics remain open.
This is not a universal check on every MEX sender: its high-offline gate
must be preserved and classification values must not be guessed as trusted
server identities. Missing native optional JID reads also do not authorize
Rust to accept malformed ACK routing fields.

0ZJ.A0S builds a new map, constructs1r7 factory context, iterates configured
1r8 factories calling AL1, and inserts each handler through A19. Optional
295 handler is conditional. A19 enumerates1rA.A02 declared tags and throws
on duplicate map entries;1rc's A02 branch declares exactly notification.
This closes tag registration/insertion mechanics, not all configured factory
implementations or consumer origin authority.

Negative lookup:1rc.A01 has no own bytecode body; the shared1rA method's
instance branch supplies this behavior. Do not infer lack of validation
from an exact subclass-method search alone. No Rust policy changes or
live requests were made. Sender authority remains upstream/unproven.

Reproduce apk-smali1rA.A01/A02,2FP.A06/A09 and0ZJ.A0S/A19 with
signatures/exceptions. Earlier1rA.A03 export carries offline>=11 call gate.

- [shared-offline-preflight.txt](shared-offline-preflight.txt) — SHA-256 `513741f4230527d4ea1fdb7f151610961c962bb737e0d66c61742dfebd966d19`.
- [offline-device-classification.txt](offline-device-classification.txt) — SHA-256 `48798dc44ff0ae7162ad90fea7c54cfa0498c6dfb2d2390bb6fc4b02f012f354`.
- [offline-class-result.txt](offline-class-result.txt) — SHA-256 `ea259733126a6084de5e78521c48f9a2412bbda191e65a0163a89371b73975eb`.
- [handler-map-construction.txt](handler-map-construction.txt) — SHA-256 `96e64014c4f4238bedb03128387ba114741e636618245ba4c95d343a974e73bb`.
- [handler-map-insertion.txt](handler-map-insertion.txt) — SHA-256 `5acde23d6f188328557a56d9593cfc9bfa191da4960a5f2f1fe2a6012b3bca89`.
- [handler-tag-declaration.txt](handler-tag-declaration.txt) — SHA-256 `f1a9e0f0cfd210e978f8fc8f86167bf812a5196456de4c30939a2065a3954648`.

## Offline device classification table (2026-11-04)

**Confirmed static**, same pinned APK.2FP.A07 takes nullable DeviceJid,
09F account source and two booleans. The following is **derived from the
retained branch instructions**, not observed traffic or trust labels.

| Condition in order | Result |
| --- | --- |
| first boolean true, second true | 7 |
| first boolean true, second false | 8 |
| first false, DeviceJid null | null |
| BRq(device) true and EpS.A00 true | 5 |
| BRq(device) true and EpS.A00 false | 3 |
| BRq false, device number0, BVC(user) true | 1 |
| BRq false, device number0, BVC(user) false | 2 |
| BRq false, device number nonzero, EpS.A00 true | 6 |
| BRq false, device number nonzero, EpS.A00 false | 4 |

A06 supplies first boolean from Exz.A01 and passes its final boolean
argument. The notification preflight supplies final false, so result7
is not selected by that specific call. Exz.A01 compares device.userJid
to Exz.A00 constant; its initializer constructs a fixed platform identity
and primary device. This is static protocol identity, not a user credential
or a value Rust should invent for account identity. Semantic authority of
that constant and account BRq/BVC implementations remain untraced.

EpS.A00 is exactly instance-of Dv4 or Dv6. No human-readable trust
category is inferred from obfuscated subtype names.0EB.A00 preserves
DeviceJid, otherwise converts recognized UserJid to primary device,
otherwise null. A server-only/non-user JID can therefore fall outside
this classifier; null classification is not itself a native rejection.

Combined with existing2FP.A09, only results5..8 choose the noted callback
path when offline>=11. Other cases continue through different branches
in1rA.A03. This is classification/routing evidence, not a universal
MEX sender-authorization requirement. No Rust policy or live traffic changed.

Reproduce apk-smali2FP.A07,Exz.A01/<clinit>,EpS.A00,0EB.A00 with
signatures/exceptions. Existing A06/A09 and1rA exports bind the caller.

- [device-classifier.txt](device-classifier.txt) — SHA-256 `726f9c8a002ec0c1857450c4fdf01cb905f704eb655341331177c72bc56359d8`.
- [device-special-identity.txt](device-special-identity.txt) — SHA-256 `12428568891cb7100533cb13285a67a4e310d436c9c03195645642756cd922fa`.
- [device-conversion.txt](device-conversion.txt) — SHA-256 `c52a92cc1a5b7016342ea43399be0a0ece0ea1002c83c64e1cc9214905f86bef`.
- [device-class-subtypes.txt](device-class-subtypes.txt) — SHA-256 `7597c7dd10407ea30e135478a3897158104f2eea651df1b6120f6faef7f3e730`.
- [device-special-constant.txt](device-special-constant.txt) — SHA-256 `452365061c96023e11a6a34eadc80246ca05fc53011faa3d4814288b44e1ba1a`.

## JSON adapter and Java decimal expiry (2026-11-04)

**Confirmed static, APK 2.26.43.72:** 7jz.A00 selects the first update
child and wraps its raw A01 byte body in 7p0. 8AF.A02 selects RsF.A00
as 8GA then its 8sX data. 8AF.A05 maps 7MU to the generated
NotificationUserReachoutTimelockUpdateResponse class; its constructor
only delegates the nonnull 8sX to 8GA. This does not establish missing
boolean defaults or the complete JSON decoder grammar. Rust's strict
recognized-body refusal remains a local integrity policy.

Existing 6oe.A03 evidence calls Java Long.parseLong. **Source correction,
offline tested:** active expiry strings now reuse the existing Java-style
BMP decimal reader, accepting leading plus and BMP decimal digits, while
rejecting whitespace, radix prefixes, underscores, supplementary digits
and overflow. JSON i64 input remains local compatibility; the nonnegative
local bound remains. Explicit inactive still clears without parsing expiry.
The shared decimal table is platform-reference material, not a newly
recovered APK digit table or a claim across every Android version.
A real-store regression checks accepted strings and preservation of an
existing restriction after rejected values. No live traffic was sent.

**Confirmed static / unknown semantics:** 7k9.A00 maps nullable 7gL enum
ordinals 1..20 to 02X integer constants, defaulting null/unhandled to
02X.A00. Human-readable variant names and numeric constant meanings
remain unresolved; this is not a complete enforcement-policy port.

Reproduce with xtask apk-smali on the pinned APK, selecting 7jz.A00,
8AF.A02/A05, the generated response constructor and 7k9.A00 with
--signatures --exceptions. These are checked DEX text exports, not REA
Evidence envelopes, and do not increase the REA coverage count.

- [mex-json-body-adapter.txt](mex-json-body-adapter.txt) — SHA-256 `597740bbd705e137d459346ce25bbc6814da871b434f4c497258ead8ba688580`.
- [mex-response-data-selection.txt](mex-response-data-selection.txt) — SHA-256 `ac8cb2e8d9a10af1053ddf504aa8667384c897f164e889274101a8c729346d02`.
- [mex-response-class-map.txt](mex-response-class-map.txt) — SHA-256 `65c5417c5936ae5b3fa6a826332981d2b83b40914a3ecf5c9e1ab27f8796e9ad`.
- [reachout-generated-response.txt](reachout-generated-response.txt) — SHA-256 `a47a0e99db505b9f2042cd4865b24070bbd217b0590676f4f0efef2cef203bd2`.
- [reachout-enforcement-enum-map.txt](reachout-enforcement-enum-map.txt) — SHA-256 `7c666080755e8280d1858f195a5ddfd3a8d99d41d92671b7572b0ea14fc0671e`.

## Enforcement names and stored type mapping (2026-11-04)

**Confirmed static / derived table**, same pinned APK SHA-256 above.
This resolves the previous section's unknown enum names and integer values,
without establishing complete enforcement behavior. 7gL.<clinit> supplies
the names/ordinals; its constructor sets serverValue to that name and
toString returns serverValue. 7k9.A00 maps ordinals to boxed 02X fields;
02X.<clinit> resolves their integer values. The already retained 7vL.A00
then separately maps internal 0..18 identically, all other values to 20,
before writing TOwmL_type. Do not use the enum ordinal as a saved type.

| Server enum name | Ordinal | Internal integer | Saved TOwmL_type |
| --- | ---: | ---: | ---: |
| `UNSET_OR_UNRECOGNIZED_ENUM_VALUE` | 0 | 0 | 0 |
| `BIZ_COMMERCE_VIOLATION_ADULT` | 1 | 3 | 3 |
| `BIZ_COMMERCE_VIOLATION_ALCOHOL` | 2 | 4 | 4 |
| `BIZ_COMMERCE_VIOLATION_ANIMALS` | 3 | 5 | 5 |
| `BIZ_COMMERCE_VIOLATION_BODY_PARTS_FLUIDS` | 4 | 6 | 6 |
| `BIZ_COMMERCE_VIOLATION_DATING` | 5 | 7 | 7 |
| `BIZ_COMMERCE_VIOLATION_DIGITAL_SERVICES_PRODUCTS` | 6 | 8 | 8 |
| `BIZ_COMMERCE_VIOLATION_DRUGS` | 7 | 9 | 9 |
| `BIZ_COMMERCE_VIOLATION_DRUGS_ONLY_OTC` | 8 | 10 | 10 |
| `BIZ_COMMERCE_VIOLATION_GAMBLING` | 9 | 11 | 11 |
| `BIZ_COMMERCE_VIOLATION_HEALTHCARE` | 10 | 12 | 12 |
| `BIZ_COMMERCE_VIOLATION_REAL_FAKE_CURRENCY` | 11 | 13 | 13 |
| `BIZ_COMMERCE_VIOLATION_SUPPLEMENTS` | 12 | 14 | 14 |
| `BIZ_COMMERCE_VIOLATION_TOBACCO` | 13 | 15 | 15 |
| `BIZ_COMMERCE_VIOLATION_VIOLENT_CONTENT` | 14 | 16 | 16 |
| `BIZ_COMMERCE_VIOLATION_WEAPONS` | 15 | 17 | 17 |
| `BIZ_QUALITY` | 16 | 1 | 1 |
| `BULK_MESSAGING` | 17 | 2 | 2 |
| `DEFAULT` | 18 | 0 | 0 |
| `RESTRICT_ALL_COMPANIONS` | 19 | 19 | 20 |
| `SCAM` | 20 | 18 | 18 |
| `WEB_COMPANION_ONLY` | 21 | 0 | 0 |

Null and unhandled enum ordinals use internal 0 in 7k9.A00. In particular,
WEB_COMPANION_ONLY exists in 7gL but ordinal 21 falls through the mapping
to 0; its name is not evidence of a separate implemented policy here.
RESTRICT_ALL_COMPANIONS maps 19 -> saved 20. The saved integer is a local
APK representation, not proven to be a numeric wire field.

**Bounded consumer trace:** exact TOwmL_type xref returns seven references
in six methods. 7vL.A00 accounts for two writes (active/reset). 7lA.A00
reads with default 0, selects string resource 0x7f123caf for saved 20
versus 0x7f123cae otherwise, then configures a WDSBanner. Resource wording
was not resolved. 6pn.A04 reads with default 0 and discards the result:
this method alone establishes no send gate. The other xref methods
(2Ne.A0G, ReachoutTimelockInfoBottomSheet.A2A, 8EG.onClick) remain untraced.
No complete sender, companion, or outreach policy follows from this table.
Rust still retains active/expiry only, and its group-create guard does not
cover every outbound operation. No request or runtime behavior changed.

**Negative search retained:** exact guessed anchor
reachout_timelock_enforcement_type had no references; actual preference
anchor TOwmL_type supplied the bounded result above. This is not an
exhaustive proof of absence through indirect preference access.

Reproduce xtask apk-smali 7gL (whole class), 02X.<clinit>, 6pn.A04 and
7lA.A00 with --signatures --exceptions; apk-xref --anchor TOwmL_type.
The full existing state-manager.txt retains 7vL.A00 and its switch.
Checked DEX TXT exports below are not REA Evidence envelopes.

- [reachout-enforcement-enum-values.txt](reachout-enforcement-enum-values.txt) — SHA-256 `2fe756cf89a0f2d1d26a24092d17c47649d9866ff50608d100bcb949285b6c56`.
- [boxed-integer-constants.txt](boxed-integer-constants.txt) — SHA-256 `502b9ccb95499cf04ed2cb7b8b0fc6aab932ab042bdcb035a85e5abe9032658f`.
- [reachout-type-preference-xref.txt](reachout-type-preference-xref.txt) — SHA-256 `3c21c0af6bb58b222150a9692455d89a4b6e85bb9c95598535bc6ad3b0204ce5`.
- [reachout-type-consumer.txt](reachout-type-consumer.txt) — SHA-256 `087b7d71fe3c08fefd6ad4f9a9e7fb169050fbbd81ee9d02dc152dd2ae6d9d22`.
- [reachout-type-presenter.txt](reachout-type-presenter.txt) — SHA-256 `48da7c0c3cb4e87fbb6c1484360906f1c4c38d24cd7011d0b68ec112c9c7934a`.

## Outreach decision chain and recipient set (2026-11-04)

**Confirmed static**, pinned APK 2.26.43.72 (same SHA-256 above).
6po.A01 caches the conjunction of TOwmL_is_active and config 0x1f48
in an AtomicReference<Boolean>; A02 lazily initializes/reads this cache.
Neither method reads end time. A03 is A02 AND TOwmL_is_visible. A00
invokes 0ht.A0U only when A02 is true. This does not establish native
expiry scheduling or authorize omitting expiry in Rust: scheduler, cache
invalidation, account-switch and config lifetimes remain open.

6pn.A05 and static A02 delegate the active predicate. A03 constructs an
information bottom sheet, requiring active; when not visible it invokes
7uB.A00 and emits a 7It event before constructing the sheet. Event semantics
and runtime necessity are not established, and no synthetic telemetry was
implemented. A04's stored-type read remains discarded.

### JID-record predicate 6pn.A06

A06(record, require_visible) returns false at each of these exits:

- Record is null, or 6po.A02 is false.
- Record.A0A JID fails 0Dn.A0k (null or getType outside 0,7,11,18,22,26).
- Recipient belongs to 0ht.A09's set.
- Record.A0D.A0J exists and 11w.A01 is true; this checks nonnull
  A07 string startsWith("ent:"). Producer meaning is not yet traced.
- Account 09F.BVC(recipient) is true, via 6oe.A1W; concrete implementation
  and semantics remain unresolved.
- require_visible is true but 6po.A03 is false.
- Final 0x9.A03(recipient) is true.

Otherwise it returns true. It does not read the saved enforcement type
or expiry directly. Final 0x9.A03 tests obfuscated subtype 0xC or
0x9.A06; A06 tests PhoneUserJid and delegates its user string to A0A.
A0A parses through 12J.A00 and compares two fixed numeric ranges via
12L.A00. Exact constants are retained, but full parser/comparator and
subtype semantics remain untraced. These branches are protocol research,
not instructions to manufacture exempt identities or message metadata.

### Set provenance and bounded caller interpretation

0ht.A0U iterates keys of 0hv.A0P, includes 0Dn.A0k matches, adds the
UserJid and any nonnull 0Ws.A0K counterpart, then sets A0C=true. It
adds to the existing set; this method does not clear it. 0hv.A0P selects
jid, incoming_tc_token, incoming_tc_token_timestamp from wa_trusted_contacts,
parses UserJid, calls A06 to construct each value, and returns the map.
The A0U loop uses keys, not token bytes or timestamps. Do not conflate
this membership with Rust held_tctoken's eligibility or invent token
validity/expiry filtering from this trace. Row insertion/removal, A06 value
construction, initial set and account lifecycle remain open.

Caller census: 6po.A02 has 11 direct callers, A03 has seven; 6pn.A06
has 12 call sites in 11 methods. Lists below retain exact DEX offsets.
2Q4.A00 reaches 6pn.A06(record,false) at byte offset 0x1c6 after
several other predicates; if true, it returns true immediately. Its
complete method is retained. This establishes participation in a higher
JID eligibility decision, not universal send refusal: the meaning of that method's
return and its callers remains open. Other A06 callers are not yet traced.

**Rust comparison / remaining work:** notifications persists active/ends,
and groups.queue_create checks active+unexpired before reservation. That
guard is based on an earlier observed group-create refusal, not a port of
6pn.A06. Other outbound paths do not yet enforce this native decision
chain. No blanket block, feature-config value, exemption or native-cache
lifetime was guessed here. Full outbound coverage remains unfinished.
No Rust code changed and no live request was made.

Reproduce apk-smali with --signatures --exceptions for the methods named
by each export; apk-callers 6po.A02/A03 and 6pn.A06. Exports are checked
DEX text, not REA Evidence envelopes or runtime observations.

- [reachout-policy-manager.txt](reachout-policy-manager.txt) — SHA-256 `7f5e2393edec6563718cf199b93fa87879e29999a1c06700e30c6cba34cbe12c`.
- [reachout-policy-A02-callers.txt](reachout-policy-A02-callers.txt) — SHA-256 `8d186d8f828c09fc0b36d456fb6460c4bfb9195fae1f6dcb2e856c465775664a`.
- [reachout-policy-A03-callers.txt](reachout-policy-A03-callers.txt) — SHA-256 `1604f93e506c4094feb25c1a8c03147a7a55c5c4701f1d3156bc3c92a357da06`.
- [reachout-outreach-policy.txt](reachout-outreach-policy.txt) — SHA-256 `a1ef1c12199d0615f90c3766f225703718a5e6e48a3d286931fe2d10faa3cf80`.
- [reachout-recipient-kind.txt](reachout-recipient-kind.txt) — SHA-256 `15ac20b5755091ceb6fa5e8645b00557e528b2d7821e65e46c50130d8a622e6b`.
- [reachout-message-exemption.txt](reachout-message-exemption.txt) — SHA-256 `579c4d6d01101ea5d84e89965fd982135005f9f8540abeb869880901c1aabd47`.
- [reachout-final-jid-predicate.txt](reachout-final-jid-predicate.txt) — SHA-256 `d1097e16b8d95a1a2a92dde0b90c4d0a7571c9e602ced288f5be06b2306536aa`.
- [reachout-exemption-set-refresh.txt](reachout-exemption-set-refresh.txt) — SHA-256 `ebe98f9d3006cb21eb32f72e536c93313fb27862493e6953b00f04a0124c8548`.
- [reachout-outreach-callers.txt](reachout-outreach-callers.txt) — SHA-256 `7f0e2628a0163788af3b97adf4edb88cd8a2fa4ac9e0e610c6523781f9c688a9`.
- [reachout-exemption-map-source.txt](reachout-exemption-map-source.txt) — SHA-256 `a226c53e8d6a9fb252d07bcedff6eba720838813729b3efde86f044797fc9f76`.
- [reachout-final-jid-fallback.txt](reachout-final-jid-fallback.txt) — SHA-256 `b246720c1dd82b48c80c4addaba5662c9d69e8ca4916e25ace67f3b060ce68c7`.
- [reachout-outreach-caller-2Q4.txt](reachout-outreach-caller-2Q4.txt) — SHA-256 `7d8797b3284538985dbb0e519c14a150c97638e817f14b4316c265d9ae7d007c`.
- [reachout-final-user-predicate.txt](reachout-final-user-predicate.txt) — SHA-256 `cc8634770df9c10188211e2fc67c7d0ae5f42ad4c18a3154a5b92996a1fba998`.
- [reachout-message-property.txt](reachout-message-property.txt) — SHA-256 `247eac8e1189ebdbd10b982161295a957d4f187443015f2c1a6e232e2127434b`.

## Caller semantics: call gate and sticker refusal (2026-11-04)

**Confirmed static**, same pinned APK 2.26.43.72 SHA-256 above. This
resolves part of the previous higher-caller uncertainty and corrects an
overstatement about input semantics.

**Refuted prior shorthand:** 6pn.A06's 0wj argument was described as a
message, and its nested ent: field as message metadata. That was stronger
than the evidence. 2Q4.A01 accepts a JID, obtains 0wj via 0cR.A02, and
calls A00(record,JID). 0cR.A02 delegates to 0bD.A0C and A07; 0bD.A0C
looks up A09(JID), otherwise constructs new 0wj(JID), loads it through
0wc, inserts/binds it and returns it. 0wj.A0A returns its 11T.A0K JID.
This is a JID-bound cached record, not a decoded outbound message payload.
The nested ent: property's producer remains open; it must not be treated
as an outbound serialization instruction. Existing predicate exits remain
valid with the corrected record interpretation.

### Confirmed caller behavior

- **Outgoing-call preflight, 1v7.A02:** when group argument is null and
  6pn.A05 active is true, a list with more than one record enters the
  restriction branch. A single record enters it if A06(record,false) is
  true; an empty list skips it. When a UI delegate exists, the branch
  opens the information sheet, records reachout_timelock_bottom_sheet,
  returns 29x.A06, and skips later app/startOutgoingCall processing. If
  the delegate is absent, it logs through 1ey then falls through. Thus
  even this leaf is not a universal unconditional wire-send refusal.
- **Sticker sending, ShareMediaViewModel.A01:** its sendSticker branch
  invokes 2Q4.A01(destination). True logs cannot-share/read-only-chat,
  calls A03 with reason read-only chat, and routes to completion instead
  of the normal path beginning at byte 0x138. This establishes a native
  sticker send refusal that can be reached through 2Q4.A00 -> 6pn.A06.
  Other predicates in 2Q4.A00 can also cause true; read-only chat is the
  caller's aggregate description, not proof of which restriction fired.
- **Eligibility inversion, Jmt.A00:** for its selected user-JID branch,
  a missing cached record returns false; otherwise it returns NOT
  2Q4.A00(record,JID). Group handling uses separate checks. The two
  Jmt callers KI9.A00 and KIB.A01 are listed, not yet interpreted.
- **2Q4.A01 null JID:** returns true without lookup; nonnull goes through
  the JID-record lookup above. It is not a network fetch.
- **2Pp.A0y:** requires A00 false plus additional account/JID/config
  conditions before true. **2TX.A00:** A01 true sets A02=false and
  returns for either selected group/recipient path. These booleans are
  not independently proven wire obligations.
- **UI consumers:** IbM.A2d uses A00/A01 to select action branches and
  schedules forward_button_visibility_check. 3e5.run selector 37 uses
  A00 in the 2Mz UI branch; true takes an earlier UI path. Full methods
  are retained because merged classes have unrelated cases. No inference
  that every case sends messages was made.

The direct 2Q4.A00 census has five call sites in five methods; A01 has
16 sites in 13 methods (IbM has two, 2TX two, MediaViewMenu two). These
are static direct call sites, not runtime frequencies or proof that no
indirect calls exist. This pass traced the named methods above; other
A01 consumers and the send execution boundary remain open.

**Rust implication:** native policy demonstrably affects calls and sticker
sending as well as UI eligibility. Rust's existing group-create-only guard
is incomplete coverage, but blindly blocking every send on active state
would discard native recipient/config branches. Before porting, resolve
the account predicate 09F.BVC, record-property producer, set/cache/expiry
lifetimes, representation/account binding, and applicable send callers.
No invented feature config, token exception or wire field was added.
No runtime behavior changed, no live account traffic was sent, and full
protocol correctness remains unfinished.

Reproduce apk-callers 2Q4.A00/A01 and Jmt.A00, plus apk-smali for the
class/method in each export with --signatures --exceptions. The full
checked DEX exports below are not REA Evidence envelopes.

- [reachout-decision-2Q4-callers.txt](reachout-decision-2Q4-callers.txt) — SHA-256 `dc4903684243db46e9ac79885674cbfcebfa5955a9811fc3dd72d9ecb802c478`.
- [reachout-caller-1v7.txt](reachout-caller-1v7.txt) — SHA-256 `6d850155f6b3cbe253e8e6793b3c7b2d7d780f60d7248083c55cc6af9694935d`.
- [reachout-decision-message-wrapper.txt](reachout-decision-message-wrapper.txt) — SHA-256 `22bf5ea144616a49a2ce60815f8e07019e1669b5d2784bc506261c05dd39af01`.
- [reachout-decision-Jmt.txt](reachout-decision-Jmt.txt) — SHA-256 `d41734b7365e4afb056c7f4842c58eca844fcffdcc814aa2f1d43aed6f829f51`.
- [reachout-decision-3e5.txt](reachout-decision-3e5.txt) — SHA-256 `8fbb57adb6f14c138e312fb39221b2df17f58ab10c4e5fd182a0ecd9e1a72abb`.
- [reachout-decision-2Pp.txt](reachout-decision-2Pp.txt) — SHA-256 `a5a485584174b43b1563cf44365bdb8dcd1da00cf65d7ddf5c9ec67de43ffc6c`.
- [reachout-decision-IbM.txt](reachout-decision-IbM.txt) — SHA-256 `be0f999095fa2df66b88fafec3d929ffa6961b57f10d7ae4bf7f1654ea6fdd47`.
- [reachout-decision-A01-callers.txt](reachout-decision-A01-callers.txt) — SHA-256 `bcfed9c363fbc6acb9aa3544e79e5b90bb7cd8ee08cd65323077622c602f0532`.
- [reachout-decision-Jmt-callers.txt](reachout-decision-Jmt-callers.txt) — SHA-256 `35d82bec660739b8dd4e381a01381b9e4c068aae1638a5a7890583ebf443c1fd`.
- [reachout-media-share-decision.txt](reachout-media-share-decision.txt) — SHA-256 `f28702d7f25bdcdca08aef4f97af357f3b7a332398bf03730edfd7ff9336be1e`.
- [reachout-decision-2TX.txt](reachout-decision-2TX.txt) — SHA-256 `50556dd4bfb2107f4a454794bf10ac2341078df85ff0f6fbb7cccddffd55fa88`.
- [reachout-decision-record-lookup.txt](reachout-decision-record-lookup.txt) — SHA-256 `3b74afed50ad658cfc3d65d76c56ffbf93657e9fd486d4d710f1d15d9fd59cb0`.
- [reachout-decision-record-jid.txt](reachout-decision-record-jid.txt) — SHA-256 `c8e8a7128a6d865a44fcfba19d3d61031934d66fa7ba328cd06ba0da40702581`.
- [reachout-decision-record-cache.txt](reachout-decision-record-cache.txt) — SHA-256 `6d2ec4026c031f6319143e3f5a3a4d23c49e3128d75506005ba97a04194205ac`.
- [reachout-decision-record-binding.txt](reachout-decision-record-binding.txt) — SHA-256 `a2961a4cfe3df8af733528e68a9b4aefbb645dd6fd43f4b55e292b2ba856c4ed`.

## Deadline is a refresh trigger, not an inactive update (2026-11-04)

**Confirmed static**, pinned APK 2.26.43.72, SHA-256
887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e.
This closes the specific visible countdown path, not every lifecycle.

Exact TOwmL_end_time_in_ms xref has three direct references: two writes
in 7vL.A00 and one read in ReachoutTimelockInfoBottomSheet.A2A. The
sheet computes end minus 08q time, builds a 6qK CountDownTimer through
8l1.invoke when positive, and invokes 7uq.A00 when already elapsed.
6qK.onFinish sets the UI progress to zero and invokes the same 7uq.A00.
Neither expiry path writes inactive state.

7uq.A00 schedules 8bO selector 19 (literal 0x13). In 8bO.run that
selector constructs FetchReachoutTimelockQuery, response class 74W,
using 6oh.A0D(...,false), binds 0if, and registers 8dV selector 17.
That callback binds selector 18 success and 19 failure. Success reads
xwa2_fetch_account_reachout_timelock, marks is_reachout_data_fetched,
extracts is_active/time_enforcement_ends/enforcement_type and invokes
7vL.A00. Failure logs/reports and returns false without the success
manager call. Complete merged methods are retained to distinguish selectors.
Transport/query serialization, scheduler overlap/retry policy and default
GraphQL accessor semantics remain open; no Rust query was guessed.

**Cache refresh:** exact 6po.A01 direct caller list contains lazy A02,
two 7vL.A00 sites, and 8Mp.BjM. BjM compares current config 0x1f48
with its saved boolean and invokes A01 when unequal. Configuration
observer registration/lifetime and account teardown remain unresolved.
Exact TOwmL_is_active xref has five sites: 6po.A01 reader, picker A3B
reader, 7uB.A00 assertion reader, and two 7vL.A00 writes. Picker A3B
reads active with default false under config 0x1f48. 7uB.A00 is a
visibility operation, not expiry: it asserts active, writes visible=true,
and schedules 8bO selector 18 to notify 7Ky listeners. Exact-string xrefs
do not exclude indirect accesses or establish all deletion/reset paths.

**Confirmed Rust defect / offline correction:** queue_create used the
inspection helper active_reachout_timelock(now), which filters out elapsed
deadlines. That let local time alone bypass an existing active group-create
guard. It now uses a separate recorded-active helper; the displayed end
is diagnostic and does not release the guard. The public deadline-filtered
inspection helper retains its behavior and now explicitly documents that
None is not operation authorization. No checkpoint schema or data changes.

A real encrypted-store regression saves active=true with end=1, verifies
create refuses without any queued/reserved action and preserves active,
then applies an explicit inactive notification and verifies one create
can be queued. Existing notification validation-before-write remains.
This is a correction to the existing local group guard, not proof that
native group creation has identical checks or that all outreach is ported.
Rust has no automatic FetchReachoutTimelockQuery refresh in this change:
a stale active record can continue refusing until an explicit inactive
update is received. Do not delete/reset it or fabricate release state.
No live traffic or server acceptance was observed.

Reproduce apk-xref with each preference anchor, apk-callers 6po.A01,
and apk-smali for class/method named by each export with --signatures
--exceptions. These checked DEX TXT exports are not REA Evidence envelopes.

- [reachout-expiry-lifecycle.txt](reachout-expiry-lifecycle.txt) — SHA-256 `a75afeb70b0f375275b697898d586d366e35daf72a8c4f19ea8d3bc9f048f5b6`.
- [reachout-end-time-callers.txt](reachout-end-time-callers.txt) — SHA-256 `26d41a70a16e5d0c0a3700c21596736af81e6c70b54d0f1bb41d03e1b23ed783`.
- [reachout-active-pref-callers.txt](reachout-active-pref-callers.txt) — SHA-256 `f8f737dd76acd15db5ba5e6498c2480ddc60331dc9bb0195b3ef12f62d271229`.
- [reachout-cache-refresh-callers.txt](reachout-cache-refresh-callers.txt) — SHA-256 `00f4b4806408c68cd53a9020f8b35a632a5d01ad8fa0f9a7d8026d424285b285`.
- [reachout-end-time-ui-reader.txt](reachout-end-time-ui-reader.txt) — SHA-256 `23602a0f1d2111da9687cfce3b4955711005108d0ae07b8ee3a4b8f18faca452`.
- [reachout-expired-ui-action.txt](reachout-expired-ui-action.txt) — SHA-256 `d47683aae8ef29782ac188f6c37546a592c0dd5883ee08fca934786fe4ba0a6b`.
- [reachout-cache-refresh-observer.txt](reachout-cache-refresh-observer.txt) — SHA-256 `d8ab739f576df26d411fa02745ea7dec89530a2f406901270b51d6e821008719`.
- [reachout-ui-timer-factory.txt](reachout-ui-timer-factory.txt) — SHA-256 `040a5b91b92d6f2f9e469da69a5568feaf211e49988f1325ca81c41cdf568b62`.
- [reachout-expiry-worker.txt](reachout-expiry-worker.txt) — SHA-256 `457401d1d15073eaea91e84ab5142660adf7cedb3a21c1a1e3eadecc57560e49`.
- [reachout-countdown-finish.txt](reachout-countdown-finish.txt) — SHA-256 `8e33aa059e552e36a2ca9122257479ef627b7222c7dde6c47e34dd433e8a09de`.
- [reachout-fetch-callback.txt](reachout-fetch-callback.txt) — SHA-256 `dc633fc12677ac1de29ad32c9cb37e90fea09b8027b9398ae9c7f526e699f732`.
- [reachout-active-picker-reader.txt](reachout-active-picker-reader.txt) — SHA-256 `bb8b8b7e1477475b44ac59012323bbfb1b6b6b76a695f5464e43897dc7d5b8ab`.

## Refresh request construction and runtime document lookup (2026-11-04)

**Confirmed static**, same pinned APK 2.26.43.72 SHA-256 above. This
extends the countdown/fetch chain, without adding a Rust dispatcher.

8bO.run selector 19 constructs a fresh 0i1 via 3q0.A0X and calls
6oh.A0D(params,74W.class,"FetchReachoutTimelockQuery",false). The helper
constructs 0iT with call name, namespace whatsapp-android-mex, null Pando
model class and null optional function. 74W is a JSONObject-backed 0iM
response wrapper, with no field grammar in its constructor. The operation
name has one direct string reference in this APK; this is not a count
of all runtime invocations.

Fresh 0i1 constructs GraphQlCallInput through 0iG. The input has null Nha
content; Axc -> GraphQlCallInput.A03 creates a TreeMap, and private A01
returns immediately for null content. The traced caller adds no variables,
so this request's variables map is empty. 0iT initializes its extra map
as empty, returns false for hasAcsToken/hasOhaiConfig, and throws for
setNetworkTimeoutSeconds/setUseSafeStack. Those are this spec's methods,
not universal authentication requirements for all GraphQL.

### Start, transport selection and XMPP structure

2LA.A0Y casts the supplied executor to 0ig and calls A01, which creates
0iq with the spec/dependencies. 6oj.A0Y obtains it via DI key 0x1138;
full DI registration remains open. A search for 0if.<init> found no method
body; this is a bounded negative, not a diagnosis of a missing runtime.

0iq's constructor sets its long timeout argument to 0x7d00 (32000); no
unit conclusion is needed here. AQY builds 0id, invokes callback setup,
then AQV -> static A00 -> returned 0j2.A04. The fresh 0iq has default
A03=false, selecting 0j3 rather than RsH at this first layer. 0j3 has
no declared A04 body in the search; its constructor shows superclass 0j2,
whose A04 is retained. A04 checks the resolved document identity, invokes
callback failure with LI6 if absent, and only otherwise calls A06.

0j3.A06 performs another selector through 0io.A02. This caller's 0iT
has null A03 model class, so it returns 0mO.A0B, whose initializer names
REJECTED_MEX_NOT_PANDO_CAPABLE and passes false for isPando. This is a
transport-selection rejection, not a server account refusal. Therefore
the selected branch builds 0mb response handling and calls 0Sf.A0W
with message code 0x155 and the retained timeout argument. No Pando
configuration/allowlist value was invented to reach the branch.

0mF constructs JSON with queryId and variables from Axc, then 0mK builds
iq to the singleton 0mL (empty user, getServer s.whatsapp.net), xmlns
w:mex, type get, id, optional trace and query. 0mI creates trace/flow_id
from the document ID. query_id uses that same document ID; JSON is the
query body. Constructor validation conditions and order are retained in
full. Semantic shape for a nonnull ID resolved to the asset reference:

```xml
<iq to="s.whatsapp.net" xmlns="w:mex" type="get" id="{fresh-id}">
  <trace><flow_id>25627902213558773</flow_id></trace>
  <query query_id="25627902213558773">{"queryId":"25627902213558773","variables":{}}</query>
</iq>
```

This is a derived semantic illustration, not captured traffic or proof of
JSON byte ordering, wire encoding, current registry availability or server
acceptance. No input/operation-name field was added to variables by guess.

### Asset reference versus runtime registry

The bounded apk-mex-queries export records server doc_id 25627902213558773,
client_doc_id 289769408314545818597242103356, schema whatsapp_mex, namespace
whatsapp-android-mex, and provenance of the exact persisted-ID asset. They
are distinct identifiers. The export was reproduced with --check including
APK/asset hashes. It supplies release-specific reference data, not a
completed port of runtime registry resolution.

0j2.A03 calls 0ih.A00(callName,namespace), which asks 0ik.A00 for a
0il and invokes CRh. 0ik looks in a ConcurrentHashMap and otherwise
returns 5vM(namespace); 5vM.CRh returns null. 0ih catches exceptions and
also returns null. Thus the fallback does not automatically turn an asset
entry into a request. Runtime map population/implementation/teardown is
not traced in this pass; no claim that this exact asset record is always
the live registry result follows. Treating the asset ID as unconditional
dispatch authority would skip a native availability boundary.

**Remaining obligations:** registry initialization/provenance, full IQ
response/error conversion and sender/correlation checks, current account
request ownership, atomic restriction application with action settlement,
overlap and retry/wait behavior, and teardown. The existing notification
handler cannot simply be reused by relabeling an IQ result. Automatic
refresh remains unported; existing guards and uncertain outcomes stay
unchanged. No Rust runtime change, no live account traffic, and no claim
of mandatory startup polling or completed network audit.

Reproduce apk-mex-queries --operation FetchReachoutTimelockQuery and --check
against the JSON below. Reproduce apk-smali for each named class/method
with --signatures --exceptions; apk-xref --anchor FetchReachoutTimelockQuery.
TXT exports are checked DEX text, not REA Evidence envelopes.

- [reachout-fetch-response-class.txt](reachout-fetch-response-class.txt) — SHA-256 `a81c09214e5d3ac6e1806637a4300b65ded9eebc426d62db0de292f9bb02e6e8`.
- [reachout-query-constructor.txt](reachout-query-constructor.txt) — SHA-256 `ae3a0321a7f031c468ca1696bf4f2e629f96ce1d163e0523ba97b5c9176f11cd`.
- [reachout-fetch-query-anchors.txt](reachout-fetch-query-anchors.txt) — SHA-256 `11addb2f8c53f4097f6f877500c86ae7af5eda8ce34f0c7ffbf5baab55271b6a`.
- [reachout-query-spec.txt](reachout-query-spec.txt) — SHA-256 `81d971c079924305485f324099c4428ec2643eafe68f82e31996f1969787734f`.
- [reachout-query-params-factory.txt](reachout-query-params-factory.txt) — SHA-256 `9bd75a2416023f14a1c54f234d1a89d2af7a90a0bcb63d691e588ca93e6198bc`.
- [reachout-query-params-constructor.txt](reachout-query-params-constructor.txt) — SHA-256 `8251f43760df49b815b86f40c32b1892dd1ca7924dc12fa6a521f88bd3147ea4`.
- [reachout-query-execution-adapter.txt](reachout-query-execution-adapter.txt) — SHA-256 `e225e5f8cb5a763ce389c9b9224f30d4ee43d6806efe6a1f3c6cc552d161fa3b`.
- [reachout-executor-factory.txt](reachout-executor-factory.txt) — SHA-256 `d8fddd1fc83f2a8a165b287d8b7989b85451957346a2a636934e4cb955375042`.
- [reachout-query-submit.txt](reachout-query-submit.txt) — SHA-256 `f07dc9882fd13038aef5f91410eb523869bcff8ccab7bc8b142f5fa21808018a`.
- [reachout-query-start.txt](reachout-query-start.txt) — SHA-256 `b8989017d166942cf0fcddb3cb16cf627297dfaade6283e290a9265dd027b352`.
- [reachout-query-operation-constructor.txt](reachout-query-operation-constructor.txt) — SHA-256 `e4193e9c56bf0c15a9b5f50a59a8d9c5ff07fe28909f9a752bfff27f2fca9e78`.
- [reachout-query-transport-selection.txt](reachout-query-transport-selection.txt) — SHA-256 `343aef400cc13a14887ff37c5b43ad671f7c07c39ff64735630841da46d504d4`.
- [reachout-query-transport-plan.txt](reachout-query-transport-plan.txt) — SHA-256 `958e3a9fba47f9e858f85d5d27baa86e969cd25bdb9b293e6de28fc445087784`.
- [reachout-query-base-start.txt](reachout-query-base-start.txt) — SHA-256 `ab26206db889ef779ca3ac255b83e3732c1489a90445d4e25528520ff8544e49`.
- [reachout-query-default-constructor.txt](reachout-query-default-constructor.txt) — SHA-256 `ffec9d98af676e2b9d089390f9fa1e542bc3c50fa713d2e99fa238bc7de8890b`.
- [reachout-query-xmpp-send.txt](reachout-query-xmpp-send.txt) — SHA-256 `acd3dcf6e75f3d93df7222d2764718ab85e463b2ab5569a8558b658421564d7b`.
- [reachout-query-variables.txt](reachout-query-variables.txt) — SHA-256 `5a16a1796f4014d588b5e360ffb050a7bd4dccef26f88c251dbfbed530989d00`.
- [reachout-query-xmpp-plan.txt](reachout-query-xmpp-plan.txt) — SHA-256 `62d7f02952d398737fe575d321b6ca7fae7ac225264bb81832196471e92ac02e`.
- [reachout-query-xmpp-node.txt](reachout-query-xmpp-node.txt) — SHA-256 `2d592b6269ef1ed6dc3c931428388f83c66e2260f89c0b5e835cb75550b1f74f`.
- [reachout-query-variable-map.txt](reachout-query-variable-map.txt) — SHA-256 `e36dd5752da39ee4d5da5d1c0c991c96711e363966648b00bfbab03874e2ccc0`.
- [reachout-query-pando-selection.txt](reachout-query-pando-selection.txt) — SHA-256 `7c3b9e07c0976b0e86a9a2a68e8f43e3643fef5ad494aea62fbfc67fb66eebaa`.
- [reachout-query-transport-enum.txt](reachout-query-transport-enum.txt) — SHA-256 `df0c9017baeec3907cfdb4eaf7b8fb0367b7d35d8d2b244786f5c1cf2c99d515`.
- [reachout-empty-input-constructor.txt](reachout-empty-input-constructor.txt) — SHA-256 `c62ca3cb1c1a5fcc7ce6dd67d62cb3cc1c8a719b160c5dc6054690b976221154`.
- [reachout-query-empty-map.txt](reachout-query-empty-map.txt) — SHA-256 `be7f352006e7b4f0f1031e4ded586a17e32011c0d6dce4362f4af2f5ad21689d`.
- [reachout-query-trace-node.txt](reachout-query-trace-node.txt) — SHA-256 `02916dbb651271097c9d487edc9af7885ba6b238789325776f4c174af895d508`.
- [reachout-query-server-jid.txt](reachout-query-server-jid.txt) — SHA-256 `980fe80b7482094daa3b7da1b6664efd2789018c46a6792ce436d99c8beaf33f`.
- [reachout-query-server-constructor.txt](reachout-query-server-constructor.txt) — SHA-256 `89b8f6102b053f434f2e2ff2111f82788ae0cd649ac1d5657bbbea01cce38851`.
- [reachout-empty-map-walk.txt](reachout-empty-map-walk.txt) — SHA-256 `d988264b82ccb34f3ff854f28d43fae26aa46cf74a66d4b73e3d7c2a2c0322be`.
- [reachout-query-server-domain.txt](reachout-query-server-domain.txt) — SHA-256 `2876e9fac7544fdd0191992747d9102c7a7ff4f014dd24ff4bca793fa0bec523`.
- [reachout-query-document-lookup.txt](reachout-query-document-lookup.txt) — SHA-256 `1f8e0b493b537ed2598f966dc687ddaff2f3e9fada601ac96863f3aefe5e7673`.
- [reachout-query-build-registry.txt](reachout-query-build-registry.txt) — SHA-256 `c93d83f50b7007138ea3728caca6303b987d24be1aa1292979b92911b5d6bd85`.
- [reachout-query-persisted-id-reader.txt](reachout-query-persisted-id-reader.txt) — SHA-256 `bf880f036691c5b9a3681d405feff3803576d652464c2f903fa8f68ee713f307`.
- [2.26.43.72-reachout-query-ids.json](../../apk/2.26.43.72-reachout-query-ids.json) — SHA-256 `180be85a07a6b6288aec031480aee82baeb94565cf0bffa98de866124907db8b`.

## Executor registration and asset-backed server-ID lookup (2026-11-04)

**Confirmed static**, same pinned APK 2.26.43.72 SHA-256 above. This
resolves the previous section's missing normal executor-registration
chain; runtime execution and every alternative registrant remain unproven.

0ig.<init> obtains 0ih under DI key 0x1139, stores it as A09, obtains
0ii under 0x113a, and registers three build namespaces through 0ik.A01.
The MEX registration uses 0ij.A03.buildConfigName and 1Wy selector 22
(literal 0x16). 1Wy.invoke selector 22 calls 0ii.A00(0ij.A03).
0ij.A03 is WHATSAPP_ANDROID_MEX, namespace whatsapp-android-mex and
jsonPath whatsapp-android-mex_client_persist_ids.json. The export named
asset-loader.txt is the full 0ij enum declaration, not the I/O implementation.

0ii.A00 first checks a ConcurrentHashMap keyed by 0ij. If absent, it
constructs 0im with application context, namespace, jsonPath and two
client-ID gating flags, then uses putIfAbsent and returns the winning
existing/new provider. 0im initializes a lazy JSON value through 0in.
0in.invoke opens that named asset through AssetManager, allocates using
available(), reads into it, closes the stream, decodes UTF-8 and builds
JSONObject. This is static asset input, not an HTTP endpoint.

0im.CRh(operation) gets that JSONObject, obtains the operation object,
and returns getString("doc_id"). JSONException/IOException return an
empty string. Thus in this registered provider the reference server ID
25627902213558773 is connected to the native runtime lookup, without
substituting client_doc_id 289769408314545818597242103356. As already
retained, 0j2.A03 rejects empty results and 0j2.A04 takes local failure
before send when document identity is absent. Successful asset loading
is not proof of request acceptance.

**Client IDs remain separate:** 0im.AHH applies its client-ID gate flag
and config 0x5cae, optionally a flatbuffer flag/config 0x6285 and Mjc
reader, otherwise reads client_doc_id from the JSON operation. CRh does
not call AHH or inspect those flags. No config value is invented; the
full method is retained without claiming all client-ID use sites traced.

### Global registration scope and unresolved alternatives

0ik.<clinit> initializes its global ConcurrentHashMap. A01 checks
containsKey(namespace); only if absent does it invoke the supplier and
put the returned provider. The pair is not an atomic putIfAbsent, unlike
0ii's provider cache. Sequential registrations preserve an existing key;
concurrent ordering is not established. The exact A00 field census has
three accesses: initialization, lookup and registration. This does not
prove there are no indirect mutations or runtime resets.

The A01 direct caller census has 12 sites in five methods: 0ig.<init>
(three), MjY.<init> (three), Mja.<init> (one), MrW.Bk4 (four), N5D.invoke
(one). Mja.<init> also registers literal whatsapp-android-mex with N5J
selector 42 and the same DI key 0x113a. Its constructor is retained,
but that alternate supplier and other registrant lifetimes remain
untraced. Do not claim 0ig is the sole global registrant or that this
pass determines which one wins every startup sequence. The exact A09
write census has one write in 0ig.<init>, tying its lookup dependency.

**Implementation consequence:** the normal executor's static registration
and asset-backed CRh chain is now established, so that specific research
gap should not be repeated. Remaining work is alternative registration/
teardown and full correlated IQ result/error handling, durable request
ownership/settlement with restriction application, and refresh timing.
Rust automatic refresh remains unported. No feature config, extra query,
wire field, or new dispatch policy was guessed. No Rust runtime changes
or live traffic were made.

Reproduce apk-field-refs 0ik.A00 ConcurrentHashMap and 0ig.A09 0ih
(writes-only), apk-callers 0ik.A01, exact asset-name apk-xref, and
apk-smali on named class/method exports with --signatures --exceptions.
These are checked DEX TXT exports, not REA Evidence envelopes. Existing
bounded APK/asset query-ID export remains the reference data artifact.

- [reachout-registry-field-refs.txt](reachout-registry-field-refs.txt) — SHA-256 `4a39f305c3bf51e398254388f6f58de8a55a683af50025ca8a51f0ec6ec00036`.
- [reachout-registry-asset-anchor.txt](reachout-registry-asset-anchor.txt) — SHA-256 `e24dabcdd8d8e3c92e1b4d4b738456c41ee7c77502585d5eb60b2765d6b53078`.
- [reachout-registry-registration-callers.txt](reachout-registry-registration-callers.txt) — SHA-256 `6eeeaae8726466e3b027a8278817349c2e8a55d77383f77f3ae647289de8fe52`.
- [reachout-registry-full.txt](reachout-registry-full.txt) — SHA-256 `808f15a34a25c21b1c6d9f16fee12d1191e7527152a1971f693210a971c35813`.
- [reachout-registry-asset-loader.txt](reachout-registry-asset-loader.txt) — SHA-256 `60bc2fe9f51acf94c647993e8b32e5e5a136aa7e37237790f29c0ca670fab964`.
- [reachout-executor-registry-init.txt](reachout-executor-registry-init.txt) — SHA-256 `d925846a0de7919d503cc4d60eedad4c22925957879f163fc48d4ec1d71ca8d0`.
- [reachout-registry-supplier.txt](reachout-registry-supplier.txt) — SHA-256 `6be11a728048c9d454fddfe65aa1aaa473674a0e5e2222c01907b2e1fbc1ec63`.
- [reachout-registry-load-plan.txt](reachout-registry-load-plan.txt) — SHA-256 `9d93cb0042df94b7a2d00afd36479beb4efdb13ee5c366e382474311e5e87485`.
- [reachout-registry-json-reader.txt](reachout-registry-json-reader.txt) — SHA-256 `6430bae3b54bdd82335bb1d242f1292d3fe931540eccb5acdbb0f0e96322e088`.
- [reachout-registry-asset-read.txt](reachout-registry-asset-read.txt) — SHA-256 `004fb18eed85c30206fbddf5cdc68e041e57858deff6a5c745881c800247781d`.
- [reachout-registry-lookup-owner.txt](reachout-registry-lookup-owner.txt) — SHA-256 `c9de850401cd2390c8b5e75aca89d5e44d3e62ec31c492d42e262c3f348d522a`.
- [reachout-registry-alternate-owner.txt](reachout-registry-alternate-owner.txt) — SHA-256 `853572a45d8698420461c507670d222c5fc92f6d6916b0c0ed923141bc76ae95`.

## IQ response conversion and refresh callback admission (2026-11-04)

**Confirmed static**, pinned APK 2.26.43.72 with the same SHA-256 above.
This traces the refresh's selected XMPP response conversion and callback
admission; no Rust fetch/response application is implemented here.

0mb delegates incoming success/error entry points to 0md; timeout BsS
delegates to 0md.BsS -> 0mY.BL6. 0md.A00 tries three request-bound
constructors in order: 7Ou(node,original0mK) success -> BKn; on LIL,
7Ow(node,original0mK) request error -> BKl; on LIL, 7Ol(node,original0mK)
server error -> BKo. If all fail, it throws LIL with the accumulated
shape errors. Complete constructors are retained; merged class overloads
for unrelated notification/media shapes are not refresh response evidence.

The success overload names iq, result, id, to, Argo|Json and result
body paths and passes fields from original 0mK into 8B6 validators.
This establishes request-bound shape validation, not a completed proof
of every 8B6 argument/default/equality rule or upstream sender authority.
In particular, the literal mex in a validator argument must not be
misread as an asserted iq type=mex requirement. The generic matcher,
pending-request lookup, socket/account correlation and sender ownership
remain obligations to trace before porting.

### Body conversion before typed callback

7jy.A00 extracts success A02's 7NM byte body. Its selector 0 maps to
7p0 format integer 1; the other branch maps 0; absent wrapper throws.
Which wire alternative supplies each selector still needs its constructor
trace; do not assume all MEX results are raw JSON or all are Argo.

0mZ.BKn obtains that wrapper, then invokes 0is.A03 using the request's
tree class/call name/build namespace and optional converter. This refresh
spec's optional function is null, so A03 takes A04 plus T4f typed
conversion, not the optional custom converter branch. A04 selects 0iw
for format 1 and 0iy/7wg byte input for other values. 0iy labels its
pipeline JSON; it composes 0iv with a DI-provided 0it parser and three
1WU converters. 0iv distinguishes parse failures (including OOM),
data/error conversion and composite results. These exports preserve the
remaining decoder boundary; exact bytes/UTF-8/top-level field grammar
are not yet reconstructed from these shared methods alone.

0is.A01/A02 convert a successful SUg data/errors pair using TXW and
construct RsF; conversion exceptions become explicit 7MI failure values.
T4f uses the requested model's JSONObject constructor for the 0iM
branch, while other model classes can use an 8sX wrapper. The already
retained 74W JSONObject constructor is the refresh model. RsF.A02 for
a 0iM is true only when its backing JSONObject length is positive;
RsF.A03 is true when the error JSONArray length is positive. This is
not field-level validation that the timelock object is present/valid.

0mZ.BKn forwards a successfully converted RsF through CDQ even when
its error flag is set; its metrics calls are not state application. A
conversion failure takes explicit failure handling instead. BKl/BKo
construct distinct request/server error wrappers, and BL6 handles timeout.
No automatic replay or new identity is justified by these error branches.

### The refresh error callback prevents applying partial data

BaseMexCallback.CDQ -> A04 implements admission after conversion:

- With errors, it calls A08(Hbw(errors)). A false return ends without
  A05, even if the converted data object is nonempty.
- With no errors, A02 data-present permits A05. Without data, default
  A07=false declines A05. 0id does not override A07.
- A05 exceptions are wrapped/reported and rethrown, not rewritten into
  successful inactive state.

0id.A05 invokes its configured success function; A08 invokes its error
function and unboxes the boolean. In the previously retained refresh
8dV selectors 17/18/19, selector 17 binds those callbacks, selector 19
logs/reports and returns false, and selector 18 obtains the timelock
object and calls 7vL.A00. Therefore this refresh's GraphQL errors
prevent partial data reaching that state-manager callback. An empty
converted data object also does not reach it. Nonempty data still
requires the already noted field accessor/default behavior; a JSON
wrapper is not proof that missing is_active must mean inactive.

**Port obligations retained:** do not clear restrictions merely from IQ
type=result, generic action settlement, parse success, errors plus data,
empty model or elapsed deadline. A future durable port must bind the
original request/account, validate the selected body and supported fields,
retain refusal/failure reasons, and compose accepted state application
with its action settlement. Native preference writes are not proof of
a Rust transaction or replay-safe scheduler. Current notification ACK
policy is separate and is not silently changed from fetch evidence.
Full decoder grammar, field defaults, request correlation, lifecycle and
automatic refresh remain unfinished. No runtime change/live traffic.

Reproduce apk-smali on the named class/method in each export with
--signatures --exceptions. Select the 7Ou/7Ow/7Ol overload whose second
argument is 0mK. TXT exports are checked DEX text, not REA envelopes.

- [reachout-iq-response-adapter.txt](reachout-iq-response-adapter.txt) — SHA-256 `0257321e14266bc9770ecbf6efe63c5091f7fac0bbf804d131ec1ddc22360af8`.
- [reachout-iq-json-conversion.txt](reachout-iq-json-conversion.txt) — SHA-256 `52dbdc3c3107b3f47b5f070245e85fdedf386f9ec26d189ce9103de97125b810`.
- [reachout-iq-result-wrapper.txt](reachout-iq-result-wrapper.txt) — SHA-256 `29c7c21c5fbb6f9df0b9e413939f77b110852ffa3f963a04a392929cd0d2b81d`.
- [reachout-iq-body-selection.txt](reachout-iq-body-selection.txt) — SHA-256 `692416847360959e79c1fce4c548549f065001e6223ea0fd511c0890837cad33`.
- [reachout-iq-json-parser.txt](reachout-iq-json-parser.txt) — SHA-256 `47f223d01a043dab13fb19a82c35dbf3b23d9f59d499fbca9f6f35d0594ab6f6`.
- [reachout-iq-success-shape.txt](reachout-iq-success-shape.txt) — SHA-256 `8a2569401e9890eebe527b5d35d33c1dbe42a022ed4e75f7afe0322624f04ff0`.
- [reachout-iq-parser-selection.txt](reachout-iq-parser-selection.txt) — SHA-256 `cf9a916db313b0babc7ebdc18696d13edba8c36d863e2aa802bb3b1e4166908d`.
- [reachout-iq-json-decoder.txt](reachout-iq-json-decoder.txt) — SHA-256 `5beab4841f477cb29b37d844731fbb682f35234a96f447f769da1c24440f18aa`.
- [reachout-iq-decoder-failures.txt](reachout-iq-decoder-failures.txt) — SHA-256 `5b974ace178794c01c2eac5104795ecfbf21c0b0704a4b7c1f2893ad0bbf73dd`.
- [reachout-iq-callback-disposition.txt](reachout-iq-callback-disposition.txt) — SHA-256 `91150a717f000f46ce6b63d1fa1c7ea9c2570089263809dd4fdf78b96c4f54d2`.
- [reachout-iq-base-callback.txt](reachout-iq-base-callback.txt) — SHA-256 `07eaa3df9e7339dc4b1e8ae02258447b2cc02e7bd587b588a7e0c6a0f1fa63d6`.
- [reachout-iq-callback-data-gate.txt](reachout-iq-callback-data-gate.txt) — SHA-256 `bf39447aec03ecad37bbca3000685380ce7af3520e36965c2901c92462f63a04`.
- [reachout-iq-typed-result.txt](reachout-iq-typed-result.txt) — SHA-256 `e04c18a9d9290b57335336d3cdb852574e270d228e6c0ad4ba586c2394fa7d28`.
- [reachout-iq-empty-data-policy.txt](reachout-iq-empty-data-policy.txt) — SHA-256 `acd2384892c873df24b2bd40de0a4644690911b977ba0e11b4061c387af4b138`.
- [reachout-iq-typed-data-construction.txt](reachout-iq-typed-data-construction.txt) — SHA-256 `427a9c8615bbf18d2e6fc4c7b84c2f2f947fe9bd2d7117a66d4186baac5cf8a3`.
- [reachout-iq-request-error-shape.txt](reachout-iq-request-error-shape.txt) — SHA-256 `d3311ff90469721ff21e1ce3a6a7a777af9b4513e9d5e8d49928b0c04458744b`.
- [reachout-iq-server-error-shape.txt](reachout-iq-server-error-shape.txt) — SHA-256 `b9a2725fd499957b7255b31f14161cc3322281766abdeb198fd0f4fdbbd06f71`.
- [reachout-iq-data-error-flags.txt](reachout-iq-data-error-flags.txt) — SHA-256 `8b949e4fb1f67ec1b32c33b8e5ef72789729a63a47ddb000bd25c29ac227b50d`.
- [reachout-iq-json-model-converter.txt](reachout-iq-json-model-converter.txt) — SHA-256 `6a739847d95514faeaa64a0b28af4fad308891c27c5b1c6760c432773a82a7fe`.

## Fetch field defaults and body-format selectors (2026-11-04)

**Confirmed static**, same pinned APK 2.26.43.72 SHA-256 above. This
resolves specific fetch accessor and format-selector unknowns from the
previous section; no notification coercion parity is inferred.

### Fetch object and scalar accessors

74W and nested 74V constructors delegate nonnull JSONObject to 0iM.
0iM.<init> assigns that same object to A00/A01. A04(class,field) uses
optJSONObject; absent/nonobject returns null. Otherwise it reflects the
requested JSONObject constructor. Listed reflection failures are saved
to static A02 and return null. A05 calls A04 and throws IllegalStateException
with Required value was null when it returns null. Thus the previously
retained refresh 8dV(18) requires the nested
xwa2_fetch_account_reachout_timelock object before setting fetched and
applying state. An empty object can still be constructed; object presence
is not field-level validation.

0iM.A0G(field) reads optInt(field), sets default=(value==1), and returns
optBoolean(field,default). This confirms a coercing boolean accessor,
not a strict required bool. In particular, a missing field follows the
JSONObject false-default behavior; the fetch's missing is_active is not
a required-value exception in this accessor. Full numeric/string Android
coercion vectors are not newly established by this trace.

0iM.A0E(field) returns null when JSONObject.isNull(field), otherwise
optString(field). In the refresh callback this supplies
time_enforcement_ends to 7vL.A00, whose active/inactive duration handling
is already retained. Do not replace it with a mandatory numeric JSON
field, or treat accessor success as a valid active expiry.

**Scope correction:** prior fetch field defaults were open; this accessor
chain now resolves them for the JSONObject-based fetch model. The push
notification path uses 8sX typed B48/AwY accessors, not these 0iM methods.
It remains invalid to silently change Rust notification strictness on the
basis of fetch accessors. Rust has no refresh applier here. A future
faithful fetch port must account for native defaults and clearly identify
any stronger local preservation policy rather than claiming parity.

### Argo and JSON variant mapping

7Ou's request-bound success constructor supplies 8TV selectors 37 and
38 to 8B6.A0K for its result alternative. Initial search for 8TV.invoke
found no body; the actual interface method is ABJ, retained with its
merged dispatch table and constructors. Unrelated cases are not evidence
for refresh body handling.

- 8TV.ABJ selector 37 at byte 0x936 inspects result with the format
  literal argo and constructs 7NM selector 0.
- Selector 38 at byte 0x8b6 inspects result with format literal json
  and constructs 7NM selector 1. The helper 1Xl.A1Y returns constant
  true after null checks, closing the latter selector value.
- 7NM.<init> stores its selector unchanged in $t and stores byte body.
- Previously retained 7jy.A00 maps 7NM selector 0 to 7p0 format 1
  (Argo), and selector 1 to 7p0 format 0 (JSON). 0is.A04 selects 0iw
  for format 1 and the JSON 0iy pipeline for format 0.

These are distinct selector domains: 8TV case, 7NM variant, and 7p0
parser format. Do not equate their integers. The JSON branch's generic
format-validation call differs from the Argo branch (return handling/
required flag); complete 8B6 semantics and missing-format acceptance
remain untraced. Likewise this does not establish all raw-body limits,
Argo decoding, JSON top-level extraction or malformed-byte handling.
No raw JSON fallback for an unrecognized Argo body is justified.

**Remaining implementation work:** complete decoder/shape validators,
account/request correlation, native alternative-registry lifetimes and
durable fetch/apply/settlement. Current group reservation and notification
guards remain unchanged. No new network operation, schema change,
synthetic server state or live request. Overall network audit unfinished.

Reproduce apk-smali with --signatures --exceptions on 0iM.A04/A05/
A0E/A0G/<init>, 74V.<init>, 8TV (ABJ/constructors), 1Xl.A1Y and
7NM.<init>. Existing 7Ou/7jy/0is exports connect the format chain.
TXT files are checked DEX text, not REA Evidence envelopes.

- [reachout-fetch-bool-default.txt](reachout-fetch-bool-default.txt) — SHA-256 `8beafa165cbd2c38f83e7aac88c1bc064943e14c587e0f4cd36ca99f00591274`.
- [reachout-fetch-object-accessor.txt](reachout-fetch-object-accessor.txt) — SHA-256 `e3ba61a25f584e6fe7f1063262ac70285edbc9a3fe2445205ff070ecfa631940`.
- [reachout-result-alternative-adapters.txt](reachout-result-alternative-adapters.txt) — SHA-256 `b60c95eb04fd04651bd8cce78c37f6413d4b4ed65fa700489a118f07a45fa056`.
- [reachout-fetch-object-converter.txt](reachout-fetch-object-converter.txt) — SHA-256 `e60a4ac371e78dfec0e83ebbcce55e197e70e25fdc798dcc5a87126185de43bc`.
- [reachout-fetch-model-constructor.txt](reachout-fetch-model-constructor.txt) — SHA-256 `30e0b2ae7c619b1ba63856b4589e111183124a0b1ee2a32b9a0442cf964bede5`.
- [reachout-fetch-string-default.txt](reachout-fetch-string-default.txt) — SHA-256 `ed46f34fe8ffc8e1f1e12234a6046dfd09dbff39303d4a6645f74b68ae2e8850`.
- [reachout-alternative-selector-constant.txt](reachout-alternative-selector-constant.txt) — SHA-256 `5796b66eba71c974c1a2feb760d0a5cee2bdc24ef72a105bbc0ac014d39b1a55`.
- [reachout-fetch-timelock-model.txt](reachout-fetch-timelock-model.txt) — SHA-256 `6386cd8e24fa1faf198e79b3720b16ef07248f07057d1d3871bf36d026b7e02e`.
- [reachout-result-body-wrapper.txt](reachout-result-body-wrapper.txt) — SHA-256 `3e888d5e424d757b02ebe9bc5b82d6633f131fae994ad8e15bc5446aad755150`.
