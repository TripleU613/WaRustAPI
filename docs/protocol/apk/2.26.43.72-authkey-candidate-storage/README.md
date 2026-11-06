# Authentication-key candidate and storage readback, APK43.72

**Confirmed static:** WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected xtask DEX exports through the supported fallback; previously diagnosed Android provider remains unavailable. No runtime observation, real key material or REA Evidence envelope claim. checksums.json binds every other file.

1nq.A00 asks1oA.A00("best") for a provider, invokes generatePrivateKey, then generatePublicKey(private), wraps private bytes in1ns and public bytes in1nk, and returns1nq(private,public).1oA maps best to org.whispersystems.curve25519.OpportunisticCurve25519Provider via reflection; actual provider selection, randomness/clamping and native implementation remain untraced. This establishes the provider family and component roles, not a new independently verified cryptographic implementation.

1nq.A02 serializes private component.A01 followed by public component.A01 via1nr.A05's sequential ByteArrayOutputStream writes.1nq.A01 accepts exactly64 bytes, splits32/32 via1nr.A06, and reconstructs first/private then second/public. It does not derive/compare a public key from the private key at this layer. The [rotation request](../2.26.43.72-authkey-rotation/request.txt) sends only1nq.A02:1nk.A01 public bytes, with a32-byte bounds-helper call; do not confuse the A02 field with the A02() full-keypair method. Candidate/public binding validation elsewhere remains unknown.

144.A0B obtains the serialized keypair and calls144.A06; it returns true after normal completion and catches RuntimeException to false. A06 calls privateA09 and, only on false, calls it once more. Two false results throw IllegalStateException("unable to write client static keypair"). A throwing first call is not caught/retried withinA06. This is at most two local storage attempts, not permission to send a second IQ or rotate to another candidate.

144.A09 encrypts the supplied bytes via149.A03, callsA0A with client_static_keypair_pwd_enc, then on a successful write callsA01(SharedPreferences), serializes the returned keypair and compares Arrays.equals with the supplied bytes. Only an equal round trip returns true. Missing/readback-different/write-failed paths reach false and diagnostics. Encryption algorithm, A0A commit semantics and A01 decryption/storage authority remain untraced. Byte equality does not establish crash durability or whole-keystore atomicity. The [callback](../2.26.43.72-authkey-rotation/callback.txt) contains multiple storage branches, so this password-backed helper is not proof all Android-keystore branches have identical readback behavior.

The complete1nq export also has destroy/isDestroyed: destroy delegates to both components once and sets its flag. Actual component byte wiping remains unknown; no zeroization guarantee follows from the names. Candidate persistence before network publication, old/new key ownership during uncertain replies, concurrent rotation and login coordination remain missing port requirements. No Rust runtime change or live account operation.

Reproduce from repository root:
```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/1nq;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1oA;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1nr;' --method A05 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1nr;' --method A06 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/144;' --method A0B --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/144;' --method A06 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/144;' --method A09 --signatures --exceptions
```
