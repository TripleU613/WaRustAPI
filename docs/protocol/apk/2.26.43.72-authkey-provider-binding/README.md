# Authentication-key local provider binding, APK43.72

**Confirmed static:** WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected xtask DEX method/class exports through supported fallback; previously diagnosed Android provider remains unavailable. No runtime execution, key data or REA Evidence envelope claim. checksums.json binds every other file. Only the specified dependency slot and crypto paths are interpreted, not every factory case.

## Concrete construction chain

144 constructor reads00T.A03(0x1a3e), casts149 and storesA0F.00T.A03 obtains06o and invokesA04(null,context,id); A04 enters the supplied context's binding resolver and calls1Uw.A00(id,...).1Uw.A00 computes(id>>14)&0x7f;0x1a3e selects0 ->A01. A01 masks0x3fff, skips lower ranges and at0e5e..0e70 selectsA0H for0x193b<slot<=0x1a54. A0H's packed switch maps6718(0x1a3e) to0bfe, constructing14A. These explicit masks/ranges/switch targets close the binding; nearby slots create different classes and are not interchangeable. Branch displacements are code units; displayed PCs are bytes.

14A extends149. Its constructor gets14C at dependency0x1a43. A03 delegates14C.A01(0cJ.A0Y,plaintext); A05 delegates14C.A02(record,0cJ.A0Y). The Integer argument changes only the failure diagnostic between READ_SELFTEST/READ_ACTIVE in this concrete body; it is not passed into decryption or used to pick a different key. Value/initialization of0cJ.A0Y remains untraced. Runtime dependency overrides/global resolver lifecycle remain outside this static construction path.

## Crypto record fields established by data flow

14C.A01 allocates random arrays of4,16,16 bytes through147.A00 SecureRandom and nextBytes. It Base64-encodes the last16 bytes with flag3 as a string suffix, concatenates the supplied string plus that suffix, and calls privateA00 with the4-byte array as salt. A00 obtains String.getBytes, converts each signed byte to char, builds PBEKeySpec(chars,salt,16,128), and uses SecretKeyFactory PBKDF2WithHmacSHA1And8BIT. It wraps the derived encoding with AES SecretKeySpec before returning encoded bytes. Provider-specific byte/character semantics are not independently executed or verified.

Encryption requests Cipher AES/OFB/NoPadding, uses the random16-byte middle array in IvParameterSpec, and doFinal(plaintext). Constructor registers v3..v8 map to1np(stringSuffix,ciphertext,IV,salt,type2); the earlier [record constructor and JSON parser](../2.26.43.72-authkey-commit-reload/README.md) bind A01=suffix,A02=ciphertext,A03=IV,A04=salt,A00=2. Thus the final encoded JSON order for this branch is type,ciphertext,IV,salt,suffix. These field roles follow actual calls, not array-size guesses.

14C.A02 rejects record type other than2, derives from supplied string+record suffix with record salt, uses record IV and decrypt mode2, and doFinal(record ciphertext). Selected checked crypto exceptions return null; decryption additionally catches NullPointerException. Encryption's catch list is retained and differs. Neither complete body performs an explicit MAC/tag creation or verification; this bounded negative result does not establish platform integrity, successful key import or a recommended storage construction. Do not label the type2 record as authenticated ciphertext based on decrypt success alone.

## Port implications and remaining scope

This closes concrete password-backed helper roles left unknown by the prior audit. It does not port Android keystore/PWD encryption into Rust, which has its own encrypted transactional storage. The wire publication still carries only the public component. Matching Android's local ciphertext is not a protocol requirement. Candidate ownership, crash-safe old/new key selection, uncertainty across network publication and future-login coordination remain the reliability work.147 random-source construction,0cJ.A0Y initialization, actual platform crypto/provider behavior, other keystore branches and dependency lifecycle remain unverified. No Rust runtime change or live operation.

Reproduce from repository root:
```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/144;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/00T;' --method A03 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/00T;' --method A04 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1Uw;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1Uw;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1Uw;' --method A0H --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/14A;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/14C;' --signatures --exceptions
```
