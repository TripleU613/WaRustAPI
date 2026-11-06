# Privacy setter pending ownership is separate from IQ refusal

**Confirmed selected static / corrected offline**, WhatsApp2.26.43.72
build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports, not new REA Evidence records. No APK execution,
account operation or live privacy-setting change.

## Insert pending desired value before transport selection

0oe.A0P(category,value) constructs7wp(value) and puts it into A0E under category
at001a..0020, before checking provider66/property0x5c57.7wp constructor stores
that desired string in A00. The field census finds13 direct physical references
across the APK, including this producer, A0L/A0R and several UI readers. This is
not an exhaustive mutation/aliasing/lifecycle census: an escaped Map reference
can be mutated without another direct field access.

The true property branch schedules8ho selector5 for MEX. The false branch binds
6pB(null,manager,null,false) to8R5, constructs classic IQ set/privacy with one
privacy/category(name,value) child, typed0mL.A00 destination and a fresh A0G ID.
It calls0Sf.A0W with family69 and duration0. This does not establish an effective
property value, MEX envelope, real write or infinite callback lifetime; selected
classic duration0 does not add a positive message4 expiry timer. The shared
[callback/timer registration evidence](../2.26.43.72-abprops-callback-timer/README.md)
remains the timer authority. After selected dispatch/scheduling it calls A0Q on
the category singleton and notifies8OA selector2. Downstream optimistic UI/local
state and category callers are not reconstructed in this chunk.

## Setter errors do not retire this native pending entry

Fresh complete8R5 class matches the
[retained callback](../../rea/2.26.43.72-account-privacy-refresh/smax-callback-dex.txt)
byte-for-byte. Bvj scans direct error children for its first nonnull code,
converts through0Cd.A06 and calls6pB.A00(Integer). Exact nullable numeric grammar
is outside this trace; it must not be replaced with the blocklist default0 rule.
The flag census finds four direct physical references, with its constructor
the only selected writer; no later direct DEX field write was found. Reflection/
native mutation is not reconstructed.6pB.A00 first checks A01. Constructor
binding proves A0P supplied false, so it
returns at0022 without touching manager/A0E, invoking a continuation, adopting
server state or scheduling refresh. The same setter's BsS also has the false
A01 gate. No pending-category retirement follows from these selected failure
callbacks. An outer refusal is not ownership/reconciliation completion.

Conversely8R5.CK4 requires its first child be privacy, builds a map and calls
6pB.A01, which invokes0oe.A0R before its optional continuation gate. The retained
[complete A0R](../../rea/2.26.43.72-account-privacy-refresh/privacy-map-adoption-dex.txt)
compares the final server value with a category's pending7wp.A00: contradictory
admitted values skip without removing it; matching values remove before dispatch.
Invalid/unsupported values have another removal branch. Whole native pending
lifecycle, every UI/alias writer and durable crash recovery remain unported;
this does not say that an entry can never be cleared elsewhere.

## Rust correction

Rust has no typed privacy setter or equivalent native pending map. Its existing
local guard detects generic queued category-setter actions and holds fresh
readreceipts adoption/ACK until that ownership can be reconciled. Previously
ActionStatus::Error plus a saved outer IQ error bypassed the guard, permitting
a later notification to overwrite local state and be ACKed solely because the
network request was refused. The selected native failure callback provides no
such authority. The guard now holds all recognized unowned category setters,
including true refusals and legacy misclassified errors. It does not rewrite
the saved refusal/status or retry the action. Successful/queued/uncertain holds
and already-processed notification redelivery ACKs keep their existing rules.

This is a bounded protection while full setter ownership/reconciliation is
missing, not a completed native setter or proof every generic custom request
has the same semantics. The guard remains broader than native category-keyed
pending entries. No fabricated desired value, manual reset, automatic refresh,
state replacement or generic account-sync ACK is added.

## Verification cases

The updated outer_refusals_do_not_retire_unowned_privacy_setter_intent
regression fails on the previous bypass because it returned an ACK. It now
checks actual503 and legacy misclassification, unchanged checkpoints/preference,
reopen persistence and no requeued action. A production-session regression
preserves an outer503, captures a contradictory fresh readreceipts notification
without ACK/adoption, then services a ping and reopens without clearing the
hold or replaying the setter. Existing processed-redelivery/ACK-cut tests retain
their earlier authority; this change does not revoke a previously processed ID.

```sh
cargo test --locked -p whatsapp-api --all-features outer_refusals_do_not_retire_unowned_privacy_setter_intent
cargo test --locked -p whatsapp-api --all-features refused_privacy_setter_holds_fresh_notification_without_ack_after_reopen
```

## Reproduction

From repository root with rebuilt xtask and pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/privacy-setter-pending-lifecycle-repro
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/0oe;' --method A0P --signatures --exceptions > "$OUT/setter.txt"
target/debug/xtask apk-field-refs "$APK" --class 'LX/0oe;' --field A0E --type 'Ljava/util/Map;' > "$OUT/pending-field-refs.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/7wp;' --method '<init>' --signatures --exceptions > "$OUT/pending-value.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6pB;' --method '<init>' --signatures --exceptions > "$OUT/consumer-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6pB;' --method A00 --signatures --exceptions > "$OUT/consumer-error.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6pB;' --method A01 --signatures --exceptions > "$OUT/consumer-success.txt"
target/debug/xtask apk-field-refs "$APK" --class 'LX/6pB;' --field A01 --type Z > "$OUT/consumer-flag-refs.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8R5;' --signatures --exceptions > "$OUT/callback.txt"
```

Compare the first seven exports to this bundle and callback.txt to the linked
retained class. checksums.json binds every other bundle file by bytes/SHA256.
Branch operands are code units:6pB.A00's0004+0xf*2 reaches0022 return;
0oe.A0P's003c+0x43*2 reaches00c2 classic branch. No live acceptance claimed.
