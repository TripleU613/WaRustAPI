# Client-auth-token trigger boundary — APK 2.26.43.72

Confirmed static checked DEX evidence, not runtime observations or REA Evidence
envelopes. APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`, `apk-callers APK --class LX/F2Y; --method A02`, and
`apk-field-refs APK --class LX/0AI; --field A04 --type I --writes-only`.
Exports preserve tool output, whose short signature labels are not Java signatures;
use the full signature comments. Checksums cover all retained files.

## Push argument and bounded provenance

GcmListenerService.A04(Bundle) reads map key `pn` at 01fa..0206 into v22.
The call at 03a8..03ae passes v11..v34 to F2Y.A02's v58..v81 parameters:
v22 therefore maps to v69. No intervening v22 rewrite appears in this export.
The previously retained [full push consumer](../../rea/2.26.43.72-authkey-outcomes/push-consumer-checked.txt)
labels v69 `pushNonce` in its diagnostic, copying it into v31 at 0134.
At 26fa it tests v69 for nonnull; 26fe..270c loads its token manager and
calls 1jA.A01 with v31. This supports the push-nonce interpretation of `pn`,
not an assumption that it denotes a phone number.

However, v31 is reused at 1d28 and 287e in other parts of this large method.
A complete control-flow proof that every path into the manager invocation
preserves the original nonce is still open. The nonnull guard and initial
argument mapping are confirmed; end-to-end manager-input identity is partial.
The full method's earlier branches also remain relevant to eligibility.
Framework delivery authentication, origin validation, and alternative push
producers have not been established by this selected GCM method.

Exact-class caller search finds five physical F2Y.A02 invocation sites across
GcmListenerService.A02/A04, EoH.A00 and Fei.run. This is not five distinct
runtime events or a complete interface/virtual dispatch census.

## Numeric gate, without inferred account eligibility

0AI.A0O returns exactly whether field A04 equals 2. The existing token-manager
trace defers input in its AtomicReference when this predicate is false.
The state-writer inventory has five sites: constructor, A0M, 0YU.handleMessage,
and 1iM.A02/A04. The retained 1iM.A02 writes 3, clears A05, closes a condition
variable and logs `server disconnected`; despite this export's filename
`connected-state.txt`, it is not evidence that A02 establishes connected state.
1iM.A04 writes 4. The producer of state 2, transition ordering and all
registration/account prerequisites remain untraced. Numeric equality alone
must not authorize a token request in Rust.

## Implementation boundary

No production sender, automatic challenge replay, route or live operation is
added. Rust uncertain holds remain. Next proof obligations are the consumer
control-flow join, trusted input provenance, state-2 producer and lifecycle,
failure/backoff, generation/account binding and durable recovery. This chunk
does not establish ban prevention or whole-app reverse-engineering coverage.
