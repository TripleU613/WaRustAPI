# Known-null call ACK sender: recoverable generic exception — APK 2.26.43.72

**Confirmed selected static**, build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed 2026-11-06. Complete selected DEX methods/catches, xtask fallback,
not REA Evidence envelopes or live runtime. Complete1s2.run was re-exported and
compared exactly against the retained file. No account use.

## Exception selection matters

[1rH.A03](../2.26.43.72-distribution-ack-admission/native.txt) reads typed from
first, requires id and parses/diverts numeric459 before class routing. For
ordinary call ACK,04c2 branches to062a when from is null and throws
IllegalStateException at062a..0636, before children and call id/type checks.
[Optional typed accessor/parser](../2.26.43.72-ack-address-accessors/README.md)
establish absent/empty text from as known-null cases. Complete native parsing
of every other address spelling/cache representation is not assumed here.

The mapped handler call at106c..1072 in
[complete1s2.run](../2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
is within code_units060f..085c. IllegalStateException selects its Exception
handler085c (byte10b8), not the LIL-specific0873 handler (byte10e6). It logs,
tests2K0 (not this exception), then invokes1rg.A00(node) at10dc..10e2.
The next goto at10e2 with code-unit delta-084e returns to byte0046 to read on.
Thus the selected path does not enter the generic throwable cleanup/rethrow at
1128..1174, nor the LIL ACK corruption handler/reader error2 exit.

New complete1rg.A00 runs message-specific A01 only for root message, then
calls A03(node,500). Its own Exception catch logs and returns. The
[retained A03](../2.26.43.72-stanza-corruption-routing/nack-dispatch.txt)
first calls A02, then gates NACK construction on the known recoverable root-tag
set. Root ack is excluded and returns before Efa/Eoh NACK construction.
New A02 builds28V(error code, root tag, optional type) and submits through0NX.CSW.
That provider's complete telemetry/network effects remain untraced; Rust does
not claim to implement them or infer an account policy.

## Rust correction

The preceding call-field guard was too broad for simultaneous known-null from
and empty/missing call fields. Rust now bypasses only those later class checks
when from is absent or empty text. Required id presence and numeric admission
still run first; numeric459 still precedes class routing. Valid nonnull sender
call-field and confirmed sender-type failures retain their fatal LIL behavior.
No call-state feature, error response, challenge recovery or retry is added.

Nonempty unparseable sender spellings, nontext/cache coercion, child conversion,
other exceptions/providers and concrete recovery remain open. This finding
must not become a blanket nonfatal policy for all malformed call ACKs.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/1rg;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rg;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1s2;' --method run --signatures --exceptions > /tmp/call-null-reader.txt
cmp /tmp/call-null-reader.txt docs/protocol/apk/2.26.43.72-about-callback-lifetime/reply-dispatch.txt
```

checksums.json covers every other file here. Complete caller/accessor/set
exports are linked rather than duplicated. No zero-ban or server-acceptance proof.
