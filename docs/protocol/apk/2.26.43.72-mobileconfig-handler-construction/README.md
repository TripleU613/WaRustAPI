# MobileConfig handler construction and callable lifetime

**Confirmed selected static evidence**, APK 2.26.43.72 (code 264307238),
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The reconstructed arm64 libmobileconfig-jni.so is 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No library, APK or account operation runs.

## Construction connects fetch to the completion holder

ELF RELATIVE relocations connect vtable slot 0xd4990 to 0x58788 and its
RTTI to MobileConfigFetcherClient. Handler-holder vtable 0xd4570 has destructor
entries 0x59884/0x59898; its RTTI names MobileConfigFetcherHandlerHolder.
construction.json preserves exact slots and names, checked against the pinned
member by inspect-construction.py using the preceding native inspector.

In the selected fetch body, x4 is saved as x24 at 0x587ac. At 0x58914–0x5891c,
that argument is passed through helper 0x60148 into temporary callable storage.
A 64-byte allocation follows. The same helper copies temporary callable storage
again at 0x58944 and into the allocated holder at offset 0x10 at 0x5895c. The
holder vtable is installed at 0x58954. The helper reads its source target at
+0x20: null writes null to the destination; an external target uses virtual
slot +0x10 to clone; an inline target stores the destination address and branches
to virtual slot +0x18. This is a **copy-shaped callable handoff**, not proof of
its concrete target type or captured account identity.

The resulting target-pointer position is holder+0x10+0x20 = holder+0x30,
matching the [completion wrapper's load](../2.26.43.72-mobileconfig-native-completion/README.md).
The selected destructor moves to holder+0x10 and delegates cleanup to 0xa975c.
These observations connect creation/storage to completion access; they do not
establish once-only invocation, thread safety or the final lifetime of Java refs.
In particular, the previously retained completion path does not clear the stored
pointer before invoking it. Do not infer that completion consumes the callable.

## Java bridge binding

0x58cdc–0x58cf0 uses method name `fetch` at 0x18fff and the signature at 0x1e1a8:

```
(Ljava/lang/String;Ljava/lang/String;Ljava/util/Map;Lcom/facebook/mobileconfig/MobileConfigFetcherHandler;Ljava/lang/String;Ljava/lang/Integer;)V
```

The method ID is cached at 0xdc020 and loaded for the call at 0x58ba0.
0x58b80–0x58bc8 prepares the string/handler/optional-object arguments, calls
0x4cc84 and checks pending JNI exceptions. The selected constructor body contains
two hybrid-object setup branches; both precede this fetch call. Their complete
exception paths and the runtime mode determining construction are untraced.
The signature agrees with the earlier [Java fetch route](../2.26.43.72-mobileconfig-fetch-route/README.md);
this does not establish a live fetch or server acceptance.

## Reproduce and remaining boundary

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-handler-construction/inspect-construction.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-construction.json
cmp /tmp/mc-construction.json docs/protocol/apk/2.26.43.72-mobileconfig-handler-construction/construction.json
```

The bounded LLVM windows use `llvm-objdump -d --no-show-raw-insn` and the start/stop
addresses in their files. Nearest exported-symbol headings are not function
identities. RELA addends, not zero raw file pointers, identify the selected
vtable/RTTI references. This extends the preceding REA observations with explicit
ELF and LLVM inspection, not provider pseudocode or runtime capture.

**Unknown:** upstream creation of the callable passed as x4, concrete captures,
JSON validation, response adoption, file commit, retry policy and cache activation.
The nearby 0x594c0 candidate inspected during discovery is vector destruction,
not evidence of callable construction. No Rust sender or hydration behavior is
changed. The next useful trace is the upstream caller supplying x4, retaining
these ownership and persistence gaps instead of guessing a commit contract.

Follow-up: [requester dispatch](../2.26.43.72-mobileconfig-requester-dispatch/README.md)
now resolves the intermediate dispatcher for the selected default-updater route
and supplies the callback as fetch-interface x4. Actual fetcher installation and
response persistence remain open; the historical boundary above is narrowed by
that conditional handoff evidence.
