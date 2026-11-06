# Auth-key commit and reload boundary, APK43.72

**Confirmed static:** WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected xtask DEX exports through supported fallback; previously diagnosed Android provider remains unavailable. No live operation, actual key data or REA Evidence envelope claim. checksums.json binds every other file.

144.A0A obtains keystore SharedPreferences, serializes non-null1np throughA00, then edit/putString(key,json)/commit. It returns the commit Boolean and logs false; null record or null JSON yields false. This closes the previous unknown commit call site: password-backed replacement uses commit, whereas notification counter/timestamp setters use apply. This establishes the explicit persistence API/check, not a platform power-loss guarantee or transaction across the separate key/counter/connection state writes. The [readback helper](../2.26.43.72-authkey-candidate-storage/README.md) proceeds to reload only on a true write result, then compares bytes.

144.A01 reads client_static_keypair_pwd_enc, returns null if absent, parsesJSONArray ->1no.A00 and requires record.A00==2. A mismatch logs and returns null; type2 is an expected record type, not proof of a specific cipher algorithm. It calls149.A05(record,02X.A01), returns null on null plaintext, otherwise1nq.A01 requires64 bytes and splits private/public32/32. It catchesJSONException over its selected parse range. This is not a universal exception catch; provider RuntimeException behavior remains separate.

1np.A00 writes an array starting with integerA00 then Base64.encodeToString(A02,3), followed by non-nullA03/A04 byte arrays encoded with flag3 and non-nullA01 string. These are flag literals; no platform Base64 execution was observed. The constructor fixes field positions.1no.A00 requires at least2 entries, reads index0 integer and index1 string, and for type2 additionally reads index2/3/4. It Base64-decodes byte entries with flag3. For type2 it requires each decoded byte array nonempty and string nonempty. It does not demand exactly5 elements or assert cryptographic byte sizes in this parser. Other record types have different optionality; they are not ported or exhaustively interpreted here. JSON/IllegalArgumentException parse failures return null. Fields A03/A04/A01 are not assigned guessed IV/salt/cipher names without concrete provider evidence.

149.A03([B) andA05(1np,Integer) exports have no body and are marked abstract/native by the exporter. They do not expose concrete algorithms, nonce generation, password derivation or authenticated-decryption failure policy. A concrete provider binding must be traced next; do not claim that retaining these declarations completes encryption analysis.02X.A01's semantic selector and platform SharedPreferences implementation/disk durability are not established here.

Rust need not copy Android's storage format: its encrypted transactional store is a separate local design. A compatible rotation port still needs public-key wire correctness, durably owned candidate/old-key state, matching response authority, safe current-key replacement and recovery after uncertain publication. No Rust runtime change, key migration or replay added.

From repository root:
```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/144;' --method A0A --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/144;' --method A01 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1np;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1np;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1no;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/149;' --method A03 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/149;' --method A05 --signatures --exceptions
```
