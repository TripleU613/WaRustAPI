# MobileConfig snapshots, translation and invalidation

**Confirmed static / runtime adoption unported**, 2026-10-11. WhatsApp
2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This extends the [effective Boolean trace](mobileconfig-boolean.md).
The Java-manager wrapper is resolved here; manager storage, callback
registration and complete update authority remain open.
The later [storage trace](mobileconfig-storage.md) establishes file selection,
buffer validation and translation-table assignment for the Java manager.

## Java-manager Boolean wrapper

The existing 099.A01 factory constructs Lwj and, when its unwrapped Java
manager provides HXc, wraps it in
[IIY](apk/2.26.37.71-mobileconfig-snapshots/wrapper-owner.txt). IIY retains
the underlying 09G, Heh translator and 092 exposure dependency.

[IIY.AVn](apk/2.26.37.71-mobileconfig-snapshots/wrapper-default.txt) first
calls [0Dp.A00](apk/2.26.37.71-mobileconfig-snapshots/silent-lookup.txt)
with the original specifier. That helper returns null unless its global A03
is enabled and A01 supplies a 0Dt; otherwise it calls 0Dt.A05. A non-null
I3z reaches [IIY.A03](apk/2.26.37.71-mobileconfig-snapshots/wrapper-silent-value.txt):
null data, a type other than Boolean (1), or a null converted Boolean returns
the encoded bit-61 default. Valid data returns its Boolean. Optional metadata
records silent-push value or silent-push fallback default. An invalid non-null
overlay does not retry the underlying snapshot.

When lookup returns null, AVn translates through Heh and delegates to the
underlying AVn. [AVo](apk/2.26.37.71-mobileconfig-snapshots/wrapper-boolean.txt)
uses the same lookup/delegation split but keeps the caller's explicit default.
This is distinct from the underlying 09H overlay-selection path; importing a
raw AB Boolean does not reproduce either authority.

## Specifier translation

[Heh.A00](apk/2.26.37.71-mobileconfig-snapshots/translation.txt) returns an
input unchanged when bit 62 is set. Otherwise it extracts config index bits
32..47 and the parameter index through
[0ED.A00](apk/2.26.37.71-mobileconfig-snapshots/parameter-index.txt). Unmarked
specifiers use bits 16..31; marked specifiers further restrict the extracted
parameter index to 12 bits.

For an unmarked input, Heh obtains HXc from its provider and a config key from
the unit-indexed [[I table. Missing HXc/table/config key or no match uses
65535. Its packed HXc int pairs are searched in two ranges split at HXc.A00,
matching configKey (second int >>> 12) and parameter (second int & 4095).
Odd pair-array lengths fall back. A matching first int supplies its high
16 bits with a signed right shift. The DEX establishes termination and the
two search ranges; decompiled Java displayed malformed nested loops and must
not be ported literally.

[0ED.A01](apk/2.26.37.71-mobileconfig-snapshots/specifier-builder.txt)
rebuilds the value from type, config/parameter composite, translated index,
unit, bit-60 flag and bit-61 default. Its Java signed shifts/casts/OR operations
are significant: the translated index is not blindly masked to u16 before
being ORed. HXc population, ordering guarantees and missing-index snapshot
behavior still require their own closure.

## Two context caches and mode selection

099.A08 owns latest cache A0X; A07 owns per-config AtomicReferenceArray A0Y.
Their [latest](rea/2.26.37.71-mobileconfig-snapshots/latest-field-refs.txt)
and [per-config](rea/2.26.37.71-mobileconfig-snapshots/per-config-field-refs.txt)
physical reference censuses find six and seven sites respectively. They do
not establish escaped-reference or native mutation coverage.

The [0E8 constructor](apk/2.26.37.71-mobileconfig-snapshots/context-mode-init.txt)
sets A01 and metadata A02 false; the
[0E7 constructor](apk/2.26.37.71-mobileconfig-snapshots/base-mode-init.txt)
sets its A00 false. The [static initializer](apk/2.26.37.71-mobileconfig-snapshots/context-modes.txt)
establishes:

| 0E8 mode | Latest-selection A01 | Base A00 |
|---|---|---|
| A05 | False | False |
| A03 | True | False |
| A06 | False | True |
| A04 | True | True |

[0E8.A00](apk/2.26.37.71-mobileconfig-snapshots/context-mode-builder.txt)
clones those shared modes before mutation. Its Java initializer observation
omitted the A05 field initializer; actual DEX includes it at 0000..000e.
The existing 099.AVo chooses latest A08 iff mode.A01, otherwise per-config A07.
These are 0E8 fields, not the separate 00F fallback-cache mode Boolean.

## Snapshot update boundaries

[onConfigChanged(String[])](apk/2.26.37.71-mobileconfig-snapshots/config-changed.txt)
ignores null/empty input. Otherwise it synchronizes, constructs a fresh
context via A01 and replaces only A0X. Construction Throwable is logged and
A0X becomes null, permitting later lazy reconstruction. The two-argument
overload delegates and does not use its second argument. Neither body clears
A0Y nor the separate 00D fallback caches.

[099.A0D session refresh](apk/2.26.37.71-mobileconfig-snapshots/session-reset.txt)
refreshes its overrides table, replaces A0Y, nulls A0X and clears its context
tracking/09B map state under synchronization. Its logout predicate is
unit == 2 && session == ""; DEX 0000..0026 establishes false for other units
where Java showed an uninitialized local. Logout also clears the 3Bi map,
updates A0G/A0H/A0F/A0E and nulls A0Z. This is not the 00D.A0o reset.

[onEpConfigChanged](apk/2.26.37.71-mobileconfig-snapshots/ep-changed.txt)
has a separate parameter/version comparison and restart decision flow. At
the guarded clear branch (DEX code-unit 0285, byte 050a), it synchronizes and
sets one in-range A0Y entry null. It does not globally replace the array.
Full EP action admission, restart decisions and manager update ordering are
not closed by that one operation. The retained source is smali because Java
was an anti-decompilation stub.

The remaining A0Y reader in
[IhL.run selector 30](apk/2.26.37.71-mobileconfig-snapshots/consistency-reader.txt)
collects non-null cached entries for consistency logging after A05 and
isConsistencyLoggingNeeded qualify. Its presence is not an additional cache
adoption/reset. The full method is retained to preserve switch identity.

## Remaining port requirements

The next required boundaries are manager handle/storage construction,
translation-table population, overlay updates, callback registration/order
and complete EP policy. Until those can be reproduced, effective predicates
remain unported. No real preferences, timestamps, identities or keys changed;
no live request was made.

The [recipe](apk/recipes/2.26.37.71-mobileconfig-snapshots.json) retains
16 DEX topics, including both config-change overloads. Seventeen full REA
envelopes and two full field inventories retain the observed sources and
coverage limitations. Missing decompiled constructor bodies are observations,
not proof of absent DEX code.
