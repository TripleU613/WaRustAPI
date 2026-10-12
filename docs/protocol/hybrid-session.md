# Staged hybrid session bootstrap and encrypted exchange

2026-10-12. The experimental `signal::hybrid` adapter now constructs fresh
Alice/Bob version-4 records and prepares native encrypted messages. Two offline
Rust peers bootstrap, exchange replies, reopen records and continue ratcheting
with ML-KEM-1024, both with and without an ordinary one-time EC prekey.
This closes a functional offline session path. It does not enable live PQ
admission or prove Android/server interoperability.

## Native state and cryptographic flow

Pinned APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [nine-topic recipe](apk/recipes/2.26.37.71-hybrid-session.json) reproduces
[fourteen full DEX evidence files](apk/2.26.37.71-hybrid-session/targets.json).
Complete REA records retain [Bob bootstrap](rea/2.26.37.71-hybrid-session/bob-bootstrap.json),
[root step](rea/2.26.37.71-hybrid-session/root-step.json),
[receive and pending clear](rea/2.26.37.71-hybrid-session/receiving-chain.json),
[message keys](rea/2.26.37.71-hybrid-session/message-keys.json),
[chain HMAC](rea/2.26.37.71-hybrid-session/chain-hmac.json) and
[version/KDF selection](rea/2.26.37.71-hybrid-session/version-kdf.json).
REA does not decompile the chain-seed initializer; its retained
[DEX initializer](apk/2.26.37.71-hybrid-session/chain-seeds.txt) establishes
message seed byte 1 (A04) and next-chain seed byte 2 (A03).

CQc.A00 Alice bootstrap uses the [ordered hybrid input/KDF](hybrid-kdf.md),
then COt.A00 derives a sending root/chain using the signed-prekey public and
fresh local sending-ratchet private key. It retains the original signed-prekey
receiver chain, assigns version 4, registration IDs, Alice base key and both
ordinary/KEM pending metadata.

Cnp.A04 Bob bootstrap loads its signed key, optional ordinary one-time key and
KEM key by the received ID. Its EC agreement order mirrors Alice, with the
private/public sides reversed. It decapsulates the received KEM ciphertext,
appends the secret last, selects the same hybrid label, and initializes its
sender chain with the signed-prekey pair and initial chain. Version is 4 when
KEM input is present. Key lookup, signature/trust authority, prior-session
promotion, consumption/deletion and atomic installation are outside this
adapter; callers supply already-authorized parameters and computed KEM secrets.

COt.A00 uses the prior root as HMAC-SHA256 extract salt, ratchet agreement as
input, `WhisperRatchet` as expansion info and 64-byte root/chain output. Cew.A00
selects DZC for both versions 3 and 4. Ckg.A01 derives 80 `WhisperMessageKeys`
bytes from the HMAC message seed, split into AES key 32, MAC key 32 and IV 16.
The existing backend provides these agreement/root/chain/message-key primitives
and AES-CBC encryption; the adapter does not implement arithmetic or a new cipher.
It uses the [native wire encoder](signal-wire.md) for 0x44 and MACed framing.

Cnp.A02 decrypts only the matching session version, verifies MAC/padding, and
clears both ordinary and KEM pending state after successful decryption. The
backend's ordinary pending clear removes the combined canonical pending record,
including its KEM fields. The offline reply test observes the same logical clear.
This is not a claim that the canonical Rust record is the native BYp disk format.

## API and storage contract

Under `experimental-signal-crypto`:

- `initialize_alice(parameters, kem_shared, sender_ratchet, registrations, pending)`
  builds a fresh candidate. Registration tuple is local/remote. Pending tuple is
  optional ordinary ID, signed ID, and borrowed `IncomingKem` ID/ciphertext.
- `initialize_bob(parameters, kem_shared, registrations)` builds a fresh candidate
  from already-loaded authorized keys and the matching decapsulated secret.
- `HybridSession::from_bytes` reopens a trusted version-4 canonical record;
  `to_bytes` exports secret-bearing bytes in Zeroizing storage. Debug is redacted.
- `HybridSession::prepare(payload)` returns the existing `PreparedDirect` result:
  native ciphertext/kind, updated secret-bearing record and remote public identity.
  It adds native 1..16-byte application padding before AES-CBC padding, steps
  the sending chain, and retains pending KEM metadata for repeated prekey sends.

The adapter supports the traced signed-prekey ratchet role and ML-KEM-1024
1568-byte ciphertext, with `UsePQRatchet::No`. Hybrid key exchange does not imply
a continuous post-quantum ratchet. Other parameter roles/PQR modes refuse.
The input KEM shared-secret type is exactly 32 bytes. Ordinary prekey IDs,
registration IDs and KEM IDs are supplied; no IDs are allocated here.

Preparation leaves the source record unchanged and returns a candidate.
**The caller must atomically commit that candidate before sending or preparing
another payload from the same source session.** Reusing an old source can reuse
message-key/counter material, as with the existing staged direct adapter.
Backend counter leasing remains active, and serialized/reopened candidates
advance beyond reserved sender counters. No lease is waived or reset.

Canonical backend components retain the KEM fields in pendingPreKey; this avoids
forking the backend or importing its private modules. Component projection on
an isolated clone is used only to inspect pending metadata, so its reservation
materialization cannot rewind or mutate the source record. Fresh construction
uses validated component import after setting pending fields. Preparation clones
the full record rather than creating a current-only record; archives follow
the backend's existing bounded serialization and counter-reservation rules.

Input payloads retain the local 64 KiB bound and wire encoders enforce the final
envelope bound. Canonical record import is locally bounded to 1 MiB. These are
local resource limits, not newly inferred native size policies. Serialized
records contain secret keys and must be stored encrypted; explicit byte exports
must not be logged. The adapter is not a key-store authority or APK DB importer.

## Verification and remaining integration

The offline test generates actual EC identities/base/signed/ratchet/optional
one-time keys and an ML-KEM-1024 keypair. Alice encapsulates and Bob decapsulates.
Both candidate records derive independently from their respective key roles.
Alice emits a 0x44 prekey wrapper with a 0x44 nested message and matching KEM
ID/ciphertext. The existing pinned direct receiver decrypts the nested message
with Bob's candidate; Bob emits an encrypted native reply; Alice authenticates
it, clears pending metadata, and sends another message that Bob decrypts.

Reopening Alice's unacknowledged prepared record retains the same KEM ciphertext
and emits another prekey wrapper at a higher reserved counter. Reopening after
authenticated reply emits a normal `msg`. Forged MACs and a modified KEM
ciphertext's fallback secret cannot decrypt; source records remain identical.
Oversized preparation fails without source mutation and wrong KEM ciphertext
width refuses initialization. Tests cover both optional ordinary-prekey cases.
These are two Rust peers, not an independently executed Android exchange.
The separate [native KEM/NIST comparison](native-kem-vectors.md), independent
KDF/wire vectors and static DEX traces support their respective narrower scopes.

The normal client send/receive coordinator is **not connected** to this adapter.
All existing PQ directory/receive/identity-refresh guards remain active. Still
needed: authenticated bundle authority, local KEM inventory/upload and reuse
rules, incoming ID/key lookup, atomic candidate adoption/archive/consumption,
application outbox integration and bounded fresh live interoperability. No saved
account key, request ID, uncertain send or server wait changed; no live operation
ran. RustCrypto's primitive still lacks an independent audit as noted in
[the KEM dependency review](rust-kem.md).

Validation passed 977 workspace/all-feature tests, the crypto-only encrypted
exchange/reopen regression, strict all-target Clippy, warnings-denied Rustdoc,
26 default CLI/API-symbol tests, the experimental CLI build, formatting and
local link/artifact hygiene. The nine-topic APK bundle reproduced all fourteen
files / 127,187 bytes exactly. No Android/server hybrid session was exercised.

The subsequent [held-session send integration](held-hybrid-send.md) routes the
normal text/encoded send adapter through native encryption for correctly owned
held version-4 records. Fresh PQ query/adoption and incoming bootstrap remain
guarded; the older statement that no coordinator path is connected is superseded
for this held-session case only.
