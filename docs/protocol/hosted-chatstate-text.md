# Hosted chatstate textual qualification

2026-11-06. **Confirmed static / implemented offline**, APK2.26.43.72
build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
This closes the pre-entry qualifier loss recorded by the
[hosted metadata projection](apk/2.26.43.72-hosted-chatstate/README.md).
No APK execution, runtime probe, account operation or new REA Evidence record.

## Qualifier presence precedes numeric value

The complete retained [0Dd.A00 parser](apk/2.26.43.72-message-ack-chat/jid-parser.txt)
was freshly re-exported and matched byte-for-byte. At0196..01c4 it finds the
last dot and colon in the local part. Only when both are absent does it invoke
0De.A02 as an unqualified user alias. A present qualifier takes the device
path even if its numeric component is0. Hosted/hosted.lid later select Dv4/Dv6;
the [complete constructors](apk/2.26.43.72-hosted-chatstate/README.md) require
original device integer99 and throw09P otherwise. Numeric zero cannot be
promoted to an unqualified hosted user merely because it is zero.

Rust's shared Jid::parse currently chooses Plain whenever agent/device numbers
are both0. Incoming chatstate's text fallback therefore lost the distinction
for user:0@hosted, user.0@hosted and user.0:0@hosted.lid. The subsequent hosted
projector treated these as ordinary aliases and reported user metadata. The
red regression reproduced this with :0@hosted.

## Scoped correction

The incoming chatstate address reader now preserves qualifier presence for raw
hosted/hosted.lid text, setting AD_JID kind on its local parsed copy when the
local part contains dot or colon. The existing hosted projector then applies
the device99 rule. Both from and participant use the same reader: an invalid
from yields no supported observation; an invalid participant yields no author.
The original text/Node remains unchanged. No caller's Jid object is mutated.

This does not change shared Jid parsing, saved identities, device hashes,
formatter inputs, authenticated alias adoption or outbound validation. PN/LID
text keeps its existing behavior, including ordinary device0 projection.
Typed attributes already marked AD_JID retain that distinction. An upstream
caller that already collapsed a qualified string to a Plain Jid has lost
information before this reader and cannot be repaired here. Complete native
text grammar, numeric/agent coercions, special families and full trust/admission
semantics remain open; this is ephemeral metadata, not a global parser port.

Tests cover both hosted domains with :0/.0/.0:0/:2 as from and group participant,
unchanged input, valid plain/:99/.0:99 forms and unchanged PN device0 handling.
Existing typed AD0/0 and hosted99 tests continue passing. No ACK, refresh,
request, replay, journal/identity adoption or live-account operation.

## Reproduce

From repository root:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0Dd;' --method A00 --signatures --exceptions > /tmp/wa-hosted-zero-parser.txt
cmp /tmp/wa-hosted-zero-parser.txt docs/protocol/apk/2.26.43.72-message-ack-chat/jid-parser.txt
cargo test --locked -p whatsapp-api --all-features hosted_chatstate_text_retains_explicit_zero_device_qualification
```

The shared full parser/constructor exports remain under their original checked
identities; no duplicate bundle or raw APK was committed. Full protocol-family
admission and every remaining network family still require the audit ledger.

Verification:1543 workspace tests across nine suites including doc tests, strict
workspace/all-target/all-feature Clippy, fmt, OpenAPI library-symbol and
CLI-reference tests, APK coverage freshness and repository hygiene passed.
The APK SHA256 and retained full0Dd.A00 export were rechecked exactly.
