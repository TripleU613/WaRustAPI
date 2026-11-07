# Upload transfer classification is not reusable-media acceptance

**Confirmed static**, pinned WhatsApp2.26.43.72/code264307238, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed during this audit. Baseline `35a320dd`. Offline xtask DEX exports,
not REA provider output or runtime observations. The known provider failure is
[retained separately](../2.26.43.72-retry-key-envelope-bytes/rea-provider-attempt.json).
No Rust runtime change, request, account operation or replay.

## Transfer gate and mode-dependent missing path

[MtZ.Cb6](transfer-result-gate.txt) installs MtR selector2 at0342..034c.
The existing [JSON callback](../../rea/2.26.43.72-media-upload-response/response-callback.txt)
constructs LzN, populates its optional fields, and sets MKA.A03 on its selected
JSONException handler. Parsed optional fields alone are not success authority.

At04ba the transfer calls K2k.A03. Its [complete bridge](code-bridge.txt)
delegates to [abstract A04](abstract-executor.txt), then closes its selected
resources and returns the delegate's integer. The concrete executor's complete
code/redirect behavior is not newly traced. Thus the following is a classifier
for the returned code, not proof that every numeric value can arrive from HTTP.

MtZ's returned-code branch at0508..050c admits nonnegative values below400 to
its response-mode checks. This is broader than exactly200;400 and other error
branches are separate. When MKA.A01 enables opaque mode, LzN.A00 is checked
for a nonempty whole-string Lkh pattern match. If MKA.A02 requires a path and
that opaque check fails, LzN.A02 must be nonempty to bypass the missing-path
branch. Otherwise configuration key0xce4 (3300) determines outcome29 versus0
at0576..0590. Without the required-path condition, the same selected path reaches
outcome0. The runtime flag value is unknown. The ordinary string check here is
nonempty, not the later M0g query/signature-presence normalization.

The pair is constructed at06f4, its first component becomes Lyh.A06 through the
[result constructor](result-type.txt), and the second becomes Lyh.A00. LzN,
MKA.A03 and a separate mode boolean are copied into Lyh.A01/A05/A04. These fields
are distinct; a parse-error flag does not automatically replace the outcome in
this selected construction. Other exception branches produce their own results.
No global retry/restriction meaning is assigned to these local numeric outcomes.

## Downstream adoption remains necessary

The existing [MOu.A07](../../rea/2.26.43.72-media-upload-response/result-adoption.txt)
reads Lyh.A06 and, on its BFP-selected zero-outcome path, rejects missing
transferred stream info with another local result at0076..0096. It separately
adopts response metadata and normalizes ordinary path through M0g, as already
[documented](../../rea/2.26.43.72-media-upload-response/README.md).
At032e..0378 it also reads Lyh.A05 when deriving the M88 boolean; that is not
standalone durable completion or permission to resend.

[Five direct A06 field references](result-status-references.txt) identify the
constructor plus MtZ,MOu,MvO andLHd. This audit newly retains
[MvO.ARB](upload-runner.txt): it distinguishes outcome17 before the selected
MOu.A07 call at0540 and takes additional result-dependent branches afterward.
LHd.A0c, effective providers/flags, all worker loop decisions, complete persistence
and restart/backoff authority remain untraced here. A field-reference census is
not a runtime execution or completeness proof.

## Consequence for Rust

Rust upload_prepared currently requires HTTP200, valid JSON and its supported
locations subset before producing protobuf and owned uploaded state. It preserves
response/capture before validation and retains unresolved outcomes without replay.
This trace establishes a broader native classifier and downstream gates, **not**
that Rust should accept all1xx/2xx/3xx, missing paths or parse-error replies.
No acceptance rule is widened. Complete selected request mode, concrete executor,
effective3300 flag, downstream result/persistence and native retry policy must be
established before claiming parity or changing those rules.

Reproduce every new complete method/reference export from the checkout:

```sh
python3 docs/protocol/apk/2.26.43.72-upload-result-classification/reproduce.py ../android-lab/apks/WhatsApp.1.apk
```

The script validates the pinned APK hash and byte-compares all exports. PCs are
bytes, branch deltas code units; exception tables state code units explicitly.
Existing callback/adoption bodies are linked rather than duplicated. This closes
part of transfer classification, while reusable-media completion remains partial.

Validation: six retained exports reproduced byte-for-byte from the pinned APK.
APK coverage, formatting, CLI-reference regeneration, OpenAPI library-symbol
test, whitespace and staged repository hygiene passed. This research-only
chunk did not rerun the full runtime workspace suite.
