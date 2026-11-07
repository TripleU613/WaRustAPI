# Unhandled receipt root copied487 — APK 2.26.43.72

**Confirmed selected static / implemented supported-route response**, build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask exports; no target execution, account traffic or REA Evidence IDs.

## Route before context

The retained [complete1rU.A05](../2.26.43.72-receipt-router-boundary/receipt-route.txt)
first resolves root/context metadata, then tests a FIRST participants/message_id,
ordinary status set, special delivery branches, conditional view, simple mapped
roots, retry, enc_rekey_retry and root type=error. The remaining0768 branch invokes
[2Cx.A00](../2.26.43.72-retry-ack-dispatch/unhandled-receipt.txt) with the EXISTING
Eoh. It records local diagnostic490, copies Eoh and adds WIRE error487. It does
not invoke fresh1rg parse-error construction or preserve the received body.

[0Dn.A00](device-chat-cast.txt) unwraps DeviceJid.userJid or calls
[0Di.A00](base-chat-cast.txt), which accepts only0DY instances. This is the earlier
chat-resolution boundary; it cannot be inferred from a type string alone.
[F0R.A00](context-base.txt) reads source, required outer id, optional participant
and type into Efa. Its receipt-specific caller additionally selects UserJid
recipient, category and edit (retained in the earlier routing bundle).
Negative distinction: the existing0x0.A04 message-context export constructs an
outgoing message descriptor, not this incoming F0R receipt context.

The view special branch additionally requires native0Dn.A0a to succeed.
[That predicate](type21-predicate.txt) tests getType()==21, not arbitrary
class=status presence. Supported ordinary user/group senders do not satisfy it;
view therefore reaches the unhandled fallback there even with class=status.
Newsletter/broadcast/other sender routing and feature states are not generalized.

## Runtime correction

Previously valid unknown root types reached no response. Shared receipt ACK
construction now selects copied487 for supported ordinary user/group routes
that survive the preceding selectors and local address/string/writer guards.
It preserves outer id, original type (including a present empty string), selected
to/participant/recipient, and nonempty edit other than0. It emits no category or
received aggregate children. The [complete F3e.A05 writer](../2.26.43.72-prekey-upload-replies/prekey-ack-wire-builder.txt)
suppresses type only for delivery, not empty string; it separately suppresses
empty/zero edit. Its additional descriptor attributes carry error487.

The seven ordinary kinds, mapped simple kinds, retry, rekey retry and root error
are excluded from this fallback. A first participants/message_id also excludes
it regardless of root kind; a key-only or later participants child does not.
An unrelated error child does not change unknown-root routing. A valid native
special type is not labelled unknown merely because Rust lacks its handler.

Existing capability guards remain: missing/empty/over128 outer id, unsupported
context address/string carriers and unwritable selected fields grant no response.
These are local limits, not new native errors. Unsupported native-null optional
contexts are not broadened. Full sender-family/binary/typed-value equivalence,
provider/readiness/queue policy and remote acceptance remain unverified.

## Tests and persistence

Unit cases cover unknown/empty/view types, no body/key-only/irrelevant error/later
message_id children, special-type exclusions and early aggregate precedence.
PN/LID/group and class=status controls verify the selected view fallback; server,
broadcast/newsletter, invalid outer id and unsupported edit remain unacknowledged.
A normal receive-loop test asserts the exact copied response after durable raw
capture, unchanged outbox/ratchet/identity rows, and a second explicit delivery.
A synthetic write cut terminates that connection; reopening preserves state and
does not replay the uncertain ACK from its saved capture. No fact/query/resend or
identity work is introduced by the unhandled branch. These tests establish local
construction/order, not native runtime execution or server acceptance.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Di;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/F0R;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0a --signatures --exceptions
cargo test --locked --workspace --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
```

checksums.json covers every other file. Earlier complete routing/context/writer
exports are linked rather than duplicated; no APK/private state/bulk tree included.
