//! Private explicit local human decision; no peer key selection or network.
use serde::Deserialize;
use std::{fs::File, io::Read, os::unix::fs::PermissionsExt, path::Path, sync::Arc};
use whatsapp_api::{
    Error,
    model::{
        ids::AccountId,
        jid::{Jid, JidKind, Server},
    },
    store::{EnvKeySource, OpenOptions, ProtocolStore},
};
use zeroize::Zeroizing;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    peer: String,
    expected_fingerprint: String,
    expected_action_sequence: u64,
    verified: bool,
}
fn invalid(reason: &'static str) -> Error {
    Error::invalid("peer_verification_config", reason)
}
fn parse(bytes: &[u8]) -> Result<(Decision, Jid), Error> {
    let input: Decision = serde_json::from_slice(bytes).map_err(|_| invalid("invalid decision JSON; requires peer, expected_fingerprint, expected_action_sequence, verified; unknown fields refused"))?;
    let peer = Jid::parse(&input.peer).map_err(|_| invalid("peer must be a plain primary LID"))?;
    if peer.server != Server::Lid
        || peer.kind != JidKind::Plain
        || peer.device != 0
        || peer.agent != 0
        || peer.user.is_empty()
        || peer.to_string() != input.peer
    {
        return Err(invalid("peer must be a plain primary LID"));
    }
    if input.expected_fingerprint.len() != 64
        || hex::decode(&input.expected_fingerprint).is_err()
        || input.expected_action_sequence > i64::MAX as u64
    {
        return Err(invalid(
            "requires SHA-256 hex fingerprint and action sequence in signed int64 range",
        ));
    }
    Ok((input, peer))
}
fn read(path: &Path) -> Result<Zeroizing<Vec<u8>>, Error> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > 65536 {
        return Err(invalid("config must be a regular file at most 64 KiB"));
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(invalid(
            "private config requires no group/other permissions (chmod 600)",
        ));
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return Err(invalid("config exceeds 64 KiB"));
    }
    Ok(bytes)
}
pub(super) fn run(cli: &super::Cli, path: &Path) -> Result<(), Error> {
    let (input, peer) = parse(&read(path)?)?;
    let store = ProtocolStore::open(OpenOptions {
        path: cli.store.clone(),
        account_id: AccountId::new(cli.account.clone())?,
        key_source: Arc::new(EnvKeySource::new(cli.key_env.clone())),
        create: false,
    })?;
    let state = whatsapp_api::transport::peer_verification::set_local(
        &store,
        &peer,
        &input.expected_fingerprint,
        input.expected_action_sequence,
        input.verified,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&state)
            .map_err(|_| invalid("cannot encode decision result"))?
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_decision_validation_refuses_aliases_and_redacts_errors() {
        let mut input = serde_json::json!({"peer":"222222222222222@lid", "expected_fingerprint":"ab".repeat(32), "expected_action_sequence":0, "verified":true});
        assert!(parse(&serde_json::to_vec(&input).unwrap()).is_ok());
        for peer in [
            "secret-sentinel",
            "222222222222222:0@lid",
            "222222222222222:2@lid",
            "222222222222222@s.whatsapp.net",
        ] {
            input["peer"] = peer.into();
            assert!(
                !parse(&serde_json::to_vec(&input).unwrap())
                    .err()
                    .unwrap()
                    .to_string()
                    .contains(peer)
            );
        }
        input["peer"] = "222222222222222@lid".into();
        input["verified"] = "secret-sentinel".into();
        assert!(
            !parse(&serde_json::to_vec(&input).unwrap())
                .err()
                .unwrap()
                .to_string()
                .contains("secret-sentinel")
        );
    }
    #[test]
    fn decision_file_is_private_and_bounded() {
        let path =
            std::env::temp_dir().join(format!("telewhatsapp-decision-{}.json", std::process::id()));
        std::fs::write(&path, b"{}").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read(&path).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(read(&path).is_ok());
        std::fs::write(&path, vec![0; 65537]).unwrap();
        assert!(read(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
