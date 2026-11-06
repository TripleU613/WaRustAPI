# Blocklist set-result first-child admission

**Confirmed pinned mismatch/correction**, WhatsApp2.26.43.72 build264307238,
APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static selected DEX plus synthetic encrypted-store tests; no live request.

The retained [8R7.CK4 callback](../2.26.43.72-blocklist-set-reconciliation/set-success.txt)
first calls0Tp.A0D, requires tag list, and parses6qD.A02 **before** reading matched.
A0D's complete export here returns children[0], or throwsLIL for absent/empty
children. It does not search for any later list. The
[complete parser](../2.26.43.72-blocklist-snapshot-children/parser.txt)
gets dhash from this selected list, validates every item tag, and produces80Z.
Its A01 hash then goes to the selected callback branch. Complete prior REA
callback/parser Evidence is reused; no duplicate REA record is added.

Rust previously accepted an empty set result, searched every list for
matched=false, and searched root/immediate-child hashes. That could apply an
unproven requested change/hash, select a later result branch or ignore malformed
list contents. Now it requires the first child to be list, validates that list
through the same side-effect-free snapshot parser used by fetch, then branches
on only its matched value. Only its dhash is admitted; root/trailing hashes do
not override it and later matched=false cannot redirect a normal first list.

The existing matched=false hold remains: no requested-change/hash adoption,
once-only inspection marker and reconciliation warning. Malformed results
refuse projection with fixed redacted errors before any durable cache/hash/
marker/reconciliation writes. Recorded IQ settlement remains retained; no wire
request is recreated. Native first-list selection does not require rejecting
all trailing children; this correction does not invent that restriction.

The new regression covers block and unblock with absent child, wrong first tag
followed by list, unknown list child, unknown child with matched=false, invalid
identity, normal first list followed by an unmatched/conflicting later list,
root-only hash and admitted unmatched first list. It verifies encrypted-store
reopen, immutable settled reply/status, all checkpoint bytes and journal on
refusal, selected hash/membership, once-only valid projection and no pending
resend. The old projector fails the first absent-child case by returning Ok(1).
Existing positive set fixtures now include the required list; they no longer
enshrine the unsupported no-list acceptance.

Limits: snapshot validation deliberately retains this adapter's strict primary
identity/metadata/content policy. Native typed-null fallback/skip and separate
metadata/alias stores remain gaps; no claim of full6qD admission parity.
Fetch no-list/unchanged compatibility is unchanged. No-list old saved set
histories now refuse rather than automatically skip/quarantine/repair; already
projected historical state is not rewritten. Full migration and live provider
acceptance remain unproven. No new sender, retry or zero-restriction guarantee.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Tp;' --method A0D --signatures --exceptions > /tmp/blocklist-set-first-child.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/8R7;' --method CK4 --signatures --exceptions > /tmp/blocklist-set-callback.txt
cargo test --locked -p whatsapp-api --all-features set_result_first_list_admission -- --nocapture
```

Compare the callback to the retained set-success.txt, and the first-child method
to required-first-child.txt. checksums.json binds every other bundle file.
