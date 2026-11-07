# Token configuration handles, caches and file framing

**Confirmed static**, APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Repository static DEX exports, not REA Evidence envelopes or live observations.
Follows the [concrete reader](../2.26.43.72-token-sdk-value-reader/README.md).

## Handle producers

Declared override coverage for09k.getLatestHandle spans114178 APK classes and
finds09l delegation,09m null,0EV Java file handles and MobileConfigManagerHolderImpl.
0EV lazily chooses its cachedL8k orNuz according toA04, resolving a filename
throughA00/A02 and returning null if no nonempty path produces a handle. ItsA02
optionally reads/strips/parses latest.idx, tries <integer>.mctable, then falls
back toA01's filtered file list. The comparator and filter are not traced here,
so numeric ordering/selection is not asserted from the filename alone.
The native-holder wrapper calls getLatestHandleHolder; null remains null, while
mUseFileRepo converts its filename toL8k. Native holder implementation and complete
manager selection remain outside this bundle's proof.

L8k caches the ByteBuffer returned by MobileConfigFileRepository. Nuz delegates
toL8j.A00 mapped-buffer handling, retained as a distinct untraced alternative.
The shared file repository synchronizes a filename cache and reference counts;
an existing entry whose buffer was released throws rather than reading another
file silently. A new read allocates a direct little-endian buffer, performs a
channel read/flip, validates against file length and closes resources. Failed
validation or selected IO exceptions return null. This is not a complete
file-replacement/concurrency or release-lifecycle proof.

## Header branch and cursor are not full table validation

MGo's magic is3c f1. A file shorter than8 or a buffer limit below the supplied
length returns an error. If both leading magic bytes match, signed version byte
must be1, signed body-offset byte must be positive and below length-2, stored
little-endian length must equal supplied length, and the last two bytes must
repeat the magic. Success places the buffer cursor at the body offset. A first
or second magic mismatch instead succeeds as a raw buffer at cursor0. This
allows raw historical table handling; it does not prove that arbitrary nonmagic
bytes are a valid config table. Nv3's constructor/table checks still follow.
The params_map_v4 asset prefixes from the earlier research are not automatically
this configuration-file framing or effective user values.

HeaderOracle.java is an **analyst transcription** of this branch, executed only
against synthetic buffers on the host JDK. Twelve vectors retain Java's signed
byte and cursor behavior, including partial magic, version, negative offset,
size/footer failures and a valid frame. Labels summarize branches, not exact
APK error strings. This is neither APK execution nor a complete table/file parser.

## Two cache transitions

An exactA0Y field-write scan finds constructor,A08,A0D andonConfigChanged. The
callback with a nonempty changed-name array prewarmsA01 under a monitor and
replaces latest-contextA0Y, setting null on caught failure. It then forwards the
callback toA0W when present. The two-argument overload delegates to the first.
It does not replace the per-config AtomicReferenceArrayA0Z in these bodies.

A0D refreshes session state: under its monitor it reloads overrides, replacesA0Z,
clearsA0Y, exposure/cache collections and a ready flag. Its logged-out unit-type2
branch additionally clears selected globals/flags. The direct caller inventory
finds09u.A0C and0A8.BPZ. A0C initializes the C++ manager and refreshes session state
before prewarming saved indices. The previously retained BPZ callsA0D at0722
after manager setup; its later selected result branches make00E ready and call
A0t. These are distinct SDK context and00E integer-cache transitions. A callback
alone therefore does not prove every cached token property has refreshed.

## Reproduction and remaining scope

All exports use existing apk-smali --signatures --exceptions and the class/method
named in their retained full signatures. Inventories use apk-virtual-overrides
for09k.getLatestHandle()LX/Own; with --include-interfaces, apk-field-refs for09u.A0Y
with --writes-only, and apk-callers for09u.A0D. Reuse the full prior0A8.BPZ in
[MobileConfig binding](../2.26.43.72-token-mobileconfig-binding/README.md).

```sh
java docs/protocol/apk/2.26.43.72-token-config-refresh/HeaderOracle.java > /tmp/token-header-vectors.json
cmp /tmp/token-header-vectors.json docs/protocol/apk/2.26.43.72-token-config-refresh/header-vectors.json
```

Next boundary: actual manager/flag selection, native/file hydration and updates,
translation maps and invalidation into effective00E values on the token path.
No private account files were opened. No Rust send policy, wire request, reset
or live account operation changed. Effective timing and full network parity
remain unproven.

The subsequent [parameter-map header trace](../2.26.43.72-token-param-header/README.md)
decodes selected packaged v4 headers and schema matching before lightweight-manager
translation installation. Effective values and body semantics remain open.
