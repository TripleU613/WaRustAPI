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
