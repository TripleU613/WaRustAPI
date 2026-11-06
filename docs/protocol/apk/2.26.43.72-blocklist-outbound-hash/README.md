# Blocklist outbound hash: distinct get/set omission rules

**Confirmed selected APK mismatch/correction**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static APK construction plus synthetic Rust/store and controlled host-Java
reference. No APK/device/account or live request executed.

## APK rules differ

[1dH.A0M get](../2.26.43.72-blocklist-callback-timing/query.txt) reads09S.A0c:
null or String.length()==0 omits the item. Nonempty hashes, even whitespace-only,
are carried verbatim in item.dhash. It does not apply the set blank predicate.

[1dH.A05 set](../2.26.43.72-blocklist-callback-timing/set.txt) reads09S.A0c into
v21 at0404;0522..054a omits dhash when null or0Cf.A0q is true. This selected
construction is shared by block/unblock. A nonblank hash is not trimmed before
putting it into the attribute. The
[complete blank/character predicates](../2.26.43.72-blocklist-metadata-adoption/README.md)
show empty or all UTF16 units satisfying Character.isWhitespace OR isSpaceChar.

The distinction includes U+001C..001F (blank under this Java predicate) and
U+0085 (not blank), so blindly using Rust str::trim/is_whitespace would change
behavior. NBSP is blank, zero-width space is not. A string with nonblank content
and surrounding spaces is sent unchanged. Surrogate units are not blank.

## Independent controlled reference and Rust correction

BlankUnits.java is an authored fixture that invokes the installed Java21
character APIs for all65,536 UTF16 units. It emits the sorted true-unit list in
blank-units.json; it does not encode expected ranges itself or run APK code.
Executed with Temurin21.0.13+9.0.LTS. Rust's private predicate is checked against
the entire reference domain, including false units; whole-string UTF16 all()
preserves empty-string semantics. This is a controlled Java21 reference, not
actual Android/OEM execution or proof of every historical Unicode table.

queue_fetch now filters only absent/empty saved hashes. queue_set filters absent/
blank hashes with the measured predicate. Saved hash bytes are never trimmed,
rewritten or deleted by construction. Nonblank values retain exact content;
get retains nonempty whitespace. Snapshot persistence semantics are unchanged.

The outbound_hash_presence_matches_distinct_get_and_set_predicates regression
covers absent/empty, ASCII whitespace, NBSP, U+001C, U+0085, zero-width space,
supplementary emoji, surrounding whitespace and ordinary text. Each synthetic
store queues get/block/unblock, reserves locally and inspects exact attributes
and unchanged checkpoint. The old code fails the empty-hash get case by creating
an item. No request is transmitted. A second test compares all UTF16 units with
the independent emitted fixture.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1dH;' --method A0M --signatures --exceptions > /tmp/blocklist-get-hash.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1dH;' --method A05 --signatures --exceptions > /tmp/blocklist-set-hash.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Cf;' --method A0q --signatures --exceptions > /tmp/blocklist-blank.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Fv;' --method A01 --signatures --exceptions > /tmp/blocklist-space.txt
/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS/bin/java docs/protocol/apk/2.26.43.72-blocklist-outbound-hash/BlankUnits.java > /tmp/blocklist-blank-units.json
cmp /tmp/blocklist-blank-units.json docs/protocol/apk/2.26.43.72-blocklist-outbound-hash/blank-units.json
cargo test --locked -p whatsapp-api --all-features outbound_hash_presence
cargo test --locked -p whatsapp-api --all-features blank_hash_units
```

Compare the four DEX exports to the linked retained query/set/blank/space files.
checksums.json binds every other bundle file; no duplicate REA or DEX copy is
added. Native alias/migration/metadata parity, set-result hash gates and actual
provider acceptance remain open. This does not claim zero restrictions.
