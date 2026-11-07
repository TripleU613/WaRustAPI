# MobileConfig upgrade dispatch and schema/hash gates

Follow-up: [reader framing and hash-prefix evidence](../2.26.43.72-mobileconfig-reader-framing/README.md)
resolves 0xa8fa4 to the prefix before the first colon, and binds the reader's
framing/verifier handoff. The original unknowns below describe this bundle's
bounded trace; complete body verification and installation remain unresolved.

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution, network request or saved-state mutation.

## Resolved methods are upgrade paths

Following the [restore continuation](../2.26.43.72-mobileconfig-restore/README.md),
the selected StorageManager vtable binds +0x90 to **0x84a24**, whose trace string
is `FBMobileConfigStorageManager.upgradeAndSwapFlatbuffer`, and +0x98 to
**0x85060**. Slot+0xa0 binds to **0x85164**, trace string
`FBMobileConfigStorageManager.upgradeSchema`. These names describe the selected
code, not proof that any restored configuration was activated.

Both first obtain a string/associated word through virtual slot+0xe0 (selected
target 0x878e0, body untraced). Treat it as selected state, not a verified current
file. Both use **0x8bda0**, whose diagnostic names it `isSchemaMismatch`, but
they inspect its result differently. A boolean interpretation would lose a
material branch condition.

## Schema helper returns more than a boolean

Helper 0x8bda0 calls 0xa2a70 with selected input, manager flags and numeric mode
10. That reader's validation is not established here. It obtains a RequestData
string through 0x5faec and a table-derived string through 0xa3e58, then sends
each through **0xa8fa4** into temporary strings. It compares those temporaries
through 0x531f8. The selected return contributes **0x100 when unequal**, zero
when equal. The transformation performed by 0xa8fa4 remains unknown; do not
equate this comparison with raw wire hash equality.

The lower return portion has separate schema/hash/count logic. A table pointer
must expose an offset through its table metadata, and its selected 32-bit field
must equal **0x1e240 (123456)** before the richer comparison branch. That numeric
field's semantic name is unproven. In that branch the helper also compares the
raw RequestData/table strings, examines manager bytes +0x230/+0x274, and under
selected flags compares a RequestData-derived sum with five reader-produced
32-bit fields. Count helper 0x8ceb0 first refreshes through 0xaac90 and sums four
32-bit words at RequestData+0x19c; the other sum uses five fields. Counter
categories, overflow domain and complete reader checks remain unknown.

Missing selected table metadata, a different numeric field, raw hash inequality
and count inequality reach different branches. This report retains the full
helper but does not collapse those branches into a universal validator or assign
all manager flags public meanings. Its result is a bit-bearing integer, not a
server verdict, and the callers below remain authoritative about each used bit.

## Caller gates differ

In **0x84a24**, the shortcut requires:

- result **bit 0** set;
- manager byte+0x268 bit 0 set, or manager word+0x26c equal to 2 or 3;
- helper **0x8bfe0** returns low-bit true for a derived path and selected index.

When the shortcut succeeds it returns internal code 12 for word+0x26c equal to
3, otherwise 5. If any shortcut condition fails, it proceeds to virtual+0xa0
(0x85164), passing result bits 0 and 8 separately plus an output-reason byte.
Thus a false shortcut result is not itself final load rejection. A missing
initial string instead sets manager+0x2f8, records a diagnostic and returns the
selected internal code 1.

In **0x85060**, an empty initial string returns code **9**. Otherwise it first
stores the associated word at manager+0x18, then requires **low 16 bits of the
schema result >= 0x100**, plus the same manager flag/mode condition. Failure
returns **10**. It does not substitute a test of result bit 0 for that numeric
comparison. Passing invokes 0x8bfe0; low-bit true returns **13** (mode 3) or
**14** (other mode), while false returns **15** or **16**, respectively. These
are internal load/upgrade codes, not HTTP statuses or established Rust enums.
The early manager+0x18 write precedes the gate and cannot prove activation.

Helper 0x8bfe0's diagnostic is
`FBMobileConfigStorageManager::buildLatestConfigWithTranslationTable`. Its
retained entry checks RequestData availability, constructs a file reader via
0x88144 with numeric mode 9, invokes reader slot+0x10 and follows source
selection through 0x8b8bc. Its large remaining body is not modeled here. A true
return is therefore reported only as a branch outcome, not verified active state.

## Main upgrade failure has cleanup and diagnostic effects

After virtual+0xa0 returns, 0x84a24 stores 1 with release ordering into global
0xdba78 regardless of the low-bit outcome. On the true branch it invokes
virtual+0xd0 (selected target 0x873ec, body untraced), writes the selected index
to manager+0x18, and attempts a string-file write through the previously traced
0x79dfc helper; its return is not tested in that selected continuation.

On false, it stores 1 with release ordering into global 0xdba74. A particular
reason byte 7 reports a soft error and sets manager+0x60. It then calls the
previously traced **0x8cee8 cleanup helper** with `upgrade_failure`, sets
manager+0x2f8 to 1 and diagnostic `UpdateFailed`, and reports
`mobile_config_sync_fetch_upgrade_failed` / `schema_upgrade_failed`. The final
returned code is the reason byte from the upgrade helper. Neither global byte
is a demonstrated success flag; one is written on both outcomes.

The retained 0x85164 entry calls 0x97d08, 0x8ede0 and 0x8f3fc, then generates
two local objects through 0x91198/0x902a4. One selected branch writes them via
storage virtual+0xb0 before later work. Its retained finishing region invokes
0x8acd0, constructs an internal result, calls 0x8ada0 and records timing and
status. The middle paths, object identities and installation effects are still
untraced. In particular, 0x8ada0 itself has conditional file/helper work and a
missing-manager-member error path; it is not sufficient evidence of activation.

## Reproduction and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-upgrade-dispatch/inspect-upgrade.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-upgrade.json
cmp /tmp/mc-upgrade.json docs/protocol/apk/2.26.43.72-mobileconfig-upgrade-dispatch/upgrade.json
```

The inspector pins member/ELF identity, strings, selected calls and RELA bindings.
Dispatch examples are analyst projections, not native loading/activation tests.
LLVM retains whole outer dispatch/schema helper regions and bounded inner
upgrade/build/follow-up windows; all exceptions and callees are not modeled.
Nearest JNI_OnLoad headings are not local function identities.

rea-upgrade.json retains complete REA Evidence
`ev_f45acee9bd210b5392d5ad9656222f231d66d56b6e01b207f186dfb8f6c7e9c8`,
command `e scr.color=0; pd 44 @ 0x85098`, exit 0, untruncated output, unchanged
member hash and an entrypoint warning. It is a bounded dispatch window; provider
version/external inputs are not bound and annotations are not runtime reads.

**Remaining:** file-reader validation and normalized-string helper, numeric
schema-field identity, flag producers, complete build/upgrade/installation body,
durable active-state admission, startup callers and retry/account ownership.
No Rust runtime or saved account state changed.
