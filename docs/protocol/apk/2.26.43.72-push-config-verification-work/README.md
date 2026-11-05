# Push-config verification work and numeric defaults — APK 2.26.43.72

Confirmed static checked DEX TXT, not runtime observations or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `xtask apk-callers APK --class CLASS --method METHOD`.
Complete selected methods are retained and checksummed; serviceA05/A06 are
extracted from the full checked class export. Only identified projection,
enqueue, comparison and send-invocation blocks are interpreted. Complete
Firebase/ADM lifecycle and side effects of all helper calls remain open.

## Config consumer queues verification work, not the corrective IQ

The previously traced [type6 consumer](../2.26.43.72-push-config-projection/consumer.txt)
calls1uj.A04 with seven Strings, with FBNS handled separately. A04 creates
an Intent for RegistrationIntentService with action com.whatsapp.action.VERIFY,
then calls0Gq.A00(Context, Intent, serviceClass,4). This method itself does
not build/send the config IQ. The meaning/lifecycle of the enqueuer's final
integer and its durable job guarantees are not reconstructed here.

The helper includes the following String extras only when input is nonnull
and String.length is nonzero. Whitespace is not explicitly trimmed.

| Consumer input | Verification Intent extra |
| --- | --- |
| gcmToken | registrationId |
| admToken | admRegistrationIdFromServer |
| mutedChatsHash | mutedChatsHash |
| appMuteConfig | appMuteConfig |
| pkey | pKeyHash |

Empty/missing Strings therefore become omitted extras in this bridge; that
is not proof of a server clear/delete instruction. The two numeric extras
are always put as ints: num_acc becomes numberOfAccountsFromServer with
fallback1; voip_payload_type becomes voipPayloadType with fallback0.
0HF.A00 returns its fallback for TextUtils.isEmpty, otherwise invokes
Integer.parseInt and returns the fallback on NumberFormatException. It has
no explicit trim, clamp or positivity check. These are Java integer parsing
semantics, not a new hard wire bound or a rule that all malformed config is
safe to accept. The preceding wire schema and its validation bounds must be
traced independently.

A04 catches selected enqueue-time exceptions, logs/reports the failure and
can return normally. Its exception table contains Exception/RuntimeException/
IllegalArgumentException coverage for the selected enqueue call; paths can
also rethrow. Reporting hooks themselves are not proven nonthrowing. Thus a
normal config callback return is not proof verification work was accepted,
completed, or that a corrective network request reached the server.

## Later service compares state under additional gates

RegistrationIntentService.A05 receives Intent, service, config object, two
Strings and two booleans. It immediately returns false if both booleans are
false. Their full GCM caller interpretation is not inferred here. In the
selected comparison paths it reads numberOfAccountsFromServer with default1
and compares it with local0Qq.A0A; mismatch sets its send-needed variable.
It reads voipPayloadType with default0 and compares remote configuration
0x1014; mismatch likewise sets that variable and logs the calling-push reason.
Mute configuration and pKey reconciliation add feature/helper-dependent paths;
their complete equality, key-rotation and persistence policy remain open.

If no send is needed it returns false. If a send is needed, the selected
final gate allows progress when0GQ.A08 is true OR0GQ.A02 equals2 OR0GS.A00
is at least7. Those numeric/method predicates are retained without assigning
unsupported account-status labels. When admitted it invokesEbv.A00 and
returns that boolean. That is not proof of server acceptance or all durable
request ownership. FullEbv construction, readiness, failure/backoff and
post-restart reconciliation remain untraced here.

## Concrete callers keep token comparison separate

The exactA05 caller inventory finds the service'sA06 and2Fn.onComplete.
A06 is an ADM path: it compares its supplied token against the Intent's
admRegistrationIdFromServer, passes provider string adm, first booleantrue
and second boolean=not equal intoA05, restoring Binder identity on normal
and exceptional exits. This does not authorize fabricating an ADM token.

The retained1uf.A0B RegistrationIntentService branch distinguishes VERIFY
and REFRESH, obtains platform token state, and can invokeA06 or launch the
GCM completion path. 2Fn.onComplete is retained as the other concrete caller;
its complete token/application-version/refresh flag derivation, task failures
and persistence ordering remain untraced. The helper and service do not
collapse server values, local provider tokens, and the opaque client-auth cat
value into one interchangeable identity.

## Implementation boundary

Verification Intent projection, numeric defaults and selected service gates
are connected. The enqueuer/provider lifecycle, complete comparisons, Ebv
sender and server outcomes, generation ownership and durable recovery remain
open. No Rust config sender, retry, route or live account operation is added.
Existing uncertainty and no-replay behavior remain unchanged.
