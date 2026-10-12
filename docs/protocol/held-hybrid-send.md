# Held hybrid sessions in the shared send path

2026-10-12. The normal shared `signal::direct::prepare_payload` path now uses
native version-4 encryption for an already-held canonical version-4 record.
Text `prepare` delegates to this path, as do encoded payloads, media/control
preparation and pairwise group distribution callers. Fresh PQ directory
adoption and incoming KEM bootstrap remain guarded and unfinished.

## Evidence, behavior and account binding

The [native wire evidence](signal-wire.md) establishes 0x44 before MAC and
paired KEM fields. The [session evidence](hybrid-session.md) establishes the
root/chain, encryption, pending-state and authenticated-clear behavior.
No new APK conclusion is inferred from this integration change.

Previously the shared backend send path requested version 4 but emitted 0x43,
and its outer prekey constructor dropped pending KEM fields. The regression
first failed on actual output byte 67 instead of expected byte 68. The shared
path now calls the staged native encoder for held version 4; classic creation
and version-3 sends retain their existing path.

Before any hybrid preparation, the adapter requires:

- an explicit saved peer pin equal to the record's remote identity;
- record local identity and registration ID equal to the supplied saved account;
- a consistent account private/public identity pair;
- a consistent saved sending-ratchet private/public pair.

A missing pin refuses. A changed peer pin returns the existing typed
IdentityChanged for the selected recipient. Account or ratchet mismatch refuses
without generating a candidate. These ownership/integrity checks are local
preservation guards, not a newly claimed native rejection policy.

Existing fetched-bundle identity and signed-key verification occur before this
branch, even when the held record could encrypt. An invalid signature cannot be
ignored merely because a version-4 record exists. Existing recipient/namespace,
identity policy, query reservations and store OCC remain the coordinator's
responsibility; this change does not infer fresh key authority from a held record.

The wrapper accepts an already-parsed record internally, avoiding a second
serialized-record parse or reconstructing a current-only record. Staged
preparation keeps the source unchanged and returns the existing PreparedDirect
candidate with native ciphertext, kind, updated secret record and same remote
identity. Native pending KEM metadata and ordinary IDs persist for unacknowledged
prekey sends. Existing counter leasing stays active.

## Coordinator and failure tests

The encrypted exchange test now sends Bob's reply through ordinary text
`direct::prepare` with a valid fetched same-identity signed bundle, then sends
Alice's next encoded protobuf through `prepare_payload` without a new bundle.
Both outputs use 0x44 and the other peer decrypts them. This tests the normal
shared path rather than calling only the low-level hybrid helper.

Negative cases exercise missing/different pin, another local identity, another
registration ID, a corrupted account private key, a corrupted ratchet private
key and an invalid fetched signature. Source bytes remain unchanged. These run
with and without an ordinary one-time key in the held hybrid exchange.

The all-feature coordinator branch creates a temporary encrypted account store,
installs the synthetic held hybrid record and pin, queues one normal text intent,
and passes a correlated classic same-identity directory reply to the existing
outgoing Pending::prepare. Its prepared stanza carries 0x44 at both prekey and
nested layers, retains the KEM ciphertext and decrypts at the other fixture peer.
The existing transaction commits the candidate with one attempt, the original
client request ID, one wire ID and DeliveryUnknown before any socket write.
The saved account rows stay equal. Reopening retains the candidate/wire ID and
refuses automatically replaying the uncertain in-flight intent.

That directory fixture is intentionally **classic metadata**: the public
consumable parser still refuses selected pqkey replies before this send path.
The test does not prove that a normal outgoing query containing PQ metadata can
reach this branch, nor does it lift the existing fresh PQ bundle guard. The
branch is useful for correctly owned held records and already-admitted metadata;
full authenticated PQ directory/coordinator integration remains required.

## Scope and remaining work

There is no new public command or API signature: existing text/encoded send
preparation now handles held version 4. The low-level staged helper remains
available. No new account key, request ID, wire format, dependency, server wait,
retry policy or live network operation is introduced by this change.

Native KEM inventory/upload, directory authority/adoption, changed-key archival,
incoming key lookup/consumption, companion/group fresh bootstrap and Android/
server interoperability remain unfinished or unverified. Existing selected-PQ
admission and receive guards remain active. The primitive's independent-audit
limitation is still documented in [the KEM review](rust-kem.md).

Validation passed 977 workspace/all-feature tests, the crypto-only shared-send
exchange, strict all-target Clippy, warnings-denied Rustdoc, 26 default CLI/API
symbol tests, experimental CLI build, formatting and local link/artifact hygiene.
The coordinator branch runs only with transport/Noise enabled; it performs no
socket I/O. No live held-session or fresh PQ directory operation was exercised.
