//! Comparison app-state key-share decoding and fingerprint checks.
//! Decoding is NOT sender authorization. Import is crate-private and called
//! only by the authenticated own-device inbox path; keys are not yet activated.
//! Evidence: LX/BL6, BPH, BM0, BRp, BSp; BjO.A0s; 0oK.A0B (2.26.37.70).
use crate::{Error, Result};
use prost::Message;
use std::collections::BTreeSet;
use zeroize::{Zeroize, Zeroizing};

const MAX_KEYS: usize = 256;
const MAX_DEVICES: usize = 1024;
fn invalid(reason: &'static str) -> Error {
    Error::invalid("app_state_key_share", reason)
}

#[derive(Message)]
struct Share {
    #[prost(message, repeated, tag = "1")]
    keys: Vec<WireKey>,
}
#[derive(Message)]
struct WireKey {
    #[prost(message, optional, tag = "1")]
    id: Option<KeyId>,
    #[prost(message, optional, tag = "2")]
    data: Option<KeyData>,
}
#[derive(Message)]
struct KeyId {
    #[prost(bytes = "vec", optional, tag = "1")]
    id: Option<Vec<u8>>,
}
#[derive(Message)]
struct KeyData {
    #[prost(bytes = "vec", optional, tag = "1")]
    data: Option<Vec<u8>>,
    #[prost(message, optional, tag = "2")]
    fingerprint: Option<WireFingerprint>,
    #[prost(int64, optional, tag = "3")]
    timestamp: Option<i64>,
}
impl Drop for KeyData {
    fn drop(&mut self) {
        if let Some(data) = &mut self.data {
            data.zeroize();
        }
    }
}
#[derive(Message)]
struct WireFingerprint {
    #[prost(uint32, optional, tag = "1")]
    raw_id: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    current_index: Option<u32>,
    #[prost(uint32, repeated, packed = "true", tag = "3")]
    device_indexes: Vec<u32>,
}

/// Fingerprint metadata, separate from key material and sender authentication.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Fingerprint {
    raw_id: u32,
    current_index: u32,
    device_indexes: BTreeSet<u32>,
}
impl std::fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fingerprint")
            .field("device_count", &self.device_indexes.len())
            .finish_non_exhaustive()
    }
}
impl Fingerprint {
    /// APK 0oK.A0B: same raw ID; add indexes allocated since this fingerprint,
    /// then require exact equality with the current registered-device index set.
    /// Inputs must come from authenticated account/device state, never a request
    /// profile or the untrusted key-share itself. This is NOT sender authorization.
    pub fn matches_registered_devices(
        &self,
        raw_id: u32,
        current_index: u32,
        registered: &BTreeSet<u32>,
    ) -> bool {
        if raw_id != self.raw_id || registered.is_empty() || registered.len() > MAX_DEVICES {
            return false;
        }
        let added = current_index.saturating_sub(self.current_index);
        if added as u64 > MAX_DEVICES as u64 {
            return false;
        }
        let mut expected = self.device_indexes.clone();
        if self.current_index < current_index {
            expected.extend((self.current_index + 1)..=current_index);
        }
        &expected == registered
    }
}
struct DecodedKey {
    id: [u8; 6],
    data: Zeroizing<Vec<u8>>,
    fingerprint: Fingerprint,
    timestamp: i64,
}

#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
#[derive(serde::Serialize, serde::Deserialize)]
struct StoredKey {
    data: Vec<u8>,
    fingerprint: Fingerprint,
    timestamp: i64,
}
#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
impl Drop for StoredKey {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

/// Selected mutation keys, not proof of device authorization or completed sync.
#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
pub struct SelectedKey {
    id: [u8; 6],
    keys: super::app_state_crypto::MutationKeys,
}
#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
impl std::fmt::Debug for SelectedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SelectedKey(<redacted>)")
    }
}
#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
impl SelectedKey {
    /// Exact six-byte identifier to bind into an outgoing mutation.
    pub fn id(&self) -> &[u8; 6] {
        &self.id
    }
    /// Derived integrity/encryption keys, with redacted Debug and zeroizing drop.
    pub fn keys(&self) -> &super::app_state_crypto::MutationKeys {
        &self.keys
    }
}

/// APK 0oL.A02/0oK.A04: select epoch DESC/device ASC, then check age and
/// fingerprint. Never fall back to an older key when the selected one fails.
///
/// Caller must supply independently authorized account/device state from the
/// same transaction's account, not values copied from an incoming share. Age
/// policy must be explicitly resolved (APK flag 0x3898); no default is guessed.
/// This read-only primitive does not activate keys, establish device trust,
/// generate replacements or authorize incoming historical mutation keys.
/// Future-dated keys are conservatively refused as a local clock-safety policy.
#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
pub fn select_active_key(
    tx: &crate::store::WriteTx<'_>,
    raw_id: u32,
    current_index: u32,
    registered: &BTreeSet<u32>,
    now_ms: i64,
    max_age_ms: i64,
) -> Result<Option<SelectedKey>> {
    use crate::store::Table;
    if now_ms < 0 || max_age_ms < 0 {
        return Err(invalid("invalid key age policy"));
    }
    let mut latest: Option<([u8; 6], Zeroizing<Vec<u8>>)> = None;
    let mut cursor = None;
    loop {
        let rows = tx.scan_after(Table::Checkpoints, 1, cursor.as_deref(), 256)?;
        if rows.is_empty() {
            break;
        }
        cursor = rows.last().map(|(name, _)| name.clone());
        for (name, bytes) in rows {
            let bytes = Zeroizing::new(bytes);
            let Some(encoded) = name.strip_prefix(b"app-state-key:") else {
                continue;
            };
            let mut id = [0; 6];
            hex::decode_to_slice(encoded, &mut id)
                .map_err(|_| invalid("invalid stored key identifier"))?;
            let rank = |id: &[u8; 6]| {
                (
                    // 0oK.A09 sign-extends 1Lz.A01's Java int before SQL bind.
                    i32::from_be_bytes([id[2], id[3], id[4], id[5]]),
                    std::cmp::Reverse(u16::from_be_bytes([id[0], id[1]])),
                )
            };
            if latest.as_ref().is_none_or(|(old, _)| rank(&id) > rank(old)) {
                latest = Some((id, bytes));
            }
        }
    }
    let Some((id, bytes)) = latest else {
        return Ok(None);
    };
    let saved: StoredKey =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid stored key"))?;
    let Some(age) = now_ms.checked_sub(saved.timestamp).filter(|age| *age >= 0) else {
        return Ok(None);
    };
    if age > max_age_ms
        || !saved
            .fingerprint
            .matches_registered_devices(raw_id, current_index, registered)
    {
        return Ok(None);
    }
    if saved.data.len() != 32 {
        return Err(invalid("unsupported active key length"));
    }
    Ok(Some(SelectedKey {
        id,
        keys: super::app_state_crypto::MutationKeys::derive(&saved.data)?,
    }))
}

#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
#[derive(Message)]
struct Envelope {
    #[prost(message, optional, tag = "12")]
    protocol: Option<Protocol>,
}
#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
#[derive(Message)]
struct Protocol {
    #[prost(int32, optional, tag = "2")]
    kind: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "7")]
    share: Option<Vec<u8>>,
}
#[cfg(any(
    test,
    all(feature = "experimental-noise", feature = "experimental-transport")
))]
impl Drop for Protocol {
    fn drop(&mut self) {
        if let Some(bytes) = &mut self.share {
            bytes.zeroize();
        }
    }
}

/// Validated structure only. Secret bytes have no public formatter/export and
/// are zeroized on drop. Import requires a separate authenticated own-device path.
pub struct KeyShare {
    keys: Vec<DecodedKey>,
}
impl std::fmt::Debug for KeyShare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyShare")
            .field("key_count", &self.keys.len())
            .finish_non_exhaustive()
    }
}
impl KeyShare {
    /// Select a top-level type-6 key-share from an authenticated peer message.
    /// Other protocol messages are not key imports, even if field 7 is present.
    #[cfg(any(
        test,
        all(feature = "experimental-noise", feature = "experimental-transport")
    ))]
    pub(crate) fn from_peer_message(bytes: &[u8]) -> Result<Option<Self>> {
        if bytes.len() > 1_048_576 {
            return Err(invalid("peer message exceeds local byte bound"));
        }
        let envelope = Envelope::decode(bytes).map_err(|_| invalid("invalid peer protobuf"))?;
        let Some(protocol) = envelope.protocol else {
            return Ok(None);
        };
        if protocol.kind != Some(6) {
            return Ok(None);
        }
        let bytes = protocol
            .share
            .as_deref()
            .ok_or_else(|| invalid("missing key-share body"))?;
        Self::decode(bytes).map(Some)
    }

    /// APK 0oK.A09: insert new IDs only, never replace existing key material.
    /// Caller must authenticate the own-device sender before calling, and use
    /// the same transaction as the ratchet/inbox commit. Stored keys have no
    /// automatic activation path: explicit authorized-state selection and
    /// snapshot/patch MAC verification are still required before use.
    #[cfg(any(
        test,
        all(feature = "experimental-noise", feature = "experimental-transport")
    ))]
    pub(crate) fn import(&self, tx: &mut crate::store::WriteTx<'_>) -> Result<usize> {
        use crate::store::Table;
        let mut inserted = 0;
        for key in &self.keys {
            let name = format!("app-state-key:{}", hex::encode(key.id));
            if tx
                .get_json::<StoredKey>(Table::Checkpoints, name.as_bytes(), 1)?
                .is_some()
            {
                continue;
            }
            let saved = StoredKey {
                data: key.data.to_vec(),
                fingerprint: key.fingerprint.clone(),
                timestamp: key.timestamp,
            };
            tx.put_json(Table::Checkpoints, name.as_bytes(), 1, &saved)?;
            inserted += 1;
        }
        Ok(inserted)
    }

    /// Decode the AppStateSyncKeyShare body (ProtocolMessage field 7), not a
    /// whole Message. Bounds are local resource limits, not server requirements.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 65536 {
            return Err(invalid("share exceeds local byte bound"));
        }
        let share = Share::decode(bytes).map_err(|_| invalid("invalid key-share protobuf"))?;
        if share.keys.len() > MAX_KEYS {
            return Err(invalid("too many shared keys"));
        }
        let mut keys = Vec::new();
        let mut ids = BTreeSet::new();
        for wire in share.keys {
            let id: [u8; 6] = wire
                .id
                .and_then(|v| v.id)
                .ok_or_else(|| invalid("missing key id"))?
                .try_into()
                .map_err(|_| invalid("unsupported key id width"))?;
            if !ids.insert(id) {
                return Err(invalid("duplicate key id"));
            }
            let mut data = wire.data.ok_or_else(|| invalid("missing key data"))?;
            let fingerprint = data
                .fingerprint
                .take()
                .ok_or_else(|| invalid("missing fingerprint"))?;
            let raw_id = fingerprint
                .raw_id
                .ok_or_else(|| invalid("missing fingerprint raw id"))?;
            let current_index = fingerprint
                .current_index
                .ok_or_else(|| invalid("missing fingerprint current index"))?;
            if fingerprint.device_indexes.is_empty()
                || fingerprint.device_indexes.len() > MAX_DEVICES
            {
                return Err(invalid("invalid fingerprint device count"));
            }
            let fingerprint = Fingerprint {
                raw_id,
                current_index,
                device_indexes: fingerprint.device_indexes.into_iter().collect(),
            };
            let secret = Zeroizing::new(
                data.data
                    .take()
                    .ok_or_else(|| invalid("missing key bytes"))?,
            );
            if secret.is_empty() || secret.len() > 4096 {
                return Err(invalid("key bytes outside local bounds"));
            }
            keys.push(DecodedKey {
                id,
                data: secret,
                fingerprint,
                timestamp: data.timestamp.unwrap_or(0),
            });
        }
        Ok(Self { keys })
    }
    /// Number of structurally valid keys, safe for redacted diagnostics.
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    /// Whether no keys were included.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
    /// Inspect only fingerprint metadata, never raw key material.
    pub fn fingerprints(&self) -> impl Iterator<Item = &Fingerprint> {
        self.keys.iter().map(|v| &v.fingerprint)
    }
    /// Check consistency of a repeated share without exposing secret bytes.
    pub fn same_keys(&self, other: &Self) -> bool {
        self.keys.len() == other.keys.len()
            && self.keys.iter().all(|a| {
                other.keys.iter().any(|b| {
                    a.id == b.id
                        && a.data == b.data
                        && a.fingerprint == b.fingerprint
                        && a.timestamp == b.timestamp
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Independently written protobuf: keys[0] id=00:01:00:00:00:02,
    // keyData="test", fingerprint raw=7/current=2/devices=[1,2], timestamp=9.
    const FIXTURE:&[u8]=b"\x0a\x1e\x0a\x08\x0a\x06\x00\x01\x00\x00\x00\x02\x12\x12\x0a\x04test\x12\x08\x08\x07\x10\x02\x1a\x02\x01\x02\x18\x09";
    #[test]
    fn selection_pages_past_unrelated_history_and_considers_every_key() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("pages.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let registered = BTreeSet::from([1, 2]);
        store
            .write(|tx| {
                for index in 0..10_000 {
                    tx.put(
                        Table::Checkpoints,
                        format!("a-history:{index:05}").as_bytes(),
                        1,
                        b"unrelated",
                    )?;
                }
                // Physical order is device-first, unlike the APK's epoch-first
                // selection. Force candidates across several page boundaries.
                for device in 1..=600u16 {
                    let mut share = KeyShare::decode(FIXTURE)?;
                    share.keys[0].id[..2].copy_from_slice(&device.to_be_bytes());
                    share.keys[0].id[2..].copy_from_slice(&u32::from(device).to_be_bytes());
                    share.keys[0].data = Zeroizing::new((0..32).collect());
                    share.import(tx)?;
                }
                let selected = select_active_key(tx, 7, 2, &registered, 10, 10)?.unwrap();
                assert_eq!(selected.id(), &[2, 88, 0, 0, 2, 88]);
                Ok(())
            })
            .unwrap();
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        store
            .write(|tx| {
                assert_eq!(
                    select_active_key(tx, 7, 2, &registered, 10, 10)?
                        .unwrap()
                        .id(),
                    &[2, 88, 0, 0, 2, 88]
                );
                // Ineligible final candidate must not cause an earlier-page fallback.
                let name = b"app-state-key:025800000258";
                let mut saved: StoredKey = tx.get_json(Table::Checkpoints, name, 1)?.unwrap();
                saved.timestamp = 0;
                tx.put_json(Table::Checkpoints, name, 1, &saved)?;
                assert!(select_active_key(tx, 7, 2, &registered, 10, 9)?.is_none());
                assert_eq!(tx.count(Table::Checkpoints)?, 10_600);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn active_selection_orders_then_validates_without_fallback_or_writes() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("selection.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let registered = BTreeSet::from([1, 2]);
        assert!(
            store
                .write(|tx| select_active_key(tx, 7, 2, &registered, 30, 10))
                .unwrap()
                .is_none()
        );
        for (device, epoch, timestamp) in
            [(1u16, 1u32, 30), (3, 2, 30), (1, 2, 20), (1, u32::MAX, 30)]
        {
            let mut share = KeyShare::decode(FIXTURE).unwrap();
            share.keys[0].id[..2].copy_from_slice(&device.to_be_bytes());
            share.keys[0].id[2..].copy_from_slice(&epoch.to_be_bytes());
            share.keys[0].data = Zeroizing::new((0..32).collect());
            share.keys[0].timestamp = timestamp;
            store.write(|tx| share.import(tx)).unwrap();
        }
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        let before = store.read(|tx| tx.scan(Table::Checkpoints, 1, 20)).unwrap();
        let selected = store
            .write(|tx| select_active_key(tx, 7, 2, &registered, 30, 10))
            .unwrap()
            .unwrap();
        // Epoch 2 wins over newer timestamp at epoch 1; device 1 wins the tie
        // despite device 3 having a newer timestamp. Exact max-age is accepted.
        assert_eq!(selected.id(), &[0, 1, 0, 0, 0, 2]);
        assert_eq!(format!("{selected:?}"), "SelectedKey(<redacted>)");
        assert!(!format!("{:?}", selected.keys()).contains("[0, 1"));
        for (raw, current, members, now, age) in [
            (7, 2, BTreeSet::from([1, 2]), 31, 10), // expired; no fallback
            (8, 2, BTreeSet::from([1, 2]), 30, 10), // wrong raw ID
            (7, 2, BTreeSet::from([1]), 30, 10),    // removed companion
            (7, 2, BTreeSet::from([1, 2]), 19, 10), // future key (local guard)
            (7, 2, BTreeSet::new(), 30, 10),
        ] {
            assert!(
                store
                    .write(|tx| select_active_key(tx, raw, current, &members, now, age))
                    .unwrap()
                    .is_none()
            );
        }
        assert!(
            store
                .write(|tx| select_active_key(tx, 7, 3, &BTreeSet::from([1, 2, 3]), 30, 10))
                .unwrap()
                .is_some()
        );
        assert!(
            store
                .write(|tx| select_active_key(tx, 7, 2, &registered, 30, -1))
                .is_err()
        );
        assert_eq!(
            store.read(|tx| tx.scan(Table::Checkpoints, 1, 20)).unwrap(),
            before
        );
        store
            .write(|tx| {
                tx.put(
                    Table::Checkpoints,
                    b"app-state-key:000100000002",
                    1,
                    b"malformed",
                )
            })
            .unwrap();
        assert!(
            store
                .write(|tx| select_active_key(tx, 7, 2, &registered, 30, 10))
                .is_err()
        );
    }

    #[test]
    fn import_is_atomic_insert_only_and_persists_across_reopen() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
        };
        let dir = tempfile::tempdir().unwrap();
        let options = || OpenOptions {
            path: dir.path().join("keys.redb"),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([4; 32]))),
            create: true,
        };
        let store = ProtocolStore::open(options()).unwrap();
        let share = KeyShare::decode(FIXTURE).unwrap();
        let rollback: Result<()> = store.write(|tx| {
            assert_eq!(share.import(tx)?, 1);
            Err(Error::Cancelled)
        });
        assert!(rollback.is_err());
        assert_eq!(store.read(|tx| tx.count(Table::Checkpoints)).unwrap(), 0);
        assert_eq!(store.write(|tx| share.import(tx)).unwrap(), 1);
        drop(store);
        let store = ProtocolStore::open(options()).unwrap();
        assert_eq!(store.write(|tx| share.import(tx)).unwrap(), 0);
        let mut altered = KeyShare::decode(FIXTURE).unwrap();
        altered.keys[0].data[0] ^= 1;
        altered.keys[0].timestamp = 123;
        assert_eq!(store.write(|tx| altered.import(tx)).unwrap(), 0);
        let saved: StoredKey = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"app-state-key:000100000002", 1))
            .unwrap()
            .unwrap();
        assert_eq!(saved.data, b"test");
        assert_eq!(saved.timestamp, 9);
        assert_eq!(saved.fingerprint, share.keys[0].fingerprint);
    }

    #[test]
    fn only_top_level_type_six_selects_a_share() {
        let mut envelope = vec![0x62, 0x24, 0x10, 0x06, 0x3a, 0x20];
        envelope.extend_from_slice(FIXTURE);
        assert_eq!(
            KeyShare::from_peer_message(&envelope)
                .unwrap()
                .unwrap()
                .len(),
            1
        );
        envelope[3] = 4;
        assert!(KeyShare::from_peer_message(&envelope).unwrap().is_none());
        assert!(
            KeyShare::from_peer_message(b"\x0a\x04test")
                .unwrap()
                .is_none()
        );
        assert!(KeyShare::from_peer_message(b"\x62\x02\x10\x06").is_err());
        assert!(KeyShare::from_peer_message(&vec![0; 1_048_577]).is_err());
    }
    #[test]
    fn independent_key_share_fixture_and_redaction() {
        let share = KeyShare::decode(FIXTURE).unwrap();
        assert_eq!(share.len(), 1);
        assert!(share.same_keys(&KeyShare::decode(FIXTURE).unwrap()));
        assert!(!format!("{share:?}").contains("test"));
        let mut envelope = vec![0x62, 0x24, 0x10, 0x06, 0x3a, 0x20];
        envelope.extend_from_slice(FIXTURE);
        let summary = super::super::direct::payload_summary(&envelope).unwrap();
        assert!(summary.contains("1 structurally valid keys; inspection does not import"));
        assert!(!summary.contains("test"));
        let fp = share.fingerprints().next().unwrap();
        assert!(fp.matches_registered_devices(7, 2, &BTreeSet::from([1, 2])));
        assert!(fp.matches_registered_devices(7, 3, &BTreeSet::from([1, 2, 3])));
        assert!(!fp.matches_registered_devices(7, 3, &BTreeSet::from([2, 3])));
        assert!(!fp.matches_registered_devices(8, 2, &BTreeSet::from([1, 2])));
        assert!(!fp.matches_registered_devices(7, u32::MAX, &BTreeSet::from([1, 2])));
    }
    #[test]
    fn missing_fields_duplicate_ids_and_resource_bounds_are_rejected() {
        for bytes in [b"\x0a\x00".as_slice(), &FIXTURE[..FIXTURE.len() - 1]] {
            assert!(KeyShare::decode(bytes).is_err());
        }
        let doubled = [FIXTURE, FIXTURE].concat();
        assert!(KeyShare::decode(&doubled).is_err());
        assert!(KeyShare::decode(&vec![0; 65537]).is_err());
        let mut share = Share::decode(FIXTURE).unwrap();
        share.keys[0]
            .data
            .as_mut()
            .unwrap()
            .fingerprint
            .as_mut()
            .unwrap()
            .raw_id = None;
        assert!(KeyShare::decode(&share.encode_to_vec()).is_err());
    }
}
