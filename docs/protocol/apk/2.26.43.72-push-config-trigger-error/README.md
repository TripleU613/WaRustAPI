# Push-config GET startup order and error bridge — APK 2.26.43.72

**Confirmed static**, checked DEX TXT, not runtime observations or REA Evidence
envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-callers APK --class LX/1rj; --method A02` and
`xtask apk-smali APK --class CLASS [--method METHOD] --signatures --exceptions`.
Selected methods are complete except the explicitly labelled startup excerpt,
which matches the [full retained source](../../rea/2.26.43.72-authkey-outcomes/startup-trigger-checked.txt).
Only1rv branches of the shared callback methods are interpreted here.

## Direct caller and local ordering

Exact direct caller inventory finds one caller of1rj.A02, in0ZJ.A0z. This is a
DEX direct-reference result, not a proof against reflection or every indirect
runtime trigger. In the selected block:

1. 0b0c calls1rj.A05 and saves its boolean.
2. 0b24 calls1jA.A00, the previously traced pending-token getAndSet(null)/drain.
3. If that saved boolean is true,0b3e calls19i.A00, the conditional rotation path.
4. 0b7c invokes1rj.A02, which registers and sends the push-config GET.
5. 0b86 calls0Sb.A02 and, if it returns a node,0b96 passes that node to1rj.A03.

The boolean at0b2a branches around the rotation call, landing on the config
fetch block; it does not skip config fetch in this local control flow. There is
no config-result wait betweenA02 and the nextA02/A03 node-producing sequence
in this block. This does not prove the config response completes before later
startup tasks, nor that every possible login/resume path reaches this block.
Full upstream branch eligibility and0Sb's node meaning remain open. Token drain
and rotation effects retain their [earlier scoped evidence](../2.26.43.72-client-auth-token-readiness/README.md).

## Lower-level errors consume ownership before decoding codes

The [IQ reader](../2.26.43.72-iq-reader-admission/processor.txt) handles an error
by removing its lower-level callback entry first. For ordinary1rv it iterates
named error children, reads code and calls Integer.parseInt followed by1ru.A01.
Missing codes skip that child; an invalid integer can throw, with no local catch
table. Multiple code-bearing children can callA01 repeatedly even though there
was just one map removal. Do not infer a single failure notification per response
from map removal alone. A malformed code can prevent later forwarding; this
chunk does not invent a recovery/reinsert path for that case.

1rv declares only its constructor and extends1ru.1ru.A01's1rv branch calls
writer.A07(1lZ).A0o(code). A0o logs client_config_error, buildsA06(code) and
submitsCPm. A06 constructs Message.what=0, arg1=27, arg2=code. This differs
from successful config projection's type6 message. Base1ru.A00 is a no-op,
but its existence is not proof of a timeout schedule or that it is invoked for
this request; timeout/cancellation trigger ownership remains untraced here.

## Selected type27 handler and404

The previously [retained15F.BLn](../2.26.43.72-push-config-projection/consumer.txt)
recognizes type27, reads arg2 and logs the code. Non404 returns without the
VERIFY helper in this selected branch.404 calls1uj.A04 with Context,0Gq and
all seven String arguments null. It then conditionally calls1uo.A00(null) if
1uo.A01 is true. Full injected handler membership and1uo effects remain open;
this describes what happens if the selected handler receives this message.

The previously [tracedA04](../2.26.43.72-push-config-verification-work/README.md)
builds VERIFY service work. With these nulls it omits the String extras and
uses account-count1 and VOIP0 parse fallbacks. Thus404 can request local
reconciliation despite providing no successful config values. This is not a
direct config-GET retry, proof of a corrective SET, cleared token, or successful
work enqueue/completion. Later provider/service gates still apply and enqueue
failures can return normally. No code409 SET policy is borrowed for this GET.

## Remaining work

Complete upstream startup gates,0Sb traffic,404 provider reconciliation,1uo
side effects, timeouts/cancellation, malformed-error handling outside this reader,
header validation, generation binding and durable recovery remain incomplete.
No Rust request, retry, route or live account operation is added. This evidence
cannot establish a server restriction cause or a zero-ban guarantee.
