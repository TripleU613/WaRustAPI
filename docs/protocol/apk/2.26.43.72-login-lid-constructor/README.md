# Login LID constructor grammar and setter boundary

**Confirmed static**, pinned WhatsApp 2.26.43.72/code264307238, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `8f7759f8`. Offline xtask fallback; no REA-provider output or Android
execution. See the [retained provider failure](../2.26.43.72-retry-key-envelope-bytes/rea-provider-attempt.json).
No live operation. This closes the constructor question left by the
[initial reader audit](../2.26.43.72-login-success-reader/README.md).

## Constructor validation precedes the getter's compliance log

[0Dx](lid-user-type.txt) identifies itself with server lid and type18. Its
constructor accepts 1..15 characters, rejects a leading zero and checks every
character against ASCII 0..9, throwing09P on violation. This is construction
validation, separate from isProtocolCompliant logging in the getter.
The [nullable string parser](nullable-string-parser.txt),0Dd.A02, calls A00
and catches failed construction to return null. Reuse the existing complete
[A00 parser](../2.26.43.72-message-ack-chat/jid-parser.txt) and
[0De.A02 factory](../2.26.43.72-message-ack-chat/user-factory.txt), which bind
lid user construction to0Dx. The new callsite evidence is login's
[0Tp.A0A read](../2.26.43.72-login-success-reader/jid-getter.txt) and
[0ZJ.A13](../2.26.43.72-login-success-reader/login-reader.txt), not an assumption
that every ACK policy applies to login. Already typed JIDs bypass string parsing;
the selected constructor establishes what a normal typed LidUserJid can contain.
Upstream binary typed-attribute allocation and every malformed-input path are
not comprehensively traced here.

## Setter follows existing retained evidence

A code-body scan with class filter empty and method substring ChR finds three
declarations: abstract09F, delegate09G and concrete09O, retained together in
[setter-definitions.txt](setter-definitions.txt). This corroborates the already
[documented fill-only path](../../own-companion-ingress-current.md#me-is-persisted-identity-data-not-standalone-authentication-proof);
it is not a newly discovered universal identity replacement rule.
The scan covers DEX declarations selected by those filters, not runtime binding.

09O.ChR accepts a type18 JID through [0Dn.A0Z](lid-type-predicate.txt), or a
[0EM device](lid-device-type.txt), whose server is lid/type19; the latter's
userJid is cast to0Dx. Other inputs return. It calls
[A07's initialization gate](own-record-initialization-gate.txt), enters its
monitor and returns if held0E2 already has nonnull LID. For an absent LID it
compares getRawString with0Cz.A01 and conditionally applies self_lid preference,
then, if0E2 exists, builds a new record retaining PN/country/number fields.
[09O.A00](record-adoption.txt) adopts that record and refreshes in-memory device
views/providers, with a09P cleanup handler. It is not a Me-file writer.
[09O.A02](record-notify.txt) compares old/new records and invokes09f.Bnp on change.
Observer bodies, disk completion guarantees and all setter origins remain
separate. Effective login flag, selected provider instance and complete own-LID
replacement lifecycle remain unknown. The initialization gate is retained but
its callable implementation was not newly followed.

## Rust correction and limits

The shared connection.login_user helper previously accepted letters, leading
zero, Unicode digits, spaces and >15-digit LID users. A new owned-login regression
failed at baseline because letters-private@lid settled as success. It now reuses
native_plain_lid_user_valid only for Server::Lid, covering supplied and requested
saved identity validation in the existing transaction. Fixed redacted diagnostics,
rollback, prior capture, uncertain reservation and no-replay policy remain.
PN parsing, agent/device admission and valid-LID replacement behavior are not
changed or claimed native-complete; porting the conditional fill-only setter is
not justified without its provider/flag/lifecycle boundary.

Regression uses text and typed inputs; valid 1/15-digit controls succeed, invalid
alphabetic/zero/leading-zero/16-digit/Unicode/space users refuse without changing
account/checkpoint/pin/session/outbox rows or event head. Reopen retains capture
and blocks another reservation. Saved malformed LID cases also refuse updates;
absent fields still leave history untouched. Existing valid device normalization
and shared CLI partial-update tests remain controls. Synthetic data only.

Reproduce all new retained outputs and the two reused parser/factory outputs:

```sh
python3 docs/protocol/apk/2.26.43.72-login-lid-constructor/reproduce.py ../android-lab/apks/WhatsApp.1.apk
cargo test -p whatsapp-api --all-features login_
```

Validation: ten new/reused method outputs reproduced byte-for-byte. The failing
baseline regression is recorded above. Full workspace/all-feature tests (1,413
library tests plus workspace/integration suites), strict all-target/all-feature
Clippy, CLI-reference regeneration, OpenAPI library-symbol test, APK coverage,
formatting, whitespace and staged hygiene passed. No fresh live validation.
