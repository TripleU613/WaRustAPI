# Sender-key message signature retry cause

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No new APK export: this follow-up reuses the complete checked-DEX methods in
[direct cause paths/Fj2.A00](../2.26.43.72-direct-retry-cause-paths/README.md),
[group cause paths/Eyj.A01 and0Yl.A00](../2.26.43.72-group-counter-error-paths/README.md)
and [EtY/F2q/DxB](../2.26.43.72-outgoing-retry-error-map/README.md).
These are local exports, not REA Evidence envelopes. No target/account execution.

Fj2.A00 verifies the sender-key message signature using the supplied saved public
key; false produces EDY(String), exact Invalid signature! at0036..0046.
Eyj.A01 invokes that checker; its EDQ/EDR catches do not catch EDY. 0Yl.A00
turns EDY into status -1005 viaEHq; EtY's exact string refinement yields -1203,
F2q internal kind11, DxB wire5. This is receive-message signature verification,
not device-key or Kyber-prekey bundle verification inEcx.A00.

The installed backend group_decrypt parses SenderKeyMessage, selects the saved
chain, verifies its signature against the saved signing key and returns typed
SignatureValidationFailed on false, before message-key advancement/decryption.
Backend source identity/line references are in backend-source-identity.json.
**Inferred compatibility mapping / implemented offline:** only receive_group
maps this condition to SignalInvalidSenderKeySignature. Inbox retains it as
InvalidSenderKeySignature and normal receive chooses RetryCause::SenderKeySignature
-> retry/error5. Fixed diagnostics contain no provider address/signature payload.

The internal cause enum also preserves MissingSession -> wire1; it has no generic
fallback code. Existing public unclassified builders retain their behavior.
No signature mapping is applied to SKDM/direct/prekey/bundle errors. Count1,
original message timestamp, existing routing, durable selector and once-only
reservation are unchanged. Held/uncertain retry records are never rewritten or
replayed when causes change. No delivery confirmation is earned by a bad signature.
Offline CLI replay accepts the outcome and sends nothing.

Offline reproduction: cargo test --locked -p whatsapp-api --all-features
undecryptable_message_requests_once_before_write_cut_and_never_replays.
The production-loop fixture creates a sender key, installs a matching trusted
test distribution, encrypts synthetic content and flips a trailing signature
byte. It asserts error5/original timestamp, persisted uncertain reservation before
a failed write, no second retry/delivery ACK on reconnect/redelivery, and unchanged
sender-key storage. Direct/group missing-session cases still assert error1.
This does not execute the APK, prove runtime packet acceptance or prove full
native admission/count/key/routing/ACK parity. Other typed causes remain open.

Validation:1328 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Offline only.
