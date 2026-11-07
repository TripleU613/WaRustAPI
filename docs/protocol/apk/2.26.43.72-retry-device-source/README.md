# Retry requesting-device selection — APK 2.26.43.72

**Confirmed selected static / implemented**, build264307238; pinned SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask predicate exports and linked complete parser/handler;
no target execution, account traffic or REA Evidence IDs.

## Original chat selects the device source

[F10.A04](../2.26.43.72-retry-receipt-keys/receipt-parser.txt) selects a UserJid
recipient or normalized context.A02, applies A01 and constructs the original
message key. [Complete160.A04](../2.26.43.72-retry-ack-dispatch/retry-handler.txt)
reads that key's chat at0148. It tests [A0P](A0P-predicate.txt) for broadcast
types3/5/6 and [A0m](A0m-predicate.txt). The latter uses
[A0l](A0l-predicate.txt) and [A0V](A0V-predicate.txt): group types1/2/28.

Both successful tests branch to0faa: context.A01 PARTICIPANT, followed by shared
DeviceJid conversion at0168. Otherwise0164 reads context.A02 ROOT SOURCE before
that same conversion. Displayed offsets are bytes; the two positive branch
offsets at015c/0160 are code units, landing at0faa. The backward branch at0fae
lands at0168. Root and participant are not interchangeable fallback sources.
The [incoming context builder](../2.26.43.72-unhandled-receipt-root/context-base.txt)
stores source in A02 and participant in A01; an outgoing-message context builder
would not prove this incoming binding.

Thus a direct retry with optional participant naming another user/device still
asks on behalf of its root sender. A group/broadcast retry uses the participant.
Selecting an explicit direct recipient also selects the direct device-source
branch; a group root cannot borrow its participant to become a direct requester.
Native null-device processing has additional paths and is NOT inferred to be a
schema error or automatic success; the public Rust model requires a requesting
user/device and leaves nonrepresentable cases without a RetryRequest.

## Runtime correction and constructor boundary

Rust previously used participant whenever generic parsing succeeded, regardless
of original chat type. An unrelated optional participant could suppress a valid
direct resend or influence which existing ratchet/query history it considered.
The parser now uses original chat class to select source versus participant.
It retains supported group/broadcast/temporary/interop-group source selection
without implementing their resend policy. Group retries remain unsupported by
the current ordinary direct responder; no new group resend is authorized.

The parser's local address reader now preserves known text-constructor nulls
before generic parsing. Null root source or explicit recipient grants no ordinary
RetryRequest; a null selected group actor cannot fall back to the group itself.
An irrelevant direct participant does not override the root source even when its
text is constructor-null. Existing typed/binary objects, explicit-recipient
capability limits, carried-key/count/age/original-send/identity policies remain
separate. Full factory/cache/binary and native null-device handling remain open.

## Tests and corrected older expectations

Parser regressions cover misleading PN/LID direct participants, constructor-null
participants, explicit direct recipient plus group source, missing/null group
actors, invalid root source despite a valid recipient/participant, and invalid
explicit recipient text. Existing group fixtures used by unrelated parser tests
now include their required asking participant; earlier group-as-device fallback
expectations were not native evidence.

The saved-session workflow's successful same-ID direct resend now carries a
misleading participant while retaining the authenticated root source. The normal
outbox/ratchet reservation succeeds against that source, duplicate suppression
still applies, and later refetch policy remains exercised. Separate null-source
variants with matching explicit recipient/participant leave all outbox,
checkpoint, session and identity rows unchanged and emit no resend/query/ACK.
The wrong-original-chat regression now stores its unanswered reason against the
actual source-selected device, rather than the misleading participant.
No PN/LID mapping is learned, identity adopted or live account operation performed.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0P --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0m --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0l --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0V --signatures --exceptions
cargo test --locked --workspace --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
```

checksums.json covers every other file. Existing complete handler/parser/context
exports are linked, not duplicated. No APK/private state/bulk tree is included.
