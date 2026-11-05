# Push-config projection and selected consumer — APK 2.26.43.72

Confirmed static checked DEX TXT, not runtime observations or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `xtask apk-xref APK --anchor gcmToken`.
Complete selected methods are retained, including one method extracted from
the checked0ln class export. Every file is checksummed. Only the identified
config/type6 blocks of larger methods are interpreted.

## Eight internal values, without invented wire attribute names

The [result callback](../2.26.43.72-lower-level-iq-ownership/result-callback-dispatch.txt)
iterates parsed variants, passing84d with eight mutable holders. 84d's
constructor binds holder fields to the result's later2Dz argument order;
2Dz's constructor and1lZ.A0S then bind these to the following Bundle keys.
These are internal message keys, not proven XML/wire attribute names.

| Internal Bundle key | Selected7OZ visitor source |
| --- | --- |
| gcmToken |7NQ.A02 when7NQ.A04 equals gcm |
| fbnsToken |7NQ.A02 when kind equals fbns |
| admToken |7NQ.A02 when kind equals adm |
| appMuteConfig |7NQ.A00 Long converted to String, when kind=gcm and value nonnull |
| mutedChatsHash |7NL.A01 when the7NL object is present, on the shared continuation |
| num_acc |7NQ.A01 Long converted to String when nonnull, on shared continuation |
| pkey |7NQ.A03 String when nonnull, on shared continuation |
| voip_payload_type |7NQ.A05 String when nonnull, on shared continuation |

3q2.A0z explicitly uses Number.longValue followed by String.valueOf(long).
It does not establish wire numeric bounds or timing units. The visitor's
shared continuation is reached after gcm/fbns/adm token selection, and also
by the unmatched-kind branch; this does not prove the parser accepts arbitrary
kind values. The retained7OU/7OV/7OW/7OX/7OY A89 implementations only check
the visitor argument and return; no holder writes occur in those bodies.
This is a selected-variant observation, not a complete runtime variant census.

Iteration can assign the same holder repeatedly. The resulting projection
is per-field overwrite behavior, not necessarily one whole last record:
optional-null checks can leave an earlier holder value intact. No duplicate
wire acceptance/rejection or full collection ordering rule is inferred without
finishing the7Ow/7Nk/7NQ/7NL parsing chain.

## Internal dispatch preserves a separate configuration message

1lZ.A0S places all eight Strings in a Bundle carried by a Message with
arg1=6. 1uW.A00 returns arg1. 1lY.CPm tests0ln.A0G(type). If recognized,
A07 posts the Message to0m2 with what=1 and arg1=type. If not, CPm sets
what=0 and posts it to1lY, whose selected handler forwards through0ZJ.A0s
andA0q; type6 is not A0q's special4/87 branch, so it goes to0YS.CPn.
The retained0YU.CPn implementation posts its what=5 message.

A0G recognizes6 directly whenA0Q is true, otherwise consults registered
handler indices. 0ln.A00's direct type6 branch resolves SQ0.A03 and invokes
0Vw.BLn; its other path uses the registered-handler list. 15F.Amf advertises6
along with other message types, and15F.BLn has a matching type6 consumer.
Complete SQ0 provider identity, injected handler membership and every fallback
binding remain open. These are selected routing/consumer bodies, not proof
that every runtime configuration applies this handler.

## Selected type6 consumer is push-registration state

15F.BLn type6 requires its Message.obj to be a Bundle, reads the eight keys,
logs token lengths and pkey presence, then calls1uj.A04 with Context,0Gq and
seven Strings in this order: gcmToken, mutedChatsHash, appMuteConfig, num_acc,
pkey, voip_payload_type, admToken. fbnsToken is handled separately: when
1uo.A01 returns true, the consumer invokes1uo.A00(fbnsToken).
Missing values remain nullable in this selected body; it does not prove that
any wire field is required, safe to omit or safe to replace with empty text.

The selected keys and type6 body do not directly identify heartbeat intervals
or install a timer. Deeper1uj/1uo behavior can have further effects and is
untraced, so this is not a global statement that push configuration cannot
affect timing. Other error/message types in15F are not interpreted here.
Push configuration is distinct from client-auth-token storage/rotation; these
keys must not be substituted for the previously traced opaque cat token.

## Implementation boundary

Eight-field projection and selected consumer invocation are connected. Wire
schema/cardinality, header/helper parity, handler/provider binding, actual
registration side effects, persistence and failure/reconciliation policy
remain open. No Rust config sender, application, retry, route or live account
operation is added. Existing uncertainty and no-replay behavior remain unchanged.
