# MobileConfig updater callback owner and dispatch boundary

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
This research neither loads native code nor sends requests.

## A specific updater callback is now identifiable

RTTI string 0x219e1 identifies the libc++ function wrapper for a lambda inside
FBMobileConfigDefaultUpdater::queueRequest, with signature void(bool, const
string&). RELATIVE relocations connect name pointer 0xd3810, typeinfo 0xd3808,
vtable typeinfo slot 0xd53f8 and callable address point 0xd5400. Its invocation
slot +0x30 (0xd5430) resolves to 0x80604. This is a concrete callback candidate,
not a string-only guess about arbitrary code.

0x7e450–0x7e500 obtains a virtual dispatcher, copies capture storage using
0x80490, allocates an 80-byte function object, installs address point 0xd5400,
and places the object in callable storage at frame offset -0x20. That is the
+0x20 target position of the function object starting at frame offset -0x40.
It then passes that callable address as x1 to virtual slot +0x10 of the obtained
dispatcher at 0x7e500. The helper 0x80490 copies three pointer pairs and remaining
scalar bytes; its matching cleanup includes shared/weak-reference cleanup.
The exact semantic names of each capture are not all established.

**Unresolved connection:** that intermediate dispatcher has not been followed
to the [Java fetch bridge's x4 argument](../2.26.43.72-mobileconfig-handler-construction/README.md).
Do not equate this candidate with the Java handler's stored callable yet. It is
an independently identified native updater callback and a useful upstream seed.
Other fetch routes can use other callbacks.

## Completion retains a weak owner, not an unconditional live updater

0x80604 saves the response string pointer from x2 and reads the bool byte through
x1. Helper 0xcaf88 preserves the function object as x19. It passes capture+0x08
to 0x76d38, which initializes an empty output pair, reads the weak control pointer,
and calls the imported std::__shared_weak_count::lock at 0xcd210. Only a successful
lock copies the captured object pointer to the output pair.

For a nonnull locked owner, the callback copies another captured pair from
function-object+0x28, forwards the bool in w3 and original string pointer in x5,
and invokes the owner's vtable slot +0x38 at 0x80694. Additional captured fields
are forwarded from +0x38, +0x3c, +0x40, +0x44 and +0x48. They are **not** treated
as proven account IDs, timings, retry counts or policy enums.

The independently identified FBMobileConfigDefaultUpdater vtable starts at
0xd5360; its +0x38 slot (0xd5398) is 0x7e5ac. Thus this is the selected destination
**if the locked object's vtable is that updater implementation**; overrides and
runtime object identity are not observed. The selected entry saves incoming bool
and string separately and has further virtual processing. It is not a validated
JSON/file-commit contract yet.

For a null locked owner, forwarding at 0x80694 is skipped. When captured +0x40 is
zero and +0x3c is 3, it goes directly to common cleanup. Otherwise it constructs
`updater deallocated` (0x1a80b), calls 0xb7990 using capture+0x18's field +0x78,
and calls 0x7f764 on that capture. Those helpers have side effects still to trace;
this branch must not be described as a universal no-op or proof of cancellation.
A bounded 0x7f764 window includes a byte clear and optional callback-related work,
but its complete effects are unresolved. All paths retained here release the
locked-owner temporary in common cleanup; complete exception coverage is unproven.

## Evidence and reproduction

inspect-callback.py reuses the preceding pinned-member inspector and checks
RELA/name connections, selected direct BL instructions and the BLR at 0x80694.

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-updater-callback/inspect-callback.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-updater-callback.json
cmp /tmp/mc-updater-callback.json docs/protocol/apk/2.26.43.72-mobileconfig-updater-callback/callback.json
```

Bounded LLVM disassembly windows retain exact addresses. Symbol headings based on
nearest exported JNI_OnLoad do not identify these local functions. Discovery also
inspected selected dispatch and construction neighborhoods; secondary-windows.txt
retains those bounded observations without claiming complete request processing.

rea-callback-window.json preserves complete REA Evidence
`ev_63349474833bce58dcae08976fb9bde105d02c8389c227729472859350f39cf0`:
`e scr.color=0; pd 62 @ 0x80604`, exit 0, unchanged member hash, no output truncation.
The provider warns it cannot determine the entrypoint. The 62-instruction window
ends before common cleanup and is not a complete function; the LLVM window
extends through the selected return/stack-check path. Rizin's address comments
that dereference register-relative operands are not runtime memory observations.

**Remaining:** resolve the intermediate dispatcher and actual callback captures,
then follow response parsing, persistence, retries and cache activation. Owner
lifetime guarding does not establish account-generation guarding. No Rust runtime
change, live operation, server-acceptance claim or zero-ban guarantee follows.

Follow-up: [requester dispatch](../2.26.43.72-mobileconfig-requester-dispatch/README.md)
now resolves the intermediate dispatcher for the selected default-updater route
and supplies the callback as fetch-interface x4. Actual fetcher installation and
response persistence remain open; the historical boundary above is narrowed by
that conditional handoff evidence.
