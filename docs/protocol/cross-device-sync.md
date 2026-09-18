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
