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

## Authenticated empty records are skipped (2026-11-06)

**Confirmed pinned static / implemented offline:** the same complete123.A08
checks decrypted length0 and loops to the next encrypted record. This does not
establish a keepalive meaning or permission to transmit empty records.

Rust now skips zero-length authenticated plaintext before node decoding in the
normal split reader and both low-level stanza receive APIs. The raw encrypted
record API still returns the authenticated plaintext, including empty records;
pure offline decode_frame([]) still reports unexpected_eof. This distinction
keeps authentication/counter consumption in the cryptographic layer and stanza
admission in its caller. Empty records create no stanza capture, ACK, callback
settlement or new retry. The skip yields cooperatively so buffered records do
not prevent cancellation/deadline polling. It introduces no timeout of its own
and stays within the caller's existing reply deadline.

The certificate-verified in-memory encrypted fixture first failed with
Undecodable { bytes:0, class:unexpected_eof }. It now receives the following
literal IQ through BOTH read_node and read_frame, demonstrating consecutive
record decryption with correct counters. A second fixture supplies an empty
record and no stanza: both bounded reads expire instead of returning a false
diagnostic; cancellation during the following pending read consumes the channel
and later writes refuse. Existing request-deadline regression also remains.
These encrypted fixtures exercise low-level APIs; the normal split-reader guard
is code-reviewed and covered by the broader workspace suite, not claimed as a
new end-to-end actor/socket empty-record test.

Full fragmentation/reassembly support, native feature-dependent compression
internals, nested controls, all corruption routing and complete retry/lifecycle
parity remain outside these corrections. No live account operation or
ban-prevention guarantee is established.

## Compression failure consumes the receive epoch (2026-11-06)

**Confirmed selected static / implemented offline:**123.A08's two compressed
branches use InflaterInputStream; the streaming variant drains the remainder to
EOF before returning its parsed node. IOException during read/parse/drain is
not caught as a recoverable stanza exception.1s2.run cleans its reader-local
map, rethrows to the outer IOException handler, invokes onError and returns.
[Complete sources, exact exception ranges and concrete listener routing](apk/2.26.43.72-compression-failure/README.md).
This is distinct from nativeLIL error2 and is not evidence of native replay or
account-ban policy. Android inflater runtime behavior was not executed here.

Rust previously treated CodecError::Compression/classcompression as an ordinary
undecodable frame and could keep reading or writing. terminal_frame_error now
maps that class to PeerProtocol with a fixed diagnostic. Existing normal-reader
coordination consumes the split epoch and preserves the exact authenticated
record before queue publication; actor/captured waits/CLI windows stop before a
later ACK or ping. read_frame closes the channel and returns the capture carrier;
read_node closes and returns the transport error. No-store callers still cannot
persist captures on their own. No new NACK, acceptance, callback completion,
reservation reset or resend is introduced. Already-started writes remain uncertain.

The existing bounded inflation failure also uses classcompression and now
terminates. This is a local fail-closed size policy, NOT a native maximum or a
claim that every malformed input takes the identical native exception branch.
Unknown noncompression codec classes retain their existing capture behavior.
Successfully inflated but undecodable nodes keep their distinct codec classes;
this change does not establish parity for those parse-failure branches.
Generic reader-local map cleanup is not ported into durable state deletion.

The real encrypted receive fixture failed before correction because a subsequent
write was not rejected asClosed. Both read_node/read_frame now reject later reads
and writes after invalid zlib and corrupted-Adler-checksum records containing an
otherwise complete literal IQ. The extended actor ping fixture also failed before
correction; it now captures and stops before a following ping can produce a
response. Actor delivery uses the existing fake node link, not a new full socket
integration test. A real durable in-flight receipt stays outcome_unknown after
compression failure followed by its matching ACK and a ping, survives reopen,
and is not replayed by the next activated actor. Captured-wait tests retain exact
raw bytes and an unchanged synthetic uncertain checkpoint, leaving the following
matching reply unread. Positive compression and local output-bound codec controls
remain in the workspace suite. No live account operation was performed.

## Truncated nodes consume the receive epoch (2026-11-06)

**Confirmed selected static / implemented offline:** native123.A06 reads every
declared buffer byte and throws IOException on early EOF.123.A07 invokes it for
all packed pairs BEFORE validating digits, so invalid prefixes do not mask
missing suffixes. NativeA00/A0A separately reject missing list/string tokens
with LIL; initial node read failures leave1s2.run through outer error handlers.
[Complete methods, exact catch routing and remaining native format gaps](apk/2.26.43.72-truncated-node/README.md).

Rust now checks the full packed byte count before interpreting any digit. The
existing CodecError::UnexpectedEof/classunexpected_eof maps through
terminal_frame_error to PeerProtocol(truncated stanza frame). Normal reader
coordination consumes the epoch and captures before publication, actor/captured
waits/CLI windows stop before following nodes, and both low-level stanza APIs
consume their channels. Pure offline decode still reports the codec error;
authenticated zero-length plaintext records remain skipped before this decoder.
A successfully inflated but truncated node now stops too, rather than being
mistaken for an ordinary undecodable record that permits continued routing.

Complete invalid packed values are also terminal: nativeA07's bad-nibble and
negative-count LIL are initial parse failures. invalid_packed_digit and the new
public CodecError::InvalidPackedLength/classinvalid_packed_length now map to
PeerProtocol with a fixed diagnostic. Giving the negative-count case its own
class avoids treating the entire invalid_node family as terminal. The packed
channel fixture failed before correction on a complete bad digit; it now tests
bad digits and negative counts through both encrypted receive APIs. Actor
uncertain-receipt/no-following-ping and compressed/uncompressed captured-wait
matrices include both classes. Colon12, full hex alphabet and ignored odd
padding remain valid. Saved old invalid_node captures are not reclassified or
replayed; only fresh decoding receives the new negative-count class.

This applies one local terminal policy to Rust's truncation class. The selected
native buffer/token failure families are confirmed; exact native exception
numbers, every reference/native JID format, negative array/count behavior and
all malformed-input routing are not claimed equivalent. Other nonterminal
codec classes retain their current capture behavior. No new NACK, retry,
settlement, callback completion, durable deletion or replay permission is added.

Literal fixtures cover missing list/count/description, raw strings, attribute
value, binary body, child node and even/odd packed suffixes. The packed invalid
prefix fixture failed before correction (invalid_packed_digit instead of
unexpected_eof); complete invalid and valid packed controls keep their results.
The certificate-verified local encrypted fixture also failed before correction
because later writes were still possible. It now covers compressed/uncompressed
truncated binary bodies through BOTH read_node and read_frame, rejecting later
reads/writes. Actor fixtures cover capture/stop before a following ping and a
real durable in-flight receipt's matching ACK: the receipt remains uncertain
through reopen and no replay occurs on the next activation. Actor inputs use
the existing fake link, not a new full socket integration. Captured-wait tests
retain exact compressed/uncompressed raw frames and an unchanged synthetic
uncertain checkpoint, leaving the next matching reply unread. No live account
operation or restriction-prevention guarantee follows from these controls.

## Reproduce the evidence

The linked checked bundles already retain complete methods, APK identity,
exception tables and checksums. From checkout root:

```sh
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A08 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1s2;' --method run --signatures --exceptions
```

PCs are bytes; branch deltas are code units. Static refusal and offline tests
are not observed server acceptance or a ban-prevention guarantee.
