# Upload Java-to-native submission and header serialization — APK2.26.43.72

**Confirmed selected static**, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed ARM64 libwhatsappmerged.so SHA256
`d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d`.
Checked DEX and host LLVM23.1.3; no target/library/account execution. No new REA
native decompilation. Follows the [expected-size header binding](../2.26.43.72-media-expected-length-source/README.md).

## Selected Java request path

JAT.A04's existing selected send calls0eA.A01 with its TigonRequest and IxI body
provider. Complete0eA.A01 dispatches through Kag(selector2) while retaining those
objects. Selected Kag.invokeSuspend branch0240 onward wraps callbacks inIxE,
reads request properties, adds diagnostic inflight markers, and calls
TigonHttpClientBase.sendRequest(request,provider,callbacks,null executor).
Other Kag selectors are retained for completeness, not attributed to uploads.

The selected ordinary callback overload delegates to private sendRequestInternal.
That method synchronizes proxy-setup admission, optionally calls
setSystemProxyHostAndPort, then passes the same request and provider to its
TigonXplatService holder. Proxy helper behavior is outside this trace; no claim
is made that native stack routing matches Rust's no_proxy setting.

TigonXplatService.sendRequest(request,TigonBodyProvider,callbacks,executor)
serializes the request through1Fb.A01, serializes its property container through
1Fa.A03, then passes both byte buffers/their used lengths, original request and
body provider to native sendRequestBodyBufferIntegerBuffer. The overload taking
ByteBuffer[] is separate and is NOT this selected media-provider call.

Complete1Fb.A01 reads method, URL and existing request.headers; it passes headers
to1Fa.A05. The complete map writer emits0 for null, otherwise the current map
size and every key/value string throughA04. Neither selected method takes the
body provider or adds a Content-Length literal/entry. Thus no provider-derived
length synthesis occurs in THESE request/map serializers. Property serialization
and downstream native middleware remain separate; this is not a whole-stack
negative claim or a proof headers cannot change elsewhere.

## Direct length-getter census

Exact physical callers ofTigonBodyProvider.getContentLength find onlyIxI.A00,
the previously traced stream writer reporting length toTigonBodyStream. This
census does not establish all reflection/JNI calls, overrides, dynamic dispatch,
concurrent mutations or middleware. ELF defined text-string search has no literal
getContentLength match; encoded/cached/dynamic lookup is not excluded. Absence
of that literal is not proof native code never learns a length.

## Internal -1 candidate is not ordinary-upload proof

A native content-length write at4e6290..4e62b8 creates an MCF string using
literal -1 (f7c89, length2) and stores it under the same key788e58 verified in
the prior queue trace. Helper310914 sets dictionary argument and value; exact
bytes/literals are verified byinspect-submit-boundary.py. The immediately
preceding block builds sec-websocket-protocol metadata. This is a bounded
internal dictionary write; receiver/operation binding and later serialization
are not resolved. It does NOT establish an ordinary media-upload automatic
header, nor that literal -1 is sent as an HTTP Content-Length. The later native
copy/sentinel handling remains distinct. Do not add that value to Rust.

**Unknown:** selected properties serializer's full effects, native request
conversion and middleware, effective transport/configuration, actual header
synthesis when JAT omits explicit Content-Length and final wire framing. Rust's
known byte body/actual length remains unchanged. No missing mandatory header
or construction mismatch established; no new sender, retry or live validation.
This closes the selected Java header-serialization boundary while leaving native
middleware scope explicit.

## Reproduction

From repository root, with existing recovered pinned library:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
LIB=../android-lab/output/metrics-native/reconstructed-reproduction/libwhatsappmerged.so
target/debug/xtask apk-callers "$APK" --class 'Lcom/facebook/tigon/TigonBodyProvider;' --method getContentLength > java-callers.txt
target/debug/xtask apk-smali "$APK" --class 'LX/0eA;' --method A01 --signatures --exceptions > submit.txt
target/debug/xtask apk-smali "$APK" --class 'LX/Kag;' --method invokeSuspend --signatures --exceptions > submit-coroutine.txt
target/debug/xtask apk-smali "$APK" --class 'Lcom/crossapp/tigonhttp/TigonHttpClientBase;' --method sendRequest --signatures --exceptions > native-submit.txt
target/debug/xtask apk-smali "$APK" --class 'Lcom/facebook/tigon/TigonXplatService;' --method sendRequest --signatures --exceptions > xplat-submit.txt
target/debug/xtask apk-smali "$APK" --class 'LX/1Fb;' --method A01 --signatures --exceptions > request-serializer.txt
target/debug/xtask apk-smali "$APK" --class 'LX/1Fa;' --method A05 --signatures --exceptions > header-serializer.txt
strings -t x "$LIB" | rg 'getContentLength|Content-Length|content-length' > native-strings.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x4e6230 --stop-address=0x4e62d4 "$LIB" > synthesis-candidate.txt
llvm-objdump -d --no-show-raw-insn --start-address=0x310914 --stop-address=0x310920 "$LIB" > dictionary-helper.txt
python3 docs/protocol/apk/2.26.43.72-media-java-native-submit/inspect-submit-boundary.py "$LIB" > windows.json
```

Python stdlib and prior digest-pinned ELF resolver. Nearest LLVM symbol labels
are not function attribution. windows.json pins selected bytes and literals;
checksums.json covers all other files with bytes/SHA256. Full binaries stay local.
