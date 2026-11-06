# Current presence and typing targets

**Confirmed selected static / implemented offline**, APK2.26.43.72
build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Focused DEX exports and one labelled dispatcher excerpt; no new REA Evidence
records, runtime observations or account operations.

## Writer and concrete target types

DEc.A01 builds chatstate/composing, adds media=audio for media integer1, and
leaves media absent on the other selected branch. Its ordinary target class
set is UserJid,1bc,2p3; the bot branch has additional gates and a nested bot/jid.
DEd.handleMessage builds chatstate/paused with the same ordinary target set.
Selected send calls are0Sf.A0X with discriminator4 for composing and5 for paused.
These are not IQ requests; no result callback or new ACK policy is derived.

1bc extends GroupJid and getServer returns g.us, closing the old inferred-group
label for this current-build writer. UserJid extends0DY. Its getPrimaryDevice
asks0EB.A01 for device0 and returns DeviceJid. DeviceJid's class extends Jid,
as does server-only0mL (getServer=s.whatsapp.net); neither extends UserJid or1bc.
Thus a server-only address is not the named chat target type, and a primary
device object is still a device object even when its numeric device is0.
Full2p3, bot/interop target behavior and all class factories are not reconstructed.

0Tk.A03 uses Class.isAssignableFrom over the provided list and invokes00L.A0C
on mismatch. Reuse [assertion helper](../2.26.43.72-report-response-bounds/bound-assertion.txt):
it logs an AssertionError without an explicit throw. These are expected native
types, not proof of a hard server rejection or account-restriction trigger.
Rust enforces its declared supported primary-chat subset before construction.

## Subscription writer re-anchor

0v0.A00 switchcase12 enters04b6. The excerpt retains that table entry and the
selected branch body04b6..0622; the shared dispatcher tail and error/reconnect
handling are outside this excerpt. It reads optional token bytes and group
context (1bd.A03), builds presence/type=subscribe, optional context, and to with
native target classes1bc/UserJid. Optional tctoken is wrapped through smax:any;
the selected byte-length constants are1 and120. Current0Tj.A03 appends a child; A04 merges same-tag or smax:any nodes.
The inner any data merges into tctoken, the outer any supplies its tctoken child
to empty presence: the existing Rust plain tctoken byte child is confirmed.
No wire correction is needed. Complete helpers retain other merge branches
without claiming they are all reconstructed. The branch hands the node to1r0.Cyt.

Native allows a group target in its schema; Rust's existing subscription API
continues supporting people only. Empty/AD-encoded group context is now refused
locally instead of treating a namespace enum as enough identity. Native context
factory omission/failure policy and all subscription caller gates remain open.
No new group subscription or token requirement is inferred.

## Timing and producer gates remain separate

DEd message0 returns while message1 is pending. Without pending message2 it can
send immediately through DEc when0AI.A06 is true and update per-chat time/bot
maps. With message2 pending, it uses System.currentTimeMillis against the last
send: elapsed strictly greater than10000 or changed mentioned-bot state can
permit another send through the same gate. The continuation schedules message2
at2500ms. Message1/2 can emit paused only when no message0 is pending and its
selected predicate/gate permits, then remove pending1/2.

These are event-driven suppression/pause rules, not proof of a mandatory
periodic network heartbeat. The effective0AI flag, upstream activity/cancel
producers, mentioned-bot state, wall-clock changes and restart behavior remain
unported. Rust's explicit one-composing/hold/paused CLI behavior is unchanged;
no automatic timing traffic is added from bare constants.

## Rust correction and tests

chat_target, chatstate and subscribe previously checked server enum and numeric
device only. They could accept empty-user server addresses, nonzero agent with
device0, and explicit AD_JID kind with agent0/device0. A group context had only
a server-enum check. The red regression confirmed server-only composing built
a stanza. The shared named_primary check now requires nonempty user and
!Jid.is_ad(), alongside each API's existing supported server classes.
Jid.is_ad preserves explicit AD wire-kind distinctions even for zero components.
No stripping, normalization, saved identity mutation or global JID-parser change.

CLI typing/subscription parses chat_target before transport::run, so these
inputs refuse before connecting. Library builders refuse before making the
outbound node. Tests cover PN/LID/group empty users, agent-only/device-only/
AD0/0 inputs, invalid contexts, raw .agent:device spelling, and valid primary
chat/token cases. Existing codec round trips retain supported wire shapes.
The first raw fixture used underscore agent syntax; the repository text syntax
is .agent:device, corrected before final verification. No runtime defect was
inferred from that test-fixture typo.

## Bounded negatives and reproduction

Exact log-prefix xrefs found nothing because the actual literals include
suffixes such as '; toJid=' or '; jid='. Anchors composing/chatstate/subscribe
located the writers. Ime.A00's composing reference belongs to avatar UI work,
not the selected stanza reader; no incoming chatstate parser is claimed here.
UserJid constructor and DeviceJid.getServer filters found no bodies, which did
not establish absent classes; focused other methods retain the class hierarchy.
Full intermediate classes and the giant dispatcher remain local.

From repository root with built xtask and pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-presence-target-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tj;' --method A03 --signatures --exceptions > "$OUT/token-child-add.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tj;' --method A04 --signatures --exceptions > "$OUT/token-any-merge.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/DEc;' --method 'A01' --signatures --exceptions > "$OUT/composing-writer.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/DEd;' --method 'handleMessage' --signatures --exceptions > "$OUT/timer-paused.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1bc;' --method 'getServer' --signatures --exceptions > "$OUT/group-class.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/UserJid;' --method 'getPrimaryDevice' --signatures --exceptions > "$OUT/user-primary.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/DeviceJid;' --method '<clinit>' --signatures --exceptions > "$OUT/device-class.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0mL;' --method 'getServer' --signatures --exceptions > "$OUT/server-class.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tk;' --method 'A03' --signatures --exceptions > "$OUT/target-check.txt"
target/debug/xtask apk-xref "$APK" --anchor composing > "$OUT/composing-xref.txt"
target/debug/xtask apk-xref "$APK" --anchor chatstate > "$OUT/chatstate-xref.txt"
target/debug/xtask apk-xref "$APK" --anchor subscribe > "$OUT/subscribe-xref.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0v0;' --method A00 --signatures --exceptions > "$OUT/subscription-full.txt"
python3 - "$OUT" <<'RECIPE'
from pathlib import Path
import re,sys
out=Path(sys.argv[1]);lines=(out/'subscription-full.txt').read_text().splitlines()
selected=['# Selected DEX excerpt; not the complete dispatcher.']+lines[:6]
for line in lines[6:]:
    m=re.match(r'\s*([0-9a-f]+)\.\.',line)
    if m and (int(m[1],16)==0xf0 or 0x4b6<=int(m[1],16)<0x622):
        selected.append(line)
(out/'subscription-selected.txt').write_text('\n'.join(selected)+'\n')
RECIPE
```

Compare every retained .txt byte-for-byte. checksums.json covers every retained
file except itself. Shared assertion/merge evidence stays under its existing
identity. No APK, media, private state or bulk tree is committed.

Verification:1540 workspace tests across nine suites including doc tests, strict
workspace/all-target/all-feature Clippy, fmt, OpenAPI library-symbol and
CLI-reference tests, APK coverage freshness and repository hygiene passed.
All thirteen retained DEX/xref exports reproduce byte-for-byte from the APK.
Focused suite: `cargo test --locked -p whatsapp-api --all-features transport::presence::tests`.
