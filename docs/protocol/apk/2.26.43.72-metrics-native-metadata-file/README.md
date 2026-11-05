# Native metrics metadata file boundary — APK 2.26.43.72

**Confirmed static / derived flow**, reconstructed ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
Pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Reconstruction verification](../2.26.43.72-superpack-reconstructed-members/README.md)
binds the image. No metrics code, account or live traffic was executed. Local
Rizin exports are not REA Evidence envelopes.

## Concrete path and open modes

The [snapshot writer](../2.26.43.72-metrics-native-drain-persistence/README.md)
uses helper 0x6fdb7c. That helper reads configured string slot +0x28, copies it
through 0x6fdc64 and appends exact suffix **`/wam-metadata`**. It creates a file URL
and calls opener **0x733ce0** with the raw mode index passed into the helper.
The full lifetime/authority of the configured directory remains untraced.

Opener 0x733ce0 converts the URL to a filesystem path, allocates a 0x400-byte
C-string buffer, obtains its C representation and calls fopen. It returns a
native file object on success; conversion/open errors use the diagnostic helper,
release selected allocations and return null. No automatic retry is present in
this selected opener. File locks, permissions and parent directory setup are
not established here.

ELF relative relocations resolve its three mode-table slots to **0: rb, 1: wb,
2: ab**. The metadata recovery caller passes mode zero through helper 0x5ce230;
the selected snapshot-writing body passes mode one through 0x5cf848. These are
actual fopen strings, not guessed Android numeric flags. Opening the write handle
with wb is a truncating open; this selected path does not demonstrate atomic
replacement. It is not a reason to weaken Rust's durable-store behavior.

## Rewind and the normal snapshot guard

The cached snapshot file handle is not blindly appended to: before constructing
and writing the snapshot, 0x6fdca0 invokes **0x7343a4** with offset zero. The
helper calls fseek with that offset and raw origin zero, handles errors and
clears a state byte on success. A nonzero result skips the write. The
[previous fwrite/fflush trace](../2.26.43.72-metrics-native-drain-persistence/README.md)
therefore follows a selected rewind-to-start path. Failure/recovery across
crashes, handle replacement and all other payload-file behavior remain open.

The normal pending-worker interior call at **0x7242b0** passes raw argument zero
to wrapper 0x724fc8; another call at 0x725360 also passes zero. That wrapper
checks the current context on this zero branch and keeps the zero argument for
0x6fdca0, matching that function's raw zero guard. Snapshot helper 0x71fdf4's
complete body is retained alongside the call sites. This closes a concrete
normal-path argument gap from the earlier bundle, not every caller's reachability.
The separate terminating branch at 0x724f1c supplies one. Its complete outcome
remains unresolved and must not inherit the normal-path write claim.

The direct-branch scan finds only 0x725014 calling 0x6fdca0 in executable
PROGBITS sections of this image. File-boundary.json retains every selected direct
B/BL reference to opener, snapshot writer and wrapper. This does not search
indirect calls, callbacks, other libraries or runtime-generated code.

## Recovery envelope and selected fields

Recovery body **0x72b1d8** opens the metadata file in rb mode, asks helper 0x734174
for **0x69c (1692) bytes**, and checks returned data/error plus exact data length.
It compares the first three bytes with **MTD** and byte 3 with version **4**.
The selected writer's literal at VA 0x18a190 is exactly **4d 54 44 04**, matching
that envelope. This is a local metrics metadata format, not a network header.
No native read/write or actual saved metadata file was observed.

After those checks it allocates a local snapshot and feeds raw fields into
setter helpers: u64 at offset 4 and byte at 0xc; four-u32 groups at 0x10, 0x20
and 0x30; four u32 at 0x40..0x4c; a bounded key/value region beginning at 0x50;
and trailing u32/double at 0x690/0x694. The key/value loop processes at most 100
16-byte entries and stops when the next key is zero. Full meanings, cross-field
constraints and counter authority have not been reconstructed. Some helper
validation/fallback paths remain semantic unknowns. Do not map these bytes onto
Rust account credentials, message acknowledgements or replay authority.

On invalid/open-failed input it reaches selected default-state construction
paths. A later validator 0x71fc84 can also reject/replace state; its full policy
remains untraced. The selected read requests the fixed prefix; this is not proof
that trailing file bytes are rejected. Cleanup calls native close/release helpers;
complete fclose/error policy is not assigned from those addresses alone.

## Reproduction and remaining work

Run `python inspect-file-boundary.py RECONSTRUCTED_LIBWHATSAPP.so`; stdout must
match file-boundary.json. It reuses the adjacent pinned-image resolver, resolves
mode relocations, verifies exact path/magic/writer bytes, hashes mapped windows
and independently scans executable-section direct branches. Reproduce assembly
with Rizin `pD SIZE @ VA` using those windows. Complete selected path/open/recovery,
seek, snapshot creation and normal call-site bodies are retained; callees remain
separate unknowns. The existing [section map](../2.26.43.72-metrics-native-event-routing/elf-sections.txt)
and [tool version](../2.26.43.72-superpack-decoder-entry/rizin-version.txt) apply.

Next trace the snapshot validator and actual event buffer format/flush/transport,
then server-response and replay behavior. No Rust runtime, telemetry, retry,
sender, HTTP route or live account/network operation changed.
