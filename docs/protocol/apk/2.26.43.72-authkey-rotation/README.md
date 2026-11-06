# Auth-key rotation notification lifecycle, APK43.72

**Confirmed static:** WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues diagnosed unavailable Android provider. Complete selected exports retained; checksums.json binds every other file. No APK execution, account operation or REA Evidence envelope claim.

## Handler and ACK boundary

[Existing notification type map/census](../2.26.43.72-notification-handler-census/README.md) binds authkey_rotation to248 and19h. The complete19h constructor registers248 with0v6 executor enabled. A09 checks248, invokes144.A0G(5), then19i.A00; it does not inspect the notification payload or callA07 to defer acknowledgement. Reuse [base dispatch](../2.26.43.72-count-content-error-ack/README.md): normal non-deferred completion reaches the base ACK afterA09 returns. That is not proof the subsequent IQ succeeded or that key persistence completed. Exceptions and base metadata admission retain their separately traced rules.

144.A0G clamps its integer to0..5 and calls SharedPreferences keystore/edit/putInt/apply for remaining_auth_key_rotation_attempts. Android apply invocation is not proof of a synchronous durable fsync. Repeated notifications resetting this counter are not permission for a Rust caller to reset uncertainty or retry a key publication.

## Request and timing gates

19i.A00 returns when remaining attempts<=0. It computes now through08q.A00, reads last_succeeded_auth_key_rotation_attempt (default-1), and returns when now-last_success<86400000. It reads last_failed_auth_key_rotation_attempt(default-1): the sentinel bypasses that check; otherwise now-last_failed must be strictly greater than1800000.08q.A00 prefers elapsedRealtime plusA03, then elapsedRealtime plusA04, otherwise currentTimeMillis minusA02. These are millisecond clock inputs; offset provenance, clock rollback/reboot behavior and overflow policy remain untraced. Do not substitute a freshly invented wall-clock deadline.

After the gates,19i reuses itsA00 candidate1nq or creates it with1nq.A00. Candidate generation and secret representation are not reconstructed. It allocates an IQ ID with0Sf.A0G and constructs iq/to=0mL(server s.whatsapp.net)/xmlns=w:auth:key/type=set/id, with a key child carrying candidate.A02:1nk.A01 bytes.0Tk.A04 is called with32/32 bounds on those bytes; assertion semantics are not inferred as a normal Boolean rejection. It creates2IS(144,19i,candidate) and calls0Sf.A0W with command331 and32000 delay, ignoring its Boolean return. This is a send/response boundary, not proof that exactly32seconds bounds the complete lifecycle or authorizes a resend. Retaining/reusing the in-memory candidate is distinct from durable publication ownership.

## Response outcomes remain distinct

2IS.BsS(delivery failure) logs and returns: no decrement, failed timestamp write, candidate clearing or timer scheduling appears in that complete body. Bvj(error) logs a literal500 diagnostic and calls privateA00. That diagnostic does not prove the actual received code is500: this method does not inspect its node. PrivateA00 records the current08q time as last_failed_auth_key_rotation_attempt, reads remaining attempts(default0), subtracts1, then uses clamping144.A0G.

CK4(success) enters the144.A0G object monitor and performs configuration-dependent Android-keystore/password-backed replacement, including144.A0B/A0C/A06 helpers. Complete helper semantics, old-key recovery and atomic durability are untraced; log text about lost old authkey is not an observed loss. On its local accepted replacement branch it updates144.A00, resets connection_lc, invokes145.A04, exits the monitor and sets remaining attempts to0 plus last_succeeded timestamp. Failed local replacement instead calls the same private failure recorder. Thus an IQ callback alone is not proof every key-storage branch succeeded. No candidate-clear assignment to19i.A00 appears in this callback; lifecycle outside this export remains unknown.

## Rust port boundary

Current connection.rs notification handling has no authkey_rotation consumer/ACK or w:auth:key rotation lifecycle. The initial receive-order inspection confirms capture precedes the selected ACK handlers, and groups_dirty is excluded from the second generic group ACK; no duplicate group ACK defect was found in that bounded path. Sender-key group rotation code found by text search is a different mechanism from client authentication-key replacement. This finding adds an explicit missing reliability obligation, not a claim of full receive audit completion.

Do not add a generic ACK and label rotation complete. A port needs authenticated target admission, durable candidate/publication ownership, uncertainty across disconnect/restart, exact response correlation and failure distinctions, safe local-key replacement and future-login coordination. Candidate cryptography, all helper state transitions, retry triggers, clocks and wire acceptance remain unfinished. No Rust runtime change or live operation.

Reproduce from repo root:
```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/19h;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/144;' --method A0G --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/19i;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/2IS;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/08q;' --method A00 --signatures --exceptions
```
