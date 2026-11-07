# Retry key envelope byte binding

**Confirmed selected static DEX evidence**, WhatsApp 2.26.43.72,
version code264307238, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Rust/source baseline `8043f147`. No target execution, account operation, key
mutation, retry request or runtime change.

## Questions and results

The [earlier direct-distribution trace](../2.26.43.72-retry-prekey-distribution/README.md)
bound getter results to retry keys/key, keys/skey and optional keys/pq, but left
exact KEM constructor/serialization semantics open. This bundle follows those
specific producer calls; it does not claim complete carried-key retry support.

1. **PQ public bytes: confirmed unchanged at this selected constructor chain.**
   Retained [0Yj.A00](../2.26.43.72-retry-prekey-distribution/kyber-public-envelope.txt)
   reads Dfd.publicKey_ and privateKey_, constructs KEMKeyPair([B,[B), then takes
   publicKey.A00 for ETs value. The exact [byte-array pair overload](kem-pair-constructor.txt)
   wraps the arrays with KEMPublicKey/KEMSecretKey and delegates to its object
   overload. [KEMPublicKey constructor](kem-public-constructor.txt) stores the
   supplied array directly in A00. This chain adds/removes no type byte and
   performs no public-key length/algorithm validation in the inspected methods.
   That does not establish valid runtime row contents or signing input.
2. **Signed classic bytes: confirmed parsed 32-byte value.**
   [0WJ.A05](signed-envelope.txt) obtains id_ via1nr.A04, calls EtN.A00, takes
   ERZ.A01/Ehd.A01, and retains Dfd.signature_ unchanged for ETs signature.
   [EtN.A00](signed-record-pair.txt) routes stored public bytes through
   [F3k.A02](signed-public-key-decode.txt): it requires at least33 bytes and
   first byte0x05, copies exactly32 bytes from offset1 into a new array, and
   constructs Ehd. [Ehd constructor](signed-public-key-constructor.txt) stores
   that array as A01. A longer stored array is not rejected by this decoder;
   trailing bytes are not copied. This is a stored-record parser fact, not
   permission to loosen Rust wire/public-key admission elsewhere.
3. **ID encoding: corroborated, already known in ordinary-prekey research.**
   [1nr.A04](id-writer.txt) emits low24 bits in three big-endian bytes, with no
   input range check in that helper. The retained retry getters call that same
   helper. Rust's existing 24-bit checks remain appropriate local admission;
   no wrap/truncation policy is added. This is not a newly discovered ID rule.

The [ETs constructor mapping](../2.26.43.72-retry-prekey-distribution/key-fields-constructor.txt)
remains id->A01, value->A00, signature->A02. The earlier job/writer trace carries
these fields to the corresponding retry key children; it is reused, not repeated.
The [ordinary getter](../2.26.43.72-retry-prekey-distribution/ordinary-direct-key.txt)
separately serializes its parsed classic key then removes the first byte.
Do not apply that classic strip operation to the PQ constructor chain above.

## Method and limitations

The REA Android package call failed before inspection because its configured
JAVA_HOME pointed at missing Java21. [Complete provider result](rea-provider-attempt.json)
is retained as a failure record, not successful REA evidence. The host's installed
Java resolves to Java27; no MCP configuration or package installation changed.
All method text here is produced by the repository's execution-free DEX inspector:

```
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/F3k;' --method A02 --signatures --exceptions
```

Use the class/method descriptors in each file to reproduce the other selectors.
The inspector retains an unknown opcode rendering0x5d in Ehd's constructor; the
public array store at000e..0012 is separately explicit. No conclusion depends on
that unresolved byte-field instruction. Printed instruction addresses use the
inspector's byte-coordinate rendering; exception comments explicitly use code units.
No synthetic output is presented as execution of these Android methods.

The complete generation/signing input, effective feature gates, durable ordinary/
PQ selection and direct-distribution ownership, upload exclusion, crash recovery,
retry count progression, receive-side admission and server acceptance remain
separate work. This closes selected byte plumbing only. It does not justify
copying arbitrary stored KEM bytes into a new request or enabling carried keys.
