//! APK 11T.A08 reconciliation of DeviceJid -> ADV key-index maps.
use super::*;
use crate::model::jid::{Jid, Server};
use std::collections::BTreeMap;

/// Comparison Avv.A03 device-address hash branches. Runtime flag 0x415b
/// selects the APK branch; callers must supply the established version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceHashVersion {
    /// `2:` hash of sorted `user.0:device@server` strings.
    V2,
    /// `3:` hash of sorted `user:device@server` strings.
    V3,
}

/// Candidate membership map, not persisted authority or Signal session state.
/// Inputs must already come from authenticated account-scoped device metadata.
#[derive(Clone, PartialEq, Eq)]
pub struct ReconciledDevices {
    devices: BTreeMap<Jid, u32>,
}
impl std::fmt::Debug for ReconciledDevices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconciledDevices")
            .field("device_count", &self.devices.len())
            .finish_non_exhaustive()
    }
}
impl ReconciledDevices {
    /// Retained advertised devices and their ADV indexes. This does not contain
    /// or authorize identity keys; never equate an ADV index with JID.device.
    pub fn devices(&self) -> &BTreeMap<Jid, u32> {
        &self.devices
    }

    /// Notification device-list hash (0bN.A0I -> Avv.A03), not an identity-key
    /// digest or signature. Includes the primary device, even when omitted from
    /// the companion map. A match alone must never authorize new identity keys.
    pub fn device_hash(&self, owner: &Jid, version: DeviceHashVersion) -> Result<String> {
        use sha2::{Digest, Sha256};
        if owner.agent != 0
            || owner.device != 0
            || owner.user.is_empty()
            || !owner.user.is_ascii()
            || !matches!(owner.server, Server::User | Server::Lid)
            || self.devices.len() > 1024
        {
            return Err(bad());
        }
        let (prefix, agent) = match version {
            DeviceHashVersion::V2 => ("2:", ".0"),
            DeviceHashVersion::V3 => ("3:", ""),
        };
        let address = |device| format!("{}{agent}:{device}@{}", owner.user, owner.server.as_str());
        let mut addresses = BTreeSet::from([address(0)]);
        for jid in self.devices.keys() {
            if jid.user != owner.user || jid.server != owner.server || jid.agent != 0 {
                return Err(bad());
            }
            addresses.insert(address(jid.device));
        }
        // Canonical addresses are ASCII: Rust lexicographic order matches Java
        // String ordering. Numeric device order would differ for 2 and 10.
        let mut digest = Sha256::new();
        for address in addresses {
            digest.update(address.as_bytes());
        }
        let digest = digest.finalize();
        Ok(format!(
            "{prefix}{}",
            crate::registration::token::standard_base64_padded(&digest[..6])
        ))
    }

    /// Full SHA-256 identity-key digest from comparison 11T.A05. Keys must be
    /// resolved from authenticated/pinned identity state, including our own key.
    /// No missing-device omission or identity lookup from the incoming list.
    /// The APK truncates this using a runtime-configured length; this method
    /// deliberately does not guess that length or claim an advertised hash match.
    pub fn identity_hash(
        &self,
        mut pinned_key: impl FnMut(&Jid) -> Result<[u8; 32]>,
    ) -> Result<[u8; 32]> {
        use sha2::{Digest, Sha256};
        if self.devices.len() < 2 {
            return Err(bad());
        }
        let mut keys = self
            .devices
            .keys()
            .map(&mut pinned_key)
            .collect::<Result<Vec<_>>>()?;
        // DMq.compare masks each Java byte with 0xff: unsigned lexicographic
        // byte ordering, not JID ordering, signed-byte order, or deduplication.
        keys.sort_unstable();
        let mut hash = Sha256::new();
        for key in keys {
            hash.update(key);
        }
        Ok(hash.finalize().into())
    }
}

impl VerifiedIndexList {
    /// Reconcile authenticated device maps using comparison 11T.A08: filter
    /// incoming companions to positive signed indexes, retain incoming primary,
    /// preserve locally newer allocations, and prevent a same-device downgrade
    /// when the previous index remains valid. An absent old device within this
    /// list's allocation range is removed from the candidate only, not from disk.
    ///
    /// `expected_raw_id` is the saved account baseline. Raw-ID replacement is
    /// refused; broader APK reset/recovery branches require separate handling.
    /// One address domain/account per call; PN/LID alias reconciliation is not
    /// inferred. Bounds and duplicate-index rejection are local defensive policy.
    pub fn reconcile_devices(
        &self,
        owner: &Jid,
        expected_raw_id: u32,
        current: &BTreeMap<Jid, u32>,
        incoming: &BTreeMap<Jid, u32>,
    ) -> Result<ReconciledDevices> {
        if expected_raw_id != self.raw_id
            || owner.device != 0
            || owner.agent != 0
            || owner.user.is_empty()
            || !matches!(owner.server, Server::User | Server::Lid)
            || current.len() > 1024
            || incoming.len() > 1024
        {
            return Err(bad());
        }
        for devices in [current, incoming] {
            let mut indexes = BTreeSet::new();
            for (jid, index) in devices {
                if jid.user != owner.user
                    || jid.server != owner.server
                    || jid.agent != 0
                    || !indexes.insert(*index)
                {
                    return Err(bad());
                }
            }
        }
        let mut devices = BTreeMap::new();
        for (jid, index) in incoming {
            if jid.device == 0 || (*index > 0 && self.indexes.contains(index)) {
                devices.insert(jid.clone(), *index);
            }
        }
        for (jid, index) in current {
            if *index > self.current_index
                || incoming
                    .get(jid)
                    .is_some_and(|next| index > next && self.indexes.contains(index))
            {
                devices.insert(jid.clone(), *index);
            }
        }
        // Individually consistent old/new maps must not introduce a collision
        // through preservation of a newer local allocation.
        let indexes: BTreeSet<_> = devices.values().copied().collect();
        if indexes.len() != devices.len() || devices.len() > 1024 {
            return Err(bad());
        }
        Ok(ReconciledDevices { devices })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn jid(device: u16) -> Jid {
        Jid::parse(&format!("15550000000:{device}@s.whatsapp.net")).unwrap()
    }
    fn list() -> VerifiedIndexList {
        VerifiedIndexList {
            raw_id: 7,
            timestamp: 9,
            current_index: 3,
            indexes: BTreeSet::from([1, 2, 3]),
        }
    }
    #[test]
    fn device_hash_vectors_include_primary_and_sort_addresses_not_device_numbers() {
        for (server, v2, v3) in [
            (Server::User, "2:4EG9dWqA", "3:xpe8Jb4d"),
            (Server::Lid, "2:672CCHH7", "3:sA6SR/XE"),
        ] {
            let owner = Jid::new("15550000000", server.clone()).unwrap();
            let mut candidate = ReconciledDevices {
                devices: BTreeMap::from([
                    (
                        Jid::new_ad("15550000000", 0, 2, server.clone()).unwrap(),
                        17,
                    ),
                    (Jid::new_ad("15550000000", 0, 10, server).unwrap(), 19),
                ]),
            };
            // Independent Python hashlib/base64 vectors over explicit address strings.
            assert_eq!(
                candidate
                    .device_hash(&owner, DeviceHashVersion::V2)
                    .unwrap(),
                v2
            );
            assert_eq!(
                candidate
                    .device_hash(&owner, DeviceHashVersion::V3)
                    .unwrap(),
                v3
            );
            candidate.devices.insert(owner.clone(), 0);
            assert_eq!(
                candidate
                    .device_hash(&owner, DeviceHashVersion::V3)
                    .unwrap(),
                v3
            );
            let other = Jid::new("15550000001", owner.server.clone()).unwrap();
            assert!(
                candidate
                    .device_hash(&other, DeviceHashVersion::V3)
                    .is_err()
            );
            candidate
                .devices
                .remove(&Jid::new_ad("15550000000", 0, 2, owner.server.clone()).unwrap());
            assert_ne!(
                candidate
                    .device_hash(&owner, DeviceHashVersion::V3)
                    .unwrap(),
                v3
            );
        }
    }

    #[test]
    fn identity_hash_uses_unsigned_key_sort_and_requires_every_key() {
        let devices = ReconciledDevices {
            devices: BTreeMap::from([(jid(0), 0), (jid(10), 1), (jid(20), 2)]),
        };
        let key = |jid: &Jid| {
            Ok(match jid.device {
                0 => [128; 32],
                10 => [127; 32],
                _ => [0; 32],
            })
        };
        // Independent Python hashlib over sorted raw 32-byte synthetic keys.
        assert_eq!(
            hex::encode(devices.identity_hash(key).unwrap()),
            "4fb84084dcbd0d21a9042ab8c700feaa698d4a6b4711c7c3c78eb47c009360de"
        );
        assert!(
            devices
                .identity_hash(|jid| if jid.device == 10 {
                    Err(bad())
                } else {
                    key(jid)
                })
                .is_err()
        );
        let different = devices
            .identity_hash(|jid| {
                if jid.device == 20 {
                    Ok([1; 32])
                } else {
                    key(jid)
                }
            })
            .unwrap();
        assert_ne!(different, devices.identity_hash(key).unwrap());
        let single = ReconciledDevices {
            devices: BTreeMap::from([(jid(0), 0)]),
        };
        assert!(single.identity_hash(key).is_err());
    }
    #[test]
    fn filters_removed_indexes_and_preserves_newer_local_allocations() {
        let current = BTreeMap::from([(jid(0), 0), (jid(10), 2), (jid(20), 4), (jid(30), 3)]);
        let incoming = BTreeMap::from([(jid(0), 0), (jid(10), 1), (jid(40), 3), (jid(50), 99)]);
        let before = current.clone();
        let result = list()
            .reconcile_devices(&jid(0), 7, &current, &incoming)
            .unwrap();
        assert_eq!(
            result.devices(),
            &BTreeMap::from([(jid(0), 0), (jid(10), 2), (jid(20), 4), (jid(40), 3)])
        );
        assert_eq!(current, before);
        assert!(!format!("{result:?}").contains("15550000000"));
        // The index is 2, not the transport device number 10.
        assert_eq!(result.devices()[&jid(10)], 2);
    }
    #[test]
    fn zero_companions_and_unsigned_old_indexes_are_not_retained() {
        let current = BTreeMap::from([(jid(1), 1)]);
        let incoming = BTreeMap::from([(jid(2), 0), (jid(3), 2)]);
        let result = list()
            .reconcile_devices(&jid(0), 7, &current, &incoming)
            .unwrap();
        assert_eq!(result.devices(), &BTreeMap::from([(jid(3), 2)]));
        assert!(
            list()
                .reconcile_devices(&jid(0), 8, &current, &incoming)
                .is_err()
        );
    }
    #[test]
    fn account_substitution_duplicate_indexes_and_merge_collisions_fail() {
        let empty = BTreeMap::new();
        for wrong in [
            "15550000001:1@s.whatsapp.net",
            "15550000000:1@lid",
            "15550000000:1@g.us",
        ] {
            let incoming = BTreeMap::from([(Jid::parse(wrong).unwrap(), 1)]);
            assert!(
                list()
                    .reconcile_devices(&jid(0), 7, &empty, &incoming)
                    .is_err()
            );
        }
        assert!(
            list()
                .reconcile_devices(
                    &jid(0),
                    7,
                    &empty,
                    &BTreeMap::from([(jid(1), 1), (jid(2), 1)])
                )
                .is_err()
        );
        let current = BTreeMap::from([(jid(1), 3)]);
        let incoming = BTreeMap::from([(jid(1), 1), (jid(2), 3)]);
        assert!(
            list()
                .reconcile_devices(&jid(0), 7, &current, &incoming)
                .is_err()
        );
        assert!(
            list()
                .reconcile_devices(&jid(1), 7, &empty, &empty)
                .is_err()
        );
    }
}
