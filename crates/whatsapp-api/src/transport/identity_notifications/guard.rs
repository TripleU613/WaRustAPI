//! Local snapshot guards for APK-derived notification key reads, not native admission.
use super::super::{devices, identity_conflicts};
use crate::{
    Error, Result,
    model::{ids::EventId, jid::Jid},
    store::{ReadTx, Table, WriteTx},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Relationship to the identity saved when this notification read was queued.
/// Matching guards never establish native admission or authorize a store write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshGuard {
    /// Historical read has no original snapshot; never backfilled.
    Untracked,
    /// Pin, exact-device transition, user primary counter and generation match.
    Matches,
    /// The saved identity bytes changed or disappeared after queueing.
    PinChanged,
    /// An authenticated exact-device transition occurred, including A->B->A.
    TransitionChanged,
    /// This user's primary identity counter changed in the same namespace.
    PrimaryVersionChanged,
    /// The account generation differs from the original snapshot.
    GenerationChanged,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    peer: Jid,
    pin_fingerprint: [u8; 32],
    generation: u32,
    // Absence of these JSON members is corruption, distinct from captured null.
    #[serde(deserialize_with = "required_option")]
    transition: Option<EventId>,
    #[serde(deserialize_with = "required_option")]
    primary_version: Option<i64>,
}
fn required_option<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
fn key(id: &str) -> String {
    format!("identity-refresh-snapshot:{id}")
}
fn invalid() -> Error {
    Error::invalid("identity_notification", "invalid saved refresh snapshot")
}

/// Save inside the same transaction as the original action and event markers.
/// A duplicate event never reaches this function or overwrites its old baseline.
pub(super) fn capture_in(tx: &mut WriteTx<'_>, id: &str, peer: &Jid, pin: &[u8]) -> Result<()> {
    if tx.get(Table::Checkpoints, key(id).as_bytes(), 1)?.is_some() {
        return Err(invalid());
    }
    let snapshot = Snapshot {
        peer: peer.clone(),
        pin_fingerprint: Sha256::digest(pin).into(),
        generation: tx.generation().0,
        transition: identity_conflicts::transition_write_in(tx, peer)?,
        // Native 37.71 0Z7.A00/A02 tracks primary changes per exact PN/LID
        // user. Keep it separate from the new local exact-device event marker.
        primary_version: devices::primary_device_version_write_in(tx, &peer.to_non_ad())?,
    };
    tx.put_json(Table::Checkpoints, key(id).as_bytes(), 1, &snapshot)
}

pub(super) fn inspect_in(
    tx: &ReadTx<'_>,
    id: &str,
    peer: &Jid,
    pin: Option<&[u8]>,
) -> Result<RefreshGuard> {
    let Some(bytes) = tx.get(Table::Checkpoints, key(id).as_bytes(), 1)? else {
        return Ok(RefreshGuard::Untracked);
    };
    let snapshot: Snapshot = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if snapshot.peer != *peer {
        return Err(invalid());
    }
    if snapshot.generation != tx.generation().0 {
        return Ok(RefreshGuard::GenerationChanged);
    }
    if pin.map(|pin| <[u8; 32]>::from(Sha256::digest(pin))) != Some(snapshot.pin_fingerprint) {
        return Ok(RefreshGuard::PinChanged);
    }
    if identity_conflicts::transition_in(tx, peer)? != snapshot.transition {
        return Ok(RefreshGuard::TransitionChanged);
    }
    if devices::primary_device_version_in(tx, &peer.to_non_ad())? != snapshot.primary_version {
        return Ok(RefreshGuard::PrimaryVersionChanged);
    }
    Ok(RefreshGuard::Matches)
}
