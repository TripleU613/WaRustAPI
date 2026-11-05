# Receipt-carried retry keys — APK 2.26.43.72

**Confirmed static / local capability correction.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete checked DEX exports name exact signatures. No app, key adoption,
network request or live retry was executed; these are local exports, not REA
Evidence envelopes.

## Current parser and keys policy are distinct from a directory fetch

`F10.A04(0Tp,Eoh)` checks for a `keys` child at00c8..00d8. When present it reads
required identity/type, optional one-time `key`, required signed `skey` with
signature and further device/PQ material. Parsed fields are retained in EYC;
[the earlier complete receipt handler](../../rea/2.26.43.72-retry-enc-count/receipt-handler.txt)
loads those fields before its session/resend flow.

Current **163.A02**, not the older comparison-build151.A01 name, contains the
receipt-key processing policy. Selected branches require supplied key/identity
material and, under its boolean gate, an existing session with matching remote
registration. Device0/property0x6f6f and further gates can invoke AEA verification;
a rejection logs/returns false. Accepted branches can process receipt keys and
perform conditional legacy/PQ session cleanup. Full policy semantics and all
fallback outcomes are not ported or claimed here.

`163.A05` calls A02 before its A04 session/base-key policy; its full export
retains the surrounding effective-mode gates. A02's false return is **not**
proof that the whole APK handler always refuses every resend: A05 has further
session policy. `163.A01` is a separate current-build PQ-session cleanup helper
with a legacy-bundle diagnostic. Its complete export prevents confusing that
name with the older151.A01 key-processing method. No unsupported universal
Android refusal rule is inferred.

## Rust explicitly refuses its unported branch

`transport::retry_answer::answer` previously ignored the `keys` child, potentially
answering from a held session or reserving a directory query. Documentation
already identified receipt keys as unported, but that limit was not enforced.
After existing original/recipient/count/acceptance/age checks, the driver now
records local reason **`receipt_keys_unsupported`** and returns no send/query.
It does so for any top-level `keys` child, including empty/malformed content,
without trying to validate or adopt its key material.

This is an explicit **local capability limit**, not an APK/server error or
restriction code. Porting requires the real parser, identity/session/verification,
mode and fallback policies; ordinary directory fetch is not a proven substitute.
The parser remains available to other consumers; this change scopes the refusal
to the normal direct retry-answer driver and does not change receipt ACK policy.

Regression checks in the full saved/fresh retry workflow exercise matching and
changed registration with empty, byte-valued and partial structured keys. Each
returns no node, preserves outbox/session/pin rows, creates only one unanswered
checkpoint and creates no pending answer/query job. Repeated unsupported receipts
preserve the same checkpoint. A later supported no-keys receipt follows the
ordinary saved-session resend workflow. Fixtures are not valid native bundles
and do not claim key validation coverage.

## Reproduction

```sh
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/F10;' --method A04 --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/163;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/163;' --method A05 --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/163;' --method A01 --signatures --exceptions
```

Compare stdout respectively with receipt-parser.txt, key-processing-policy.txt,
session-policy-entry.txt and legacy-pq-session-cleanup.txt. Each selects one
method. Checksums cover the README and complete exports.

**Inferred:** explicit unsupported handling avoids conflating this branch with
supported retries. **Observed live:** none. **Unknown:** full key-policy parity,
AEA/PQ/legacy fallbacks, higher-count base-key state and root normalization.
No zero-ban or server-acceptance conclusion follows from this offline correction.
