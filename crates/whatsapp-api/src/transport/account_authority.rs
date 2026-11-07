//! Local queued-work account ownership, not a wire field or replay authority.
//! Excludes observation-only timestamps; includes identity private bytes only
//! as hash input. No raw key or metadata is returned or logged.
use crate::Result;

pub(super) fn fingerprint(
    generation: crate::model::ids::AccountGeneration,
    account: Option<&crate::session::AccountRecord>,
) -> Result<[u8; 32]> {
    use sha2::{Digest, Sha256};
    let metadata = serde_json::to_vec(&(
        generation,
        account.map(|a| {
            (
                a.generation,
                &a.phone,
                &a.own_jid,
                a.registration,
                a.signal_registration_id,
            )
        }),
    ))
    .map_err(|_| crate::Error::invalid("transport", "account authority serialization"))?;
    let mut hash = Sha256::new();
    hash.update(b"WaRustAPI transport account authority v1\0");
    hash.update((metadata.len() as u64).to_be_bytes());
    hash.update(metadata);
    if let Some(identity) = account.and_then(|a| a.signal_identity.as_ref()) {
        hash.update([1]);
        hash.update(identity.0.public);
        hash.update(identity.0.private);
    } else {
        hash.update([0]);
    }
    Ok(hash.finalize().into())
}
