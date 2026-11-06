# Hosted LID device99, APK43.72

Confirmed static WhatsApp2.26.43.72 code264307238, APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected DEX exports through the supported xtask fallback; previously diagnosed Android provider remains unavailable. No runtime observation or REA Evidence envelope claim. checksums.json binds every other file.

From repository root:
```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/Dv6;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0EM;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/DBh;' --method A0f --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1Xl;' --method A0x --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0EL;' --signatures --exceptions
cargo test --locked -p whatsapp-api --all-features device_hash_vectors
```

Reuse [the mapping branch](../2.26.43.72-self-device-hash-inputs/lid-map.txt): 0Dn.A04 constructs Dv6 when device==99 and 0EM otherwise. Dv6 extends0EM, stores the LID and device, and throws09P for any device other than99. Its getServer is exactly hosted.lid, getType25, and getRawString uses DBh.A0f (user plus colon), the decimal device, and1Xl.A0x (append @ and server): user:99@hosted.lid. This is a concrete domain distinction, not just a cosmetic display suffix.

The complete0EM export returns agent1, server lid and type19. Dv6 does not override getAgent in its seven exported method bodies. This is another reason not to equate APK subtype agent properties with Rust's admitted agent0 addresses. Neither complete body export contains getRawStringWithNoAgent. The separate0EL query produced `no method bodies match class filter "LX/0EL;"`; its declaration/dispatch and complete inheritance resolution are not established by this exporter. Do not promote this negative result into proof that the class is absent or that every inherited method is resolved.

[Current formatters](../2.26.43.72-device-hash-formats/README.md) preserve getServer in format2; the retained DeviceJid no-agent method also reads virtual getServer/getDevice. Thus collapsing hosted.lid into lid would change these formatter inputs. Full format3 dispatch through0EL remains an explicit gap; no new hosted digest vector is asserted.

Confirmed local source boundary: signal/adv/devices.rs primary_only, reconcile_devices and device_hash admit only Server::User or Server::Lid; reconciliation rejects a member with a different server or nonzero agent. Jid's ability to represent Server::HostedLid does not imply hash/reconciliation support. No incorrect hosted-address normalization was found in that bounded API scope. Retain these refusals rather than inventing a mixed-domain self-device collection rule.

Unknown: hosted primary/address construction outside this branch,0EL declarations, full self-device set/alias authentication and cache adoption, effective hash-version config and native membership lifecycle. No Rust behavior, request or account state changed. Static correctness findings cannot guarantee zero bans.
