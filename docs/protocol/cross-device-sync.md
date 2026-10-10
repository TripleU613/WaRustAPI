# Cross-device synchronization: comparison evidence

Status: **not implemented end to end**. Trusted-contact notifications are not
app-state synchronization. Receiving now admits an own-device peer route only
with an existing pinned device identity/session. That route now imports new key
IDs atomically with the inbox/ratchet, without replacing existing keys.
Device-session establishment, key activation and snapshot/patch application
remain incomplete.

All references are static evidence from **2.26.37.70**, SHA-256
`b2090b0207e9b97449a5b48e98eb4d1649ae8e5e738e9b7431d6242717fee732`,
not the pinned 2.26.36.75 profile.

## Confirmed structures and call paths

- `LX/C1B.<clinit>` `003c..0072`: ProtocolMessage types 5/6/7 are history-sync
  notification, app-state key-share and app-state key-request. Type 4 is an
  ephemeral-setting sync response, not a user reply or full account sync.
- `LX/BXE.dynamicMethod` `01c4..01d2`: type field 2, ephemeral expiration 4,
  setting timestamp 5, history notification 6, key-share 7, key-request 8.
- `LX/D7M.AD8` `001c..0054` builds type 6 and its key-share body. `CF7`
  `0008..004a` decodes it into `LX/BjO`, initially setting `A01=false`.
- `LX/BL6.dynamicMethod` `0036..0056`: repeated field 1 contains `LX/BPH`
  key records. `LX/BjO.A0s` `003e..0104` reads key ID (`BM0`), key data
  (`BRp`), fingerprint (`BSp`) and timestamp; missing required structures fail.
- `LX/C5H.A00` `0008..004a`: fingerprints require raw-ID/current-index presence
  and a nonempty device-index list. Opaque key bytes alone are insufficient.
- `LX/Cz4.BnT` `001c..0054` checks `BjO.A01` and `0oK.A0A`, then invokes
  `0oK.A02`, `BjO.A0s` and `0oK.A09`. Further tracing resolves this as the
  **outgoing** share path: `0oK.A0C` `007c..008a` sets `A01` from its caller's
  boolean before queuing `SendPeerMessageJob`. It is not an incoming trust flag.
- The actual incoming path is `LX/1gz.A00`: `015a..0172` requires category
  `peer`; `017c..01ba` requires a sender DeviceJid and checks its UserJid through
  the account's `08Z.BO2`; `01be..0216` recognizes the key-share, decodes it and
  stores through `0oK.A09`. Own-device authorization must precede import.
- `0oK.A09` uses transactional `INSERT OR IGNORE` for device-ID/epoch/key data,
  timestamp and fingerprint (`0034..00ce`). It does not blindly overwrite an
  existing key. `Chq.<init>(VII)` constructs a six-byte key ID: two-byte device
  ID followed by four-byte epoch, both big endian.
- `0oK.A0B` `0002..007a` checks the fingerprint raw ID against account state,
  unions indexes allocated since the fingerprint's current index, then requires
  exact registered-index-set equality. `A01` builds that set from companion
  records plus this device's own index. Removed devices cause a mismatch.
- `CI5.<init>` `0010..009a` derives 160 bytes with label `WhatsApp Mutation Keys`
  and splits five consecutive 32-byte keys: index, value encryption, value MAC,
  snapshot MAC, patch MAC. `1Ng.A00/A01` and `1Nh.A00` establish HKDF-SHA256,
  zero salt, previous-block/info/counter expansion.
- `Cz6.A01` authenticates `operation || key-ID || IV+ciphertext || length`, using
  HMAC-SHA512 truncated to its first 32 bytes. Length is the eight-byte big-endian
  length of operation+key-ID (`AvT.A00`); six-byte IDs give length 7. `B1e.<clinit>`
  with `B1f.<clinit>` maps SET/REMOVE to MAC bytes 1/2, not protobuf enum 0/1.
- `Cz6.AKC` `0074..00e8` verifies value MAC before CBC decryption, then checks
  HMAC-SHA256 of the exact UTF-8 index. `Cz6.ADk` authenticates state hash,
  big-endian version and collection name with the snapshot key. `ADf` authenticates
  snapshot MAC, ordered value MACs, version and collection with the patch key;
  `BkO.run` `0ac0..0b70` confirms snapshot/patch argument roles and ordering.
- `C6C.A00` requires a 128-byte current hash (`0000..0018`), expands each input
  with HKDF to 128 bytes (`004a..0054`), and reads/writes little-endian 16-bit
  words with wrapping addition or subtraction (`005e..00dc`). `0rN.<clinit>`
  supplies label `WhatsApp Patch Integrity`; `BkO.run` `09e6..0a1c` subtracts
  old contributions before adding the new set. This is distinct from mutation
  encryption-key derivation and from the snapshot MAC.
- Snapshot `BTa.dynamicMethod` fields are version=1, repeated `BSR` records=2,
  MAC=3, key ID=4. `BSR` wraps index/value/key-ID messages at fields 1/2/3;
  DEX field definitions identify these as `BMW/BMX/BMV`, each with byte field 1.
  Version wrapper `BMY` has uint64 field 1. These are snapshot protobufs, not
  the external-media reference used to obtain their bytes.
- Patch `BVq.dynamicMethod` `003c..00b2` maps version=1, repeated `BQ6`
  mutations=2, external mutations=3, snapshot MAC=4, patch MAC=5, key ID=6,
  exit code=7, device index=8 and client debug data=9. `BQ6.dynamicMethod`
  `0028..004a` maps operation=1 and `BSR` record=2; the `BkO.run` write at
  `08b2..08b6` corroborates the record type. `Cuo.isInRange` delegates to
  `B1f.forNumber`; `B1f.<clinit>` assigns SET=0/REMOVE=1.

## Integrity primitives and receipt findings

`signal::app_state_crypto::MutationKeys` implements derivation and constant-time
index/value/snapshot/patch MAC verification. Test vectors were generated separately
using Python's standard-library HMAC implementation and explicit RFC5869 loops,
with synthetic input key bytes 0..31. Tests alter operation, key ID, ciphertext,
index serialization, hash, version, collection, MAC length and mutation order.
Derived key storage is zeroized and Debug is redacted. The API does not load or
activate stored keys or apply patches; valid MACs
alone do not establish account/device authorization.

`signal::app_state_hash::StateHash` now calculates the rolling hash candidate with
the observed HKDF label and little-endian modulo-65536 arithmetic. Independent
Python HMAC/integer vectors cover one contribution, addition, subtraction and
wraparound; inverse/replacement/order tests also pass. The source state is never
mutated. This primitive cannot decide which stored old MAC a REMOVE/replacement
must subtract: collection/index tracking and snapshot-MAC validation must still
surround it before committing any state.

`decrypt_mutation` follows `Cz6.AKC`: value-MAC verification precedes AES-256-CBC
decryption and PKCS padding validation. `BTv.dynamicMethod` maps index/value/padding/
version to fields 1/2/3/4. `ClC.A03` requires index and version; a SET needs an action
with timestamp (`BX8.A00` presence bit, timestamp field 1), while REMOVE can omit it.
`ClC.A02` validates the JSON index array. The local decoder accepts nonempty arrays
of nonempty strings, verifies the exact original index MAC and exposes only then
a redacted, zeroizing mutation object. Action bytes remain opaque; action-specific
schema validation and collection-version ordering are not replaced by this check.
An independent Python/OpenSSL encrypted SET/REMOVE fixture tests decryption; every
byte of the SET is individually tampered in negative tests. Authenticated malformed
protobuf/action/index fixtures also fail without releasing a decoded mutation.

`signal::app_state_collection::CollectionState` now composes these primitives into
a side-effect-free candidate verifier: patch MAC first, individual mutation and
index verification, old-MAC removal/new-MAC insertion, rolling hash calculation,
then snapshot MAC. It rejects duplicate indexes and retains wire order for patch
MACs/actions. A key resolver must supply account/device-authorized keys; the API
does not automatically activate imported keys. Only a fully verified candidate
and its authenticated action payloads are returned; the source remains unchanged.
Independent SET/REMOVE/replacement vectors and a valid-patch-MAC/invalid-snapshot
case exercise the combined path. Strict next-version and missing-REMOVE refusal
are conservative local restrictions, not claims that all APK recovery branches
are implemented. Action handlers and the live collection request/response path
remain unfinished; transactional staging is described below.

`CollectionSnapshot::decode` now parses that snapshot schema with local byte and
record limits checked before repeated-message allocation. Required fields and
fixed-width IDs/MACs must be present; ambiguous repeated singleton fields fail.
`CollectionState::verify_snapshot` authenticates/decrypts every SET record, rejects
duplicate indexes, computes its rolling hash and verifies the snapshot MAC before
returning a candidate. Local anti-rollback rules reject older versions and a
different state at the same version; identical verified snapshots are idempotent.
A literal protobuf fixture containing the independent encrypted SET/MAC vector
tests bootstrap, duplicates, missing fields, wrong collection, key refusal and
version conflicts. This does not fetch external snapshots, validate their freshness,
apply action payloads, or independently authorize a device.

`app_state_collection::storage` stages patches and snapshots against the saved
baseline inside a store write transaction. It saves version/hash/index-MAC state
and a separate encrypted, version-keyed pending-action batch together. Verification
occurs under the same serialized writer, so a detached stale candidate cannot
overwrite a newer baseline. Sequential patches refuse replay; an authenticated
identical snapshot is a no-op. Older snapshots cannot reset state. Pending actions
retain their original ordering and SET/REMOVE semantics, but are **not applied**
to chats/settings yet. Callers must propagate transaction errors, authorize keys
for the current account/generation and establish snapshot freshness. No live key
activation or sync requests were added. Synthetic tests cover rollback, restart,
stale patches, snapshot idempotence/rollback, and retention of earlier batches.

`CollectionPatch::decode` now accepts inline patch protobufs using those fields,
sharing snapshot record wrappers. A bounded pre-scan rejects excessive records,
duplicate top-level singleton fields, and currently unsupported external-mutation
or exit-code branches before allocation/verification. Such branches are not silently
treated as empty/partial patches. Missing enum uses protobuf SET default; unknown
explicit values fail. Literal wire fixtures feed independent SET/REMOVE crypto
vectors through collection verification; truncation and unsupported-branch tests
pass. Decoding does not authorize keys, trust device-index/debug metadata, fetch
external data, or issue a live collection request.

Peer receipts need a subtype-aware path. `Cns.A04` uses `DbQ.Aow` only for classes
implementing that interface; otherwise its non-sender branch leaves the receipt
type absent. DEX class definitions show `BjO` (key-share) extends `Bik` without
`DbQ`, while `BjP/BjR/BjS` implement it and return `peer_msg`. Consequently,
`category=peer` alone does **not** justify adding `type=peer_msg` to a receipt.
The sender flag and full device-route construction still need tracing before
enabling peer acknowledgements. No speculative receipt was transmitted.

## Implemented foundation

`signal::app_state::KeyShare` decodes the APK-derived structure, bounds resources,
requires key IDs/data/fingerprint structure, rejects duplicate IDs, redacts Debug
and zeroizes key bytes. Its fingerprint comparison implements the observed
raw-ID/index-set check without an unbounded integer loop. An independently
written protobuf fixture covers decoding, new indexes, removed devices and
wrong-account IDs. Redacted payload inspection reports key counts only.

Decoding has **no persistence or networking side effects**. Neither successful
decoding nor a matching fingerprint authenticates the sender. The crate-private
import step is wired only after successful Signal decryption on the pinned
own-device peer route. It rechecks the saved account's own identity at commit,
then inserts previously unseen IDs into encrypted storage in the inbox/ratchet
transaction. Existing IDs are left unchanged, matching `0oK.A09`. The incoming
`1gz.A00` path stores before a fingerprint-use decision; we no longer conflate
storage with activation. There is no public key-export or activation API.
The initial peer route uses category `peer`, a non-primary PN DeviceJid matching
the saved account's own JID, and existing pinned identity/session records. It
accepts only `msg`, never TOFU/prekey bootstrapping through this path. Tests reject
foreign accounts, missing own identity, primary/group/LID substitutions and both
message types when the device session is unpinned. Own-LID routing and peer ACKs
remain unimplemented; this is not a claim of live linked-device interoperability.
Real Signal-encrypted fixture tests distinguish a normal contact share (no import)
from a pinned own-device share, reject an account change before commit, and check
restart/deduplication. Storage tests cover rollback and changed duplicate IDs.

## Remaining dependencies, not claimed complete

### Signed ADV indexes and trust boundary

`08l.A02/A03` read `adv_current_key_index` and `adv_raw_id`, defaulting to -1
when absent. `0oK.A01` gathers companion records' `Ci9.A07` plus this device's
own ADV index; `1IW.A0F` `01a8..01be` stores `Ci9.A07` as `adv_key_index`.
These are not necessarily transport JID device numbers. `8oE.A01` separately
decodes each device's `jid`, `key-index` and optional `lid`.

`11Q.A09` reads device add/remove notifications, hashes and optional
`key-index-list` bytes/`ts`; decoding that notification alone is not key activation.
`11T.A07` `0110..014a` verifies a signed list over exact `08F.A0I || details`.
`08F.<clinit>` `0142..014c` fixes that prefix to bytes **06 02**. `BRD` fields
are details=1, accountSignature=2, accountSignatureKey=3. `BUg` details fields are
rawId uint32=1, timestamp uint64=2, currentIndex uint32=3, packed validIndexes
uint32=4 and accountType enum=5. `Cse` delegates to `Aw0`, whose values are
E2EE=0, HOSTED=1, NON_E2EE=2.

`signal::adv::VerifiedIndexList` implements the **already trusted account-key**
branch. It requires a valid signature, expected stanza timestamp and saved time
floor; any bundled key must match the independent trusted key. The APK also has
an ADV-provided different-key branch, but its reconciliation is not implemented
and this verifier does not use it to replace identity. It accepts E2EE only,
bounds allocation and refuses duplicate singleton/index fields, empty indexes
or indexes above the current counter as conservative local policies. It exposes
a fingerprint comparison without exporting key bytes. Synthetic literal protobuf
fixtures signed with the existing Signal implementation test the domain, every
byte's tampering, wrong key/time, malformed signed lists and redacted Debug.
These are adapter tests, not an independent Android signature capture.

This verifier does not fetch a device list, persist authority, reconcile companion
membership, establish a session or activate stored sync keys. Those dependencies
still precede live collection processing; no new identity or ADV state was invented.

`VerifiedIndexList::reconcile_devices` now provides the separate **candidate**
reconciliation step traced to `11T.A08`. `004a..008c` retains incoming primary
devices and companions whose positive ADV index is in the signed list.
`0136..018c` preserves a current map entry allocated above the signed current
counter. `018c..0206` preserves a higher current index for a device also present
in the incoming map when that old index remains signed-valid. Absent old entries
within the covered allocation range are not retained. Input maps stay unchanged.
Tests cover these branches, index/device-number separation, missing/zero/unsigned
entries, foreign accounts and preservation-induced collisions. Local bounds,
duplicate-index refusal and same-domain ownership checks fail closed; raw-account
ID replacement and automatic PN/LID aliasing remain unsupported.

This result is not a permission to trust new device identity keys. Authenticated
map ingestion, identity-key/device-hash checks, atomic authority persistence and
sync-key activation remain to be connected. No device removal or key activation
has been performed against the live account.

### Identity-key hash

`11T.A05` extracts each `B29.A00 -> B2D.A01` raw public-key byte array, sorts
using `DMq`, then updates SHA-256 with every array in order. `B2D.<init>` stores
raw key bytes separately from the type byte. `DMq.compare` `001c..0028` masks
both compared Java bytes with `0xff`, establishing unsigned lexicographic order.
The repository's DEX formatter now decodes the missing 22s literal instructions;
its regression test checks the unsigned mask and signed-literal representation.

`ReconciledDevices::identity_hash` computes the full 32-byte digest through a
required pinned-key resolver, refusing missing identities and fewer than two
devices. An independent Python/hashlib vector intentionally includes bytes above
0x7f to catch signed-byte sorting; changed/missing-key tests pass. It does not
deduplicate equal keys or hash JID order. `11T.A00/A01` truncate this digest using
runtime integer flag `0x136`; the effective value is not established here and is
not guessed. This is an identity-key hash primitive, **not yet verification of the
notification's `device_hash`**, nor proof of authorization for new identities.
Its caller bindings, truncation and atomic trusted-state update remain unfinished.

### Notification device-list hash (separate from identity-key hash)

Comparison `AgU.run` `0136..0158` passes notification PN/LID hashes to
`0bN.A0I`. That method rejects an empty hash, obtains `A0B`'s device set and
compares `Avv.A03`'s string exactly. `A0B` includes the primary device along with
stored companions. This path does **not** call the identity-key digest above.

`Avv.A03` selects two formats with runtime boolean flag `0x415b`: false hashes
lexicographically sorted `user.0:device@server` strings and prefixes `2:`; true
uses `DeviceJid.getRawStringWithNoAgent` (`user:device@server`, including `:0`)
and prefixes `3:`. Each hashes concatenated strings with SHA-256 (`AvM.A14`),
takes the first six bytes and uses standard Base64 with NO_WRAP (`AvM.A12`,
Android flag 2). No separator is added between strings. Neither ADV indexes nor
identity keys enter this digest.

`ReconciledDevices::device_hash` implements both explicit versions, includes the
primary once and refuses cross-account/domain maps. Independent Python vectors
cover PN/LID, numeric-versus-lexicographic order (devices 2/10), missing-primary
insertion, changed membership and standard Base64 `/`. No runtime flag value is
guessed. Hash agreement is a consistency check, not device identity authorization;
notification ingestion, signed-list binding and trusted-state persistence remain
unfinished. No live device state was modified.

Caller boundary: `AgU.A01` `0022..0036` tests `08Z.BO2` and routes the own-user
branch to `0176..019a` (`Add self device`, `invalid-device-notification`), rather
than its foreign-user signed-list persistence path. The latter merges saved and
added maps (`00d6..00fa`), reconciles (`0144..015c`), then calls `0bN.A0H` and
`11T.A0C`. Do not use this add-notification handler as evidence for own-device
authority establishment; the separate account-sync `17V` / `Af0` path still
needs tracing before activating our stored app-state keys.

### Own-account notification path

Comparison `17V.A09` handles the first child named `devices`. At `01e4..0234`,
a non-`modify`, non-offline notification passes `A02`, parses `8oE.A01`, and
queues `Af0` case 0 then case 1. `modify` or presence of the parent `offline`
attribute instead requests account-device refresh through `1Tc.A01`
(`023c..0270`); this is not permission to discard saved state.

`17V.A02` requires at least one `device` child. Its comparison is now decoded
explicitly: `cmp-long v0,v1,v4` at `006a`, where v1 is saved
`adv_timestamp_sec` (default -1) and v4 is `key-index-list.ts` (0 if the child
is absent). It returns false when saved > incoming, true otherwise. Equal
timestamps pass this predicate. This is a notification freshness predicate,
**not signature verification or authority establishment**. The DEX formatter's
new literal/operand regression prevents reversing this comparison by inference.

`Af0` case 1 (`02b4..0382`) converts parsed `A35.A00` indexes into maps and calls
`0jV.A0Y(map,true,true)`. `0jV.A0Y` normalizes addresses using `A04`, compares
saved companions with incoming indexes, and selects absent or index-changed
entries for removal (`0214..02d2`). It skips our own device and handles unknown
companions separately (`02fa..04d4`), including `unknown_companion` and pending
linking state. It also explicitly guards PN writes after a self-store LID
migration (`0396..03e0`). These branches must not be replaced with the foreign
contact `11T.A08` merge algorithm or automatic trust of incoming identities.

The refresh chain for devices is `1Tc.A01` `05cc..061e` ->
`101.A03([08Z.CMp()],2)`. Next integration work must trace that query's response
and the authoritative ADV baseline update, including how linked identities are
established. `17V/Af0` alone provides neither. No sync key was activated and no
companion/session state was changed during this analysis.

Further query trace: `101.A03` reserves/deduplicates users through `Avz` before
`DMV.run` case 11 creates `SyncDevicesJob` (`11ba..1200`). The job's `A0G`
calls `0ss.A09` at `0046..004c` and waits on `Aw7.get`. Continue at that producer
and its response handler; job scheduling is not the actual wire request.
The research-tool change passed all 17 xtask tests, strict xtask Clippy,
formatting and repository hygiene. These checks validate the formatter, not live
cross-device interoperability.

### Refresh response versus primary ADV ownership

`0ss.A09` uses `0xh.A0W` (`DEVICE_REQUEST`, enum value 0x30), `0xo.A0H`, and
`A06(...,true,true)` to create the device-only sync operation. In the response
processing path, `8oj.A00` `0b94..0c60` selects the self-account map, runs
`11T.A08` on signed indexes, packages the candidate in `9tN`, then invokes
`11T.A04` with the response's ADV timestamp. This is a distinct path from the
notification-only handler above.

`11T.A04` compares the envelope timestamp with the signed timestamp at
`005e..006a`, applies additional age/raw-ID checks, and only calls `0bN.A0E`
for both self and `08Z.BMx()` at `024c..0278`. `0bN.A0E` asserts that mode and
persists `adv_expected_timestamp_sec_in_companion_mode` along with raw ID and
timestamp. This is **companion-mode** baseline replacement, not a primary
client bootstrap recipe. Raw-ID mismatch also has reset branches that this
client must not invoke to get past missing state.

Primary-side baseline construction is local: `11W.A05` creates a random raw ID
(zero mapped to 1) and current index 1. `DLc.run` `01b2..026c` advances existing
counters using companion membership, handles overflow, or initializes missing
state before continuing into `Clt.A01`. `11W.A06` updates the timestamp after
the related update succeeds. Separately, `11J.BFD` `153e..1570` persists raw ID,
current index and timestamp during **companion registration completion**. These
must remain different flows; we have neither linked a companion nor copied a
companion's ADV baseline into this primary account.

Implementation consequence: the signed-list/candidate/hash utilities already
present are not sufficient to activate own-account sync keys. Primary linking,
allocation and acknowledgement handling remain necessary, and linked-device
interop needs genuine authorized counterpart state. No random baseline was
created in the live store merely to satisfy the fingerprint validator.

### Primary pairing completion and identity provenance

The primary QR pairing path is now traced further. This is evidence for future
implementation, not an enabled linking command or authorization of a new device.

- `Clt.A01` constructs device identity and signed index-list material, then
  schedules `DMB` case 1 (`0484..04b8`). `DMB.run` puts `ref`, `pub-key`,
  `device-identity` and `key-index-list` (with `ts`) into `pair-device`
  (`021a..0280`, `06c6..06e0`). Optional client-properties, encryption metadata
  and challenge branches also exist; their runtime flags are not assumed.
  The request registers `DCw` as callback through `0h8.A0O`, with 32,000 ms at
  `06f6..0710`. That request timeout is not a registration cooldown.
- `DCw.C8h` accepts a `type=result` containing `device`/`jid`, parses optional
  `companion-props`, and constructs `Ci9` (`0018..0148`). It records the received
  device in `0jV.A02` (`014c..0196`), not an active trusted device/session.
  A result may instead carry `retry-ts`, `retry-after-challenges`, or
  `retry-with-method`. Those branches are not success and must not be worked
  around by inventing challenge responses or repeatedly replaying pairing.
- The success callback schedules `DMl` case 45 (`DCw.C8h` `0452..0468`). Its
  `0084..0130` branch requires the saved pairing object `CJm` and its `B29`
  identity (`CJm.A00`), then builds `CYl(CJm,Ci9,B29)`. A positive prepared
  `BUg.timestamp` is committed through `11W.A06` (`0100..0116`). This settles
  the primary ADV timestamp from the pairing operation, rather than adopting
  arbitrary incoming key-share state.
- `DKc.run` case 0 (`1234..1312`) puts that `CYl` in `0jV.A00`, sets pending-login
  flag `A0X`, and starts an account-sync timer. The delay comes from flag `0x3d10`
  when positive, otherwise 60 seconds (`1294..12cc`). `0jV.A0Q` handles its
  timeout using `account_sync_timeout`. A server-assigned JID is therefore an
  intermediate milestone, not completion of linking.
- In the device-list update path, `0jV.A0Y` matches a new entry to the pending
  device after alias normalization (`0416..044a`). With its relevant boolean
  argument enabled, it calls `A0V(CYl)` (`04ca..04d4`). Unknown self devices
  take a separate `unknown_companion` branch (`047a..04c8`); they are not
  automatically treated as this pending authorized companion.
- `0jV.A0V` handles duplicate registration, cancels the timer and schedules
  `DMU` case 0. That worker persists companion metadata with `1IW.A0F`, including
  its observed LID conversion branch (`DMU.run` `2c2a..2cd2`), then schedules
  `DMk` case 6. `DMk.run` `22de..235c` takes the identity from **saved
  `CYl.A02`**, asserts the own-device address through `08Z.BL8`, calls
  `0jc.A0K` to save it, clears the pending-login flag, then starts the next
  bootstrap worker (`DMU` case 1). The notification itself does not supply
  authority to trust a replacement Signal identity.

`0jc.A0K` writes identities transactionally (`00b4..0180`) and has handling for
identity changes; this is not permission for our client to silently replace a
pinned identity. Local implementation must bind any new companion to the actual
operator-authorized pairing material and preserve existing identity-conflict
guards. Session establishment, full QR input validation, notification correlation
and bootstrap completion still need implementation/tracing. The next bootstrap
path is `DMU` case 1 at `3336` onward, through `0kv` and `Cyn`; do not restart
this investigation at the signed-list verifier or treat import as activation.
`DMU` `33c2..33de` explicitly refuses critical-data upload in companion mode;
`33de..349e` reserves the primary bootstrap, sets a separately configured timeout,
and `34a0..34b6` dispatches `D1x` case 34. Improved `const-wide/32` decoding
confirms the fallback at `344a` is `0xea60` (60,000 ms); `0kv.<clinit>` sets the
upper cap to 300 seconds. This is separate from the pending-login/request timers.
Follow that observer dispatch and `Cyn` completion before declaring synchronization
ready. Device assignment, device registration, identity persistence and critical
bootstrap completion are separate milestones.

### Critical bootstrap completion and active-key selection

`D1x.COO` case 34 (`05b8..05e0`) schedules `BkO` case 2 through `Cz4`/`0oA`.
That worker prepares a pairing-bound bootstrap ID via `0oP.A07(CYl.A02,
CJm.A05)` (`0e7a..0e92`), queues peer preparation, then shares existing keys
(`0f02..0f2a`, `10c0..10fe`). With no keys, generation is conditional on flag
`0x6b88` (`0f2a..0f4e`); its effective value has not been established. Critical
app-state mutations are gathered from handlers selected through `AwK.A01`
(`0fde..108e`). This is not merely downloading a snapshot.

`0kv` tracks history and app-state outcomes independently in `0kw.A02/A01`.
`08t` distinguishes initial=1, running=3, failed=4 and succeeded=5. `0kw.A01`
finishes when either component fails or both succeed; `0kv.A00` `0012..0034`
reports success only if both succeeded. `A01` receives history completion and
`A02` receives app-state completion. Duplicate/late completions cannot overwrite
an already-terminal component (`0kv.A02` `000a..0022`). `Cyn.onSuccess` clears
the pending pairing and removes its observer (`01a2..0208`); `Cyn.BnX` records
failure and takes separate timeout/app-state/history failure paths. None of
these callbacks alone is a key-share trust check.

`0oK.A04` selects exactly the candidate returned by `0oL.A02`: SQL orders by
**epoch descending, device ID ascending**, limit 1 (`0oL.A02` `0028..0038`). It
does not sort by timestamp or scan for any older matching fingerprint. The
candidate must pass an age check and `A0B`'s registered-device fingerprint check.
Arithmetic decoding confirms `now - candidate.timestamp` at `0oK.A04` `004a`,
then strict-greater-than rejection against the age limit (`004c..0054`). Flag
`0x3898` supplies days (`0nB.A02`), converted through duration helpers; its
effective value remains unknown.

The read-only `app_state::select_active_key` primitive now implements that
candidate ordering, age and fingerprint check against explicitly supplied
authorized account state. `0oK.A09` `0082..0090` reads epoch with `1Lz.A01` and
sign-extends its Java int for the SQL binding: ordering uses signed i32 epoch,
not unsigned byte order. Device IDs remain unsigned 16-bit (`Chq.A00`). Tests
include the high-bit epoch case, ties, exact age boundary, missing/removed device
indexes, raw-ID mismatch, corruption, restart and unchanged persisted records.
An ineligible newest candidate yields no key; it never selects an older fallback.
Future-dated timestamps are additionally refused as a documented local clock
safety guard. Derivation requires the observed 32-byte generated-key length.
Selection reads bounded 256-row pages within the same transaction, instead of
refusing when unrelated checkpoint history reaches 10,000 rows. A 10,600-row
fixture checks selection across pages, uncommitted imports, restart and no older
fallback when the final candidate expires. Physical database ordering is not
mistaken for APK epoch ordering. Write-transaction pagination tests also cover
blinded keys, exclusive cursors and rollback; selection still writes no state.

This is not wired into live sync: callers still need independently authorized
account-device state and the resolved age policy. Neither request JSON nor the
share's own fingerprint may supply those facts. The selector does not generate
keys, write an activation marker, authorize historical incoming mutation keys,
or bypass collection MAC verification.

If lookup fails, `BkO.run` `013e..0168` attempts `0oK.A05` before reporting
missing active key. Generation refuses while `A0A` detects a pending active-key
share, allocates an epoch, creates 32 random key bytes, and binds the fingerprint
to saved raw ID/current index/registered indexes (`A05` `0014..00b6`). With no
companions it imports locally (`00d4..0108`); otherwise it queues shares to them
(`010a..0154`). This is observed primary behavior, not authorization to initialize
ADV state or keys in the live account to bypass missing linking prerequisites.
Generation/share completion and authorized collection transport remain unfinished.

Follow-up tracing resolves the generation/share boundary more precisely:

- `0oL.A01` `0028..005a` reads SQL `MAX(epoch)` as long then converts to Java
  int. `0oK.A05` `0014..004a` takes the signed maximum of that value and the
  manager's in-memory epoch. Only a zero maximum uses `SecureRandom.nextInt(65536)`;
  the selected value is incremented by one. The manager epoch is updated before
  sharing (`00a8..00ac`). These are allocation rules, not permission to manufacture
  missing account raw IDs or registered-device indexes.
- With no companions, `A05` imports and returns the generated key (`00ec..010a`).
  With companions, it invokes `A0C(..., true)` for each eligible saved device and
  returns **null** (`014c..015a`), not the generated key as immediately usable.
- `A0C` records the share through `0ob.A02` before queuing `SendPeerMessageJob`
  (`007c..00c4`). A negative insertion result returns false without scheduling.
  The `BjO.A01` flag marks an active-key share; the separate inherited `Bik.A01`
  flag records acknowledgement. They must not be conflated.
- `SendPeerMessageJob.A0G` awaits the `0h8.A0B` future (`05b2..05e0`), then sets
  `Bik.A01`, updates `peer_messages.acked` by row ID (`05e0..066a`), and dispatches
  `D1x` case 19 (`066a..067c`). `D1x.COO` `04a0..04b8` calls observer `0oO.BnT`.
  This is the job's acknowledged-send path, not proof of a recipient UI action;
  exact wire ACK correlation still requires transport tracing.
- `Cz4.BnT` `001c..0054` imports an active share only after `0oK.A0A` returns
  false. `A0A` `0048..005c` returns true for an unacknowledged `Bik.A01=false`
  share whose `BjO.A01=true`. Thus queueing alone must not activate a key, and
  one completed share must not skip other pending active shares.

The Rust selector does not implement this outgoing share lifecycle yet. No live
keys were generated, shares queued, or local acknowledgement states fabricated
during this trace.

The acknowledgement callback chain is now traced further:

- `0h8.A0B` registers an `Aw7` future in `0hn.A01` before sending
  (`0012..002e`, `0052..0058`). `0hn.A01` stores it in a map keyed by `CZH`,
  with a `D2L` wrapper (case 0), not a map indexed only by message ID.
- Incoming `1Qy.A02` `0018..00a6` constructs that key from `from`, `participant`,
  `recipient`, `id`, `class`, `type` and `edit`. Its normal dispatch reaches
  `1Dz.A1E` → `1Dy.BaF` → `0y8.A08` → handler `0yG` case 6. The handler removes
  the matching map entry and invokes its success callback (`0010..006c`).
  `D2L.BjR` case 0 forwards a null completion value (`006e..007e`); completion
  of that future is not itself a decoded peer-message receipt.
- `CZH.equals` compares class, destination, ID, participant, recipient, type,
  edit and other optional key fields (`0026..0156`). `0h8.A02` normalizes only
  device-zero addresses to their user JIDs for destination/participant; it does
  not erase nonzero companion-device identity.

The existing opened-receipt queue supports only direct primary routes. Its ACK
handler now refuses participant/recipient/edit attributes, in addition to its
existing class, ID, type and destination checks. Regression tests keep the
reservation unsettled after each mismatched route. A general peer-send key and
its error/recovery branches are still not implemented; this trace must not be
treated as authorization to acknowledge every peer message or activate keys.

The reader changes passed 18 xtask tests, including literal DEX operands and
truncation coverage for wide constants/two-address arithmetic, and strict xtask
Clippy. They validate the research tool, not live bootstrap interoperability.

### Action projections: pin/unpin and mute/unmute

Mute evidence from the same comparison APK:

- `AwJ.<clinit>` `002c..0042` maps `A0t` to `mute`; `28A.<init>`
  `0008..0026`, `003e..005c` constructs SET schema version 2 with the two-element
  index `["mute", chatJid]`.
- `BX8.dynamicMethod` `007a..0084`, descriptor at `046c`, places mute in field 4.
  `BTb.dynamicMethod` `0036..0060` maps muted=1 (bool), mute end=2 (int64),
  auto-muted=3 (bool), everyone-mention end=4 (int64).
- `289.A0E` requires the action timestamp, mute message and explicit muted bit
  (`0068..009e`). It retains the optional everyone-mention timestamp
  (`00a8..00ca`). The ordinary end is retained only when muted, present and
  nonnegative or -1; otherwise it becomes zero (`00e6..0154`).
- `289.A0U` keeps -1 unchanged; other mute-end values pass through `08B.A06`
  before settings application (`0012..0064`). Everyone-mention application has
  a separate policy check (`0064..0088`) and clock/expiry conversion. Unmuting
  an actively muted chat is logged and applied (`0120..020c`), not automatically
  rejected merely because its existing timer is still active.

The encrypted `ChatMute` projection follows the decoded value rules, preserving
server-domain timestamps and optional mention/auto-muted metadata. Mixed pin/mute
batches, their application cursor and queue removal commit atomically. Literal
protobuf tests cover true/false presence, finite/indefinite/missing/invalid-negative
end times, optional fields and rejection of ambiguous/unknown fields. Mixed-batch
tests cover restart, idempotence and rollback in both directions when a later pin
or mute conflicts. The existing conservative timestamp conflict rule is local,
not a claim to implement all APK conflict resolution.

`08B.A06` `0000..0016` resolves the clock conversion exactly as
`server_deadline + System.currentTimeMillis() - 08B.A00()`. `08B.A00` uses
elapsed-realtime plus an available server offset, with other saved-offset/local
clock fallbacks; it is not unconditionally the wall clock. `1a9.A01` `0000..0030`
also confirms -1 as indefinite and strict future-time comparison for active mute.

`ChatMute::resolve` now produces explicit `Unmuted`, `Indefinite` or local
`Until(milliseconds)` values from contemporaneous caller-supplied local/server
clocks. Everyone-mention resolution additionally requires an explicit policy
boolean. Disabled policy leaves that setting unchanged; a missing mention timer
also leaves it unchanged for a muted action, but clears it for an unmute action,
matching `289.A0U` `00c6..0120`. An explicit mention timer is independent of the
ordinary muted flag. Invalid clock samples and unrepresentable local deadlines
fail as conservative local guards rather than wrapping integer arithmetic.
Tests cover both directions of skew, exact expiry, finite/indefinite settings,
missing mention values, policy disabling, independent unmute and overflow. Raw
authenticated timestamps are never rewritten by resolution.

This is authenticated-state projection and timer resolution, not live sync or
notification suppression. Runtime clock observation and policy provisioning,
local actions/alias reconciliation and snapshot application remain unfinished.

Comparison `BX8.dynamicMethod` puts timestamp at field 1 and pin action at field
5; `BN2.dynamicMethod` puts its optional `pinned` boolean at field 1. `AwJ.<clinit>`
`0044..0058` identifies `pin_v1`. `B0D.<init>` constructs the two-element
index from that name and the chat JID, uses SET, and passes version 5 through
`Bga/AwI` (`AwI.A03`, consumed by `0rN.A00`). `AzZ.A0E` `12b4..12fe` requires
the pinned presence bit before constructing the incoming pin action.

The pending-action applier now projects that supported SET action into encrypted
chat pin metadata, exposed by `chat_pin`. It requires the exact supported index,
schema, timestamp and boolean presence; additional action fields fail rather than
being silently discarded. Pin false is an unpin SET, not a guessed REMOVE handler.
Projection writes, batch removal and a separate applied-version cursor commit in
one transaction. Each batch records its prior integrity version to prevent skipping
pending predecessors. Unsupported batches remain pending, including snapshots
until absent-action reconciliation is implemented. Older/conflicting timestamps
are conservatively refused; general APK conflict resolution is not claimed.
Tests exercise pin/unpin, missing/unknown fields, ordering, rollback after a write,
reopen/idempotence and retention of unsupported batches. This is a local projection,
not a live sync result, server-side setting change or public-client UI integration.
Old experimental batches without predecessor metadata are refused, not silently
migrated. No private/live store was changed.

### Retained authenticated records for reconciliation

Comparison `0oQ.A04` selects `pending_mutations` when its boolean argument is
true (`0026..002e`) and `syncd_mutations` otherwise (`004a..004e`). Both include
the original mutation index/value/version, not just projected chat settings.
`B09.A07` loads saved pin actions through `AvR.A0i` before reconciliation;
`B09.A01` also reads pending pin mutations. The APK's pending mutations must not
be conflated with our incoming verified-batch application queue.

Collection staging now retains current authenticated index/value/schema records
in encrypted storage, separately from that application queue and integrity state.
SET replaces a record, REMOVE removes it, and a verified snapshot replaces the
record set. All three stores commit in the same transaction. Removing an applied
queue entry therefore need not discard the action needed for later reconciliation.
This is not a complete copy of the APK's database schema or dependency metadata.

An older store without retained records refuses incremental patches. A fully
authenticated snapshot matching its current integrity state can rebuild only the
missing record cache, without recreating pending actions or changing its version,
applied cursor or projected settings. Existing records are not silently repaired.
Tests cover authenticated SET/REMOVE, restart, stale patches, snapshot replacement,
rollback and same-version cache recovery; a tampered snapshot cannot repair it.

Snapshot **projection** remains unsupported. `B09.A07` merges saved/supplied
actions through `B09.A00`, orders them, applies a runtime pin limit, and considers
PN/LID aliases and archive state (`B09.A08`). Simply clearing pins absent from an
incoming batch would not implement that behavior. Full reconciliation and live
authorized sync ingestion remain unfinished; no live account was changed here.

### Further integration

Update 2026-09-18: contact device-list ingestion is implemented in
`transport::devices` from 2.26.37.71 evidence (see messaging-transport.md):
`devices` notifications, the `usync devices version=2` query, signed-list
verification with the pinned primary identity and `11T.A08` reconciliation.
Items 1–5 below still describe own-device authorization, linked-device sessions
and key activation, which remain open.

1. Wire own-device sender authorization from the observed incoming path; decoding
   a key-share is not authorization to replace keys.
2. Implement linked-device sessions and validated device-list changes without
   replacing primary identity or resetting uncertain outcomes.
3. Complete account-device fingerprint validation before key use and implement
   authenticated key-request handling; import storage/restart tests now exist.
4. Trace snapshot/patch ordering, derivation and integrity checks before applying
   state changes. Integrate verified changes into the journal/view-once state.
5. Validate with actual authorized linked-device state; no fabricated sync keys.

Unknown notifications remain encrypted captures. Authenticated unsupported
protocol payloads remain encrypted inbox records with opaque event references,
not key-bearing journal copies. No speculative key requests or device linking
have been added. See the [completion checklist](../messaging-completion.md).


### Message-history notice identification (2026-10-07)

**Confirmed, static APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
`LX/BZc.dynamicMethod` contains `messageHistoryNotice_` at `0422..0426`.
The encoded protobuf-lite descriptor, decoded with `apk-proto-fields`, assigns
it Message field **102**. The string-array slot (`0x55`) is not the wire field
number. The dex field table types `BZc.messageHistoryNotice_` as `LX/BV6;`.
`BV6.dynamicMethod` names contextInfo (`0036..003a`), messageHistoryMetadata
(`003e..0042`) and botHistoryShareSyncMetadata (`0048..004c`); its descriptor
at `0052..0056` assigns fields **1, 2, 3** respectively. This is separate from
Message.messageHistoryBundle (field 70), not another name for that envelope.

Reproduce from the local APK with the repository's existing research CLI:

```sh
target/debug/xtask apk-xref ~/Downloads/WhatsApp.apk --anchor messageHistoryNotice_
target/debug/xtask apk-proto-fields ~/Downloads/WhatsApp.apk --class 'LX/BZc;'
target/debug/xtask apk-proto-fields ~/Downloads/WhatsApp.apk --class 'LX/BV6;'
```

**Inferred**: the previously observed 49-byte `[102]` payload belongs to this
message type, based on the observed wire field and the current APK descriptor.
Size alone says nothing about its contents or availability of history. The
redacted `lab-inbox --inspect` summary now labels this envelope and lists its
nested field numbers. It retains unknown field numbers, refuses truncated nested
protobuf and prints no metadata values. Inspection does not import history,
request media, change sessions, adopt keys, or authorize an own-device sender.
Literal protobuf tests distinguish this notice from other control messages and
verify redaction, malformed input and retention of unknown field numbers.

**Unknown / negative result**: runtime dispatch and authorization for this notice
have not been traced. Its metadata field is typed `LX/BX5;`, but the current
`apk-proto-fields` heuristic refuses that class's mixed repeated/singular
schema (`no descriptor string decoded cleanly; not guessing`). A private static
dump of `BX5.dynamicMethod` has been retained; no inferred metadata field map has
been encoded in the client. Full notice behavior remains unfinished. Raw dex
and private captures remain outside Git.

**Confirmed, local inspection defect and correction**: while adding this label,
`transport_receive::run(inspect=true)` was found to print poll questions and
option strings despite its redaction claim. It now prints question byte length,
option count/byte lengths, selectable count and secret-presence only. A fixture
containing private text, Unicode and a nonzero secret verifies neither text nor
key values enter the summary. This changes diagnostic output, not poll messages
or stored plaintext. No historical terminal-log cleanup is claimed.

**Observed, saved-account offline inspection (2026-10-07)**: the rebuilt release
CLI completed `--profile l lab-inbox --inspect` with exit 0. Exactly one saved
payload was labelled: 49 bytes, top-level fields `[102]`, nested notice fields
`[1, 2]`. This corroborates a context-plus-history-metadata notice shape without
exposing either value. The full inspection output was retained under private
`lab/accounts/l/`; nothing was sent and captures were not replayed. Opening the
store still runs its normal metadata/table initialization transaction, so this
is not a claim that the database file stayed byte-identical. No notice was
applied and no incoming-message session was advanced.

### History metadata reader correction (2026-10-07)

**Confirmed, local tooling correction with APK 2.26.37.71 evidence**: the
`BX5` refusal above came from our fixed-triple descriptor reader, not absent
metadata in the APK. `BX5.dynamicMethod` at `0066..006a` encodes repeated strings
without a presence index, interleaved with presence-bearing int64 fields.
The corrected `apk-proto-fields` fully validates this mixed descriptor and prints
historyReceivers=1, oldestMessageTimestampInWindow=2, messageCount=3,
nonHistoryReceivers=4, oldestMessageTimestampInBundle=5. The strings in fields
1/4 do not establish how receivers are authorized. Our earlier claim that all
fields carry an incrementing third index is refuted; see
[the corrected reader and runtime evidence](apk-analysis.md#reading-protobuf-field-numbers-apk-proto-fields).
The original refusal is retained above as historical evidence, superseded by
this correction. No server request or private account change was needed.

### Group-history notice dispatch (2.26.37.71)

**Confirmed, static APK** (same hash above): string anchors for
`FMessageGroupHistoryNoticeProtobuf/parseProtobufMessage` identify `LX/DA8.CF4`.
It checks Message field-102 presence (`0008..0020`), requires a group chat
(`0020..003c`) and normally requires app data `group_history` (`004c..0076`).
Two internal boolean branches can skip that app-data check (`003c..004c`);
their producers/meaning remain **unknown**, so they are not compatibility
switches in our client. The metadata must exist (`0092..00ac`), have at least
one history receiver (`00ac..00bc`), contain a strictly positive oldest-window
timestamp (`00bc..00d8`) and a strictly positive message count (`00d8..00ec`).
The APK emits distinct refusals for each invalid case at `023c..028a`.

`DA8.CF4` reads feature flag `0x3d6a` (`0120..0134`). Its enabled branch creates
`9NL` with type `0x76` (118) and copies metadata through `9ku.A00`
(`0134..0170`), retaining optional bot-share metadata (`0170..0184`). Its other
branch makes the existing fallback message with code `0x2718` (10008)
(`022a..023c`). Whether that flag is enabled for this account is **unknown**.
The handler compares history receivers with own PN/LID identities and the
message sender (`0188..022a`); this affects its notice-marking path, not a proof
that a receiver string independently authorizes downloading history.

`LX/9ku.A00` copies count, oldest-window and optional oldest-bundle values
without unit conversion (`0004..0076`), then receiver lists (`0076..0182`).
Timestamp units and subsequent rendered text remain **unknown** at this trace
boundary. `LX/AVZ.Cir` validates app data `group_history`: it accepts `9NL`, the
`1il` fallback with code 10008, or `1ik`; other message classes are rejected
(`0006..0082`). `DA8.AD7` also serializes this notice into field 102, rather than
encoding it as a ProtocolMessage (`0030..0114`).

**Inferred**: this envelope describes shared group history as a conversation
notice; it is not itself an app-state key share or an instruction to treat its
49-byte body as actual history messages. Dispatch, rendering, feature selection
and receipt policy in TeleWhatsApp remain unfinished. It remains a preserved
unsupported journal reference, with redacted structural inspection available;
no text was invented and no history/media request was sent.

Reproduce the handler anchors/dumps with `apk-xref` and `apk-smali`, for example:

```sh
target/debug/xtask apk-xref ~/Downloads/WhatsApp.apk --anchor 'FMessageGroupHistoryNoticeProtobuf/parseProtobufMessage does not have metadata'
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/DA8;' --method CF4
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/AVZ;' --method Cir
```

These scoped raw dumps were retained privately. They establish parsing branches,
not execution of those branches by the saved account or completion of group
history transfer.

### Supported metadata decoding and diagnostics (2026-10-07)

**Confirmed, local implementation with APK evidence**:
`transport::history_notice::decode` decodes the Message field-102 envelope and
validates the traced `BX5` metadata. `DA8.CF4`'s required positive timestamp/count
and nonempty receivers are enforced. Optional oldest-bundle values are retained
raw, including negative values: `9ku.A00` copies that optional field, and no
additional validity or timestamp-unit conversion has been established here.

**Confirmed, static receiver conversion**: `LX/8kZ.A1L` (`0000..0020`) reads a
receiver string and calls `0D8.A01`; `LX/0D8.A01` (`0000..002a`) parses it through
`0D1.A00`, requires a `UserJid`, and throws otherwise. Our supported subset is
primary numeric PN/LID. Hosted, legacy and other identity forms are not silently
classified as invalid APK inputs; they remain unsupported **locally**. The
receiver parser's broader concrete subclass behavior remains **unknown**.
No full receiver address is returned by the metadata model or diagnostic errors.

The decoder refuses duplicate singleton fields, wrong wire types and unknown
metadata/envelope fields instead of accepting prost's merged/ignored values.
These are conservative local inspection guards, not claims that the APK rejects
all such protobuf encodings. It also applies local 65,536-byte and 1,024-entry
per-list bounds. Context and bot bodies remain opaque: only their presence is
returned, with no claim of validation or authorization.

`lab-inbox --inspect` now prints `metadata=validated` and receiver/non-receiver
counts plus optional-body presence flags, or `metadata=unsupported` with a fixed
redacted reason. It does not print receiver strings, count values, timestamps,
context or bot contents. Unknown notice fields remain structurally visible in
the summary. A validation failure does not destroy the encrypted inbox record,
request media or advance a session. The public pure decoder returns `None` for
another message type and a local error for a present unsupported notice.

Literal APK-schema fixtures exercise required presence, zero/negative values,
optional raw timestamps, both receiver lists, opaque bodies, duplicate fields,
unknown fields/wire types, every truncated prefix, redaction and local bounds.
The notice is still a preserved unsupported journal reference. Runtime dispatch,
feature selection, rendering, actual history ingestion and receipt policy remain
unfinished; no stored notice is promoted to a user reply or sync-completion event.

The inspector uses the full-envelope decoder, not only the nested metadata
helper. A mixed text/notice or duplicate notice cannot acquire a validated
label merely because its last nested metadata happens to be valid. The redacted
summary regression includes that mixed-envelope case.

**Observed, saved-account offline inspection (2026-10-07)**: the rebuilt release
CLI completed `--profile l lab-inbox --inspect` with exit 0. The one saved
49-byte field-102 notice returned `metadata=validated`, receivers=1,
non_receivers=0, oldest_bundle_present=false, context_present=true and
bot_metadata_present=false. No receiver, count or timestamp value was printed.
The complete output remains in ignored, mode-0600 private lab storage. Captures
were not replayed, no request or receipt was sent, no history was applied and
no message session was advanced. Normal store-open metadata initialization
still prevents claiming byte-identical database files. This confirms supported
metadata in the already-stored notice, not runtime display or history transfer.

### Ordinary notice dispatch and historical group backfill (2026-10-07)

**Confirmed, static APK 2.26.37.71**: `Cp5.A0B` selects the message's `meta`
child (`0000..000e`) and reads its `appdata` attribute (`0012..0022`). It
separately recognizes category `peer`, `member_tag`, and `medianotify` branches;
these are not guessed history-notice bypasses. `DEm.CF0` calls `Ayp.A0H` to
select `meta`, reads `appdata`, and constructs `DFN` (`0000..0028`). `DFN`'s
string representation names this value `IncomingAppDataValues(metaAppData=…)`.
`DEZ.AR2` serializes appdata from `1in.ATi` (`0942..0958`). Together with
`DA8.CF4`'s group-history check, this supports the normal meta/app-data gate.
The exact downstream DFN-to-CoT.A0P setter bridge remains **untraced**; the
producer relationship there remains an **inference**, not another confirmed
method offset. No appdata value is invented on receive.

**Confirmed, local implementation**: `decode_for_stanza` supports an ordinary
group `message` with exactly one `meta` child and one `appdata=group_history`
attribute. A direct chat, category-peer context, missing/ambiguous meta, appdata
placed on `enc` or the outer message, or unknown appdata remains unsupported.
Its caller must already authenticate/store the plaintext. The untraced APK
internal app-data bypasses are not enabled, and no APK UI feature value is
fabricated.

The atomic receive journal now emits `SyncEvent::GroupHistoryNotice` for that
supported context, with diagnostic kind `group_history_notice`. It retains the
original message key, address-free validated metadata, ordinary stanza timestamp
and local observation time. This is distinct from a `NewMessage` human reply;
it does not create history messages, request media or emit a sync-completion
signal. There is no corresponding outgoing MessageContent form; deserializing
this received event as outgoing content fails. Automatic read eligibility still
excludes field 102. Feature-controlled rendering, timestamp units and history
transfer remain unfinished. Rebuilt clients understand the new event variant;
older binaries do not know it.

**Confirmed, local backfill defect and correction**: `inbox_events::restore`
used only `session_sender`, which intentionally refuses participant-addressed
group/broadcast stanzas. It therefore silently skipped those authenticated
historical captures. It now checks the original sender-key chat route first,
then the direct route, and verifies `received:<chat>:<id>` against the original
capture. Consumed/cleared records remain excluded. Existing typed markers stay
idempotent; old unsupported placeholders upgrade in one marker/event transaction.
No capture or marker deletion is used to replay an event.

Fixtures cover normal/incorrect meta placement, direct/peer rejection, typed
control serialization, no automatic read, group/broadcast original-route backfill,
wrong-route refusal, rollback before commit, reopen and once-only upgrade.
The group rollback fixture verifies stored identity/session records remain
unchanged. These tests prove local projection; live incoming notice processing
and official rendering are not claimed.

**Observed, saved-account offline projection (2026-10-07)**: with the rebuilt
release, a private copy of the existing database upgraded exactly one event.
Its second `reproject` returned zero; complete before/after `events --summary`
batches showed only one added `group_history_notice` and no other kind change.
The same bounded canonical backfill then upgraded one event, followed by zero
on repeat, with the same sole kind delta. The original encrypted database copy
and all canonical/copy summaries and reproject logs remain in private
mode-0600 lab storage under a mode-0700 directory. No capture or event marker was
deleted, no consumer batch was acknowledged, and no network request/receipt was
sent. The prior unsupported journal event is retained as history; the projection
marker now points at its typed control-event upgrade. This demonstrates durable
recognition of the previously received notice, not a new human reply, APK UI
rendering, or completed history transfer. The receive/backfill code does not
write Signal identity/session/prekey/account-generation/send-state tables;
normal store-open metadata initialization still changes physical file bytes.

### Contact ADV verification and retention (2026-10-07)

**Confirmed, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The new hash-pinned core topics preserve eleven exact selected methods, including
the actual contact-sync caller. This build names the signed/details protobuf
classes **Grm/GsF**; older BRD/BUg references above belong to the earlier build
and must not be copied as 37.71 class names.

- [11T.A07](apk/2.26.37.71-core/adv-contact-signature.txt)
  loads the primary Signal identity at `0014..0030`, constructs the optional
  supplied account-signature key at `0034..005e`, and selects it when different
  (`00bc..0110`). Thus “a saved identity always selects local” is refuted.
  Signature verification is over prefix/details (`0110..014a`). On success
  `0192..01b4` returns parsed details and a nonnull replacement key only for the
  supplied-key branch; it does not itself persist trust. Missing keys or bad
  signature yield NAc(null,null,true); malformed protobuf yields null. Neither
  supplies a valid candidate. No human safety-number verification is established.
- [Lck.A00](apk/2.26.37.71-core/adv-contact-sync-updater.txt)
  is one of the **two exact A07 callers** found across the APK; the other is
  ODU.A01. `099a..09f2` compares saved and incoming envelope timestamps and
  diverts a strictly older result before signature verification or device-map
  application. A missing signed list with companion entries is refused
  (`0a50..0aa8`); primary-only/missing-map cases have a separate unsigned path
  (`0aaa..0b68`). “No signed list always means the same primary fallback” is
  not a complete APK rule.
- In Lck `0b6c..0b98`, a null verifier result or null parsed details jumps to
  `0cc8..0cd8`, records a failed outcome, and rejoins the contact-updater
  result path. It does **not** construct/apply the signed candidate. Valid
  details select the self/current contact map, merge via 11T.A08
  (`0bf6..0c0e`) and create the baseline/candidate callback (`0c0e..0c8a`).
  The device-only contact branch reaches that merge through `0cbe..0cc8`.
  This caller evidence matters: A08 alone does not define failure retention.
- [11T.A04](apk/2.26.37.71-core/adv-contact-validation.txt)
  returns false on null details or envelope/signed timestamp mismatch
  (`0036..00dc`). Its wall-clock age threshold uses integer flag **0x2da**,
  clamped to 1..35 days (`00ec..012a`); older multi-index candidates return
  false before applying callbacks (`012c..018c`). The effective runtime flag
  and signed-long division overflow/negative-time parity are **unknown**;
  the exporter still shows undecoded `0xbe` division instructions. Our saved
  timestamp floor is not a substitute for this complete age policy.
- A04 raw-ID mismatch (`0192..0214`, `028a..029e`) has reset/cleanup branches;
  these are distinct from stale/malformed/signature failure. The contact
  cleanup helper is traced in the later
  [identity follow-up section](#adv-identity-follow-up-and-raw-id-cleanup-2026-10-07);
  complete downstream session/group effects remain **untraced**. No saved
  account reset is authorized by this finding. A valid candidate eventually
  reaches 0bN.A0H (`029e..02c4`); the self/companion path separately updates
  its ADV baseline and device map (`024c..027c`).
- Lck invokes [11T.A0C](apk/2.26.37.71-core/adv-contact-identity-apply.txt)
  after successful candidate validation, conditionally on feature **0x3285**
  (`0cd8..0cf2`). Its runtime value remains **unknown**. The later
  [identity follow-up trace](#adv-identity-follow-up-and-raw-id-cleanup-2026-10-07)
  establishes that enabled skips this immediate call; it does not establish a
  general deferred successful-application path. A0C first removes the primary session through
  [0jc.A0s](apk/2.26.37.71-core/adv-primary-session-removal.txt), then calls
  [0jc.A14](apk/2.26.37.71-core/adv-primary-identity-save.txt) to save the
  supplied key. A14 refuses saving one's companion identity as primary and
  delegates to [0jc.A0K](apk/2.26.37.71-core/adv-primary-identity-commit.txt).
  A0K writes primary identity bytes through 0kc.A0H (`00f2..0108`) and commits
  its own transaction (`0164..0180`). A0s commits its own session-removal
  transaction earlier. **No transaction spanning both is established** by this
  sequence. Alias target selection/0kc internals and downstream cleanup/events
  remain untraced; these methods do not prove normal Rust receive parity.
- [ODU.A01](apk/2.26.37.71-core/adv-contact-add-notification.txt)
  separately handles notification addition. Missing/bad ADV material follows
  its invalid-input path (`003e..00dc`), including scheduling 11S.A01 on the
  explicit verifier-failure flag. Validated data merges current/add maps and
  applies through 0bN.A0H before A0C (`00dc..0170`).
  [11S.A01](apk/2.26.37.71-core/adv-contact-session-cleanup-schedule.txt)
  schedules DNy discriminator 0x28. The later
  [identity follow-up trace](#adv-identity-follow-up-and-raw-id-cleanup-2026-10-07)
  resolves this as an identity-directory read, not immediate session cleanup.
  This is not proof of automatic identity replacement on signature failure.

**Observed local implementation discrepancy / refuted claim:** devices.rs and
its regression comment claimed stale/raw-ID failure kept the saved list. Actual
`ingest_user` replaces the device vector with primary-only, preserves raw ID and
saved list_timestamp, marks companions unverified, and clears matching pending
work as successfully ingested. The test asserted metadata, not retained devices.
This chunk corrects those comments, **not the runtime behavior**. In particular,
stale/signature/malformed supplied-list retention and failed-query accounting
remain unfinished; last verified companion data can currently be lost. Missing
signed material, unpinned identity, invalid supplied material and raw-ID mismatch
must be treated as separate cases when implementing the APK caller behavior.

**Unknown / next implementation boundary:** the current strict ADV verifier
refuses a different bundled key; full supplied-key candidate validation, feature
selection, identity/session archival, device authority binding to a changed pin,
and atomic Rust application still need integration. Existing incoming-message
identity changes authenticate the message; that is not proof that an old cached
ADV list was signed by the new pin. No real account, key, session, pending action,
request ID or live request was changed by this static investigation.

Validation: exact APK re-export/check verified all 38 files/967529 bytes;
checksum-verified cached diff preserved all 22 older topic texts. CLI reference
regeneration, four CLI OpenAPI coverage checks, API symbol check, formatting and
staged whitespace passed. Hygiene verified 659 local links and 2642000 bytes
across all four independent APK bundles. This source/comment-only chunk makes
no runtime behavior change; no new workspace-test or live-interop claim.


### Rejected device candidate retention (2026-10-07)

**Confirmed substrate, APK 37.71:** Lck.A00 `099a..09f2` diverts older
stanza timestamps before application; `0b6c..0cd8` does not construct/apply
signed candidates for a missing/failed verification result. `0a50..0aa8`
refuses unsigned companion entries, separately from the unsigned primary-only
path. The eleven selected core methods and preceding caller analysis establish
these distinctions. `11T.A07` selects/verifies the key; A08 reconciles only a
valid candidate. A04 raw-ID mismatch has separate reset branches.

**Implemented / observed offline:** `ingest_user` now refuses a failed candidate
without replacing the cached device vector, verified timestamp/raw ID, trust
flags, notification hash, fetched_at or any other saved-list metadata. It leaves
the matching pending revision intact. Rejected supplied signed material never
becomes a primary-only successful replacement. No cache is invented when none
existed. Non-hosted unsigned companions refuse; unsigned primary-only replies
keep their existing supported path. Missing pinned key is explicit because the
strict verifier cannot establish supplied-key authority; that limitation is ours.

`DeviceVerificationFailure` has fixed reasons: conflicting_user_rows, unsigned_companions,
invalid_signed_list_shape, missing_timestamp, missing_pinned_identity,
stale_timestamp, invalid_signed_list, raw_id_mismatch, reconciliation_refused.
Invalid signed list combines signature/protobuf and conservative verifier guards,
including a different bundled key; it does not diagnose the server's internals.
Raw-ID mismatch remains an explicit **local unsupported-replacement policy**:
no account or contact reset is inferred or performed from the APK reset branch.
Wall-clock age flag/parity and full signed-long time behavior remain unfinished.

A query persists nullable `verification_failures` reason/count maps, counting
unique rejected users even with duplicate rows. Historical and unsettled rows
are null; a settled result with none is an empty map. `IngestSummary` exposes
`verification_errors` separately from actual server `partial_errors`.
`accepted=true` / server_acknowledged still means an IQ result, even when local
verification failed. `code` remains a recognized server refusal code or null;
local reasons never manufacture a server code. `require_complete` refuses local
verification failure before group/status message reservation, even with a cached
list. Background partial success still saves other users, and the per-connection
cursor defers rejected users despite newer notifications. A later distinct valid
read can settle the retained work without reviving a refused/old reservation.

`device-query --id` exports nullable verification_errors plus the fixed
verification_failures count map offline, without private generations, addresses,
keys, timestamps, supplied query ID or raw bodies. The send guard directs the
operator to this existing inspector. No new command, flag or HTTP route exists.

**Observed regression scope:** a genuine synthetic signed baseline with a
companion survives nine refused reads (seven failure categories, including
malformed/tampered/foreign-key signed material) byte-for-byte with its saved pin
and session marker. Duplicate rows count one rejection. New notifications do not
retry the same refused user during that connection. All refusal records and
pending work survive encrypted-store reopen; a later valid signed read adds a
companion and clears pending while leaving identity/session unchanged. Another
mixed reply saves an unsigned-primary success beside a missing-pin refusal,
then advances only the successful user's new revision. CLI tests distinguish
null history from fixed reason counts without exporting identifiers.

**Unknown / unfinished:** this is candidate failure retention over the traced
caller substrate, not complete supplied-key identity replacement, age policy,
raw-ID reset, PN/LID authority binding, own-account sync or human verification.
A valid old cached list is historical evidence, not proof it was signed by a
subsequently changed pin. Live failed-signature/stale/refusal cases are not
injected against the server. No canonical account or live request was changed
while implementing these regressions.

Conflicting duplicate-row correction: a valid first row could previously clear
pending before a later refused row. Selection now preflights all requested-user
rows before mutation; exact duplicates collapse, differing rows refuse with fixed
local conflicting_user_rows, preserving cached/pending state in both orders and
across reopen. This is local ambiguity policy; APK duplicate dispatch is unknown.

Diagnosability: a background local-verification failure emits WARN under the
default CLI filter with its matching opaque local reservation ID and count, so
`device-query --id` is usable without guessing an ID. No address/key/raw response
is logged; inspection itself omits the supplied ID. Ordinary success remains INFO.


Observed live 2026-10-07 22:13:51 UTC: zero outbox/queued-action/pending-receipt/
pending-device preflight, one existing-peer read and a 20-second authenticated
receive-only session completed cleanly. Saved CLI query: server_acknowledged,
requested_users=1, ingested=1, partial_errors=0, omitted_users=0,
superseded_users=0, verification_errors=0, verification_failures={}, code=null.
Pending users returned to zero; outbox, 17 actions (error1/ok15/sent1) and all
nine ACKed receipts unchanged. No chat sends or resets. This proves the supported
unsigned-primary live path and new diagnostics schema, not live rejected-list or
supplied-key behavior; the primary-only list still has no signed index evidence.

Validation: 654 workspace/all-feature tests and 24 default CLI tests passed;
strict workspace/all-target Clippy, warning-free Rustdoc, CLI reference/API
coverage and formatting passed. Fifteen actual isolated CLI probes covered all
nine local reasons, zero success, historical/unsettled null, absent, unchanged
repeat and invalid-ID redaction. Conflicting duplicates preserve cache/pending
in either order and after encrypted reopen. Hygiene663 local links and 2642000
bytes across four unchanged APK checksum bundles passed. No live malformed,
signature-refusal or raw-ID replacement claim is made.

### Signed device authority and authenticated refresh (2026-10-07)

**Confirmed substrate, APK 37.71:** selected 11T.A07 `0014..0030` loads the
primary Signal identity and `0110..014a` verifies signed details with its chosen
key. Lck.A00/A04 apply only a validated candidate; A0C and 0jc.A0s/A14/A0K
establish identity/session application as a separate action. The already-ported
incoming nested-message and outgoing signed-directory policies archive their old
pins/sessions and emit IdentityChanged only after authentication. These sources
establish which key verifies membership; they do **not** establish our new local
storage field or heartbeat scheduling as APK wire fields/jobs.

**Observed local gap:** DeviceList retained only a historical verified flag,
raw ID and timestamp. It had no link to the actual verification key. After a
valid primary replacement, the cached companions could still feed group/status
planning under the old flag. Incoming/outgoing authenticated replacement did
not queue a fresh device read, and the receive loop only scanned pending work
at startup, device notification, settlement or timeout. A new pending marker
alone would therefore remain idle in an otherwise active session.

**Implemented local authority invariant:** a successful signed ingestion now
persists SHA-256 of its exact raw 32-byte verifier key in private nullable
`verified_primary_key_fingerprint`, inside the same list transaction. It is not
an APK attribute or a human verification claim. Encoded Signal type byte 05 is
removed consistently before binding/checking. Historical rows lacking this
field remain null; neither inspection nor pin replacement stamps the current
key onto an old list. No old signed bytes are retained/reverified by this change.

`DeviceListAuthority` reports binding_known, primary_pin_present,
primary_pin_matches (nullable) and companions_current. The last is true only
when the saved list was successfully signed-verified and its actual verifier
binding equals the current primary pin in the same PN/LID namespace. A missing
binding or pin remains unknown; a different pin reports false. Raw saved
history, devices, metadata and its original historical verified bit remain
untouched by inspection/key change. This is membership authority, not companion
identity-key trust, full inventory or a human safety-number verification.

`devices` shows these flags in human output and adds nullable `authority` to
JSON beside `list` and `pending_users`. Its JSON preserves the existing explicit
private saved-address output but strips the new private binding fingerprint.
`peer-identity` device-key coverage adds nullable device_authority; old serialized
coverage defaults null. Group/status planning reads each list/pin relationship
together and refuses companion entries with missing/changed authority before
preparing a send. Primary-only planning keeps its prior supported behavior.
The old OpenAPI DeviceList skeleton (jid/integer devices/synced_at) did not match
actual CLI output and is corrected to owner/Rust Jid objects/device entries and
saved metadata; the existing CLI is still offline, with no HTTP handler.

The existing authenticated primary-replacement archive path now queues the
refresh **inside the same transaction** as pin/session/history/event changes.
Transaction abort rolls it back too. It preserves any prior pending notification
hint/revision history; no key, hash or identity is invented. Only supported remote
primary PN/LID addresses qualify; own observed PN/LID and companion-device keys
are excluded. Unauthenticated conflict observation and an untrusted supplied ADV
key cannot call this authenticated archive path. This does not newly authorize
pin replacement or reset any account/contact/session.

Normal receiving checks eligible pending work on its existing heartbeat when
no device query is active. Work queued after startup can thus progress without
a device notification or reconnect. Existing cursor semantics remain: one attempt
per unchanged revision, failed/unknown users deferred for that connection,
reservations durable before writing, no old query replay. This local follow-up
read is not claimed to reproduce the APK's exact feature/job selection.

**Observed offline coverage:** a genuine synthetic signed companion list has
current authority under its verifier key; removing the new field yields unknown
history without mutation. An injected abort after authenticated replacement
leaves the original pin/current authority and no refresh marker. A committed
replacement leaves the old list byte-for-byte, reports changed authority and
retains pending work across encrypted reopen. A fresh valid list signed by the
already replaced pin restores current authority normally, without supplied-key
adoption. Group/status planning refuses unbound and changed cached companions
without modifying their history. The hook excludes a companion and observed own
LID. An injected receive test queues work **after** activation's initial scan,
then picks it up/settles it on heartbeat with no notification or reconnect.

**Observed test-harness correction:** the first heartbeat test queued work while
activation still awaited prekey-upload writing. Startup therefore picked it up,
so the expected same-heartbeat ping never arrived within the short observation
window. The fixture now lets the startup scan finish before queueing new work.
No live session/request was repeated because of this synthetic harness failure.

**Unknown / unfinished:** live primary replacement/companion rebinding, full
supplied-key ADV application, raw-ID replacement, age-flag selection, PN/LID
alias/authority transfer, own-account inventory and full public group fan-out.
Each plan recipient has a consistent local list/pin read; this is not a claim of
whole multi-recipient atomic preparation. Historical companion lists require a
fresh real read, rather than a guessed migration. No canonical pin was rotated
or private draft sent to manufacture a live identity change for these tests.

CLI output correction: devices --json --request previously prepended a human
queue notice to JSON. The notice now appears only in human mode, preserving one
parseable JSON object for the existing flag combination. The queued operation
remains offline and the request/data format is unchanged.


Observed live 2026-10-07 22:33:56 UTC: a zero-send/action/receipt/device-pending preflight,
one existing-peer read and a 20-second authenticated receive-only session
completed cleanly. Saved query: server_acknowledged, requested/ingested=1,
partial_errors/omitted/superseded/verification_errors=0, verification_failures={},
code=null. Pending devices returned to zero; outbox, all 17 actions
(error1/ok15/sent1) and nine ACKed receipts unchanged, no chat sends. Authority:
binding_known=false, primary_pin_present=true, primary_pin_matches=null,
companions_current=false. This accurately reports the unsigned primary-only
reply with no signed index list; it does not prove live companion binding,
identity replacement, own inventory or a supplied-key branch.

Validation: 658 workspace/all-feature tests and 24 default CLI tests passed.
Strict workspace/all-target Clippy, warning-free Rustdoc, CLI reference/API
coverage, formatting and staged whitespace passed. Thirteen actual isolated
CLI JSON/human/request probes covered bound/current, changed, missing pin,
historical unknown, unsigned primary and absent list; no private verifier
fingerprint was exported and --json --request parsed as one object. Fixture
metadata probes test inspection, not cryptographic signing; genuine signed
baseline/replacement tests cover authority separately. The own-primary hook
additionally passed explicit PN/LID/companion exclusions. Hygiene668 local
links and 2642000 bytes across four unchanged APK checksum bundles passed.

## ADV identity follow-up and raw-ID cleanup (2026-10-07)

**Confirmed static evidence**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This resolves previously labelled unknown caller effects; no live identity
change, account reset or new key adoption was performed.

- [DNy.run](apk/2.26.37.71-core/adv-identity-query-worker.txt), discriminator
  **0x28 / 40**, branches to `11b0`. It first attempts missing-LID resolution
  through Cio.A02 (`11c0..1218`), then constructs one `user@jid` per supplied
  entry and `iq type=get xmlns=encrypt to=server`, containing
  **`identity` with those user children** (`1232..12da`). It sends via 0h8.A0T
  with DF6 discriminator **0**, selector **0x99** and timeout **0x7d00 / 32000**
  (`12da..12f2`). This is an identity-only read, not the existing full
  `encrypt/key` prekey-bundle query, and has no immediate pin/session write.
  Missing-LID resolver internals and selector semantics remain **unknown**.
- [DF6.C8f](apk/2.26.37.71-core/adv-identity-reply.txt), discriminator **0**,
  requires `list/user`, parses each **DeviceJid**, and routes a per-user
  `error@code` to 11S.A00 (`003c..00ba`). Otherwise it reads `identity` and
  `type` bodies plus optional `device-identity`; the type body must contain
  exactly **one byte**, otherwise it throws (`00ba..0120`, `0898..08a6`).
  Valid type shape schedules DNk discriminator **7** on 0kY. This trace does
  not establish a full signed-prekey requirement for an identity-only reply.
- [DNk constructor](apk/2.26.37.71-core/adv-identity-save-worker-constructor.txt)
  maps the reply's type bytes, device JID, identity node, optional ADV node
  and DF6 handler into the fields used by
  [DNk.run](apk/2.26.37.71-core/adv-identity-save-worker.txt), case **7** at
  `02aa..03dc`. It opens a 0jp.A05 context for that device, extracts the
  identity bytes and type byte, and branches on `isPrimary` (`02da..0318`).
  Companions must pass **11T.A0D** with source discriminator **5** and false
  final flag (`0318..0354`); failure logs invalid device identity and invokes
  11T.A0B. Primary replies bypass that companion ADV verifier. Success joins
  the one-byte type with the identity body, parses it through B7H.A02,
  invokes **0jc.A14**, then schedules AhK discriminator **7** (`0354..03bc`).
  Invalid keys log failure; the opened context closes. **Unknown:** complete
  A0D/A0B, B7H type/key constraints, AhK continuation, alias addressing and
  downstream session invalidation. Do not treat the one-byte shape check as
  complete cryptographic validation or copy this into automatic Rust adoption.
- [DF6.BmU](apk/2.26.37.71-core/adv-identity-error-reply.txt), case **0**,
  sends the recognized outer error to 11S.A00 for every requested entry
  (`0030..0068`). [11S.A00](apk/2.26.37.71-core/adv-identity-query-error.txt)
  emits a local failure event and schedules DLx discriminator **0x11** only
  for non-primary devices (`0032..0094`). Its cleanup effects remain
  **unknown**. Delivery-failure discriminator 0 in DF6.BjL returns directly;
  no automatic retry is established by this handler.
- Lck.A00's feature **0x3285** test is an `if-nez` skip over immediate
  11T.A0C (`0ce0..0cf2`): **disabled** calls it, **enabled** skips it.
  The key-bearing callback it constructs earlier is ODa discriminator
  **0x15 / 21** (`0c32..0c44`).
  [ODa.run](apk/2.26.37.71-core/adv-candidate-identity-callback.txt), case 21,
  calls the same 11T.A0C with owner and candidate key (`11a2..11c6`).
  [Its constructor topic](apk/2.26.37.71-core/adv-candidate-identity-callback-constructor.txt)
  includes **two overloads with the same DEX shorty**; field layout identifies
  the owner/key roles consumed by this case, but shorty selection alone does
  not uniquely identify a full Java signature. 11T.A04 invokes its runnable
  only on the traced raw-ID mismatch self/companion branch (`01ac..0214`),
  before candidate application. **Refuted inference:** this callback is not
  proof of a general deferred application whenever 0x3285 is enabled.
- [0bN.A0F](apk/2.26.37.71-core/adv-contact-raw-id-cleanup.txt) rejects an
  own-account owner, collects cached device keys **excluding the primary**,
  and calls 1IW.A0D with those companions and the supplied reason
  (`0028..009c`). If companions existed, it also calls JhE.A0L with an empty
  added set and the removed set (`009c..00c4`).
  [1IW.A0D](apk/2.26.37.71-core/adv-contact-companion-removal.txt) independently
  asserts remote owner and **never remove primary**, resolves an owner set,
  and runs database/cache removal plus downstream device updates
  (`0008..01aa`). Non-null reasons can invoke A06 and B7j/0bp cleanup
  helpers; empty removals have a separate reason-dependent cleanup
  (`01c8..0248`). The reason `identity_changed` affects an update flag
  (`0128..013e`). **Unknown:** A06/B7j/0bp/JhE internals, alias completeness,
  full session/group mutation and all transaction boundaries. This is peer
  companion cleanup, not permission to reset our account or rotate identity.

**Implementation gap at this trace:** `peer-keys` uses a full signed
prekey-bundle reply and cannot stand in for this `encrypt/identity` read.
Failed ADV candidates currently remain durable and deferred; Rust does not
perform automatic APK identity-only recovery. The subsequent
[primary query staging](#primary-identity-only-query-2026-10-07) implements
the distinct explicit read without adoption. Trace the remaining identity type,
companion verification and continuation helpers before implementing a distinct
bounded, correlated identity read and authenticated application policy. The
existing strict verifier also still lacks supplied-key candidate application,
raw-ID transition parity and effective age/feature selection. Preserve prior
pins, sessions, device history and unresolved work while closing these gaps.

**Validation:** the exact pinned APK exported 43 topics / 172 selected methods
in 48 files, 1,332,728 bytes. Cached comparison against the preserved previous
core snapshot reported 10 added topics and all 33 previous topics unchanged;
manifest, inventory and tokens remained unchanged. The promoted cache comparison
took 0.041250 seconds. Its topic statuses matched the temporary export; path
fields intentionally differ by output directory, so whole-JSON equality is not
claimed. All cached file checksums were verified by the diff command. The
58 xtask tests and regenerated CLI reference passed. This is source/research
validation, not live recovery or whole-APK semantic equivalence.

## Primary identity-only query (2026-10-07)

**Confirmed static**, APK **2.26.37.71** (same exact SHA-256 as the preceding
trace): [B7H.A02](apk/2.26.37.71-core/adv-public-key-parser.txt)
`0008..003c` requires at least 33 encoded bytes, accepts only type **5**, and
copies the first **32** public bytes after the type. Extra trailing bytes are
ignored. [11T.A02](apk/2.26.37.71-core/adv-typed-public-key.txt) prefixes the
supplied type byte, delegates to that parser and returns null on invalid-key
exception (`0000..003c`). This resolves the earlier key-shape unknown; it is
not evidence of signature validity or human trust.

[11T.A0D](apk/2.26.37.71-core/adv-companion-identity-validation.txt)
confirms that companion identities require ADV processing. It permits an
already-held identical device key on one early branch (`0088..00ec`), treats
in-progress login separately (`00ec..010e`), and otherwise requires ADV and
identity material. It parses Grz/GsB and selects a supplied account-signature
key or existing primary (`0160..01cc`). The account signature covers the
selected account prefix + serialized details + companion public bytes
(`024c..02ca`); the device signature covers its prefix + details + companion
bytes + primary public bytes (`02cc..0308`). **Both** B7H.A09 checks must pass
(`030a..0374`); account/device encryption-type compatibility is checked before
that. Teardown, self/remote, raw-ID and stored-account-type branches follow;
these complete mutation policies remain **unported/unverified**. Do not adopt
companions merely because a `device-identity` node exists.

[AhK.run](apk/2.26.37.71-core/adv-identity-success-event.txt), case **7**,
resolves the device's user, selects event 0MN.A03 and argument 1, then invokes
ATU.A00 (`0dba..0dea`, `0e2e..0e3c`). **Unknown:** complete event consumers,
receive replay behavior and downstream session cleanup. The handler does not
itself prove that saving a returned identity immediately recovers receiving.

**Implemented local staging:** `peer-identity-query --chat <primary-PN-or-LID>`
queues one APK-shaped encrypt/identity/user read using the existing durable
one-shot action actor. Repeats reuse the same ID even after refusal, malformed
reply or uncertain write; no refresh/reset exists. `--show` only inspects the
saved correlated evidence. It validates the IQ ID/type, exactly one matching
requested primary user, unique type/identity children and the APK key-shape
rules. Extra/duplicate/foreign user rows produce a fixed `user_set` failure;
those ambiguity guards are **local invariants**, not claimed APK wire rules.
Byte and UTF-8 text node bodies are both supported because the binary decoder
can represent raw strings as text. Root/per-user refusal codes remain explicit;
raw error text is never emitted and no wait duration is invented.

The redacted result reports queued / outcome_unknown / server_refused /
invalid_reply / primary_identity_received, fixed envelope/user_set/type/key
failure, optional recognized error code, SHA-256 diagnostic fingerprint of the
canonical 33-byte public identity, exact-namespace pin match and optional ADV
node presence. These hashes are not native safety numbers. Missing pin gives
null; PN/LID aliases are not guessed. Saved raw-32-byte pins are normalized
for comparison only. Inspection is a single read transaction and never writes
pins, sessions, device lists, events, verification or chat intents.

**Remaining implementation gap:** this is the distinct normal request and
reproducible inspection needed before wiring automatic ADV-failure recovery.
It does not yet queue automatically on failure, apply returned identities,
verify companions, implement account sync, or reproduce the downstream event
and cleanup policies. The existing full `peer-keys` path remains separate.
Returned primary keys are directory evidence, not independently signed-prekey
proof; no automatic key adoption or new identity-change authority was added.

**Offline validation:** four regressions cover once-only request shape and
uncertain write/refused/malformed reuse, wrong correlation, duplicate/foreign
users, type/length/trailing-byte semantics, Text/Bytes body parity, exact LID
and raw pin normalization, secret-free failure output, inspection immutability
and encrypted-store reopen with prior pin/session/device history preserved.
Synthetic arbitrary pin/session bytes test preservation, not cryptographic
session validity. No live failure recovery is established by these tests.

**Observed live (2026-10-07):** one existing-peer identity-only read was queued
with no eligible chat sends, queued actions, pending receipts or device work.
A bounded 20-second saved-account receive session returned cleanly. The new
one-shot action settled `ok`; inspection reported `primary_identity_received`,
failure/code null, `pin_matches=false`, `device_identity_present=false`.
A separate offline cross-check matched its canonical fingerprint to the already
saved `peer-keys` **verified_bundle** with a valid signed prekey. This agreement
is not a signature on the identity-only reply and does not authorize an alias
transfer or pin replacement. Exact peer pin/session inspection, outbox, receipts
and all prior actions stayed unchanged. Repeating queue returned the original
selector locally without creating another action or session. Private evidence
is indexed by `lab/releases/identity-query-live/active.json`; no raw keys,
addresses or reply bodies are exported. No chat send, key adoption, automatic
ADV-failure recovery or live companion handling was tested.

**Validation:** 662 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target Clippy and warning-free Rustdoc passed. CLI reference and
API symbol/command coverage passed after regeneration. The first workspace run
found a stale reference because it ran before the parallel regeneration;
regeneration followed by a fresh workspace run resolved it. Four newly selected
APK methods expand the core to 47 topics / 176 methods, 52 files / 1,439,603
bytes. Cached diff against the preserved previous core took 0.053777 seconds:
four added topics, all 43 existing topics unchanged, manifest/inventory/tokens
unchanged, targets updated; checksums verified. No whole-APK equivalence claim.

## Primary save callbacks and device sender-key storage (2026-10-07)

**Confirmed static**, APK **2.26.37.71**, exact SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:

- [0kc.A0H](apk/2.26.37.71-core/identity-pin-write-route.txt) selects the
  default C0F.A05 classification unless flag 0x6f43 or 0x6f6f causes A09
  lookup (`0008..0042`). [A0G](apk/2.26.37.71-core/identity-pin-database-write.txt)
  separately gates recipient normalization on 0x79da, then writes identities
  with recipient_id, recipient_type, **device_id**, public_key, timestamp and
  account_encryption_attestation_type (`0008..00e2`). Effective feature values,
  classification meaning and normalization internals remain **unknown**.
  Timestamp field presence is confirmed; the exporter still leaves its division
  opcode undecoded. No exact signed-long parity is claimed.
- 0jc.A0K commits its pin transaction before dispatching later callbacks.
  **Corrected 2026-10-08:** the null-key/old-value-present branch reaches
  [0k4.A0L](apk/2.26.37.71-core/identity-pin-first-event.txt) and D3f discriminator
  0x29 -> DeS.BqD (identity-deleted observer), not first-key notification.
  The historical stable topic name is misleading and retained for cached diffs.
  A nonnull key with no old value instead selects D3f 0x2a -> DeS.BqB.
  [Deletion observer trace and branch evidence](peer-identity.md#sender-key-group-selection-and-deleted-identity-observers-2026-10-08)
  preserve the correction. Changed identity creates D3V discriminator **1**
  (`0236..027e` in the existing primary-identity-commit topic).
  [D3V constructor](apk/2.26.37.71-core/identity-change-callback-constructor.txt)
  retains device/metadata/boolean; [D3V.COL](apk/2.26.37.71-core/identity-change-callback.txt)
  case 1 calls **DeS.BqC** (`0048..0068`). **Negative result:** D3V has no
  run method; it is an observer callback, not a scheduled Runnable. The full
  APK method-name scan found BqC bodies on Fco, AU1, D3k and D4x plus the
  abstract DeS declaration. Other callback names/native code remain outside
  that search; it does not prove there are no other side effects.
- [D4x.BqC](apk/2.26.37.71-core/identity-change-observer.txt) calls
  [D4x.A00](apk/2.26.37.71-core/identity-change-cleanup.txt) with the first
  boolean false. After security-notification handling, the primary remote
  branch invokes **0bN.A0F(reason=identity_changed)** (`0476..04b0`), tying
  changed identity to the previously traced companion cleanup. Pending
  receipt-device rows have additional cleanup (`04b0..0868`). A later branch
  discovers groups with `sent_sender_key=1` for this user (`0af0..0bb4`), then
  rotates/removes selected sender-key records through 0jc.A19 and can reset
  sent-sender-key flags (`0868..0cc8`). Exact own/feature/category branches,
  sender-key removal helper semantics, receipt updates and alias effects
  remain **unported/unknown**. Rust must not indiscriminately delete ACKed
  receipts or reset our account based on this trace.
- The identity-read success continuation is distinct: AhK.run case 7 sets
  discriminator **1** and jumps directly to ATU.A00 at `0e36`, skipping the
  nearby discriminator-16 setup. [ATU.A00](apk/2.26.37.71-core/identity-success-dispatch.txt)
  delegates to 078.A00; [ATU.COL](apk/2.26.37.71-core/identity-success-callback.txt)
  case **1** calls **0Y6.BdR** then **BdO** for the user (`00ce..00ee`).
  The nearby case 16 list callback is not this path. The method-name scan
  found BdO bodies on 1za, 3B5, AL2, AL3, AL4, Kv7, 0Y9, 1We, FWe and Fco,
  plus the abstract 0Y6 declaration. Fco.BdO normalizes to the primary device;
  [Fco.A00](apk/2.26.37.71-core/identity-wait-completion.txt) completes the
  matching user's held future, and [Fco.BqC](apk/2.26.37.71-core/identity-wait-observer.txt)
  reaches the same helper. **Unknown:** complete event registration and wait
  consumers; these dispatches alone do not prove incoming replay/session repair.

**Refuted local assumption / implemented correction:** Rust group sender-key
storage previously stripped the sender's device ID, with a regression test
explicitly requiring companions to share the primary record. That merged
independent authenticated chains. APK
[Cnr](apk/2.26.37.71-core/sender-key-name.txt) constructors preserve a B76
address, raw chat and extra string; A03 emits raw chat, recipient ID, recipient
category, **device ID** and the extra string (`0000..003c`). Equality includes
all three stored components. C9B.A00 selects **A** for primary and **B** for
companion (`0008..0020` in
[the category helper](apk/2.26.37.71-core/sender-key-device-category.txt)); those
additional APK variants remain **unported**, not invented from this observation.

Rust now retains the actual sender device in its private storage selector and
libsignal name for SKDM processing, group decryption, own-key preparation and
encryption. Both group and broadcast callers already preserve their participant
device. Primary spellings remain unchanged; historical flattened rows stay in
place and are **not** silently reassigned or copied to companions. A companion
without its own key now reports unsupported under the existing receive path
instead of borrowing primary history. This can expose a missing SKDM previously
masked by flattening; no historical sender ownership is guessed and no captures,
event markers or ratchets are rewound. PN/LID aliases and full category/actor
variants remain separate unfinished work.

**Offline evidence:** the updated selector regression separates primary,
companion 3 and companion 4 while keeping primary spelling. A new real-crypto
store/reopen regression installs independent SKDMs, decrypts and atomically
advances each chain for both PN and LID namespaces; using a primary key to read
the companion ciphertext fails. Historical primary storage is not used as a
companion fallback. This is device separation, not complete identity-triggered
sender-key rotation, companion inventory, live group parity or actor coverage.

**Next action:** primary directory adoption must include its applicable changed
identity cleanup policy; pin writes alone are not full recovery. Trace 0jc.A19,
category/alias selection, sent-distribution metadata and wait consumers before
wiring automatic recovery. Current identity-only queries remain read-only.

Validation of this correction: 663 workspace/all-feature tests, 24 default CLI
tests, strict workspace/all-target Clippy, warning-free Rustdoc and regenerated
CLI/API coverage passed. The 13 added source topics expanded the core to 60
topics / 197 methods, 65 files / 1,528,287 bytes. Cached comparison took 0.053107
seconds: all 47 prior topics unchanged; manifest/inventory/tokens unchanged;
checksums verified. A rebuilt-client 20-second receive-only session ended
cleanly with peer pins, device inspection, actions, outbox and receipts unchanged,
no added query or chat sends. Private evidence index:
`lab/releases/sender-key-live/active.json`. Live companion decryption and
identity-triggered rotation remain unverified; no primary key was adopted.


## Scoped sender-key removal and recipient translation (2026-10-07)

**Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:

- [0jc.A19](apk/2.26.37.71-core/sender-key-remove.txt) acquires the
  Cnr-scoped lock via 0jp.A01 (`0008..001a`), parses the raw group and translates
  the sender-key name (`001a..0036`). It deletes from `sender_keys` using
  **group_id, sender_account_id, sender_account_type, device_id, bucket_id**
  equality (`0074..0094`), with the tuple supplied by Cnr.A03. Its boolean is
  affected-row-count > 0, not permission to delete all keys for a user. The
  database is closed before F1E(group) cache invalidation through 0ka.A01
  (`009e..00bc`); invalid-group handling returns false (`00d2..00fc`). Cache
  callback internals and transaction coupling to identity replacement remain
  **unknown**.
- [0kX.A00](apk/2.26.37.71-core/sender-key-name-translation.txt) calls
  **0kT.A02(sender, operation, "sender_keys")** and rebuilds Cnr with the
  translated sender while retaining the original raw chat and bucket. A key
  deletion based only on an untranslated input address is therefore incomplete.
- [0kT.A02](apk/2.26.37.71-core/sender-key-recipient-translation.txt)
  attempts translation only when B76.A01 is zero. It parses B76.A04 through
  [0kT.A01](apk/2.26.37.71-core/sender-key-phone-parse.txt), which excludes
  `"0"`/`"Server"` and calls PhoneUserJid.Companion.A03. Predicate 0hd.A02,
  feature **0x536a**, and predicate 0hd.A09 select a specialized branch through
  0DE.A08 and a 0hh type check (`0020..0060`). The alternative uses
  **0Xw.A0C** (`008c..009c`). Both rebuild a device address using the original
  **B76.A00 device ID**, then preserve B76.A02/A03 in B7H.A06. A null result from 0Xw.A0C
  calls the private A00 diagnostic (`00a0..00a6`) and returns the original B76.
  Nonzero account type, absent parsed address and other bypass branches return
  the original directly; the diagnostic is not universal.
  **Updated qualification:** A01 itself returns null for 0hd.A02-matching
  values, so reaching the later specialized branch would require that repeated
  predicate to change; stable-predicate reachability is only inferred.
  [Mapping repository and row-selection trace](peer-identity.md#canonical-key-mapping-and-row-selection-in-rust-2026-10-08)
  establishes stored-row ranking and typed materialization checks. Feature
  values, authority/freshness, full classifier/wrapper semantics and diagnostic
  execution remain unknown. Arbitrary PN/LID substitution is not established.
- [B7H.A06](apk/2.26.37.71-core/sender-key-address-build.txt) derives
  recipient category via B79.A00 and device via DeviceJid.getDevice, and keeps
  the two additional B76 components (`001c..003c`). Translation does **not**
  flatten companions onto the primary device.
- In the already cached [D4x.A00](apk/2.26.37.71-core/identity-change-cleanup.txt),
  `0906..092a` obtains the selected sender through
  [1xi.A0l](apk/2.26.37.71-core/sender-key-account-provider.txt) and
  **08Z.Apw** (spelled `Apw` in bytecode); the alternative at `0ae4..0af0`
  uses `Apz`. [08i.Apw](apk/2.26.37.71-core/sender-key-account-address-first.txt)
  calls 08i.A04 and returns A0G; [08i.Apz](apk/2.26.37.71-core/sender-key-account-address-second.txt)
  calls A09 and returns A00. A method-name scan found these getters on 08i and
  the 08a wrapper plus abstract 08Z. **Inferred:** the selected sender is an
  account-provider address, rather than the changed remote participant itself.
  Getter initialization and full PN/LID account selection are still untraced;
  do not promote that inference to complete ownership proof.

**Refuted implementation shortcut:** neither this deletion nor the previously
traced primary `sent_sender_key=1` group query establishes a blanket deletion
of that remote peer's incoming sender chains. Rust must retain authenticated
incoming history and unrelated device/bucket records. Full APK bucket/actor
selection and outgoing distribution history are needed before automatic rotation.

**Observed source audit:** the comparison group/status send implementation is
`crates/whatsapp-cli/src/transport_group_send.rs` (`transmit`), not an API
`transport/outbox_sender.rs` module (the latter path does not exist). It loads
one own sender-key record, distributes an SKDM to every target, then reserves
its advanced key and send intent atomically with pairwise state. Searching
`crates/` for `sent_sender_key` finds no distribution-history implementation.
Always redistributing an existing key does not rotate it after a peer identity
change. These are implementation gaps, not server requirements. No automatic
cleanup, pin adoption, identity reset, group send or live request was performed
for this trace. The next implementation needs durable group/device distribution
history and authenticated-change invalidation, preserving uncertain intents.


Validation of the scoped-removal trace: 58 xtask tests and CLI reference
regeneration passed. Eight added source topics expand the core to 68 topics /
205 methods, 73 files / 1,544,967 bytes. The checksum-verified cached diff
took 0.067145 seconds: eight added topics and all 60 prior topics unchanged;
manifest, inventory and tokens unchanged. These are static/offline checks;
no live behavior or automatic rotation is claimed.


## Durable own sender-key exposure and rotation (2026-10-07)

**Confirmed static:** APK 2.26.37.71 D4x.A00 selects groups with
`sent_sender_key=1` for a changed primary (`0af0..0bb4`) and calls scoped
0jc.A19 deletion (`0868..0cc8`). The preceding
[scoped removal trace](#scoped-sender-key-removal-and-recipient-translation-2026-10-07)
preserves account/device/bucket separation. Full APK ownership initialization,
feature and bucket selection remain **unknown/unported**.

**Implemented local subset / inferred compatibility:** the supported comparison
sender now records exact own-key recipient-device exposure inside the transaction
that reserves its advanced key, pairwise state and once-only send intent.
Reservation is deliberately conservative: an uncertain write may have exposed
the key even without an ACK. This metadata is not labelled APK sent-flag parity
and does not manufacture historical exposure records.

An already policy-approved authenticated remote primary replacement marks only
indexed own keys exposed to that exact PN/LID namespace as `rotation_pending`,
in the existing archive/pin/session/event transaction. First-seen/equal keys,
companions and our own PN/LID are excluded. Unauthenticated conflict observations
and identity-only directory reads do not invoke this transition. No mapping is
inferred between PN and LID; separately indexed aliases require their own
supported transition. Incoming sender chains, cached receipts, send outcomes
and unknown historical exposure are preserved.

The next **newly reserved** comparison group/status send mints a fresh own key
when pending. Its commit compares both the original sender-key bytes and exposure
snapshot; a concurrent identity change cannot be cleared by a stale preparation.
The prior own key is archived encrypted under the existing request selector,
and recipient history starts anew for the fresh chain. No additional send is
scheduled. Repeating an acknowledged/uncertain/refused request retains the
existing once-only behavior, rather than rotating or retransmitting that intent.
Absent exposure metadata means historical coverage is unknown: old keys are not
blindly rotated based on guessed membership. Normal actor group-send integration,
full bucket variants, companion identity cleanup and retrospective history remain
unfinished.

**CLI/API:** `peer-identity --chat <primary PN> --json` exposes the redacted
`pending_sender_key_rotations` count separately for the phone and each mapped
primary LID. Zero is not proof of complete historical coverage. Old serialized
inspection output defaults this field to zero. Queue state, addresses, key bytes
and distribution recipients are not printed; inspection remains offline and
CLI-only, with no new HTTP route. Library transaction helpers live in
`transport::sender_key_distribution` and the comparison send uses them directly.

**Offline tests:** authenticated archival triggers exact-primary invalidation
while first-seen/equal/companion calls do not; reopen retains pending state and
unrelated LID/group/incoming/uncertain records stay unchanged. Transaction abort
rolls back invalidation, stale reservations fail OCC, and a fresh reservation
resets only its new generation's exposure. Real libsignal encryption confirms
fresh-key ciphertext fails under the old chain and decrypts under the new SKDM;
marking pending alone does not erase the old key. Initial test compilation exposed
incorrect helper/signature names in the harness; corrected to the existing
extract/receive API before these three tests passed. Live group rotation remains
**unverified**; no draft/group send is authorized by this implementation.


Validation: three new focused tests and the final 666 workspace/all-feature
tests passed, alongside 24 default CLI tests, strict workspace/all-target
Clippy, warning-free Rustdoc and CLI reference generation. The first workspace
run hit `store::tests::wrong_key_is_typed` reporting AlreadyOwned on its correct-key
reopen after wrong-key rejection. A focused rerun and the full final rerun passed;
root cause remains unknown, and no lock guard or key policy was relaxed.


**Observed live receive regression:** rebuilt client, 20-second receive-only
window after outbox/action/receipt/device-work preflight. Session ended cleanly;
peer inspection (including pending rotation counts), devices, actions, outbox
and receipts were unchanged. No query or chat intent was added. Private evidence:
`lab/releases/sender-distribution-live/active.json`. This checks existing receive
compatibility only; no identity transition or group rotation occurred live.

## Encoding-independent primary-change follow-ups (2026-10-07)

**Confirmed locally:** authenticated primary identity-change follow-ups compared
normalized saved addresses against the original peer including `JidKind`. A
synthetic AgentDevice/0 primary therefore skipped indexed own sender-key rotation;
the same representation of our own PN queued an unnecessary device refresh. The
expanded pre-fix archive test reported rotation count 0 instead of 1; the own
refresh test found a nonempty pending queue. These are Rust comparison failures,
not server refusals or evidence of an APK retry policy.

**Correction:** `devices::after_primary_identity_change_in` and
`sender_key_distribution::after_primary_change_in` first retain the existing
primary-only user/LID, agent-0, device-0 guard, then normalize encoding for own
exclusions and recipient matching. A companion or agent is never promoted to
primary. `pending_for_peer_in` ignores encoding kind while preserving agent and
device so inspection reflects the same logical primary. PN/LID equivalence is
not inferred. Stored exposure snapshots, sender chains, request IDs, old lists,
refusals and uncertain send reservations remain intact; no query is automatically
adopted and no send is replayed.

**APK context:** build **2.26.37.71 D4x.A00** selects changed-primary groups with
`sent_sender_key=1` at `0af0..0bb4`, with the identity-change device follow-up at
`0476..04b0`; **B7H.A06** preserves account category and device in recipient
address construction. These retained sources support the existing scoped
follow-ups. This change corrects local model equality; APK internal encoding
normalization and the actual occurrence of alternate primary encodings on these
live paths remain **unknown**. Testing all Rust kind variants does not establish
that all variants occur in this APK.

**Offline evidence:** authenticated incoming PN archive and signed-directory LID
archive entry points queue rotation across encoding variants and store reopen;
LID refresh remains scoped to LID, not a guessed PN alias. Inspection accepts
kind variants but rejects companions. Own PN/LID exclusion tests leave complete
checkpoint records unchanged across reopen; foreign agent/companion exclusions
remain covered. Existing transaction rollback and stale exposure-reservation tests
still enforce atomicity. **Unknown live:** a fresh authenticated changed-primary
message with alternate encoding and the next newly authorized group send. No
identity replacement, settled request replay or group send was manufactured.

**Negative comparison audit:** `peer_verification::invalidate_in` addresses
metadata through a formatted JID selector; `Jid::Display` retains
user/server/agent/device but does not serialize kind. The same kind-equality
failure was not established there, so verification storage was not changed.

**Validation:** 673 workspace/all-feature tests, 24 default CLI tests, strict
Clippy, warning-free Rustdoc, CLI reference/API checks and rebuilt CLI passed.
Read-only inspection of the existing private account left outbox/actions/receipts,
devices and pins unchanged. Evidence index remains private at
`lab/releases/primary-kind-inspection/active.json`; no network request or key
replacement occurred during this inspection.

## Own PN/LID sender selection and persisted device index (2026-10-07)

**Confirmed static, APK 2.26.37.71:** cleanup selects our account's sender
address, not the changed remote participant. The prior account-provider ownership
inference can now be narrowed using these retained initialization paths:

- [08i.A04](apk/2.26.37.71-core/sender-key-account-lid-initialize.txt) lazily
  initializes LIDs under a monitor. It reads preference `self_lid` with null
  default (`001e..0036`), parses it, calls A02 and sets its loaded flag
  (`0036..004e`). [08i.A02](apk/2.26.37.71-core/sender-key-account-lid-set.txt)
  stores a LID user in A0H and builds A0G using Apt (`0014..0030`), or retains
  an existing LID device JID and its user (`0032..005a`). Thus the already cached
  **Apw** getter returns our persisted LID device address after this initializer.
  Invalid-type/caught-error behavior does not establish unconditional clearing
  of a previously held A0G; no invalid-address reset is inferred.
- [08i.Apt](apk/2.26.37.71-core/sender-key-account-device-index.txt) reads integer
  `registration_device_id`, default **0** (`0000..001c`). A LID user is converted
  by [0DE.A04](apk/2.26.37.71-core/sender-key-account-lid-device-build.txt) into
  an ordinary 0e1 device except for index **0x63**, which constructs Bli. The
  builder retains the supplied device index. This is not permission to invent
  one or to flatten an existing companion to primary.
- [08i.A09](apk/2.26.37.71-core/sender-key-account-pn-ensure.txt) ensures Me
  loading using **1XY(this, 1)** and 08t.A04/A00. The nonzero discriminator in
  [1XY.call](apk/2.26.37.71-core/sender-key-account-me-load.txt) opens app-private
  file `me`, reads the serialized object and passes a **Me** to 08i.A01
  (`0042..008c`); MeOldFormat converts before the same path (`0094..00b4`).
  **Negative:** searching 1XY for `run` found no body. It is a `call` callback;
  the unrelated discriminator-zero branch must not be mistaken for Me loading.
- [08i.A01](apk/2.26.37.71-core/sender-key-account-me-set.txt) sets A0F to Me,
  parses **Me.jabber_id** as PhoneUserJid (`0000..002a`) and builds **A00** with
  Apt and DeviceJid.Companion.A01 (`002a..0052`). The cached **Apz** getter
  returns this PN device address. Null/invalid Me resets PN fields and invokes
  08m.A00(0); those account-loading side effects are not ported as a Rust reset.
  [0DK.A01](apk/2.26.37.71-core/sender-key-account-device-build.txt) preserves
  the supplied device index and user agent while building the route; index
  0x63 selects `hosted.lid` or `hosted`. Full hosted-account semantics remain
  **unknown**, and existing support is not broadened.
- Me.lid is parsed by [08i.A00](apk/2.26.37.71-core/sender-key-account-lid-parse.txt)
  for the A04 consistency checks (`009a..016e`). Missing/unparseable/mismatched
  Me/prefs produces named diagnostics and queues **Igs discriminator 0x11**.
  It is **not established** as an immediate fallback that overwrites `self_lid`:
  the direct A02 write precedes the consistency diagnostics. Igs reconciliation
  side effects, storage writers and network provenance remain **unknown**.

**Confirmed selection condition:** in retained **D4x.A00 `08fe..092a`**, the
return from **0bA.A0k(group)** is placed in v1 (`0904..0906`); true selects
**Apw/LID**, false branches to **Apz/PN** at `0ae4..0af0`. The resulting device
JID passes through [B7H.A04](apk/2.26.37.71-core/sender-key-account-recipient-wrap.txt)
into B7H.A06 with **B78.A02** and **B77.A03**. The sender's device survives;
these enum constants' complete bucket/actor semantics remain **unknown**.

[0bA.A0k](apk/2.26.37.71-core/sender-key-group-account-select.txt) accepts a 0c3
group, obtains its row through [0bA.A0B](apk/2.26.37.71-core/sender-key-group-account-row.txt)
(which delegates to 0bI.A0G), and returns
[1jL.A0Y](apk/2.26.37.71-core/sender-key-group-account-flag.txt). A0Y is exactly
**row.A00 == 2**, not a test on a particular recipient JID's textual suffix.
It selects LID in this cleanup path. **Unknown:** row initialization, persisted
mode producer, transitions and complete correspondence to the outbound group
addressing mode. The Rust comparison planner chooses a homogeneous membership
namespace and the CLI currently requires own LID; this is not full parity with
this authoritative APK group-row selector or PN fallback.

**Implementation boundary:** no arbitrary alias mapping, primary adoption,
companion identity cleanup or sender-key deletion was added. Existing exposure
rotation remains the documented supported subset. This trace reduces the
ownership uncertainty and records the precise next prerequisite: derive the
group-row mode producer before changing automatic own-address selection.
Static analysis cannot establish that the saved Rust account has a current
complete device inventory. No live/account request was sent for this research.

Own-account trace validation: 58 xtask tests, CLI reference regeneration and five
API checks passed. The core now holds 81 topics / 219 methods, 86 files /
1,586,459 bytes under its 2 MiB limit. The checksum-verified cached diff took
0.080554 seconds: 13 added topics, all 68 prior topics unchanged; manifest,
inventory and tokens unchanged. This measures cached scoped source comparison,
not full-app decompilation or automatic porting. No live request was needed.

## Group-mode producers, storage and cache classification (2026-10-07)

**Confirmed static, APK 2.26.37.71:** this continues the own-address selector
trace; it does not establish Rust mode parity or authorize an account migration.

`xtask apk-field-refs <APK> --class 'LX/1jL;' --field A00 --type I --writes-only`
found exactly five instructions in this APK. The full read/write scan found 26
instructions. Retained writers are:

| Writer | Write offset within method | Established input |
| --- | --- | --- |
| [1jL constructor](apk/2.26.37.71-core/sender-key-group-mode-row-init.txt) | 00c4..00c8 | Supplied mode integer; not an unconditional mode-2 default |
| [0yw.A00](apk/2.26.37.71-core/sender-key-group-mode-participant-update.txt) | 0008..0012 | Literal 2 before participant-update processing |
| [0yw.A0a](apk/2.26.37.71-core/sender-key-group-mode-participant-add.txt) | 0078..0082 | Literal 2 only inside the 0DE.A0R branch and when final boolean parameter is true |
| [0yw.A0y](apk/2.26.37.71-core/sender-key-group-mode-migrate.txt) | 014e..0160 | Literal 2 in the local group-migration branch |
| [31R.A01](apk/2.26.37.71-core/sender-key-group-mode-broadcast-migrate.txt) | 00d6..00d8, 0158..015c | Literal 2 in BroadcastListLidMigrationHelper; not a cursor loader |

The scan's absolute DEX write locations respectively are classes2.dex 0x149324,
classes.dex 0x75bf76 / 0x75f7ea / 0x761e8c, and classes3.dex 0x33e998.
These identify this build only. **Negative:** 0bA.A0V is a mode reader while
marking participant/device sender-key exposure; it is not a mode writer or a
row-save function. A read reference alone did not establish mode persistence.

### How the participant row gets its mode

[0bI.A0G](apk/2.26.37.71-core/sender-key-group-mode-row-load.txt) obtains the
participant row through 0bE.A0C, then updates own-membership caches.
[0bE.A0C](apk/2.26.37.71-core/sender-key-group-mode-cache-load.txt) returns a
cached 1jL if present; a cache miss constructs a row from **B7i.A00(chat)**
(`00ca..00f4`). The constructor stores that supplied integer in A00. Later
participant initialization also has CAG/mapping branches: complete fallback,
member-map generation and concurrency semantics remain **unknown/unported**.

[B7i.A00](apk/2.26.37.71-core/sender-key-group-mode-classify.txt) classifies
rather than simply parsing an `addressing_mode` string:

- `0DE.A0R` true: calls B7i.A02 and returns 2, **discarding** that return value.
- `0DE.A0W` true, or B7i.A04 true: returns 2.
- Otherwise a chat converted by 8kU.A0X and matching 0Fs.A0Y returns 1.
- Remaining `0DE.A0i` true returns 2; otherwise returns 0.

[0Fs.A0Y](apk/2.26.37.71-core/sender-key-group-mode-type-predicate.txt)
returns whether the group classification from 0Fs.A0A equals **3**. Its complete
classification producer and every 0DE predicate's chat-category meaning remain
**unknown** in this trace; a numeric branch is not a complete enum specification.
D4x.A00 uses mode 2 to select own LID and its other modes to select own PN, as
already retained. This is not equivalent to choosing a namespace by membership
text, nor to assuming every non-LID contact must initialize mode 1.

### Persisted contact mode and asymmetric lookup defaults

[0Da constructor](apk/2.26.37.71-core/sender-key-group-mode-contact-default.txt)
initializes **A0X = "lid"** (`000c..0014`).
[B7i.A01](apk/2.26.37.71-core/sender-key-group-mode-contact-lookup.txt)
returns the held contact's A0X, or literal `lid` if the contact is absent.
[B7i.A04](apk/2.26.37.71-core/sender-key-group-mode-lid-predicate.txt) instead
returns true only for a present contact whose A0X is exactly `lid`; absent
contact returns false. These methods' missing-contact behavior is **not equal**.
[B7i.A02](apk/2.26.37.71-core/sender-key-group-mode-broadcast-predicate.txt)
returns true even when the saved contact would have used PN, emitting the
`broadcast_would_have_used_pn` diagnostic. That diagnostic is not a refusal.
No equivalent bypass or forced migration is inferred for Rust.

[0Y9.A0s](apk/2.26.37.71-core/sender-key-group-mode-contact-set.txt) puts its
supplied mode in the held contact A0X (`0064..0074`), calls **0gt.A0V**, then
notifies 0gq.A0E. [0gt.A0V](apk/2.26.37.71-core/sender-key-group-mode-contact-save.txt)
updates base contact fields and calls **9R1.A0J** (`007a..00a0`).
[9R1.A0J](apk/2.26.37.71-core/sender-key-group-mode-settings-save.txt) reads
A0X (`0132..013a`), stores it as **addressing_mode** (`029c..02aa`) in
**wa_group_admin_settings**, and commits its database transaction
(`032a..0346`). Invalid group/broadcast input returns; errors have their own
handling and do not prove successful commit.

[0lO.A01](apk/2.26.37.71-core/sender-key-group-mode-settings-load.txt) reads
that cursor column (`042a..043e`). Case-insensitive `pn` becomes literal **pn**;
other returned values (including null) become literal **lid** (`043e..0456`).
This establishes a storage reload rule, not a network parser or permission to
invent a mode when a server reply is missing it. Missing-column resolution and
complete cursor-provider lifecycle remain **unknown**.

**Additional scan boundary:** the contact A0X write scan found 13 instructions:
0DV constructors (VL and VLLLLIJZ), 0Da constructor, 0Y9.A0Y/A0s, 0lO.A01,
0yv.A09 (two writes), 0yw.A0J, 1xq.A0A, 2xG.A00, 3S4.invoke and 9O2.A07.
Those other writers' complete mode inputs are **untraced**, so this is not a
complete mode-provenance graph. The participant-row constructor caller scan
found 0bE.A0C, 0yv.A09 and F4w.A01; the other full call paths remain untraced.
0yw.A00 has five call instructions in 0yv.A09 (two), 9O2.A07 (two) and 0yw.A0U.
Their existence does not establish every entry condition or all response formats.

**Migration boundaries:** 0yw.A0y logs migration to `lid`; it returns true if
already migrated and otherwise may schedule group-info recovery via 0Xm.A0H,
with an additional 0x30c1 feature branch. A local row mode write is not proof of
network authorization or completed reconciliation. 31R.A01 uses PN/LID mapping
and checks replacement list sizes; its mode write participates in that flow.
[0bA.A0h](apk/2.26.37.71-core/sender-key-group-mode-alias-schedule.txt) invokes
1jL.A0U and [290.A01](apk/2.26.37.71-core/sender-key-group-mode-alias-update.txt),
which removes mapped entries from private 28z cache when 1jL.A0G finds no row.
**Negative:** A0h is not a simple database mode-save method. Full migration
atomicity and mapping freshness remain unknown; Rust must not copy aliases or
reset identities based on these partial paths.

**Rust gap:** current saved GroupSnapshot and membership planner do not carry this
complete APK contact mode/category state. Comparison sending still requires own
LID even for a membership-planned PN group. Correct automatic own-address
selection needs mode provenance and integration, not an unconditional PN fallback
or a guess that homogeneous membership proves all APK predicates. This research
makes that implementation requirement concrete; no saved account mode, identity,
key, request ID, cooldown or send intent changed. No live request was sent.

Group-mode trace validation: 58 xtask tests, CLI reference regeneration and five
API checks passed. The core contains 100 topics / 238 methods, 105 files /
1,731,431 bytes under the 2 MiB budget. The checksum-verified cached diff took
0.075441 seconds: 19 added topics, all 81 prior topics unchanged; manifest,
inventory and tokens unchanged. No live/account request or runtime migration
was used for this source trace.

## Concrete JID kinds in group-mode classification (2026-10-07)

**Confirmed static, APK 2.26.37.71:** the mode predicates inspect concrete
`Jid.getType()` values, not the JID wire-encoding kind. Retained sources resolve
these previously unknown categories:

| Predicate | Exact type | Concrete source | Established namespace |
| --- | --- | --- | --- |
| [0DE.A0R](apk/2.26.37.71-core/sender-key-mode-broadcast-kind.txt) | 3 | [9Ln](apk/2.26.37.71-core/sender-key-mode-broadcast-jid.txt) | broadcast list (`broadcast`) |
| [0DE.A0W](apk/2.26.37.71-core/sender-key-mode-interop-kind.txt) | 28 / 0x1c | [Gym](apk/2.26.37.71-core/sender-key-mode-interop-jid.txt) | interop.g.us |
| [0DE.A0i](apk/2.26.37.71-core/sender-key-mode-status-kind.txt) | 5 | [8oR](apk/2.26.37.71-core/sender-key-mode-status-jid.txt) | status@broadcast |
| [0DE.A0m](apk/2.26.37.71-core/sender-key-mode-cache-group-kind.txt) | 1 or 2, explicitly excluding 28 | [1bn](apk/2.26.37.71-core/sender-key-mode-normal-jid.txt) / [9Lo](apk/2.26.37.71-core/sender-key-mode-temporary-jid.txt) | g.us / temp |

The three type-equality predicates return false for null. A0m first excludes
A0W, then accepts types 1 or 2. [Supplementary 0DE.A0n source](group-mode-category-evidence.txt)
returns the union of A0m and A0W: types 1, 2 and 28. It is not identical to A0m.
The status singleton's constructor passes `status` to Jid and its getServer
returns `broadcast`; this proves status identity, not merely a shared parent.
The temporary group's getServer returns literal `temp`. **Refuted:** type 2 is
not established as an old g.us PN group; it is a temporary group namespace.
`interop.g.us` is also distinct from both g.us and encoding `JidKind::Interop`.
No unsupported namespace is enabled in Rust by these source findings.

**B7i.A00 interpretation with the retained preceding trace:** broadcast lists
(type 3) and interop groups (type 28) select mode 2 before the normal contact/
group-type branch. A present contact marked exactly `lid` selects mode 2.
Otherwise 0Fs.A0Y checks persisted group type 3 within its A0n domain: true
selects participant mode 1, false allows status (type 5) to select mode 2, with
remaining cases selecting 0. D4x.A00 selects own LID for participant mode 2
and own PN for the other modes. Do not confuse **JID type 3** (broadcast list),
**group type 3** (default subgroup), and **participant mode 1**: they are three
different fields used in different branches.

**Existing evidence reconciled:** `0Fs.A0A` was already retained in the group
research [group-type cache reader](apk/2.26.37.71-groups/group-type-cache-read.txt)
and documented under group subject types. It reads 0Ze.A04 from the initialized
chat map or returns 0 on cache miss; 0Fs.A02 reads database column `group_type`.
ADs.A01 separately maps a complete reply's default_sub_group child to value 3.
These are **confirmed** producer/read facts. Mapping Rust's complete-reply
`GroupType::DefaultSubgroup` to this selected branch is **inferred compatibility**;
the entire callback/cache provenance graph and history-merging lifecycle are not
proved by the numeric agreement. An unknown historical Rust classification is
not proof of an APK cache miss, ordinary group or participant mode 0.

**Negative search results and scope:** getType body searches on 0c3 and
GroupJid found none; a full 0c3 body search found none, and GroupJid only exposed
its static companion initializer. Those base names do not establish a concrete
type. 9Lq had no getServer body; concrete status/list subclasses supply it.
The broad LX/ getType method-name scan matched 77 methods across 75 classes,
including unrelated non-JID types. Those counts are not a complete JID catalog
and unrelated numeric matches were excluded. Ordinary/temp/interop/status/list
sources carry the selected class's getter and server evidence together.

**Remaining runtime boundary:** the Rust planner still uses membership
namespace and excludes temp/interop groups; it does not have full authoritative
cached contact mode, chat-type initialization or history merging. This trace
closes the predicate meaning, not automatic mode synchronization or PN fallback.
No identity policy, key chain, request reservation, account mode or live traffic
changed. The new core topics remain hash-pinned and release-diffable; the small
A0n supplementary evidence is explicitly outside that checksum bundle.

Concrete-kind trace validation: 58 xtask tests, CLI reference regeneration and
five API checks passed. Core snapshot: 109 topics / 275 methods, 114 files /
1,755,155 bytes (2 MiB bound); supplementary A0n source is separate. The
checksum-verified cached diff took 0.197003 seconds: nine added topics, all
100 prior topics unchanged; manifest, inventory and tokens unchanged. These are
static/offline findings; no live behavior or automatic mode adoption is claimed.

## Persisted group-type updates feeding sender mode (2026-10-07)

**Confirmed static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
[0aK.A03](apk/2.26.37.71-core/group-type-existing-update.txt) asks
0Fs.A0F for an existing chat (0002..000e). A missing row logs the requested
type and returns (000e..0046); it does not initialize a chat. An existing row
receives the supplied integer in 0Ze.A04 at 008c..0090, then calls A00 with
null created_timestamp (0090..0096). This is an explicit metadata update,
not inference from member addresses or message contents.

[0aK.A00](apk/2.26.37.71-core/group-type-persist.txt) opens a database handle
and transaction (0004..0018). Under the row monitor it copies subject,
group_type=A04, and optional growth-lock fields into ContentValues
(0036..0098); created_timestamp is copied only when its argument is nonnull.
It calls [0VS.A0Y](apk/2.26.37.71-core/group-type-persist-upsert.txt) at
0098..009e. A false result logs insert failure but still follows the normal
transaction A00/close path and returns true (00a0..00d8). Do not interpret
A03's void return or this normal Boolean as proof of an actual changed row.
Exception handlers exist, but their complete typed/rollback effects are not
established by this selected structural trace; no Rust rollback policy is
inferred from instruction order alone.

0VS.A0Y first calls [A06](apk/2.26.37.71-core/group-type-persist-update.txt).
When the updated-row count is zero it supplies jid_row_id, calls A09, stores
the returned row ID, and returns false for -1 (0000..0054). A06 writes table
`chat` through 0IY.A02 with either `account_jid_row_id = ?` or `jid_row_id = ?`
according to 0ZR.A03 (00c0..00ee / 012e..0134). It also computes `hidden`
from separate chat-type/state predicates and repairs row IDs on successful
updates. Full A09 insertion/schema and those visibility predicates remain
**unknown**; this is not permission to copy the Android storage schema into
our encrypted store or change chat visibility.

[Selected caller/dispatch evidence](group-type-update-callers-evidence.txt)
records the only two A03 call instructions found by an exact class/method
scan: Aha.run at classes6.dex byte 0x5c2e08 and Ahc.run at 0x5c6636.
Aha's dispatcher case 31 enters 0854; it iterates supplied group JIDs,
invokes 0a8.A06, passes literal 0 to A03 (0892..089e), and schedules AhX
selector 0x26. Ahc case 7 enters 13e6; it passes AC3.A02 as chat and
AC3.A00 as the new type (1406..1418), then calls 0a8.A07 and notification
schedulers. [AC3](apk/2.26.37.71-core/group-type-subgroup-model.txt)
toString explicitly labels A00 as groupType. Runnable constructor overloads
share shorties and differ in field assignments; no exact constructor-producer
mapping, network origin, parent/removal semantics or complete callback
provenance was established. The literal-zero path cannot serve as a generic
fallback for missing classification.

**Additional database/type scope:**
[0a2.A04](apk/2.26.37.71-core/group-type-secondary-load.txt) reads required
cursor column group_type (00b0..00bc) into 0Ze.A04 (03f8..0408).
[0a2.A0D](apk/2.26.37.71-core/group-type-secondary-init.txt) builds an
alternate model with type 4 at 00bc..00c2 before 9lE.A00;
[ADu.A0G](apk/2.26.37.71-core/group-type-secondary-result.txt) also writes
4 to a newly constructed row at 02a6..02ac before that wrapper. These are
confirmed writes, not evidence that binary complete group replies produce
an additional GroupType enum value. The current Rust enum intentionally
covers the independently traced ADs.A01 complete-reply classification;
meaning, routing relevance and full lifecycle of type 4 remain unknown.

**Two-stage predicate correction:** retained 0Fs.A0Y accepts A0n's JID
categories 1/2/28 before calling A0A. A0A independently requires A0m, which
excludes type 28; therefore type 28 cannot reach true through this A0Y path,
even if a cached row has group_type=3. B7i.A00 already handles type 28 as
mode 2 before A0Y. [A0n](apk/2.26.37.71-core/sender-key-mode-group-domain.txt)
is now in the checksum bundle as well as the earlier supplementary excerpt.
Do not claim A0Y reads type 3 for every category in its outer domain.

**Negative search coverage:** the full 0Ze.A04 write scan remains 21
instructions (retained in the caller appendix), spanning constructor/cache,
secondary-model, subgroup and history paths. Only the selected paths above
and earlier linked evidence are traced; history merge and subgroup producers
remain incomplete. A broad A0Y caller scan found 137 call instructions,
including UI/non-sender consumers; that is not 137 sender-mode producers.
No account inspection, network request, key/pin replacement, reservation
change or automatic PN fallback was performed. Runtime authoritative contact
mode and complete group-type synchronization remain unfinished.

Group-type trace validation: 58 xtask tests, regenerated CLI reference and five
API checks passed. The core checksum bundle now has 118 topics / 289 methods,
123 files / 1,829,892 bytes (2 MiB bound). The verified cached diff took
0.168698 seconds: nine added topics, all 109 previous topics unchanged;
manifest, inventory and tokens unchanged. Selected runnable excerpts remain
outside the checksum bundle. No runtime behavior or live validation is claimed.

Addressing-mode producer trace (2026-10-07, **confirmed static APK 37.71**):
complete binary group replies do not directly read the mode attribute. Generated
validators use exact pn/lid membership; incoming group-message parsing has a
separate case-insensitive PN check. Five contact-mode setter callers now have
selected paths retained, including gated info-error and PN-conversion branches;
triggers, full callback provenance and runtime integration remain unfinished.
No live request or account change. [Evidence and boundaries](groups.md#addressing-mode-producers-and-separate-parser-rules-2026-10-07).

Group pairwise companion correction (2026-10-07, **confirmed offline**):
held and pinned companion msg sessions retain their actual participant device;
primary/alias fallback, unpinned and prekey-bootstrap paths are excluded from
this local subset. Real Signal PN/LID primary/device-3 control fixtures now
install sender keys and retain duplicate markers across reopen. B7H forward/
reverse device construction is linked; CPQ.A00 is refuted as a 37.71 session
selector. Full companion bootstrap/identity policy and live device coverage
remain unfinished. [Evidence and scope](groups.md#group-pairwise-session-device-correction-2026-10-07).


## Identity wait caller and extensions verification (2026-10-09)

**Confirmed static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The earlier Fco held-future trace did not establish its caller or consumer.
The exact DEX constructor scan found one `Fco.<init>` caller: `FJa.A02`.
The [caller scan](identity-wait-caller-evidence.txt) also finds FJa.A02 callers
on FJp, FwE and the named FlowsMetadataAssetsManager coroutine. This is a
method-reference scan, not exhaustive native/reflection runtime reachability.

- [FJa.A02](apk/2.26.37.71-identity-wait/identity-wait-extension-entry.txt)
  requires two nonempty supplied strings (0014..0038), reads feature **0x176a**
  (0048..006c), and on the enabled branch builds F3t and calls FDx.A00.
  A passing verification completes immediately; a failure queues Fan in
  Ezi.A05's per-user callback map (00ac..00f8). The first waiter creates a
  SettableFuture/Fco, registers Fco on **0k4 and 1MB** via 078.A0K
  (00f8..013c), requests that user's primary identity through 11S.A01
  (014c..0162), then schedules Fr9 discriminator **17 / 0x11** (0162..0178).
  An existing per-user list instead appends the callback (00da..00e6,
  0184..018c); this is local coalescing, not repeated independent identity reads.
  The disabled-feature branch checks saved keys directly (0194..02d4).
- [Full Fco methods](apk/2.26.37.71-identity-wait/identity-wait-full-observer.txt):
  constructor stores the future and user. BdO normalizes its user argument to
  primary; BpH, BqB and BqC invoke A00, which compares with the held user's
  primary and sets the future to 05R.A00 on equality. **Negative:** BdR and
  BqD are empty. Changed-key metadata/boolean are not checked by BqC; future
  completion is an event wakeup, not a cryptographic verification result.
- [Fr9 factory](apk/2.26.37.71-identity-wait/identity-wait-timed-factory.txt) and
  [constructors](apk/2.26.37.71-identity-wait/identity-wait-timed-constructor.txt)
  retain the future, Ezi, observer and user consumed by case 17. Three overloaded
  constructors share this shorty; the factory's selected full-signature linkage
  is not uniquely established by shorty filtering alone. The case's casts and
  caller arguments independently establish the roles consumed at runtime.
- [Fr9.run](apk/2.26.37.71-identity-wait/identity-wait-timed-worker.txt) dispatches
  case 17 to **2034**. At 204c..205a it waits **30 seconds** (`0x1e`,
  TimeUnit.SECONDS), then under the map monitor unregisters both observers,
  removes that user's callback list, and invokes every G1I.BoF (205a..20b8).
  The Exception handler logs `Cannot fetch the identity for business!` and
  performs the same unregister/remove/callback work (20b8..2126); a catch-all
  also cleans up/calls callbacks and rethrows (2126..2190). Checked exception
  ranges are retained in the [caller artifact](identity-wait-caller-evidence.txt)
  (code-unit addresses, distinct from the byte offsets quoted here). This timeout belongs to the local extensions wait;
  it is not the separate identity IQ's 32000 timeout or a server cooldown.
- [Fan.BoF](apk/2.26.37.71-identity-wait/identity-wait-extension-callback.txt)
  calls FDx.A00 again on its saved arguments, and passes the returned boolean
  to F3t.A00 (0014..0022). [FDx.A00](apk/2.26.37.71-identity-wait/identity-wait-extension-verifier.txt)
  builds primary plus conditionally filtered companion addresses, reads saved
  identities via 0jc.A0i, decodes both supplied Base64 strings and checks
  **B7H.A09** against held public identities (00a4..0112). It returns true
  on a passing check, false otherwise. Companion selection predicate 1lL.A04
  and Cbb.A00 semantics are not established by this scoped trace.
  [F3t.A00](apk/2.26.37.71-identity-wait/identity-wait-extension-result.txt) chooses
  FJa.A01 on true and FJa.A00 on false. Full downstream effects remain untraced.

**Refuted inference:** Fco future completion is not proof of successful identity
verification, session repair or incoming-message replay. Both wait completion
and Exception timeout invoke a callback that rechecks saved keys. This caller
chain concerns extensions/business/Flows metadata; no receive ratchet, captured
message or receipt is accessed in the traced wait case. That does not exclude
other identity-event observers having receive effects. The earlier AhK -> ATU
identity-success event is still real, but this particular subscriber is not
sufficient evidence for automatic recovery in Rust.

**Unknown/unported:** effective 0x176a for our account, complete FJa result paths,
observer threading/registration lifecycle, all other 0Y6 consumers, the native
wire-to-Java factory, complete identity-only adoption and downstream receive
replay/cleanup. No Rust pin/session policy, timeout, account state, queued action
or network behavior changed. This narrows the next recovery trace to actual
receive consumers rather than porting this extensions waiter into messaging.

Reproduce with the existing `apk-callers` commands in the caller artifact and
`apk-research-bundle <apk> --recipe docs/protocol/apk/recipes/2.26.37.71-identity-wait.json
--output <new-dir>`. Cached `apk-research-diff <old-dir> <new-dir>` verifies
checksums and reports topic changes without scanning the APK again. These are
existing offline research commands, not new HTTP routes or live capabilities.

**Recorded negative export result:** adding these eight topics to the existing
core recipe produced 2,158,098 bytes and was refused by the fixed 2,097,152-byte
bundle limit before writing a replacement. The original core snapshot and
recipe are restored unchanged. The separate hash-pinned identity-wait recipe
keeps this chain within the existing limit and enables checked exception tables;
no limit was raised and no existing evidence was discarded.

Validation: the separate recipe exported **8 topics / 18 selected methods** in
13 files, **206,678 bytes**, and `--check` independently reproduced all output.
The [cached same-bundle check](apk/identity-wait-cache-check.json) verified all
checksums and reported 8 unchanged topics / 4 unchanged metadata artifacts in
**0.019347 seconds**. This measures this scoped cached
comparison, not a new release, whole-APK semantics or automatic patching. The
original core files/recipe are Git-identical. All **63 xtask tests**, regenerated
CLI reference, API library-symbol verification and formatting passed.
Repository hygiene also passed: 1,008 local links and 4,786,515 bytes across
8 checksum bundles. No private state was opened or live request sent.


## Identity-success consumer inventory (2026-10-09)

**Confirmed static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The earlier AhK case 7 -> ATU case 1 continuation calls both **0Y6.BdR** and
**BdO**. A fresh full-DEX method-name scan returned **11 methods for each**:
10 concrete classes and the abstract 0Y6 declaration. The
[caller/inventory evidence](identity-wait-caller-evidence.txt) records commands
and scope. Fco's full methods remain in the existing wait snapshot; the
[expanded recipe](apk/recipes/2.26.37.71-identity-wait.json) adds both callbacks
for every other class, plus AL2's constructor and its caller's full task.
These are complete physical method bodies for this scan, not a complete
native/reflection/virtual-dispatch reachability claim. Named bodies alone
do not prove every class is registered on this particular event source; complete
registration/dispatch closure remains unestablished.

| Class | BdO | BdR / traced immediate effect |
| --- | --- | --- |
| Fco | Completes matching primary future | Empty |
| AL2 | Compares held user and counts down 9N0.A08 | Empty |
| FWe | Empty | Six discriminator paths: business-profile/catalog UI or own-account predicate; downstream helpers untraced |
| 1za | Empty | Matching-user UI worker, options-menu invalidation, optional Do9 refresh |
| 3B5 | Empty | Matching-user 34O.A00 helper; internals untraced |
| AL3 | Empty | Guarded execution of AhL case 18; worker internals untraced |
| AL4 | Empty | Business-profile fields, contact-info/layout and related UI helpers |
| Kv7 | Empty | Search callbacks or JNG business-profile work/metrics; downstream helpers untraced |
| 0Y9 | Empty | Resolves 0gq then calls A0F(user); that helper remains untraced |
| 1We | Empty | ConversationsFragment businessProfileObserver refresh or 1Oa.A0n helper |
| 0Y6 | Abstract declaration | Abstract declaration |

**Confirmed second waiter:** AL2.<init> stores the 9N0 task and target user;
the constructor scan finds exactly one DEX caller, **9N0.<init>**.
[AL2.BdO](apk/2.26.37.71-identity-wait/identity-success-al2-bdo.txt)
0000..001e compares the received user and counts down the held task's latch.
[9N0](apk/2.26.37.71-identity-wait/identity-success-business-profile-task.txt)
identifies its purpose with the literal
`ConversationRowContact/OpenBusinessProfileNotInContactListTask`.
A0Y registers the 0Y6 observer on 1MB (0022..002c). A0Z's enabled 0x51fc
branch handles vCard phone lookup and counts down immediately (0004..0050);
the other branch selects a normal contact request, then waits **30000 ms**
(00cc..00de). InterruptedException logs a business-identity wait interruption
(00e0..00ec). A0a/A0b unregister; A0b builds and opens the business-profile UI.
The returned latch boolean is not used as proof of key validation. These are
method-relative byte offsets; retained catch tables use code units.

**Negative result / narrowed recovery evidence:** neither BdO's two nonempty
bodies nor the scanned BdR bodies directly decrypts a message, selects/stores a
Signal session or queues a message retry/receipt. The literal 1We diagnostic
names this observer a `businessProfileObserver`, and both concrete BdO waiters
have business/Flows consumers. This refutes treating these particular callbacks
as established incoming replay. It does **not** prove transitive helpers or
other identity-save observers cannot affect receive. 0gq.A0F, AhL case 18,
1Oa.A0n and other helpers remain explicit unknowns. Broader identity-change
cleanup via D4x/DeS is a separate already-recorded branch.

**Next receive boundary, confirmed local inspection:** inbox.session_sender
currently requires sender_pn for a primary LID sender, while its companion LID
projection already uses the actual namespace without an alias. The current
primary alias requirement is **our implementation**, not an APK-mandatory-field
claim established here. Whether/when that primary alias can be absent and how
the APK then keys message/chat/deduplication requires its actual receive routing
trace before a correction; this callback inventory does not answer it.
No alias, pin, session, request ID, event marker or account state changed.
No live request was made for this static inventory.

The [cached coverage diff](apk/identity-success-consumer-coverage-diff.json)
verified all checksums: **22 added topics, 8 prior topics unchanged**, no changed
or removed method topics, in **0.014128 seconds**. This
compares coverage of the same APK, not different releases or semantic equivalence.
The expanded snapshot has **30 topics / 44 selected methods** (including the
2 abstract interface declarations), **35 files / 250,138 bytes**. The earlier
18-method cache check remains a historical measurement of the pre-expansion
snapshot. All 63 xtask tests, regenerated CLI reference, API library-symbol
verification and formatting passed.
Independent `--check` reproduced all 35 files / 250,138 bytes. Repository
hygiene passed: 1,016 local links and 4,829,975 bytes across 8 checksum bundles.

## Confirmed clock source continuation (2026-10-09)

**Confirmed local / 37.71 source-backed:** [clock runtime](token-clock-producers.md#confirmed-runtime-clock-inputs-2026-10-09)
now records login success and server-ping time separately from historical inferred
message/receipt samples. Chat-mute resolution prefers confirmed elapsed/offset
state; offline clock-inspect reports provenance without time or epoch values.
NTP-preferred A00 and complete native mute-clock parity remain unported.

## Companion ADV serialization correction (2026-10-09)

[APK parse/serialize evidence](adv-details-serialization.md) supersedes the
original-details assumption for 11T.A0D's companion device signatures only.
GsB unknown fields/closed enums are preserved; native Azc has NON_E2EE=2.
Signed index-list input is a separate consumer. Existing membership, own-device,
teardown, freshness and nested-message authentication gates remain required.
