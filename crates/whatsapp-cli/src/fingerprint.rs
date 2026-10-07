//! Explicit-input offline fingerprint calculation. Never opens an account store.
use serde::Deserialize;
use std::{fs::File, io::Read, os::unix::fs::PermissionsExt, path::Path};
use whatsapp_api::{
    Error,
    fingerprint::{self, NativeDisplayPair},
};
use zeroize::Zeroizing;

const MAX_CONFIG_BYTES: u64 = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Side {
    display_identifier: String,
    raw_identity_keys_hex: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    local: Side,
    remote: Side,
}

fn invalid(message: &'static str) -> Error {
    Error::invalid("fingerprint_config", message)
}

fn decode_keys(side: &Side) -> Result<Vec<[u8; 32]>, Error> {
    if side.raw_identity_keys_hex.is_empty()
        || side.raw_identity_keys_hex.len() > fingerprint::MAX_IDENTITY_KEYS
    {
        return Err(invalid(
            "each side requires 1..128 raw public identity keys",
        ));
    }
    side.raw_identity_keys_hex
        .iter()
        .map(|key| {
            if key.len() != 64 {
                return Err(invalid(
                    "each raw public identity key must be exactly 64 hex characters",
                ));
            }
            let mut decoded = [0u8; 32];
            hex::decode_to_slice(key, &mut decoded)
                .map_err(|_| invalid("invalid raw public identity key hex"))?;
            Ok(decoded)
        })
        .collect()
}

fn calculate(bytes: &[u8]) -> Result<NativeDisplayPair, Error> {
    // Serde diagnostics can contain the offending value; never print them.
    let input: Input = serde_json::from_slice(bytes).map_err(|_| {
        invalid("invalid fingerprint JSON; requires local/remote sides and rejects unknown fields")
    })?;
    fingerprint::native_display_pair(
        &input.local.display_identifier,
        &decode_keys(&input.local)?,
        &input.remote.display_identifier,
        &decode_keys(&input.remote)?,
    )
}

fn read(path: &Path) -> Result<NativeDisplayPair, Error> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_CONFIG_BYTES {
        return Err(invalid("config must be a regular file at most 64 KiB"));
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(invalid(
            "config contains private identifiers; remove group/other permissions (chmod 600)",
        ));
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err(invalid("config exceeds 64 KiB"));
    }
    calculate(&bytes)
}

pub(super) fn run(path: &Path) -> Result<(), Error> {
    let result = read(path)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&result)
            .map_err(|_| invalid("cannot encode fingerprint result"))?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> serde_json::Value {
        serde_json::json!({
            "local":{"display_identifier":"synthetic-local", "raw_identity_keys_hex":["10".repeat(32),"01".repeat(32)]},
            "remote":{"display_identifier":"synthetic-remote", "raw_identity_keys_hex":[hex::encode((0u8..32).collect::<Vec<_>>())]}
        })
    }

    #[test]
    fn private_inputs_never_appear_in_result_or_parse_errors() {
        let bytes = serde_json::to_vec(&input()).unwrap();
        let result = calculate(&bytes).unwrap();
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(
            result.combined,
            "128588181678439595160634291930622605791988983247407933286410"
        );
        assert!(!json.contains("synthetic"));
        assert!(!json.contains(&"10".repeat(32)));
        let mut bad = input();
        bad["local"]["secret-sentinel"] = true.into();
        assert!(
            !calculate(&serde_json::to_vec(&bad).unwrap())
                .err()
                .unwrap()
                .to_string()
                .contains("secret-sentinel")
        );
        bad = input();
        bad["remote"]["raw_identity_keys_hex"][0] = "secret-sentinel".into();
        assert!(
            !calculate(&serde_json::to_vec(&bad).unwrap())
                .err()
                .unwrap()
                .to_string()
                .contains("secret-sentinel")
        );
        bad["remote"]["raw_identity_keys_hex"][0] = "zz".repeat(32).into();
        assert!(calculate(&serde_json::to_vec(&bad).unwrap()).is_err());
    }

    #[test]
    fn file_reader_requires_private_bounded_regular_file() {
        let path = std::env::temp_dir().join(format!(
            "telewhatsapp-fingerprint-{}.json",
            std::process::id()
        ));
        let bytes = serde_json::to_vec(&input()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read(&path).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(read(&path).is_ok());
        std::fs::write(&path, vec![b' '; (MAX_CONFIG_BYTES + 1) as usize]).unwrap();
        assert!(read(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(read(&std::env::temp_dir()).is_err());
    }
}
