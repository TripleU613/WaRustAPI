# MobileConfig storage response receiver binding

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution, account read or live request.

## Owner and receiver are identified through construction

The [previous type boundary](../2.26.43.72-mobileconfig-request-data-type/README.md)
left the updater's receiver at +0x2d8 untyped. Its construction traced that pair
to enclosing owner+0x88. The selected construction region now supplies the
missing identity and pointer geometry:

1. A 1200-byte allocation at 0x775b0 contains a storage object at allocation+0x20.
   At 0x77614 it installs address point 0xd5580. Its typeinfo/name relocations
   identify FBMobileConfigStorageManager.
2. At 0x77810–0x77818, helper 0x79c28 creates a shared alias pair at stack+0xf0
   from that object pointer and its allocation/control object. 0x77824 reloads
   the object as x22; stack+0xf8 supplies its control pointer as x24.
3. A separate manager allocation installs address point 0xd4d90 at allocation
   +0x18 at 0x7795c. Its RTTI identifies FBMobileConfigManager. Alias x26 advances
   to allocation+0x58, then 0x779a8 stores the storage pair at x26+0x48. That is
   allocation+0xa0, equivalently **manager object+0x88**.
4. The prior updater construction copies owner+0x88 to updater+0x2d8. Therefore,
   when that owner follows this selected manager construction, the completion
   receiver is the storage-manager object, not RequestData or the manager itself.

Shared-count increments accompany the pair copies. They do not prove account
identity, race safety or that no later writer replaces the pair. The exact
selected factory routing and other possible constructor routes remain qualified;
this static chain is not observed runtime object identity.

## The virtual target is now resolved for this selected route

The updater call at 0x7e69c invokes receiver vtable slot +0x10. RELATIVE
relocation 0xd5590 resolves that slot of storage address point 0xd5580 to
**0x82c4c**. The typeinfo chain is 0xd5578 -> 0xd38d8 -> name pointer 0xd38e0 ->
0x2215e (FBMobileConfigStorageManager). The manager's independent chain is
0xd4d88 -> 0xd3420 -> 0xd3428 -> 0x20ab3 (FBMobileConfigManager).

This replaces the earlier unbound same-slot candidate 0x72f00 for this selected
receiver chain; it does not say that manager method is globally unused. The
RequestData constructor's own vtable identity remains distinct and unchanged.

The retained 0x82c4c entry saves the response pointer x1 and context pair x2,
retains scalar/output arguments, invokes logging/measurement-related virtual
calls, and reaches conditional processing through a receiver member at +0x98.
Its later response branches have not been followed here. A real target address
is not proof of successful JSON parsing, adoption, retry behavior, persisted
files or cache activation. These are the next concrete trace boundaries.

## Discovery and reproduction

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-storage-receiver/inspect-receiver.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-storage-receiver.json
cmp /tmp/mc-storage-receiver.json docs/protocol/apk/2.26.43.72-mobileconfig-storage-receiver/receiver.json
```

The inspector reuses pinned ELF/member validation, checks both RTTI chains, the
storage response slot and the shared-alias BL. LLVM windows retain the selected
constructor region and alias helper. Updater-copy and indirect-call windows are
reused from the preceding type bundle. The wider constructor region includes a
separate logger construction; it must not be confused with the storage object's
later installation at allocation+0x20. Bounded reads of 0x732d0's unrelated
allocation field +0x88 and JNI/caller neighborhoods remain discovery evidence,
not alternate resolved receiver paths. Equal offsets in unrelated objects do not
identify equal fields. Nearest JNI_OnLoad headings are not local function names.

rea-storage-entry.json retains complete REA Evidence
`ev_49ee749b1ed574309844a6a9f5b32ae5d132b6021706ef41a7abb2e1166a7f39`:
`e scr.color=0; pd 36 @ 0x82c4c`, exit 0, untruncated output, unchanged member hash,
with an entrypoint warning. This entry window does not cover the complete response
method; the LLVM entry extends through the first selected branch setup.
Register-relative annotations are not runtime memory observations.

**Remaining:** later receiver replacement/overrides, complete factory/caller
admission, processing at storage-manager+0x98, response parsing and field
validation, persistence ordering, retries, cache activation and account ownership.
No Rust runtime or saved account behavior changed.
