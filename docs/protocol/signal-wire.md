# APK-compatible Signal wire encoding

2026-10-12. Pinned APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The experimental Rust `signal::wire` encoder constructs the APK's version byte
before authenticating the nested Signal message, and encodes paired KEM fields
in the outer prekey envelope. It is a low-level encoder, not encryption,
identity authority, ratchet advancement or completed hybrid-session support.

## Native framing and MAC

The [five-topic recipe](apk/recipes/2.26.37.71-signal-wire.json) retains complete
constructor overloads, MAC method and both protobuf descriptors in a
[ten-file bundle](apk/2.26.37.71-signal-wire/targets.json).
The complete [REA MAC record](rea/2.26.37.71-signal-wire/mac.json) complements
[the previously retained constructor/pending-state traces](prekey-kem-wire.md).

DRy and DRw both construct `(version << 4) | version`, yielding 0x33 or 0x44.
DRy's protobuf fields are ratchet bytes 1, uint32 counter 2, uint32 previous
counter 3, ciphertext bytes 4. Both zero counters and present-empty bytes remain
present. DRy.A00 uses HMAC-SHA256 under the supplied MAC key; for versions >=3
it prepends serialized sender identity, then serialized receiver identity to the
original header+protobuf bytes and truncates the result to the first eight bytes.
The version byte is authenticated. Rewriting 0x43 into 0x44 on an already
MACed message is incorrect.

DRw's outer protobuf uses ordinary prekey ID 1 (optional), base bytes 2,
identity bytes 3, nested message bytes 4, registration uint32 5, signed prekey
uint32 6, KEM prekey uint32 7 and KEM ciphertext bytes 8. Its KEM optional writes
both fields together. This outer envelope has no separate MAC. Native outbound
constructors use the same session version for inner and outer messages.

## Public Rust API and bounds

Under `experimental-signal-crypto`:

- `wire::encode_signal(version, mac_key, identities, ratchet, counters, ciphertext)`
  encodes already-encrypted ciphertext and computes its MAC. Identity tuple
  order is `(sender, receiver)`; counters are `(current, previous)`. MAC keys
  have a fixed 32-byte input type. Supported versions are 3 and 4.
- `wire::encode_prekey(version, registration, prekeys, keys, nested, kem)` wraps
  canonical nested bytes. Prekeys are `(optional_one_time_id, signed_id)`;
  keys are `(base, sender_identity)`; KEM is optional `(id, ciphertext)`.
  Presence is retained for ID zero and empty bytes. These values alone do not
  prove a valid KEM key/ciphertext or trusted peer.

Prost encodes these small observed schemas in ascending tag order. HMAC is
provided by the existing HKDF dependency's maintained HMAC implementation;
no new dependency, primitive, unsafe code or vendored backend is introduced.

Both encoders enforce the existing local 64 KiB envelope bound, with input
bounds before allocation. The prekey constructor requires the nested message's
byte to match the same canonical version and validates its structural shape
with the existing backend parser. This is a local outbound-constructor rule,
not a claim that the APK's incoming parser rejects every differing low nibble.
The constructor does not verify the nested MAC or reauthenticate supplied bytes;
callers must use correctly authenticated bytes from the matching session.

No serializer is added for secret MAC keys and no CLI key-material command is
introduced. The helpers neither read nor write a store, adopt keys, allocate
IDs, advance counters, perform encryption nor send anything. They must be wired
into the authenticated session coordinator before normal hybrid messaging.

## Independent vectors and remaining integration

The [Python standard-HMAC/protobuf reproducer](../../tools/apk-research/signal_wire_vectors.py)
creates [24 nested and eight outer synthetic fixtures](signal-wire-vectors.json).
They cover versions 3/4, counter zero/127/128/u32 maximum, empty/16/32-byte
ciphertext, ordinary ID absence/zero and KEM absence/presence with a 1568-byte
opaque ciphertext. The public identity/ratchet and MAC bytes are fixed synthetic
inputs, not account keys or authenticated DH/KEM sessions. Neither fixture
creation nor the Rust comparison executes APK code or encrypts plaintext.

Rust compares complete wire bytes with those independent fixtures. The existing
backend verifier validates every nested MAC, and changing only the low version
nibble fails authentication. Version 3 bytes also match the backend constructor;
version 4 compares native 0x44 against the backend's differing 0x43 output.
Outer bytes round-trip through the backend parser and the KEM metadata inspector.
Present-zero/empty KEM values are checked separately. Unsupported versions,
noncanonical nested headers and oversized envelopes refuse with fixed errors.

```sh
python tools/apk-research/signal_wire_vectors.py \
  docs/protocol/signal-wire-vectors.json --check
cargo test -p whatsapp-api --all-features \
  signal::wire::tests::exact_native_version_bytes_and_macs_match_independent_vectors
```

This closes version-4 header/MAC encoding in the new low-level API. The existing
classic backend and application send path are unchanged; their version-4
constructor gap is not silently patched after authentication. Hybrid chain
construction, pending records, KEM inventory, outgoing coordinator integration,
incoming decapsulation and live interoperability remain unfinished. All current
PQ admission/receive guards stay active. No saved identities, sessions, requests,
IDs, uncertain sends or server waits changed. No live operation ran.

Validation passed 976 workspace/all-feature tests, strict all-target Clippy,
warnings-denied Rustdoc, 26 default CLI/API-symbol tests, the experimental CLI
build, formatting and local link/artifact checks. Independent fixture creation
reproduced 24 nested and eight outer cases; the APK bundle reproduced ten files
/ 61,581 bytes exactly. No live version-4 or hybrid-session operation was tested.
