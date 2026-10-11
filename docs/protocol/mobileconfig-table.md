# Managed MobileConfig table and Boolean metadata

**Confirmed static / runtime adoption unported**, 2026-10-11. WhatsApp
2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This extends the [persisted storage trace](mobileconfig-storage.md) with the
managed root table and Boolean source metadata. It does not establish the
native manager's update transport or complete effective account values.

## Table root and structural helpers

[Lwj construction](apk/2.26.37.71-mobileconfig-table/table-owner.txt)
requires a handle, non-null buffer and positive capacity, sets little-endian
order, then computes root = buffer.position + getInt(buffer.position). That
initial root read precedes the constructor's table-initialization catch range.
It is not made safe merely by the earlier file envelope check.

[MZZ.A01](apk/2.26.37.71-mobileconfig-table/field-offset.txt) computes
vtable = root - getInt(root), compares the requested offset with the signed
getShort(vtable), then returns the signed getShort(vtable + offset), or zero.
[A02](apk/2.26.37.71-mobileconfig-table/vector-start.txt) computes a vector
data address from root + fieldOffset, its relative int and four length bytes.
[A03](apk/2.26.37.71-mobileconfig-table/vector-length.txt) reads that length
at the relative vector address. These helpers perform native ByteBuffer reads;
the envelope validator does not prove all nested offsets are in range.

[M6H.A06](apk/2.26.37.71-mobileconfig-table/root-sentinel.txt) reads vtable
field 28 or zero. Construction requires 123456 there, and accepts field 34
when absent, zero or 123456. It then retains field offsets, including:

| Vtable offset | Stored field | Use |
|---|---|---|
| 6 | Lwi.A01 | Boolean value bytes |
| 8 | Lwi.A00 | Boolean metadata int vector |
| 26 | Lwi.A04 | Logging ID strings / limiter length |
| 32 | Lwi.A0A map input | Nested table map |
| 36 | Read by Ayv | Snapshot version long |
| 42 | Lwi.A09 | Auxiliary strings |

Field 32's vector has an upper count guard of 65536 before nested-map reads;
the guard alone does not reject negative lengths. Initialization catches
IndexOutOfBoundsException in its retained range; later logging-table setup has
its own bounds/underflow handling. Other failures preserve their exception
behavior. The exact DEX catch tables are retained.

[isValid](apk/2.26.37.71-mobileconfig-table/is-valid.txt) calls
[1xi.A1U](apk/2.26.37.71-mobileconfig-table/nonnull-helper.txt), which only
tests for non-null. It is not a complete structural verifier or proof that a
particular server value exists.

## Boolean metadata and value source

[Lwj.A00](apk/2.26.37.71-mobileconfig-table/metadata-index.txt) extracts
the specifier type and low-16-bit slot. For Boolean type 1, a present Lwi.A00
metadata vector reaches [Lwj.A02](apk/2.26.37.71-mobileconfig-table/metadata-word.txt),
which reads vectorStart + 4 * slot. An absent metadata vector returns zero.
Null type reports invalid type; unsupported types, missing table or caught
bounds/underflow failures return invalid-slot metadata.

The [MkK static initializer](apk/2.26.37.71-mobileconfig-table/metadata-sentinels.txt)
defines signed metadata constants: A00 = (-1 << 8) | 17 = -239,
A02 = (-2 << 8) | 17 = -495, and A01 = (-3 << 8) | 17 = -751.
The REA Java observation had no body; DEX supplies these values.
[Lwj.A07](apk/2.26.37.71-mobileconfig-table/source-metadata.txt) resolves:

| Metadata / table state | Source |
|---|---|
| Null table | DEFAULT__NO_DATA_ON_DISK |
| A00 sentinel or literal 17 | DEFAULT__MISSING_SERVER_VALUE |
| A02 sentinel | DEFAULT__MISSING_SERVER_VALUE_INVALID_TYPE |
| A01 sentinel | DEFAULT__MISSING_SERVER_VALUE_INVALID_SLOT_ID |
| Remaining low-bit combination 0 | SERVER |
| Bit 0 only | DEFAULT__SERVER_RETURNED_NULL |
| Bit 4 only | DEFAULT__SERVER_RETURNED_NULL_EMPTY_UNIT_ID |
| Bits 0 and 4 | DEFAULT__MISSING_SERVER_VALUE |

[Dep.A00](apk/2.26.37.71-mobileconfig-table/unit-source-bit.txt) maps a
nonzero bit-4 mask to 2; bit 0 supplies 1. Missing-server source omits a
snapshot version; other post-sentinel sources attach
[Ayv](apk/2.26.37.71-mobileconfig-table/snapshot-version.txt). Ayv returns
-1 for a missing table or caught bounds failure, zero for an absent field 36,
otherwise that field's long.

The previously traced 09H.AVo consults A07 only when mode metadata is enabled;
in that branch only SERVER and DEFAULT__SERVER_RETURNED_NULL proceed to A09.
Other sources return the supplied default. Without requested metadata this
source filter is skipped. The
[Boolean value read](apk/2.26.37.71-mobileconfig-boolean/snapshot-read.txt)
still independently uses its value byte: mask 0x11 retains the default,
otherwise bit 7 supplies the Boolean; a missing value vector returns false.
An absent metadata vector's zero therefore must not be generalized into an
inferred missing-server/default value.

## Per-context read side effects

When metadata's low-byte mask 0x06 qualifies, both
[A0A](apk/2.26.37.71-mobileconfig-table/access-without-exposure.txt) and
[A0B](apk/2.26.37.71-mobileconfig-table/exposure.txt) select limiter index
(metadata >>> 8) & 0xffffff and use separate compareAndSet(0,1) limiters.
They resolve the field-26 logging ID and snapshot identity. A0A calls
logAccessWithoutExposure; A0B calls logExposure with the specifier and,
when bit 3 qualifies, a stack trace. Their exact catch ranges and failure
behavior are retained; full manager logging/update effects remain open.
These operations are not configuration adoption or receipt transmission.

The remaining boundaries include complete parameter-map and overlay parsing,
manager selection/update transport, and callback lifecycle. No runtime
configuration, account identity, keys or request IDs changed; no live request
was made. The [recipe](apk/recipes/2.26.37.71-mobileconfig-table.json)
retains 15 DEX topics; sixteen full REA envelopes retain engine identity,
source observations and limitations.
