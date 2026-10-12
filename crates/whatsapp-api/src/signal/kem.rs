//! Experimental ML-KEM-1024 primitive; no hybrid session or directory admission.
//! APK/NIST comparison and limits: docs/protocol/native-kem-vectors.md.

use crate::{Error, Result};
use ml_kem::{
    Ciphertext, Decapsulate, DecapsulationKey1024, Encapsulate, EncapsulationKey1024,
    ExpandedDecapsulationKey, KeyExport, MlKem1024, Seed,
};
use zeroize::{Zeroize, Zeroizing};

/// Owned secret bytes, zeroized on drop and redacted in Debug.
/// Explicit byte access is for the caller's cryptographic/storage boundary.
pub struct KemSecret(Zeroizing<Vec<u8>>);

impl KemSecret {
    /// Borrow secret material. Do not log it or store it unencrypted.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl std::fmt::Debug for KemSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KemSecret(<redacted>)")
    }
}

pub(crate) fn public_key(bytes: &[u8]) -> Result<EncapsulationKey1024> {
    let encoded = bytes
        .try_into()
        .map_err(|_| Error::invalid("kem_public_key", "expected 1568 bytes"))?;
    EncapsulationKey1024::new(&encoded)
        .map_err(|_| Error::invalid("kem_public_key", "invalid ML-KEM-1024 public key"))
}

#[allow(deprecated)] // APK stores the expanded 3168-byte key, not the 64-byte seed.
fn from_seed(mut seed: Seed) -> (Vec<u8>, KemSecret) {
    use ml_kem::ExpandedKeyEncoding;
    let key = DecapsulationKey1024::from_seed(seed);
    seed.zeroize();
    let secret = Zeroizing::new(key.to_expanded_bytes());
    (
        key.encapsulation_key().to_bytes().to_vec(),
        KemSecret(Zeroizing::new(secret.to_vec())),
    )
}

/// Generate a fresh ML-KEM-1024 public key and expanded secret key using OS entropy.
/// This does not register a prekey, allocate an ID, or create a Signal session.
pub fn generate_key_pair() -> Result<(Vec<u8>, KemSecret)> {
    let mut seed = Zeroizing::new(Seed::default());
    getrandom::fill(seed.as_mut_slice())
        .map_err(|_| Error::invalid("kem_entropy", "entropy unavailable"))?;
    Ok(from_seed(*seed))
}

/// Encapsulate to a validated ML-KEM-1024 public key using the crypto RNG.
/// Returns ciphertext and shared secret; no signature/trust check is implied.
pub fn encapsulate(public: &[u8]) -> Result<(Vec<u8>, KemSecret)> {
    let public = public_key(public)?;
    let (ciphertext, mut shared) = public.encapsulate_with_rng(&mut rand::rng());
    let secret = KemSecret(Zeroizing::new(shared.to_vec()));
    shared.zeroize();
    Ok((ciphertext.to_vec(), secret))
}

/// Decapsulate with a validated expanded secret key and a 1568-byte ciphertext.
/// Invalid ciphertexts use ML-KEM implicit rejection; success is not authentication.
#[allow(deprecated)] // Native expanded-key encoding is required at this boundary.
pub fn decapsulate(secret: &[u8], ciphertext: &[u8]) -> Result<KemSecret> {
    let secret = Zeroizing::new(
        ExpandedDecapsulationKey::<MlKem1024>::try_from(secret)
            .map_err(|_| Error::invalid("kem_secret_key", "expected 3168 bytes"))?,
    );
    let key = DecapsulationKey1024::from_expanded(&secret)
        .map_err(|_| Error::invalid("kem_secret_key", "invalid ML-KEM-1024 secret key"))?;
    let ciphertext = Ciphertext::<MlKem1024>::try_from(ciphertext)
        .map_err(|_| Error::invalid("kem_ciphertext", "expected 1568 bytes"))?;
    let mut shared = key.decapsulate(&ciphertext);
    let result = KemSecret(Zeroizing::new(shared.to_vec()));
    shared.zeroize();
    Ok(result)
}

/// Derive the initial hybrid root and chain keys from already-computed agreements.
/// Order: identity/signed, base/identity, base/signed, optional base/one-time,
/// then KEM shared secret. This does not verify identities or create/adopt a session.
pub fn initial_hybrid_keys(
    identity_signed: &[u8; 32],
    base_identity: &[u8; 32],
    base_signed: &[u8; 32],
    base_one_time: Option<&[u8; 32]>,
    kem_shared: &[u8; 32],
) -> (KemSecret, KemSecret) {
    let mut material = Zeroizing::new([0xff; 192]);
    let mut length = 32;
    for agreement in [identity_signed, base_identity, base_signed]
        .into_iter()
        .chain(base_one_time)
        .chain(std::iter::once(kem_shared))
    {
        material[length..length + 32].copy_from_slice(agreement);
        length += 32;
    }
    let mut output = Zeroizing::new([0u8; 64]);
    hkdf::Hkdf::<sha2::Sha256>::new(None, &material[..length])
        .expand(
            b"WhisperText_X25519_SHA-256_CRYSTALS-KYBER-1024",
            output.as_mut(),
        )
        .expect("fixed 64-byte HKDF output");
    (
        KemSecret(Zeroizing::new(output[..32].to_vec())),
        KemSecret(Zeroizing::new(output[32..].to_vec())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ml_kem::Seed;

    #[test]
    fn hybrid_initial_keys_match_independent_hmac_vectors() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/hybrid-kdf-vectors.json"
        ))
        .unwrap();
        let value = |v: &serde_json::Value| -> [u8; 32] {
            hex::decode(v.as_str().unwrap())
                .unwrap()
                .try_into()
                .unwrap()
        };
        let cases = fixture["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 16);
        for case in cases {
            let agreements = [
                &case["agreements"][0],
                &case["agreements"][1],
                &case["agreements"][2],
            ]
            .map(value);
            let one_time = (!case["one_time"].is_null()).then(|| value(&case["one_time"]));
            let kem = value(&case["kem_shared"]);
            let (root, chain) = initial_hybrid_keys(
                &agreements[0],
                &agreements[1],
                &agreements[2],
                one_time.as_ref(),
                &kem,
            );
            assert_eq!(root.as_bytes(), value(&case["root"]));
            assert_eq!(chain.as_bytes(), value(&case["chain"]));
        }
    }

    #[test]
    fn agrees_with_native_and_nist_vectors() {
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../docs/protocol/native-kem-nist-vectors.json"
        ))
        .unwrap();
        let mut count = 0;
        for case in fixtures["cases"].as_array().unwrap() {
            let input = &case["input"];
            let expected = &case["expected"];
            let bytes = |value: &serde_json::Value| hex::decode(value.as_str().unwrap()).unwrap();
            match case["operation"].as_str().unwrap() {
                "keyGen" => {
                    let mut seed = bytes(&input["d"]);
                    seed.extend(bytes(&input["z"]));
                    let (public, secret) = from_seed(Seed::try_from(seed.as_slice()).unwrap());
                    assert_eq!(public, bytes(&expected["ek"]));
                    assert_eq!(secret.as_bytes(), bytes(&expected["dk"]));
                }
                "encapsulation" => {
                    let public = public_key(&bytes(&input["ek"])).unwrap();
                    let m = ml_kem::B32::try_from(bytes(&input["m"]).as_slice()).unwrap();
                    let (ciphertext, secret) = public.encapsulate_deterministic(&m);
                    assert_eq!(ciphertext.as_slice(), bytes(&expected["c"]));
                    assert_eq!(secret.as_slice(), bytes(&expected["k"]));
                }
                "decapsulation" => {
                    let secret = decapsulate(&bytes(&input["dk"]), &bytes(&input["c"])).unwrap();
                    assert_eq!(secret.as_bytes(), bytes(&expected["k"]));
                }
                _ => panic!("unexpected fixture operation"),
            }
            count += 1;
        }
        assert_eq!(count, 16);
    }

    #[test]
    fn round_trip_and_invalid_inputs_are_secret_free() {
        let (public, secret) = generate_key_pair().unwrap();
        let (ciphertext, shared) = encapsulate(&public).unwrap();
        assert_eq!(
            shared.as_bytes(),
            decapsulate(secret.as_bytes(), &ciphertext)
                .unwrap()
                .as_bytes()
        );
        assert_eq!(format!("{secret:?}"), "KemSecret(<redacted>)");
        assert_eq!(format!("{shared:?}"), "KemSecret(<redacted>)");
        for invalid in [&[][..], &[0; 800][..], &[0xff; 1568][..]] {
            assert!(encapsulate(invalid).is_err());
        }
        assert!(decapsulate(&[], &ciphertext).is_err());
        assert!(decapsulate(secret.as_bytes(), &[]).is_err());
        let mut damaged = secret.as_bytes().to_vec();
        damaged[3104] ^= 1; // expanded public-key hash, not ciphertext implicit rejection
        assert!(decapsulate(&damaged, &ciphertext).is_err());
        let error = encapsulate(b"PRIVATE-CANARY").unwrap_err().to_string();
        assert!(!error.contains("PRIVATE"));
    }
}
