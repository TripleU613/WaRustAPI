# Native metrics request framing — APK 2.26.43.72

**Confirmed static / derived**, recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`;
pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Recovery provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
No request or native request-builder execution was observed. Local Rizin exports
are not REA Evidence envelopes. This is metrics-specific construction evidence,
not proof of mandatory telemetry or protection from account restrictions.

## Method and header sources

The [complete sender 0x7280dc..0x728b24](../2.26.43.72-metrics-native-transport-responses/buffer-sender-assembly.txt)
at 0x7289a4..0x7289c0 loads GOT slot 0xc27d98, dereferences the imported
`kMCIURLRequestHTTPMethodPOST`, and passes endpoint, dictionary and body to
MCIURLRequestCreateWithHTTPBodyNoCopy. The resolver verifies the exact GLOB_DAT
relocation and imported symbol; this does not observe its runtime object value.

The dictionary count is five. Keys and corresponding values are:

| Header | Source |
| --- | --- |
| Content-Type | `multipart/form-data; boundary=pfBvf7MwXQtqX5egJfP0Lg==` |
| Accept-Charset | `utf-8` |
| Accept | `text/plain` |
| Host | 0x730a54 selector 0x1a, converted from ASCII to MCFString |
| X-FB-Request-Analytics-Tags | helper 0x72ca94 result; null-result fallback `{}` |

Host provenance follows x24 assigned at 0x728704 and consumed at 0x728950.
An earlier x24 loaded from credential +0x28 is a different value, used before
that overwrite in optional metadata construction. It must not be mistaken for
Host. Selector entry indexes four tables; its full decoding and resulting
strings are **unknown** here. No endpoint hostname or token value is invented.
Analytics helper inputs and complete schema remain **unknown**.

## Multipart layout

Confirmed append order in the selected sender, with exact literal lengths and
escaped CRLF retained in request-literals.json:

1. Opening boundary `--pfBvf7MwXQtqX5egJfP0Lg==\r\n`.
2. Plain-text `access_token` part: value from 0x730a54 selector 0x17.
3. Plain-text `credential` part: value saved at sp+0x28 at 0x728370 and
   supplied at 0x728504. Earlier formatting uses `%s+%s`; complete cryptographic
   derivation and issuance are not established by this bundle.
4. Binary `message` part, filename `WAMEventBuffer.dat`, content type
   `application/octet-stream`, transfer encoding `binary`; bytes/length come
   from the selected MCFData object saved at sp+0x38.
5. JSON `meta_data` part, followed by CRLF and closing boundary
   `--pfBvf7MwXQtqX5egJfP0Lg==--\r\n\r\n`.

Plain part helper 0x72b004 appends the 64-byte text/plain/disposition prefix,
name, five-byte quote/CRLF terminator, value, CRLF and next boundary. Its
argument helper explicitly preserves x1 as name and x2 as value. Both plain
parts use this helper. Message headers are 156 bytes; metadata headers 84 bytes.
Metadata format literals are `{"t": %ld,"p": %u,"rt": %s}` and the variant
adding `,"config_id": "%s"` before the closing brace. Full semantic sources,
units, escaping and supported value domains remain **unknown**; literal member
names alone do not establish those meanings.

## Endpoint and credential ordering

The sender retrieves the task's endpoint array from manager dictionary +0x68,
requires positive count, retains index zero, and removes that index before
request construction (0x728324..0x728358). The retained endpoint then supplies
the request URL. This consumption order matters when comparing failure/retry
behavior; it does not authorize retries for other request families.

Selected current/spare credential slots are +0x78/+0x80; use counter +0x38
and clock comparison against +0x30 gate construction. A consumed credential's
counter is decremented before request building. Full provider, refresh, clock
units and durable ownership remain **unknown**. These static gates do not
justify fabricated credentials or translating native metrics rules into login
or messaging policy. No Rust behavior changed.

## Reproduce and scope

Run `python inspect-request.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must match
request-literals.json. It reuses the adjacent digest-pinned ELF section resolver,
resolves literal descriptors and method relocation, and preserves exact bytes.
Helper assembly exports use `rizin -2 -q -e scr.color=0 -c 'pD SIZE @ VA; q'`;
start/end boundaries appear in their addresses. The string-selector export is
only an entry window, not its complete body. Existing full sender supplies call
sites. Checksums cover all other files in this bundle.

**Inferred:** explicit framing and admission order are useful Rust compatibility
inputs. **Observed live:** none. **Unknown:** complete request validity, selector
decoding, credential cryptography/provider, metadata/analytics schema, transport
ownership and every server requirement. No reverse-engineering percentage or
ban-prevention guarantee follows from this chunk.
