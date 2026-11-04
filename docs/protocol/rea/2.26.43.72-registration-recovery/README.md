# Registration response and guessing waits — 2.26.43.72

**Confirmed static, 2026-11-04.** APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
REA 6.5.0 / headless JADX 0.7.1. No APK execution, account operation or
fresh live validation occurred. This is a focused continuation of existing
Rust registration behavior, not a claim of full registration parity.

## Retained evidence

| File | Observation |
| --- | --- |
| [package-summary.json](package-summary.json) | `com.whatsapp`, 2.26.43.72 / 264307238; 106,897 parsed classes, 15,728 resources. Inventory is not understanding or conversion coverage. |
| [bridge-class.json](bridge-class.json) | Current exact bridge method names/prototypes; old obfuscated names must not be reused by assumption. |
| [register-phone-number.json](register-phone-number.json) | `KotlinRegistrationBridge.A07`: `guess_wait` -> `HS3.A0K`, `retry_after` -> `HS3.A0V`; distinct fields. |
| [verify-security-code.json](verify-security-code.json) | `KotlinRegistrationBridge.A0B`: `guess_wait` -> `H6x.A0C`. |
| [response-conversion.json](response-conversion.json) | `Hfq.A06`: maps `HS3.A0K` into `H6x.A0C` and `HS3.A0V` into `H6x.A0M`, separately. |
| [otp-pin-handoff.json](otp-pin-handoff.json) | `VerifyPhoneNumber.A5n`: password `guessed_too_fast` reads `HS3.A0K` through `A11`, then starts the input timer; ordinary OTP branches differ. JADX duplicates switch branches, so do not infer control flow from repeated rendered source. |
| [password-wait-parser.json](password-wait-parser.json) | `VerifyPhoneNumber.A11`: signed decimal long, positive seconds multiplied by 1,000; absent/nonpositive -> 0, malformed/overflow -> null. |
| [pin-wait-consumer.json](pin-wait-consumer.json) | `IBz.run`: smali fallback preserves the real method when Java decompilation fails. `:pswitch_6d7` reads guess_wait, parses long and multiplies by seconds-to-ms, then calls `VerifyTwoFactorAuth.A14`. `:pswitch_716` selects retry_after for one mode and guess_wait otherwise. |
| [pin-wait-timer.json](pin-wait-timer.json) | `VerifyTwoFactorAuth.A14`: waits >=1 second persist `code_retry_time` and disable input; smaller waits remove the deadline and cancel the timer. |
| [initial-pin-handoff.json](initial-pin-handoff.json) | `VerifyPhoneNumber.A2B`: this entrypoint does not forward guess_wait when opening the PIN screen. This bounded negative prevents applying the field blindly to initial PIN challenges; other entrypoints remain untraced. |

The envelope Evidence IDs, exact DEX prototypes, representations, fallback
markers, source-completeness flags and provider limitations are retained in each
JSON. The large runnable method is retained whole so its registers, labels and
exception handlers remain available; it is not interpreted as one homogeneous
registration branch. The whole class-name inventory stays outside Git.

The three `wa-*-xref/fields.txt` files are ordinary `xtask` output, not REA
Evidence. Reproduce with the exact APK:

```sh
target/debug/xtask apk-xref /path/to/WhatsApp.1.apk --anchor guess_wait
target/debug/xtask apk-field-refs /path/to/WhatsApp.1.apk \
  --class 'LX/HS3;' --field A0K --type 'Ljava/lang/String;'
target/debug/xtask apk-field-refs /path/to/WhatsApp.1.apk \
  --class 'LX/H6x;' --field A0C --type 'Ljava/lang/String;'
```

REA calls use `inspect_android_method` with the class/method above and explicit
`overload_index: 0`. `ownership.json` explicitly associates method evidence with
`registration/wire.rs` and `registration/mod.rs`; association does not mark the
whole method or subsystem as ported. `xtask apk-coverage` reports it separately
from existing recipe-bundle citations.

## Rust continuation and limits

**Implemented/offline:** `RegistrationResponse.guess_wait` parses the server
field as seconds, exposes numeric `guess_wait_seconds` in redacted response
summaries and the existing `lab-signup exist` diagnostic, and retains it in the
maximum wait on refused/cooling-down states. A shorter generic wait cannot
shorten a longer guessing delay, and a guessing delay alone never clears the
refusal. Serialization preserves the deadline. No new retry command or live
permission was added.

This is a conservative extension of the existing Rust refusal policy, not a
complete port of the Android PIN screen. Rust uses its existing strict
nonnegative-integer duration parser: malformed/negative values fail locally
rather than becoming permission to retry. Android's mode-specific wait
selection, malformed/overflow recovery, UI and all PIN re-entry branches are
not claimed equivalent. Their remaining work is in the
[whole-APK queue](../../apk-porting.md).

**Corrected stale blocker:** ordinary OTP `stale`/`incorrect`/`mismatch` recovery
already exists in `registration/mod.rs` (commit `c7ff30d` for wrong OTP).
`mismatch` gates the next answer with `retry_after` and a 60-second fallback.
It was incorrect to describe that path as wholly missing based on an older
unknowns entry. Do not replace that timer with guess_wait based on field names.

Tests cover numeric/string/zero/null input, invalid-value redaction, distinct
retry and guessing fields, refusal preservation, maximum cooldown and serialized
deadline preservation. Full workspace checks are recorded in STATUS.md.
