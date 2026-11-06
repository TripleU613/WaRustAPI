# Fetch-result selection: first direct child named list

**Confirmed pinned mismatch/correction**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX helper, reused callback/parser Evidence and synthetic
store tests. No APK/device/account or live request execution.

The retained [6p9.CK4 fetch callback](../2.26.43.72-blocklist-set-reconciliation/fetch-success.txt)
uses0Tp.A0F("list") at0014..0020. The complete helper here loops its immediate
A02 child array in order, uses TextUtils.equals against each description and
returns immediately on the first match. Absent children/no match return null.
It does not recurse, collect all lists, reject duplicate named children, or
require list to be the first child of the IQ. When a list is found, CK4 parses
that selected list with6qD.A02; no fallback to another list is shown on failure.
No-match handling is the separately retained unchanged/receive-time callback.

This differs from the [set callback's required first child](../2.26.43.72-blocklist-set-first-list/README.md).
The two result selectors must not be unified by guessing one envelope rule.
Complete earlier REA callback/parser Evidence is reused; no new REA record.

Rust previously found a list but refused if any later list existed. A valid
first snapshot followed by another or malformed list therefore stalled local
projection, even though the native selector had already returned the first.
apply_fetch_in now uses the first direct named list and validates/applies only
that selected node. Earlier unrelated direct children and later lists do not
veto it. A malformed selected first match still refuses without writes and
never falls through. Nested-only lists are not selected; no-match preserves
cached membership/hash/reconciliation under existing compatibility behavior.

The five-case fetch_uses_first_direct_named_list_without_falling_through_malformed_after_reopen
regression covers two valid lists, prefix extension then list, valid then
malformed list, malformed then valid list, and a nested-only list. It checks
selected membership/hash, reconciliation, once-only valid projection, retained
reply, journal, encrypted reopen and no pending resend. Malformed-first refusal
preserves all checkpoint bytes and uses redacted error text. The old projector
fails two_valid by returning the ambiguous-snapshot error.

Limits: selected-list identity/metadata/content validation remains this adapter's
strict policy; full native typed-null fallback/skip, dirty migration and metadata
adoption remain open. No receive-time field, sender, retry, automatic fetch or
historical repair is introduced. Old valid unprojected duplicate-list histories
can now project once using their original first direct list; already projected
state is not rewritten. This does not establish live acceptance or zero bans.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Tp;' --method A0F --signatures --exceptions > /tmp/blocklist-first-named-child.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/6p9;' --method CK4 --signatures --exceptions > /tmp/blocklist-fetch-success.txt
cargo test --locked -p whatsapp-api --all-features fetch_uses_first_direct -- --nocapture
```

Compare the callback to the linked retained fetch-success.txt and helper to
first-named-child.txt. checksums.json binds every other bundle file.
