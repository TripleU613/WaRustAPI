# Group constructor versus groups_dirty raw-string boundary

**Confirmed static**, pinned WhatsApp2.26.43.72/code264307238, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `35a690f3`. Offline xtask fallback, not REA-provider/runtime output.
The existing provider failure is [retained separately](../2.26.43.72-retry-key-envelope-bytes/rea-provider-attempt.json).
No live operation or runtime admission change.

[0Dd.A00](../2.26.43.72-message-ack-chat/jid-parser.txt) constructs1bc for g.us.
The complete [1bc constructor](group-constructor.txt) accepts either
[0nW.A02's modern predicate](modern-user-predicate.txt) or its own legacy path.
The modern predicate requires 1..20 ASCII digits with first digit1..9 and no
hyphen. Legacy splits at the first hyphen, validates the prefix through
[0Dl.A02](legacy-prefix-predicate.txt), and requires exactly10 ASCII suffix
digits with first1..9. The prefix is5..20 ASCII digits, nonzero-leading, with
prefix10 disallowed unless1008. Failed construction throws09P. This corroborates
the existing messaging.ack_non_user_text_valid group-constructor predicate;
it is not a newly invented global Rust JID requirement.

The selected [0w9.A09 dirty branch](../2.26.43.72-group-event-selection/dirty-event-selection-excerpt.txt)
at28b4 instead obtains group@jid via0Tp.A0M as a raw string and puts nonnull
strings into pairs. No typed GroupJid getter or1bc call occurs in this selected
branch before that collection. This does not prove there is no downstream
constructor; scheduling/worker parsing and alternate producers remain separate.
Do not apply the constructor blindly to this raw-string scheduling boundary.
Rust's existing local require_group_id admits its supported shorter legacy
fixtures; it is not tightened by this trace. Full native queue-to-request group
identity admission remains unresolved, not declared compliant by this choice.

## Runtime integration proof added for the existing ACK guard

The previous shared LID ACK-route fix is now tested through actual
handle_dirty_notification plus notification_ack, rather than only a pure base
ACK helper. Text and typed malformed LID from/participant (alphabetic, zero,
leading-zero,16-digit and fullwidth forms) are captured first. Dedicated handling
returns no ACK and creates no action; generic handling must also return None,
so it cannot fall back to ACK after dirty-content refusal. Account/checkpoint/
pin/session/outbox rows and journal stay equal. Reopen and a second handling
attempt preserve those exact snapshots and continue returning no ACK.
The valid dirty queue-before-ACK/dedup integration control remains unchanged.
No new runtime guard or native malformed-envelope policy is claimed in this
chunk; this strengthens the store-level evidence for the existing correction.

Reproduce new constructors and reused factory export:

```sh
python3 docs/protocol/apk/2.26.43.72-group-dirty-constructor-boundary/reproduce.py ../android-lab/apks/WhatsApp.1.apk
cargo test -p whatsapp-api --all-features malformed_dirty_ack_routes
```

The script verifies the APK digest and byte-compares complete exports. Existing
bounded dirty selection is linked rather than copied as a complete method.
Global group grammar, native raw-string consumer admission, all group-event ACK
paths and live acceptance remain outside this bounded result.

Validation: three new complete exports and the reused factory reproduced
byte-for-byte. Full workspace/all-feature tests passed, including 1,421 library
tests; strict Clippy, CLI-reference regeneration, OpenAPI library-symbol test,
APK coverage, formatting, whitespace and staged hygiene passed.
