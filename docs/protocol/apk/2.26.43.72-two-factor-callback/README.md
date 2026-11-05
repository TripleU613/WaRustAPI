# Two-step verification changes: timer, replies and diagnostic privacy

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX method exports, not runtime observation or REA Evidence envelopes.
No live PIN/email change, verification, disabling or account operation occurred.

## Current request and callback timer

The exact2fa anchor locates1HL.A07. Its selected online branch constructs
account/set/to=s.whatsapp.net IQ with one2fa child. Code and email are added
independently only for nonnull inputs, including supplied empty strings.
The helper does not impose the Rust six-digit/email/local-at-least-one-field
validation rules in this method; those remain explicitly local API bounds.
The initial07p.A0T gate posts a UI runnable and returns when false; its full
runtime connectivity and observer policy remains open, not a fabricated gate.
This is an account-setting IQ, not the separate registration HTTP PIN challenge.

1HL.A07 createsFUS with the originally supplied values, then passes32000ms
(0x7d00), logical type111 (0x6f), to0Sf.A0W at017c..018a.
[Shared timer/admission evidence](../2.26.43.72-abprops-callback-timer/README.md)
retainsA0W/A0A and0ln.A08: callback/timer registration precedes writer dispatch,
and promotion does not restart it. Timer expiry and reply are competing callback
claims. Nominal duration alone is not an independent native reply-age predicate.

## Failure and success are distinct operations

FUS.BsS postsFel(FUS). Its exact constructor sets selector24; Fel.run case24
(01fa..0224) resolves1HL and callsA06(-1, Delivery failure.). A06 logs the
fixed failure and notifies registeredD6x.CMq observers. The selected method
contains no request resubmission or setting write; full observer behavior is
untraced. It does not establish server refusal or non-delivery.

FUS.CK4 does not read its response-node parameter. It resolves1HL and schedules
FcH with the original code/email and integer0. That runnable stores/clears
settings, conditionally uses14C encryption/round-trip validation, updates nag/
email metadata and may enqueue Fetch2FAEmailStatusJob when email is absent and
its saved status is unknown. Full encryption/provider/job/observer semantics
remain unported; this is not permission to mimic a plaintext fallback, adopt
credentials speculatively or schedule extra requests.

FUS.Bvj obtains an error pair, defaults to-1/null when absent, and postsFej.
Fej's full body is not traced here. Thus result, outer refusal and delivery
failure must not be conflated. The success callback does not interpret nested
error extensions inside an outer result. Synthetic extensions in tests are
not observed server responses.

## Rust corrections

**Implemented offline:** shared private request-shape matching identifies the
account/set2fa operation, independently of unrelated account namespaces and
registration HTTP. Normal action callbacks now expire32s from immediately
before awaited local write, including write delay. Expiry releases only live
authority, preserves durable Sent uncertainty and continues receiving without
retry, status reset, credential adoption or rollback. Result/error cancels its
timer; no absolute reply-age test is added. Typed two-factor-set outer result
followsFUS.CK4, ignoring nested error extensions; outer error/code and other
action refusal policies remain intact. Full native local-setting adoption and
error/observer/job policies remain separate.

**Confirmed local source / fixed:** TwoFactorChange previously derivedDebug,
which exposed borrowed PIN and recovery email when formatted. CustomDebug now
shows None versus Some(redacted) without either value, including empty clear
operations. This changes diagnostic formatting, not borrowed inputs, validation,
request bytes or encrypted persistence. It is not a claim that every possible
log/capture in the client is audited or that no plaintext exists in memory.

Paused-clock test covers PIN set, explicit empty disable and email-only change,
31/32s boundary,5s local-write delay, result/error, synthetic result/error403
extension, duplicate claims and unchanged persisted rows after expiry. Normal-
driver expiry test now includes this change with33s local-write delay, late
result, subsequent receipt ACK, no resend and clean stop. Debug tests cover
normal/pretty formatting, absent/empty/populated fields and unchanged requests.
No new CLI/HTTP route or account operation is added. Rust serializes receive/
expiry after awaited writes; full Android thread scheduling is not reproduced.

## Reproduction and limits

`target/debug/xtask apk-xref APK --anchor '2fa'`;
`apk-smali APK --class CLASS [--method METHOD] --signatures --exceptions`:
1HL.A07 (set), FUS class (callback), Fel.<init>/run
(timeout-constructor/timeout), FcH.run (success),1HL.A06 (failure-observer).
OnlyFel selector24 and the identified success path are interpreted, with complete
selected bodies retained. The namespace anchor also found0DS.A0Y's separate
local preference storage; it was not treated as the IQ producer or a model for
Rust credential persistence. checksums.json covers every other file here.
APK, bulk tree and real credentials are not committed. Fresh interoperability,
full callback policy and native local-setting lifecycle remain unfinished.
