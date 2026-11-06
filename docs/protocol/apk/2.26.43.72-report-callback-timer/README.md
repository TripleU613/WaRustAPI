# Individual report callback timer

**Confirmed selected static / implemented offline**, APK2.26.43.72
build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Focused DEX exports, not runtime observations or new REA Evidence records.
No report or live account operation was performed.

## Producer and duration

IndividualSpamReportRpc.A00 builds report data and calls the generated
SpamRPCManager.A01. The manager constructs Dzb's exact retained eight-argument
Jid/Dz0/Dyx/Dyx/String/List/List/List overload, then submits its node and
request ID to MessageClientSmaxWrapper.A00. The caller-supplied int is forwarded
as the logical request discriminator; it is not the duration.

The wrapper supplies the literal32000 (0x7d00) and false to0Sf.A0F. A0F
constructs Cgd with selector2 and the continuation. The false branch uses
0Sf.A0A, then explicitly registers0ln.A08 on pending fallback. Reuse
[shared registration/timeout evidence](../2.26.43.72-abprops-callback-timer/README.md):
A08 posts handler message4 only for positive durations, and expiry removes
callback ownership and invokes BsS. The new Cgd constructor binds selector and
continuation; its complete [BsS](../2.26.43.72-default-timer-callback/get-failure.txt)
was re-exported and matches byte-for-byte. Selector2 resumes with BWM.
The wrapper maps BWM to null; manager null returns its wrapped failure variant.
A lost/expired callback is not proof of remote refusal or permission to resend.
Effective queue readiness, native coroutine cancellation and restart policy are
not reconstructed by this selected route.

## Wire and response evidence, with limits

Dzb constructs type=set, generated ID, spam_list and its optional jid, plus
three data lists whose upper constants are210/5/5. Lower bounds come from
B23.A03's returned long; they were not newly traced here. The attached primary
Dyx fragment is produced by DzW.A02 with selector5. That constructor's non3
branch builds xmlns=spam, spam_list, optional wi_trace_id and spam_flow with a
100-entry schema list. The existing Rust seven allowed flow values are a local
subset, not a complete current native vocabulary. No new flow, message child,
trace ID or optional data is fabricated. Other synthetic constructors stay local.

The complete Rv2 response constructor is retained because the manager first
tries selector1 success, then catches LIL and tries selector0 error, retaining
schema failures separately. Shk.A00 locates report/id via the generated schema
reader, with optional/null behavior and upper constant20; when present it creates
7NL. IndividualSpamReportRpc obtains7NL.A01 or null and constructs7UQ. The
previous unknown report-ID path is thus located, but complete correlation/source,
length units, schema coercion and native error-model adoption remain unported.
No report-ID value is persisted or claimed by this correction. Generic action
Ok is an envelope outcome, not native report acceptance or moderation action.

Bounded discovery negative: spam_flow string xref found six sites; most are
UI/Falco/bundle uses. 82x.A00 only puts UI arguments into a Bundle and does not
establish request requirements. Exact string searches for SpamRPCManager and
IndividualSpamReportRpc found no constant xrefs; class-substring apk-smali
located the real named classes instead. No unrelated UI method is committed.

## Rust correction and verification

callback_seconds now assigns32s to matching spam/set with one spam_list carrying
a user-like jid and textual spam_flow. These are the supported individual report
wire forms, not every IQ or a kind-label-only rule. The existing reservation-time
monotonic deadline and write guard remain authoritative. Expiry removes only
live callback ownership; the durable Sent row stays uncertain, preserves the
original request and is not reserved again after reopen. No automatic resend,
new error status, response projector, report sender or HTTP route was added.

The red regression observed no deadline before correction. Green paused-clock
coverage accepts result/error at31,999ms, expires at32,000ms, rejects late reply,
preserves the unsettled row/reply absence and survives encrypted-store reopen
without resend. A ping labeled report keeps its own untimed policy. No live
account operation or inferred cause of account restrictions.

## Reproduce

From repository root with built xtask and pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-report-timer-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/reportinfra/rpc/IndividualSpamReportRpc;' --signatures --exceptions > "$OUT/rpc-full.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/smax/generated/spam/outgoing/SpamRPCManager;' --method A01 --signatures --exceptions > "$OUT/manager.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/xmpp/messaging/MessageClientSmaxWrapper;' --method A00 --signatures --exceptions > "$OUT/wrapper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Sf;' --method A0F --signatures --exceptions > "$OUT/suspend-send.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Cgd;' --method '<init>' --signatures --exceptions > "$OUT/callback-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Cgd;' --method BsS --signatures --exceptions > "$OUT/timeout-reused.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Dzb;' --method '<init>' --signatures --exceptions > "$OUT/request-full.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Rv2;' --method '<init>' --signatures --exceptions > "$OUT/response-model.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Shk;' --method A00 --signatures --exceptions > "$OUT/success-body.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/DzW;' --method A02 --signatures --exceptions > "$OUT/flow-builder.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Dyx;' --method '<init>' --signatures --exceptions > "$OUT/flow-full.txt"
target/debug/xtask apk-xref "$APK" --anchor spam_flow > "$OUT/flow-xref.txt"
python3 - "$OUT" <<'RECIPE'
from pathlib import Path
import sys,re
out=Path(sys.argv[1])
for dest,src,signature in [
 ('rpc.txt','rpc-full.txt','Lcom/whatsapp/reportinfra/rpc/IndividualSpamReportRpc;->A00('),
 ('request.txt','request-full.txt','LX/Dzb;-><init>(Lcom/whatsapp/infra/core/jid/Jid;LX/Dz0;LX/Dyx;LX/Dyx;Ljava/lang/String;Ljava/util/List;Ljava/util/List;Ljava/util/List;)V'),
 ('flow-fragment.txt','flow-full.txt','LX/Dyx;-><init>(Ljava/lang/String;Ljava/lang/String;I)V')]:
    blocks=[b for b in re.findall(r'\.method .*?\.end method',(out/src).read_text(),re.S) if signature in b]
    assert len(blocks)==1
    (out/dest).write_text('# Selected complete DEX method; other overloads remain local.\n'+blocks[0]+'\n')
RECIPE
```

Compare all retained .txt exactly; timeout-reused.txt matches the linked existing
Cgd.BsS export. Full intermediate overload exports stay local. checksums.json
covers every retained file except itself. No APK, message content, raw account
state, media or bulk class tree is committed.

Verification:1538 workspace tests across nine suites including doc tests, strict
workspace/all-target/all-feature Clippy, fmt, OpenAPI library-symbol and
CLI-reference tests, APK coverage freshness and repository hygiene passed.
All eleven retained .txt exports and the reused Cgd.BsS method reproduce exactly.
Focused regression: `cargo test --locked -p whatsapp-api --all-features report_callback_expires_at_native_timeout_without_replay`.

## Response/bounds follow-up (2026-11-06)

[Exact helper trace](../2.26.43.72-report-response-bounds/README.md) resolves the
list lower bounds to0 and report/id length units to UTF16. It also traces
request-ID equality and nullable report model/source behavior, and corrects
unrelated result-error extensions. Full native schema/source admission and
report application remain unported.
