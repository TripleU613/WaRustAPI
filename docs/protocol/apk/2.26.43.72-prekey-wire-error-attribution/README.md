# Key-fetch wire error attribution and signed-code inspection

Confirmed static, APK 2.26.43.72 code 264307238, pinned SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target execution, live account operation or REA Evidence envelope claim.
This connects selected wire callbacks to the payload consumed by the
[prekey backoff listener](../2.26.43.72-prekey-error-backoff/README.md).

## Current producer and callback

0v0.A00 at 1696 reads captured jids, identityJids, session scope and PQ flags.
At 1726..1748 it constructs Dxh(scope,owner,jids,fetchPq,isUpgrade) and inserts it
into the legacy owner's callback map under the request ID. It later builds IQ
get/xmlns=encrypt with key child and sends through Cyt. This is a current-build
key-fetch callback binding, not reuse of the older DF6/DFC obfuscated names.
The selected excerpt does not re-prove every conditional wire attribute; prior
request-shape audits remain separate.

Dxh extends 1ru and captures the original device array. The retained
[ordinary legacy dispatcher](../2.26.43.72-about-callback-lifetime/legacy-iq-base.txt)
selects outer error, removes its callback, iterates direct error children and
parses each present code with Integer.parseInt before A01(int). Dxh.A01 maps
all captured devices to scoped EoY keys and calls 1lZ.A1S with that signed code,
PQ-upgrade flag and conditional original request count. Thus outer error applies
to captured requested devices, not an arbitrary returned user. Multiple codes
can invoke this native callback multiple times; Rust's unique-refusal inspection
is a deliberately narrower attribution contract, not native duplicate emulation.

## Per-user error within successful directory result

Dxh.A02 requires list, parses returned DeviceJid, then reads that user's first
error child. Its required code reader 0Tp.A04 -> A06 uses Integer.parseInt;
NumberFormatException becomes LIL. Errors are stored by returned DeviceJid.
After parsing it iterates the originally captured request array: only originally
requested devices found in the returned set are considered for error grouping;
missing requested entries take A15's no-prekey path. Matching errors are grouped
by signed code in SparseArray and each group goes through A1S. Successful parsed
bundles take A1N separately. A valid enclosing result therefore need not mean all
requested devices yielded usable keys.

This does not prove duplicate-user policy, explicit-zero/text JID equivalence,
full key/Signal/ADV validation, or every missing-entry effect. Rust retains its
existing exact selected-peer and ambiguity guards.

## Event shape and listener boundary

A1S delegates to A0e and submits through 1lW.CPm. A0e constructs what=0,
arg1=76, arg2=0, obj=EoY[], with errorCode signed int, isPqSessionUpgrade and
pqUpgradeDevicesRequested in Bundle. These match 14o.BLn selector 76 exactly.
14o.Amf registers 74,75,76,77,78,82,83,84. Concrete 1lY.CPm obtains arg1 through
1uW.A00; a recognized type goes through router A07, otherwise its handler fallback.
Global registration/DI instance authority and complete handler ordering remain
separate; source payload compatibility does not prove a live event was delivered.
No retry is added merely because this mapping exists.

## Rust correction

DirectoryInspection.server_error_code previously used Option<u16> with Rust
ASCII parsing. Valid signed, >65535 and Unicode-digit Java integer codes were
reported as null even after unique peer/refusal attribution. It now uses
Option<i32>, the existing Java decimal reader and checked i32 conversion.
This widens the public Rust field type and JSON numeric range; field names and
old valid positive values remain unchanged. Generic ActionOutcome.error_code and
other inspectors' policies are not changed by this producer-specific correction.

The inspector still reads the retained correlated reply, requires one unique
root or selected-user error context and never exposes raw error text. Invalid,
absent or out-of-range code remains null while the existing refusal status stays
held. Unlike native required parsing, this read-only diagnostic does not throw a
new protocol error from malformed saved code. No pin/key adoption, callback
resurrection, new fetch, backoff bypass, refresh permission or resend is added.

A regression covers all 13
[executed JVM Integer.parseInt oracle cases](../2.26.43.72-about-error-codes/README.md)
in both outer and selected-user contexts, including a foreign user's earlier 503.
It verifies selected code, redacted output, no fingerprint/pin adoption, refusal
of refresh, unchanged inspection after reopen and no new send reservation.
Existing ambiguous-context cases continue to reject instead of choosing a code.
These are offline store/parser tests, not server acceptance observations.

## Reproduce and remaining work

Use xtask apk-xref with ConnectionThreadRequestsImpl/on-get-pre-key-error.
apk-callers class 1lZ/method A1S and class Dxh/method <init> reproduce censuses.
apk-smali with --signatures --exceptions exports full Dxh, 1lZ.A1S/A0e,
0Tp.A04/A06, 14o.Amf, 1lY.CPm and 1uW.A00. Writer excerpt names full 0v0.A00
export hash and byte-PC ranges. Checksums cover every other file. Raw APK and
bulk writer export stay local.

Remaining: complete job scheduling and durable backoff/retry ownership, per-user
no-prekey effects, duplicate handling, every JID grammar branch and cancellation
across write uncertainty. No causal restriction-prevention or zero-ban claim.
