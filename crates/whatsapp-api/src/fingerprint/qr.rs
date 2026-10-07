//! Native V4 combined fingerprint protobuf serialization, APK 2.26.37.71.
//! This exports comparison data, not a QR image, authentication, scheme selection
//! or a human-verification result. Evidence: docs/protocol/safety-number.md.
use super::{MAX_DISPLAY_IDENTIFIER_BYTES, serialized_keys};
use crate::{Error, Result};
use prost::Message;
use serde::Deserialize;
use sha2::{Digest, Sha512};

/// Native QR hosted-state enum; distinct from saved identity attestation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostedState {
    /// Native descriptor E2EE=0.
    E2ee,
    /// Native descriptor HOSTED=1.
    Hosted,
}

/// Explicit caller-supplied native V4 individual inputs.
/// Keys and full JID strings are private comparison data; Debug redacts them.
pub struct V4Side<'a> {
    /// Selected raw public-key inventory, not inferred from a primary session.
    pub keys: &'a [[u8; 32]],
    /// Full ASCII LID JID used in QR tag 3, not its parsed display user.
    pub lid_jid: &'a str,
    /// Already normalized stable PN identifier, when selected by the caller.
    pub pn_identifier: Option<&'a str>,
    /// Caller-selected username identifier; presence can suppress local PN.
    pub username_identifier: Option<&'a str>,
    /// None represents the APK's absent/sentinel state; never encoded as -1.
    pub hosted_state: Option<HostedState>,
}
impl std::fmt::Debug for V4Side<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V4Side")
            .field("key_count", &self.keys.len())
            .field("has_pn", &self.pn_identifier.is_some())
            .field("has_username", &self.username_identifier.is_some())
            .field("hosted_state", &self.hosted_state)
            .finish_non_exhaustive()
    }
}

// 37.71 merged descriptor 0x740b60, fields 0x740c80..0x740de8;
// combined descriptor 0x740c08. These are the recovered native schema,
// not Java legacy M76/BTe whose nested tag 2 has different meaning.
#[derive(Clone, PartialEq, Message)]
struct Individual {
    #[prost(bytes = "vec", optional, tag = "1")]
    public_key: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pn_identifier: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "3")]
    lid_identifier: Option<Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "4")]
    username_identifier: Option<Vec<u8>>,
    // Native HostedState descriptor 0x740f58 declares only 0/1.
    // These nonnegative enum members have int32 varint wire encoding; the
    // typed encoder below cannot emit an unknown enum or -1 sentinel.
    #[prost(int32, optional, tag = "5")]
    hosted_state: Option<i32>,
    #[prost(bytes = "vec", optional, tag = "6")]
    hashed_public_key: Option<Vec<u8>>,
}
#[derive(Clone, PartialEq, Message)]
struct Combined {
    #[prost(uint32, optional, tag = "1")]
    version: Option<u32>,
    #[prost(message, optional, tag = "2")]
    local: Option<Individual>,
    #[prost(message, optional, tag = "3")]
    remote: Option<Individual>,
}

fn ascii(value: &str) -> Result<Vec<u8>> {
    // MCF ASCII validation 37.71 merged 0x531200 accepts bytes 1..127,
    // including an empty value. Length bound is our resource policy.
    if value.len() > MAX_DISPLAY_IDENTIFIER_BYTES
        || !value.bytes().all(|byte| (1..=127).contains(&byte))
    {
        return Err(Error::invalid(
            "fingerprint.qr_identifier",
            "requires ASCII without NUL, at most 256 bytes",
        ));
    }
    Ok(value.as_bytes().to_vec())
}

fn individual(side: &V4Side<'_>, version: u8, local: bool) -> Result<Individual> {
    if side.lid_jid.is_empty() {
        return Err(Error::invalid(
            "fingerprint.lid_jid",
            "full LID JID input is missing",
        ));
    }
    let keys = serialized_keys(side.keys)?;
    let mut public_key = None;
    let mut hashed_public_key = None;
    if version == 0 {
        public_key = Some(keys);
    } else {
        // Native create 0x714764..0x71476c stores H(K,K), not the display
        // digest. 0x713938 hashes seed||K then 5199 digest||K rounds.
        let mut hash = Sha512::new();
        hash.update(&keys);
        hash.update(&keys);
        let mut digest = hash.finalize();
        for _ in 1..5200 {
            let mut hash = Sha512::new();
            hash.update(digest);
            hash.update(&keys);
            digest = hash.finalize();
        }
        hashed_public_key = Some(digest.to_vec());
    }
    let pn = side.pn_identifier.map(ascii).transpose()?;
    let username = side.username_identifier.map(ascii).transpose()?;
    Ok(Individual {
        public_key,
        // Main 0x715050..0x71506c suppresses PN with present username
        // only when flag bit 0 is set. Combined 0x714974..0x714998 passes
        // flag 1 for local and flag 0 for remote; do not suppress both.
        pn_identifier: if local && username.is_some() {
            None
        } else {
            pn
        },
        lid_identifier: Some(ascii(side.lid_jid)?),
        username_identifier: username,
        hosted_state: side.hosted_state.map(|state| match state {
            HostedState::E2ee => 0,
            HostedState::Hosted => 1,
        }),
        hashed_public_key,
    })
}

/// Encode native V4 combined fingerprint comparison bytes for version 0 or 1.
///
/// Version 0 carries sorted serialized keys; version 1 their identifier-free
/// digest. Full LID/optional identifier bytes are preserved without normalization.
/// Callers must provide actual chosen inputs; this function does not validate
/// APK JID grammar, attest device membership, choose the active scheme, render a
/// QR image, compare a peer payload or commit verification. Optional identifiers
/// can be empty as in the native ASCII primitive. Serialization alone does not
/// establish that comparison requirements (including PN/username) are satisfied.
pub fn encode_v4(local: &V4Side<'_>, remote: &V4Side<'_>, version: u8) -> Result<Vec<u8>> {
    if version > 1 {
        return Err(Error::invalid(
            "fingerprint.qr_version",
            "native QR version must be 0 or 1",
        ));
    }
    Ok(Combined {
        // Explicit Some(0) preserves the mandatory version presence bit.
        version: Some(u32::from(version)),
        local: Some(individual(local, version, true)?),
        remote: Some(individual(remote, version, false)?),
    }
    .encode_to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apk_wire_fixtures_preserve_version_keys_orientation_and_optional_presence() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/fingerprint/native-qr-3771.json"
        ))
        .unwrap();
        let keys = |name: &str| -> Vec<[u8; 32]> {
            fixture[name]["raw_identity_keys_hex"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| {
                    hex::decode(x.as_str().unwrap())
                        .unwrap()
                        .try_into()
                        .unwrap()
                })
                .collect()
        };
        let local_keys = keys("local");
        let remote_keys = keys("remote");
        let local = V4Side {
            keys: &local_keys,
            lid_jid: "222222222222222@lid",
            pn_identifier: Some("15550100"),
            username_identifier: Some("synthetic-local"),
            hosted_state: Some(HostedState::E2ee),
        };
        let remote = V4Side {
            keys: &remote_keys,
            lid_jid: "333333333333333@lid",
            pn_identifier: Some("15550101"),
            username_identifier: Some("synthetic-remote"),
            hosted_state: Some(HostedState::Hosted),
        };
        for version in 0..=1 {
            let bytes = encode_v4(&local, &remote, version).unwrap();
            assert_eq!(
                hex::encode(&bytes),
                fixture[format!("version_{version}_hex")]
            );
            let parsed = Combined::decode(bytes.as_slice()).unwrap();
            assert_eq!(parsed.version, Some(u32::from(version)));
            let first = parsed.local.unwrap();
            let second = parsed.remote.unwrap();
            assert_eq!(first.pn_identifier, None);
            assert_eq!(second.pn_identifier, Some(b"15550101".to_vec()));
            assert_eq!(
                first.lid_identifier,
                Some(local.lid_jid.as_bytes().to_vec())
            );
            assert_eq!(first.hosted_state, Some(0));
            assert_eq!(second.hosted_state, Some(1));
            assert_eq!(first.public_key.is_some(), version == 0);
            assert_eq!(
                first.hashed_public_key.as_ref().map(Vec::len),
                if version == 1 { Some(64) } else { None }
            );
        }
        assert!(!format!("{local:?}").contains("222222"));
        assert!(!format!("{local:?}").contains("15550100"));
    }

    #[test]
    fn missing_unknown_or_invalid_inputs_refuse_and_absence_differs_from_empty() {
        let keys = [[1u8; 32]];
        let mut side = V4Side {
            keys: &keys,
            lid_jid: "222222222222222@lid",
            pn_identifier: Some(""),
            username_identifier: None,
            hosted_state: None,
        };
        let bytes = encode_v4(&side, &side, 0).unwrap();
        let parsed = Combined::decode(bytes.as_slice()).unwrap().local.unwrap();
        assert_eq!(parsed.pn_identifier, Some(vec![]));
        assert_eq!(parsed.username_identifier, None);
        assert_eq!(parsed.hosted_state, None);
        side.username_identifier = Some("");
        let parsed = Combined::decode(encode_v4(&side, &side, 0).unwrap().as_slice()).unwrap();
        assert_eq!(parsed.local.unwrap().pn_identifier, None);
        assert_eq!(parsed.remote.unwrap().pn_identifier, Some(vec![]));
        assert!(encode_v4(&side, &side, 2).is_err());
        side.lid_jid = "";
        assert!(encode_v4(&side, &side, 0).is_err());
        side.lid_jid = "secret\0sentinel";
        assert!(
            !encode_v4(&side, &side, 0)
                .unwrap_err()
                .to_string()
                .contains("sentinel")
        );
        side.lid_jid = "222222222222222@lid";
        side.pn_identifier = Some("secret\u{80}sentinel");
        assert!(encode_v4(&side, &side, 0).is_err());
        side.pn_identifier = None;
        side.keys = &[];
        assert!(encode_v4(&side, &side, 0).is_err());
    }
}
