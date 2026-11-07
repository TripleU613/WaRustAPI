# RequestData identity and unresolved response receiver

**Confirmed selected static**, APK 2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
reconstructed arm64 libmobileconfig-jni.so SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No target execution or account operation.

The [previous constructor handoff](../2.26.43.72-mobileconfig-loader-handoff/README.md)
is now typed: 0xa9430 installs address point 0xd6400 at 0xa9490.
RELATIVE relocations connect its typeinfo slot 0xd63f8 to 0xd3e00,
whose name pointer 0xd3e08 resolves to 0x24b7c:
N12mobileconfig25FBMobileConfigRequestDataE. The following selected entries
resolve to destructors 0xa98a4 and 0xa987c; the retained destructor also accesses
the state at object+0x258. Thus the earlier generic term configuration object
refers here specifically to FBMobileConfigRequestData and its retained/derived
request state. Those field traces remain valid; they do not prove configuration
activation, response adoption or a manager implementation's identity.

The updater completion path at 0x7e69c loads a receiver from updater+0x2d8 and
invokes its vtable slot +0x10. Construction geometry narrows its source: the
shared updater object lives at allocation+0x18, while the construction alias
x27 advances to allocation+0xc0. The pointer stored at x27+0x230 therefore lands
at updater+0x2d8. It originates from the enclosing owner's +0x88 pair, retained
in a stack temporary. The enclosing owner's concrete type and that field's
producer still need binding; this is not proof it contains the RequestData object.
Do not connect two fields merely because both have offset +0x88 in different
unbound objects.

A candidate at 0x72f00 was inspected because it occupies slot +0x10 in a separate
manager vtable. Its selected body handles strings and optional worker dispatch.
That does not establish it as the updater's response target. The receiver's own
vtable must be resolved first; an equal slot offset in another class is not a
callgraph edge. rejected-manager-candidate.txt preserves the bounded read, not a
claim that this candidate is globally unreachable. Adjacent 0x7d000 and 0x7c9f8
neighborhoods were also inspected and remain explicitly bounded discovery reads.

This narrows the next action: bind the enclosing owner and producer of its +0x88
pair, then resolve the actual response-processing vtable. The currently retained
indirect call forwards the response string and other arguments, but JSON parsing,
persistence, cache activation and account ownership remain unresolved. No Rust
behavior is changed based on an unbound target.

Reproduce:

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-request-data-type/inspect-type.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-request-data-type.json
cmp /tmp/mc-request-data-type.json docs/protocol/apk/2.26.43.72-mobileconfig-request-data-type/type.json
```

The inspector reuses pinned ELF validation and checks RTTI relocations/name.
Constructor assembly is retained in the prior handoff bundle; new LLVM windows
retain destruction, updater construction geometry and bounded candidate reads.
Nearest JNI_OnLoad headings are not local function identities.
rea-constructor-type.json retains complete REA Evidence
`ev_bfa184d06703ed8727a2019b2768783af4486dd4e7c68c0bcf33c0a31609fe2f`: the actual evidence ID is recorded in that JSON envelope.
It is the bounded command `e scr.color=0; pd 28 @ 0xa9470`, with unchanged member
hash, exit 0 and no output truncation, plus an entrypoint warning. It does not
resolve the response receiver or observe runtime object identity.

Follow-up: [storage receiver construction](../2.26.43.72-mobileconfig-storage-receiver/README.md)
now binds the selected enclosing owner to FBMobileConfigManager and its +0x88
pair to FBMobileConfigStorageManager, resolving the selected response slot to
0x82c4c. RequestData remains distinct; other routes and later writes stay open.
