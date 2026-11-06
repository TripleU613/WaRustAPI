# Current device hash formats and input scope, APK43.72

Confirmed static WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues diagnosed unavailable Android provider. Complete selected methods retained; no execution/live traffic or REA Evidence envelope claim. checksums.json binds every other file.

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/F2N;' --method 'A04' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/F2N;' --method 'A05' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/DeviceJid;' --method 'getRawStringWithNoAgent' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/DBb;' --method 'A0x' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/8t2;' --method 'A1D' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0jD;' --method 'A0C' --signatures --exceptions
cargo test --locked -p whatsapp-api --all-features device_hash_vectors
```

From repo root. Reuse [0x415b selector and cached hash getter](../2.26.43.72-usync-mex-device-cache/README.md). F2N.A03 chooses A05 when0x415b true and A04 otherwise. Effective config remains unknown; both explicit Rust versions remain available rather than selecting a guessed default.

A04 formats each input as user.0:device@server, sorts strings with Collections.sort, updates SHA-256 sequentially with String.getBytes (no delimiter), takes digest bytes0..6 and prefixes2: to8t2.A1D output. A05 instead formats DeviceJid.getRawStringWithNoAgent, which explicitly includes colon plus device even for0, sorts and hashes identically, and prefixes3:. DBb.A0x explicitly selects SHA-256;8t2.A1D invokes Android Base64.encodeToString with flag2. The canonical addresses in Rust's admitted scope are ASCII, so its byte encoding/lexicographic sort agree with the selected string operations; arbitrary non-ASCII/custom Jid subclasses remain outside that comparison.

0jD.A0C has self-PN and self-LID branches usingA0B/A0A then adding the corresponding current own-device address; those collection helpers and own-device policy are not reconstructed here. The non-self branch copies1nG.A09 keys and explicitly addsUserJid.getPrimaryDevice. Do not replace the own-device branches with that primary-only assumption. The precedingA07 copies the resulting set before hashing and returns an empty string for an empty/null-target path.

Rust ReconciledDevices::device_hash implements both formats over authenticated account-scoped PN/LID candidates, includes primary once, rejects cross-owner/domain/nonzero-agent inputs and keeps ADV indexes out of hash content. Its existing independent Python vectors cover PN/LID and lexicographic2/10 order, repeated primary and invalid owner controls. The targeted vector test passes against this comparison. No implementation mismatch was found in the selected explicit-format/other-user scope; no new test merely mirrors these functions.

This reanchors previously comparison-APK format evidence to43.72 and closes the two formatter bodies left unknown by the MEX cache audit. Effective0x415b, full cache/alias authentication, self-device collection policy, input snapshot coherence and MEX response adoption remain incomplete. No Rust hash/version policy, query, cache fabrication or account operation added. A matching membership hash is not identity-key authentication.
