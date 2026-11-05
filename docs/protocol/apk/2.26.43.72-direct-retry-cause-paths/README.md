# Direct decrypt cause paths and coordinator correction

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked-DEX exports, not REA Evidence envelopes. No target execution/live traffic.

## Corrected coordinator identity

The earlier old-counter bundle labelled0WJ.A0P ordinary/direct message decrypt.
That label was wrong: its record/chain types and explicit diagnostic identify
SignalCoordinatorDefault/decryptForGroupUsingFastRatchet. Its existing complete
message-decrypt.txt export remains unchanged; its README is corrected.
Ordinary direct decrypt is0WJ.A0N, explicitly decryptForIndividual, invoking
F22.A05. Prekey direct decrypt is0WJ.A0O, invokingF22.A04. The retained76-method
signature inventory locates these methods without committing the whole class.

0WJ.A0N catches ED4 at code-unit00a3 -> byte0146, supplies -1001 at0148, and
passes it throughEHq.A00. F22.A05 callsF22.A01 -> F22.A02; the latter's known
old-counter ED4 is not caught byA01's EDY-only tables. Thus the ordinary direct
old-counter chain independently supports the existing typed no-retry change.
No behavior was changed to conceal the previous attribution error.

## Missing session and aggregate failures

F22.A05 loads the session, tests its existence, and constructs EDS with
“No session for: ” at0044..005c if absent. A0N catches EDS at code-unit0093,
byte0126, passes -1008 at0128. Previously retainedEtY/F2q/DxB tables admit
-1008 -> internal1 -> wire1. This confirms the selected direct missing-session
category, not a universal code for every missing/corrupt key.

F22.A01 tries current state viaA02 and, for ordinary message mode, older states.
Only EDY is caught in its two protected regions. Exhausted/skip-older paths
construct EDY(List); its already retained constructor supplies the explicit
outer message “No valid sessions.” with a nested cause. EHq passes that outer
message with -1005, andEtY refines it to -1205 -> internal13 -> wire8.
Consequently a Bad Mac! or future-counter EDY fromA02 can become an aggregate
no-valid-session failure before coordinator classification. Mapping the raw
Rust primitive's BadMac directly to wire7 would skip this native wrapper policy.
ED4 is not aggregated by these EDY-only catches.

## Sender-key signature producer

Fj2.A00 splits a64-byte signature from the serialized sender-key message,
verifies it via1oA.A01, and on false throws EDY(String) with exact
“Invalid signature!” at0036..0046. Eyj.A01 invokes this checker, does not catch
EDY in its EDQ/EDR catches, and0Yl.A00 maps EDY to -1005. EtY then refines
this exact message to -1203 -> internal11 -> wire5. Fast-ratchet0WJ.A0P also
invokesFj2.A00 and maps EDY to -1005. These selected receive signature paths
are distinct fromEcx.A00's device/Kyber bundle signature failures using EDQ.

**Inferred:** Rust needs producer context and aggregation semantics when
preserving causes; a single global primitive-error-to-wire table is insufficient.
**Unknown:** every session selection policy, provider equivalence, other error
producers, complete admission/count/key/routing/ACK policy and server acceptance.
No retry error code or production behavior added in this research correction.

Reproduce using target/debug/xtask apk-smali APK --class 'LX/0WJ;' --method A0N
--signatures --exceptions; substituteF22/A05,A01 orFj2/A00 for selected exports.
Omit --method for0WJ inventory then retain full signature lines only. Prior
constructor/writer exports remain in the linked historical bundles.

Validation:31 filtered retry tests, formatting, APK coverage, staged whitespace
and research checksum/link hygiene passed. This chunk changes research and a
source attribution comment only; the full workspace suite was not rerun.
