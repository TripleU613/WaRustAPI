# Translation readiness and its independent publication suffix

**Confirmed selected static evidence**, same pinned APK/native member as README.
No native execution or account/network operation. This follows 0x8bfe0,
`buildLatestConfigWithTranslationTable`, called by the reload and upgrade gates.
Nested parser and translation construction semantics remain incomplete.

RequestData availability false logs and returns false. Otherwise the helper
constructs a reader through 0x88144 with mode argument 9. Its root probe chooses
a table-derived hash or empty string for source helper 0x8b8bc. Source optional
flag at stack+0x148 false enters a diagnostic path and returns false without
the publication suffix. True selects translator construction by manager word
+0x26c: exactly 3 uses the larger object; all other values use the vector form.

The larger allocation is 184 bytes, shared object pointer allocation+0x18,
vptr **0xd62e0**. Its selected virtual+0x20 relocation **0xd6300 -> 0xa84e4**
returns byte **object+0x98** (allocation+0xb0). The vector allocation is 56
bytes, shared object pointer allocation+0x18, vptr **0xd6320**; its selected
virtual+0x20 relocation **0xd6340 -> 0xa8724** returns whether vector begin/end
at **object+8/+0x10** differ. These are the concrete readiness predicates.

The mode-3 path includes two strings projected through 0xa8fa4 (prefix before
first colon) and compared via 0x531f8; equality sets allocation+0xb0 to 1,
inequality sets it to 0. Parse-related failure paths also clear that byte.
The vector path likewise compares projected strings before building its mapping
vectors. Its full key matching, counts, indexes and exceptional bounds paths
are not claimed complete here; readiness is only vector nonemptiness at the
final boundary, not a complete semantic validation proof.

At **0x8c968**, the shared translator virtual+0x20 is invoked; its returned
low bit is saved in w22. False skips publication and reaches cleanup. True:

1. Attach the translator shared pair to **reader+0xa0** via 0x75fc8 at 0x8c9a0.
2. Store **supplied index+1** (32-bit addition) in manager+0x18 before locking
   manager mutex+0x34. Enclosing synchronization and overflow semantics at the
   application level remain unknown.
3. Replace shared reader pair at **manager+0x20** via 0x75fc8 at 0x8c9c8.
4. Probe the stored handle's virtual+0x10 while holding the mutex. Selected
   reader binding 0x886ec tests root+0x50 nonnull. Store status **4** if handle
   nonnull and probe true, otherwise **5**, at **manager+0x30**; then unlock.
5. Owner+0x440, reader nonnull and probe true can invoke the previously traced
   keyed follow-up 0xc4338. Record a steady-clock-derived duration via 0x8bc04,
   invoke manager virtual+0x50 (selected RELA 0xd55d0 -> 0x83d50), then test
   and attempt to remove a path derived by 0xc5294. Removal return is ignored
   by this suffix. Full cleanup path semantics remain unresolved.

The final normal return at **0x8cb04** is **w22 & 1**, retained from translator
readiness. It is not replaced with the root probe or status test. Thus the
selected suffix allows a true return with status 5 if its handle/root probe
fails; feasibility of reaching that combination through every nested builder
is not established. Do not equate true with durable activation or server
acceptance. Conversely, reload has no publication call after translation, but
the translation helper can publish internally as shown here.

Reproduce with translation-body.txt, translation-readiness.txt,
translation-slots.txt and translation-reader-args.txt. Full REA evidence in
rea-translation.json covers the readiness gate and shared-handle replacement.
No Rust runtime behavior is changed from incomplete nested translation evidence.
