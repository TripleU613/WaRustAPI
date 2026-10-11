# Java MobileConfig manager: persisted snapshots

**Confirmed static / runtime adoption unported**, 2026-10-11. WhatsApp
2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This extends the [snapshot and translation trace](mobileconfig-snapshots.md).
It establishes the Java manager's file-loading boundary, not the native
manager's network update or complete table semantics.

## Manager and file selection

[0Dk construction](apk/2.26.37.71-mobileconfig-storage/manager-owner.txt)
retains a root directory, session, unit, exposure dependency and two mode
Booleans. Its [getLatestHandle](apk/2.26.37.71-mobileconfig-storage/latest-handle.txt)
returns a cached GQr when A04 is true, otherwise a cached GQt. If the selected
handle is null it resolves the current file path and constructs a handle only
when that path is nonempty. This getter alone does not establish later handle
replacement or lifecycle.

[A00](apk/2.26.37.71-mobileconfig-storage/manager-directory.txt) combines
the root with [0Dl.A00](apk/2.26.37.71-mobileconfig-storage/unit-directory.txt).
Unit 1 uses `sessionless.data/`, unit 2 uses `<session>.data/`, and other units
use `u<unit>_<session>.data/` with this release's empty variant suffix. These are
native file-layout names, not directories created in TeleWhatsApp.

[A02](apk/2.26.37.71-mobileconfig-storage/index-selection.txt), when enabled
by A0C, tries a trimmed integer in latest.idx and the corresponding
`<integer>.mctable` file; errors or a missing file fall back to A01. A successful
canonical-path result is returned without checking whether some higher-numbered
file exists. [A01](apk/2.26.37.71-mobileconfig-storage/table-selection.txt)
filters .mctable names, parses the filename prefix as an Integer and skips
NumberFormatException. [Comparator selector 0](apk/2.26.37.71-mobileconfig-storage/table-order.txt)
compares the second operand's integer to the first through
[natural comparison](apk/2.26.37.71-mobileconfig-storage/natural-compare.txt),
so candidates are descending by filename number, not modification time.
Canonical-path errors are logged and selection continues. Missing candidates
return the empty string. The [filter's DEX](apk/2.26.37.71-mobileconfig-storage/filename-filter.txt)
settles the unavailable standalone REA body.

## Buffer ownership and validation

[GQr.getJavaByteBuffer](apk/2.26.37.71-mobileconfig-storage/cached-buffer.txt)
returns its cached non-null buffer; otherwise it calls
[MobileConfigFileRepository.getJavaByteBuffer](apk/2.26.37.71-mobileconfig-storage/buffer-cache.txt).
That repository synchronizes its filename-keyed cache and increments an
existing buffer's reference count, or loads and inserts a new buffer. A cached
entry with a released/null buffer throws. Release/finalization and replacement
policy remain outside this trace.

The repository's [readBuffer](apk/2.26.37.71-mobileconfig-storage/direct-read.txt)
allocates a direct ByteBuffer, sets little-endian order, performs its channel
read, flips and validates it. [GQt.getJavaByteBuffer](apk/2.26.37.71-mobileconfig-storage/mapped-buffer.txt)
instead calls [GQv.A00](apk/2.26.37.71-mobileconfig-storage/mapped-read.txt),
which maps the channel read-only, sets little-endian order and uses the same
validator. Both close their channel/input resources and return null on a
reported validation error or IOException. Other failures retain their native
exception behavior; neither helper is a permission to invent an empty config.

[Hq6.A00](apk/2.26.37.71-mobileconfig-storage/file-validation.txt) rejects
file lengths below eight and buffers whose limit is below the supplied length.
It checks the two magic bytes `3c f1` from the
[static initializer](apk/2.26.37.71-mobileconfig-storage/file-magic.txt).
If either header byte mismatches, it resets position to zero and accepts the
bare body. With matching magic, it requires version 1, a positive signed-byte
body offset smaller than length minus two, and an encoded little-endian file
length equal to the supplied length. It then checks both footer magic bytes
and positions the buffer at the body offset. This is envelope validation;
complete nested FlatBuffer validation is not established by this method.

## Translation table provenance

The [HXc.A00](rea/2.26.37.71-mobileconfig-storage/translation-A00-refs.txt)
and [HXc.A01](rea/2.26.37.71-mobileconfig-storage/translation-A01-refs.txt)
censuses each find constructor initialization, 09N.BJ1 assignment and Heh.A00
read. BJ1's retained
[DEX](apk/2.26.37.71-mobileconfig-boolean/manager-init.txt) tries a
`params_map_v4_u<unit>.txt` candidate and the u0 variant, reads a header when
needed and compares its schema hash against the config table. It invokes
[Hvv.A01](apk/2.26.37.71-mobileconfig-storage/params-load.txt), then requires
its parsed int array and checks the parsed header's schema again when present.
At 05be..05dc it assigns HXc.A00 from HbE.A01 and HXc.A01 from Hvv.A05,
then attaches HXc to 0Dk.A00. The lazy translator therefore consumes saved
parameter-map data; the generated WA property lookup alone is insufficient.

Hvv.A01 recognizes v4 headers (or the filename fallback), finds a corresponding
params_names file and delegates actual parsing to A00/A02. Full parameter-map
parsing, signed packing, source ordering and error recovery are still open.
The Java observation contains an uninitialized local and is not a complete
parser implementation to copy.

## Saved overlays and network boundary

[0Dk.A03](apk/2.26.37.71-mobileconfig-storage/silent-store-load.txt) is a
separate unit-2 saved silent-push initialization path. It connects the 09K
mapping, session-indexed 0Dt cache and saved JSON values, using a table-file
modification timestamp in its eligibility path. Its decompiled typed branches
contain contradictory conditions; full DEX is retained. Complete overlay
admission/application requires its helpers and exact bytecode branches and is
not inferred from the JSON keys here.

[0Dk.updateConfigs](apk/2.26.37.71-mobileconfig-storage/update-configs.txt)
logs, invokes the supplied callback's onNetworkComplete(false) when non-null,
and returns false. It performs no fetch in this body. This Java fallback method
must not be substituted for the native manager update protocol or interpreted
as successful server configuration.

The next required closures are table decoding/source metadata, complete
parameter-map and overlay application, manager selection and native update
transport, plus the callback/cache lifecycle already identified. No local
account state was changed and no live request was made. Saved AB replies
remain inspection evidence until the effective pipeline is reproduced.

The [recipe](apk/recipes/2.26.37.71-mobileconfig-storage.json) retains
20 DEX topics. Twenty-four full REA envelopes and two physical field inventories
retain source observations and limitations. Missing Java bodies are not
evidence of missing DEX code.
