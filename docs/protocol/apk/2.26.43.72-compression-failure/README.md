# Compression failure leaves the reader — APK2.26.43.72

**Confirmed selected static**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports; fallback follows the previously diagnosed
unavailable Android provider. These are not REA Evidence or runtime observations.
No target execution or live account operation.

## Inflation failures precede stanza dispatch

[Complete123.A08](../2.26.43.72-stream-end/frame-reader.txt) wraps the
compressed remainder in java.util.zip.InflaterInputStream. With A04 true,
008e..0094 constructs the inflater, A01 parses through BufferedInputStream,
and00b2..00c0 drains to EOF before returning the parsed node. With A04 false,
00fa..0104 constructs the inflater and011c..0138 reads all output before A01.
Its exception table has cleanup paths; it does not catch IOException and
continue to the next encrypted record. A checksum/inflation exception cannot
return the already parsed node from the streaming branch. Exact Android inflater
implementation, buffer-boundary timing and malformed-byte runtime matrix were
not executed here; invalid/truncated/checksum zlib fixtures are local Rust tests.

[Complete1s2.run](../2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
calls A08 at0058..005e. The002a..0034 code-unit range catches IOException at088a
(byte1114). That block logs, iterates its reader-local1ru map calling A00,
clears the map, and rethrows at1172. Its surrounding085c..08ba range routes
IOException to0912 (byte1224). This logs ReaderThread/io-error, selects default
error1 (v17 initialized at003c), or error3 for a SocketException containing
connection reset, calls1lX.onError at12be..12c4 and returns. This is not the
separate LIL corruption/error2 path or recoverable parsed-stanza handler branch.
No following stanza is read by this reader after that return.

The concrete1lY listener binding was established by the
[prior initialization trace](../2.26.43.72-stream-end/README.md).
[onError](reader-error-submit.txt) posts what2 with the supplied error in arg1.
The already retained [handleMessage](../2.26.43.72-stream-end/observer-handler.txt)
case2 checks the finished flag and A1W: finished invokes A1M; selected A1W true
invokes A1G(-1); otherwise A1J receives arg1. [A1J](error-manager-proxy.txt)
only delegates A0m. A0m internals and global reconnect/backoff/replay authority
are not established by this extension. Reader-local map cleanup must not be
confused with durable outbox deletion or general callback-map clearing.

## Rust consequence and verification

[Canonical behavior and regression scope](../../frame-admission.md#compression-failure-consumes-the-receive-epoch-2026-11-06).
The shared compression class now consumes the epoch after capture. The same
class covers the existing local decompression cap; terminating on that cap is
an explicit local refusal policy, not a traced native size limit.

## Reproduce

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A08 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1s2;' --method run --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1lY;' --method onError --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0ZJ;' --method A1J --signatures --exceptions
```

PCs are bytes; catch ranges/targets are code units, converted by multiplying2.
Linked methods are already retained completely and are not duplicated here.
checksums.json covers every other file in this bundle.
