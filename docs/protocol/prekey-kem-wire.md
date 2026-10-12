# Incoming KEM prekey wire metadata and receive boundary

2026-10-12. Pinned APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Incoming prekey inspection now retains paired KEM ID/ciphertext metadata rather
than silently losing it through the classic backend. Incomplete pairs refuse;
consumable receive stays unsupported until hybrid sessions are integrated.
This is a wire inspection and receive fix, not completed PQ messaging.

## Native schema, pair rule and version byte

The [four-topic recipe](apk/recipes/2.26.37.71-prekey-kem-wire.json) reproduces
[nine evidence files](apk/2.26.37.71-prekey-kem-wire/targets.json), retaining full
constructor overloads, DEX signatures, register flow and exception tables.
Complete REA records preserve [prekey encoding](rea/2.26.37.71-prekey-kem-wire/prekey-encode.json),
[prekey decoding](rea/2.26.37.71-prekey-kem-wire/prekey-decode.json),
[inner Signal encoding](rea/2.26.37.71-prekey-kem-wire/signal-encode.json) and
[pending-state consumption](rea/2.26.37.71-prekey-kem-wire/pending-send.json).

BY9.dynamicMethod's descriptor assigns:

| Tag | Field | Wire type |
| --- | --- | --- |
| 1 | optional ordinary prekey ID | uint32 |
| 2 | base key | bytes |
| 3 | identity key | bytes |
| 4 | nested Signal message | bytes |
| 5 | registration ID | uint32 |
| 6 | signed prekey ID | uint32 |
| 7 | KEM prekey ID | uint32 |
| 8 | KEM ciphertext | bytes |

DRw's byte-array constructor accepts high-nibble versions 3 or 4 and ignores
the low nibble. It requires ordinary fields 2/3/4/6 by presence, reads optional
registration/one-time IDs, then requires KEM fields 7 and 8 either both present
or both absent. One alone throws Incomplete message. ID zero and present-empty
ciphertext are still present at this metadata boundary, not proof of a valid
KEM input. It separately constructs DRy from the nested message.

Cnp.A03 reads a pendingKyberPreKey record when its state bit is set and passes
its ID/ciphertext together to DRw's outbound constructor. That constructor
writes both fields and prepends `(version << 4) | version`. DRy's outbound
constructor uses the same byte expression before computing the nested MAC.
Thus native version 4 emits **0x44** at both layers, while version 3 emits 0x33.

The locked backend accepts high versions 3..4 but its typed PreKeySignalMessage
retains neither KEM field even though the underlying waproto schema has them.
Its constructors use CIPHERTEXT_MESSAGE_CURRENT_VERSION=3 as the low nibble,
so requesting version 4 emits 0x43. Its pending setter and initialization also
remain classic. These precise gaps must be fixed for normal hybrid messaging;
this change does not rewrite version bytes or alter authenticated nested data.

## Rust inspection and admission

`signal::direct::incoming_prekey_kem(bytes)` returns optional borrowed
`IncomingKem { prekey_id, ciphertext }`. Debug is fully redacted. The original
ciphertext slice is retained without reencoding, copying or normalization.
This is envelope metadata, not signature/MAC verification, identity trust,
KEM format validation or an authenticated key selector.

A shared parser validates the existing bounded classic/nested structure and
then scans the original outer protobuf for the KEM pair. The existing
`incoming_prekey_id`, identity-match and public-fingerprint inspection APIs use
that same parser, so incomplete KEM metadata cannot appear complete to another
caller. Their metadata output on a complete pair is still inspection only.

The shared protobuf byte scanner keeps the last correctly typed scalar/bytes
field, including zero/empty values; other wire types remain unknown. Unknown
groups are skipped as groups rather than promoted to root fields. Native-style
uint32 truncation reuses the house parser. The complete envelope remains subject
to the backend's existing parser behavior and the local 64 KiB bound; this does
not prove every Java protobuf malformed-input or recursion behavior is matched.

`receive_prekey_with_policy` refuses any complete KEM pair before interpreting
saved sessions, keys or pins. Direct and group pairwise inbox paths return
Unsupported before companion authority, identity-conflict observation or local
prekey lookup. Unsupported group stanzas with a pairwise KEM envelope remain
whole-stanza unsupported, including an accompanying sender-key ciphertext;
partial group/session integration is not attempted. Original captures survive
for later supported replay. Classic envelopes without KEM retain their behavior.

## Regression, recovery and remaining work

The regression used a genuinely encrypted classic nested message and appended
KEM ID zero plus present-empty ciphertext. Before this fix, receive succeeded
because the backend discarded those fields and the classic MAC passed. It now
refuses rather than consuming a one-time key or deriving classic state from an
envelope carrying unimplemented KEM input. The bytes are deliberately not a
valid native KEM fixture; this proves the unsupported boundary.

Tests cover complete/absent/partial pairs, repeated last values, zero/empty,
high-version 4 with an ignored low nibble, redacted Debug, wrong-wire scanner
fields, unknown groups, truncation and malformed input. All inspection callers
refuse partial metadata. Direct and group inbox tests preserve every saved table
and journal head after capture under pinned and APK-authenticated policies.
Reopen repeats refusal with a held classic session and consumed ordinary prekey;
no conflict, archive, ratchet, prekey deletion or user event appears. The existing
classic receive, redelivery and ratchet-continuation checks still run.

[Rust KEM](rust-kem.md) and [initial hybrid KDF](hybrid-kdf.md) exist. Pending
record integration, native version-byte encoding, full hybrid chain construction,
local KEM inventory, incoming ciphertext decapsulation, key authority and live
interoperability remain unfinished. No account or network operation ran for this
change. No at-rest format, dependency, request ID or server wait changed.

Validation passed 975 workspace/all-feature tests, including ordinary own-device
route preservation and direct/group KEM table/journal snapshots after reopen.
The crypto-only regression, strict all-target Clippy, warnings-denied Rustdoc,
26 default CLI/API-symbol tests, experimental CLI build, formatting and local
link/artifact hygiene passed. The APK bundle reproduced nine files / 67,602
bytes exactly. No live PQ receive or outgoing version-4 operation was tested.
