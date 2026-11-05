# Native metrics endpoint list construction — APK 2.26.43.72

**Confirmed static / derived; observed bounded synthetic append/store.** Pinned
APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
No provider, Java runtime, actual URL parser or network operation was executed.
Local Rizin/CLI exports and Unicorn results are not REA Evidence envelopes.
No production Rust change or mandatory metrics requirement follows.

## Task selection and fresh list ownership

Scheduler **0x727d40** checks manager byte+0xc0: value1 logs already sending
and returns. A selected earlier branch, enabled only when byte+0xb0 equals1,
compares MCFAbsoluteTimeGetCurrent minus stored double+0xb8 to literal15.0;
strictly greater clears+0xc0 and updates+0xb8. This is a selected busy-flag
transition, not a universal retry duration or proof obsolete completions cannot
interfere. Its broader configuration/clock contract remains unknown.

For nonempty task dictionary+0x60, the scheduler snapshots keys and retains at
most one selected key in a local array. Dictionary snapshot order is not a
proved FIFO or priority policy. It removes that key's old endpoint entry from
manager dictionary **+0x68**, then creates an empty mutable URL array. If
provider+0x48 exists, it requests the provider list; absent provider leaves no
custom list. Empty task dictionary follows a separate callback path, not an
endpoint send.

## Exact Java provider binding and conditions

Native object bridge **0x6ecde0** loads class cache0xc44bd0 and method cache
**0xc468c0**. The [JNI_OnLoad tables](../2.26.43.72-metrics-jni-method-cache/README.md)
resolve index26 to **JniBridge.jnidispatchO(I)Ljava/lang/Object;**. Scheduler
supplies selector **0**, then converts the returned Java object to MCF data
through0x6ea11c→0x6e9f88. Full generic object-deserializer validation is outside
this trace. The exact Java method's other selector1 is unrelated.

Selector0 obtains 2CQ through0WS.A06 and creates an ArrayList. In order:

1. It gets a JzG record through0ZW.A03. Only if0ZX.A04 is true, config flag
   **0xfea** is true and the record exists, it adds prefix + record.A03 + `:` +
   record.A01 integer + suffix.
2. Only if2CQ.A01 config flag **0xfb4** is true, it prepares0VM and reads its
   A0P record. A missing record contributes nothing. It iterates record.A0C
   in list order. For each1t1, a **nonnull empty A0B set skips** that entry;
   null or nonempty sets proceed. It appends prefix + A05 + suffix if A05 is
   nonnull, then prefix + A01 + suffix if A01 is nonnull. Missing A05 does
   not prevent appending a nonnull A01.

Here prefix0cJ.A0T decodes to **https://**, suffix0cJ.A0X to
**/deidentified_telemetry**. The retained <clinit> excerpt binds both assignments;
0cJ.A00 XORs each Java char with0x12. The resolver decodes only these two
selected nonsecret literals. No whole-class constant dump or runtime hostname
was retained. Empty/custom config values, flag semantics and upstream record
producers remain unknown. The method catches Exception, reports it via0Wg,
and returns null. Dynamic host/port contents were not observed or invented.

## Native conversion, fallback and dictionary store

At0x727ef8..0x728024 the selected custom list is processed in index order.
Each element goes to MCFURLCreateWithString; a null result logs invalid endpoint
and is skipped. Nonnull URL objects are appended to the new array. This proves
handling of the parser's result, not its accepted schemes, host restrictions,
normalization or TLS policy. Do not replace URL validation with a string test
based on this trace. No explicit deduplication occurs in the selected loop.

After custom processing (including null/empty list), it always calls native
string selector **0x16/22**, converts that ASCII string with
MCFURLCreateWithCString and appends its URL. A null conversion result aborts;
its real plaintext URL/selector decoding remains **unknown** here. The default
is last, not prepended in front of valid custom entries. No fabricated fallback
URL or credential was supplied to production code.

It then stores the complete array at dictionary+0x68 under the selected task
key and tail-calls sender **0x7280dc**. For this single freshly built list, the
count is successful custom conversions plus one fallback. A nonempty array is
not proof any endpoint is reachable or that transmission succeeded.

## Consumption and response fallback use that stored list

The [sender](../2.26.43.72-metrics-native-request-builder/README.md) gets this
array, retains/removes index0 **before** request construction. The selected
[response worker](../2.26.43.72-metrics-native-transport-responses/README.md)
checks whether more endpoints remain and, when its error/flag branch permits,
calls that sender directly. This branch does not call the scheduler to rebuild
custom URLs or append the default again. Do not turn one finite-list traversal
into an unbounded fresh-list retry or general HTTP401 retry rule.

A later independent scheduler invocation can rebuild the selected task's list;
complete scheduler triggers, timing and generation fencing remain unproven.
Busy-flag clearing is not a durable delivery result. Nothing here authorizes
replaying an account/login/message request or ignoring server waits.

## Fixtures, reproduction and remaining work

**Seven bounded native cases** execute append/store instructions starting at
0x727ef8, stopping at0x728024 or controlled native abort. Cases cover absent
list, empty list, one/all/mixed/several successful conversions, and failed
fallback conversion. MCF arrays, URL-conversion results, logging, fixed-string
selector, dictionary and release are explicit synthetic adapters. They verify
preserved order, skipped failures, last fallback and successful dictionary
binding; they do not execute the provider, actual URL parsing, whole scheduler,
sender, response worker or network. Native abort is an expected fixture result
only for the invalid-fallback case.

Run `python inspect-endpoints.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
endpoint-boundaries.json (pinned windows, JNI table binding, selected decoded
Java constants and labels). With Unicorn2.1.4 run
`python check-endpoints.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
endpoint-results.json. Each case has100000-instruction/one-second caps;
unexpected imports/interrupts or unsupported loader relocations fail closed.
Rizin windows.json uses `pD SIZE @ VA`. Java exports use xtask apk-smali with
exact class/method/signatures/exceptions; jnidispatchO is selected by exact full
prototype from the prefix-matched export. Checksums cover all other files.

**Inferred:** preserving list ownership/order prevents changing selected native
fallback behavior. **Observed live:** none. **Unknown:** actual dynamic endpoints,
selector22 plaintext, upstream provider policy, full retry scheduling and
stale-completion ownership, credential crypto and deployment necessity. These
findings narrow endpoint production; they do not establish full transport parity,
server restriction policy or a ban-prevention guarantee.
