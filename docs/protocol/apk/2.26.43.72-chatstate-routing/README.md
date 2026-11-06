# Incoming chatstate address routing

**Confirmed selected static / implemented offline**, APK2.26.43.72
build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Focused complete DEX methods and two labelled excerpts; no new REA Evidence
records or live account operation.

## Reader and event binding

Reuse complete [1rA.A03](../2.26.43.72-presence-reader/reader.txt) and
[A02](../2.26.43.72-presence-reader/tag-dispatch.txt). A02 names chatstate for
1rB, whose constructor stores the supplied1lZ. Selected A03 branch0576..0620
reads exactly child index0, from and participant via generic JID access, then
passes each address through0Dn.A00. It does not search later state children.
Composing reads optional media, converts the participant through0Dj.A00 and
calls1lZ.A0w. Paused performs the same participant cast and callsA0v.

Reuse complete [device unwrapping](../2.26.43.72-message-ack-chat/unwrap-device.txt),
[predicate](../2.26.43.72-message-ack-chat/device-types.txt) and
[chat cast](../2.26.43.72-message-ack-chat/chat-cast.txt), re-exported byte-for-byte.
0Dn.A00 unwraps recognized DeviceJid.userJid; otherwise0Di.A00 retains only0DY.
A0S checks types17/24/25/19/23/27, not a generic textual suffix rule. The existing
[declared getType census/factory binding](../2.26.43.72-message-ack-chat/README.md)
locates PN/LID device types17/19 and UserJid types0/18. Those selected getType
bodies match a fresh census with interface ancestry. This is declared structure,
not runtime dispatch or complete JID factory parity.

0Dj.A00 returns only UserJid, otherwise null. A group cannot be the supplied
user author. A server-only0mL is neither UserJid nor0DY (reuse the
[current target hierarchy](../2.26.43.72-presence-targets/README.md)). A null
converted chat can continue into native event construction; its downstream
handling is not established here. Rust's required chat metadata cannot
represent that null value and omits the supported server-only observation.

A0w compares media exactly with audio and callsA0D with integer0/1. A0D stores
jid/author/media in a Bundle and builds Android message20. A0v callsA0C, which
stores jid/author and builds message21. Both enqueue through1lW.CPm. Full event
consumer/state/UI policy and native ACK/refresh consequences are not reconstructed.
No transport ACK requirement is inferred from these Android message numbers.

## Rust correction and limits

parse_chatstate previously preserved PN/LID device suffixes and returned a
group or server-only address as participant. The red regression showed the
wrong device-qualified chat. Supported PN/LID from and participant now unwrap
to their plain user metadata, including explicit AD0/0 wire kind. Group and
empty-user PN/LID participants become None. Empty-user PN/LID from returns None
because this metadata type requires an actual chat. The original Node remains
unchanged; no stored alias, authenticated identity or durable state is adopted.

Other domains and special JID representations retain their previous metadata
behavior pending complete class mapping. This is explicitly not a complete
native from/author trust or admission validator. Hosted/legacy/interop/bot,
all native factories, null-chat downstream dispatch and consumer policy remain
open. No unsupported domain is silently normalized from an assumed type number.
Outbound typing still refuses device targets; incoming unwrapping is a separate
native reader rule and does not loosen those outbound guards.

First-child state selection, exact audio mapping and all raw capture behavior
remain. CLI uses this only for ephemeral redacted typing display; the normal
actor has no new state adoption or ACK. Tests cover PN/LID device/agent/AD0/0,
group authors, server-only metadata, untouched input, valid group chat and first
unsupported child before later composing. Existing plain typing tests still pass.
No new request, retry, replay, automatic typing event or account operation.

## Bounded discovery and reproduction

DeviceJid has no exact declared getType override, so that virtual-override root
was rejected. Switching to Jid's abstract getType declaration produced the
census; selected PN/LID bodies were checked against the retained census rather
than committed again. UserJid/DeviceJid class ancestry remains separate from
the allowed numeric type predicate. No untraced family mapping was guessed.

From repository root with built xtask and pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-chatstate-routing-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/1rB;' --method '<init>' --signatures --exceptions > "$OUT/reader-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Dj;' --method 'A00' --signatures --exceptions > "$OUT/user-cast.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1lZ;' --method 'A0w' --signatures --exceptions > "$OUT/composing-dispatch.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1lZ;' --method 'A0D' --signatures --exceptions > "$OUT/composing-event.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1lZ;' --method 'A0v' --signatures --exceptions > "$OUT/paused-dispatch.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1lZ;' --method 'A0C' --signatures --exceptions > "$OUT/paused-event.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1rA;' --method A03 --signatures --exceptions > "$OUT/reader-full.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1rA;' --method A02 --signatures --exceptions > "$OUT/tag-full.txt"
python3 - "$OUT" <<'RECIPE'
from pathlib import Path
import sys,re
out=Path(sys.argv[1])
for dest,src,lo,hi in [('reader-selected.txt','reader-full.txt',0x576,0x620),('tag-selected.txt','tag-full.txt',0x102,0x118)]:
    lines=(out/src).read_text().splitlines()
    selected=['# Selected DEX excerpt; not a complete method.']+lines[:6]
    for line in lines[6:]:
        m=re.match(r'\s*([0-9a-f]+)\.\.',line)
        if m and lo<=int(m[1],16)<hi:selected.append(line)
    (out/dest).write_text('\n'.join(selected)+'\n')
RECIPE
```

Compare all retained .txt byte-for-byte. Full reader methods stay under the
existing presence-reader identity; local intermediates are not committed.
Shared conversion/census procedures are in their linked README. checksums.json
covers every retained file except itself. No APK, private state or bulk tree.

Verification:1541 workspace tests across nine suites including doc tests, strict
workspace/all-target/all-feature Clippy, fmt, OpenAPI library-symbol and
CLI-reference tests, APK coverage freshness and repository hygiene passed.
All eight retained dispatch/reader exports reproduce exactly; reused conversion
methods and selected PN/LID census bodies match their retained sources.
Focused suite: `cargo test --locked -p whatsapp-api --all-features transport::presence::tests`.

## Hosted family follow-up (2026-11-06)

[Hosted projection](../2.26.43.72-hosted-chatstate/README.md) now binds the
hosted/hosted.lid underlying user and device99 rule in the same metadata
adapter. Other-family and full text/admission limits remain; this does not
change the original wire/hash/store address handling.
