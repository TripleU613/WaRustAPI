# Decrypted frame admission

## Header-only and fragmented frames terminate the epoch (2026-11-06)

**Confirmed selected static / implemented offline, APK2.26.43.72**, build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The [complete123.A08 frame reader](apk/2.26.43.72-stream-end/frame-reader.txt)
checks decrypted record length before ordinary node decoding: length1 branches
to header-only LIL at01e0. It tests compression mask0x02 and fragmentation
mask0x01. Both selected compressed/uncompressed paths reject unsupported
fragmentation rather than passing it into a stanza handler. The
[complete1s2.run](apk/2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
has selected LIL corruption handling that reports an error and returns. This
is distinct from root token2's ordinary logout callback. Full reconnect/backoff
and diagnostic side effects remain separately untraced.

Rust previously ignoredmask0x01: a flagged payload containing a syntactically
complete IQ/ping could decode and reach semantic routing. Header-only frames
fell into generic decode/compression errors and could be captured while the
actor kept receiving. Both are now recognized before node handling.

## Decode and receive behavior

unpack_frame_payload returns publicCodecError::HeaderOnlyFrame (class
header_only_frame) for a flags-only record. It returnsUnsupportedFragment
(classfragmented_frame) whenmask0x01 is set on a longer record, before parsing
or inflation. Header-only takes precedence for one-byte records. The native
compressed implementation has multiple feature-dependent branches; this early
Rust fragmentation check is a local refusal ordering, not a claim that every
native malformed compressed+fragment combination produces the same first error.
Other high bits keep their previous behavior; they are not all relabeled
fragmentation. Compression remains0x02 and existing inflation bounds remain.

The publicmessaging::terminal_frame_error maps recognized fixed classes to
transport outcomes: stream_end ->PeerClosed; header_only_frame/fragmented_frame
->PeerProtocol with fixed diagnostics. Unknown classes returnNone and keep the
existing capture policy. This helper grants no replay or settlement authority.

The normal reader consumes the split epoch, captures the exact authenticated
frame with epoch metadata BEFORE queue publication, and stops reading. Capture
failure reports an error and leaves the epoch closed. The actor's idempotent
capture then returns the mapped transport error before later nodes/ACKs/pings.
No new write may start on the consumed channel; bytes from an already-started
write cannot be retracted and remain uncertain. This reuses the
[root stream-end coordination](node-serializer.md#root-stream-end-captures-and-consumes-the-receive-epoch-2026-11-06).

Low-levelread_frame consumes the channel and returns its legacyUndecodable
carrier for capture. Captured waits preserve then returnPeerProtocol; allthree
CLI comparison loops preserve then end their windows. read_node, which has no
store argument, consumes and returnsPeerProtocol without persisting the raw
frame itself. Capture-requiring callers use read_frame or captured waits.
Undecodable inspection reports the fixed classes and never acknowledges or
reprocesses them as successful messages.

PeerProtocol uses the existingError::Transport recovery category. This preserves
caller-owned bounded fresh-handshake policy; it does not copy native global
backoff, declare a ban/refusal or permit replay of uncertain original work.
There is no new retry loop, request field, NACK, delivery fact, reservation
reset, key rollback or live account operation.

## Verification and limits

The new codec and actor fixtures failed before implementation. The actor's
fragment contains a valid IQ/ping body; neither it nor a ping queued after the
invalid frame may produce a response. Its capture retains the failure class
and original frame size. The literal matrix covers header flags0/1/2/3/255,
fragment combinations1/3/5/255 and a high-bit-only control that still decodes.

The captured-wait regression now coversstream_end, fragmented_frame and
header_only_frame with compressed/uncompressed variants where meaningful.
It leaves the following matching ACK unread, preserves exact original capture
and an uncertain checkpoint across reopen. That checkpoint is a synthetic
control, not a proof of every durable outbox family. The existing real in-flight
receipt/root-end test continues to verify outcome_unknown and no replay after
reopen through the same terminal path.

A synthetic certificate-verified encrypted-record exchange covers both malformed
frame families through BOTH read_node and read_frame, verifying later reads and
writes returnClosed. These tests perform no WhatsApp account operation. Existing
workspace/property tests retain ordinary-frame, root-control and failure rules.

**Remaining known difference:** nativeA08 skips authenticated zero-length
plaintext records and loops to the next encrypted record. Rust still exposes
that as its existing undecodable/EOF diagnostic path; this chunk does not claim
to fix it or invent a keepalive meaning. Full fragmentation/reassembly support,
native feature-dependent compression internals, nested controls, all corruption
routing and complete retry/lifecycle parity remain outside this correction.

## Reproduce the evidence

The linked checked bundles already retain complete methods, APK identity,
exception tables and checksums. From checkout root:

```sh
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A08 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1s2;' --method run --signatures --exceptions
```

PCs are bytes; branch deltas are code units. Static refusal and offline tests
are not observed server acceptance or a ban-prevention guarantee.
