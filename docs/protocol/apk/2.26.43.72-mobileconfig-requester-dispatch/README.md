# MobileConfig requester dispatch and local empty-query refusal

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution, account access or network request occurs.

## The intermediate dispatcher is a requester

The preceding [updater callback](../2.26.43.72-mobileconfig-updater-callback/README.md)
passes its callable to a dispatcher obtained through the updater's virtual slot
+0x48. The default updater's slot 0xd53a8 resolves to 0x7f648. That getter returns
its cached pointer at updater+0x320 when present. Otherwise it allocates 88 bytes,
installs vtable address point 0xd6460 and caches the result at +0x320. RELATIVE
relocations tie the vtable's typeinfo to the name FBMobileConfigRequester and
its +0x10 slot 0xd6470 to 0xabf18. Thus the earlier virtual call at 0x7e500 resolves
to this requester for that selected default implementation.

Construction copies updater+0x310/+0x318 into requester+0x40/+0x48. In 0xabf18,
x1 is preserved as the caller's callable, x2 as an output timestamp pointer, and
x3 as request data. The function builds a query vector via 0xac09c, then uses the
fetcher stored at requester+0x40. Immediately before its first virtual method is
called at 0xabfec, it copies the caller's callable with the previously traced
0x60148 helper into stack storage passed as x4.

| Fetch interface argument | Selected preparation |
| --- | --- |
| x0 | requester+0x40 fetcher pointer |
| x1 | static short string `POST` at 0x24c70 |
| x2 | copied endpoint short string on stack |
| x3 | constructed query vector on stack |
| x4 | copied caller-supplied callable on stack |
| x5 | request-data string at +0x80 |
| x6 | request-data word at +0xf0 masked to 40 bits |

The [Java bridge](../2.26.43.72-mobileconfig-handler-construction/README.md)
is MobileConfigFetcherClient's first virtual method, 0x58788. Therefore, **when
that implementation is installed**, this route supplies the identified updater
callable in exactly its x4 position, which the bridge copies into the handler
holder. The requester retains the fetcher implementation provided during
construction; this trace does not prove every runtime installs the Java bridge.
It also does not prove all fetch variants use this queueRequest callback.

## A local refusal bypasses fetch-interface invocation

After constructing the query vector, 0xabf64 reads a word at request-data+0x9c.
If it equals 1, execution proceeds to the fetch path even when the vector is
empty. Otherwise the vector begin/end pointers are compared; if equal, execution
branches to 0xac02c. That branch records system_clock::now through the timestamp
pointer and invokes the caller's stored callable via 0x5971c with boolean false
and `Empty query params` (0x16f64). It then joins cleanup. The fetch call at
0xabfec is bypassed on that selected branch.

Do not equate an empty native query vector with the previously derived JSON
`queries: []` field: they belong to different stages and their relationship has
not been traced. The numeric meaning of request-data+0x9c is also unresolved.
This is a local failure, not a server error, timeout or ban signal. The callback
can run synchronously before this requester method returns; do not assume every
completion necessarily follows a network round trip.

On the selected fetch branch, the timestamp is also written before interface
invocation. The endpoint source depends on requester+0x08: a nonzero byte selects
a static libc++ short string encoding `mobileconfig`; zero selects the global
string object at 0xdb9d0, whose initialization/content is not resolved here.
The static `POST` string is interface input, not proof of a particular HTTP
request: the earlier Java route can map SDK logical requests to MEX/WWW pipelines.
The 40-bit mask is confirmed assembly; its enum/optional semantics are untraced.

## Reproduction and limitations

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-requester-dispatch/inspect-dispatch.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-requester.json
cmp /tmp/mc-requester.json docs/protocol/apk/2.26.43.72-mobileconfig-requester-dispatch/dispatch.json
```

The inspector reuses the pinned member's ELF/hash validation, checks RELATIVE
vtable/typeinfo bindings, selected BL/BLR instructions, refusal text and the two
static short-string representations. Bounded LLVM windows retain construction,
normal/refusal/cleanup instructions and the untraced query producer's entry.
Nearest-exported-symbol headings do not name the local functions.

rea-requester-window.json retains complete REA Evidence
`ev_c393afd78a03a5056879c825cbd3a2ed08235304fbba275d0bd6b959fb86ee5c`,
`e scr.color=0; pd 80 @ 0xabf18`: exit 0, no output truncation, unchanged artifact
hash, with an entrypoint warning. The 80-instruction window is explicitly bounded;
LLVM retains through the next function boundary. Register-relative Rizin address
annotations are not observed runtime memory.

Discovery excluded the manager-vtable +0x48 getter at 0x73588 (a string copy):
the earlier receiver is an updater, so the updater vtable must be used. It also
inspected Tigon and fetcher/base destruction neighborhoods; a +0x10 destructor
there is not the requester dispatch. excluded-candidates.txt preserves those
bounded reads and the uncompleted factory-call neighborhood at 0x7b800. A raw
reference to typeinfo can be inheritance metadata rather than a vtable; the
bindings here also agree with installed construction address points.

**Remaining:** full query-vector construction, request-data field producers,
fetcher injection/replacement, global endpoint initialization and completion's
response parsing/file commit/cache activation. No Rust sender or hydration change
is justified by interface geometry alone. This closes the selected conditional
callback handoff, not the complete network/configuration lifecycle.
