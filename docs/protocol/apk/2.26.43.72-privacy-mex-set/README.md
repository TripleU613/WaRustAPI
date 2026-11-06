# MEX privacy setter construction and response admission

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK/runtime/account execution or production Rust change. DEX exports are not
new REA Evidence records; existing enum/converter evidence is reused.

## Corrective setter branch reaches this mutation

The [retained0oe.A0P](../2.26.43.72-privacy-setter-pending-lifecycle/README.md)
selects MEX through its Boolean property, creates6pB with false continuation
flag and8ho selector5. That constructor stores manager/consumer/category/value;
selected invokeSuspend024c..02a8 calls MexSetPrivacySettingsHandler.A00 with
them. Shared resume/return joins are retained; other synthetic cases stay local.
Effective classic/MEX selection remains unported, not a compiled-default guess.

A00 calls A01(category,value), passes a normal resulting map to6pB.A01 and
passes a failure to6pB.A00(null). Reuse the preceding failure/success consumers:
success reaches0oe.A0R's category-keyed admission/pending reconciliation;
this setter's false-flag failure does not retire its pending desired value.
This wrapper does not convert MEX failure into a guessed classic numeric code.

## Operation and selected input

A01 translates category/value before submission. Unknown category or setting
returns a wrapped IllegalArgumentException failure. For the corrective pair,
readreceipts selects7gM.A0K and none selects7gK.A09. Existing complete
[feature initializer](../../rea/2.26.43.72-privacy-mex-config/09-X-C7gM-clinit.json)
and [setting initializer](../../rea/2.26.43.72-privacy-mex-config/08-X-C7gK-clinit.json)
bind these to Enum.name READRECEIPTS/NONE, respectively. General feature/value
translation tables are retained in A01, not newly claimed as Rust support.

A01 constructs8gk with selector1 (v10=1), not selector0. Its other selector
contains unrelated calling UI work and is excluded. Constructor/create/invoke
bindings are retained. Selected invokeSuspend0132..01be builds a nested
GraphQlCallInput under setting, with Enum.name strings feature and setting:

```json
{"setting":{"feature":"READRECEIPTS","setting":"NONE"}}
```

This is the selected input object, not a complete final wire envelope. The
3q4.A16 helper attaches it through the input's GraphQlCallInput/Nha builder.
No own-JID field appears in this selected construction; that does not prove
absence of identity/authentication context elsewhere in the SDK. Do not copy
the separate GetPrivacySettingsQuery query_input/own-LID structure here.

The0iT call binds namespace whatsapp-android-mex, operation
**SetPrivacySettingMutation** (singular Setting), response model74U, null second
Class/function arguments and final Booleantrue. The Boolean's full SDK meaning,
actual headers/body serialization and effective timeout are not reconstructed.
The persisted-query extractor retains exact identifiers as decimal strings:

| Field | Value |
| --- | --- |
| doc_id | 26887749497493182 |
| client_doc_id | 97653612510239257036471585514 |
| operation_name_hash | 976536125 |
| operation_text_hash | 10239257036471585514 |
| schema | whatsapp_mex |

persisted-query.json binds the exact APK/asset/namespace provenance. Values
must not pass through lossy floating-point conversion. The asset record and
SDK call-name/namespace are confirmed; complete final dispatch authority is
not established by the asset alone.

## Mutation success is not just an outer IQ result

The selected response looks up xwa2_privacy_feature_update as74T, and enters
its success branch only when that model is nonnull and A0G(success) returns
true. A missing model/false result produces wrapped IllegalStateException.
The complete Boolean/object reader coercion contracts are not newly traced.

It reads feature/setting through0iM.A0D with unrecognized-enum defaults. A0D
returns null when JSONObject.isNull is true; otherwise optString feeds4gY.A00.
That helper uppercases with Locale.US before Enum.valueOf, returns the supplied
default for null/empty text or caught IllegalArgumentException, and does not
trim. Native response enum handling is not a strict-uppercase-only rule.
Full platform missing-field/Unicode/coercion behavior is outside this trace.

- Both enum objects nonnull:83S.A01/A00 convert them back to category/value.
  If both translations are nonnull, their singleton map becomes the result.
  No same-as-request category comparison appears in this selected branch;
  the downstream category-keyed pending reconciler remains separate authority.
- Either enum object null: successful mutation uses the captured original
  category/value pair for its singleton map. This branch can ignore the other
  nonnull response field; it is not a partial merge.
- Both nonnull but either translation null: wrapped translation failure. A
  present invalid/unrecognized enum is not automatically request-value fallback.
- Missing mutation model/success=false: failure, not adoption of requested values.

A01 catches Giw only around the selected asynchronous submission/result block,
logs its error-list description and wraps failure. Full other-exception, SDK
retry, deadline and uncertain-write recovery behavior remains untraced. There
is no new Rust success projection or automatic corrective send in this chunk.

## Existing Rust receipt boundary

A bounded source check of transport/receipts confirms typed queue_read accepts
only ReceiptKind::Read; opened/retry paths have their separate builders.
Read-self selection remains explicitly unported. receipt_ack_matches compares
exact type as well as ID/address/participant, so an ordinary typed read cannot
be settled by a read-self ACK. This does not establish every historical/manual
row shape or implement the separate event227 global preference effect. Those
ACKs remain captured, not magically processed by ordinary receipt settlement.

## Reproduce

From repository root with the pinned APK and rebuilt xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/privacy-mex-set-repro
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/privacy/MexSetPrivacySettingsHandler;' --method A01 --signatures --exceptions > "$OUT/translate-and-dispatch.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/privacy/MexSetPrivacySettingsHandler;' --method A00 --signatures --exceptions > "$OUT/completion-owner.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8ho;' --method '<init>' --signatures --exceptions > "$OUT/owner-constructors.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8ho;' --method invokeSuspend --signatures --exceptions > "$OUT/owner.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8gk;' --signatures --exceptions > "$OUT/builder.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0iM;' --method A0D --signatures --exceptions > "$OUT/enum-reader.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/4gY;' --method A00 --signatures --exceptions > "$OUT/enum-parse.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/3q4;' --method A16 --signatures --exceptions > "$OUT/input-attach.txt"
target/debug/xtask apk-mex-queries "$APK" --operation SetPrivacySettingMutation > "$OUT/persisted-query.json"
python3 - <<'PYCODE'
from pathlib import Path
import re
root=Path('/tmp/privacy-mex-set-repro')
lines=(root/'builder.txt').read_text().splitlines()
selected=lines[:4]
keep=False
for line in lines[4:]:
    if line.startswith('.method '):
        keep=any((' '+n+'(') in line for n in ['<init>','create','invoke'])
    if keep:
        selected.append(line)
    if line=='.end method' and keep:
        selected.append('')
        keep=False
(root/'builder-lifecycle.txt').write_text('\n'.join(selected)+'\n')
for source,name,ranges,header in [('builder','builder-selected.txt',[(0,0x1d0)],6),('owner','owner-selected.txt',[(0,0x12),(0x24c,0x2a8),(0x2f6,0x2fc),(0x330,0x33a),(0x506,0x50c)],10)]:
    lines=(root/(source+'.txt')).read_text().splitlines()
    if name=='builder-selected.txt':
        start=next(i for i,l in enumerate(lines) if l.startswith('.method public invokeSuspend('))
        lines=lines[:4]+lines[start:]
    selected=['# Selected DEX excerpt; not a complete method.']+lines[:header]
    for line in lines[header:]:
        m=re.match(r'    ([0-9a-f]+)\.\.',line)
        if m and any(lo<=int(m[1],16)<hi for lo,hi in ranges):
            selected.append(line)
    (root/name).write_text('\n'.join(selected)+'\n')
PYCODE
```

Compare complete exports, persisted record and three extracted files with this
bundle. checksums.json binds every other file by bytes/SHA256. Remaining work:
SDK final envelope/timeout/refusal policy, effective mode, durable pending and
correction ownership/recovery. No new runtime capability or REA ID is claimed.
