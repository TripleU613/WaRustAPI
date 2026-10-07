//! Explicit-input offline fingerprint calculation. Never opens an account store.
use serde::Deserialize;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};
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
    lid_jid: Option<String>,
    pn_identifier: Option<String>,
    username_identifier: Option<String>,
    hosted_state: Option<fingerprint::qr::HostedState>,
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

fn parse(bytes: &[u8]) -> Result<Input, Error> {
    // Serde diagnostics can contain the offending value; never print them.
    let input: Input = serde_json::from_slice(bytes).map_err(|_| {
        invalid("invalid fingerprint JSON; requires local/remote sides and rejects unknown fields")
    })?;
    Ok(input)
}

fn display(input: &Input) -> Result<NativeDisplayPair, Error> {
    fingerprint::native_display_pair(
        &input.local.display_identifier,
        &decode_keys(&input.local)?,
        &input.remote.display_identifier,
        &decode_keys(&input.remote)?,
    )
}

#[cfg(test)]
fn calculate(bytes: &[u8]) -> Result<NativeDisplayPair, Error> {
    display(&parse(bytes)?)
}

fn read(path: &Path) -> Result<Zeroizing<Vec<u8>>, Error> {
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
    Ok(bytes)
}

fn qr_side<'a>(side: &'a Side, keys: &'a [[u8; 32]]) -> Result<fingerprint::qr::V4Side<'a>, Error> {
    let lid = side
        .lid_jid
        .as_deref()
        .ok_or_else(|| invalid("QR export requires lid_jid on each side"))?;
    let parsed = whatsapp_api::model::jid::Jid::parse(lid)
        .map_err(|_| invalid("QR export requires a plain primary LID JID on each side"))?;
    // The CLI supports the ordinary primary @lid subset, not all APK parser
    // variants. Display uses parsed user while native QR tag 3 retains full JID.
    if parsed.server != whatsapp_api::model::jid::Server::Lid
        || parsed.device != 0
        || parsed.agent != 0
        || parsed.kind != whatsapp_api::model::jid::JidKind::Plain
        // Our generic JID parser normalizes zero qualifiers to Plain; the
        // diagnostic's ordinary-JID subset requires the exact plain raw text.
        || parsed.to_string() != lid
        || parsed.user != side.display_identifier
    {
        return Err(invalid(
            "QR lid_jid must be plain @lid with user equal to display_identifier",
        ));
    }
    // APK 37.71 V4 comparison 0x7152fc..0x7155d8 rejects a side lacking
    // both PN and username, even when its LID/key match. Refuse an export
    // that cannot satisfy this presence prerequisite rather than guess a PN.
    if side.pn_identifier.is_none() && side.username_identifier.is_none() {
        return Err(invalid(
            "QR comparison requires pn_identifier or username_identifier on each side",
        ));
    }
    Ok(fingerprint::qr::V4Side {
        keys,
        lid_jid: lid,
        pn_identifier: side.pn_identifier.as_deref(),
        username_identifier: side.username_identifier.as_deref(),
        hosted_state: side.hosted_state,
    })
}

fn qr(input: &Input, version: u8) -> Result<Vec<u8>, Error> {
    let local_keys = decode_keys(&input.local)?;
    let remote_keys = decode_keys(&input.remote)?;
    fingerprint::qr::encode_v4(
        &qr_side(&input.local, &local_keys)?,
        &qr_side(&input.remote, &remote_keys)?,
        version,
    )
}

fn export(path: &Path, payload: &[u8]) -> Result<(), Error> {
    // Create-only, private binary export: version 0 includes public identities
    // and full contact JIDs. Never print protobuf/hex or overwrite an old QR.
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(payload)?;
    file.sync_all()?;
    Ok(())
}

pub(super) fn run(
    path: &Path,
    qr_version: Option<u8>,
    qr_output: Option<&Path>,
) -> Result<(), Error> {
    if qr_version.is_some() != qr_output.is_some() {
        return Err(invalid(
            "QR export requires both --qr-version and --qr-output",
        ));
    }
    // Parse once, so the displayed number and exported QR share the same inputs
    // even if an operator updates the config while this command is running.
    let bytes = read(path)?;
    let input = parse(&bytes)?;
    let result = display(&input)?;
    let mut value =
        serde_json::to_value(result).map_err(|_| invalid("cannot encode fingerprint result"))?;
    if let (Some(version), Some(output)) = (qr_version, qr_output) {
        let payload = qr(&input, version)?;
        export(output, &payload)?;
        value["qr_export"] = serde_json::json!({
            "format":"native_v4_protobuf", "version":version,
            "bytes":payload.len(), "private_file_created":true
        });
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&value)
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
    fn qr_payload_is_private_create_only_and_requires_matching_lid_display_inputs() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../whatsapp-api/fixtures/fingerprint/native-qr-3771.json"
        ))
        .unwrap();
        let mut value = input();
        for side in ["local", "remote"] {
            for field in [
                "lid_jid",
                "pn_identifier",
                "username_identifier",
                "raw_identity_keys_hex",
            ] {
                value[side][field] = fixture[side][field].clone();
            }
            value[side]["display_identifier"] = fixture[side]["lid_jid"]
                .as_str()
                .unwrap()
                .split('@')
                .next()
                .unwrap()
                .into();
        }
        value["local"]["hosted_state"] = "e2ee".into();
        value["remote"]["hosted_state"] = "hosted".into();
        let data = serde_json::to_vec(&value).unwrap();
        let config = parse(&data).unwrap();
        let path = std::env::temp_dir().join(format!(
            "telewhatsapp-fingerprint-qr-{}.pb",
            std::process::id()
        ));
        let payload = qr(&config, 0).unwrap();
        assert_eq!(hex::encode(&payload), fixture["version_0_hex"]);
        export(&path, &payload).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert!(export(&path, b"overwrite refused").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), payload);
        std::fs::remove_file(path).unwrap();
        value["local"]["display_identifier"] = "secret-sentinel".into();
        let config = parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            !qr(&config, 0)
                .unwrap_err()
                .to_string()
                .contains("secret-sentinel")
        );
        value["local"]["display_identifier"] = "222222222222222".into();
        for jid in [
            "222222222222222:0@lid",
            "222222222222222:2@lid",
            "222222222222222@hosted.lid",
            "222222222222222@s.whatsapp.net",
        ] {
            value["local"]["lid_jid"] = jid.into();
            assert!(qr(&parse(&serde_json::to_vec(&value).unwrap()).unwrap(), 0).is_err());
        }
        assert!(qr(&parse(&serde_json::to_vec(&input()).unwrap()).unwrap(), 0).is_err());
        value["local"]["hosted_state"] = "secret-sentinel".into();
        assert!(
            !parse(&serde_json::to_vec(&value).unwrap())
                .err()
                .unwrap()
                .to_string()
                .contains("secret-sentinel")
        );
    }

    #[test]
    fn qr_cli_flags_require_pairing_and_known_version() {
        use clap::Parser;
        let base = [
            "whatsapp-cli",
            "fingerprint-display",
            "--config",
            "private.json",
        ];
        assert!(crate::Cli::try_parse_from(base).is_ok());
        assert!(crate::Cli::try_parse_from(base.into_iter().chain(["--qr-version", "0"])).is_err());
        assert!(
            crate::Cli::try_parse_from(base.into_iter().chain(["--qr-output", "private.pb"]))
                .is_err()
        );
        assert!(
            crate::Cli::try_parse_from(base.into_iter().chain([
                "--qr-version",
                "2",
                "--qr-output",
                "private.pb"
            ]))
            .is_err()
        );
        assert!(
            crate::Cli::try_parse_from(base.into_iter().chain([
                "--qr-version",
                "1",
                "--qr-output",
                "private.pb"
            ]))
            .is_ok()
        );
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
