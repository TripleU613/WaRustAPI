# Usync outer refusal: distinct defaults and exact JVM arithmetic

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution or live account operations. This follows the
[protocol-result backoff audit](../2.26.43.72-contact-protocol-backoff/README.md)
and [factory comparison](../2.26.43.72-contact-factory-selection/README.md).
No REA Evidence ID is invented for DEX exports or the desktop JVM oracle.

## Outer IQ error is not the protocol-result error branch

0pD.Bvj removes the owned H6l from its in-memory request map. A missing owner
logs and performs cleanup; it does not create a new callback from durable state.
For an existing request, code starts at0 and backoff at-1. It finds the first
direct error child; present code is Integer.parseInt and present backoff is
Long.parseLong. Missing attributes preserve those defaults. Present backoff
uses mul-long by1000, then Math.min with3600000. Negative values and signed
wraparound are preserved by these operations; this is not an unsigned duration,
a minimum wait or saturating multiplication. The method has no local catch table.
Malformed present numeric data is not replaced with a guessed default.

It calls the bound0p9.BLg with captured context/SID, code and backoff, completes
the future with null and cleans up. This differs materially from0pD.A01's
protocol-error child parser, where absent backoff uses7200 seconds before the
one-hour cap. Importing that default into Bvj would manufacture a wait that
this callback did not supply. A literal backoff=-1 seconds becomes-1000ms;
absence remains the native-1 sentinel, so attribute presence is meaningful.

Observer scope and applicability still matter. The prior source confirms
positive waits can produce observer-specific absolute deadlines. The new
physical HxV.A0C preference-field census has only constructor assignment and
Hzo.BLg read; selected query methods have no direct read of that field. That
bounded result does not prove the entire caller graph lacks a wait gate:
other owners/executors and escaped aliases remain outside this field census.
No universal contact or all-IQ backoff gate is inferred.

## Implemented durable inspection

Correlated durable usync get-action outer errors now retain optional
ActionOutcome.usync_refusal atomically with reply settlement. The parser reads
only a first direct error child's code/backoff. Fields are parsed signed code,
code_present, backoff_millis, backoff_present and fixed failure code/backoff.
Absent code is Some(0), not proof that the server literally sent zero. Absent
backoff is Some(-1), while malformed present data is null with a fixed failure.
A code parse failure leaves backoff unparsed; a backoff failure retains the
already parsed code. Nontext attributes are fixed local parse failures rather
than user data in diagnostics. Raw error text/phones/IDs are not added to this
metadata structure. Existing generic action error fields keep their own contract.

Existing actions plain/JSON exposes the metadata. Ordinary non-usync actions,
non-get requests and outer results do not get this field. In particular an
error nested in a successful usync protocol result is not parsed as Bvj data.
Legacy rows omit it and are not backfilled: absence means unparsed/unknown,
not a proven absent server wait. Contact projection preserves this field, and
duplicate replies cannot rewrite it or recreate live ownership.

This is relative metadata inspection, **not an enforced cooldown**. It does not
choose an observer, manufacture an absolute native A00 deadline, mutate request
eligibility, retry or reset an action. Callback-time absolute clock evidence,
factory-specific read/reset gates, protocol-result metadata and full wait policy
remain open. Future wait adoption must not treat legacy missing metadata as a
zero wait or start a historical relative wait at the later inspection time.

## Independent verification and reproduction

UsyncRefusalOracle.java executes only desktop JVM Integer.parseInt,
Long.parseLong, wrapping multiplication and Math.min. numeric-vectors.json is
its exact output. It covers absent values, explicit zero/negative, one-hour
cap/equality, signs, Arabic decimal digits, whitespace/empty values, i32/i64
overflow and Java-long multiplication wraparound. Rust tests compare the actual
parser with this independent output. Other tests cover first-child selection,
nontext redaction, absent error child, namespace/outer-result separation, durable
settlement/reopen/contact projection and duplicate/no-replay behavior.

Reproduce current Bvj with existing xtask apk-smali APK --class 'LX/0pD;'
--method Bvj --signatures --exceptions. Reproduce the field census with
apk-field-refs on HxV.A0C of typeLX/0pQ;. Regenerate the oracle using
java UsyncRefusalOracle.java and require byte equality with numeric-vectors.json.
No target code runs. Checksums cover every other file in this bundle.
