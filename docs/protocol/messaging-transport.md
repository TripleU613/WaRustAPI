# Messaging transport

The opt-in `experimental-transport` CLI feature now supports a comparison-build
connection, primary login and one durable direct text send. A live test received
server acknowledgement and operator-confirmed handset receipt. The operator
reported replying, but this client has not read that reply; protocol delivery/read
receipts remain unverified. Default
CLI `connect`/`send` and the ordinary outbox actor remain unconnected to this lab path.

## Implemented

- Canonical comparison-APK raw/direct endpoints, with bounded DNS/TCP connection
  establishment and no application retry or fallback loop.
- One `WA 06 03` header per connection, then three-byte big-endian frame lengths.
- Bounded fragmented/coalesced frame reads and complete frame writes.
- Connection epochs, operation deadlines, redacted I/O errors and socket closure
  after EOF, transport errors or canceled I/O. A partially written record is
  never retried on the same socket. Local oversize rejection performs no I/O.
- Certificate-verified classical handshake, primary login from the saved numeric
  registration login and Noise identity, and comparison-token stanza encoding.
- Recipient signed-prekey verification, pinned peer identity, upstream Signal
  encryption, atomic session/send-intent commit before I/O, and matching message
  acknowledgement persistence. No retries of refused or uncertain sends.

Wire choices come from the **2.26.37.70 comparison** mapping in
`transport/comparison.rs`; they do not change the pinned 2.26.36.75 profile's
evidence status. The default 65,535-byte body limit and 15/30-second deadlines
are configurable local limits, not claims about server requirements.
TLS-edge and pseudo-HTTP port-80 endpoints are explicitly rejected until their
wrappers are implemented, never silently downgraded.

Tests exercise exact wire bytes, a real loopback TCP round trip, coalesced
maximum-size frames, malformed/truncated input, stale epochs, cancellation,
deadlines and local validation. Additional tests cover real-signature certificate
checks, withheld login for an untrusted root, encrypted records, primary login
construction and a local Signal encrypt/decrypt round trip with APK padding.

## Explicit lab usage

Build `cargo build --locked -p whatsapp-cli --features experimental-transport`.
Use the same encrypted store, account, private registration JSON and master-key
environment as the completed registration. The optional message file is separate
from frozen registration inputs; it contains `to` (E.164 digits), `text`, and a
stable `request_id` (1–64 ASCII letters/digits/hyphens). Keep it private, mode 600.

```sh
whatsapp-cli --store <saved.redb> --account <account> lab-transport \
  --config <private-registration.json> --allow-live-request --login \
  --message <private-message.json>
```

Without `--login`, this checks only the server certificate. Without `--message`,
login sends no chat. Reusing an acknowledged message's request ID reports the
saved outcome without a connection; changing its recipient/text is rejected.
An uncertain/refused send is preserved and blocked, not re-encrypted or replayed.
The lab checkpoints retain encrypted key replies, pending stanzas, ciphertext,
intent and acknowledgement. Do not remove them to retry.

## Evidence and observed results (2026-09-18)

All APK offsets below are method-relative, for comparison **2.26.37.70**:

- `LX/1N7` constructor/A03/A04: protobuf handshake envelope, `WA 06 03`
  prologue, remote key/certificate verification and encrypted client finish.
  `LX/Aw1.<clinit>` provides the committed `WhatsAppLongTerm1` root.
- `LX/1NZ.A06` at 0x000e–0x001c hashes the first ephemeral only. There is no
  empty payload hash after ClientHello. Standard `snow::HandshakeState` adds
  that hash; its first live attempt failed decryption. `transport/classic.rs`
  reproduces APK sequencing using existing X25519/AES-GCM primitives from snow's
  resolver, SHA-256 and HKDF. No certificate check was relaxed.
- `LX/1LC.AAU` 0x003e–0x0068 supplies numeric identity; `LX/1Ed.AAU`
  0x0032–0x0150 and 0x03fa–0x0478 supply Android/version/locale. `LX/1IC.AAU`
  supplies passive/short-connect/reason. Unknown hardware fields are omitted,
  not fabricated. This minimal 34-byte login received `success` live.
- `LX/0y0.A00` 0x1786–0x193a constructs `iq/get`, `xmlns=encrypt`,
  `key/user/jid`. One live result supplied identity, signed key and one-time key.
- `LX/0jc.A0Q` calls `LX/Cb1.A00` before encryption; the latter appends 1–16
  copies of a random padding-count byte. `LX/C6f.A00` parses `enc`, `v=2`,
  `type=pkmsg/msg`. The comparison token dictionary comes from the committed
  APK bundle, separately packaged as a library fixture.
- The first direct encrypted text received matching `ack`, `class=message`,
  with no `error`. The same request was then replay-tested locally: no network
  connection and no second send. Recipient and keys are intentionally omitted.

Wire message IDs currently use random 128-bit uppercase hex: a locally chosen
opaque identifier whose acceptance was observed, not a recovered APK ID recipe.

## Remaining work

The initial path targets the specified recipient's primary phone JID only.
Companion-device discovery/fanout, LID routing, continuous inbound processing,
delivery/read receipts, prekey replenishment, robust session reconciliation and
integration with the ordinary outbox/runtime remain incomplete. The classic
branch worked in this experiment; this does not prove hybrid modes unnecessary.

The low-level frame API is for encoded handshake/encrypted records, not plaintext
chat or raw credentials. It serializes operations through one mutable connection;
a long-lived concurrent reader/writer actor remains future integration work.
