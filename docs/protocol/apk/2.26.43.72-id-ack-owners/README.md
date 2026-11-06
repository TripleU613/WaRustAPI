# ID-based ACK records: selected owners and reconnect — APK 2.26.43.72

**Confirmed selected static**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete DEX methods/catches, exact direct field/caller
censuses; xtask fallback, not REA Evidence envelopes or runtime evidence.
No account operation or native resend is executed.

## Bounded producer census

The prior [generic ACK trace](../2.26.43.72-receipt-callback-key/README.md)
found independent0TV ID maps after the full-key0Th lookup. A05 has five direct
DEX references:0Sf.A04, constructor,0TV.A01,0YB.A01 and0m2.handleMessage.
The collection can be passed through aliases; this is not a proof about all
reflection, dynamically loaded code, native access or runtime dispatch.

0Sf.A04 takes Message, nullable1ue callback,0Sf, id, long and tracking boolean.
When readiness0AI.A06 is true, tracking=false jumps from004a to0162 and submits
without adding an ID record. Tracking=true uses challenged A04 if that id is
already there, otherwise A05; it creates Eo7(original Message, supplied or
previous callback, incremented/default-one count). Positive supplied time
schedules0ln.A06 before submission. When readiness is false it retains
EVD(Message,callback,id,tracking) in0TX.A04, with a separate timed branch.
These readiness/submission steps do not prove bytes written or server success.

A0L calls A04 with tracking=true and time0, callbacknull. A0M passes false.
The six direct A04 callers are A0L,A0M,0YB.A01,14E.A05,EzI.A02 andFdv.run.
14E.A05 andFdv.run supply false, so those shown calls cannot create this
tracked A05 record on the ready branch. Their full wire meanings are not
inferred from Message selectors61/85. EzI.A02 takes EV7/VoipStanzaChildNode;
configured offer/accept paths can supply a callback, positive time and true,
while its other shown path calls A0L. This confirms a VoIP owner, not an
ordinary receipt producer.

A0L's three direct callers are A08(map resubmission),0YB.A01 andEzI.A02.
A0M's seven direct invocations are retained too. The tracked insertion path's
selected original producer is thus bound to VoIP through this direct census;
no ordinary receipt producer was established. This is a bounded static result,
not universal exclusion of every runtime producer.

## Reconnect and challenge records have their own policy

0YB.A01 at055e..05ba obtains A05, calls0TV.A00 to select a map, then0Sf.A08;
it separately selects challenged records through0TV.A03 and submits those
through A08 too. A08 iterates id/Message entries and calls tracked A0L.
Later0656..06b8 it drains pending EVD records using their tracking/callback
fields. This is native resubmission machinery; it is not authorization to
replay any Rust uncertain receipt or chat send.

Eo7.A00 only extracts EV7 when Message.arg1=206.0TV.A00 uses that extraction,
VoIP tag branches (including offer/terminate), attempt count, config0x3c4a and
additional provider policy. Complete selected method is retained; effective
config, runtime eligibility and all consequences are not emulated.0TV.A03
runs that selector on challenged A04.0TV.A02 first removes from active A05 via
A01 and, if absent, falls back to challenged A04 under its monitor. The previous
auth-challenge bundle retains A01; no account-wide recovery claim is added.

## Rust comparison / negative finding

Strict receipt correlation is not loosened and no resend is added. The mere
presence of an ID-only native callback map did not establish that ordinary
receipt completion should be ID-only. Full-key callback ownership, observed
server ACK, specialized receipt state and this tracked native queue are distinct.
Native VoIP/challenge/reconnect emulation and all external write/recovery policy
remain incomplete. No confirmed Rust construction mismatch was established in
this bounded owner audit; no runtime code is changed.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-field-refs "$APK" --class 'LX/0TV;' --field A05 --type 'Ljava/util/LinkedHashMap;'
target/debug/xtask apk-callers "$APK" --class 'LX/0Sf;' --method A04
target/debug/xtask apk-callers "$APK" --class 'LX/0Sf;' --method A0L
target/debug/xtask apk-callers "$APK" --class 'LX/0Sf;' --method A0M
target/debug/xtask apk-smali "$APK" --class 'LX/0Sf;' --method A04 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Sf;' --method A0L --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Sf;' --method A0M --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Sf;' --method A08 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0YB;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/EzI;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/14E;' --method A05 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Fdv;' --method run --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Eo7;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0TV;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0TV;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0TV;' --method A03 --signatures --exceptions
```

checksums.json covers every other file. Other complete evidence is linked
instead of duplicated. Static findings cannot prove account restrictions.
