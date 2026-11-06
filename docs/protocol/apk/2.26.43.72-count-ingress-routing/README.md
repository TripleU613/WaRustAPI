# Conditional count-notification ingress routing

**Confirmed static**, WhatsApp2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
New complete exports use xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions. Provider-type declaration census uses --class ''
--method Avt; constructor caller census targets JoW.<init>. These are DEX text,
not REA Evidence IDs. Only the selected paths below are interpreted.

## Reused current reader and registry evidence

[1s2.run](../2.26.43.72-count-content-error-ack/incoming-reader.txt),
052e..0774, handles notification routing. It lowercases type with Locale.ROOT,
looks up JoW.A01 and, only for an absent mex entry, consults the operation map.
J9D and Rqj kinds use separate branches gated by0x4632 and create tracked
Eza.A02/A01 wrappers respectively. They are not blanket count-handler policy.

The [existing KWt builder](../../rea/2.26.43.72-reachout-state/registry-builder.txt)
060c..071c builds maps from injected set0x1ddd. J9C registrations enter the
notification-type map; J9B registrations enter the MEX-operation map. Duplicate
normalized keys throw; mex is forbidden as a J9C registration. Newly retained
[JoW constructor](registry-constructor.txt) assigns its supplied maps directly.
Its exact constructor caller is [KWt.invoke](registry-constructor-callers.txt).
This closes map identity, not the effective injected set or all subclass paths.

[Ki2 contract](provider-contract.txt) exposes Avt and B51. The APK-wide physical
Avt declaration census finds interfaceKi2 and ten bodies in KL6/7/8,T3n..T3t.
Each body returns its stored descriptor. Complete constructors retained here
show notification names location,w:gp2,psa and operation names
NotificationScheduledMessagePost,NotificationScheduledMessageReveal,
AccountSyncUsernameNotification,LidChangeNotification,UsernameDeleteNotification,
UsernameSetNotification,UsernameUpdateNotification. None of these ten declares
encrypt. This is bounded negative evidence, not proof that injected registry
contents cannot include inherited/dynamic variants. No complete set claim.

## Absent-provider fallback can still be tracked

At06fa the reader checks whether the selected provider is null. A nonnull
other kind branches to104a (direct handler lookup). Null reaches config3992;
false branches to04da, leaving wrapper v8 null and later reaching the direct
handler path. True reaches071e and creates Eza.A03(1kz.A07, node,...,
!0ZP.A04(node)) at0768. Thus absence of a named encrypt provider does not prove
absence of tracked notification ingress.

[Tracked type enum](tracked-notification-types.txt),0264, includes encrypt.
[Eza.A03](tracked-wrapper-builder.txt) reads source/id/t/type/participant,
selects the matching E8e type and constructs1k4. Its
[constructor](tracked-wrapper-constructor.txt) extends1k3, whose retained
[BX2 predicate](../2.26.43.72-count-content-error-ack/incoming-kind-predicates.txt)
returns true. Malformed wrapper metadata can fail before the count parser;
this is not permission to synthesize missing fields.

## Persistence policy is separate from wrapper kind

[0ZP.A04](fallback-persistence-policy.txt) parses offline with default -1.
It obtains threshold viaA00(config3cd4,-1,100). For offline=-1 and returned
threshold=-1 it returns true directly. Otherwise offline>=threshold can set
receiver.A01=true, and the function returns that retained buffering flag.
Threshold effective value/clamping and clearing A01 are outside this trace.
The reader inverts this result for the wrapper Boolean; do not identify that
Boolean as proven durable persistence or readiness without its consumer trace.

[0ZP.A05](named-persistence-policy.txt) additionally examines the selected queue
backend,0av.A0J and1id.A0N before falling back toA04; it is not interchangeable
withA04. Its full bodies are retained as input for the next consumer trace.

## Result and unresolved work

The prior binary tracked/untracked uncertainty is now a concrete conditional
reader branch: unmatched provider +3992 true can yield tracked encrypt; false
reaches untracked/direct handling. Runtime configuration, full injected provider
membership, wrapper Boolean consumer, handler connection/queue ownership and
count-specific bulk ACK-list provenance remain open. Existing
[watchdog](../2.26.43.72-deferred-ack-watchdog/README.md) remains diagnostic;
this does not turn its6000ms interval into a send deadline.
No Rust route, config default, ACK/socket order, replay or account operation was
changed from these conditional facts. Static/offline only.
