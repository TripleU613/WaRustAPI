# MobileConfig record source loader and shared handoff

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution, account access or live request.

## Deferred record refresh loads from a configured path

The preceding [grouping trace](../2.26.43.72-mobileconfig-schema-grouping/README.md)
gets its existing record vector through 0xa99c0. On the deferred in-memory route,
0xa9a8c claims the refresh byte with the previously traced CAS pattern and checks
whether configuration+0x258 already contains a source pointer. A nonnull pointer
skips loading. Otherwise it tests configuration byte +0x209 for equality to 1.

When that byte is not 1, it constructs a local loader object, invokes 0xbb28c
with the string at configuration+0x130, obtains a shared record pair via 0xbfc6c,
and stores that pair at the beginning of configuration+0x258 using 0x5b7d8.
It also gets another shared pair through 0x5c900 and stores it at configuration
+0x248; that pair's semantic role remains untraced. Loader cleanup follows.

When byte +0x209 equals 1, it takes a process-global mutex at 0xdba10 and searches
a string-keyed tree rooted near 0xdb578/0xdb580. On a matching entry it copies the
shared loader pair; on a miss it constructs a shared loader, calls 0xbb28c with
configuration+0x130, optionally loads a second nonempty local path, and inserts
that loader into the tree. The selected entry initializes that second path as
empty. Cache-key string construction and all its producer/lifetime implications
are not fully modeled; do not assume this cache is account-bound or keyed by
file contents. After unlocking, it publishes the same two shared-pair destinations
as the local-loader branch. The refresh completion byte is set before release.
This is in-memory publication evidence, not proof of configuration file commit.

## Filename classification and file-open boundary

0xbb28c derives a final path component using separator literal `/\\` and compares
it to these native constants:

- params_map_v4_u1.txt
- params_map_v4_u2.txt
- params_map_kMobileConfigSessionless.txt
- params_map_kMobileConfigSessionbased.txt

The first three matches set a local classification flag to 1; the fourth uses
its equality result. Other names yield flag 0 and still join later handling.
This is **not a whitelist rejection** or evidence that u4/u6 assets are ignored.
The full downstream use of that classification and loader flags remains untraced.

A selected subsequent path calls 0xbc0ac with the path string. That function
constructs a libc++ basic_filebuf and calls 0x990c0 with open-mode word 8. The
adapter selects the string's short/long pointer and tail-branches to the imported
basic_filebuf::open(const char*, unsigned) at 0xcd3e0. A null result reaches an
io-state update using word 4. Thus a file-stream open is statically reachable;
this is not proof a file exists, an open succeeded or any particular bytes parsed.

On selected later branches, bb28c calls path transformer 0xc5af8 and content
handling 0xc541c, plus other loaders/decoders. Their bounded entry reads are
retained as unfinished boundaries. Do not call the path transformer itself a
file-read operation or treat a filename match as a parser validation result.

## The record accessor retains shared ownership

0xbfc6c reads the loader's child pointer at +0x28. If child byte +0x98 equals 1,
it copies the pair at child+0x50; otherwise it copies loader+0x150. For a nonnull
control pointer it increments the shared reference count via 0x4c570. Both local
and cached loading paths publish that returned pair into configuration+0x258.
This narrows the earlier source-retention gap: configuration retains shared
record ownership on these selected paths, while its grouped table contains views
into those records. Replacement synchronization, account-generation ownership
and crash/restart recovery remain unproven; this is not a proof of race safety.

## Packaged assets are identified separately

The pinned APK ZIP contains assets/params_map_v4_u1.txt, u2.txt, u4.txt and u6.txt
(with the same full prefix). inspect-source.py verifies the full APK hash before
listing matching members; source.json retains exact names. The u1/u2 basenames
match native classification constants, but configuration+0x130's producer has not
been followed to those APK entries. Therefore asset presence does not establish
the selected runtime path, extraction location or decoder branch. Earlier
[token-map evidence](../2.26.43.72-token-param-header/README.md) remains a separate
format observation; do not apply its binary layout to this loader without binding.
No raw APK member contents are committed here.

## Reproduce and remaining work

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-record-source/inspect-source.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so ../android-lab/apks/WhatsApp.1.apk > /tmp/mc-record-source.json
cmp /tmp/mc-record-source.json docs/protocol/apk/2.26.43.72-mobileconfig-record-source/source.json
```

The script checks both pinned identities, native filename literals, selected BL
targets and ZIP member names. LLVM windows retain source getter/refresh logic,
classification and file-open/record-accessor evidence, plus explicitly bounded
discovery entries for unfinished helpers. Nearest JNI_OnLoad headings are not
local function identities.

rea-loader-window.json retains complete REA Evidence
`ev_a5b0ee27a21e275a9db5ffd620c0b42d51b2d94223113b2c1829f7bf4fc1dcd5`:
`e scr.color=0; pd 66 @ 0xbb28c`, exit 0, untruncated output, unchanged member
hash, with an entrypoint warning. This bounded window does not cover the whole
loader; the wider LLVM entry window likewise does not resolve all decoder branches.
Register-relative annotations are not observed runtime memory.

**Remaining:** configuration path producer, chosen file/content identity, decoder
branches and record population, cache-key semantics, replacement synchronization,
request-data construction and response parsing/file commit/cache activation.
No Rust runtime or account behavior changes are justified by this handoff alone.
