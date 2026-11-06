# Played receipt privacy gate — APK 2.26.43.72

Confirmed selected static, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Selected complete DEX exports/catches with
[REA played job](../../rea/2.26.43.72-played-privacy/README.md).
No user image, viewer, account or live request was used.

## Selection is not always played

The previously retained [complete played A0L](../2.26.43.72-receipt-registration-completion/played-job.txt)
resolves its route throughExf.A05, then0xW.A08 on the selected chat. At0048,
false branches to006a's const0; true passes through0Dn.A0a. The resulting
v7 is A08(chat) AND NOT A0a(chat). At0094/0098 a false v7 or true1dI.A00
selects played-self at01d8; otherwise009c selects played. The checked DEX
branches resolve a missing z initialization in JADX's reconstructed Java.

0xW.A08 returns true for the A0l/A0a/A0g classifier cases; otherwise it
requires09S.A1H and both privateA00/A01. Previously retained
[current read-policy Evidence](../../rea/2.26.43.72-read-receipt-policy/README.md)
checks those classifiers as types1/2 excluding28, type21 and type7, respectively.
09S.A1H reads read_receipts_enabled (compiled defaulttrue, not current state).
Ordinary numeric PN/type0 and LID/type18 are bound by the
[current user factory trace](../2.26.43.72-message-ack-chat/README.md), so they
have no shown classifier bypass of this preference. Therefore known disabled
privacy is a necessary negative condition for ordinary direct played.

1dI.A00 delegates to1dH.A0T. The complete check includes a special set branch,
A0U membership and conditional PN-to-LID lookup via0Ws.A0E; its diagnostic
identifies BlockListManager. This binds the additional predicate's concrete
implementation without claiming complete blocklist loading, synchronization,
provider identity or effective membership. Private contact/addon/spam and
dependent-account gates remain as already traced, not fully ported here.

For !v7 the job writes selected played_self_receipt rows before dispatch;
played-self plus playedSelfFromPeer returns at0214. Full self/companion routing,
coexistence, source eligibility and native retries remain separate authorities.
Do not replace played with a guessed played-self node.

## Rust correction

queue_opened now marks new viewer-completion intents automatic=true. The
existing validated server preference can hold a normal direct played intent
pending with server_read_receipts_disabled, atomically with its queued inbox
marker. The shared privacy check covers automatic read and played at queue,
reservation and owned writer handoff. Wire bytes, original selector, consumed
source and one-time ticket remain intact. This does not undo consumption.

Known-disabled pending work does not consume reservation slots. Enabling the
preference can release the same still-pending wire once. A late change after
reservation refuses the writer without releasing its outcome_unknown state.
No uncertain retry/reset, new ID or played-self substitute is added. Unknown
preferences, explicit reads, historical unknown provenance and existing group
behavior remain unchanged. Full positive contact/block policy is still absent;
a true/unknown preference is not proof of complete native admission.

## Reproduction

From repository root, pinned APK and current built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/played-privacy-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0xW;' --method A08 --signatures --exceptions > "$OUT/admission.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dI;' --method A00 --signatures --exceptions > "$OUT/block-wrapper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/1dH;' --method A0T --signatures --exceptions > "$OUT/block-check.txt"
```

checksums.json binds every other bundle file. Reused sources retain their
original checksum/REA provenance. Static parity is not proof of zero bans.
